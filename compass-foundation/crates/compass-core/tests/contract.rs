use compass_core::{
    codec::{canonical, decode},
    compare::{Comparison, compare},
    model::{Coverage, Entry, Policy, Snapshot, decode_path_key, digest, path_key},
};
use serde_json::{Value, json};

const BASELINE: &[u8] = include_bytes!("../../../research/contracts/fixtures/baseline.json");
const CURRENT: &[u8] = include_bytes!("../../../research/contracts/fixtures/current.json");
const EXPECTED: &[u8] =
    include_bytes!("../../../research/contracts/fixtures/expected-comparison.json");
fn baseline() -> Snapshot {
    decode(BASELINE).unwrap()
}
fn current() -> Snapshot {
    decode(CURRENT).unwrap()
}
fn seal(s: &mut Snapshot) {
    s.entries
        .sort_by_key(|e| decode_path_key(&e.path_key).unwrap());
    s.directory_coverage
        .sort_by_key(|c| decode_path_key(&c.path_key).unwrap());
    s.stats.observed_files = s
        .entries
        .iter()
        .filter(|e| e.state == "observed")
        .count()
        .to_string();
    s.stats.excluded_entries = s
        .entries
        .iter()
        .filter(|e| e.state == "excluded")
        .count()
        .to_string();
    s.stats.unknown_entries = s
        .entries
        .iter()
        .filter(|e| !["observed", "excluded"].contains(&e.state.as_str()))
        .count()
        .to_string();
    s.stats.bytes_read = s
        .entries
        .iter()
        .filter_map(|e| e.size_bytes.as_ref())
        .map(|v| v.parse::<u64>().unwrap())
        .sum::<u64>()
        .to_string();
    s.stats.read_attempts = s.stats.observed_files.clone();
    let partial = s.stats.unknown_entries != "0"
        || s.directory_coverage.iter().any(|c| c.state != "complete");
    s.quality = if partial { "partial" } else { "complete" }.into();
    s.state_digest = if partial {
        None
    } else {
        Some(s.compute_state_digest().unwrap())
    };
    s.validate().unwrap();
}
fn observed(path: &[u8], content: &[u8]) -> Entry {
    Entry {
        path_key: path_key(path),
        kind: "file".into(),
        state: "observed".into(),
        reason: None,
        content_sha256: Some(digest(content)),
        size_bytes: Some(content.len().to_string()),
        executable: Some(false),
    }
}
fn coverage(path: &[u8], state: &str) -> Coverage {
    Coverage {
        path_key: path_key(path),
        state: state.into(),
        reason: (state != "complete").then(|| "enumeration_failed".into()),
    }
}
fn assert_rejected(s: &Snapshot) {
    assert!(s.validate().is_err(), "invalid manifest accepted");
}

