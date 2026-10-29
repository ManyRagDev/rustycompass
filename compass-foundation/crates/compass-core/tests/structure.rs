use compass_core::structure::{self, Document, Graph, Relation, Resolution};
use sha2::{Digest, Sha256};
use std::sync::atomic::AtomicBool;

fn doc(path: &str, source: &str) -> Document {
    Document {
        path_key: compass_core::model::path_key(path.as_bytes()),
        path: path.into(),
        content_sha256: format!("{:x}", Sha256::digest(source.as_bytes())),
        bytes: source.as_bytes().to_vec(),
    }
}
fn graph(docs: &[Document]) -> Graph {
    structure::build(docs, "fixture-generation".into(), &AtomicBool::new(false)).unwrap()
}
fn target<'a>(g: &'a Graph, path: &str, name: &str) -> &'a str {
    &g.nodes
        .iter()
        .find(|n| n.path == path && n.name == name && n.kind != "file")
        .unwrap()
        .id
}
fn call_to(g: &Graph, path: &str, literal: &str, target_id: &str) -> bool {
    g.edges.iter().any(|e| {
        e.kind == Relation::Calls
            && e.name == literal
            && e.evidence.path_key == compass_core::model::path_key(path.as_bytes())
            && e.target.as_deref() == Some(target_id)
            && e.resolution == Resolution::Resolved
    })
}

#[test]
fn independent_language_controls_and_byte_evidence() {
    let controls = [
        (
            "main.py",
            "# coração\r\ndef leaf():\r\n    return 1\r\ndef root():\r\n    return leaf()\r\n",
        ),
        (
            "main.rs",
            "// coração\nfn leaf() -> i32 { 1 }\nfn root() -> i32 { leaf() }\n",
        ),
        (
            "main.js",
            "// coração\nfunction leaf() { return 1; }\nfunction root() { return leaf(); }\n",
        ),
        (
            "main.ts",
            "// coração\nfunction leaf(): number { return 1; }\nfunction root(): number { return leaf(); }\n",
        ),
        (
            "main.jsx",
            "// coração\nfunction leaf() { return 1; }\nfunction root() { return <b>{leaf()}</b>; }\n",
        ),
        (
            "main.tsx",
            "// coração\nfunction leaf(): number { return 1; }\nfunction root() { return <b>{leaf()}</b>; }\n",
        ),
    ];
    for (path, source) in controls {
        let input = doc(path, source);
        let g = graph(std::slice::from_ref(&input));
        assert_eq!(g.coverage[0].parse_errors, 0, "{path}");
        assert!(
            call_to(&g, path, "leaf", target(&g, path, "leaf")),
            "{path}: {:?}",
            g.edges
        );
        let definition = g.nodes.iter().find(|n| n.name == "leaf").unwrap();
        assert_eq!(definition.start_line, 2);
        let code = &input.bytes[definition.start_byte..definition.end_byte];
        assert!(std::str::from_utf8(code).unwrap().contains("leaf"));
        let invocation = g
            .edges
            .iter()
            .find(|e| e.kind == Relation::Calls && e.name == "leaf")
            .unwrap();
        assert_eq!(
            &input.bytes[invocation.evidence.start_byte..invocation.evidence.end_byte],
            b"leaf"
        );
        assert_eq!(invocation.evidence.content_sha256, input.content_sha256);
        assert_eq!(
            invocation.evidence.start_line,
            if path.ends_with("py") { 5 } else { 3 }
        );
        let file = g.nodes.iter().find(|n| n.kind == "file").unwrap();
        assert!(g.edges.iter().any(|e| e.kind == Relation::Contains
            && e.source == file.id
            && e.target.as_deref() == Some(&definition.id)));
    }
}

