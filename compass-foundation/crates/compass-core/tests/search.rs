use compass_core::search::{self, Error, Limits, Request};
use std::{path::PathBuf, sync::atomic::AtomicBool};
#[test]
fn public_invalid_query_never_opens_inputs() {
    for q in ["", "a\nb", "a\rb", "a\0b", &"é".repeat(129)] {
        let r = Request {
            snapshot: PathBuf::from("/absent"),
            source: PathBuf::from("/absent"),
            query: q.into(),
            max_matches: 20,
            limits: Limits::default(),
        };
        assert!(matches!(
            search::search(&r, &AtomicBool::new(false)),
            Err(Error("invalid_arguments"))
        ));
    }
}
#[test]
fn public_limits_are_closed_and_canonical() {
    use serde_json::json;
    assert!(Limits::reduced(Some(&json!({})), None).is_err());
    assert!(Limits::reduced(None, Some(0)).is_err());
    assert_eq!(Limits::default().get("max_line_bytes"), 4096);
}
#[test]
fn public_search_exact_bytes_and_immutable_inputs() {
    use compass_core::{
        codec,
        model::{Snapshot, digest},
    };
    use serde_json::json;
    use std::fs;
    let t = tempfile::tempdir().unwrap();
    let raw = include_bytes!("../../../research/proposals/d0-c2/fixtures/manifests/a.json");
    let s: Snapshot = codec::decode(raw).unwrap();
    let p = t.path();
    fs::write(p.join("snapshot"), raw).unwrap();
    fs::create_dir(p.join("objects")).unwrap();
    fs::write(p.join("objects").join(digest(b"a\n")), b"a\n").unwrap();
    let d=codec::canonical(&json!({"source_schema_version":"1","source_contract_version":"0.1.0","workspace_id":s.workspace_id,"snapshot_id":s.snapshot_id,"policy_id":s.policy_id,"snapshot_file_sha256":digest(raw)})).unwrap();
    fs::write(p.join("source.json"), &d).unwrap();
    let r = Request {
        snapshot: p.join("snapshot"),
        source: p.into(),
        query: "a".into(),
        max_matches: 20,
        limits: Limits::default(),
    };
    let o = search::search(&r, &AtomicBool::new(false)).unwrap();
    assert_eq!(o.exit_code, 0);
    assert_eq!(o.value["matches"][0]["text"], "a\n");
    assert_eq!(o.value["matches"][0]["start_byte"], "0");
    assert_eq!(o.value["matches"][0]["end_byte"], "2");
    assert_eq!(fs::read(&r.snapshot).unwrap(), raw);
    assert_eq!(fs::read(p.join("source.json")).unwrap(), d);
    assert_eq!(
        fs::read(p.join("objects").join(digest(b"a\n"))).unwrap(),
        b"a\n"
    );
}
