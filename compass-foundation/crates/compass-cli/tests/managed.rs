use serde_json::Value;
use std::{fs, path::Path, process::Command};
fn call(args: &[&str]) -> (i32, Value) {
    let output = Command::new(env!("CARGO_BIN_EXE_compass"))
        .arg("--json")
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.stderr.is_empty(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value = serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "invalid JSON {e}: {}",
            String::from_utf8_lossy(&output.stdout)
        )
    });
    (output.status.code().unwrap(), value)
}
fn path(p: &Path) -> &str {
    p.to_str().unwrap()
}
#[test]
fn prepare_query_update_old_generation_and_read() {
    let root = tempfile::tempdir().unwrap();
    let store = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join("lib.rs"),
        "pub fn getUserProfile() { let revision = 1; }\n",
    )
    .unwrap();
    fs::write(root.path().join(".env"), "secret credentials").unwrap();
    let (status, prepared) = call(&[
        "collection",
        "prepare",
        "--root",
        path(root.path()),
        "--store",
        path(store.path()),
        "--max-disk-bytes",
        "16777216",
    ]);
    assert_eq!(status, 0, "{prepared}");
    let generation = prepared["generation"].as_str().unwrap();
    let (status, query) = call(&[
        "query",
        "--store",
        path(store.path()),
        "--query",
        "getUserProfile",
        "--mode",
        "lexical",
    ]);
    assert_eq!(status, 0, "{query}");
    assert!(!query["results"].as_array().unwrap().is_empty());
    let key = query["results"][0]["path_key"].as_str().unwrap();
    let (status, read) = call(&[
        "read",
        "--snapshot",
        prepared["snapshot"].as_str().unwrap(),
        "--source",
        prepared["source"].as_str().unwrap(),
        "--path-key",
        key,
    ]);
    assert_eq!(status, 0, "{read}");
    assert!(
        read["excerpt"]["text"]
            .as_str()
            .unwrap()
            .contains("revision = 1")
    );
    fs::write(
        root.path().join("lib.rs"),
        "pub fn getUserProfile() { let revision = 2; }\n",
    )
    .unwrap();
    let (status, updated) = call(&["collection", "update", "--store", path(store.path())]);
    assert_eq!(status, 0, "{updated}");
    assert_ne!(updated["generation"], prepared["generation"]);
    let (status, old) = call(&[
        "query",
        "--store",
        path(store.path()),
        "--generation",
        generation,
        "--query",
        "getUserProfile",
    ]);
    assert_eq!(status, 0, "{old}");
    assert!(
        old["results"][0]["text"]
            .as_str()
            .unwrap()
            .contains("revision = 1")
    );
    let (status, current) = call(&[
        "query",
        "--store",
        path(store.path()),
        "--query",
        "getUserProfile",
    ]);
    assert_eq!(status, 0, "{current}");
    assert!(
        current["results"][0]["text"]
            .as_str()
            .unwrap()
            .contains("revision = 2")
    );
    let (status, files) = call(&["files", "--store", path(store.path())]);
    assert_eq!(status, 0, "{files}");
    assert_eq!(files["total"], 1);
    let (status, status_body) = call(&["collection", "status", "--store", path(store.path())]);
    assert_eq!(status, 0, "{status_body}");
    assert_eq!(status_body["derived"]["status"], "available");
}
#[test]
fn pagination_requires_generation_before_access() {
    let (status, error) = call(&[
        "query",
        "--store",
        "/does/not/exist",
        "--query",
        "word",
        "--offset",
        "1",
    ]);
    assert_eq!(status, 2);
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("requires --generation")
    );
    let (status, error) = call(&["files", "--store", "/does/not/exist", "--offset", "1"]);
    assert_eq!(status, 2);
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("requires --generation")
    );
}
#[test]
fn retrieve_cursor_pins_generation_across_update() {
    let root = tempfile::tempdir().unwrap();
    let store = tempfile::tempdir().unwrap();
    for i in 0..30 {
        fs::write(
            root.path().join(format!("file_{i}.rs")),
            format!("pub fn sharedSymbol{i}() {{ let sharedToken = {i}; }}\n"),
        )
        .unwrap();
    }
    let (status, prepared) = call(&[
        "collection",
        "prepare",
        "--root",
        path(root.path()),
        "--store",
        path(store.path()),
        "--max-disk-bytes",
        "16777216",
    ]);
    assert_eq!(status, 0, "{prepared}");
    let (status, pack) = call(&[
        "retrieve",
        "--store",
        path(store.path()),
        "--query",
        "sharedToken",
        "--mode",
        "lexical",
        "--budget-bytes",
        "4096",
    ]);
    assert_eq!(status, 0, "{pack}");
    assert!(serde_json::to_vec(&pack).unwrap().len() <= 4096);
    let cursor = pack["next_cursor"].as_str().expect("continuation expected");
    fs::write(
        root.path().join("new.rs"),
        "pub fn changed() { let sharedToken = 99; }\n",
    )
    .unwrap();
    let (status, updated) = call(&["collection", "update", "--store", path(store.path())]);
    assert_eq!(status, 0, "{updated}");
    let (status, continued) = call(&[
        "retrieve",
        "--store",
        path(store.path()),
        "--query",
        "sharedToken",
        "--mode",
        "lexical",
        "--budget-bytes",
        "4096",
        "--cursor",
        cursor,
    ]);
    assert_eq!(status, 0, "{continued}");
    assert_eq!(continued["generation"], prepared["generation"]);
    let (status, error) = call(&[
        "retrieve",
        "--store",
        path(store.path()),
        "--generation",
        updated["generation"].as_str().unwrap(),
        "--query",
        "sharedToken",
        "--mode",
        "lexical",
        "--budget-bytes",
        "4096",
        "--cursor",
        cursor,
    ]);
    assert_eq!(status, 2, "{error}");
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("cursor generation")
    );
    let (status, neighbors) = call(&[
        "neighbors",
        "--store",
        path(store.path()),
        "--query",
        "sharedSymbol1",
    ]);
    assert_eq!(status, 0, "{neighbors}");
    assert!(!neighbors["nodes"].as_array().unwrap().is_empty());
}

