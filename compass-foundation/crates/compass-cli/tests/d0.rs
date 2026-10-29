use serde_json::Value;
use std::{fs, process::Command};
fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_compass"))
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn public_help_arguments_and_canonical_error_envelopes() {
    let help = run(&["d0", "--help"]);
    assert!(help.status.success());
    assert!(
        String::from_utf8(help.stdout)
            .unwrap()
            .contains("--baseline-source")
    );
    for args in [
        vec!["d0", "--json"],
        vec![
            "d0",
            "--baseline",
            "x",
            "--current",
            "y",
            "--deadline-ms",
            "0",
            "--json",
        ],
        vec![
            "d0",
            "--baseline",
            "x",
            "--current",
            "y",
            "--unknown",
            "--json",
        ],
    ] {
        let out = run(&args);
        assert_eq!(out.status.code(), Some(2));
        assert_eq!(
            out.stdout,
            b"{\"error\":{\"code\":\"invalid_arguments\",\"message\":\"invalid_arguments\"}}"
        );
    }
    let out = run(&[
        "d0",
        "--baseline",
        "/absent-i1-a",
        "--current",
        "/absent-i1-b",
        "--json",
    ]);
    assert_eq!(out.status.code(), Some(1));
    assert!(!out.stdout.ends_with(b"\n"));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["error"]["code"], "input_invalid");
    let temp = tempfile::tempdir().unwrap();
    let p = temp.path().join("limits.json");
    fs::write(&p, b"{}").unwrap();
    let out = run(&[
        "d0",
        "--baseline",
        "x",
        "--current",
        "y",
        "--limits",
        p.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn strict_manifest_profiles_limits_and_empty_source_envelopes() {
    use serde_json::json;
    let temp = tempfile::tempdir().unwrap();
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../research/proposals/d0-c2/fixtures");
    let a = temp.path().join("a");
    let b = temp.path().join("b");
    let profile = temp.path().join("limits");
    let valid = fs::read(fixture.join("manifests/empty.json")).unwrap();
    fs::write(&b, &valid).unwrap();
    let invoke = |limit: bool| {
        let mut args = vec![
            "d0",
            "--baseline",
            a.to_str().unwrap(),
            "--current",
            b.to_str().unwrap(),
            "--json",
        ];
        if limit {
            args.extend(["--limits", profile.to_str().unwrap()]);
        }
        run(&args)
    };
    fs::write(&a, &valid).unwrap();
    assert!(invoke(false).status.success());
    for raw in [
        b"{\"x\":1,\"x\":2}".as_slice(),
        b"\xef\xbb\xbf{}",
        b"{} {}",
        b"\xff",
        b"NaN",
    ] {
        fs::write(&a, raw).unwrap();
        let out = invoke(false);
        assert_eq!(out.status.code(), Some(1));
        let v: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(v["error"]["code"], "input_invalid");
        assert_eq!(v["error"]["message"], "input_invalid");
        assert!(!out.stdout.ends_with(b"\n"));
    }
    fs::write(&a, &valid).unwrap();
    let defaults: Value =
        serde_json::from_slice(&fs::read(fixture.join("limits-default.json")).unwrap()).unwrap();
    for field in [
        "max_json_depth",
        "max_records",
        "max_manifest_bytes",
        "max_output_bytes",
    ] {
        let mut limits = defaults.clone();
        limits[field] = json!("1");
        fs::write(&profile, serde_json::to_vec(&limits).unwrap()).unwrap();
        let out = invoke(true);
        assert_eq!(out.status.code(), Some(1));
        let v: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(
            v["error"]["code"],
            if field == "max_output_bytes" {
                "output_limit"
            } else {
                "input_limit"
            }
        );
    }
    for value in [
        json!({}),
        json!({"max_file_bytes":"1"}),
        {
            let mut v = defaults.clone();
            v["max_file_bytes"] = json!("0");
            v
        },
        {
            let mut v = defaults.clone();
            v["max_file_bytes"] = json!(4194304);
            v
        },
        {
            let mut v = defaults.clone();
            v["max_file_bytes"] = json!("4194305");
            v
        },
    ] {
        fs::write(&profile, serde_json::to_vec(&value).unwrap()).unwrap();
        let out = invoke(true);
        assert_eq!(out.status.code(), Some(2));
        let v: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(v["error"]["code"], "invalid_arguments");
    }
    let out = run(&[
        "d0",
        "--baseline",
        a.to_str().unwrap(),
        "--current",
        b.to_str().unwrap(),
        "--baseline-source",
        temp.path().join("missing").to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(out.status.code(), Some(1));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["error"]["code"], "source_invalid");
}

#[test]
fn noncanonical_manifest_correct_raw_binding_and_human_mode() {
    use compass_core::model::digest;
    let temp = tempfile::tempdir().unwrap();
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../research/proposals/d0-c2/fixtures");
    let snapshot: Value =
        serde_json::from_slice(&fs::read(fixture.join("manifests/empty.json")).unwrap()).unwrap();
    let raw = serde_json::to_vec_pretty(&snapshot).unwrap();
    let path = temp.path().join("manifest");
    fs::write(&path, &raw).unwrap();
    let mut desc: Value =
        serde_json::from_slice(&fs::read(fixture.join("packages/empty/source.json")).unwrap())
            .unwrap();
    desc["snapshot_file_sha256"] = Value::String(digest(&raw));
    fs::write(
        temp.path().join("source.json"),
        serde_json::to_vec_pretty(&desc).unwrap(),
    )
    .unwrap();
    let mut args = vec![
        "d0",
        "--baseline",
        path.to_str().unwrap(),
        "--current",
        path.to_str().unwrap(),
        "--baseline-source",
        temp.path().to_str().unwrap(),
        "--current-source",
        temp.path().to_str().unwrap(),
    ];
    let human = run(&args);
    assert!(human.status.success());
    let text = String::from_utf8(human.stdout).unwrap();
    assert!(text.contains("integral:") && text.contains("known subset"));
    args.push("--json");
    let out = run(&args);
    assert!(out.status.success());
    assert!(!out.stdout.ends_with(b"\n"));
    let value: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(value["measurement"]["zero_basis"], "empty_universe");
    assert_eq!(value["baseline"]["snapshot_file_sha256"], digest(&raw));
    assert_eq!(value["producer_build"], compass_core::BUILD_ID);
}