#[test]
fn independent_import_alias_and_namespace_controls() {
    for (lib, lib_code, app, app_code, literal) in [
        (
            "lib.py",
            "def leaf():\n    return 1\n",
            "app.py",
            "from lib import leaf as work\ndef run():\n    return work()\n",
            "work",
        ),
        (
            "lib.py",
            "def leaf():\n    return 1\n",
            "app.py",
            "import lib as tools\ndef run():\n    return tools.leaf()\n",
            "tools.leaf",
        ),
        (
            "lib.js",
            "export function leaf() { return 1; }",
            "app.js",
            "import { leaf as work } from './lib'; export function run() { return work(); }",
            "work",
        ),
        (
            "lib.ts",
            "export function leaf(): number { return 1; }",
            "app.ts",
            "import * as tools from './lib'; export function run() { return tools.leaf(); }",
            "tools.leaf",
        ),
        (
            "src/util.rs",
            "pub fn leaf() -> i32 { 1 }",
            "src/lib.rs",
            "mod util; use crate::util::leaf; pub fn run() -> i32 { leaf() }",
            "leaf",
        ),
    ] {
        let g = graph(&[doc(lib, lib_code), doc(app, app_code)]);
        assert!(
            call_to(&g, app, literal, target(&g, lib, "leaf")),
            "{app}: {:?}",
            g.edges
        );
        let file_id = &g
            .nodes
            .iter()
            .find(|n| n.path == lib && n.kind == "file")
            .unwrap()
            .id;
        assert!(
            g.edges.iter().any(|e| e.kind == Relation::Imports
                && e.target.as_deref() == Some(file_id)
                && e.resolution == Resolution::Resolved),
            "{app}"
        );
    }
}

#[test]
fn revisions_invalidate_targets_and_order_is_deterministic() {
    let original = doc("lib.py", "def leaf():\n    return 1\n");
    let consumer = doc(
        "app.py",
        "from lib import leaf\ndef run():\n    return leaf()\n",
    );
    let first = graph(&[original.clone(), consumer.clone()]);
    let reversed = graph(&[consumer.clone(), original]);
    assert_eq!(
        serde_json::to_value(&first).unwrap(),
        serde_json::to_value(&reversed).unwrap()
    );
    let old_target = target(&first, "lib.py", "leaf");
    let changed = graph(&[doc("lib.py", "def renamed():\n    return 2\n"), consumer]);
    assert!(!changed.nodes.iter().any(|n| n.id == old_target));
    assert!(
        !changed
            .edges
            .iter()
            .any(|e| e.target.as_deref() == Some(old_target))
    );
    assert!(
        changed
            .edges
            .iter()
            .any(|e| e.kind == Relation::Calls && e.name == "leaf" && e.target.is_none())
    );
}

#[test]
fn ambiguity_dynamic_dispatch_shadowing_and_errors_stay_explicit() {
    for (path, code, call) in [
        (
            "a.py",
            "def leaf():\n    pass\ndef leaf():\n    pass\nleaf()\n",
            "leaf",
        ),
        (
            "a.py",
            "def leaf():\n    pass\ndef run(leaf: object):\n    leaf()\n",
            "leaf",
        ),
        ("a.py", "def run(obj):\n    return obj.work()\n", "obj.work"),
        (
            "a.js",
            "function leaf() {}\nleaf = external; leaf();",
            "leaf",
        ),
        ("a.rs", "fn run() { println!(\"x\"); }", "println"),
        (
            "a.py",
            "def leaf():\n    pass\ndef broken(:\nleaf()\n",
            "leaf",
        ),
        (
            "a.py",
            "def leaf():\n    pass\n@decorate\ndef changed():\n    pass\nchanged()\n",
            "changed",
        ),
    ] {
        let g = graph(&[doc(path, code)]);
        let found: Vec<_> = g
            .edges
            .iter()
            .filter(|e| e.kind == Relation::Calls && e.name == call)
            .collect();
        assert!(!found.is_empty(), "missing {path} {call}: {:?}", g.edges);
        assert!(
            found.iter().all(|e| e.target.is_none()
                && e.resolution != Resolution::Resolved
                && e.reason.is_some()),
            "{path}: {found:?}"
        );
    }
}

#[test]
fn unavailable_modules_private_exports_unsupported_and_invalid_inputs() {
    let g = graph(&[
        doc("lib.js", "function privateFn() {}"),
        doc(
            "a.js",
            "import { privateFn } from './lib'; import { external } from 'package'; privateFn(); external();",
        ),
        doc("a.go", "package main\nfunc leaf() {}"),
    ]);
    assert!(
        g.edges
            .iter()
            .filter(|e| e.kind == Relation::Calls)
            .all(|e| e.target.is_none())
    );
    assert!(
        g.coverage
            .iter()
            .any(|c| c.path_key == compass_core::model::path_key(b"a.go")
                && c.status == Resolution::Unavailable
                && c.reason.as_deref() == Some("unsupported_language"))
    );
    assert!(g.edges.iter().any(|e| e.kind == Relation::Imports
        && e.name == "package"
        && e.resolution == Resolution::Unavailable));
    let mut invalid = doc("a.py", "pass\n");
    invalid.bytes.push(b' ');
    assert!(structure::build(&[invalid], "x".into(), &AtomicBool::new(false)).is_err());
    let duplicate = doc("a.py", "pass\n");
    assert!(
        structure::build(
            &[duplicate.clone(), duplicate],
            "x".into(),
            &AtomicBool::new(false)
        )
        .is_err()
    );
}

