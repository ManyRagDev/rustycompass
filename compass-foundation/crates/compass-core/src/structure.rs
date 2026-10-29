//! A conservative, revision-bound factual graph. Parsing never executes project code.
//!
//! Resolved means a unique syntactic declaration in this corpus, not a compiler/type
//! checker proof. Dynamic dispatch, macro expansion and ambiguous bindings remain
//! explicit edges without a target. Module lookup uses paths only (no config loading).
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use tree_sitter::{Node as Syntax, Parser};

#[derive(Clone, Debug)]
pub struct Document {
    pub path_key: String,
    pub path: String,
    pub content_sha256: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Resolution {
    Resolved,
    Partial,
    Unavailable,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Relation {
    Contains,
    Imports,
    Calls,
    References,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Evidence {
    pub path_key: String,
    pub content_sha256: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub path_key: String,
    pub path: String,
    pub content_sha256: String,
    pub kind: String,
    pub name: String,
    /// Declaration scopes, from the outermost declaration to the immediate parent.
    pub scope: Vec<String>,
    pub parent_id: Option<String>,
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Edge {
    pub id: String,
    pub source: String,
    pub target: Option<String>,
    pub kind: Relation,
    /// The literal source name, retained even when the target is unavailable.
    pub name: String,
    pub resolution: Resolution,
    pub evidence: Evidence,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Coverage {
    pub path_key: String,
    pub content_sha256: String,
    pub language: Option<String>,
    pub status: Resolution,
    pub parse_errors: usize,
    pub symbols: usize,
    pub resolved_relations: usize,
    pub unresolved_relations: usize,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Graph {
    pub schema_version: String,
    pub generation: String,
    pub producer: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub coverage: Vec<Coverage>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Neighborhood {
    pub generation: String,
    pub root: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub truncated: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Language {
    Python,
    Rust,
    JavaScript,
    TypeScript,
    Tsx,
}

impl Language {
    fn from_path(path: &str) -> Option<Self> {
        match path.rsplit('.').next()? {
            "py" | "pyi" => Some(Self::Python),
            "rs" => Some(Self::Rust),
            "js" | "jsx" | "mjs" | "cjs" => Some(Self::JavaScript),
            "ts" | "mts" | "cts" => Some(Self::TypeScript),
            "tsx" => Some(Self::Tsx),
            _ => None,
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Python => "python",
            Self::Rust => "rust",
            Self::JavaScript => "javascript",
            Self::TypeScript => "typescript",
            Self::Tsx => "tsx",
        }
    }
    fn grammar(self) -> tree_sitter::Language {
        match self {
            Self::Python => tree_sitter_python::LANGUAGE.into(),
            Self::Rust => tree_sitter_rust::LANGUAGE.into(),
            Self::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Self::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Self::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
        }
    }
}

#[derive(Clone)]
struct Symbol {
    index: usize,
    name_start: usize,
    name_end: usize,
    // Bindings are visible within this lexical interval. Declaration nodes retain
    // their own (narrower) span, which callers can read without scanning a scope.
    visible_start: usize,
    visible_end: usize,
    exported: bool,
    safe: bool,
}

#[derive(Clone)]
struct Import {
    module: String,
    imported: Option<String>,
    local: Option<String>,
    namespace: bool,
    evidence: Evidence,
    source: String,
    file: usize,
    candidates: Vec<usize>,
    unsupported: bool,
    visible_start: usize,
    visible_end: usize,
}

#[derive(Clone)]
struct Use {
    file: usize,
    source: String,
    name: String,
    parts: Vec<String>,
    kind: Relation,
    evidence: Evidence,
    dynamic: bool,
}

struct Parsed {
    language: Option<Language>,
    file_id: String,
    symbols: Vec<Symbol>,
    imports: Vec<Import>,
    uses: Vec<Use>,
    errors: usize,
    unavailable: Option<String>,
    limitation: Option<String>,
    visited: usize,
}

fn check(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Relaxed) {
        Err(Error::cancelled())
    } else {
        Ok(())
    }
}

fn hash(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}

fn identity(parts: &[&str]) -> String {
    // Length-prefixing makes identities unambiguous even for unusual native paths.
    let mut h = Sha256::new();
    for part in parts {
        h.update((part.len() as u64).to_le_bytes());
        h.update(part.as_bytes());
    }
    format!("{:x}", h.finalize())
}

fn text<'a>(node: Syntax<'_>, bytes: &'a [u8]) -> &'a str {
    std::str::from_utf8(&bytes[node.start_byte()..node.end_byte()]).unwrap_or("")
}

fn evidence(node: Syntax<'_>, doc: &Document) -> Evidence {
    Evidence {
        path_key: doc.path_key.clone(),
        content_sha256: doc.content_sha256.clone(),
        start_byte: node.start_byte(),
        end_byte: node.end_byte(),
        start_line: node.start_position().row + 1,
        end_line: node.end_position().row + 1,
    }
}

fn make_edge(
    source: String,
    target: Option<String>,
    kind: Relation,
    name: String,
    ev: Evidence,
    resolution: Resolution,
    reason: Option<String>,
) -> Edge {
    let id = identity(&[
        "edge-v1",
        &source,
        target.as_deref().unwrap_or(""),
        &format!("{kind:?}"),
        &name,
        &ev.path_key,
        &ev.content_sha256,
        &ev.start_byte.to_string(),
        &ev.end_byte.to_string(),
    ]);
    Edge {
        id,
        source,
        target,
        kind,
        name,
        resolution,
        evidence: ev,
        reason,
    }
}

fn declaration(node: Syntax<'_>, language: Language) -> Option<(&'static str, Syntax<'_>)> {
    let kind = match node.kind() {
        "function_definition"
        | "function_declaration"
        | "generator_function_declaration"
        | "function_item"
        | "function_expression" => "function",
        "function_signature_item" | "function_signature" => "signature",
        "method_definition" | "method_signature" => "method",
        "class_definition" | "class_declaration" | "abstract_class_declaration" => "class",
        "struct_item" | "enum_item" | "union_item" | "enum_declaration" => "type",
        "trait_item" | "interface_declaration" | "type_alias_declaration" | "type_item" => "type",
        "mod_item" | "internal_module" => "module",
        "impl_item" => "impl",
        "const_item" | "static_item" | "variable_declarator" => "variable",
        "assignment_expression"
        | "augmented_assignment_expression"
        | "augmented_assignment"
        | "for_statement"
        | "for_in_statement" => "variable",
        "let_declaration" if language == Language::Rust => "variable",
        "assignment" if language == Language::Python => "variable",
        _ => return None,
    };
    let name = node
        .child_by_field_name("name")
        .or_else(|| node.child_by_field_name("pattern"))
        .or_else(|| node.child_by_field_name("left"))
        .or_else(|| {
            (node.kind() == "impl_item")
                .then(|| node.child_by_field_name("type"))
                .flatten()
        })?;
    if matches!(
        name.kind(),
        "identifier" | "type_identifier" | "property_identifier"
    ) {
        Some((kind, name))
    } else {
        None
    }
}

fn binding_scope(mut node: Syntax<'_>, language: Language) -> (usize, usize) {
    let function_scoped = node
        .parent()
        .is_some_and(|p| p.kind() == "variable_declaration");
    while let Some(parent) = node.parent() {
        let k = parent.kind();
        if matches!(
            k,
            "function_definition"
                | "function_declaration"
                | "generator_function_declaration"
                | "function_item"
                | "method_definition"
                | "arrow_function"
                | "function_expression"
                | "class_definition"
                | "class_declaration"
                | "mod_item"
                | "impl_item"
                | "trait_item"
                | "internal_module"
                | "module"
                | "program"
                | "source_file"
        ) || (language != Language::Python
            && !function_scoped
            && matches!(k, "block" | "statement_block"))
        {
            return (parent.start_byte(), parent.end_byte());
        }
        node = parent;
    }
    (node.start_byte(), node.end_byte())
}

#[derive(Clone, Copy)]
struct LexicalScope<'a> {
    parent: &'a str,
    names: &'a [String],
}
#[derive(Clone, Copy)]
struct Input<'a> {
    doc: &'a Document,
    file: usize,
}

fn add_symbol(
    syntax: Syntax<'_>,
    name_node: Syntax<'_>,
    kind: &str,
    doc: &Document,
    parsed: &mut Parsed,
    nodes: &mut Vec<Node>,
    lexical: LexicalScope<'_>,
) -> String {
    let LexicalScope {
        parent,
        names: scope,
    } = lexical;
    let name = text(name_node, &doc.bytes).to_owned();
    let ev = evidence(syntax, doc);
    let id = identity(&[
        "symbol-v1",
        &doc.path_key,
        &doc.content_sha256,
        kind,
        &name,
        &ev.start_byte.to_string(),
        &ev.end_byte.to_string(),
    ]);
    let (visible_start, visible_end) = binding_scope(syntax, parsed.language.unwrap());
    let exported = syntax
        .parent()
        .is_some_and(|p| p.kind() == "export_statement");
    let safe = !syntax.parent().is_some_and(|p| {
        p.kind() == "decorated_definition"
            || p.parent()
                .is_some_and(|p| matches!(p.kind(), "impl_item" | "trait_item"))
    });
    parsed.symbols.push(Symbol {
        index: nodes.len(),
        name_start: name_node.start_byte(),
        name_end: name_node.end_byte(),
        visible_start,
        visible_end,
        exported,
        safe,
    });
    nodes.push(Node {
        id: id.clone(),
        path_key: doc.path_key.clone(),
        path: doc.path.clone(),
        content_sha256: doc.content_sha256.clone(),
        kind: kind.to_owned(),
        name,
        scope: scope.to_vec(),
        parent_id: Some(parent.to_owned()),
        start_byte: ev.start_byte,
        end_byte: ev.end_byte,
        start_line: ev.start_line,
        end_line: ev.end_line,
    });
    id
}

fn is_parameter(node: Syntax<'_>) -> bool {
    if node.kind() != "identifier" {
        return false;
    }
    let mut current = node;
    while let Some(parent) = current.parent() {
        match parent.kind() {
            "parameters" | "formal_parameters" => return true,
            "arrow_function" => return parent.child_by_field_name("parameter") == Some(node),
            "typed_parameter"
            | "default_parameter"
            | "typed_default_parameter"
            | "required_parameter"
            | "optional_parameter"
            | "parameter" => {
                // Identifiers in annotations/default expressions are references.
                let bound = parent
                    .child_by_field_name("pattern")
                    .or_else(|| parent.child_by_field_name("name"))
                    .or_else(|| parent.child_by_field_name("left"))
                    .or_else(|| {
                        (parent.kind() == "typed_parameter")
                            .then(|| parent.named_child(0))
                            .flatten()
                    });
                return bound.is_some_and(|b| {
                    b.start_byte() <= node.start_byte() && node.end_byte() <= b.end_byte()
                });
            }
            "identifier" | "list_splat_pattern" | "dictionary_splat_pattern" => {}
            _ => return false,
        }
        current = parent;
    }
    false
}

fn in_import(mut node: Syntax<'_>) -> bool {
    while let Some(parent) = node.parent() {
        if matches!(
            parent.kind(),
            "import_statement"
                | "import_from_statement"
                | "use_declaration"
                | "extern_crate_declaration"
        ) {
            return true;
        }
        node = parent;
    }
    false
}

fn access_parts(node: Syntax<'_>, bytes: &[u8]) -> Option<Vec<String>> {
    match node.kind() {
        "identifier" | "type_identifier" | "self" | "crate" | "super" => {
            Some(vec![text(node, bytes).to_owned()])
        }
        "attribute"
        | "member_expression"
        | "field_expression"
        | "scoped_identifier"
        | "scoped_type_identifier" => {
            let left = node
                .child_by_field_name("object")
                .or_else(|| node.child_by_field_name("value"))
                .or_else(|| node.child_by_field_name("path"))?;
            let right = node
                .child_by_field_name("attribute")
                .or_else(|| node.child_by_field_name("property"))
                .or_else(|| node.child_by_field_name("field"))
                .or_else(|| node.child_by_field_name("name"))?;
            if !matches!(
                right.kind(),
                "identifier" | "property_identifier" | "field_identifier" | "type_identifier"
            ) {
                return None;
            }
            let mut out = access_parts(left, bytes)?;
            out.push(text(right, bytes).to_owned());
            Some(out)
        }
        _ => None,
    }
}

fn walk(
    syntax: Syntax<'_>,
    input: Input<'_>,
    parsed: &mut Parsed,
    nodes: &mut Vec<Node>,
    lexical: LexicalScope<'_>,
    cancel: &AtomicBool,
    depth: usize,
) -> Result<()> {
    let Input { doc, file } = input;
    let LexicalScope {
        parent,
        names: scope,
    } = lexical;
    check(cancel)?;
    if depth >= 256 || parsed.visited >= 200_000 {
        parsed.limitation = Some("syntax_traversal_limit".into());
        return Ok(());
    }
    parsed.visited += 1;
    if syntax.is_error() || syntax.is_missing() {
        parsed.errors += 1;
    }
    let language = parsed.language.unwrap();
    let mut next_parent = parent.to_owned();
    let mut next_scope = scope.to_vec();
    if language == Language::Rust
        && matches!(syntax.kind(), "attribute_item" | "inner_attribute_item")
    {
        parsed.limitation = Some("rust_attributes_require_configuration_resolution".into());
    }
    // Conservative fallback when binding visibility requires unmodelled semantics.
    if matches!(
        syntax.kind(),
        "catch_clause"
            | "lambda"
            | "lambda_expression"
            | "list_comprehension"
            | "set_comprehension"
            | "dictionary_comprehension"
            | "generator_expression"
            | "global_statement"
            | "nonlocal_statement"
            | "with_statement"
            | "except_clause"
            | "match_statement"
            | "object_pattern"
            | "array_pattern"
            | "tuple_pattern"
            | "list_pattern"
            | "macro_definition"
    ) {
        parsed.limitation = Some("unmodelled_binding_or_dynamic_scope".into());
    }
    if matches!(
        syntax.kind(),
        "let_declaration"
            | "variable_declarator"
            | "assignment"
            | "assignment_expression"
            | "for_statement"
            | "for_in_statement"
    ) {
        let binding = syntax
            .child_by_field_name("pattern")
            .or_else(|| syntax.child_by_field_name("name"))
            .or_else(|| syntax.child_by_field_name("left"));
        if binding.is_some_and(|n| {
            !matches!(
                n.kind(),
                "identifier"
                    | "member_expression"
                    | "field_expression"
                    | "attribute"
                    | "subscript"
                    | "subscript_expression"
            )
        }) {
            parsed.limitation = Some("destructured_binding_not_resolved".into());
        }
    }
    if let Some((kind, name)) = declaration(syntax, language) {
        next_parent = add_symbol(syntax, name, kind, doc, parsed, nodes, lexical);
        next_scope.push(text(name, &doc.bytes).to_owned());
    } else if is_parameter(syntax) {
        add_symbol(syntax, syntax, "parameter", doc, parsed, nodes, lexical);
    }
    if matches!(
        syntax.kind(),
        "import_statement"
            | "import_from_statement"
            | "use_declaration"
            | "extern_crate_declaration"
    ) {
        parse_import(syntax, doc, file, parsed, parent);
        if parsed
            .imports
            .iter()
            .any(|i| i.unsupported && i.local.is_none())
        {
            parsed.limitation = Some("wildcard_import_not_resolved".into());
        }
        return Ok(());
    }
    let call = matches!(syntax.kind(), "call" | "call_expression" | "new_expression");
    let macro_call = syntax.kind() == "macro_invocation";
    if call || macro_call {
        let callee = syntax
            .child_by_field_name("function")
            .or_else(|| syntax.child_by_field_name("constructor"))
            .or_else(|| syntax.child_by_field_name("macro"));
        if let Some(callee) = callee {
            let parts = access_parts(callee, &doc.bytes);
            parsed.uses.push(Use {
                file,
                source: parent.to_owned(),
                name: text(callee, &doc.bytes).to_owned(),
                parts: parts.clone().unwrap_or_default(),
                kind: Relation::Calls,
                evidence: evidence(callee, doc),
                dynamic: macro_call || parts.is_none(),
            });
        }
    }
    if syntax.kind() == "identifier" && !in_import(syntax) {
        let is_decl = parsed
            .symbols
            .iter()
            .any(|s| s.name_start == syntax.start_byte() && s.name_end == syntax.end_byte());
        let field_name = syntax.parent().and_then(|p| {
            (0..p.child_count()).find_map(|i| {
                (p.child(i) == Some(syntax))
                    .then(|| p.field_name_for_child(i as u32))
                    .flatten()
            })
        });
        let is_property = matches!(field_name, Some("attribute" | "property" | "field"));
        if !is_decl && !is_property {
            parsed.uses.push(Use {
                file,
                source: parent.to_owned(),
                name: text(syntax, &doc.bytes).to_owned(),
                parts: vec![text(syntax, &doc.bytes).to_owned()],
                kind: Relation::References,
                evidence: evidence(syntax, doc),
                dynamic: false,
            });
        }
    }
    let mut cursor = syntax.walk();
    for child in syntax.named_children(&mut cursor) {
        walk(
            child,
            input,
            parsed,
            nodes,
            LexicalScope {
                parent: &next_parent,
                names: &next_scope,
            },
            cancel,
            depth + 1,
        )?;
    }
    Ok(())
}

fn descendants<'a>(node: Syntax<'a>, kind: &str) -> Vec<Syntax<'a>> {
    let mut found = Vec::new();
    let mut pending = vec![node];
    while let Some(n) = pending.pop() {
        if n.kind() == kind {
            found.push(n);
        }
        let mut cursor = n.walk();
        pending.extend(n.named_children(&mut cursor));
    }
    found.sort_by_key(Syntax::start_byte);
    found
}

fn parse_import(node: Syntax<'_>, doc: &Document, file: usize, parsed: &mut Parsed, source: &str) {
    let ev = evidence(node, doc);
    let language = parsed.language.unwrap();
    let (visible_start, visible_end) = binding_scope(node, language);
    let mut push = |module: String,
                    imported: Option<String>,
                    local: Option<String>,
                    namespace: bool,
                    unsupported: bool| {
        parsed.imports.push(Import {
            module,
            imported,
            local,
            namespace,
            unsupported,
            visible_start,
            visible_end,
            evidence: ev.clone(),
            source: source.to_owned(),
            file,
            candidates: Vec::new(),
        });
    };
    match language {
        Language::Python => {
            if node.kind() == "import_from_statement" {
                let module_node = node.child_by_field_name("module_name");
                let module = module_node
                    .map(|n| text(n, &doc.bytes).to_owned())
                    .unwrap_or_default();
                let mut cursor = node.walk();
                for child in node.named_children(&mut cursor) {
                    if Some(child) == module_node {
                        continue;
                    }
                    if child.kind() == "aliased_import" {
                        let name = child
                            .child_by_field_name("name")
                            .map(|n| text(n, &doc.bytes))
                            .unwrap_or("");
                        let alias = child
                            .child_by_field_name("alias")
                            .map(|n| text(n, &doc.bytes))
                            .unwrap_or(name);
                        push(
                            module.clone(),
                            Some(name.into()),
                            Some(alias.into()),
                            false,
                            false,
                        );
                    } else if child.kind() == "dotted_name" {
                        let name = text(child, &doc.bytes).to_owned();
                        push(module.clone(), Some(name.clone()), Some(name), false, false);
                    } else if child.kind() == "wildcard_import" {
                        push(module.clone(), None, None, false, true);
                    }
                }
            } else {
                let mut cursor = node.walk();
                for child in node.named_children(&mut cursor) {
                    if child.kind() == "aliased_import" {
                        let module = child
                            .child_by_field_name("name")
                            .map(|n| text(n, &doc.bytes))
                            .unwrap_or("");
                        let alias = child
                            .child_by_field_name("alias")
                            .map(|n| text(n, &doc.bytes))
                            .unwrap_or(module);
                        push(module.into(), None, Some(alias.into()), true, false);
                    } else if child.kind() == "dotted_name" {
                        let module = text(child, &doc.bytes);
                        // `import a.b` binds `a`; retaining the full name avoids pretending
                        // that a.b declarations are members of a.
                        push(
                            module.into(),
                            None,
                            Some(module.split('.').next().unwrap_or(module).into()),
                            true,
                            module.contains('.'),
                        );
                    }
                }
            }
        }
        Language::Rust => {
            let value = text(node, &doc.bytes)
                .trim()
                .trim_start_matches("pub ")
                .trim_start_matches("use ")
                .trim_end_matches(';')
                .trim();
            let (path, alias) = value
                .split_once(" as ")
                .map_or((value, None), |(p, a)| (p, Some(a)));
            let unsupported =
                path.contains(['{', '}', '*', '!']) || node.kind() == "extern_crate_declaration";
            let parts: Vec<_> = path.split("::").collect();
            let imported = parts.last().map(|s| s.to_string());
            let module = parts[..parts.len().saturating_sub(1)].join("::");
            let local = alias.map(str::to_owned).or_else(|| imported.clone());
            push(module, imported, local, false, unsupported);
        }
        _ => {
            let module = node
                .child_by_field_name("source")
                .map(|n| text(n, &doc.bytes).trim_matches(['\'', '"']).to_owned())
                .unwrap_or_default();
            let specifiers = descendants(node, "import_specifier");
            for specifier in specifiers {
                let name = specifier
                    .child_by_field_name("name")
                    .map(|n| text(n, &doc.bytes))
                    .unwrap_or("");
                let alias = specifier
                    .child_by_field_name("alias")
                    .map(|n| text(n, &doc.bytes))
                    .unwrap_or(name);
                push(
                    module.clone(),
                    Some(name.into()),
                    Some(alias.into()),
                    false,
                    false,
                );
            }
            for namespace in descendants(node, "namespace_import") {
                let local = descendants(namespace, "identifier")
                    .first()
                    .map(|n| text(*n, &doc.bytes).to_owned());
                push(module.clone(), None, local, true, false);
            }
            let mut has_default = false;
            for clause in descendants(node, "import_clause") {
                let mut cursor = clause.walk();
                for child in clause.named_children(&mut cursor) {
                    if child.kind() == "identifier" {
                        has_default = true;
                        push(
                            module.clone(),
                            Some("default".into()),
                            Some(text(child, &doc.bytes).into()),
                            false,
                            true,
                        );
                    }
                }
            }
            if !has_default
                && descendants(node, "import_specifier").is_empty()
                && descendants(node, "namespace_import").is_empty()
            {
                push(module, None, None, false, false);
            }
        }
    }
}

fn normalize(path: &str) -> String {
    let mut out = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                out.pop();
            }
            _ => out.push(part),
        }
    }
    out.join("/")
}

