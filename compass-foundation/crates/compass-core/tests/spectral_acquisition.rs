use compass_core::{
    codec,
    collection::{Collection, PrepareRequest},
    model, retrieval,
    spectral_acquisition::{self, Request},
};
use serde_json::Value;
use std::{fs, sync::atomic::AtomicBool};
fn flag() -> AtomicBool {
    AtomicBool::new(false)
}
fn prepared(source: &str) -> (tempfile::TempDir, tempfile::TempDir, Collection) {
    let root = tempfile::tempdir().unwrap();
    let store = tempfile::tempdir().unwrap();
    fs::write(root.path().join("main.py"), source).unwrap();
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
    let c = Collection::open(store.path(), None).unwrap();
    (root, store, c)
}
fn request(scope: &str, budget: usize) -> Request {
    Request {
        query: "entry".into(),
        mode: "hybrid".into(),
        scope: scope.into(),
        budget_bytes: budget,
    }
}
fn verify(pack: &Value, source: &str) {
    assert_eq!(pack["payload_bytes"], codec::canonical(pack).unwrap().len());
    assert!(
        codec::canonical(pack).unwrap().len() <= pack["budget_bytes"].as_u64().unwrap() as usize
    );
    for item in pack["items"].as_array().unwrap() {
        let a = item["start_byte"].as_u64().unwrap() as usize;
        let b = item["end_byte"].as_u64().unwrap() as usize;
        assert_eq!(
            item["text"].as_str().unwrap().as_bytes(),
            &source.as_bytes()[a..b]
        );
        assert_eq!(item["content_sha256"], model::digest(source.as_bytes()));
    }
    assert_eq!(pack["schema_version"], "3");
    assert_eq!(pack["acquisition"]["state"], "terminal");
    assert!(pack["next_cursor"].is_null());
    assert!(pack.get("completion").is_none());
}
#[test]
fn explicit_options_reject_every_contradiction() {
    use spectral_acquisition::validate_options as v;
    assert!(v(None, None, false, false, None, "heat").is_ok());
    assert!(v(None, Some("local"), false, false, None, "none").is_err());
    for scope in [None, Some("bad")] {
        assert!(
            v(
                Some("spectral_window_v1"),
                scope,
                false,
                false,
                None,
                "none"
            )
            .is_err()
        );
    }
    for (contract, cursor, theta, expansion) in [
        (true, false, None, "none"),
        (false, true, None, "none"),
        (false, false, Some(1.0), "none"),
        (false, false, None, "heat"),
    ] {
        assert!(
            v(
                Some("spectral_window_v1"),
                Some("local"),
                contract,
                cursor,
                theta,
                expansion
            )
            .is_err()
        );
    }
    assert!(
        v(
            Some("spectral_window_v1"),
            Some("dependencies"),
            false,
            false,
            None,
            "none"
        )
        .is_ok()
    );
}
#[test]
fn scales_change_coherent_content_with_one_bounded_document_cache() {
    let source = "def entry():\n    return 1\n\ndef unrelated():\n    return 2\n\ndef third():\n    return 3\n";
    let (_root, _store, c) = prepared(source);
    let r = request("local", 16384);
    let (small, _, _) =
        spectral_acquisition::laboratory(&c, &r, "fixed_heat", Some(0.015625), &flag()).unwrap();
    let (large, work, _) =
        spectral_acquisition::laboratory(&c, &r, "fixed_heat", Some(64.0), &flag()).unwrap();
    verify(&small, source);
    verify(&large, source);
    assert_ne!(small["items"], large["items"]);
    assert_eq!(work["verified_document_reads"], 1);
    let (adaptive, work, trajectory) =
        spectral_acquisition::laboratory(&c, &r, "spectral_window_v1", None, &flag()).unwrap();
    verify(&adaptive, source);
    assert_eq!(work["verified_document_reads"], 1);
    assert!(trajectory.as_array().unwrap().len() > 2);
    assert_eq!(adaptive["acquisition"]["reason"], "spectral_window");
    if adaptive["acquisition"]["reason"] == "spectral_window" {
        assert!(
            adaptive["diagnostics"]["selected_theta"].as_f64().unwrap()
                < adaptive["diagnostics"]["max_evaluated_theta"]
                    .as_f64()
                    .unwrap()
        );
    }
}
#[test]
fn unresolved_dependency_is_terminal_with_explicit_gaps() {
    let source = "def entry(callback):\n    return callback()\n";
    let (_root, _store, c) = prepared(source);
    let pack =
        spectral_acquisition::retrieve(&c, &request("dependencies", 16384), &flag()).unwrap();
    verify(&pack, source);
    assert_ne!(pack["acquisition"]["reason"], "spectral_window");
    assert_eq!(pack["structural"]["status"], "gaps");
    assert!(!pack["coverage"]["gaps"].as_array().unwrap().is_empty());
}
#[test]
fn budget_omits_whole_body_and_never_claims_window() {
    let source = format!(
        "def entry():\n    value = '{}'\n    return value\n",
        "x".repeat(14000)
    );
    let (_root, _store, c) = prepared(&source);
    let pack = spectral_acquisition::retrieve(&c, &request("local", 4096), &flag()).unwrap();
    verify(&pack, &source);
    assert!(pack["items"].as_array().unwrap().is_empty());
    assert_ne!(pack["acquisition"]["reason"], "spectral_window");
    assert!(pack["coverage"]["omitted_units"].as_u64().unwrap() > 0);
}
#[test]
fn no_seed_and_cancellation_remain_explicit() {
    let (_root, _store, c) = prepared("def other():\n    return 1\n");
    let pack = spectral_acquisition::retrieve(&c, &request("local", 16384), &flag()).unwrap();
    assert_eq!(pack["acquisition"]["reason"], "no_evidence");
    assert!(pack["items"].as_array().unwrap().is_empty());
    assert!(
        spectral_acquisition::retrieve(&c, &request("local", 16384), &AtomicBool::new(true))
            .is_err()
    );
}
#[test]
fn all_matched_arms_share_fixed_seeds_scope_and_materializer() {
    let source = "def entry():\n    return helper()\n\ndef helper():\n    return 4\n";
    let (_root, _store, c) = prepared(source);
    for strategy in ["none", "bfs", "fixed_heat", "spectral_window_v1"] {
        let (pack, work, _) = spectral_acquisition::laboratory(
            &c,
            &request("dependencies", 16384),
            strategy,
            None,
            &flag(),
        )
        .unwrap();
        verify(&pack, source);
        assert_eq!(pack["scope"]["kind"], "dependencies");
        assert_eq!(pack["diagnostics"]["seed_count"], 1);
        assert_eq!(work["verified_document_reads"], 1);
        assert!(
            pack["items"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v["text"].as_str().unwrap().contains("return 4"))
        );
    }
}