#[test]
fn frozen_transport_and_manual_comparison() {
    let a = baseline();
    let b = current();
    a.validate().unwrap();
    b.validate().unwrap();
    let expected: Comparison = decode(EXPECTED).unwrap();
    assert_eq!(compare(&a, &b).unwrap(), expected);
    assert_eq!(
        canonical(&compare(&a, &b).unwrap()).unwrap(),
        EXPECTED.strip_suffix(b"\n").unwrap()
    );
}
#[test]
fn standard_sha256_and_jcs_vectors() {
    assert_eq!(
        digest(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        digest(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        canonical(&json!({"b":2,"a":1})).unwrap(),
        br#"{"a":1,"b":2}"#
    );
    // JCS sorts UTF-16 code units: the supplementary character precedes U+FB33.
    assert_eq!(
        String::from_utf8(canonical(&json!({"\u{fb33}":1,"😀":2})).unwrap()).unwrap(),
        "{\"😀\":2,\"דּ\":1}"
    );
    let numbers: Value = decode(br#"[333333333.33333329,1E30,4.50,2e-3,1e-27]"#).unwrap();
    assert_eq!(
        canonical(&numbers).unwrap(),
        b"[333333333.3333333,1e+30,4.5,0.002,1e-27]"
    );
}
#[test]
fn strict_json_rejects_duplicate_members_at_any_depth() {
    for input in [
        br#"{"a":1,"a":2}"#.as_slice(),
        br#"{"a":[{"b":1,"\u0062":2}]}"#,
        br#"{"a":1} null"#,
        br#"{"a":NaN}"#,
    ] {
        assert!(decode::<Value>(input).is_err());
    }
    let mut v: Value = decode(BASELINE).unwrap();
    v["extra"] = json!(0);
    assert!(decode::<Snapshot>(&canonical(&v).unwrap()).is_err());
}
#[test]
fn nullable_and_absent_fields_are_distinct() {
    let mut v: Value = decode(BASELINE).unwrap();
    v["entries"][0]["reason"] = Value::Null;
    assert!(decode::<Snapshot>(&canonical(&v).unwrap()).is_err());
    let mut v: Value = decode(BASELINE).unwrap();
    v.as_object_mut().unwrap().remove("state_digest");
    assert!(decode::<Snapshot>(&canonical(&v).unwrap()).is_err());
    let mut v = serde_json::to_value(Policy::default()).unwrap();
    v.as_object_mut().unwrap().remove("max_entries");
    assert!(decode::<Policy>(&canonical(&v).unwrap()).is_err());
}
#[test]
fn canonical_native_paths_reject_aliases_and_traversal() {
    for path in [b"/a".as_slice(), b"a/", b"a//b", b"./a", b"a/../b", b"a\0b"] {
        assert!(decode_path_key(&path_key(path)).is_err());
    }
    assert!(decode_path_key("YQ==").is_err());
    assert!(decode_path_key("YR").is_err());
    assert_eq!(decode_path_key(&path_key(b"x\xff/A")).unwrap(), b"x\xff/A");
}
#[test]
fn canonical_hashes_uuids_and_decimals_are_required() {
    let mut s = baseline();
    s.entries[0].size_bytes = Some("03".into());
    assert_rejected(&s);
    s = baseline();
    s.stats.bytes_read = "18446744073709551616".into();
    assert_rejected(&s);
    s = baseline();
    s.entries[0].content_sha256 = Some("A".repeat(64));
    assert_rejected(&s);
    s = baseline();
    s.workspace_id = "11111111-1111-5111-8111-111111111111".into();
    assert_rejected(&s);
    s = baseline();
    s.started_at = "2026-10-14T01:00:00+01:00".into();
    assert_rejected(&s);
    s = baseline();
    s.finished_at = "2026-10-13T23:00:00Z".into();
    assert_rejected(&s);
}
#[test]
fn hash_tampering_and_wrong_counters_are_rejected() {
    let mut s = baseline();
    s.state_digest = Some("0".repeat(64));
    assert_rejected(&s);
    s = baseline();
    s.policy_id = "0".repeat(64);
    assert_rejected(&s);
    s = baseline();
    s.stats.observed_files = "1".into();
    assert_rejected(&s);
    s = baseline();
    s.stats.bytes_read = "2".into();
    assert_rejected(&s);
}
#[test]
fn impossible_tree_and_missing_coverage_are_rejected() {
    let mut s = baseline();
    s.directory_coverage.clear();
    assert_rejected(&s);
    s = baseline();
    s.entries[0].path_key = path_key(b"missing/a.txt");
    assert_rejected(&s);
    s = baseline();
    s.directory_coverage.push(coverage(b"a.txt", "complete"));
    assert_rejected(&s);
    s = baseline();
    s.entries[1] = s.entries[0].clone();
    assert_rejected(&s);
    s = baseline();
    s.entries.reverse();
    assert_rejected(&s);
}
#[test]
fn revision_and_quality_cannot_contradict_states() {
    let mut s = baseline();
    s.entries[0].state = "unreadable".into();
    s.entries[0].reason = Some("permission_denied".into());
    assert_rejected(&s);
    s = baseline();
    s.quality = "partial".into();
    s.state_digest = None;
    assert_rejected(&s);
    s = baseline();
    s.directory_coverage[0] = coverage(b"", "unstable");
    assert_rejected(&s);
    s = baseline();
    s.entries[0].executable = None;
    assert_rejected(&s);
}
#[test]
fn state_digest_ignores_observation_identity_window_and_diagnostics() {
    let a = baseline();
    let mut b = a.clone();
    b.snapshot_id = current().snapshot_id;
    b.started_at = "2026-10-15T00:00:00Z".into();
    b.finished_at = "2026-10-15T00:00:01Z".into();
    b.producer_build = "other-build".into();
    b.stats.duration_ms = "50".into();
    b.validate().unwrap();
    assert_eq!(
        a.compute_state_digest().unwrap(),
        b.compute_state_digest().unwrap()
    );
    let result = compare(&a, &b).unwrap();
    assert_eq!(result.relation, "equal");
    assert_eq!(result.counts.unchanged, "2");
    assert!(result.changes.is_empty());
}
#[test]
fn empty_workspace_is_complete_and_empty_file_is_real() {
    let mut empty = baseline();
    empty.entries.clear();
    seal(&mut empty);
    let mut one = empty.clone();
    one.entries.push(observed(b"empty", b""));
    seal(&mut one);
    assert_eq!(compare(&empty, &empty).unwrap().relation, "equal");
    assert_eq!(
        compare(&empty, &one).unwrap().changes[0].classification,
        "added"
    );
    assert_eq!(
        compare(&one, &empty).unwrap().changes[0].classification,
        "removed"
    );
    assert_ne!(empty.state_digest, one.state_digest);
}
#[test]
fn executable_change_and_byte_only_change_remain_distinct() {
    let a = baseline();
    let mut b = a.clone();
    b.entries[0].executable = Some(true);
    seal(&mut b);
    let r = compare(&a, &b).unwrap();
    assert_eq!(r.changes[0].classification, "metadata_changed");
    assert!(!r.changes[0].content_changed);
    b.entries[0].content_sha256 = current().entries[0].content_sha256.clone();
    seal(&mut b);
    let r = compare(&a, &b).unwrap();
    assert_eq!(r.changes[0].classification, "modified");
    assert!(r.changes[0].executable_changed);
}
#[test]
fn rename_is_removal_and_addition_even_for_equal_hashes() {
    let a = baseline();
    let mut b = a.clone();
    b.entries[0].path_key = path_key(b"z.txt");
    seal(&mut b);
    let r = compare(&a, &b).unwrap();
    assert_eq!(
        r.changes
            .iter()
            .map(|c| c.classification.as_str())
            .collect::<Vec<_>>(),
        ["removed", "added"]
    );
}
#[test]
fn unreadable_entry_is_unknown_not_removed() {
    let a = baseline();
    let mut b = a.clone();
    b.entries[0] = Entry {
        path_key: path_key(b"a.txt"),
        kind: "file".into(),
        state: "unreadable".into(),
        reason: Some("permission_denied".into()),
        content_sha256: None,
        size_bytes: None,
        executable: None,
    };
    seal(&mut b);
    let r = compare(&a, &b).unwrap();
    assert_eq!(r.relation, "undetermined");
    assert_eq!(r.changes[0].classification, "unknown");
    assert_eq!(r.counts.removed, "0");
    assert_eq!(r.unknown_regions.len(), 1);
}
#[test]
fn unknown_ancestor_dominates_missing_and_successfully_read_descendants() {
    let mut a = baseline();
    a.entries = vec![observed(b"dir/a", b"a"), observed(b"dir/b", b"b")];
    a.directory_coverage.push(coverage(b"dir", "complete"));
    seal(&mut a);
    let mut b = a.clone();
    b.entries.pop();
    b.directory_coverage[1] = coverage(b"dir", "unreadable");
    seal(&mut b);
    let r = compare(&a, &b).unwrap();
    assert_eq!(r.relation, "undetermined");
    assert_eq!(r.counts.unknown, "2");
    assert_eq!(r.counts.removed, "0");
    assert_eq!(r.counts.unchanged, "0");
}
#[test]
fn absent_ancestor_proves_descendant_absence_only_with_complete_parent() {
    let mut a = baseline();
    a.entries = vec![observed(b"dir/a", b"a")];
    a.directory_coverage.push(coverage(b"dir", "complete"));
    seal(&mut a);
    let mut b = baseline();
    b.entries.clear();
    seal(&mut b);
    assert_eq!(
        compare(&a, &b).unwrap().changes[0].classification,
        "removed"
    );
    b.directory_coverage[0] = coverage(b"", "not_visited");
    seal(&mut b);
    assert_eq!(
        compare(&a, &b).unwrap().changes[0].classification,
        "unknown"
    );
}
#[test]
fn file_directory_and_link_transitions_follow_fixed_contract() {
    let mut a = baseline();
    a.entries = vec![observed(b"p", b"a")];
    seal(&mut a);
    let mut b = a.clone();
    b.entries = vec![observed(b"p/new", b"a")];
    b.directory_coverage.push(coverage(b"p", "complete"));
    seal(&mut b);
    let r = compare(&a, &b).unwrap();
    assert_eq!(
        r.changes
            .iter()
            .map(|c| c.classification.as_str())
            .collect::<Vec<_>>(),
        ["removed", "added"]
    );
    let r = compare(&b, &a).unwrap();
    assert_eq!(
        r.changes
            .iter()
            .map(|c| c.classification.as_str())
            .collect::<Vec<_>>(),
        ["added", "removed"]
    );
    let mut link = a.clone();
    link.entries[0] = Entry {
        path_key: path_key(b"p"),
        kind: "symlink".into(),
        state: "excluded".into(),
        reason: Some("symlink_not_followed".into()),
        content_sha256: None,
        size_bytes: None,
        executable: None,
    };
    seal(&mut link);
    assert_eq!(
        compare(&a, &link).unwrap().changes[0].classification,
        "left_scope"
    );
    assert_eq!(
        compare(&link, &a).unwrap().changes[0].classification,
        "entered_scope"
    );
    assert_eq!(
        compare(&b, &link).unwrap().changes[0].reason.as_deref(),
        Some("symlink_not_followed")
    );
}
#[test]
fn partial_empty_tree_keeps_unknown_region_without_inventing_changes() {
    let mut s = baseline();
    s.entries.clear();
    s.directory_coverage[0] = coverage(b"", "unreadable");
    seal(&mut s);
    let r = compare(&s, &s).unwrap();
    assert_eq!(r.relation, "undetermined");
    assert_eq!(r.unknown_regions.len(), 2);
    assert!(r.changes.is_empty());
}
#[test]
fn valid_incompatibility_and_unknown_version_have_distinct_exit_codes() {
    let a = baseline();
    let mut b = a.clone();
    b.workspace_id = "44444444-4444-4444-8444-444444444444".into();
    seal(&mut b);
    assert_eq!(compare(&a, &b).unwrap_err().exit_code, 2);
    b = a.clone();
    b.policy.max_file_bytes = Some("100".into());
    b.policy_id = b.policy.id().unwrap();
    seal(&mut b);
    assert_eq!(compare(&a, &b).unwrap_err().exit_code, 2);
    b = a.clone();
    b.schema_version = "future".into();
    assert_eq!(compare(&a, &b).unwrap_err().exit_code, 1);
}
#[test]
fn policy_cannot_disable_internal_or_environment_exclusions() {
    let mut p = Policy::default();
    p.exclude_directory_names.retain(|n| n != ".compass");
    assert!(p.validate().is_err());
    p = Policy::default();
    p.exclude_file_prefixes.clear();
    assert!(p.validate().is_err());
    p = Policy::default();
    p.max_entries = Some("0".into());
    assert!(p.validate().is_err());
}
