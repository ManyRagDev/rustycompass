//! Explicit, mandatory evidence acquisition. Completion is evaluated against
//! bytes in this response only; ranking and graph diffusion are never proof.
use crate::{
    Error, Result, codec,
    collection::{Collection, VerifiedDocument},
    diffusion, model,
    retrieval::RetrieveRequest,
    structure::{Graph, Node, Resolution},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::{AtomicBool, Ordering},
};
use tree_sitter::{Parser, Tree};

const POLICY: &str = "acquisition-v1-syntax-current-pack";
fn check(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Relaxed) {
        Err(Error::cancelled())
    } else {
        Ok(())
    }
}
fn invalid(s: &str) -> Error {
    Error::invalid(s)
}
fn fields(v: &Value, allowed: &[&str]) -> Result<()> {
    let o = v
        .as_object()
        .ok_or_else(|| invalid("contract object required"))?;
    if o.keys().any(|k| !allowed.contains(&k.as_str())) {
        return Err(invalid("unknown contract field"));
    }
    Ok(())
}
fn string<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v[key]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| invalid("nonempty contract string required"))
}
fn path(v: &Value) -> Result<()> {
    let s = v
        .as_str()
        .ok_or_else(|| invalid("path_key must be string"))?;
    let bytes = model::decode_path_key(s)?;
    if bytes.is_empty() {
        return Err(invalid("file path_key must not be empty"));
    }
    Ok(())
}
fn selector(v: &Value, explicit_generation: bool) -> Result<()> {
    fields(v, &["name", "path_key", "scope", "node_id"])?;
    if v.get("node_id").is_some() {
        if !explicit_generation || v.as_object().unwrap().len() != 1 {
            return Err(invalid(
                "node_id selector requires explicit generation and no name/path/scope",
            ));
        }
        let id = string(v, "node_id")?;
        if id.len() != 64
            || !id
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(invalid("invalid node_id"));
        }
    } else {
        string(v, "name")?;
        if let Some(p) = v.get("path_key") {
            path(p)?;
        }
        if let Some(s) = v.get("scope") {
            let s = s
                .as_array()
                .ok_or_else(|| invalid("scope must be string array"))?;
            if s.len() > 256 || s.iter().any(|s| s.as_str().is_none_or(str::is_empty)) {
                return Err(invalid("invalid scope"));
            }
        }
    }
    Ok(())
}
/// Validate before reading any derived index or captured object. Unknown kinds
/// remain explicit unsupported obligations; their payload is hash-bound.
pub fn validate_contract(v: &Value, explicit_generation: bool) -> Result<()> {
    if codec::canonical(v)?.len() > 16 * 1024 {
        return Err(invalid("contract exceeds 16KiB canonical JSON"));
    }
    fields(
        v,
        &[
            "schema_version",
            "intent",
            "obligations",
            "interpretation_pending",
        ],
    )?;
    if v["schema_version"] != "1" {
        return Err(invalid("unsupported contract schema_version"));
    }
    string(v, "intent")?;
    let obligations = v["obligations"]
        .as_array()
        .filter(|o| (1..=16).contains(&o.len()))
        .ok_or_else(|| invalid("contract requires 1..16 obligations"))?;
    let pending = v["interpretation_pending"]
        .as_array()
        .ok_or_else(|| invalid("interpretation_pending must be string array"))?;
    if pending.iter().any(|s| s.as_str().is_none_or(str::is_empty)) {
        return Err(invalid("invalid interpretation_pending"));
    }
    let mut ids = BTreeSet::new();
    for o in obligations {
        let id = string(o, "id")?;
        if id.len() > 128 || !ids.insert(id) {
            return Err(invalid(
                "obligation ids must be unique and at most 128 bytes",
            ));
        }
        match string(o, "kind")? {
            "symbol_definition" => {
                fields(o, &["id", "kind", "symbol"])?;
                selector(&o["symbol"], explicit_generation)?;
            }
            "literal_occurrence" => {
                fields(o, &["id", "kind", "literal", "path_key", "symbol"])?;
                string(o, "literal")?;
                if o.get("path_key").is_some() == o.get("symbol").is_some() {
                    return Err(invalid(
                        "literal requires exactly one path_key or symbol scope",
                    ));
                }
                if let Some(p) = o.get("path_key") {
                    path(p)?;
                }
                if let Some(s) = o.get("symbol") {
                    selector(s, explicit_generation)?;
                }
            }
            "resolved_relation" => {
                fields(o, &["id", "kind", "source", "target", "relation"])?;
                selector(&o["source"], explicit_generation)?;
                selector(&o["target"], explicit_generation)?;
                if !["contains", "imports", "calls", "references"].contains(&string(o, "relation")?)
                {
                    return Err(invalid("unsupported relation type"));
                }
            }
            _ => {
                if !o.is_object() {
                    return Err(invalid("obligation object required"));
                }
            }
        }
    }
    Ok(())
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Range {
    path: String,
    start: usize,
    end: usize,
}
struct Obligation {
    id: Value,
    kind: Value,
    ranges: Vec<Range>,
    reason: Option<&'static str>,
    unsupported: bool,
    resolution: Value,
}
struct Document {
    verified: VerifiedDocument,
    tree: Option<Tree>,
    lines: Vec<usize>,
}
struct Acquisition<'a> {
    collection: &'a Collection,
    graph: Graph,
    docs: BTreeMap<String, Document>,
    cancel: &'a AtomicBool,
    seeds: BTreeSet<String>,
}
fn grammar(language: &str) -> Option<tree_sitter::Language> {
    Some(match language {
        "python" => tree_sitter_python::LANGUAGE.into(),
        "rust" => tree_sitter_rust::LANGUAGE.into(),
        "javascript" => tree_sitter_javascript::LANGUAGE.into(),
        "typescript" => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        "tsx" => tree_sitter_typescript::LANGUAGE_TSX.into(),
        _ => return None,
    })
}
impl Acquisition<'_> {
    fn document(&mut self, key: &str) -> Result<&Document> {
        check(self.cancel)?;
        if !self.docs.contains_key(key) {
            let verified = self.collection.verified_document(key, self.cancel)?;
            // A syntactic proof needs lossless text. Binary/invalid UTF8 is an
            // explicit unsupported scope, never lossy text claimed as evidence.
            let language = self
                .graph
                .coverage
                .iter()
                .find(|c| c.path_key == key)
                .and_then(|c| c.language.as_deref());
            let tree = if let Some(g) = language.and_then(grammar) {
                let mut parser = Parser::new();
                parser
                    .set_language(&g)
                    .map_err(|e| Error::corrupt(e.to_string()))?;
                parser.parse(&verified.bytes, None)
            } else {
                None
            };
            let mut lines = vec![0];
            for (i, b) in verified.bytes.iter().enumerate() {
                if i % 65536 == 0 {
                    check(self.cancel)?;
                }
                if *b == b'\n' && i + 1 < verified.bytes.len() {
                    lines.push(i + 1);
                }
            }
            self.docs.insert(
                key.into(),
                Document {
                    verified,
                    tree,
                    lines,
                },
            );
        }
        Ok(&self.docs[key])
    }
    fn resolve(&self, selector: &Value) -> std::result::Result<Node, &'static str> {
        let mut found = None;
        for (i, n) in self.graph.nodes.iter().enumerate() {
            // Cancellation is propagated at the acquisition boundary too.
            if i % 1024 == 0 && self.cancel.load(Ordering::Relaxed) {
                return Err("cancelled");
            }
            let matches = if let Some(id) = selector["node_id"].as_str() {
                n.id == id
            } else {
                n.kind != "file"
                    && selector["name"] == n.name
                    && selector.get("path_key").is_none_or(|p| *p == n.path_key)
                    && selector.get("scope").is_none_or(|s| *s == json!(n.scope))
            };
            if matches {
                if found.is_some() {
                    return Err("ambiguous_symbol");
                }
                found = Some(n.clone());
            }
        }
        found.ok_or("symbol_not_found")
    }
    fn syntax_range(&mut self, node: &Node) -> Result<std::result::Result<Range, &'static str>> {
        if node.kind == "file" || node.kind == "parameter" {
            return Ok(Err("unsupported_symbol_kind"));
        }
        let coverage = self
            .graph
            .coverage
            .iter()
            .find(|c| c.path_key == node.path_key && c.content_sha256 == node.content_sha256);
        if coverage.is_none_or(|c| {
            c.parse_errors != 0 || c.language.is_none() || c.status == Resolution::Unavailable
        }) {
            return Ok(Err("parse_coverage_unavailable"));
        }
        let doc = self.document(&node.path_key)?;
        if doc.verified.content_sha256 != node.content_sha256 {
            return Err(Error::corrupt("graph symbol revision mismatch"));
        }
        if std::str::from_utf8(&doc.verified.bytes).is_err() || doc.verified.bytes.contains(&0) {
            return Ok(Err("unsupported_text_encoding"));
        }
        let Some(tree) = &doc.tree else {
            return Ok(Err("parse_coverage_unavailable"));
        };
        if tree.root_node().has_error() {
            return Ok(Err("parse_coverage_unavailable"));
        }
        let Some(mut syntax) = tree
            .root_node()
            .descendant_for_byte_range(node.start_byte, node.end_byte)
        else {
            return Err(Error::corrupt("graph symbol outside syntax"));
        };
        if syntax.start_byte() != node.start_byte || syntax.end_byte() != node.end_byte {
            return Err(Error::corrupt("graph symbol does not match syntax"));
        }
        // Lift declarators to their complete declaration, and include recognized
        // decorators/export wrappers without changing generation-bound node IDs.
        while let Some(parent) = syntax.parent() {
            if matches!(
                parent.kind(),
                "decorated_definition"
                    | "export_statement"
                    | "lexical_declaration"
                    | "variable_declaration"
            ) {
                syntax = parent;
            } else {
                break;
            }
        }
        let mut start = syntax.start_byte();
        let end = syntax.end_byte();
        let mut previous = syntax.prev_named_sibling();
        while let Some(p) = previous {
            if matches!(p.kind(), "attribute_item" | "inner_attribute_item")
                && doc.verified.bytes[p.end_byte()..start]
                    .iter()
                    .all(u8::is_ascii_whitespace)
            {
                start = p.start_byte();
                previous = p.prev_named_sibling();
            } else {
                break;
            }
        }
        Ok(Ok(Range {
            path: node.path_key.clone(),
            start,
            end,
        }))
    }
    fn identity_range(&mut self, node: &Node) -> Result<Range> {
        let doc = self.document(&node.path_key)?;
        if doc.verified.content_sha256 != node.content_sha256 {
            return Err(Error::corrupt("graph endpoint revision mismatch"));
        }
        if node.start_byte > node.end_byte || node.end_byte > doc.verified.bytes.len() {
            return Err(Error::corrupt("graph endpoint outside object"));
        }
        if node.kind == "file" {
            return Ok(line_range(doc, &node.path_key, 0, 0));
        }
        let bytes = &doc.verified.bytes[node.start_byte..node.end_byte];
        let at = bytes
            .windows(node.name.len().max(1))
            .position(|w| w == node.name.as_bytes())
            .ok_or_else(|| Error::corrupt("endpoint identity absent from source"))?
            + node.start_byte;
        Ok(line_range(doc, &node.path_key, at, at + node.name.len()))
    }
    fn acquire(&mut self, o: &Value) -> Result<Obligation> {
        check(self.cancel)?;
        let mut result = Obligation {
            id: o["id"].clone(),
            kind: o["kind"].clone(),
            ranges: vec![],
            reason: None,
            unsupported: false,
            resolution: Value::Null,
        };
        match o["kind"].as_str().unwrap() {
            "symbol_definition" => match self.resolve(&o["symbol"]) {
                Err(reason) => result.reason = Some(reason),
                Ok(n) => match self.syntax_range(&n)? {
                    Err(reason) => {
                        result.reason = Some(reason);
                        result.unsupported = reason.starts_with("unsupported");
                    }
                    Ok(range) => {
                        self.seeds.insert(n.id.clone());
                        result.ranges.push(range.clone());
                        result.resolution = json!({"node_id":n.id,"path_key":n.path_key,"content_sha256":n.content_sha256,"start_byte":range.start,"end_byte":range.end});
                    }
                },
            },
            "literal_occurrence" => {
                let scope = if let Some(key) = o["path_key"].as_str() {
                    if !self
                        .collection
                        .snapshot
                        .entries
                        .iter()
                        .any(|e| e.path_key == key && e.state == "observed")
                    {
                        result.reason = Some("path_not_observed");
                        return Ok(result);
                    }
                    let doc = self.document(key)?;
                    Range {
                        path: key.into(),
                        start: 0,
                        end: doc.verified.bytes.len(),
                    }
                } else {
                    let n = match self.resolve(&o["symbol"]) {
                        Ok(n) => n,
                        Err(reason) => {
                            result.reason = Some(reason);
                            return Ok(result);
                        }
                    };
                    self.seeds.insert(n.id.clone());
                    match self.syntax_range(&n)? {
                        Ok(range) => range,
                        Err(reason) => {
                            result.reason = Some(reason);
                            return Ok(result);
                        }
                    }
                };
                let cancel = self.cancel;
                let doc = self.document(&scope.path)?;
                if std::str::from_utf8(&doc.verified.bytes).is_err()
                    || doc.verified.bytes.contains(&0)
                {
                    result.reason = Some("unsupported_text_encoding");
                    result.unsupported = true;
                    return Ok(result);
                }
                let literal = o["literal"].as_str().unwrap().as_bytes();
                let mut at = None;
                for (i, window) in doc.verified.bytes[scope.start..scope.end]
                    .windows(literal.len())
                    .enumerate()
                {
                    if i % 65536 == 0 {
                        check(cancel)?;
                    }
                    if window == literal {
                        at = Some(scope.start + i);
                        break;
                    }
                }
                if let Some(at) = at {
                    result
                        .ranges
                        .push(line_range(doc, &scope.path, at, at + literal.len()));
                    result.resolution =
                        json!({"occurrence_start_byte":at,"occurrence_end_byte":at+literal.len()});
                } else {
                    result.reason = Some("literal_not_found");
                }
            }
            "resolved_relation" => {
                let source = match self.resolve(&o["source"]) {
                    Ok(n) => n,
                    Err(reason) => {
                        result.reason = Some(reason);
                        return Ok(result);
                    }
                };
                let target = match self.resolve(&o["target"]) {
                    Ok(n) => n,
                    Err(reason) => {
                        result.reason = Some(reason);
                        return Ok(result);
                    }
                };
                let mut found = None;
                for (i, e) in self.graph.edges.iter().enumerate() {
                    if i % 1024 == 0 {
                        check(self.cancel)?;
                    }
                    if e.source == source.id
                        && e.target.as_deref() == Some(&target.id)
                        && e.resolution == Resolution::Resolved
                        && json!(e.kind) == o["relation"]
                    {
                        found = Some(e.clone());
                        break;
                    }
                }
                let Some(edge) = found else {
                    result.reason = Some("resolved_direct_relation_not_found");
                    return Ok(result);
                };
                // Contains edges also exist for incomplete parses. Never promote
                // them (or any source edge) through missing parse coverage.
                if [&source.path_key, &target.path_key, &edge.evidence.path_key]
                    .iter()
                    .any(|key| {
                        self.graph
                            .coverage
                            .iter()
                            .find(|c| c.path_key == **key)
                            .is_none_or(|c| {
                                c.parse_errors != 0 || c.status == Resolution::Unavailable
                            })
                    })
                {
                    result.reason = Some("parse_coverage_unavailable");
                    return Ok(result);
                }
                let doc = self.document(&edge.evidence.path_key)?;
                let ev = &edge.evidence;
                if doc.verified.content_sha256 != ev.content_sha256
                    || ev.start_byte > ev.end_byte
                    || ev.end_byte > doc.verified.bytes.len()
                {
                    return Err(Error::corrupt("relation witness outside verified object"));
                }
                result
                    .ranges
                    .push(line_range(doc, &ev.path_key, ev.start_byte, ev.end_byte));
                result.ranges.push(self.identity_range(&source)?);
                result.ranges.push(self.identity_range(&target)?);
                result.resolution = json!({"edge_id":edge.id,"relation":edge.kind,"source":{"node_id":source.id,"path_key":source.path_key,"content_sha256":source.content_sha256,"name":source.name,"start_byte":source.start_byte,"end_byte":source.end_byte},"target":{"node_id":target.id,"path_key":target.path_key,"content_sha256":target.content_sha256,"name":target.name,"start_byte":target.start_byte,"end_byte":target.end_byte}});
                self.seeds.extend([source.id, target.id]);
            }
            _ => {
                result.reason = Some("unsupported_obligation_kind");
                result.unsupported = true;
            }
        }
        check(self.cancel)?;
        Ok(result)
    }
}
fn line_range(doc: &Document, path: &str, start: usize, end: usize) -> Range {
    let start_index = doc.lines.partition_point(|x| *x <= start).saturating_sub(1);
    let last_index = doc
        .lines
        .partition_point(|x| *x <= end.saturating_sub(1).max(start))
        .saturating_sub(1);
    Range {
        path: path.into(),
        start: doc.lines[start_index],
        end: doc
            .lines
            .get(last_index + 1)
            .copied()
            .unwrap_or(doc.verified.bytes.len()),
    }
}
fn union(mut ranges: Vec<Range>) -> Vec<Range> {
    ranges.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then(a.start.cmp(&b.start))
            .then(a.end.cmp(&b.end))
    });
    let mut merged: Vec<Range> = vec![];
    for range in ranges {
        if let Some(last) = merged.last_mut()
            && last.path == range.path
            && range.start <= last.end
        {
            last.end = last.end.max(range.end);
        } else {
            merged.push(range);
        }
    }
    merged
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Cursor {
    policy: String,
    generation: String,
    request_hash: String,
    progress: Vec<usize>,
}
fn decode_cursor(raw: &str) -> Result<Cursor> {
    if raw.len() > 4096 {
        return Err(invalid("cursor too large"));
    }
    serde_json::from_slice(
        &URL_SAFE_NO_PAD
            .decode(raw)
            .map_err(|_| invalid("invalid acquisition cursor"))?,
    )
    .map_err(|_| invalid("invalid acquisition cursor"))
}
pub fn cursor_generation(raw: &str) -> Result<String> {
    let cursor = decode_cursor(raw)?;
    if cursor.policy != POLICY {
        return Err(invalid("incompatible acquisition cursor policy"));
    }
    Ok(cursor.generation)
}
fn next_cursor(generation: &str, hash: &str, queue: &[Range], progress: &[usize]) -> Result<Value> {
    if queue
        .iter()
        .zip(progress)
        .all(|(r, p)| *p == r.end - r.start)
    {
        return Ok(Value::Null);
    }
    Ok(json!(URL_SAFE_NO_PAD.encode(codec::canonical(&Cursor {
        policy: POLICY.into(),
        generation: generation.into(),
        request_hash: hash.into(),
        progress: progress.to_vec()
    })?)))
}
fn item(range: &Range, docs: &BTreeMap<String, Document>, truncated: bool) -> Result<Value> {
    let d = &docs[&range.path];
    let text = std::str::from_utf8(&d.verified.bytes[range.start..range.end])
        .map_err(|_| invalid("unsupported text encoding"))?;
    let id = model::digest(&codec::canonical(&json!([
        range.path,
        d.verified.content_sha256,
        range.start,
        range.end
    ]))?);
    Ok(
        json!({"span_id":id,"path_key":range.path,"content_sha256":d.verified.content_sha256,"start_byte":range.start,"end_byte":range.end,"start_line":d.lines.partition_point(|x| *x<=range.start),"end_line":d.lines.partition_point(|x| *x<range.end).max(1),"text":text,"verification":"full_object_sha256","truncated":truncated}),
    )
}
fn completion(obligations: &[Obligation], items: &[Value], pending: &Value) -> Value {
    let mut all = true;
    let outcomes:Vec<_>=obligations.iter().map(|o| {
        let mut witnesses=vec![];
        let mut complete=o.reason.is_none() && !o.ranges.is_empty();
        for range in &o.ranges {
            let mut covered=range.start;
            for item in items.iter().filter(|item| item["path_key"]==range.path) {
                let start=item["start_byte"].as_u64().unwrap() as usize;
                let end=item["end_byte"].as_u64().unwrap() as usize;
                if start<range.end && end>range.start {
                    witnesses.push(json!({"span_id":item["span_id"],"start_byte":start.max(range.start),"end_byte":end.min(range.end)}));
                }
                if start<=covered && end>covered { covered=end; }
            }
            if covered<range.end { complete=false; }
        }
        let status=if complete {"satisfied"} else if o.unsupported {"unsupported"} else if !witnesses.is_empty() {"partial"} else {"unresolved"};
        all &= complete;
        json!({"id":o.id,"kind":o.kind,"status":status,"witnesses":witnesses,"reason":if complete {None} else {Some(o.reason.unwrap_or("evidence_not_in_current_pack"))},"resolution":o.resolution})
    }).collect();
    all &= pending.as_array().unwrap().is_empty();
    json!({"scope":"current_pack","status":if all {"SUFFICIENT_EVIDENCE_ACQUIRED"}else{"INSUFFICIENT_EVIDENCE"},"obligations":outcomes,"interpretation_pending":pending})
}
fn finalize(pack: &mut Value) -> Result<usize> {
    for _ in 0..8 {
        let size = codec::canonical(pack)?.len();
        if pack["payload_bytes"] == size {
            return Ok(size);
        }
        pack["payload_bytes"] = json!(size);
    }
    Err(Error::corrupt("pack byte count did not converge"))
}
fn make_pack(
    base: &Value,
    selected: &[Range],
    progress: &[usize],
    queue: &[Range],
    context: &Acquisition<'_>,
    completion_data: (&[Obligation], &Value),
    hash: &str,
) -> Result<Value> {
    let (obligations, pending) = completion_data;
    let mut pack = base.clone();
    pack["items"] = json!(
        union(selected.to_vec())
            .iter()
            .map(|range| item(
                range,
                &context.docs,
                queue.iter().any(|full| full.path == range.path
                    && full.start <= range.start
                    && range.end < full.end)
            ))
            .collect::<Result<Vec<_>>>()?
    );
    let emitted = union(selected.to_vec());
    let mut missing = vec![];
    for required in union(queue.to_vec()) {
        let mut at = required.start;
        for present in emitted
            .iter()
            .filter(|p| p.path == required.path && p.end > required.start && p.start < required.end)
        {
            if present.start > at {
                missing.push(Range {
                    path: required.path.clone(),
                    start: at,
                    end: present.start.min(required.end),
                });
            }
            at = at.max(present.end.min(required.end));
        }
        if at < required.end {
            missing.push(Range {
                path: required.path,
                start: at,
                end: required.end,
            });
        }
    }
    pack["omitted"]=json!(missing.iter().map(|r| json!({"path_key":r.path,"content_sha256":context.docs[&r.path].verified.content_sha256,"start_byte":r.start,"end_byte":r.end,"reason":"evidence_not_in_current_pack"})).collect::<Vec<_>>());
    pack["completion"] = completion(obligations, pack["items"].as_array().unwrap(), pending);
    pack["next_cursor"] = next_cursor(context.collection.generation_id(), hash, queue, progress)?;
    finalize(&mut pack)?;
    Ok(pack)
}
fn advance(queue: &[Range], progress: &[usize], selected: &[Range]) -> Vec<usize> {
    let selected = union(selected.to_vec());
    queue
        .iter()
        .zip(progress)
        .map(|(range, p)| {
            let mut at = range.start + *p;
            for selected in selected.iter().filter(|s| s.path == range.path) {
                if selected.start <= at && selected.end > at {
                    at = selected.end.min(range.end);
                }
            }
            at - range.start
        })
        .collect()
}
/// Acquire a current-page proof for a structured contract, independently of the
/// legacy ranked retrieval protocol. Exact selectors bypass channel truncation.
pub fn retrieve(
    c: &Collection,
    r: &RetrieveRequest,
    contract: &Value,
    explicit_generation: bool,
    cancel: &AtomicBool,
) -> Result<Value> {
    validate_contract(contract, explicit_generation)?;
    if !(2048..=1048576).contains(&r.budget_bytes)
        || !["none", "bfs", "heat"].contains(&r.expansion.as_str())
        || !r.theta.is_finite()
        || !(0.0..=10.0).contains(&r.theta)
        || !["literal", "lexical", "semantic", "hybrid"].contains(&r.mode.as_str())
        || r.query.trim().is_empty()
        || r.query.len() > 4096
    {
        return Err(invalid(
            "invalid acquisition query/mode/budget/expansion/theta",
        ));
    }
    check(cancel)?;
    let hash = model::digest(&codec::canonical(&json!([
        POLICY,
        c.snapshot.policy_id,
        contract,
        r.query,
        r.mode,
        r.expansion,
        r.theta,
        r.budget_bytes
    ]))?);
    let prior = r
        .cursor
        .as_ref()
        .map(|raw| decode_cursor(raw))
        .transpose()?;
    if prior.as_ref().is_some_and(|p| {
        p.policy != POLICY || p.generation != c.generation_id() || p.request_hash != hash
    }) {
        return Err(invalid(
            "cursor belongs to another generation/contract/request/policy",
        ));
    }
    let graph = crate::retrieval::acquisition_graph(c)?;
    let mut context = Acquisition {
        collection: c,
        graph,
        docs: BTreeMap::new(),
        cancel,
        seeds: BTreeSet::new(),
    };
    let mut obligations = vec![];
    for obligation in contract["obligations"].as_array().unwrap() {
        obligations.push(context.acquire(obligation)?);
    }
    check(cancel)?;
    let mut queue = vec![];
    for obligation in &obligations {
        for range in &obligation.ranges {
            if !queue.contains(range) {
                queue.push(range.clone());
            }
        }
    }
    let mut progress = prior.map_or_else(|| vec![0; queue.len()], |p| p.progress);
    if progress.len() != queue.len() {
        return Err(invalid("cursor evidence queue mismatch"));
    }
    for (range, p) in queue.iter().zip(&progress) {
        if *p > range.end - range.start {
            return Err(invalid("cursor offset outside required evidence"));
        }
        let at = range.start + *p;
        if at != range.start && at != range.end && !context.docs[&range.path].lines.contains(&at) {
            return Err(invalid("cursor must resume at original line boundary"));
        }
    }
    let mut expansion = json!({"method":r.expansion,"selection":"mandatory_evidence_only","scores_are_proof":false});
    if r.expansion != "none" && !context.seeds.is_empty() {
        let projection = diffusion::Graph::new(
            context.graph.nodes.iter().map(|n| n.id.clone()).collect(),
            &context.graph.projection_pairs(),
        )?;
        let seeds = context.seeds.iter().cloned().collect::<Vec<_>>();
        if r.expansion == "heat" {
            let heat = projection.heat(
                &seeds.iter().map(|s| (s.clone(), 1.0)).collect::<Vec<_>>(),
                r.theta,
                20_000_000,
                cancel,
            )?;
            expansion["projection"] = json!("resolved_contains_imports_calls_unit_v1");
            expansion["status"] = json!(heat.status);
            expansion["theta"] = json!(heat.theta);
            expansion["iterations"] = json!(heat.iterations);
            expansion["edge_visits"] = json!(heat.edge_visits);
            expansion["truncation_tail_bound"] = json!(heat.truncation_tail_bound);
            expansion["floating_point_error"] = json!(heat.floating_point_error);
            expansion["reached_nodes"] = json!(heat.scores.iter().filter(|s| **s > 0.0).count());
        } else {
            let bfs = projection.bfs(&seeds, 2, 1000, 100000, cancel)?;
            expansion["projection"] = json!("resolved_contains_imports_calls_unit_v1");
            expansion["complete"] = json!(bfs.complete);
            expansion["depth"] = json!(2);
            expansion["edge_visits"] = json!(bfs.edge_visits);
            expansion["reached_nodes"] = json!(bfs.nodes.len());
        }
    }
    let contract_hash = model::digest(&codec::canonical(contract)?);
    let base = json!({"schema_version":"2","report_type":"evidence_pack","generation":c.generation_id(),"producer":POLICY,"graph_producer":context.graph.producer,"contract_hash":contract_hash,"query":r.query,"mode":r.mode,"coverage":{"selection":"explicit_contract_selectors","ranking":"not_required_for_exact_selectors","verified_files":context.docs.len(),"proof_scope":"current_pack_text_only"},"expansion":expansion,"budget_bytes":r.budget_bytes,"payload_bytes":0,"items":[],"omitted":[],"completion":{},"next_cursor":null});
    let pending = &contract["interpretation_pending"];
    let mut selected = vec![];
    let mut pack = make_pack(
        &base,
        &selected,
        &progress,
        &queue,
        &context,
        (&obligations, pending),
        &hash,
    )?;
    // First attempt the whole union. Otherwise complete obligations in declared
    // order before spending the remaining bytes on partial original lines.
    let remaining: Vec<_> = queue
        .iter()
        .zip(&progress)
        .filter(|(q, p)| **p < q.end - q.start)
        .map(|(q, p)| Range {
            path: q.path.clone(),
            start: q.start + *p,
            end: q.end,
        })
        .collect();
    let full_progress = advance(&queue, &progress, &remaining);
    let all = make_pack(
        &base,
        &remaining,
        &full_progress,
        &queue,
        &context,
        (&obligations, pending),
        &hash,
    )?;
    if codec::canonical(&all)?.len() <= r.budget_bytes {
        pack = all;
    } else {
        for obligation in &obligations {
            check(cancel)?;
            if obligation.ranges.is_empty() {
                continue;
            }
            let mut trial = selected.clone();
            for (q, p) in queue.iter().zip(&progress) {
                if obligation.ranges.contains(q) && *p < q.end - q.start {
                    trial.push(Range {
                        path: q.path.clone(),
                        start: q.start + *p,
                        end: q.end,
                    });
                }
            }
            let next = advance(&queue, &progress, &trial);
            let candidate = make_pack(
                &base,
                &trial,
                &next,
                &queue,
                &context,
                (&obligations, pending),
                &hash,
            )?;
            if codec::canonical(&candidate)?.len() <= r.budget_bytes {
                selected = trial;
                progress = next;
                pack = candidate;
            }
        }
        for i in 0..queue.len() {
            check(cancel)?;
            let range = &queue[i];
            if progress[i] >= range.end - range.start {
                continue;
            }
            let start = range.start + progress[i];
            // A complete individual range removes its omitted-tail metadata.
            // It can therefore fit even when all shorter prefixes do not.
            let mut whole = selected.clone();
            whole.push(Range {
                path: range.path.clone(),
                start,
                end: range.end,
            });
            let next = advance(&queue, &progress, &whole);
            let candidate = make_pack(
                &base,
                &whole,
                &next,
                &queue,
                &context,
                (&obligations, pending),
                &hash,
            )?;
            if codec::canonical(&candidate)?.len() <= r.budget_bytes {
                selected = whole;
                progress = next;
                pack = candidate;
                continue;
            }
            let mut ends: Vec<_> = context.docs[&range.path]
                .lines
                .iter()
                .copied()
                .filter(|at| *at > start && *at < range.end)
                .collect();
            ends.push(range.end);
            let (mut low, mut high, mut best) = (0, ends.len(), None);
            while low < high {
                check(cancel)?;
                let middle = low + (high - low) / 2;
                let mut trial = selected.clone();
                trial.push(Range {
                    path: range.path.clone(),
                    start,
                    end: ends[middle],
                });
                let next = advance(&queue, &progress, &trial);
                let candidate = make_pack(
                    &base,
                    &trial,
                    &next,
                    &queue,
                    &context,
                    (&obligations, pending),
                    &hash,
                )?;
                if codec::canonical(&candidate)?.len() <= r.budget_bytes {
                    best = Some((trial, next, candidate));
                    low = middle + 1;
                } else {
                    high = middle;
                }
            }
            if let Some((trial, next, candidate)) = best {
                selected = trial;
                progress = next;
                pack = candidate;
            }
        }
    }
    // If no complete line can ever fit a fresh page, record the omitted range
    // and consume it. This is an explicit acquisition failure, not proof.
    if pack["items"].as_array().unwrap().is_empty() && !remaining.is_empty() {
        for omission in pack["omitted"].as_array_mut().unwrap() {
            omission["reason"] = json!("budget_cannot_fit_complete_line");
        }
        for (i, range) in queue.iter().enumerate() {
            progress[i] = range.end - range.start;
        }
        pack["next_cursor"] = next_cursor(c.generation_id(), &hash, &queue, &progress)?;
        if finalize(&mut pack)? > r.budget_bytes {
            return Err(invalid(
                "budget cannot fit acquisition omission metadata; increase budget_bytes",
            ));
        }
    }
    check(cancel)?;
    if finalize(&mut pack)? > r.budget_bytes {
        return Err(invalid(
            "budget cannot fit acquisition metadata; increase budget_bytes",
        ));
    }
    Ok(pack)
}
