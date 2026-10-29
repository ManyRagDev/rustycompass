//! Independent public stdio controls for configured managed collections.
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{self, Receiver},
    time::Duration,
};
struct Fixture {
    _root: tempfile::TempDir,
    _store: tempfile::TempDir,
    configs: tempfile::TempDir,
    prepared: Value,
}
fn cli(args: &[&str]) -> Value {
    let o = Command::new(env!("CARGO_BIN_EXE_compass"))
        .arg("--json")
        .args(args)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stdout));
    serde_json::from_slice(&o.stdout).unwrap()
}
impl Fixture {
    fn new(large: bool) -> Self {
        let root = tempfile::tempdir().unwrap();
        let store = tempfile::tempdir().unwrap();
        for i in 0..30 {
            fs::write(
                root.path().join(format!("file_{i:02}.rs")),
                format!("pub fn sharedSymbol{i}() {{ let sharedToken = 1; }}\r\n// old revision\n"),
            )
            .unwrap();
        }
        if large {
            fs::write(
                root.path().join("large.txt"),
                "large_marker ".repeat(100_000),
            )
            .unwrap();
        }
        let prepared = cli(&[
            "collection",
            "prepare",
            "--root",
            root.path().to_str().unwrap(),
            "--store",
            store.path().to_str().unwrap(),
            "--max-disk-bytes",
            "33554432",
        ]);
        Self {
            _root: root,
            _store: store,
            configs: tempfile::tempdir().unwrap(),
            prepared,
        }
    }
    fn config(&self, legacy: bool, deadline: u64, max_output: Option<u64>) -> PathBuf {
        let source = if legacy {
            json!({"snapshot":self.prepared["snapshot"],"source":self.prepared["source"]})
        } else {
            json!({"collection":self._store.path()})
        };
        let mut c = json!({"sources":{"approved":source},"deadline_ms":deadline});
        if let Some(cap) = max_output {
            c["limits"] = json!({"max_manifest_bytes":"16777216","max_source_descriptor_bytes":"65536","max_records":"10000","max_json_depth":"64","max_file_bytes":"4194304","max_total_object_bytes":"67108864","max_output_bytes":cap.to_string(),"max_logical_memory_bytes":"134217728","max_line_bytes":"4096"});
        }
        let path = self
            .configs
            .path()
            .join(format!("{legacy}-{deadline}-{:?}.json", max_output));
        fs::write(&path, serde_json::to_vec(&c).unwrap()).unwrap();
        path
    }
    fn update(&self) -> Value {
        fs::write(
            self._root.path().join("file_00.rs"),
            "pub fn sharedSymbol0() { let sharedToken = 2; }\n// new revision\n",
        )
        .unwrap();
        cli(&[
            "collection",
            "update",
            "--store",
            self._store.path().to_str().unwrap(),
        ])
    }
}
struct Session {
    child: Child,
    input: ChildStdin,
    output: Receiver<Value>,
    next: u64,
}
impl Session {
    fn new(config: &Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_compass"))
            .args(["mcp", "--config", config.to_str().unwrap()])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let input = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let (tx, output) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                let v = serde_json::from_str(&line)
                    .unwrap_or_else(|e| panic!("bad MCP JSON {e}: {line}"));
                if tx.send(v).is_err() {
                    break;
                }
            }
        });
        let mut s = Self {
            child,
            input,
            output,
            next: 1,
        };
        let init=s.request("initialize",json!({"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"independent-managed-review","version":"1"}}));
        assert_eq!(init["result"]["protocolVersion"], "2025-11-25");
        s.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
        s
    }
    fn send(&mut self, value: Value) {
        serde_json::to_writer(&mut self.input, &value).unwrap();
        self.input.write_all(b"\n").unwrap();
        self.input.flush().unwrap();
    }
    fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next;
        self.next += 1;
        self.send(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}));
        let reply = self
            .output
            .recv_timeout(Duration::from_secs(10))
            .expect("MCP request produced no response within 10s");
        assert_eq!(reply["id"], id, "{reply}");
        reply
    }
    fn tool(&mut self, name: &str, args: Value) -> Value {
        let reply = self.request("tools/call", json!({"name":name,"arguments":args}));
        assert!(
            reply.get("error").is_none(),
            "advertised tool was rejected at protocol level: {reply}"
        );
        reply["result"].clone()
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
#[test]
fn advertised_managed_tools_execute_and_legacy_catalog_remains_two() {
    let f = Fixture::new(false);
    let mut legacy = Session::new(&f.config(true, 30000, None));
    assert_eq!(
        legacy.request("tools/list", json!({}))["result"]["tools"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let mut managed = Session::new(&f.config(false, 30000, None));
    let tools = managed.request("tools/list", json!({}));
    assert_eq!(tools["result"]["tools"].as_array().unwrap().len(), 6);
    for (name, args) in [
        ("compass_files", json!({"source":"approved","limit":2})),
        (
            "compass_query",
            json!({"source":"approved","query":"sharedToken","mode":"lexical","limit":2}),
        ),
        (
            "compass_neighbors",
            json!({"source":"approved","query":"sharedSymbol0","limit":2}),
        ),
        (
            "compass_retrieve",
            json!({"source":"approved","query":"sharedToken","mode":"lexical","budget_bytes":4096}),
        ),
    ] {
        let reply = managed.tool(name, args);
        assert_eq!(reply["isError"], false, "{name}: {reply}");
        assert_eq!(
            serde_json::from_str::<Value>(reply["content"][0]["text"].as_str().unwrap()).unwrap(),
            reply["structuredContent"]
        );
        assert_eq!(
            reply["_meta"]["compass"]["tool_result_bytes"]
                .as_u64()
                .unwrap() as usize,
            serde_json::to_vec(&reply).unwrap().len()
        );
    }
}
#[test]
fn managed_arguments_cannot_select_unconfigured_paths_or_cross_tool_options() {
    let f = Fixture::new(false);
    let mut s = Session::new(&f.config(false, 30000, None));
    for (name, args) in [
        ("compass_files", json!({"source":"/tmp/unapproved"})),
        (
            "compass_files",
            json!({"source":"approved","collection":"/tmp/unapproved"}),
        ),
        ("compass_files", json!({"source":"approved","theta":1})),
        (
            "compass_query",
            json!({"source":"approved","query":"sharedToken","node_id":"x"}),
        ),
        ("compass_neighbors", json!({"source":"approved","offset":1})),
        (
            "compass_retrieve",
            json!({"source":"approved","query":"sharedToken","limit":1}),
        ),
        (
            "compass_files",
            json!({"source":"approved","generation":"../../outside"}),
        ),
        ("compass_files", json!({"source":"approved","offset":1})),
    ] {
        let reply = s.tool(name, args);
        assert_eq!(reply["isError"], true, "{name}: {reply}");
        assert!(reply["structuredContent"].get("results").is_none());
    }
}
#[test]
fn generation_and_cursor_continuations_keep_old_revision_after_update() {
    let f = Fixture::new(false);
    let old = f.prepared["generation"].as_str().unwrap();
    let mut s = Session::new(&f.config(false, 30000, None));
    let pack = s.tool(
        "compass_retrieve",
        json!({"source":"approved","query":"sharedToken","mode":"lexical","budget_bytes":4096}),
    );
    assert_eq!(pack["isError"], false, "{pack}");
    let cursor = pack["structuredContent"]["next_cursor"]
        .as_str()
        .expect("fixture must require continuation")
        .to_owned();
    let updated = f.update();
    let new = updated["generation"].as_str().unwrap();
    let continuation=s.tool("compass_retrieve",json!({"source":"approved","query":"sharedToken","mode":"lexical","budget_bytes":4096,"cursor":cursor}));
    assert_eq!(continuation["isError"], false, "{continuation}");
    assert_eq!(continuation["structuredContent"]["generation"], old);
    let mismatch=s.tool("compass_retrieve",json!({"source":"approved","query":"sharedToken","mode":"lexical","budget_bytes":4096,"cursor":cursor,"generation":new}));
    assert_eq!(mismatch["isError"], true);
    let missing = s.tool(
        "compass_read",
        json!({"source":"approved","path_key":"ZmlsZV8wMC5ycw","max_lines":1}),
    );
    assert_eq!(
        missing["structuredContent"]["error"]["code"],
        "generation_required"
    );
    let read = s.tool(
        "compass_read",
        json!({"source":"approved","path_key":"ZmlsZV8wMC5ycw","generation":old,"max_lines":1}),
    );
    assert_eq!(read["isError"], false, "{read}");
    assert!(
        read["structuredContent"]["excerpt"]["text"]
            .as_str()
            .unwrap()
            .contains("= 1")
    );
}
#[test]
fn managed_output_limit_returns_error_without_success_payload() {
    let f = Fixture::new(false);
    let mut s = Session::new(&f.config(false, 30000, Some(600)));
    let reply = s.tool("compass_files", json!({"source":"approved","limit":30}));
    assert_eq!(reply["isError"], true, "{reply}");
    assert!(
        reply["structuredContent"]["error"]["message"]
            .as_str()
            .unwrap()
            .contains("output_limit")
    );
    assert!(reply["structuredContent"].get("results").is_none());
}
#[test]
fn managed_deadline_produces_explicit_response_and_keeps_server_usable() {
    let f = Fixture::new(true);
    let mut s = Session::new(&f.config(false, 1, None));
    let reply = s.tool("compass_files", json!({"source":"approved","limit":1}));
    assert_eq!(reply["isError"], true, "{reply}");
    assert_eq!(
        reply["structuredContent"]["error"]["code"], "deadline_exceeded",
        "{reply}"
    );
    assert_eq!(s.request("ping", json!({}))["result"], json!({}));
}

#[test]
fn managed_client_cancellation_suppresses_reply_and_allows_next_request() {
    let f = Fixture::new(true);
    let mut s = Session::new(&f.config(false, 30000, None));
    s.send(json!({"jsonrpc":"2.0","id":100,"method":"tools/call","params":{"name":"compass_files","arguments":{"source":"approved","limit":1}}}));
    s.send(json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":100,"reason":"independent cancel control"}}));
    s.send(json!({"jsonrpc":"2.0","id":101,"method":"ping","params":{}}));
    let ping = s.output.recv_timeout(Duration::from_secs(3)).unwrap();
    assert_eq!(ping["id"], 101, "cancelled call returned data: {ping}");
    assert_eq!(ping["result"], json!({}));
    assert!(
        s.output.recv_timeout(Duration::from_millis(500)).is_err(),
        "cancelled request emitted a late response"
    );
    let next = s.tool("compass_files", json!({"source":"approved","limit":1}));
    assert_eq!(next["isError"], false, "{next}");
}

#[test]
fn indexed_tools_explicitly_reject_unsupported_reduced_legacy_caps() {
    let f = Fixture::new(false);
    let path = f.config(false, 30000, Some(1048576));
    let mut config: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    config["limits"]["max_file_bytes"] = json!("1");
    fs::write(&path, serde_json::to_vec(&config).unwrap()).unwrap();
    let mut s = Session::new(&path);
    for name in [
        "compass_files",
        "compass_query",
        "compass_neighbors",
        "compass_retrieve",
    ] {
        let args = match name {
            "compass_files" => json!({"source":"approved","limit":1}),
            "compass_neighbors" => json!({"source":"approved","query":"sharedSymbol0"}),
            _ => json!({"source":"approved","query":"sharedToken","mode":"lexical"}),
        };
        let reply = s.tool(name, args);
        assert_eq!(
            reply["isError"], true,
            "custom cap was silently ignored by {name}: {reply}"
        );
        assert!(
            reply["structuredContent"]["error"]["message"]
                .as_str()
                .unwrap()
                .contains("legacy_limit_profile_not_supported_for_indexed_tools"),
            "{reply}"
        );
        assert!(reply["structuredContent"].get("results").is_none());
    }
}

#[test]
fn acquisition_contract_mcp_matches_cli_and_keeps_legacy_catalog() {
    let f = Fixture::new(false);
    let mut s = Session::new(&f.config(false, 30000, None));
    let catalog = s.request("tools/list", json!({}));
    let tools = catalog["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 6);
    let retrieve = tools
        .iter()
        .find(|t| t["name"] == "compass_retrieve")
        .unwrap();
    assert_eq!(
        retrieve["inputSchema"]["properties"]["contract"]["type"],
        "object"
    );
    let base =
        json!({"source":"approved","query":"sharedSymbol0","mode":"lexical","budget_bytes":8192});
    let legacy = s.tool("compass_retrieve", base.clone());
    assert_eq!(legacy["isError"], false, "{legacy}");
    assert!(legacy["structuredContent"].get("completion").is_none());
    let contract = json!({"schema_version":"1","intent":"locate definition","obligations":[{"id":"definition","kind":"symbol_definition","symbol":{"name":"sharedSymbol0"}}],"interpretation_pending":[]});
    let contract_path = f.configs.path().join("contract.json");
    fs::write(&contract_path, serde_json::to_vec(&contract).unwrap()).unwrap();
    let mut args = base.clone();
    args["contract"] = contract.clone();
    let acquired = s.tool("compass_retrieve", args.clone());
    assert_eq!(acquired["isError"], false, "{acquired}");
    let pack = &acquired["structuredContent"];
    assert_eq!(pack["schema_version"], "2");
    assert_eq!(
        pack["completion"]["status"], "SUFFICIENT_EVIDENCE_ACQUIRED",
        "{acquired}"
    );
    assert_eq!(
        serde_json::from_str::<Value>(acquired["content"][0]["text"].as_str().unwrap()).unwrap(),
        *pack
    );
    assert!(serde_json::to_vec(pack).unwrap().len() <= 8192);
    let from_cli = cli(&[
        "retrieve",
        "--store",
        f._store.path().to_str().unwrap(),
        "--query",
        "sharedSymbol0",
        "--mode",
        "lexical",
        "--budget-bytes",
        "8192",
        "--contract",
        contract_path.to_str().unwrap(),
    ]);
    assert_eq!(*pack, from_cli);
    args["contract"]["obligations"][0]["symbol"]["name"] = "missingSymbol".into();
    let missing = s.tool("compass_retrieve", args);
    assert_eq!(missing["isError"], false, "{missing}");
    assert_eq!(
        missing["structuredContent"]["completion"]["status"],
        "INSUFFICIENT_EVIDENCE"
    );
    assert_eq!(
        s.tool("compass_retrieve", base)["structuredContent"],
        legacy["structuredContent"]
    );
    for bad in [
        Value::Null,
        json!({}),
        json!({"schema_version":"1","intent":"x","obligations":[{"id":"x","kind":"symbol_definition","symbol":{"node_id":"unbound"}}],"interpretation_pending":[]}),
        json!({"schema_version":"1","intent":"x".repeat(17000),"obligations":[{"id":"x","kind":"symbol_definition","symbol":{"name":"sharedSymbol0"}}],"interpretation_pending":[]}),
    ] {
        let result = s.tool(
            "compass_retrieve",
            json!({"source":"approved","query":"x","mode":"lexical","contract":bad}),
        );
        assert_eq!(result["isError"], true, "{result}");
        assert!(result["structuredContent"].get("completion").is_none());
    }
    let unknown = s.tool("compass_retrieve", json!({"source":"approved","query":"x","mode":"lexical","budget_bytes":8192,"contract":{"schema_version":"1","intent":"unsupported condition","obligations":[{"id":"unknown","kind":"business_interpretation","question":"what does it mean?"}],"interpretation_pending":[]}}));
    assert_eq!(unknown["isError"], false, "{unknown}");
    assert_eq!(
        unknown["structuredContent"]["completion"]["status"],
        "INSUFFICIENT_EVIDENCE"
    );
    let disallowed = s.tool(
        "compass_query",
        json!({"source":"approved","query":"x","contract":contract}),
    );
    assert_eq!(disallowed["isError"], true);
}

#[test]
fn acquisition_cursor_is_bound_to_contract_and_generation() {
    let mut f = Fixture::new(false);
    let mut large = String::from("pub fn paginatedDefinition() {\n");
    for i in 0..500 {
        large.push_str(&format!(
            "    let padding{i} = \"retained original source line\";\n"
        ));
    }
    large.push_str("}\n");
    fs::write(f._root.path().join("paginated.rs"), large).unwrap();
    f.prepared = f.update();
    let mut s = Session::new(&f.config(false, 30000, None));
    let contract = json!({"schema_version":"1","intent":"locate all requested evidence","obligations":[{"id":"definition","kind":"symbol_definition","symbol":{"name":"paginatedDefinition"}}],"interpretation_pending":[]});
    let mut args = json!({"source":"approved","query":"sharedToken","mode":"lexical","budget_bytes":8192,"contract":contract});
    let first = s.tool("compass_retrieve", args.clone());
    assert_eq!(first["isError"], false, "{first}");
    let cursor = first["structuredContent"]["next_cursor"]
        .as_str()
        .expect("fixture needs continuation")
        .to_owned();
    let updated = f.update();
    args["cursor"] = cursor.clone().into();
    let second = s.tool("compass_retrieve", args.clone());
    assert_eq!(second["isError"], false, "{second}");
    assert_eq!(
        second["structuredContent"]["generation"],
        f.prepared["generation"]
    );
    args["contract"]["intent"] = "changed intent".into();
    assert_eq!(s.tool("compass_retrieve", args.clone())["isError"], true);
    args["contract"]["intent"] = "locate all requested evidence".into();
    args["generation"] = updated["generation"].clone();
    assert_eq!(s.tool("compass_retrieve", args.clone())["isError"], true);
    args.as_object_mut().unwrap().remove("generation");
    args.as_object_mut().unwrap().remove("contract");
    assert_eq!(s.tool("compass_retrieve", args)["isError"], true);
}

#[test]
fn spectral_policy_schema_stdio_terminal_and_explicit_conflicts() {
    let f = Fixture::new(false);
    let mut s = Session::new(&f.config(false, 30000, None));
    let tools = s.request("tools/list", json!({}));
    let tool = tools["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == "compass_retrieve")
        .unwrap();
    assert_eq!(
        tool["inputSchema"]["properties"]["acquisition_policy"]["enum"][0],
        "spectral_window_v1"
    );
    let args = json!({"source":"approved","query":"sharedSymbol0","acquisition_policy":"spectral_window_v1","scope":"local","budget_bytes":16384});
    let reply = s.tool("compass_retrieve", args.clone());
    assert_eq!(reply["isError"], false, "{reply}");
    let pack = &reply["structuredContent"];
    assert_eq!(pack["schema_version"], "3");
    assert_eq!(pack["acquisition"]["state"], "terminal");
    assert_eq!(pack["scope"]["kind"], "local");
    assert!(pack["next_cursor"].is_null());
    assert!(pack.get("completion").is_none());
    let cli_pack = cli(&[
        "retrieve",
        "--store",
        f._store.path().to_str().unwrap(),
        "--query",
        "sharedSymbol0",
        "--acquisition-policy",
        "spectral_window_v1",
        "--scope",
        "local",
        "--budget-bytes",
        "16384",
    ]);
    for observed in [pack, &cli_pack] {
        assert_eq!(
            observed["payload_bytes"],
            compass_core::codec::canonical(observed).unwrap().len()
        );
    }
    let normalize = |mut value: Value| {
        value["payload_bytes"] = Value::Null;
        value["diagnostics"]["work"]["elapsed_ms"] = Value::Null;
        value["diagnostics"]["work"]["source_revalidation_elapsed_ms"] = Value::Null;
        value["diagnostics"]["work"]["numeric"]["elapsed_ms"] = Value::Null;
        value["diagnostics"]["limits"]["numeric"]["max_elapsed_ms"] = Value::Null;
        value
    };
    assert_eq!(
        compass_core::codec::canonical(&normalize(pack.clone())).unwrap(),
        compass_core::codec::canonical(&normalize(cli_pack)).unwrap()
    );
    for (field, value) in [
        ("theta", json!(1)),
        ("expansion", json!("heat")),
        ("cursor", json!("invalid-cursor")),
        ("contract", json!({})),
    ] {
        let mut bad = args.clone();
        bad[field] = value;
        let error = s.tool("compass_retrieve", bad);
        assert_eq!(error["isError"], true, "{error}");
        assert!(error.to_string().contains("rejects"), "{error}");
    }
    let mut bad = args.clone();
    bad.as_object_mut().unwrap().remove("scope");
    assert_eq!(s.tool("compass_retrieve", bad)["isError"], true);
    let legacy = s.tool(
        "compass_retrieve",
        json!({"source":"approved","query":"sharedSymbol0"}),
    );
    assert_eq!(legacy["structuredContent"]["schema_version"], "1");
}