fn directory(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(d, _)| d)
}

fn module_candidates(import: &Import, docs: &[&Document], parsed: &[Parsed]) -> Vec<usize> {
    if import.unsupported {
        return Vec::new();
    }
    let language = parsed[import.file].language.unwrap();
    let source = normalize(&docs[import.file].path);
    let dir = directory(&source);
    let module = &import.module;
    let mut names = BTreeSet::new();
    match language {
        Language::JavaScript | Language::TypeScript | Language::Tsx => {
            if !module.starts_with('.') {
                return Vec::new();
            }
            let mut relative: Vec<_> = dir.split('/').filter(|s| !s.is_empty()).collect();
            for part in module.split('/') {
                match part {
                    "" | "." => {}
                    ".." => {
                        if relative.pop().is_none() {
                            return Vec::new();
                        }
                    }
                    part => relative.push(part),
                }
            }
            let base = relative.join("/");
            names.insert(base.clone());
            for extension in ["js", "jsx", "mjs", "cjs", "ts", "tsx", "mts", "cts"] {
                names.insert(format!("{base}.{extension}"));
                names.insert(format!("{base}/index.{extension}"));
            }
            // TypeScript source imports often retain the emitted .js suffix.
            if let Some(stem) = base.strip_suffix(".js") {
                names.insert(format!("{stem}.ts"));
                names.insert(format!("{stem}.tsx"));
            }
        }
        Language::Python => {
            let dots = module.chars().take_while(|c| *c == '.').count();
            let tail = module[dots..].replace('.', "/");
            if dots > dir.split('/').filter(|s| !s.is_empty()).count() {
                return Vec::new();
            }
            let mut base_dir = dir.to_owned();
            if dots > 0 {
                for _ in 1..dots {
                    base_dir = directory(&base_dir).to_owned();
                }
            } else {
                // Absolute imports are anchored to the declared corpus root.
                // A sibling of a package module is not implicitly on sys.path.
                base_dir.clear();
            }
            let base = normalize(&format!("{base_dir}/{tail}"));
            names.insert(format!("{base}.py"));
            names.insert(format!("{base}/__init__.py"));
            if tail.is_empty() {
                names.insert(format!("{base}/__init__.py"));
                if let Some(name) = &import.imported {
                    names.insert(normalize(&format!("{base}/{name}.py")));
                    names.insert(normalize(&format!("{base}/{name}/__init__.py")));
                }
            }
        }
        Language::Rust => {
            let parts: Vec<_> = module.split("::").filter(|s| !s.is_empty()).collect();
            if !matches!(parts.first(), Some(&"crate" | &"self" | &"super")) {
                return Vec::new();
            }
            // Conventional module layouts only. Cargo may declare custom crate
            // roots, and each src/bin target has a different crate namespace.
            let (root, relative) = if let Some((prefix, relative)) = source.rsplit_once("/src/") {
                (format!("{prefix}/src"), relative)
            } else if let Some(relative) = source.strip_prefix("src/") {
                ("src".to_owned(), relative)
            } else if matches!(source.as_str(), "lib.rs" | "main.rs") {
                (String::new(), source.as_str())
            } else {
                return Vec::new();
            };
            if relative.starts_with("bin/") {
                return Vec::new();
            }
            let roots: Vec<_> = docs
                .iter()
                .enumerate()
                .filter_map(|(i, d)| {
                    ["lib.rs", "main.rs"]
                        .iter()
                        .any(|name| normalize(&format!("{root}/{name}")) == normalize(&d.path))
                        .then_some(i)
                })
                .collect();
            if roots.is_empty()
                || roots
                    .iter()
                    .any(|i| parsed[*i].errors > 0 || parsed[*i].limitation.is_some())
            {
                return Vec::new();
            }
            let mut logical: Vec<String> = relative
                .trim_end_matches(".rs")
                .split('/')
                .map(str::to_owned)
                .collect();
            if logical
                .last()
                .is_some_and(|s| matches!(s.as_str(), "lib" | "main" | "mod"))
            {
                logical.pop();
            }
            if parts.first() == Some(&"crate") {
                logical.clear();
            }
            for (i, part) in parts.iter().enumerate() {
                match *part {
                    "crate" | "self" if i == 0 => {}
                    "super" => {
                        if logical.pop().is_none() {
                            return Vec::new();
                        }
                    }
                    "crate" | "self" => return Vec::new(),
                    part => logical.push(part.into()),
                }
            }
            if logical.is_empty() {
                for name in ["lib.rs", "main.rs"] {
                    names.insert(normalize(&format!("{root}/{name}")));
                }
            } else {
                let base = normalize(&format!("{root}/{}", logical.join("/")));
                names.insert(format!("{base}.rs"));
                names.insert(format!("{base}/mod.rs"));
            }
        }
    }
    docs.iter()
        .enumerate()
        .filter_map(|(i, doc)| {
            (names.contains(&normalize(&doc.path))
                && parsed[i].language.is_some()
                && parsed[i].unavailable.is_none())
            .then_some(i)
        })
        .collect()
}

