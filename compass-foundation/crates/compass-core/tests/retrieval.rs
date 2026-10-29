use compass_core::{
    collection::{self, Collection, PrepareRequest},
    model, retrieval,
};
use std::{fs, sync::atomic::AtomicBool};
fn flag() -> AtomicBool {
    AtomicBool::new(false)
}
fn prepare(root: &tempfile::TempDir, store: &tempfile::TempDir) {
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
#[test]
fn hybrid_omits_only_unsupported_literal_shapes() {
    let root = tempfile::tempdir().unwrap();
    let store = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join("file.rs"),
        "// needle\npub fn example() {}\n",
    )
    .unwrap();
    prepare(&root, &store);
    let c = Collection::open(store.path(), None).unwrap();
    for query in [
        "needle ".repeat(60),
        "needle\0".to_owned(),
        "needle\r\n".to_owned(),
    ] {
        let result = retrieval::query(&c, &query, "hybrid", 10, 0, &flag()).unwrap();
        assert_eq!(result["coverage"]["literal"]["status"], "unsupported");
        assert!(!result["results"].as_array().unwrap().is_empty());
        assert!(retrieval::query(&c, &query, "literal", 10, 0, &flag()).is_err());
    }
}
#[test]
fn literal_many_hits_canonicalizes_each_file_once() {
    let root = tempfile::tempdir().unwrap();
    let store = tempfile::tempdir().unwrap();
    let source = (0..240)
        .map(|i| format!("// repeated_marker {i}\n"))
        .collect::<String>();
    fs::write(root.path().join("file.rs"), source).unwrap();
    prepare(&root, &store);
    let c = Collection::open(store.path(), None).unwrap();
    let result = retrieval::query(&c, "repeated_marker", "literal", 10, 0, &flag()).unwrap();
    assert_eq!(result["coverage"]["canonicalization"]["verified_files"], 1);
    assert_eq!(result["results"].as_array().unwrap().len(), 3);
}
#[test]
fn all_source_descriptor_bindings_and_shape_are_checked() {
    let root = tempfile::tempdir().unwrap();
    let store = tempfile::tempdir().unwrap();
    fs::write(root.path().join("a"), "data\n").unwrap();
    collection::prepare(
        &PrepareRequest {
            root: root.path().into(),
            store: store.path().into(),
            max_disk_bytes: 1024 * 1024,
        },
        &flag(),
    )
    .unwrap();
    let c = Collection::open(store.path(), None).unwrap();
    let path = c.source_path().join("source.json");
    let original: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    for key in [
        "source_schema_version",
        "source_contract_version",
        "workspace_id",
        "snapshot_id",
        "policy_id",
        "snapshot_file_sha256",
    ] {
        let mut bad = original.clone();
        bad[key] = serde_json::json!("wrong");
        fs::write(&path, serde_json::to_vec(&bad).unwrap()).unwrap();
        assert!(
            Collection::open(store.path(), None).is_err(),
            "binding {key}"
        );
    }
    let mut bad = original.clone();
    bad["unexpected"] = serde_json::json!(true);
    fs::write(&path, serde_json::to_vec(&bad).unwrap()).unwrap();
    assert!(Collection::open(store.path(), None).is_err());
    fs::write(&path, serde_json::to_vec(&original).unwrap()).unwrap();
    assert_eq!(
        Collection::open(store.path(), None)
            .unwrap()
            .verified_document(&model::path_key(b"a"), &flag())
            .unwrap()
            .bytes,
        b"data\n"
    );
}