#[test]
fn contract_acquisition_cli_keeps_legacy_pack_and_checks_explicit_obligations() {
    let root = tempfile::tempdir().unwrap();
    let store = tempfile::tempdir().unwrap();
    let contracts = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join("lib.rs"),
        "pub fn requiredSymbol() { let marker = 1; }\n",
    )
    .unwrap();
    let (code, prepared) = call(&[
        "collection",
        "prepare",
        "--root",
        path(root.path()),
        "--store",
        path(store.path()),
        "--max-disk-bytes",
        "16777216",
    ]);
    assert_eq!(code, 0, "{prepared}");
    let args = [
        "retrieve",
        "--store",
        path(store.path()),
        "--query",
        "requiredSymbol",
        "--mode",
        "lexical",
        "--budget-bytes",
        "8192",
    ];
    let (code, legacy) = call(&args);
    assert_eq!(code, 0, "{legacy}");
    assert_eq!(legacy["schema_version"], "1");
    assert!(legacy.get("completion").is_none());
    let contract_path = contracts.path().join("contract.json");
    let contract = serde_json::json!({"schema_version":"1","intent":"locate definition","obligations":[{"id":"definition","kind":"symbol_definition","symbol":{"name":"requiredSymbol"}}],"interpretation_pending":["interpret its purpose"]});
    fs::write(&contract_path, serde_json::to_vec(&contract).unwrap()).unwrap();
    let mut bound = args.to_vec();
    bound.extend(["--contract", path(&contract_path)]);
    let (code, acquired) = call(&bound);
    assert_eq!(code, 0, "{acquired}");
    assert_eq!(acquired["schema_version"], "2");
    assert_eq!(
        acquired["completion"]["status"], "INSUFFICIENT_EVIDENCE",
        "interpretation remains pending: {acquired}"
    );
    let mut contract = contract;
    contract["interpretation_pending"] = serde_json::json!([]);
    fs::write(&contract_path, serde_json::to_vec(&contract).unwrap()).unwrap();
    let (code, acquired) = call(&bound);
    assert_eq!(code, 0, "{acquired}");
    assert_eq!(
        acquired["completion"]["status"], "SUFFICIENT_EVIDENCE_ACQUIRED",
        "{acquired}"
    );
    assert!(serde_json::to_vec(&acquired).unwrap().len() <= 8192);
    let mut missing = contract.clone();
    missing["obligations"][0]["symbol"]["name"] = "missingSymbol".into();
    fs::write(&contract_path, serde_json::to_vec(&missing).unwrap()).unwrap();
    let (code, acquired) = call(&bound);
    assert_eq!(code, 0, "{acquired}");
    assert_eq!(
        acquired["completion"]["status"], "INSUFFICIENT_EVIDENCE",
        "{acquired}"
    );
    let (code, unchanged) = call(&args);
    assert_eq!(code, 0);
    assert_eq!(legacy, unchanged);
}

#[test]
fn malformed_cli_contracts_are_rejected_before_store_access() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("contract.json");
    for raw in ["null".to_owned(), "{".to_owned(), " ".repeat(65537), serde_json::json!({"schema_version":"1","intent":"x","obligations":[{"id":"x","kind":"symbol_definition","symbol":{"node_id":"unbound"}}],"interpretation_pending":[]}).to_string()] {
        fs::write(&file, raw).unwrap();
        let (code, error) = call(&["retrieve", "--store", "/does/not/exist", "--query", "x", "--mode", "lexical", "--contract", path(&file)]);
        assert_eq!(code, 2, "{error}");
        assert_ne!(error["error"]["code"], "io_error", "must validate before opening store: {error}");
        assert!(!error.to_string().contains("No such file"), "{error}");
    }
}

#[test]
fn spectral_policy_is_terminal_and_rejects_conflicting_cli_options() {
    let root = tempfile::tempdir().unwrap();
    let store = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join("main.py"),
        "def entry(callback):\n    return callback()\n",
    )
    .unwrap();
    let (code, prepared) = call(&[
        "collection",
        "prepare",
        "--root",
        path(root.path()),
        "--store",
        path(store.path()),
    ]);
    assert_eq!(code, 0, "{prepared}");
    let base = [
        "retrieve",
        "--store",
        path(store.path()),
        "--query",
        "entry",
        "--acquisition-policy",
        "spectral_window_v1",
        "--scope",
        "dependencies",
    ];
    let (code, pack) = call(&base);
    assert_eq!(code, 0, "{pack}");
    assert_eq!(pack["schema_version"], "3");
    assert_eq!(pack["scope"]["kind"], "dependencies");
    assert_eq!(pack["acquisition"]["state"], "terminal");
    assert!(pack["next_cursor"].is_null());
    assert!(!pack["coverage"]["gaps"].as_array().unwrap().is_empty());
    for extra in [
        vec!["--theta", "1"],
        vec!["--expansion", "heat"],
        vec!["--cursor", "not-a-cursor"],
    ] {
        let mut args = base.to_vec();
        args.extend(extra);
        let (code, error) = call(&args);
        assert_ne!(code, 0);
        assert!(error.to_string().contains("rejects"), "{error}");
    }
    let (code, legacy) = call(&[
        "retrieve",
        "--store",
        path(store.path()),
        "--query",
        "entry",
    ]);
    assert_eq!(code, 0);
    assert_eq!(legacy["schema_version"], "1");
}
