use std::process::Command;
#[test]
fn rd22_command_words_in_query_preserve_search_route() {
    for q in ["init", "d0", "search"] {
        let o = Command::new(env!("CARGO_BIN_EXE_compass"))
            .args([
                "search",
                "--snapshot",
                "/absent-rd",
                "--source",
                "/absent-rd",
                "--query",
                q,
                "--json",
            ])
            .output()
            .unwrap();
        assert_eq!(o.status.code(), Some(1));
        assert_eq!(
            o.stdout,
            b"{\"error\":{\"code\":\"input_invalid\",\"message\":\"input_invalid\"}}"
        );
        assert!(o.stderr.is_empty());
    }
}
#[test]
fn rd18_invalid_arguments_use_exact_envelope() {
    for args in [
        vec!["search", "--json"],
        vec![
            "search",
            "--snapshot",
            "x",
            "--source",
            "x",
            "--query",
            "d0",
            "--max-matches",
            "18446744073709551616",
            "--json",
        ],
    ] {
        let o = Command::new(env!("CARGO_BIN_EXE_compass"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(o.status.code(), Some(2));
        assert_eq!(
            o.stdout,
            b"{\"error\":{\"code\":\"invalid_arguments\",\"message\":\"invalid_arguments\"}}"
        );
    }
}
#[test]
fn invalid_search_without_json_keeps_structured_transport() {
    let o = Command::new(env!("CARGO_BIN_EXE_compass"))
        .arg("search")
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(2));
    assert!(o.stderr.is_empty());
    assert!(o.stdout.ends_with(b"\n"));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["error"]["code"], "invalid_arguments");
}

#[test]
fn bug005_query_json_is_a_value_and_real_flags_select_transport() {
    let value =
        serde_json::json!({"error":{"code":"invalid_arguments","message":"invalid_arguments"}});
    let compact = serde_json::to_vec(&value).unwrap();
    let mut pretty = serde_json::to_vec_pretty(&value).unwrap();
    pretty.push(b'\n');
    for query in [vec!["--query", "--json"], vec!["--query=--json"]] {
        for position in ["none", "global", "before_query", "after_error"] {
            let mut args = vec![];
            if position == "global" {
                args.push("--json");
            }
            args.extend(["search", "--snapshot", "x", "--source", "x"]);
            if position == "before_query" {
                args.push("--json");
            }
            args.extend(query.clone());
            args.extend(["--max-matches", "bad"]);
            if position == "after_error" {
                args.push("--json");
            }
            let o = Command::new(env!("CARGO_BIN_EXE_compass"))
                .args(&args)
                .output()
                .unwrap();
            assert_eq!(o.status.code(), Some(2), "BUG-005.exit {args:?}");
            assert!(o.stderr.is_empty(), "BUG-005.stderr {args:?}");
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&o.stdout).unwrap(),
                value,
                "BUG-005.value {args:?}"
            );
            assert_eq!(
                o.stdout,
                if position == "none" {
                    &pretty
                } else {
                    &compact
                }
                .as_slice(),
                "BUG-005.format {args:?}"
            );
        }
    }
    // Only query permits a hyphenated value: a path option must leave --json a flag.
    let o = Command::new(env!("CARGO_BIN_EXE_compass"))
        .args([
            "search",
            "--snapshot",
            "--json",
            "--source",
            "x",
            "--query",
            "ordinary",
        ])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(2));
    assert!(o.stderr.is_empty());
    assert_eq!(o.stdout, compact);
}

#[test]
fn bug005_valid_hyphenated_queries_keep_typed_transport() {
    for q in ["--json", "--needle", "-x", "d0", "init", "search"] {
        for joined in [false, true] {
            for json in [false, true] {
                let mut args = vec![
                    "search".to_owned(),
                    "--snapshot".into(),
                    "/absent-rd-c01-snapshot".into(),
                    "--source".into(),
                    "/absent-rd-c01-source".into(),
                ];
                if joined {
                    args.push(format!("--query={q}"));
                } else {
                    args.extend(["--query".into(), q.into()]);
                }
                if json {
                    args.push("--json".into());
                }
                let o = Command::new(env!("CARGO_BIN_EXE_compass"))
                    .args(&args)
                    .output()
                    .unwrap();
                assert_eq!(o.status.code(), Some(1));
                assert!(o.stderr.is_empty());
                let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
                assert_eq!(v["error"]["code"], "input_invalid");
                assert_eq!(
                    o.stdout.ends_with(b"\n"),
                    !json,
                    "BUG-005.typed-format {args:?}"
                );
            }
        }
    }
}
