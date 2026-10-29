//! Independent controls for source evidence, budget accounting and continuation.
use compass_core::{
    codec,
    collection::{Collection, PrepareRequest},
    retrieval::{self, RetrieveRequest},
};
use serde_json::Value;
use std::{collections::BTreeSet, fs, os::unix::fs::symlink, sync::atomic::AtomicBool};
use tempfile::TempDir;

fn flag() -> AtomicBool {
    AtomicBool::new(false)
}
fn prepare(root: &TempDir, store: &TempDir) {
    retrieval::prepare(
        &PrepareRequest {
            root: root.path().into(),
            store: store.path().into(),
            max_disk_bytes: 32 * 1024 * 1024,
        },
        None,
        &flag(),
    )
    .unwrap();
}
fn request(budget: usize) -> RetrieveRequest {
    RetrieveRequest {
        query: "needle".into(),
        mode: "lexical".into(),
        expansion: "none".into(),
        theta: 1.0,
        budget_bytes: budget,
        cursor: None,
    }
}
fn assert_pack(c: &Collection, pack: &Value, budget: usize) {
    let size = codec::canonical(pack).unwrap().len();
    assert!(size <= budget);
    assert_eq!(pack["payload_bytes"].as_u64().unwrap() as usize, size);
    for item in pack["items"].as_array().unwrap() {
        let document = c
            .verified_document(item["path_key"].as_str().unwrap(), &flag())
            .unwrap();
        assert_eq!(item["content_sha256"], document.content_sha256);
        let start = item["start_byte"].as_u64().unwrap() as usize;
        let end = item["end_byte"].as_u64().unwrap() as usize;
        assert_eq!(
            item["text"].as_str().unwrap().as_bytes(),
            &document.bytes[start..end]
        );
        assert_eq!(item["verification"], "full_object_sha256");
    }
}

#[test]
fn exact_pack_bytes_evidence_and_generation_bound_continuation() {
    let root = TempDir::new().unwrap();
    let store = TempDir::new().unwrap();
    for i in 0..15 {
        fs::write(
            root.path().join(format!("file{i:02}.py")),
            format!("# coração needle {i}\r\ndef work{i}():\r\n    return {i}\r\n"),
        )
        .unwrap();
    }
    prepare(&root, &store);
    let old = Collection::open(store.path(), None).unwrap();
    let mut r = request(4096);
    let first = retrieval::retrieve(&old, &r, &flag()).unwrap();
    assert_pack(&old, &first, r.budget_bytes);
    let first_cursor = first["next_cursor"]
        .as_str()
        .expect("fixture must need continuation")
        .to_owned();
    let mut seen = BTreeSet::new();
    let mut pack = first;
    loop {
        assert_pack(&old, &pack, r.budget_bytes);
        for item in pack["items"]
            .as_array()
            .unwrap()
            .iter()
            .chain(pack["omitted"].as_array().unwrap())
        {
            assert!(
                seen.insert((
                    item["path_key"].as_str().unwrap().to_owned(),
                    item["start_byte"].as_u64().unwrap()
                )),
                "repeated candidate across pages"
            );
        }
        match pack["next_cursor"].as_str() {
            Some(cursor) => {
                r.cursor = Some(cursor.into());
                pack = retrieval::retrieve(&old, &r, &flag()).unwrap();
            }
            None => break,
        }
    }
    assert_eq!(seen.len(), 15);
    let mut changed = request(4096);
    changed.cursor = Some(first_cursor.clone());
    changed.query = "different".into();
    assert!(retrieval::retrieve(&old, &changed, &flag()).is_err());
    fs::write(root.path().join("file00.py"), "# needle changed\n").unwrap();
    retrieval::update(store.path(), &flag()).unwrap();
    let current = Collection::open(store.path(), None).unwrap();
    let mut continued = request(4096);
    continued.cursor = Some(first_cursor);
    assert!(retrieval::retrieve(&current, &continued, &flag()).is_err());
    let stable = retrieval::retrieve(&old, &continued, &flag()).unwrap();
    assert_pack(&old, &stable, continued.budget_bytes);
    assert_eq!(stable["generation"], old.generation_id());
    assert_ne!(stable["generation"], current.generation_id());
}