fn file_symbols<'a>(
    file: usize,
    name: &str,
    parsed: &'a [Parsed],
    nodes: &'a [Node],
) -> Vec<&'a Node> {
    parsed[file]
        .symbols
        .iter()
        .map(|s| &nodes[s.index])
        .filter(|n| n.name == name && n.parent_id.as_deref() == Some(&parsed[file].file_id))
        .collect()
}

fn resolve_use(
    usage: &Use,
    parsed: &[Parsed],
    nodes: &[Node],
) -> (Option<String>, Resolution, Option<String>) {
    let unresolved = |reason: &str, partial: bool| {
        (
            None,
            if partial {
                Resolution::Partial
            } else {
                Resolution::Unavailable
            },
            Some(reason.to_owned()),
        )
    };
    let unit = &parsed[usage.file];
    if usage.dynamic {
        return unresolved("dynamic_call_or_macro_expansion", false);
    }
    if unit.errors > 0 || unit.limitation.is_some() {
        return unresolved("source_structure_partial", true);
    }
    let Some(first) = usage.parts.first() else {
        return unresolved("dynamic_expression", false);
    };
    let position = usage.evidence.start_byte;
    let mut local: Vec<_> = unit
        .symbols
        .iter()
        .filter(|s| {
            nodes[s.index].name == *first && s.visible_start <= position && position < s.visible_end
        })
        .collect();
    // Narrower lexical scopes shadow wider ones. Multiple declarations in the
    // same scope are ambiguous even if one precedes the use (flow is not inferred).
    let mut matching: Vec<_> = unit
        .imports
        .iter()
        .filter(|i| {
            i.local.as_deref() == Some(first)
                && i.visible_start <= position
                && position < i.visible_end
        })
        .collect();
    let local_width = local.iter().map(|s| s.visible_end - s.visible_start).min();
    let import_width = matching
        .iter()
        .map(|i| i.visible_end - i.visible_start)
        .min();
    let width = local_width.into_iter().chain(import_width).min();
    local.retain(|s| Some(s.visible_end - s.visible_start) == width);
    matching.retain(|i| Some(i.visible_end - i.visible_start) == width);
    if !local.is_empty() && !matching.is_empty() {
        return unresolved("ambiguous_local_and_import_binding", true);
    }
    if !local.is_empty() {
        if local.len() != 1 {
            return unresolved("ambiguous_local_binding", true);
        }
        let target = &nodes[local[0].index];
        if !local[0].safe {
            return unresolved("decorated_or_non_lexical_declaration", false);
        }
        if unit.language == Some(Language::Python)
            && target.parent_id.as_deref() != Some(&usage.source)
            && target
                .parent_id
                .as_deref()
                .is_some_and(|p| nodes.iter().any(|n| n.id == p && n.kind == "class"))
        {
            return unresolved("class_namespace_not_a_function_lexical_scope", false);
        }
        if usage.parts.len() != 1 {
            return unresolved("member_dispatch_requires_type_resolution", false);
        }
        if usage.kind == Relation::Calls && !matches!(target.kind.as_str(), "function" | "class") {
            return unresolved("callable_value_requires_flow_resolution", false);
        }
        return (Some(target.id.clone()), Resolution::Resolved, None);
    }

    if matching.len() > 1 {
        return unresolved("ambiguous_import_binding", true);
    }
    let Some(import) = matching.first() else {
        return unresolved("declaration_not_found_in_static_scope", false);
    };
    if import.unsupported {
        return unresolved("unsupported_import_form", false);
    }
    if import.candidates.len() != 1 {
        return unresolved(
            if import.candidates.is_empty() {
                "external_or_unavailable_module"
            } else {
                "ambiguous_module_path"
            },
            import.candidates.len() > 1,
        );
    }
    let target_file = import.candidates[0];
    if parsed[target_file].errors > 0
        || parsed[target_file].limitation.is_some()
        || parsed[target_file].unavailable.is_some()
    {
        return unresolved("target_structure_partial", true);
    }
    let target_name = if import.namespace {
        if usage.parts.len() != 2 {
            return unresolved("namespace_member_not_statically_known", false);
        }
        &usage.parts[1]
    } else {
        if usage.parts.len() != 1 {
            return unresolved("member_dispatch_requires_type_resolution", false);
        }
        let Some(name) = &import.imported else {
            return unresolved("import_has_no_static_binding", false);
        };
        name
    };
    let targets = file_symbols(target_file, target_name, parsed, nodes);
    if targets.len() != 1 {
        return unresolved(
            if targets.len() > 1 {
                "ambiguous_target_declaration"
            } else {
                "target_declaration_unavailable"
            },
            targets.len() > 1,
        );
    }
    let target = targets[0];
    let symbol = parsed[target_file]
        .symbols
        .iter()
        .find(|s| nodes[s.index].id == target.id)
        .unwrap();
    if !symbol.safe {
        return unresolved("decorated_or_non_lexical_declaration", false);
    }
    if matches!(
        parsed[target_file].language,
        Some(Language::JavaScript | Language::TypeScript | Language::Tsx)
    ) && !symbol.exported
    {
        return unresolved("target_not_a_direct_named_export", false);
    }
    if usage.kind == Relation::Calls && !matches!(target.kind.as_str(), "function" | "class") {
        return unresolved("callable_value_requires_flow_resolution", false);
    }
    (Some(target.id.clone()), Resolution::Resolved, None)
}

