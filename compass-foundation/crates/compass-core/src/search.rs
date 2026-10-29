//! Literal retrieval from a declared, verified snapshot/package pair.
pub use crate::d0::{Error, Result};
#[derive(Debug)]
pub struct Output {
    pub value: Value,
    pub bytes: Vec<u8>,
    pub pretty_bytes: Vec<u8>,
    pub exit_code: i32,
}
use crate::d0::{input, resources::Context, source};
use crate::{
    codec, d0,
    model::{Snapshot, digest},
};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

#[derive(Clone, Debug)]
pub struct Limits {
    pub(crate) inner: d0::Limits,
}
impl Default for Limits {
    fn default() -> Self {
        let inner = d0::Limits {
            values: [
                ("max_manifest_bytes", 16777216),
                ("max_source_descriptor_bytes", 65536),
                ("max_records", 10000),
                ("max_json_depth", 64),
                ("max_file_bytes", 4194304),
                ("max_total_object_bytes", 67108864),
                ("max_output_bytes", 1048576),
                ("max_logical_memory_bytes", 134217728),
                ("max_line_bytes", 4096),
            ]
            .into_iter()
            .map(|(k, v)| (k.into(), v))
            .collect(),
            duration: None,
        };
        Self { inner }
    }
}
impl Limits {
    pub fn get(&self, key: &str) -> u64 {
        self.inner.get(key)
    }
    pub fn reduced(profile: Option<&Value>, duration: Option<u64>) -> Result<Self> {
        let mut l = Self::default();
        if duration.is_some_and(|n| n == 0 || n > 3600000) {
            return Err(Error("invalid_arguments"));
        }
        if let Some(v) = profile {
            let o = v.as_object().ok_or(Error("invalid_arguments"))?;
            if o.len() != 9 || o.keys().any(|k| !l.inner.values.contains_key(k)) {
                return Err(Error("invalid_arguments"));
            }
            for (k, cap) in &mut l.inner.values {
                let n = o[k]
                    .as_str()
                    .and_then(|s| crate::model::decimal(s).ok())
                    .ok_or(Error("invalid_arguments"))?;
                if n == 0 || n > *cap {
                    return Err(Error("invalid_arguments"));
                }
                *cap = n;
            }
        }
        l.inner.duration = duration;
        Ok(l)
    }
    pub(crate) fn value(&self) -> Value {
        let mut v = json!({});
        for (k, n) in &self.inner.values {
            v[k] = json!(n.to_string());
        }
        v["max_duration_ms"] = self
            .inner
            .duration
            .map(|n| json!(n.to_string()))
            .unwrap_or(Value::Null);
        v
    }
}
pub fn limits(path: Option<&Path>, duration: Option<u64>) -> Result<Limits> {
    let cancel = AtomicBool::new(false);
    let mut ctx = Context::new(Limits::default().inner, &cancel);
    let v = path
        .map(|p| {
            source::raw_file(p, 65536, &mut ctx, "invalid_arguments")
                .and_then(|b| input::parse(&b, 64, "invalid_arguments"))
        })
        .transpose()
        .map_err(|_| Error("invalid_arguments"))?;
    Limits::reduced(v.as_ref(), duration)
}
#[derive(Clone, Debug)]
pub struct Request {
    pub snapshot: PathBuf,
    pub source: PathBuf,
    pub query: String,
    pub max_matches: u64,
    pub limits: Limits,
}
fn valid_request(r: &Request) -> Result<()> {
    if r.query.is_empty()
        || r.query.len() > 256
        || r.query.bytes().any(|b| matches!(b, 0 | 10 | 13))
        || !(1..=1000).contains(&r.max_matches)
    {
        return Err(Error("invalid_arguments"));
    }
    Ok(())
}
pub fn search(r: &Request, cancel: &AtomicBool) -> Result<Output> {
    valid_request(r)?;
    let mut ctx = Context::new(r.limits.inner.clone(), cancel);
    operation(r, &mut ctx)
}
pub(crate) fn manifest(path: &Path, ctx: &mut Context<'_>) -> Result<input::Manifest> {
    ctx.check("before_manifest")?;
    let raw = source::raw_file(
        path,
        ctx.limits.get("max_manifest_bytes"),
        ctx,
        "input_invalid",
    )?;
    ctx.check("before_parse")?;
    let value = input::parse(&raw, ctx.limits.get("max_json_depth"), "input_invalid")?;
    ctx.check("after_parse")?;
    let mut n = 0u64;
    for k in ["entries", "directory_coverage", "issues"] {
        n = n
            .checked_add(value[k].as_array().ok_or(Error("input_invalid"))?.len() as u64)
            .ok_or(Error("arithmetic_overflow"))?;
    }
    if n > ctx.limits.get("max_records") {
        return Err(Error("input_limit"));
    }
    let s = 8u128 * (raw.len() as u128)
        + 4096u128 * n as u128
        + 65536
        + 8u128 * ctx.limits.get("max_output_bytes") as u128;
    ctx.add_structural(s)?;
    ctx.check("before_domain")?;
    let snapshot: Snapshot = serde_json::from_value(value).map_err(|_| Error("input_invalid"))?;
    snapshot.validate().map_err(|_| Error("input_invalid"))?;
    ctx.check("after_domain")?;
    Ok(input::Manifest { snapshot, raw })
}
// UTF-8 is validated incrementally; a partial scalar is carried into the next
// window, so no new RD traversal runs more than 65536 bytes without a check.
pub(crate) fn text_reason(b: &[u8], ctx: &mut Context<'_>) -> Result<Option<&'static str>> {
    let mut start = 0;
    let mut nul = false;
    while start < b.len() {
        ctx.check("utf8_chunk")?;
        let end = (start + 65536).min(b.len());
        match std::str::from_utf8(&b[start..end]) {
            Ok(_) => {
                nul |= b[start..end].contains(&0);
                start = end;
            }
            Err(e) if e.error_len().is_none() && end < b.len() => {
                let next = start + e.valid_up_to();
                nul |= b[start..next].contains(&0);
                start = next;
            }
            Err(_) => return Ok(Some("invalid_utf8")),
        }
    }
    ctx.check("utf8_finished")?;
    Ok(nul.then_some("nul"))
}
pub(crate) fn line_preflight(b: &[u8], ctx: &mut Context<'_>) -> Result<bool> {
    let mut len = 0u64;
    for (i, &c) in b.iter().enumerate() {
        if i % 65536 == 0 {
            ctx.check("lines_chunk")?;
        }
        len += 1;
        if len > ctx.limits.get("max_line_bytes") {
            return Ok(false);
        }
        if c == b'\n' {
            len = 0;
        }
    }
    ctx.check("lines_finished")?;
    Ok(true)
}
fn matching(b: &[u8], q: &[u8], ctx: &mut Context<'_>) -> Result<bool> {
    let mut found = false;
    for (i, w) in b.windows(q.len()).enumerate() {
        if i % 65536 == 0 {
            ctx.check("search_chunk")?;
        }
        found |= w == q;
    }
    Ok(found)
}
fn operation(r: &Request, ctx: &mut Context<'_>) -> Result<Output> {
    let result = (|| {
        ctx.check("operation_started")?;
        let m = manifest(&r.snapshot, ctx)?;
        ctx.check("before_source")?;
        let source = input::source(Some(&r.source), &m, ctx)?.ok_or(Error("source_invalid"))?;
        ctx.check("after_source")?;
        let s = &m.snapshot;
        let mut files = vec![];
        let mut hits = vec![];
        let (mut searched, mut non_text, mut unavailable, mut excluded, mut known) =
            (0u64, 0u64, 0u64, 0u64, 0u64);
        // Foundation validation guarantees strict native byte ordering.
        for entry in &s.entries {
            ctx.check("before_file")?;
            if entry.state == "excluded" {
                excluded += 1;
                continue;
            }
            if entry.state != "observed" {
                continue;
            }
            let rev = json!({"content_sha256":entry.content_sha256,"size_bytes":entry.size_bytes});
            let size = crate::model::decimal(entry.size_bytes.as_deref().unwrap())
                .map_err(|_| Error("input_invalid"))?;
            let mut check = json!({"state":"not_attempted","source_ref":source.descriptor["source_ref"],"actual_revision":null,"hash_check":null,"size_check":null,"reasons":[]});
            let mut state = "unavailable";
            let mut reasons = vec![];
            if size > ctx.limits.get("max_file_bytes") {
                reasons.push("file_bytes_limit".to_owned());
            } else if !ctx.memory(2 * (size as u128 + 1) + 65536) {
                reasons.push("memory_budget".to_owned());
            } else {
                let bytes = source.object(&rev, ctx, &mut check)?;
                if check["state"] != "verified" {
                    reasons = check["reasons"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_str().unwrap().to_owned())
                        .collect();
                } else if let Some(reason) = text_reason(&bytes, ctx)? {
                    state = "non_text";
                    reasons.push(reason.into());
                } else if !line_preflight(&bytes, ctx)? {
                    reasons.push("line_bytes_limit".into());
                } else {
                    state = "searched";
                    let (mut start, mut line) = (0usize, 0u64);
                    // Preflight bounds each line to <=4096 bytes. No line index retained.
                    for part in bytes.split_inclusive(|&b| b == b'\n') {
                        if part.is_empty() {
                            continue;
                        }
                        ctx.check("before_line")?;
                        line += 1;
                        let end = start + part.len();
                        if matching(part, r.query.as_bytes(), ctx)? {
                            known = known.checked_add(1).ok_or(Error("arithmetic_overflow"))?;
                            if (hits.len() as u64) < r.max_matches {
                                let text = std::str::from_utf8(part)
                                    .map_err(|_| Error("operational_error"))?;
                                hits.push(json!({"path_key":entry.path_key,"content_sha256":entry.content_sha256,"size_bytes":entry.size_bytes,"line":line.to_string(),"start_byte":start.to_string(),"end_byte":end.to_string(),"text":text}));
                            }
                        }
                        start = end;
                    }
                }
            }
            match state {
                "searched" => searched += 1,
                "non_text" => non_text += 1,
                _ => unavailable += 1,
            }
            check.as_object_mut().unwrap().remove("reasons");
            files.push(json!({"path_key":entry.path_key,"expected_revision":rev,"state":state,"reasons":reasons,"check":check}));
            ctx.check("after_file")?;
        }
        let mut reasons = vec![];
        if s.quality == "partial" {
            reasons.push("snapshot_partial");
        }
        if unavailable > 0 {
            reasons.push("unresolved_files");
        }
        let status = if reasons.is_empty() {
            "complete"
        } else if searched + non_text > 0 {
            "partial"
        } else {
            "unavailable"
        };
        let value = json!({"schema_version":"1","retrieval_contract_version":"0.1.0","report_type":"literal_search","producer_version":env!("CARGO_PKG_VERSION"),"producer_build":crate::BUILD_ID,"scope":"declared_snapshot_only",
        "snapshot":{"snapshot_id":s.snapshot_id,"workspace_id":s.workspace_id,"path_namespace":s.path_namespace,"policy_id":s.policy_id,"quality":s.quality,"state_digest":s.state_digest,"snapshot_file_sha256":digest(&m.raw)},"source":source.descriptor,"query":{"text":r.query,"mode":"literal_utf8_lines","max_matches":r.max_matches.to_string()},"limits":r.limits.value(),
        "scan":{"status":status,"reasons":reasons,"observed_files":files.len().to_string(),"verified_text_files":searched.to_string(),"non_text_files":non_text.to_string(),"unavailable_files":unavailable.to_string(),"excluded_entries":excluded.to_string(),"matching_lines_known":known.to_string(),"matches_returned":hits.len().to_string(),"matches_truncated":known>hits.len() as u64},"files":files,"matches":hits,
        "resources":{"object_bytes_read":ctx.read.to_string(),"object_bytes_reserved":ctx.reserved.to_string(),"logical_memory_peak":ctx.peak.to_string()}});
        ctx.check("before_canonical")?;
        let bytes = codec::canonical(&value).map_err(|_| Error("operational_error"))?;
        ctx.check("after_canonical")?;
        if bytes.len() as u64 > ctx.limits.get("max_output_bytes") {
            return Err(Error("output_limit"));
        }
        ctx.check("before_pretty")?;
        let mut pretty_bytes =
            serde_json::to_vec_pretty(&value).map_err(|_| Error("operational_error"))?;
        pretty_bytes.push(b'\n');
        ctx.check("after_pretty")?;
        ctx.check("before_publication")?;
        Ok(Output {
            pretty_bytes,
            value,
            bytes,
            exit_code: match status {
                "complete" => 0,
                "partial" => 3,
                _ => 1,
            },
        })
    })();
    ctx.check("operation_finished")?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Entry, path_key};
    use std::{
        fs,
        sync::{Arc, Barrier, atomic::Ordering},
        thread,
    };
    fn setup(raw: &[u8]) -> (tempfile::TempDir, Request) {
        let t = tempfile::tempdir().unwrap();
        let p = t.path();
        fs::create_dir(p.join("objects")).unwrap();
        let mut s: Snapshot = codec::decode(include_bytes!(
            "../../../research/proposals/d0-c2/fixtures/manifests/a.json"
        ))
        .unwrap();
        s.entries = vec![Entry {
            path_key: path_key(b"a.txt"),
            kind: "file".into(),
            state: "observed".into(),
            reason: None,
            content_sha256: Some(digest(raw)),
            size_bytes: Some(raw.len().to_string()),
            executable: Some(false),
        }];
        s.stats.bytes_read = raw.len().to_string();
        s.state_digest = Some(s.compute_state_digest().unwrap());
        let b = codec::canonical(&s).unwrap();
        fs::write(p.join("snapshot.json"), &b).unwrap();
        fs::write(p.join("objects").join(digest(raw)), raw).unwrap();
        fs::write(p.join("source.json"),codec::canonical(&json!({"source_schema_version":"1","source_contract_version":"0.1.0","workspace_id":s.workspace_id,"snapshot_id":s.snapshot_id,"policy_id":s.policy_id,"snapshot_file_sha256":digest(&b)})).unwrap()).unwrap();
        let r = Request {
            snapshot: p.join("snapshot.json"),
            source: p.into(),
            query: "alvo".into(),
            max_matches: 20,
            limits: Limits::default(),
        };
        (t, r)
    }
    fn out(r: &Request) -> Output {
        search(r, &AtomicBool::new(false)).unwrap()
    }
    #[test]
    fn rd01_08_frozen_literals_nontext_and_boundaries() {
        let f: Value =
            serde_json::from_str(include_str!("../../../verification/rd/fixtures.json")).unwrap();
        use base64::Engine;
        for x in f["literal"].as_array().unwrap() {
            let raw = base64::engine::general_purpose::STANDARD
                .decode(x["raw"].as_str().unwrap())
                .unwrap();
            let (_t, mut r) = setup(&raw);
            r.query = x["query"].as_str().unwrap().into();
            let o = out(&r);
            let got:Vec<_>=o.value["matches"].as_array().unwrap().iter().map(|h|json!({"line":h["line"],"start_byte":h["start_byte"],"end_byte":h["end_byte"],"text":h["text"]})).collect();
            assert_eq!(json!(got), x["hits"]);
            assert_eq!(
                o.value["resources"]["object_bytes_reserved"],
                (raw.len() + 1).to_string()
            );
        }
        for (b, reason) in [(b"\xff\0".as_slice(), "invalid_utf8"), (b"\0", "nul")] {
            let (_t, r) = setup(b);
            let v = out(&r).value;
            assert_eq!(v["files"][0]["state"], "non_text");
            assert_eq!(v["files"][0]["reasons"], json!([reason]));
            assert_eq!(v["files"][0]["check"]["state"], "verified");
        }
        for n in [4096, 4097] {
            let (_t, mut r) = setup(&vec![b'a'; n]);
            r.query = "a".into();
            let v = out(&r).value;
            assert_eq!(
                v["files"][0]["state"],
                if n == 4096 { "searched" } else { "unavailable" }
            );
        }
        let (_t, mut r) = setup(b"a");
        for n in [256, 257] {
            r.query = "é".repeat(n / 2) + if n % 2 == 1 { "a" } else { "" };
            assert_eq!(search(&r, &AtomicBool::new(false)).is_ok(), n == 256);
        }
    }
    #[test]
    fn rd12_synchronized_object_component_ancestor_and_dotdot() {
        for mode in ["object", "objects", "ancestor", "dotdot", "stable"] {
            let (t, mut r) = setup(b"alvo\n");
            let parent = t.path().join("parent");
            fs::create_dir(&parent).unwrap();
            let package = parent.join("package");
            fs::create_dir(&package).unwrap();
            for name in ["objects", "source.json"] {
                fs::rename(t.path().join(name), package.join(name)).unwrap();
            }
            let alias = parent.join("alias");
            fs::create_dir(&alias).unwrap();
            r.source = if mode == "dotdot" {
                alias.join("..").join("package")
            } else {
                package.clone()
            };
            let gate = Arc::new(Barrier::new(2));
            let other = gate.clone();
            let base = t.path().to_path_buf();
            let worker = thread::spawn(move || {
                other.wait();
                match mode {
                    "object" => {
                        let p = package.join("objects").join(digest(b"alvo\n"));
                        fs::rename(&p, p.with_extension("old")).unwrap();
                        fs::write(p, b"alvo\n").unwrap();
                    }
                    "objects" => {
                        fs::rename(package.join("objects"), package.join("old")).unwrap();
                        fs::create_dir(package.join("objects")).unwrap();
                        fs::hard_link(
                            package.join("old").join(digest(b"alvo\n")),
                            package.join("objects").join(digest(b"alvo\n")),
                        )
                        .unwrap();
                    }
                    "ancestor" => {
                        fs::rename(&parent, base.join("old-parent")).unwrap();
                        fs::create_dir(&parent).unwrap();
                        fs::rename(base.join("old-parent/package"), parent.join("package"))
                            .unwrap();
                    }
                    "dotdot" => {
                        fs::rename(&alias, parent.join("old-alias")).unwrap();
                        fs::create_dir(alias).unwrap();
                    }
                    _ => {}
                }
                other.wait();
            });
            let flag = AtomicBool::new(false);
            let mut ctx = Context::new(r.limits.inner.clone(), &flag);
            let mut fired = false;
            ctx.hook = Some(Box::new(move |e| {
                if e == "object_before_stability" && !fired {
                    fired = true;
                    gate.wait();
                    gate.wait();
                }
                false
            }));
            let o = operation(&r, &mut ctx).unwrap();
            worker.join().unwrap();
            let f = &o.value["files"][0];
            assert_eq!(f["check"]["hash_check"], true);
            assert_eq!(f["check"]["size_check"], true);
            if mode == "stable" {
                assert_eq!(o.exit_code, 0);
                assert_eq!(o.value["matches"].as_array().unwrap().len(), 1);
            } else {
                assert_eq!(o.exit_code, 1);
                assert_eq!(f["reasons"], json!(["source_unstable"]));
                assert_eq!(o.value["matches"], json!([]));
            }
        }
    }
    #[test]
    fn rd17_exact_structural_and_working_credit_precedes_open() {
        let (_t, mut r) = setup(b"alvo\n");
        let b = fs::read(&r.snapshot).unwrap().len() as u128;
        let d = fs::read(r.source.join("source.json")).unwrap().len() as u128;
        let sm = 8 * b + 4096 * 2 + 65536 + 8 * 1048576;
        let s = sm + 8 * d;
        let w = 65548;
        for (cap, state, peak) in [(s + w, "searched", s + w), (s + w - 1, "unavailable", s)] {
            r.limits
                .inner
                .values
                .insert("max_logical_memory_bytes".into(), cap as u64);
            let o = out(&r);
            assert_eq!(o.value["files"][0]["state"], state);
            assert_eq!(
                o.value["resources"]["logical_memory_peak"],
                peak.to_string()
            );
            if state == "unavailable" {
                assert_eq!(o.value["resources"]["object_bytes_reserved"], "0");
            }
        }
        for cap in [sm - 1, sm, s - 1] {
            r.limits
                .inner
                .values
                .insert("max_logical_memory_bytes".into(), cap as u64);
            assert!(matches!(
                search(&r, &AtomicBool::new(false)),
                Err(Error("input_limit"))
            ));
        }
        // Transport can be counted but domain must not be constructed on refusal.
        fs::write(
            &r.snapshot,
            b"{\"entries\":[],\"directory_coverage\":[],\"issues\":[]}",
        )
        .unwrap();
        r.limits
            .inner
            .values
            .insert("max_logical_memory_bytes".into(), 1);
        let flag = AtomicBool::new(false);
        let mut ctx = Context::new(r.limits.inner.clone(), &flag);
        ctx.hook = Some(Box::new(|e| {
            assert_ne!(e, "before_domain");
            false
        }));
        assert!(matches!(operation(&r, &mut ctx), Err(Error("input_limit"))));
    }
    #[test]
    fn rd19_synchronized_cancel_deadline_and_priority() {
        // A long valid UTF8 file without LF/hits exercises validation and line loops.
        for event in [
            "read_chunk",
            "utf8_chunk",
            "lines_chunk",
            "search_chunk",
            "before_pretty",
            "after_pretty",
            "before_publication",
            "operation_finished",
        ] {
            for deadline in [false, true] {
                let (_t, mut r) = setup(
                    if [
                        "search_chunk",
                        "before_pretty",
                        "after_pretty",
                        "before_publication",
                    ]
                    .contains(&event)
                    {
                        b"alvo\n"
                    } else {
                        &[]
                    },
                );
                if ["utf8_chunk", "lines_chunk"].contains(&event) {
                    let (t, rr) = setup(&vec![b'x'; 131073]);
                    r = rr; // keep temporary alive during operation
                    run_interruption(&r, event, deadline);
                    drop(t);
                } else {
                    run_interruption(&r, event, deadline);
                }
            }
        }
        fn run_interruption(r: &Request, event: &str, deadline: bool) {
            let flag = Arc::new(AtomicBool::new(false));
            let f = flag.clone();
            let gate = Arc::new(Barrier::new(2));
            let other = gate.clone();
            let worker = thread::spawn(move || {
                other.wait();
                if !deadline {
                    f.store(true, Ordering::Relaxed);
                }
                other.wait();
            });
            let mut ctx = Context::new(r.limits.inner.clone(), &flag);
            let mut fired = false;
            let mut visits = 0;
            ctx.hook = Some(Box::new(|e| {
                if e == event {
                    visits += 1;
                }
                if e == event && !fired && (event != "utf8_chunk" || visits == 2) {
                    fired = true;
                    gate.wait();
                    gate.wait();
                    return deadline;
                }
                false
            }));
            assert!(
                matches!(operation(r,&mut ctx),Err(e) if e==Error(if deadline{"deadline_exceeded"}else{"cancelled"}))
            );
            drop(ctx);
            worker.join().unwrap();
            assert!(fired);
        }
        let (_t, mut r) = setup(b"a");
        r.snapshot = PathBuf::from("/nonexistent-rd-input");
        for cancel in [false, true] {
            let flag = AtomicBool::new(false);
            let mut ctx = Context::new(r.limits.inner.clone(), &flag);
            ctx.hook = Some(Box::new(|e| {
                if e == "operation_finished" {
                    flag.store(cancel, Ordering::Relaxed);
                    return true;
                }
                false
            }));
            assert!(
                matches!(operation(&r,&mut ctx),Err(e) if e==Error(if cancel{"cancelled"}else{"deadline_exceeded"}))
            );
        }
    }
}
