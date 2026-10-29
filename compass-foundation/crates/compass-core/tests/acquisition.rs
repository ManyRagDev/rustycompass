use compass_core::{
    acquisition, codec,
    collection::{Collection, PrepareRequest},
    model,
    retrieval::{self, RetrieveRequest},
};
use serde_json::{Value, json};
use std::{fs, sync::atomic::AtomicBool};
fn flag() -> AtomicBool {
    AtomicBool::new(false)
}
fn fixture(files: &[(&str, &str)]) -> (tempfile::TempDir, tempfile::TempDir, Collection) {
    let root = tempfile::tempdir().unwrap();
    let store = tempfile::tempdir().unwrap();
    for (path, text) in files {
        fs::write(root.path().join(path), text).unwrap();
    }
    retrieval::prepare(
        &PrepareRequest {
            root: root.path().into(),
            store: store.path().into(),
            max_disk_bytes: 64 * 1024 * 1024,
        },
        None,
        &flag(),
    )
    .unwrap();
    let c = Collection::open(store.path(), None).unwrap();
    (root, store, c)
}
fn request(budget: usize) -> RetrieveRequest {
    RetrieveRequest {
        query: "purpose".into(),
        mode: "hybrid".into(),
        expansion: "none".into(),
        theta: 1.0,
        budget_bytes: budget,
        cursor: None,
    }
}
fn contract(obligations: Value) -> Value {
    json!({"schema_version":"1","intent":"inspect exact evidence","obligations":obligations,"interpretation_pending":[]})
}
fn definition(name: &str) -> Value {
    json!({"id":name,"kind":"symbol_definition","symbol":{"name":name}})
}
fn sufficient(pack: &Value) -> bool {
    pack["completion"]["status"] == "SUFFICIENT_EVIDENCE_ACQUIRED"
}
fn verify_pack(pack: &Value, budget: usize) {
    assert_eq!(pack["schema_version"], "2");
    assert_eq!(pack["payload_bytes"], codec::canonical(pack).unwrap().len());
    assert!(codec::canonical(pack).unwrap().len() <= budget);
    for o in pack["completion"]["obligations"].as_array().unwrap() {
        for w in o["witnesses"].as_array().unwrap() {
            let item = pack["items"]
                .as_array()
                .unwrap()
                .iter()
                .find(|i| i["span_id"] == w["span_id"])
                .unwrap();
            assert!(w["start_byte"].as_u64().unwrap() >= item["start_byte"].as_u64().unwrap());
            assert!(w["end_byte"].as_u64().unwrap() <= item["end_byte"].as_u64().unwrap());
            assert!(w["start_byte"].as_u64().unwrap() < w["end_byte"].as_u64().unwrap());
        }
    }
}
#[test]
fn full_syntax_spans_include_wrappers_and_long_bodies() {
    let long = format!(
        "export function lengthy() {{\n{}  return 42;\n}}\n",
        (0..220)
            .map(|n| format!("  // line {n}\n"))
            .collect::<String>()
    );
    let (_root, _store, c) = fixture(&[
        ("long.ts", &long),
        ("wrapped.py", "@decorate\ndef wrapped():\n    return 3\n"),
        ("attrs.rs", "#[inline]\npub fn attributed() -> i32 { 9 }\n"),
    ]);
    let contract = contract(json!([
        definition("lengthy"),
        definition("wrapped"),
        definition("attributed")
    ]));
    let pack = acquisition::retrieve(&c, &request(32768), &contract, false, &flag()).unwrap();
    assert!(sufficient(&pack), "{pack}");
    verify_pack(&pack, 32768);
    let texts = pack["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["text"].as_str().unwrap())
        .collect::<String>();
    assert!(texts.contains(long.trim_end()));
    assert!(texts.contains("@decorate\ndef wrapped()"));
    assert!(texts.contains("#[inline]\npub fn attributed"));
}
#[test]
fn lexical_hit_and_partial_pack_cannot_satisfy_complete_definition() {
    let source = format!(
        "function target() {{\n{}  return 'end';\n}}\n",
        (0..260)
            .map(|n| format!("  // target {n} {}\n", "x".repeat(35)))
            .collect::<String>()
    );
    let (_root, _store, c) = fixture(&[("long.js", &source)]);
    let contract = contract(json!([definition("target")]));
    let mut req = request(4096);
    let mut pages = 0;
    let mut bytes = 0;
    loop {
        let pack = acquisition::retrieve(&c, &req, &contract, false, &flag()).unwrap();
        verify_pack(&pack, 4096);
        assert!(!sufficient(&pack));
        bytes += pack["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["text"].as_str().unwrap().len())
            .sum::<usize>();
        pages += 1;
        assert!(pages < 40, "pagination did not progress");
        match pack["next_cursor"].as_str() {
            Some(cursor) => req.cursor = Some(cursor.into()),
            None => break,
        }
    }
    assert!(pages > 1);
    assert_eq!(bytes, source.trim_end().len());
}
#[test]
fn missing_ambiguous_unknown_and_pending_fail_closed() {
    let (_root, _store, c) = fixture(&[
        ("one.py", "def repeated():\n    return 1\n"),
        ("two.py", "def repeated():\n    return 2\n"),
    ]);
    let v = contract(
        json!([definition("repeated"),definition("absent"),{"id":"unknown","kind":"dataflow","query":"anything"}]),
    );
    let pack = acquisition::retrieve(&c, &request(8192), &v, false, &flag()).unwrap();
    assert!(!sufficient(&pack));
    let o = pack["completion"]["obligations"].as_array().unwrap();
    assert_eq!(o[0]["reason"], "ambiguous_symbol");
    assert_eq!(o[1]["reason"], "symbol_not_found");
    assert_eq!(o[2]["status"], "unsupported");
    let mut pending = contract(
        json!([{"id":"one","kind":"symbol_definition","symbol":{"name":"repeated","path_key":model::path_key(b"one.py")}}]),
    );
    pending["interpretation_pending"] = json!(["explain intent"]);
    let pack = acquisition::retrieve(&c, &request(8192), &pending, false, &flag()).unwrap();
    assert!(!sufficient(&pack));
    assert_eq!(pack["completion"]["obligations"][0]["status"], "satisfied");
}
#[test]
fn literal_scope_is_exact_even_when_outside_text_shares_a_line() {
    let (_root, _store, c) = fixture(&[(
        "scope.js",
        "function target(){return 1;} const outside='MARKER';\nfunction actual(){return 'MARKER';}\n",
    )]);
    let v = contract(
        json!([{"id":"scoped","kind":"literal_occurrence","literal":"MARKER","symbol":{"name":"target"}}]),
    );
    let pack = acquisition::retrieve(&c, &request(8192), &v, false, &flag()).unwrap();
    assert!(!sufficient(&pack));
    assert_eq!(
        pack["completion"]["obligations"][0]["reason"],
        "literal_not_found"
    );
    let v = contract(
        json!([{"id":"scoped","kind":"literal_occurrence","literal":"MARKER","symbol":{"name":"actual"}}]),
    );
    assert!(sufficient(
        &acquisition::retrieve(&c, &request(8192), &v, false, &flag()).unwrap()
    ));
}
#[test]
fn literal_preserves_multiline_crlf_and_unicode() {
    let source = "// before\r\nconst message = `Olá\r\n世界`;\r\n// after\r\n";
    let (_root, _store, c) = fixture(&[("literal.js", source)]);
    let v = contract(
        json!([{"id":"literal","kind":"literal_occurrence","literal":"Olá\r\n世界","path_key":model::path_key(b"literal.js")} ]),
    );
    let pack = acquisition::retrieve(&c, &request(8192), &v, false, &flag()).unwrap();
    assert!(sufficient(&pack));
    assert_eq!(
        pack["items"][0]["text"],
        "const message = `Olá\r\n世界`;\r\n"
    );
}
#[test]
fn direct_relations_are_directed_and_need_current_witnesses() {
    let (_root, _store, c) = fixture(&[(
        "calls.py",
        "def leaf():\n    return 1\ndef bridge():\n    return leaf()\ndef caller():\n    return bridge()\n",
    )]);
    let relation = |source: &str, target: &str| {
        contract(
            json!([{"id":"edge","kind":"resolved_relation","source":{"name":source},"target":{"name":target},"relation":"calls"}]),
        )
    };
    let pack = acquisition::retrieve(
        &c,
        &request(8192),
        &relation("bridge", "leaf"),
        false,
        &flag(),
    )
    .unwrap();
    assert!(sufficient(&pack), "{pack}");
    verify_pack(&pack, 8192);
    let text = pack["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["text"].as_str().unwrap())
        .collect::<String>();
    assert!(text.contains("return leaf()"));
    assert!(text.contains("def leaf()"));
    assert!(text.contains("def bridge()"));
    for (source, target) in [("leaf", "bridge"), ("caller", "leaf")] {
        let pack = acquisition::retrieve(
            &c,
            &request(8192),
            &relation(source, target),
            false,
            &flag(),
        )
        .unwrap();
        assert!(!sufficient(&pack));
    }
}
#[test]
fn parse_errors_cannot_be_proven_by_contains_edges() {
    let (_root, _store, c) = fixture(&[("broken.py", "def broken():\n    if (\n    return 2\n")]);
    let v = contract(json!([definition("broken")]));
    let pack = acquisition::retrieve(&c, &request(8192), &v, false, &flag()).unwrap();
    assert!(!sufficient(&pack));
}
#[test]
fn exact_selectors_bypass_two_hundred_candidates_and_deduplicate() {
    let source = (0..240)
        .map(|i| format!("function marker{i}() {{ return {i}; }}\n"))
        .collect::<String>();
    let (_root, _store, c) = fixture(&[("many.js", &source)]);
    let v = contract(
        json!([definition("marker239"),{"id":"literal","kind":"literal_occurrence","literal":"return 239","symbol":{"name":"marker239"}}]),
    );
    let pack = acquisition::retrieve(&c, &request(8192), &v, false, &flag()).unwrap();
    assert!(sufficient(&pack));
    assert_eq!(pack["items"].as_array().unwrap().len(), 1);
}
#[test]
fn mandatory_evidence_precedes_optional_diffusion() {
    let (_root, _store, c) = fixture(&[(
        "heat.py",
        "def leaf():\n    return 1\ndef caller():\n    return leaf()\n",
    )]);
    let v = contract(json!([definition("caller")]));
    for policy in ["none", "bfs", "heat"] {
        let mut req = request(8192);
        req.expansion = policy.into();
        let pack = acquisition::retrieve(&c, &req, &v, false, &flag()).unwrap();
        assert!(sufficient(&pack));
        assert_eq!(pack["expansion"]["method"], policy);
        if policy != "none" {
            assert!(pack["expansion"]["edge_visits"].is_number());
        }
    }
}
#[test]
fn cursor_binds_contract_request_and_generation() {
    let source = format!(
        "function target() {{\n{}\n}}\n",
        "// long body\n".repeat(2000)
    );
    let (root, store, c) = fixture(&[("long.js", &source)]);
    let v = contract(json!([definition("target")]));
    let mut req = request(4096);
    let first = acquisition::retrieve(&c, &req, &v, false, &flag()).unwrap();
    req.cursor = Some(first["next_cursor"].as_str().unwrap().into());
    assert_eq!(
        acquisition::cursor_generation(req.cursor.as_ref().unwrap()).unwrap(),
        c.generation_id()
    );
    let mut changed = v.clone();
    changed["intent"] = json!("changed");
    assert!(acquisition::retrieve(&c, &req, &changed, false, &flag()).is_err());
    let mut changed = req.clone();
    changed.mode = "literal".into();
    assert!(acquisition::retrieve(&c, &changed, &v, false, &flag()).is_err());
    fs::write(root.path().join("new.js"), "function newSymbol() {}\n").unwrap();
    retrieval::update(store.path(), &flag()).unwrap();
    let next = Collection::open(store.path(), None).unwrap();
    assert!(acquisition::retrieve(&next, &req, &v, false, &flag()).is_err());
    assert!(acquisition::retrieve(&c, &req, &v, false, &flag()).is_ok());
}
#[test]
fn node_ids_require_explicit_generation_and_corruption_cancel_fail_closed() {
    let (_root, _store, c) = fixture(&[("one.py", "def target():\n    return 1\n")]);
    let nodes = retrieval::neighbors(&c, None, Some("target"), 1, 20, &flag()).unwrap();
    let id = nodes["nodes"][0]["id"].as_str().unwrap();
    let v = contract(json!([{"id":"id","kind":"symbol_definition","symbol":{"node_id":id}}]));
    assert!(acquisition::validate_contract(&v, false).is_err());
    assert!(sufficient(
        &acquisition::retrieve(&c, &request(8192), &v, true, &flag()).unwrap()
    ));
    assert!(acquisition::retrieve(&c, &request(8192), &v, true, &AtomicBool::new(true)).is_err());
    let d = c
        .verified_document(&model::path_key(b"one.py"), &flag())
        .unwrap();
    let object = c
        .pinned_generation_dir()
        .join("source/objects")
        .join(&d.content_sha256);
    assert!(object.exists(), "{object:?}");
    fs::write(object, "tampered").unwrap();
    assert!(acquisition::retrieve(&c, &request(8192), &v, true, &flag()).is_err());
}
#[test]
fn malformed_contract_rejected_and_unfittable_line_is_omitted() {
    let mut v = contract(json!([definition("target")]));
    v["obligations"][0]["extra"] = json!(true);
    assert!(acquisition::validate_contract(&v, false).is_err());
    let huge = format!("function target() {{return '{}';}}\n", "x".repeat(12000));
    let (_root, _store, c) = fixture(&[("huge.js", &huge)]);
    let v = contract(json!([definition("target")]));
    let pack = acquisition::retrieve(&c, &request(2048), &v, false, &flag()).unwrap();
    assert!(!sufficient(&pack));
    assert!(pack["items"].as_array().unwrap().is_empty());
    assert_eq!(pack["omitted"].as_array().unwrap().len(), 1);
    assert!(pack["next_cursor"].is_null());
    verify_pack(&pack, 2048);
}
#[test]
fn exact_definition_does_not_require_unrelated_same_line_text() {
    let source = format!(
        "const preceding='{}'; function target() {{return 1;}} const unrelated='{}';\n",
        "p".repeat(10000),
        "x".repeat(20000)
    );
    let (_root, _store, c) = fixture(&[("same.js", &source)]);
    let v = contract(json!([definition("target")]));
    let pack = acquisition::retrieve(&c, &request(2048), &v, false, &flag()).unwrap();
    assert!(sufficient(&pack), "{pack}");
    assert_eq!(pack["items"][0]["text"], "function target() {return 1;}");
    verify_pack(&pack, 2048);
}
#[test]
fn full_union_is_fitted_before_incomplete_metadata() {
    let (_root, _store, c) = fixture(&[("one.py", "def target():\n    return 1\n")]);
    let v = contract(json!([definition("target")]));
    let mut req = request(65536);
    req.query = "q".repeat(700);
    let large = acquisition::retrieve(&c, &req, &v, false, &flag()).unwrap();
    req.budget_bytes = codec::canonical(&large).unwrap().len().max(2048);
    let exact = acquisition::retrieve(&c, &req, &v, false, &flag()).unwrap();
    assert!(sufficient(&exact));
    verify_pack(&exact, req.budget_bytes);
}
#[test]
fn complete_relation_range_can_fit_when_its_prefix_cannot() {
    // Completing the child witness removes omitted-tail metadata. That can make
    // the complete range smaller than an incomplete prefix, so prefix fitting
    // alone is not monotonic at this boundary (release CLI reproduction).
    let parent = format!("Parent{}", "X".repeat(1000));
    let child = "    def child(self):\n        # body marker AAAAA\n        # body marker BBBBB\n        return 1\n";
    let source = format!(
        "class {parent}:\n{}{child}",
        "    # intervening comment\n".repeat(30)
    );
    let (_root, _store, c) = fixture(&[("test.py", &source)]);
    let v = json!({"schema_version":"1","intent":"tail discontinuity probe","obligations":[{"id":"r","kind":"resolved_relation","source":{"name":parent},"target":{"name":"child"},"relation":"contains"}],"interpretation_pending":[]});
    let mut req = request(3741);
    req.query = "probe".into();
    let pack = acquisition::retrieve(&c, &req, &v, false, &flag()).unwrap();
    assert!(!sufficient(&pack));
    assert_eq!(
        pack["completion"]["obligations"][0]["status"], "partial",
        "{pack}"
    );
    assert_eq!(pack["items"].as_array().unwrap().len(), 1);
    assert_eq!(pack["items"][0]["text"], child);
    assert!(pack["next_cursor"].is_string());
    verify_pack(&pack, 3741);
}