/// Extract a complete graph from verified in-memory revisions. No filesystem,
/// package manager, compiler, source configuration or project code is consulted.
pub fn build(documents: &[Document], generation: String, cancel: &AtomicBool) -> Result<Graph> {
    check(cancel)?;
    let mut keys = BTreeSet::new();
    let mut docs: Vec<_> = documents.iter().collect();
    docs.sort_by(|a, b| a.path_key.cmp(&b.path_key));
    for doc in &docs {
        check(cancel)?;
        if !keys.insert(&doc.path_key) {
            return Err(Error::invalid("duplicate structural document path_key"));
        }
        if hash(&doc.bytes) != doc.content_sha256 {
            return Err(Error::invalid("structural document content hash mismatch"));
        }
    }
    let mut nodes = Vec::new();
    let mut parsed = Vec::new();
    for (file, doc) in docs.iter().enumerate() {
        check(cancel)?;
        let file_id = identity(&["file-v1", &doc.path_key, &doc.content_sha256]);
        nodes.push(Node {
            id: file_id.clone(),
            path_key: doc.path_key.clone(),
            path: doc.path.clone(),
            content_sha256: doc.content_sha256.clone(),
            kind: "file".into(),
            name: doc.path.clone(),
            scope: Vec::new(),
            parent_id: None,
            start_byte: 0,
            end_byte: doc.bytes.len(),
            start_line: 1,
            end_line: doc.bytes.iter().filter(|b| **b == b'\n').count() + 1,
        });
        let mut unit = Parsed {
            language: Language::from_path(&doc.path),
            file_id: file_id.clone(),
            symbols: Vec::new(),
            imports: Vec::new(),
            uses: Vec::new(),
            errors: 0,
            unavailable: None,
            limitation: None,
            visited: 0,
        };
        // Display paths may be lossy, but module lookup must use an exact native
        // identity. Distinct byte paths can otherwise collapse onto U+FFFD and
        // incorrectly become a unique import target after one file disappears.
        unit.unavailable = match crate::model::decode_path_key(&doc.path_key) {
            Ok(native) => match std::str::from_utf8(&native) {
                Ok(path) if path == doc.path => None,
                Ok(_) => Some("path_identity_mismatch".into()),
                Err(_) => Some("native_path_not_utf8".into()),
            },
            Err(_) => Some("path_identity_mismatch".into()),
        };
        if unit.unavailable.is_some() {
            parsed.push(unit);
            continue;
        }
        if let Some(language) = unit.language {
            if doc.bytes.len() > 8 * 1024 * 1024 {
                unit.unavailable = Some("source_exceeds_8_mib_parser_limit".into());
            } else if std::str::from_utf8(&doc.bytes).is_err() {
                unit.unavailable = Some("source_is_not_utf8".into());
            } else {
                let mut parser = Parser::new();
                parser.set_language(&language.grammar()).map_err(|e| {
                    Error::invalid(format!("Tree-sitter language unavailable: {e}"))
                })?;
                let mut progress = |_: &tree_sitter::ParseState| cancel.load(Ordering::Relaxed);
                let options = tree_sitter::ParseOptions::new().progress_callback(&mut progress);
                let tree = parser.parse_with_options(
                    &mut |offset, _| &doc.bytes[offset..],
                    None,
                    Some(options),
                );
                check(cancel)?;
                let tree =
                    tree.ok_or_else(|| Error::invalid("Tree-sitter did not return a tree"))?;
                walk(
                    tree.root_node(),
                    Input { doc, file },
                    &mut unit,
                    &mut nodes,
                    LexicalScope {
                        parent: &file_id,
                        names: &[],
                    },
                    cancel,
                    0,
                )?;
                // Missing anonymous delimiters are not visited by named_children.
                if tree.root_node().has_error() {
                    unit.errors = unit.errors.max(1);
                }
            }
        } else {
            unit.unavailable = Some("unsupported_language".into());
        }
        parsed.push(unit);
    }
    for file in 0..parsed.len() {
        let candidates: Vec<_> = parsed[file]
            .imports
            .iter()
            .map(|i| module_candidates(i, &docs, &parsed))
            .collect();
        for (import, candidates) in parsed[file].imports.iter_mut().zip(candidates) {
            import.candidates = candidates;
        }
    }
    let mut edges = Vec::new();
    for node in &nodes {
        check(cancel)?;
        if let Some(parent) = &node.parent_id {
            edges.push(make_edge(
                parent.clone(),
                Some(node.id.clone()),
                Relation::Contains,
                node.name.clone(),
                Evidence {
                    path_key: node.path_key.clone(),
                    content_sha256: node.content_sha256.clone(),
                    start_byte: node.start_byte,
                    end_byte: node.end_byte,
                    start_line: node.start_line,
                    end_line: node.end_line,
                },
                Resolution::Resolved,
                None,
            ));
        }
    }
    for unit in &parsed {
        for import in &unit.imports {
            check(cancel)?;
            let (target, status, reason) = if unit.errors > 0 || unit.limitation.is_some() {
                (
                    None,
                    Resolution::Partial,
                    Some("source_has_parse_errors".into()),
                )
            } else if import.unsupported {
                (
                    None,
                    Resolution::Unavailable,
                    Some("unsupported_import_form".into()),
                )
            } else if import.candidates.len() == 1 {
                let target = import.candidates[0];
                if parsed[target].errors > 0
                    || parsed[target].unavailable.is_some()
                    || parsed[target].limitation.is_some()
                {
                    (
                        None,
                        Resolution::Partial,
                        Some("target_structure_unavailable".into()),
                    )
                } else {
                    (
                        Some(parsed[target].file_id.clone()),
                        Resolution::Resolved,
                        None,
                    )
                }
            } else if import.candidates.is_empty() {
                (
                    None,
                    Resolution::Unavailable,
                    Some("external_or_unavailable_module".into()),
                )
            } else {
                (
                    None,
                    Resolution::Partial,
                    Some("ambiguous_module_path".into()),
                )
            };
            edges.push(make_edge(
                import.source.clone(),
                target,
                Relation::Imports,
                import.module.clone(),
                import.evidence.clone(),
                status,
                reason,
            ));
        }
        for usage in &unit.uses {
            check(cancel)?;
            let (target, status, reason) = resolve_use(usage, &parsed, &nodes);
            edges.push(make_edge(
                usage.source.clone(),
                target,
                usage.kind.clone(),
                usage.name.clone(),
                usage.evidence.clone(),
                status,
                reason,
            ));
        }
    }
    edges.sort_by(|a, b| a.id.cmp(&b.id));
    edges.dedup_by(|a, b| a.id == b.id);
    let coverage = parsed
        .iter()
        .zip(&docs)
        .map(|(unit, doc)| {
            let resolved = edges
                .iter()
                .filter(|e| {
                    e.evidence.path_key == doc.path_key && e.resolution == Resolution::Resolved
                })
                .count();
            let unresolved = edges
                .iter()
                .filter(|e| {
                    e.evidence.path_key == doc.path_key && e.resolution != Resolution::Resolved
                })
                .count();
            Coverage {
                path_key: doc.path_key.clone(),
                content_sha256: doc.content_sha256.clone(),
                language: unit.language.map(|l| l.name().into()),
                status: if unit.unavailable.is_some() {
                    Resolution::Unavailable
                } else if unit.errors > 0 || unit.limitation.is_some() || unresolved > 0 {
                    Resolution::Partial
                } else {
                    Resolution::Resolved
                },
                parse_errors: unit.errors,
                symbols: unit.symbols.len(),
                resolved_relations: resolved,
                unresolved_relations: unresolved,
                reason: unit
                    .unavailable
                    .clone()
                    .or_else(|| unit.limitation.clone())
                    .or_else(|| (unit.errors > 0).then(|| "source_has_parse_errors".into())),
            }
        })
        .collect();
    nodes.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(Graph {
        schema_version: "1".into(),
        generation,
        producer: format!(
            "compass-tree-sitter-v1;path-only-conservative-resolution;no-project-code;max-file=8388608;max-depth=256;max-syntax-nodes=200000;build={}",
            crate::BUILD_ID
        ),
        nodes,
        edges,
        coverage,
    })
}