#[test]
fn oversize_excerpt_remains_explicitly_omitted_with_usable_reference() {
    let root = TempDir::new().unwrap();
    let store = TempDir::new().unwrap();
    fs::write(
        root.path().join("wide.py"),
        format!("# needle {}\n", "λ".repeat(12_000)),
    )
    .unwrap();
    prepare(&root, &store);
    let c = Collection::open(store.path(), None).unwrap();
    let r = request(4096);
    let pack = retrieval::retrieve(&c, &r, &flag()).unwrap();
    assert_pack(&c, &pack, r.budget_bytes);
    assert!(pack["items"].as_array().unwrap().is_empty());
    let omitted = &pack["omitted"][0];
    assert!(matches!(
        omitted["reason"].as_str(),
        Some("budget" | "budget_cannot_fit_complete_line")
    ));
    let document = c
        .verified_document(omitted["path_key"].as_str().unwrap(), &flag())
        .unwrap();
    assert_eq!(omitted["content_sha256"], document.content_sha256);
    assert!(omitted["end_byte"].as_u64().unwrap() > 20_000);
    assert!(pack["next_cursor"].is_null());
}

#[test]
fn pinned_generation_survives_ancestor_substitution() {
    let root = TempDir::new().unwrap();
    let store = TempDir::new().unwrap();
    fs::write(
        root.path().join("a.py"),
        "# needle\ndef leaf():\n    return 1\n",
    )
    .unwrap();
    prepare(&root, &store);
    let c = Collection::open(store.path(), None).unwrap();
    let baseline = retrieval::query(&c, "needle", "lexical", 10, 0, &flag()).unwrap();
    let generation = c.generation_dir().to_path_buf();
    let saved = generation.with_extension("saved");
    fs::rename(&generation, &saved).unwrap();
    let counterfeit = TempDir::new().unwrap();
    fs::write(
        counterfeit.path().join("derived.json"),
        "malformed substituted sidecar",
    )
    .unwrap();
    symlink(counterfeit.path(), &generation).unwrap();
    let after = retrieval::query(&c, "needle", "lexical", 10, 0, &flag());
    assert!(
        after.is_ok(),
        "pinned collection followed substituted ancestor: {after:?}"
    );
    assert_eq!(after.unwrap(), baseline);
}

#[test]
fn heat_expansion_uses_remaining_budget_after_preserving_direct_evidence() {
    let root = TempDir::new().unwrap();
    let store = TempDir::new().unwrap();
    let mut main = String::from("# needle\n");
    for i in 0..6 {
        main.push_str(&format!("from lib{i} import leaf{i}\n"));
        fs::write(
            root.path().join(format!("lib{i}.py")),
            format!("def leaf{i}():\n    return {i}\n"),
        )
        .unwrap();
    }
    main.push_str("def run():\n    return leaf0()\n");
    fs::write(root.path().join("main.py"), main).unwrap();
    prepare(&root, &store);
    let c = Collection::open(store.path(), None).unwrap();
    for expansion in ["bfs", "heat"] {
        let mut r = request(8192);
        r.expansion = expansion.into();
        let pack = retrieval::retrieve(&c, &r, &flag()).unwrap();
        assert_pack(&c, &pack, r.budget_bytes);
        let items = pack["items"].as_array().unwrap();
        assert!(
            items
                .iter()
                .any(|i| i["expanded"] == false && i["text"].as_str().unwrap().contains("needle"))
        );
        let expanded: Vec<_> = items.iter().filter(|i| i["expanded"] == true).collect();
        assert!(
            !expanded.is_empty(),
            "fixture requires usable {expansion} expansion: {pack}"
        );
        let bytes: usize = expanded
            .iter()
            .map(|i| codec::canonical(*i).unwrap().len())
            .sum();
        assert!(bytes <= r.budget_bytes);
        let first_expanded = items.iter().position(|i| i["expanded"] == true).unwrap();
        assert!(
            items[..first_expanded]
                .iter()
                .all(|i| i["expanded"] == false)
        );
        assert_eq!(pack["expansion"]["method"], expansion);
    }
}

