use compass_core::{
    collection::{self, Collection, PrepareRequest},
    model,
};
use std::{fs, path::Path, sync::atomic::AtomicBool};
use tempfile::TempDir;
fn prepare(root: &Path, store: &Path) -> serde_json::Value {
    collection::prepare(
        &PrepareRequest {
            root: root.into(),
            store: store.into(),
            max_disk_bytes: 8 * 1024 * 1024,
        },
        &AtomicBool::new(false),
    )
    .unwrap()
}
#[test]
fn persistent_lexical_query_and_source_compatibility() {
    let root = TempDir::new().unwrap();
    let store = TempDir::new().unwrap();
    fs::write(
        root.path().join("UserRegistry.rs"),
        b"fn getUserProfile() {\r\n  let account_id = 7;\r\n}\r\n",
    )
    .unwrap();
    fs::write(root.path().join(".env"), b"SECRET_TOKEN").unwrap();
    fs::create_dir(root.path().join(".git")).unwrap();
    fs::write(root.path().join(".git/config"), b"SECRET_TOKEN").unwrap();
    let receipt = prepare(root.path(), store.path());
    let c = Collection::open(store.path(), None).unwrap();
    assert_eq!(c.generation_id(), receipt["generation"].as_str().unwrap());
    let q = c.query("user profile", 10, 0).unwrap();
    assert!(!q["results"].as_array().unwrap().is_empty());
    let hit = &q["results"][0];
    assert_eq!(hit["path_key"], model::path_key(b"UserRegistry.rs"));
    assert_eq!(
        hit["text"],
        "fn getUserProfile() {\r\n  let account_id = 7;\r\n}\r\n"
    );
    assert!(
        c.query("SECRET_TOKEN", 10, 0).unwrap()["results"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(c.files("registry", 10, 0).unwrap()["total"], 1);
    assert_eq!(
        c.verified_documents(&AtomicBool::new(false)).unwrap().len(),
        1
    );
    assert!(c.source_path().join("source.json").exists());
    let _manifest = compass_core::storage::load_snapshot(c.snapshot_path()).unwrap();
}
#[test]
fn updates_preserve_generations_and_reuse_unchanged_work() {
    let root = TempDir::new().unwrap();
    let store = TempDir::new().unwrap();
    fs::write(root.path().join("a"), "old_revision\n").unwrap();
    fs::write(root.path().join("b"), "unchanged\n").unwrap();
    let first = prepare(root.path(), store.path());
    let old = first["generation"].as_str().unwrap();
    fs::write(root.path().join("a"), "new_revision\n").unwrap();
    let next = collection::update(store.path(), &AtomicBool::new(false)).unwrap();
    assert_eq!(next["index"]["reused_indexed_files"], 1);
    assert_eq!(next["index"]["indexed_files"], 1);
    assert_ne!(first["generation"], next["generation"]);
    assert_eq!(
        Collection::open(store.path(), Some(old))
            .unwrap()
            .query("old_revision", 10, 0)
            .unwrap()["results"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let c = Collection::open(store.path(), None).unwrap();
    assert!(
        c.query("old_revision", 10, 0).unwrap()["results"]
            .as_array()
            .unwrap()
            .iter()
            .all(|v| v["text"] != "old_revision\n")
    );
    fs::remove_file(root.path().join("a")).unwrap();
    fs::rename(root.path().join("b"), root.path().join("renamed")).unwrap();
    collection::update(store.path(), &AtomicBool::new(false)).unwrap();
    let c = Collection::open(store.path(), None).unwrap();
    assert_eq!(c.files("", 10, 0).unwrap()["total"], 1);
    assert_eq!(c.files("renamed", 10, 0).unwrap()["total"], 1);
}
#[test]
fn disk_failure_and_cancellation_preserve_current() {
    let root = TempDir::new().unwrap();
    let store = TempDir::new().unwrap();
    fs::write(root.path().join("a"), "original\n").unwrap();
    let first = prepare(root.path(), store.path());
    for i in 0..3 {
        fs::write(
            root.path().join(format!("large-{i}")),
            vec![b'x' + i; 3 * 1024 * 1024],
        )
        .unwrap();
    }
    let error = collection::update(store.path(), &AtomicBool::new(false)).unwrap_err();
    assert!(error.message.contains("max_disk_bytes"));
    assert_eq!(
        Collection::open(store.path(), None)
            .unwrap()
            .generation_id(),
        first["generation"].as_str().unwrap()
    );
    assert_eq!(
        collection::update(store.path(), &AtomicBool::new(true))
            .unwrap_err()
            .exit_code,
        130
    );
}
#[test]
fn corruption_and_links_are_rejected() {
    let root = TempDir::new().unwrap();
    let store = TempDir::new().unwrap();
    fs::write(root.path().join("a"), "safe\n").unwrap();
    prepare(root.path(), store.path());
    let c = Collection::open(store.path(), None).unwrap();
    let object = c
        .source_path()
        .join("objects")
        .join(model::digest(b"safe\n"));
    fs::write(&object, b"evil\n").unwrap();
    assert!(c.query("safe", 10, 0).is_err());
    assert!(c.verified_documents(&AtomicBool::new(false)).is_err());
    fs::write(c.generation_dir().join("index.sqlite"), b"broken").unwrap();
    assert!(Collection::open(store.path(), None).is_err());
}
#[test]
fn internal_destination_native_names_binary_and_interrupted_staging() {
    use std::os::unix::ffi::OsStrExt;
    let root = TempDir::new().unwrap();
    let store = root.path().join("index-store");
    fs::write(
        root.path().join(std::ffi::OsStr::from_bytes(b"name\xff")),
        b"nativeName\n",
    )
    .unwrap();
    fs::write(root.path().join("binary"), [0, 255]).unwrap();
    prepare(root.path(), &store);
    let c = Collection::open(&store, None).unwrap();
    assert_eq!(
        c.verified_documents(&AtomicBool::new(false)).unwrap().len(),
        2
    );
    assert!(
        c.files("index-store", 10, 0).unwrap()["results"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    fs::create_dir(store.join(".stage-interrupted")).unwrap();
    fs::write(store.join(".stage-interrupted/junk"), "incomplete").unwrap();
    collection::update(&store, &AtomicBool::new(false)).unwrap();
    assert!(!store.join(".stage-interrupted").exists());
    assert_eq!(
        Collection::open(&store, None)
            .unwrap()
            .files("", 10, 0)
            .unwrap()["total"],
        2
    );
}
#[test]
fn logical_spans_preserve_crlf_unicode_and_chunk_boundaries() {
    let root = TempDir::new().unwrap();
    let store = TempDir::new().unwrap();
    let mut text = String::new();
    for i in 1..=165 {
        text.push_str(&format!("ação_{i} marker{i}\r\n"));
    }
    fs::write(root.path().join("lines"), &text).unwrap();
    prepare(root.path(), store.path());
    let c = Collection::open(store.path(), None).unwrap();
    let q = c.query("marker81", 10, 0).unwrap();
    let hit = q["results"]
        .as_array()
        .unwrap()
        .iter()
        .find(|h| {
            h["text"].as_str().unwrap().contains("marker81 ")
                || h["text"].as_str().unwrap().contains("marker81\r")
        })
        .unwrap();
    assert_eq!(hit["start_line"], "81");
    assert_eq!(hit["end_line"], "160");
    let a: usize = hit["start_byte"].as_str().unwrap().parse().unwrap();
    let b: usize = hit["end_byte"].as_str().unwrap().parse().unwrap();
    assert_eq!(hit["text"], &text[a..b]);
}

#[test]
fn staged_publication_compare_and_swap_preserves_current() {
    let root = TempDir::new().unwrap();
    let store = TempDir::new().unwrap();
    fs::write(root.path().join("a"), "first\n").unwrap();
    let first = prepare(root.path(), store.path());
    let a = first["generation"].as_str().unwrap();
    fs::write(root.path().join("a"), "second\n").unwrap();
    let staged = collection::update_staged(store.path(), &AtomicBool::new(false)).unwrap();
    let b = staged["generation"].as_str().unwrap();
    assert_eq!(
        Collection::open(store.path(), None)
            .unwrap()
            .generation_id(),
        a
    );
    assert_eq!(
        Collection::open(store.path(), Some(b))
            .unwrap()
            .verified_document(&model::path_key(b"a"), &AtomicBool::new(false))
            .unwrap()
            .bytes,
        b"second\n"
    );
    let another = collection::update_staged(store.path(), &AtomicBool::new(false)).unwrap();
    collection::publish_current_expected(store.path(), b, Some(a), &AtomicBool::new(false))
        .unwrap();
    assert!(
        collection::publish_current_expected(
            store.path(),
            another["generation"].as_str().unwrap(),
            Some(a),
            &AtomicBool::new(false)
        )
        .is_err()
    );
    assert_eq!(
        Collection::open(store.path(), None)
            .unwrap()
            .generation_id(),
        b
    );
}
#[test]
fn package_can_be_read_without_live_tree() {
    let root = TempDir::new().unwrap();
    let store = TempDir::new().unwrap();
    fs::write(root.path().join("file"), "first\r\nsecond\n").unwrap();
    prepare(root.path(), store.path());
    let c = Collection::open(store.path(), None).unwrap();
    fs::remove_file(root.path().join("file")).unwrap();
    let request = compass_core::read::Request {
        snapshot: c.snapshot_path().into(),
        source: c.source_path().into(),
        path_key: model::path_key(b"file"),
        start_line: 2,
        max_lines: 20,
        limits: compass_core::read::limits(None, None).unwrap(),
    };
    let result = compass_core::read::read(&request, &AtomicBool::new(false)).unwrap();
    assert_eq!(result.value["excerpt"]["text"], "second\n");
}
#[test]
fn collection_queries_honor_cancellation_and_index_tampering_after_open() {
    let root = TempDir::new().unwrap();
    let store = TempDir::new().unwrap();
    fs::write(root.path().join("a"), "find_me\n").unwrap();
    prepare(root.path(), store.path());
    let c = Collection::open(store.path(), None).unwrap();
    assert_eq!(
        c.query_cancel("find_me", 10, 0, &AtomicBool::new(true))
            .unwrap_err()
            .exit_code,
        130
    );
    assert!(c.query(&"x".repeat(32769), 10, 0).is_err());
    fs::write(c.generation_dir().join("index.sqlite"), "tampered").unwrap();
    assert!(c.query("find_me", 10, 0).is_err());
}