#[test]
fn bounded_navigation_projection_and_cancellation() {
    let g = graph(&[doc(
        "a.py",
        "def leaf():\n    return 1\ndef root():\n    return leaf()\n",
    )]);
    let root = target(&g, "a.py", "root");
    let leaf = target(&g, "a.py", "leaf");
    let cancel = AtomicBool::new(false);
    let vicinity = g.neighbors(root, 1, 100, &cancel).unwrap();
    assert!(vicinity.nodes.iter().any(|n| n.id == leaf));
    let limited = g.neighbors(root, 3, 1, &cancel).unwrap();
    assert_eq!(limited.nodes.len(), 1);
    assert!(limited.edges.len() <= 1);
    assert!(limited.truncated);
    assert!(g.neighbors(root, 33, 100, &cancel).is_err());
    assert!(g.neighbors("missing", 1, 100, &cancel).is_err());
    let pair = if root < leaf {
        (root.to_owned(), leaf.to_owned())
    } else {
        (leaf.to_owned(), root.to_owned())
    };
    assert!(g.projection_pairs().contains(&pair));
    assert!(g.projection_pairs().iter().all(|(a, b)| a < b));
    let stopped = AtomicBool::new(true);
    assert_eq!(
        structure::build(&[], "x".into(), &stopped)
            .unwrap_err()
            .code,
        "cancelled"
    );
    assert_eq!(
        g.neighbors(root, 1, 100, &stopped).unwrap_err().code,
        "cancelled"
    );
    let roundtrip: Graph = serde_json::from_value(serde_json::to_value(&g).unwrap()).unwrap();
    assert_eq!(roundtrip.projection_pairs(), g.projection_pairs());
}

#[test]
fn lexical_scope_controls_do_not_promote_associated_or_shadowed_bindings() {
    for (path, code, name) in [
        (
            "a.rs",
            "struct X; impl X { fn leaf() {} fn run() { leaf(); } }",
            "leaf",
        ),
        (
            "a.js",
            "function leaf() {} const run = leaf => leaf();",
            "leaf",
        ),
        (
            "a.js",
            "function leaf() {} function run({leaf}) { leaf(); }",
            "leaf",
        ),
        (
            "a.py",
            "class X:\n    def leaf():\n        pass\n    def run():\n        leaf()\n",
            "leaf",
        ),
        (
            "a.py",
            "def inner():\n    import lib as tools\ndef outer():\n    tools.leaf()\n",
            "tools.leaf",
        ),
    ] {
        let g = graph(&[
            doc(path, code),
            doc("lib.py", "def leaf():\n    return 1\n"),
        ]);
        let calls: Vec<_> = g
            .edges
            .iter()
            .filter(|e| e.kind == Relation::Calls && e.name == name)
            .collect();
        assert!(!calls.is_empty(), "{path}");
        assert!(
            calls
                .iter()
                .all(|e| e.target.is_none() && e.reason.is_some()),
            "{path}: {calls:?}"
        );
    }
    let g = graph(&[
        doc("lib.py", "def leaf():\n    return 1\n"),
        doc(
            "a.py",
            "def leaf():\n    return 2\ndef run():\n    from lib import leaf\n    return leaf()\n",
        ),
    ]);
    assert!(call_to(&g, "a.py", "leaf", target(&g, "lib.py", "leaf")));
}