impl Graph {
    /// Bounded undirected expansion for navigation; every returned edge retains
    /// its original direction/type. Unresolved edges are visible but never traversed.
    pub fn neighbors(
        &self,
        node: &str,
        depth: usize,
        limit: usize,
        cancel: &AtomicBool,
    ) -> Result<Neighborhood> {
        check(cancel)?;
        if limit == 0 || limit > 10_000 || depth > 32 {
            return Err(Error::invalid(
                "structural neighborhood requires limit 1..10000 and depth 0..32",
            ));
        }
        let by_id: BTreeMap<_, _> = self.nodes.iter().map(|n| (n.id.as_str(), n)).collect();
        if !by_id.contains_key(node) {
            return Err(Error::invalid("structural node not found in generation"));
        }
        let mut selected = BTreeSet::from([node.to_owned()]);
        let mut queue = VecDeque::from([(node.to_owned(), 0)]);
        let mut truncated = false;
        while let Some((current, distance)) = queue.pop_front() {
            check(cancel)?;
            if distance == depth {
                continue;
            }
            for edge in &self.edges {
                check(cancel)?;
                if edge.resolution != Resolution::Resolved {
                    continue;
                }
                let Some(target) = &edge.target else {
                    continue;
                };
                let next = if edge.source == current {
                    target
                } else if *target == current {
                    &edge.source
                } else {
                    continue;
                };
                if selected.contains(next) {
                    continue;
                }
                if selected.len() == limit {
                    truncated = true;
                    continue;
                }
                selected.insert(next.clone());
                queue.push_back((next.clone(), distance + 1));
            }
        }
        let nodes = self
            .nodes
            .iter()
            .filter(|n| selected.contains(&n.id))
            .cloned()
            .collect();
        let mut edges: Vec<_> = self
            .edges
            .iter()
            .filter(|e| {
                selected.contains(&e.source)
                    && e.target.as_ref().is_none_or(|t| selected.contains(t))
            })
            .cloned()
            .collect();
        if edges.len() > limit {
            edges.truncate(limit);
            truncated = true;
        }
        Ok(Neighborhood {
            generation: self.generation.clone(),
            root: node.into(),
            nodes,
            edges,
            truncated,
        })
    }

    /// Unit, undirected projection for numerical consumers. No inferred edge is
    /// added: only resolved contains/imports/calls are projected and deduplicated.
    pub fn projection_pairs(&self) -> Vec<(String, String)> {
        self.edges
            .iter()
            .filter(|e| {
                e.resolution == Resolution::Resolved
                    && matches!(
                        e.kind,
                        Relation::Contains | Relation::Imports | Relation::Calls
                    )
            })
            .filter_map(|e| {
                e.target.as_ref().and_then(|t| {
                    if *t == e.source {
                        None
                    } else if e.source < *t {
                        Some((e.source.clone(), t.clone()))
                    } else {
                        Some((t.clone(), e.source.clone()))
                    }
                })
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}
