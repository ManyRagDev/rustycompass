use serde_json::json;
use std::process::Command;
#[test]
fn path_value_does_not_select_output_mode_on_parser_error() {
    let error = json!({"error":{"code":"invalid_arguments","message":"invalid_arguments"}});
    let compact = serde_json::to_vec(&error).unwrap();
    let mut pretty = serde_json::to_vec_pretty(&error).unwrap();
    pretty.push(b'\n');
    for key in [vec!["--path-key", "--json"], vec!["--path-key=--json"]] {
        for position in ["none", "global", "before", "after"] {
            let mut args = vec![];
            if position == "global" {
                args.push("--json")
            }
            args.extend(["read", "--snapshot", "x", "--source", "x"]);
            if position == "before" {
                args.push("--json")
            }
            args.extend(key.clone());
            args.extend(["--start-line", "bad"]);
            if position == "after" {
                args.push("--json")
            }
            let o = Command::new(env!("CARGO_BIN_EXE_compass"))
                .args(args)
                .output()
                .unwrap();
            assert_eq!(o.status.code(), Some(2));
            assert!(o.stderr.is_empty());
            assert_eq!(
                o.stdout,
                if position == "none" {
                    &pretty
                } else {
                    &compact
                }
                .as_slice()
            );
        }
    }
}
#[test]
fn invalid_numbers_keys_and_missing_options_are_structured_errors() {
    for (option, value) in [
        ("--start-line", "0"),
        ("--start-line", "-1"),
        ("--start-line", "1.5"),
        ("--start-line", "18446744073709551616"),
        ("--max-lines", "0"),
        ("--max-lines", "1001"),
        ("--deadline-ms", "0"),
    ] {
        let o = Command::new(env!("CARGO_BIN_EXE_compass"))
            .args([
                "read",
                "--snapshot",
                "x",
                "--source",
                "x",
                "--path-key",
                "YS50eHQ",
                option,
                value,
                "--json",
            ])
            .output()
            .unwrap();
        assert_eq!(o.status.code(), Some(2));
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&o.stdout).unwrap()["error"]["code"],
            "invalid_arguments"
        );
        assert!(o.stderr.is_empty());
    }
    for key in ["", "Lw", "Li4", "YQ==", "--json"] {
        let o = Command::new(env!("CARGO_BIN_EXE_compass"))
            .args([
                "read",
                "--snapshot",
                "x",
                "--source",
                "x",
                "--path-key",
                key,
            ])
            .output()
            .unwrap();
        assert_eq!(o.status.code(), Some(2));
        assert!(o.stdout.ends_with(b"\n"));
        assert!(o.stderr.is_empty());
    }
}
