//! Independent review controls: failure paths and bytes, not an implementation oracle.
use compass_core::{
    Result,
    collection::{self, Collection, PrepareRequest},
    model::{self, Entry, Policy, Workspace},
    observe::{self, ContentCapture},
};
use std::{
    fs::{self, File},
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
use tempfile::TempDir;
fn workspace() -> Workspace {
    Workspace {
        schema_version: "1".into(),
        contract_version: "0.1.0".into(),
        workspace_id: "11111111-1111-4111-8111-111111111111".into(),
        path_namespace: "linux_bytes_v1".into(),
        created_at: "2026-10-23T00:00:00Z".into(),
    }
}
fn prepare(root: &Path, store: &Path, max: u64) -> serde_json::Value {
    collection::prepare(
        &PrepareRequest {
            root: root.into(),
            store: store.into(),
            max_disk_bytes: max,
        },
        &AtomicBool::new(false),
    )
    .unwrap()
}

struct Capture<'a> {
    path: PathBuf,
    cancel: Option<&'a AtomicBool>,
    replace: bool,
    attempts: usize,
    pending: Vec<u8>,
    accepted: Vec<(String, Vec<u8>)>,
}
impl ContentCapture for Capture<'_> {
    fn begin(&mut self, _: &[u8]) -> Result<()> {
        self.pending.clear();
        self.attempts += 1;
        Ok(())
    }
    fn chunk(&mut self, bytes: &[u8]) -> Result<()> {
        self.pending.extend_from_slice(bytes);
        if self.replace {
            self.replace = false;
            let next = self.path.with_extension("new");
            fs::write(&next, b"replacement revision\r\n").unwrap();
            fs::rename(next, &self.path).unwrap();
        }
        if let Some(cancel) = self.cancel {
            cancel.store(true, Ordering::Relaxed);
        }
        Ok(())
    }
    fn finish(&mut self, entry: &Entry) -> Result<()> {
        assert_eq!(
            entry.content_sha256.as_deref(),
            Some(model::digest(&self.pending).as_str())
        );
        assert_eq!(
            entry.size_bytes.as_deref(),
            Some(self.pending.len().to_string().as_str())
        );
        self.accepted
            .push((entry.path_key.clone(), self.pending.clone()));
        Ok(())
    }
}
#[test]
fn capture_retries_replaced_inode_and_only_accepts_exact_new_revision() {
    let root = TempDir::new().unwrap();
    let outside = TempDir::new().unwrap();
    let path = root.path().join("file");
    fs::write(&path, b"old bytes\n").unwrap();
    fs::write(root.path().join(".env"), b"do not capture").unwrap();
    fs::write(outside.path().join("secret"), b"external secret").unwrap();
    symlink(outside.path(), root.path().join("external")).unwrap();
    let mut capture = Capture {
        path,
        cancel: None,
        replace: true,
        attempts: 0,
        pending: vec![],
        accepted: vec![],
    };
    let snapshot = observe::observe_captured(
        &File::open(root.path()).unwrap(),
        &workspace(),
        &Policy::default(),
        &AtomicBool::new(false),
        &mut capture,
    )
    .unwrap();
    assert_eq!(snapshot.quality, "partial");
    assert!(
        snapshot
            .directory_coverage
            .iter()
            .any(|c| c.state == "unstable")
    );
    assert_eq!(capture.attempts, 2);
    assert_eq!(
        capture.accepted,
        vec![(
            model::path_key(b"file"),
            b"replacement revision\r\n".to_vec()
        )]
    );
    assert_eq!(
        snapshot.stats.bytes_read,
        (b"old bytes\n".len() + b"replacement revision\r\n".len()).to_string()
    );
}
#[test]
fn capture_cancel_mid_stream_produces_no_accepted_revision() {
    let root = TempDir::new().unwrap();
    let path = root.path().join("file");
    fs::write(&path, vec![b'x'; 131073]).unwrap();
    let cancel = AtomicBool::new(false);
    let mut capture = Capture {
        path,
        cancel: Some(&cancel),
        replace: false,
        attempts: 0,
        pending: vec![],
        accepted: vec![],
    };
    let result = observe::observe_captured(
        &File::open(root.path()).unwrap(),
        &workspace(),
        &Policy::default(),
        &cancel,
        &mut capture,
    );
    assert!(matches!(result,Err(e) if e.code=="cancelled"));
    assert!(capture.accepted.is_empty());
    assert!(!capture.pending.is_empty());
}
#[test]
fn failed_update_respects_logical_disk_budget_and_retains_queryable_generation() {
    let root = TempDir::new().unwrap();
    let store = TempDir::new().unwrap();
    let budget = 160 * 1024;
    fs::write(root.path().join("a"), b"original_marker\n").unwrap();
    let first = prepare(root.path(), store.path(), budget);
    fs::write(root.path().join("large"), vec![b'x'; 200 * 1024]).unwrap();
    assert!(collection::update(store.path(), &AtomicBool::new(false)).is_err());
    let status = collection::status(store.path()).unwrap();
    assert!(
        status["disk_bytes"]
            .as_str()
            .unwrap()
            .parse::<u64>()
            .unwrap()
            <= budget
    );
    assert_eq!(status["generation"], first["generation"]);
    assert_eq!(
        Collection::open(store.path(), None)
            .unwrap()
            .query("original_marker", 10, 0)
            .unwrap()["results"][0]["text"],
        "original_marker\n"
    );
}
#[test]
fn abandoned_stage_symlink_is_never_followed_or_removed() {
    let root = TempDir::new().unwrap();
    let store = TempDir::new().unwrap();
    let outside = TempDir::new().unwrap();
    fs::write(root.path().join("a"), b"safe_marker\n").unwrap();
    let first = prepare(root.path(), store.path(), 1024 * 1024);
    fs::write(outside.path().join("keep"), b"untouched").unwrap();
    symlink(outside.path(), store.path().join(".stage-malicious")).unwrap();
    assert!(collection::update(store.path(), &AtomicBool::new(false)).is_err());
    assert_eq!(fs::read(outside.path().join("keep")).unwrap(), b"untouched");
    assert_eq!(
        Collection::open(store.path(), None)
            .unwrap()
            .generation_id(),
        first["generation"].as_str().unwrap()
    );
}
#[test]
fn object_component_symlink_rejected_even_with_matching_bytes() {
    let root = TempDir::new().unwrap();
    let store = TempDir::new().unwrap();
    let outside = TempDir::new().unwrap();
    fs::write(root.path().join("a"), b"safe_marker\n").unwrap();
    prepare(root.path(), store.path(), 1024 * 1024);
    let c = Collection::open(store.path(), None).unwrap();
    fs::write(
        outside.path().join(model::digest(b"safe_marker\n")),
        b"safe_marker\n",
    )
    .unwrap();
    fs::rename(
        c.source_path().join("objects"),
        c.source_path().join("objects-old"),
    )
    .unwrap();
    symlink(outside.path(), c.source_path().join("objects")).unwrap();
    assert!(c.verified_documents(&AtomicBool::new(false)).is_err());
    assert!(c.query("safe_marker", 10, 0).is_err());
    assert!(Collection::open(store.path(), Some("../not-generation")).is_err());
}
#[test]
fn opened_collection_stays_on_pinned_generation_after_store_rename() {
    let root = TempDir::new().unwrap();
    let parent = TempDir::new().unwrap();
    let store = parent.path().join("store");
    fs::write(root.path().join("a"), b"pinned_marker\n").unwrap();
    prepare(root.path(), &store, 1024 * 1024);
    let c = Collection::open(&store, None).unwrap();
    fs::rename(&store, parent.path().join("moved")).unwrap();
    fs::create_dir(&store).unwrap();
    fs::write(store.join("current"), b"untrusted substitute").unwrap();
    let docs = c.verified_documents(&AtomicBool::new(false)).unwrap();
    assert_eq!(docs[0].bytes, b"pinned_marker\n");
    assert_eq!(
        c.query("pinned_marker", 10, 0).unwrap()["results"][0]["text"],
        "pinned_marker\n"
    );
}

#[test]
fn malformed_source_descriptor_is_rejected_before_admitting_generation() {
    let root = TempDir::new().unwrap();
    let store = TempDir::new().unwrap();
    fs::write(root.path().join("a"), b"valid_revision\n").unwrap();
    prepare(root.path(), store.path(), 1024 * 1024);
    let c = Collection::open(store.path(), None).unwrap();
    let snapshot = fs::read(c.snapshot_path()).unwrap();
    // Binding alone is insufficient: this omits required version/workspace/snapshot/policy fields.
    fs::write(
        c.source_path().join("source.json"),
        serde_json::to_vec(&serde_json::json!({"snapshot_file_sha256":model::digest(&snapshot)}))
            .unwrap(),
    )
    .unwrap();
    let request = compass_core::read::Request {
        snapshot: c.snapshot_path().into(),
        source: c.source_path().into(),
        path_key: model::path_key(b"a"),
        start_line: 1,
        max_lines: 1,
        limits: compass_core::read::limits(None, None).unwrap(),
    };
    assert!(compass_core::read::read(&request, &AtomicBool::new(false)).is_err());
    assert!(
        Collection::open(store.path(), None).is_err(),
        "generation admitted a malformed source package; legacy read rejects the same descriptor"
    );
}
