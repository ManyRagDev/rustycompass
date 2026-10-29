//! Independent controls for the new explicit acquisition contract.
use compass_core::{
    acquisition, codec,
    collection::{Collection, PrepareRequest},
    model,
    retrieval::{self, RetrieveRequest},
};
use serde_json::{Value, json};
use std::{fs, sync::atomic::AtomicBool};
fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn fixture(files: &[(&str, &str)]) -> (tempfile::TempDir, tempfile::TempDir, Collection) {
    let root = tempfile::tempdir().unwrap();
    let store = tempfile::tempdir().unwrap();
    for (name, contents) in files {
        fs::write(root.path().join(name), contents).unwrap();
    }
    retrieval::prepare(
        &PrepareRequest {
            root: root.path().into(),
            store: store.path().into(),
            max_disk_bytes: 64 * 1024 * 1024,
        },
        None,
        &cancel(),
    )
    .unwrap();
    let collection = Collection::open(store.path(), None).unwrap();
    (root, store, collection)
}
fn contract(obligations: Value) -> Value {
    json!({"schema_version":"1","intent":"independent review","obligations":obligations,"interpretation_pending":[]})
}
fn definition(id: &str, name: &str) -> Value {
    json!({"id":id,"kind":"symbol_definition","symbol":{"name":name}})
}
fn req(budget: usize) -> RetrieveRequest {
    RetrieveRequest {
        query: "irrelevant ranking input".into(),
        mode: "literal".into(),
        expansion: "none".into(),
        theta: 1.0,
        budget_bytes: budget,
        cursor: None,
    }
}
fn sufficient(v: &Value) -> bool {
    v["completion"]["status"] == "SUFFICIENT_EVIDENCE_ACQUIRED"
}
fn acquire(c: &Collection, r: &RetrieveRequest, v: &Value) -> Value {
    acquisition::retrieve(c, r, v, false, &cancel()).unwrap()
}
#[test]
fn exact_complete_pack_fits_at_its_measured_boundary() {
    let text = (0..8)
        .map(|i| format!("function boundary{i}() {{ return {i}; }}\n"))
        .collect::<String>();
    let (_root, _store, c) = fixture(&[("boundary.js", &text)]);
    let v = contract(json!(
        (0..8)
            .map(|i| definition(&format!("d{i}"), &format!("boundary{i}")))
            .collect::<Vec<_>>()
    ));
    let mut reference = acquire(&c, &req(65536), &v);
    assert!(sufficient(&reference));
    // Compute the canonical size of the identical all-evidence response after
    // changing only the requested budget; no empty-page estimate is an oracle.
    let mut boundary = codec::canonical(&reference).unwrap().len().max(2048);
    for _ in 0..8 {
        reference["budget_bytes"] = json!(boundary);
        reference["payload_bytes"] = json!(codec::canonical(&reference).unwrap().len());
        let next = codec::canonical(&reference).unwrap().len().max(2048);
        if boundary == next {
            break;
        }
        boundary = next;
    }
    let pack = acquire(&c, &req(boundary), &v);
    assert!(
        sufficient(&pack),
        "a complete valid response fits in {boundary} bytes: {pack}"
    );
    assert!(codec::canonical(&pack).unwrap().len() <= boundary);
}
#[test]
fn prior_page_evidence_never_certifies_later_page() {
    let long = format!(
        "function large() {{\n{}\n}}\n",
        "  // some original source bytes for pagination\n".repeat(400)
    );
    let (_root, _store, c) = fixture(&[
        ("large.js", &long),
        ("small.js", "function small() { return 1; }\n"),
    ]);
    let v = contract(json!([
        definition("small", "small"),
        definition("large", "large")
    ]));
    let mut r = req(5000);
    let mut pages = 0;
    let mut earlier_satisfied = false;
    loop {
        let page = acquire(&c, &r, &v);
        assert!(!sufficient(&page), "{page}");
        assert_eq!(page["completion"]["scope"], "current_pack");
        let small = &page["completion"]["obligations"][0];
        if pages == 0 {
            assert_eq!(small["status"], "satisfied");
            earlier_satisfied = true;
        } else {
            assert_ne!(small["status"], "satisfied");
            assert!(small["witnesses"].as_array().unwrap().is_empty());
        }
        assert!(!page["omitted"].as_array().unwrap().is_empty());
        for omission in page["omitted"].as_array().unwrap() {
            assert!(omission["content_sha256"].as_str().is_some(), "{omission}");
            assert!(
                omission["start_byte"].as_u64().unwrap() < omission["end_byte"].as_u64().unwrap()
            );
        }
        pages += 1;
        assert!(pages < 40);
        if let Some(cursor) = page["next_cursor"].as_str() {
            r.cursor = Some(cursor.into());
        } else {
            break;
        }
    }
    assert!(earlier_satisfied && pages > 1);
}
#[test]
fn scoped_literals_and_ambiguous_names_fail_closed() {
    let (_root, _store, c) = fixture(&[
        (
            "scope.js",
            "function selected(){return 1;} const leaked='OUTSIDE';\nfunction actual(){return 'INSIDE';}\n",
        ),
        ("one.py", "def duplicate():\n    return 1\n"),
        ("two.py", "def duplicate():\n    return 2\n"),
    ]);
    for v in [
        contract(
            json!([{"id":"scope","kind":"literal_occurrence","literal":"OUTSIDE","symbol":{"name":"selected"}}]),
        ),
        contract(json!([definition("ambiguous", "duplicate")])),
    ] {
        assert!(!sufficient(&acquire(&c, &req(8192), &v)));
    }
    let mut v = contract(
        json!([{"id":"exact","kind":"symbol_definition","symbol":{"name":"duplicate","path_key":model::path_key(b"two.py"),"scope":[]}}]),
    );
    assert!(sufficient(&acquire(&c, &req(8192), &v)));
    v["interpretation_pending"] = json!(["determine whether this is the correct implementation"]);
    assert!(!sufficient(&acquire(&c, &req(8192), &v)));
}
#[test]
fn direct_relation_and_parse_error_controls_use_actual_witnesses() {
    let (_root, _store, c) = fixture(&[
        (
            "graph.py",
            "def leaf():\n    return 1\ndef middle():\n    return leaf()\ndef entry():\n    return middle()\n",
        ),
        ("broken.py", "def broken():\n    return (\n"),
    ]);
    let relation = |source: &str, target: &str| {
        contract(
            json!([{"id":"r","kind":"resolved_relation","source":{"name":source},"target":{"name":target},"relation":"calls"}]),
        )
    };
    assert!(sufficient(&acquire(
        &c,
        &req(8192),
        &relation("middle", "leaf")
    )));
    for pair in [("leaf", "middle"), ("entry", "leaf")] {
        assert!(!sufficient(&acquire(
            &c,
            &req(8192),
            &relation(pair.0, pair.1)
        )));
    }
    assert!(!sufficient(&acquire(
        &c,
        &req(8192),
        &contract(json!([definition("bad", "broken")]))
    )));
}
#[test]
fn replaced_object_links_and_preexisting_cancellation_never_return_proof() {
    let (_root, _store, c) = fixture(&[("good.py", "def good():\n    return 1\n")]);
    let v = contract(json!([definition("good", "good")]));
    assert!(sufficient(&acquire(&c, &req(8192), &v)));
    let err = acquisition::retrieve(&c, &req(8192), &v, false, &AtomicBool::new(true)).unwrap_err();
    assert_eq!(err.code, "cancelled");
    let document = c
        .verified_document(&model::path_key(b"good.py"), &cancel())
        .unwrap();
    let object = c
        .pinned_generation_dir()
        .join("source/objects")
        .join(document.content_sha256);
    let other = tempfile::NamedTempFile::new().unwrap();
    fs::write(other.path(), document.bytes).unwrap();
    fs::remove_file(&object).unwrap();
    std::os::unix::fs::symlink(other.path(), &object).unwrap();
    assert!(
        acquisition::retrieve(&c, &req(8192), &v, false, &cancel()).is_err(),
        "even identical bytes through a replaced symlink are not an object proof"
    );
}