#[test]
fn pagination_preserves_complete_lines_that_fit_a_fresh_page() {
    let root = TempDir::new().unwrap();
    let store = TempDir::new().unwrap();
    for i in 0..3 {
        fs::write(
            root.path().join(format!("long{i}.py")),
            format!("# needle {i} {}\n", "a".repeat(1200)),
        )
        .unwrap();
    }
    prepare(&root, &store);
    let c = Collection::open(store.path(), None).unwrap();
    let mut r = request(4096);
    let mut seen = BTreeSet::new();
    for _ in 0..10 {
        let pack = retrieval::retrieve(&c, &r, &flag()).unwrap();
        assert_pack(&c, &pack, r.budget_bytes);
        assert!(
            pack["omitted"].as_array().unwrap().is_empty(),
            "line fits fresh page: {pack}"
        );
        for item in pack["items"].as_array().unwrap() {
            seen.insert(item["path_key"].as_str().unwrap().to_owned());
        }
        match pack["next_cursor"].as_str() {
            Some(cursor) => r.cursor = Some(cursor.to_owned()),
            None => break,
        }
    }
    assert_eq!(seen.len(), 3);
}

#[test]
fn partial_excerpt_continuation_has_no_byte_gap_or_duplicate() {
    let root = TempDir::new().unwrap();
    let store = TempDir::new().unwrap();
    let source: String = (0..90)
        .map(|i| format!("# needle {i:02} coração {}\r\n", "λ\t\"\\".repeat(35)))
        .collect();
    fs::write(root.path().join("long.py"), &source).unwrap();
    prepare(&root, &store);
    let c = Collection::open(store.path(), None).unwrap();
    let mut r = request(4096);
    let mut spans = vec![];
    let mut partial = false;
    for _ in 0..100 {
        let pack = retrieval::retrieve(&c, &r, &flag()).unwrap();
        assert_pack(&c, &pack, r.budget_bytes);
        assert!(
            pack["omitted"].as_array().unwrap().is_empty(),
            "individual short lines fit: {pack}"
        );
        for item in pack["items"].as_array().unwrap() {
            partial |= item["truncated"] == true;
            spans.push((
                item["start_byte"].as_u64().unwrap(),
                item["end_byte"].as_u64().unwrap(),
                item["text"].as_str().unwrap().to_owned(),
            ));
            if item["truncated"] == true {
                assert_eq!(item["remainder"]["start_byte"], item["end_byte"]);
                assert_eq!(item["remainder"]["start_line"], item["next_start_line"]);
            }
        }
        match pack["next_cursor"].as_str() {
            Some(cursor) => r.cursor = Some(cursor.to_owned()),
            None => break,
        }
    }
    assert!(partial, "fixture must exercise prefix continuation");
    spans.sort_by_key(|span| span.0);
    let mut offset = 0;
    let mut reconstructed = String::new();
    for (start, end, text) in spans {
        assert_eq!(start, offset, "duplicate or skipped bytes");
        offset = end;
        reconstructed.push_str(&text);
    }
    assert_eq!(reconstructed, source);
}

#[test]
fn many_direct_hits_reserve_expansion_share_without_starvation() {
    let root = TempDir::new().unwrap();
    let store = TempDir::new().unwrap();
    for i in 0..30 {
        fs::write(
            root.path().join(format!("root{i:02}.py")),
            format!(
                "# needle {i} {}\nfrom lib{} import leaf{}\ndef run{i}():\n    return leaf{}()\n",
                "a".repeat(120),
                i % 6,
                i % 6,
                i % 6
            ),
        )
        .unwrap();
    }
    for i in 0..6 {
        fs::write(
            root.path().join(format!("lib{i}.py")),
            format!("def leaf{i}():\n    return {i}\n"),
        )
        .unwrap();
    }
    prepare(&root, &store);
    let c = Collection::open(store.path(), None).unwrap();
    for expansion in ["bfs", "heat"] {
        let mut r = request(8192);
        r.expansion = expansion.into();
        let pack = retrieval::retrieve(&c, &r, &flag()).unwrap();
        assert_pack(&c, &pack, r.budget_bytes);
        let items = pack["items"].as_array().unwrap();
        let direct = items.iter().filter(|i| i["expanded"] == false).count();
        let expanded: Vec<_> = items.iter().filter(|i| i["expanded"] == true).collect();
        assert!(
            direct > 0 && direct < 30,
            "fixture requires direct candidates still pending: {pack}"
        );
        assert!(
            !expanded.is_empty(),
            "expansion starved while direct hits remain: {pack}"
        );
        let expansion_bytes: usize = expanded
            .iter()
            .map(|i| codec::canonical(*i).unwrap().len())
            .sum();
        assert!(
            expansion_bytes <= r.budget_bytes / 4,
            "expansion share exceeded while direct hits remain"
        );
        assert!(pack["next_cursor"].is_string());
    }
}