#[test]
fn relative_modules_respect_package_and_rust_module_boundaries() {
    for import in ["self::util::leaf", "super::foo::util::leaf"] {
        let g = graph(&[
            doc("src/lib.rs", "mod foo;"),
            doc(
                "src/foo.rs",
                &format!("use {import}; fn run() {{ leaf(); }}"),
            ),
            doc("src/foo/util.rs", "pub fn leaf() {}"),
            doc("src/util.rs", "pub fn leaf() {}"),
        ]);
        assert!(
            call_to(
                &g,
                "src/foo.rs",
                "leaf",
                target(&g, "src/foo/util.rs", "leaf")
            ),
            "{import}: {:?}",
            g.edges
        );
    }
    let g = graph(&[
        doc("src/lib.rs", "mod util;"),
        doc("src/util.rs", "pub fn leaf() {}"),
        doc(
            "src/bin/tool.rs",
            "use crate::util::leaf; fn run() { leaf(); }",
        ),
        doc("lib.py", "def leaf():\n    pass\n"),
        doc("a.py", "from .lib import leaf\nleaf()\n"),
        doc("lib.js", "export function leaf() {}"),
        doc("a.js", "import {leaf} from '../lib'; leaf();"),
    ]);
    for path in ["src/bin/tool.rs", "a.py", "a.js"] {
        assert!(
            g.edges
                .iter()
                .filter(|e| e.kind == Relation::Calls
                    && e.evidence.path_key == compass_core::model::path_key(path.as_bytes()))
                .all(|e| e.target.is_none()),
            "{path}"
        );
    }
    let g = graph(&[
        doc("pkg/lib.py", "def leaf():\n    pass\n"),
        doc("pkg/a.py", "from .lib import leaf\nleaf()\n"),
    ]);
    assert!(call_to(
        &g,
        "pkg/a.py",
        "leaf",
        target(&g, "pkg/lib.py", "leaf")
    ));
}

#[test]
fn native_path_identity_never_resolves_through_a_lossy_display_collision() {
    let valid = doc("x�.js", "export function leaf() { return 1; }");
    let mut native = doc("x�.js", "export function leaf() { return 2; }");
    native.path_key = compass_core::model::path_key(b"x\xff.js");
    let app = doc("app.js", "import {leaf} from './x�.js'; leaf();");
    let g = graph(&[valid.clone(), native.clone(), app.clone()]);
    let valid_target = g
        .nodes
        .iter()
        .find(|n| n.name == "leaf" && n.path_key == valid.path_key)
        .unwrap();
    assert!(call_to(&g, "app.js", "leaf", &valid_target.id));
    let native_nodes: Vec<_> = g
        .nodes
        .iter()
        .filter(|n| n.path_key == native.path_key)
        .collect();
    assert_eq!(native_nodes.len(), 1);
    assert_eq!(native_nodes[0].kind, "file");
    let coverage = g
        .coverage
        .iter()
        .find(|c| c.path_key == native.path_key)
        .unwrap();
    assert_eq!(coverage.status, Resolution::Unavailable);
    assert_eq!(coverage.reason.as_deref(), Some("native_path_not_utf8"));
    assert_eq!(coverage.symbols, 0);
    assert!(
        !g.edges
            .iter()
            .any(|e| e.target.as_deref() == Some(&native_nodes[0].id))
    );

    let missing = graph(&[native, app]);
    let call = missing
        .edges
        .iter()
        .find(|e| e.kind == Relation::Calls && e.name == "leaf")
        .unwrap();
    assert!(call.target.is_none());
    assert_ne!(call.resolution, Resolution::Resolved);
    assert!(
        missing
            .edges
            .iter()
            .filter(|e| e.kind == Relation::Imports)
            .all(|e| e.target.is_none())
    );

    let mut mismatch = doc("display.py", "def leaf():\n    return 3\n");
    mismatch.path_key = compass_core::model::path_key(b"actual.py");
    let g = graph(&[mismatch]);
    assert_eq!(g.nodes.len(), 1);
    assert_eq!(g.coverage[0].status, Resolution::Unavailable);
    assert_eq!(
        g.coverage[0].reason.as_deref(),
        Some("path_identity_mismatch")
    );

    let mut native_python = doc("x�.py", "def leaf():\n    return 4\n");
    native_python.path_key = compass_core::model::path_key(b"x\xff.py");
    let g = graph(&[
        native_python,
        doc("x�.py", "def valid_leaf():\n    return 5\n"),
    ]);
    assert_eq!(g.nodes.iter().filter(|n| n.kind != "file").count(), 1);
    assert!(g.nodes.iter().any(|n| n.name == "valid_leaf"));
}