#[test]
fn used_external_import_is_delivered_with_body_and_external_gap() {
    let source = "import os\n\ndef entry(amount, callback):\n    return callback(amount) + int(os.environ[\"REMOTE_FEE\"])\n";
    let (_root, _store, c) = prepared(source);
    for strategy in ["none", "bfs", "fixed_heat", "spectral_window_v1"] {
        let (pack, _, _) = spectral_acquisition::laboratory(
            &c,
            &request("dependencies", 16384),
            strategy,
            None,
            &flag(),
        )
        .unwrap();
        verify(&pack, source);
        assert!(
            pack["items"]
                .as_array()
                .unwrap()
                .iter()
                .any(|i| i["start_byte"] == 0
                    && i["end_byte"].as_u64().unwrap() as usize
                        >= source.trim_end_matches('\n').len()),
            "{strategy}: {pack}"
        );
        assert!(
            pack["coverage"]["gaps"]
                .as_array()
                .unwrap()
                .contains(&serde_json::json!("external_or_ambiguous_lexical_import"))
        );
        assert_ne!(pack["acquisition"]["reason"], "spectral_window");
    }
}
#[test]
fn lexical_import_alias_is_selected_without_unrelated_import() {
    let source = "import sys\nimport os as operating\n\ndef entry():\n    return operating.environ[\"REMOTE_FEE\"]\n";
    let (_root, _store, c) = prepared(source);
    let (pack, _, _) =
        spectral_acquisition::laboratory(&c, &request("local", 16384), "none", None, &flag())
            .unwrap();
    verify(&pack, source);
    let text = pack["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["text"].as_str().unwrap())
        .collect::<String>();
    assert!(text.contains("import os as operating"));
    assert!(!text.contains("import sys"));
}

#[test]
fn ranking_top_k_cut_prevents_a_window_after_chunk_deduplication() {
    let source = format!(
        "def entry():\n{}    return 1\n",
        "    # entry\n".repeat(210)
    );
    let (_root, _store, c) = prepared(&source);
    let mut r = request("local", 16384);
    r.mode = "literal".into();
    let pack = spectral_acquisition::retrieve(&c, &r, &flag()).unwrap();
    verify(&pack, &source);
    assert_eq!(pack["diagnostics"]["seed_count"], 1);
    assert_eq!(pack["diagnostics"]["ranking"]["candidate_cut"], true);
    assert_ne!(pack["acquisition"]["reason"], "spectral_window");
    assert!(
        pack["coverage"]["gaps"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!(
                "ranking_candidates_truncated_or_scan_partial"
            ))
    );
}

#[test]
fn invalid_derived_references_and_false_tokens_are_errors_without_panics() {
    use sha2::{Digest, Sha256};
    let source = "def entry():\n    return helper()\n\ndef helper():\n    return 1\n";
    let (_root, _store, c) = prepared(source);
    let path = c.pinned_generation_dir().join("derived.json");
    let original: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    for mutation in [
        "unknown_target",
        "false_token",
        "wrong_existing_target",
        "unknown_parent",
    ] {
        let mut derived = original.clone();
        if mutation == "unknown_parent" {
            let node = derived["data"]["graph"]["nodes"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|n| n["name"] == "entry")
                .unwrap();
            node["parent_id"] = serde_json::json!("missing-parent");
        } else {
            let edge = derived["data"]["graph"]["edges"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|e| e["kind"] == "references" && e["name"] == "helper")
                .unwrap();
            if mutation == "unknown_target" {
                edge["target"] = serde_json::json!("missing-target");
            } else if mutation == "wrong_existing_target" {
                edge["target"] = edge["source"].clone();
            } else {
                edge["evidence"]["start_byte"] = serde_json::json!(4);
                edge["evidence"]["end_byte"] = serde_json::json!(9);
                edge["evidence"]["start_line"] = serde_json::json!(1);
                edge["evidence"]["end_line"] = serde_json::json!(1);
                // Re-identify the forged edge too: the rejection must inspect source bytes.
                let parts = vec![
                    "edge-v1".to_owned(),
                    edge["source"].as_str().unwrap().into(),
                    edge["target"].as_str().unwrap().into(),
                    "References".into(),
                    edge["name"].as_str().unwrap().into(),
                    edge["evidence"]["path_key"].as_str().unwrap().into(),
                    edge["evidence"]["content_sha256"].as_str().unwrap().into(),
                    "4".into(),
                    "9".into(),
                ];
                let mut digest = Sha256::new();
                for part in parts {
                    digest.update((part.len() as u64).to_le_bytes());
                    digest.update(part.as_bytes());
                }
                edge["id"] = serde_json::json!(format!("{:x}", digest.finalize()));
            }
        }
        derived["sha256"] =
            serde_json::json!(model::digest(&codec::canonical(&derived["data"]).unwrap()));
        fs::write(&path, codec::canonical(&derived).unwrap()).unwrap();
        let error = spectral_acquisition::retrieve(&c, &request("dependencies", 16384), &flag())
            .unwrap_err();
        assert_eq!(error.code, "invalid_record", "{mutation}: {error}");
    }
}

#[test]
fn complete_generation_that_exceeds_validation_budget_is_explicitly_limited() {
    let root = tempfile::tempdir().unwrap();
    let store = tempfile::tempdir().unwrap();
    for i in 0..129 {
        fs::write(
            root.path().join(format!("source{i}.py")),
            format!("def entry{i}():\n    return {i}\n"),
        )
        .unwrap();
    }
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
    let c = Collection::open(store.path(), None).unwrap();
    let pack =
        spectral_acquisition::retrieve(&c, &request("dependencies", 16384), &flag()).unwrap();
    assert_eq!(pack["acquisition"]["reason"], "structural_work_limit");
    assert_eq!(pack["structural"]["status"], "not_revalidated");
    assert_eq!(pack["diagnostics"]["work"]["generation_documents"], 129);
    assert!(pack["items"].as_array().unwrap().is_empty());
    verify(&pack, "");
}