#[test]
fn small_definition_ignores_unrelated_bytes_on_its_source_line() {
    let selected = "function selected() { return 42; }";
    let text = format!(
        "const before = '{}'; {selected} const after = '{}';\n",
        "x".repeat(12000),
        "y".repeat(12000)
    );
    let (_root, _store, c) = fixture(&[("same-line.js", &text)]);
    let v = contract(json!([definition("small", "selected")]));
    let pack = acquire(&c, &req(3000), &v);
    assert!(sufficient(&pack), "{pack}");
    assert_eq!(pack["items"].as_array().unwrap().len(), 1);
    assert_eq!(pack["items"][0]["text"], selected);
    assert!(pack["items"][0]["start_byte"].as_u64().unwrap() > 12000);
}

#[test]
fn node_id_is_explicit_and_generation_specific() {
    let (root, store, c) = fixture(&[("identity.py", "def target():\n    return 1\n")]);
    let neighbors = retrieval::neighbors(&c, None, Some("target"), 0, 20, &cancel()).unwrap();
    let id = neighbors["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["name"] == "target")
        .unwrap()["id"]
        .clone();
    let v = contract(json!([{"id":"identity","kind":"symbol_definition","symbol":{"node_id":id}}]));
    assert!(acquisition::retrieve(&c, &req(8192), &v, false, &cancel()).is_err());
    assert!(sufficient(
        &acquisition::retrieve(&c, &req(8192), &v, true, &cancel()).unwrap()
    ));
    fs::write(
        root.path().join("identity.py"),
        "def target():\n    return 2\n",
    )
    .unwrap();
    retrieval::update(store.path(), &cancel()).unwrap();
    let latest = Collection::open(store.path(), None).unwrap();
    let current = acquisition::retrieve(&latest, &req(8192), &v, true, &cancel()).unwrap();
    assert!(
        !sufficient(&current),
        "old content identity must not silently resolve in another generation: {current}"
    );
    assert!(sufficient(
        &acquisition::retrieve(&c, &req(8192), &v, true, &cancel()).unwrap()
    ));
}

#[test]
fn paginated_definition_can_start_midline_and_stops_at_syntax_end() {
    let definition = format!(
        "function selected() {{\n{}  return 7;\n}}",
        "  // original required body line\n".repeat(300)
    );
    let text = format!("const prefix = 1; {definition} const suffix = 'unrelated';\n");
    let (_root, _store, c) = fixture(&[("midline.js", &text)]);
    let v = contract(json!([self::definition("midline", "selected")]));
    let mut request = req(3000);
    let mut received = String::new();
    let mut pages = 0;
    loop {
        let page = acquire(&c, &request, &v);
        assert!(!sufficient(&page));
        for item in page["items"].as_array().unwrap() {
            received.push_str(item["text"].as_str().unwrap());
        }
        pages += 1;
        assert!(pages < 40);
        if let Some(cursor) = page["next_cursor"].as_str() {
            request.cursor = Some(cursor.into());
        } else {
            break;
        }
    }
    assert!(pages > 1);
    assert_eq!(received, definition);
}

#[test]
fn complete_individual_witness_survives_partial_tail_metadata_discontinuity() {
    let name = format!("Parent{}", "X".repeat(1000));
    let child = "    def child(self):\n        # body marker AAAAA\n        # body marker BBBBB\n        return 1\n";
    let text = format!(
        "class {name}:\n{}{}",
        "    # intervening comment\n".repeat(30),
        child
    );
    let (_root, _store, c) = fixture(&[("test.py", &text)]);
    let v = json!({"schema_version":"1","intent":"tail discontinuity probe","obligations":[{"id":"r","kind":"resolved_relation","source":{"name":name},"target":{"name":"child"},"relation":"contains"}],"interpretation_pending":[]});
    let mut request = req(3741);
    request.query = "probe".into();
    request.mode = "hybrid".into();
    let pack = acquire(&c, &request, &v);
    assert!(!sufficient(&pack));
    assert_eq!(
        pack["completion"]["obligations"][0]["status"], "partial",
        "{pack}"
    );
    assert!(
        pack["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["text"].as_str().unwrap().contains(child)),
        "whole child witness fits despite an intermediate partial candidate's omitted-tail overhead: {pack}"
    );
    assert!(codec::canonical(&pack).unwrap().len() <= 3741);
}
