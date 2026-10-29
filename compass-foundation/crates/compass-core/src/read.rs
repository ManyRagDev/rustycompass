//! Read exact lines from one fully verified object of a declared snapshot.
pub use crate::search::{Error, Limits, Output, Result, limits};
use crate::{
    codec,
    d0::{input, resources::Context},
    model::{decimal, decode_path_key, digest},
    search::{line_preflight, manifest, text_reason},
};
use serde_json::{Value, json};
use std::{path::PathBuf, sync::atomic::AtomicBool};

#[derive(Clone, Debug)]
pub struct Request {
    pub snapshot: PathBuf,
    pub source: PathBuf,
    pub path_key: String,
    pub start_line: u64,
    pub max_lines: u64,
    pub limits: Limits,
}

pub fn read(r: &Request, cancel: &AtomicBool) -> Result<Output> {
    if r.path_key.is_empty()
        || decode_path_key(&r.path_key).is_err()
        || r.start_line == 0
        || !(1..=1000).contains(&r.max_lines)
    {
        return Err(Error("invalid_arguments"));
    }
    operation(r, &mut Context::new(r.limits.inner.clone(), cancel))
}

fn select(bytes: &[u8], r: &Request, ctx: &mut Context<'_>) -> Result<Value> {
    let (mut total, mut returned, mut offset) = (0u64, 0u64, 0usize);
    let (mut begin, mut end) = (bytes.len(), bytes.len());
    let mut text = String::new();
    // Preflight has bounded every line. No unbounded line index or start+max sum.
    for line in bytes.split_inclusive(|&b| b == b'\n') {
        ctx.check("read_selection_line")?;
        total = total.checked_add(1).ok_or(Error("arithmetic_overflow"))?;
        if total >= r.start_line && returned < r.max_lines {
            if returned == 0 {
                begin = offset;
            }
            end = offset + line.len();
            text.push_str(std::str::from_utf8(line).map_err(|_| Error("operational_error"))?);
            returned += 1;
        }
        offset += line.len();
    }
    ctx.check("read_selection_finished")?;
    let next = if returned != 0 && end < bytes.len() {
        Some(
            r.start_line
                .checked_add(returned)
                .ok_or(Error("arithmetic_overflow"))?
                .to_string(),
        )
    } else {
        None
    };
    Ok(
        json!({"total_lines":total.to_string(),"returned_lines":returned.to_string(),
        "start_byte":begin.to_string(),"end_byte":end.to_string(),"text":text,
        "truncated":next.is_some(),"next_start_line":next}),
    )
}

fn operation(r: &Request, ctx: &mut Context<'_>) -> Result<Output> {
    let result = (|| {
        ctx.check("operation_started")?;
        let m = manifest(&r.snapshot, ctx)?;
        ctx.check("before_source")?;
        let source = input::source(Some(&r.source), &m, ctx)?.ok_or(Error("source_invalid"))?;
        ctx.check("after_source")?;
        let s = &m.snapshot;
        let mut selected = None;
        for entry in &s.entries {
            ctx.check("read_lookup_entry")?;
            if entry.path_key == r.path_key && entry.state == "observed" {
                selected = Some(entry);
                break;
            }
        }
        let entry = selected.ok_or(Error("path_not_observed"))?;
        let rev = json!({"content_sha256":entry.content_sha256,"size_bytes":entry.size_bytes});
        let size =
            decimal(entry.size_bytes.as_deref().unwrap()).map_err(|_| Error("input_invalid"))?;
        let mut check = json!({"state":"not_attempted","source_ref":source.descriptor["source_ref"],
            "actual_revision":null,"hash_check":null,"size_check":null,"reasons":[]});
        let mut status = "unavailable";
        let mut reasons: Vec<String> = vec![];
        let mut excerpt = json!({"total_lines":null,"returned_lines":"0","start_byte":null,
            "end_byte":null,"text":null,"next_start_line":null,"truncated":false});
        if size > ctx.limits.get("max_file_bytes") {
            reasons.push("file_bytes_limit".into());
        } else if !ctx.memory(2 * (size as u128 + 1) + 65536) {
            reasons.push("memory_budget".into());
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
                status = "non_text";
                reasons.push(reason.into());
            } else if !line_preflight(&bytes, ctx)? {
                reasons.push("line_bytes_limit".into());
            } else {
                excerpt = select(&bytes, r, ctx)?;
                status = "available";
            }
        }
        check.as_object_mut().unwrap().remove("reasons");
        reasons.sort();
        reasons.dedup();
        let value = json!({"schema_version":"1","read_contract_version":"0.1.0","report_type":"verified_read",
            "producer_version":env!("CARGO_PKG_VERSION"),"producer_build":crate::BUILD_ID,"scope":"declared_snapshot_only",
            "snapshot":{"snapshot_id":s.snapshot_id,"workspace_id":s.workspace_id,"path_namespace":s.path_namespace,
                "policy_id":s.policy_id,"quality":s.quality,"state_digest":s.state_digest,"snapshot_file_sha256":digest(&m.raw)},
            "source":source.descriptor,"request":{"path_key":r.path_key,"start_line":r.start_line.to_string(),"max_lines":r.max_lines.to_string()},
            "limits":r.limits.value(),"status":status,"reasons":reasons,
            "file":{"path_key":entry.path_key,"expected_revision":rev,"check":check},"excerpt":excerpt,
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
            value,
            bytes,
            pretty_bytes,
            exit_code: if status == "available" { 0 } else { 1 },
        })
    })();
    ctx.check("operation_finished")?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Entry, Snapshot, path_key};
    use std::{fs, sync::atomic::Ordering};

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
        fs::write(p.join("source.json"), codec::canonical(&json!({"source_schema_version":"1","source_contract_version":"0.1.0","workspace_id":s.workspace_id,"snapshot_id":s.snapshot_id,"policy_id":s.policy_id,"snapshot_file_sha256":digest(&b)})).unwrap()).unwrap();
        let r = Request {
            snapshot: p.join("snapshot.json"),
            source: p.into(),
            path_key: path_key(b"a.txt"),
            start_line: 1,
            max_lines: 20,
            limits: Limits::default(),
        };
        (t, r)
    }
    fn out(r: &Request) -> Output {
        read(r, &AtomicBool::new(false)).unwrap()
    }
    fn unavailable(v: &Value) {
        assert_ne!(v["status"], "available");
        assert_eq!(
            v["excerpt"],
            json!({"total_lines":null,"returned_lines":"0","start_byte":null,"end_byte":null,"text":null,"next_start_line":null,"truncated":false})
        );
    }
    #[test]
    fn independent_line_fixtures_and_exact_transport() {
        let fixtures: Value =
            serde_json::from_str(include_str!("../../../verification/reading/fixtures.json"))
                .unwrap();
        for case in fixtures["cases"].as_array().unwrap() {
            let raw = case["text"].as_str().unwrap().as_bytes();
            let (_t, mut r) = setup(raw);
            r.start_line = case["start"].as_u64().unwrap();
            r.max_lines = case["max"].as_u64().unwrap();
            let o = out(&r);
            assert_eq!(o.exit_code, 0, "{}", case["name"]);
            assert_eq!(o.value["excerpt"], case["expected"], "{}", case["name"]);
            assert_eq!(o.bytes, codec::canonical(&o.value).unwrap());
            assert!(o.pretty_bytes.ends_with(b"\n"));
            assert_eq!(
                o.value["resources"]["object_bytes_read"],
                raw.len().to_string()
            );
            assert_eq!(
                o.value["resources"]["object_bytes_reserved"],
                (raw.len() + 1).to_string()
            );
        }
    }
    #[test]
    fn validates_arguments_before_opening_and_binding_before_lookup() {
        let (_t, mut r) = setup(b"a\n");
        let original = r.clone();
        r.snapshot = PathBuf::from("/no-read-manifest");
        for key in ["", "Lw", "Li4", "YQ==", "--json"] {
            r.path_key = key.into();
            assert!(matches!(
                read(&r, &AtomicBool::new(false)),
                Err(Error("invalid_arguments"))
            ));
        }
        for (start, max) in [(0, 20), (1, 0), (1, 1001)] {
            r.path_key = original.path_key.clone();
            r.start_line = start;
            r.max_lines = max;
            assert!(matches!(
                read(&r, &AtomicBool::new(false)),
                Err(Error("invalid_arguments"))
            ));
        }
        r = original;
        r.path_key = path_key(b"missing");
        assert!(matches!(
            read(&r, &AtomicBool::new(false)),
            Err(Error("path_not_observed"))
        ));
        let p = r.source.join("source.json");
        let mut d: Value = serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
        d["snapshot_file_sha256"] = json!("0".repeat(64));
        fs::write(p, codec::canonical(&d).unwrap()).unwrap();
        assert!(matches!(
            read(&r, &AtomicBool::new(false)),
            Err(Error("incompatible_inputs"))
        ));
    }
    #[test]
    fn no_excerpt_from_bad_bytes_missing_objects_or_late_long_line() {
        for raw in [
            b"\xff\0".to_vec(),
            b"ok\n\0".to_vec(),
            [b"ok\n".as_slice(), &vec![b'x'; 4097]].concat(),
        ] {
            let (_t, mut r) = setup(&raw);
            r.max_lines = 1;
            let v = out(&r).value;
            unavailable(&v);
            assert_eq!(v["file"]["check"]["state"], "verified");
            assert_eq!(
                v["reasons"],
                json!([if raw[0] == 255 {
                    "invalid_utf8"
                } else if raw.contains(&0) {
                    "nul"
                } else {
                    "line_bytes_limit"
                }])
            );
        }
        for replacement in [None, Some(b"b\n".as_slice()), Some(b"b"), Some(b"b\nc")] {
            let (_t, mut r) = setup(b"a\n");
            r.start_line = u64::MAX;
            let obj = r.source.join("objects").join(digest(b"a\n"));
            if let Some(raw) = replacement {
                fs::write(obj, raw).unwrap()
            } else {
                fs::remove_file(obj).unwrap()
            }
            let v = out(&r).value;
            unavailable(&v);
            assert_eq!(v["file"]["check"]["state"], "failed");
        }
        let (_t, r) = setup(&vec![b'a'; 4096]);
        assert_eq!(out(&r).exit_code, 0);
    }
    #[test]
    fn unicode_across_validation_windows_preserves_raw_offsets() {
        let mut raw = b"a\n".repeat(32767);
        raw.extend_from_slice("aé\nfinal".as_bytes());
        let (_t, mut r) = setup(&raw);
        r.start_line = 32768;
        r.max_lines = 1;
        let v = out(&r).value;
        assert_eq!(
            v["excerpt"],
            json!({"total_lines":"32769","returned_lines":"1","start_byte":"65534","end_byte":"65538","text":"aé\n","next_start_line":"32769","truncated":true})
        );
    }
    #[test]
    fn synchronized_replacement_cannot_publish_verified_prefix() {
        for stable in [false, true] {
            let (_t, r) = setup(b"a\nb\n");
            let obj = r.source.join("objects").join(digest(b"a\nb\n"));
            let flag = AtomicBool::new(false);
            let mut ctx = Context::new(r.limits.inner.clone(), &flag);
            let mut fired = false;
            ctx.hook = Some(Box::new(|event| {
                if event == "object_before_stability" && !fired {
                    fired = true;
                    if !stable {
                        fs::rename(&obj, obj.with_extension("old")).unwrap();
                        fs::write(&obj, b"a\nb\n").unwrap();
                    }
                }
                false
            }));
            let o = operation(&r, &mut ctx).unwrap();
            drop(ctx);
            assert!(fired);
            assert_eq!(o.value["file"]["check"]["hash_check"], true);
            if stable {
                assert_eq!(o.value["excerpt"]["text"], "a\nb\n");
            } else {
                unavailable(&o.value);
                assert_eq!(o.value["reasons"], json!(["source_unstable"]));
            }
        }
    }
    #[test]
    fn logical_memory_reservation_and_output_boundaries() {
        let (_t, mut r) = setup(b"a\n");
        let sm = 8 * fs::read(&r.snapshot).unwrap().len() as u64 + 4096 * 2 + 65536 + 8 * 1048576;
        let s = sm + 8 * fs::read(r.source.join("source.json")).unwrap().len() as u64;
        let w = 2 * 3 + 65536;
        for (cap, available, peak) in [(s + w, true, s + w), (s + w - 1, false, s)] {
            r.limits
                .inner
                .values
                .insert("max_logical_memory_bytes".into(), cap);
            let v = out(&r).value;
            assert_eq!(v["status"] == "available", available);
            assert_eq!(v["resources"]["logical_memory_peak"], peak.to_string());
            if !available {
                unavailable(&v);
                assert_eq!(v["resources"]["object_bytes_reserved"], "0");
            }
        }
        for cap in [sm - 1, sm, s - 1] {
            r.limits
                .inner
                .values
                .insert("max_logical_memory_bytes".into(), cap);
            assert!(matches!(
                read(&r, &AtomicBool::new(false)),
                Err(Error("input_limit"))
            ));
        }
        r.limits = Limits::default();
        for cap in [2, 3] {
            r.limits
                .inner
                .values
                .insert("max_total_object_bytes".into(), cap);
            let v = out(&r).value;
            assert_eq!(v["status"] == "available", cap == 3);
            if cap == 2 {
                assert_eq!(v["file"]["check"]["state"], "not_attempted");
                assert_eq!(v["reasons"], json!(["total_bytes_limit"]));
            }
        }
        r.limits = Limits::default();
        r.limits.inner.values.insert("max_output_bytes".into(), 1);
        assert!(matches!(
            read(&r, &AtomicBool::new(false)),
            Err(Error("output_limit"))
        ));
        r.limits = Limits::default();
        r.limits.inner.values.insert("max_file_bytes".into(), 1);
        let v = out(&r).value;
        unavailable(&v);
        assert_eq!(v["resources"]["object_bytes_reserved"], "0");
    }
    #[test]
    fn cancellation_deadline_and_final_error_priority() {
        for event in [
            "read_chunk",
            "utf8_chunk",
            "lines_chunk",
            "read_lookup_entry",
            "read_selection_line",
            "read_selection_finished",
            "before_canonical",
            "after_canonical",
            "before_pretty",
            "after_pretty",
            "before_publication",
            "operation_finished",
        ] {
            for cancel in [true, false] {
                let (_t, r) = setup(&b"a\n".repeat(40000));
                let flag = AtomicBool::new(false);
                let mut ctx = Context::new(r.limits.inner.clone(), &flag);
                let mut fired = false;
                ctx.hook = Some(Box::new(|e| {
                    if e == event {
                        fired = true;
                        if cancel {
                            flag.store(true, Ordering::Relaxed)
                        }
                        return true;
                    }
                    false
                }));
                assert!(
                    matches!(operation(&r,&mut ctx),Err(e) if e==Error(if cancel {"cancelled"} else {"deadline_exceeded"})),
                    "{event}"
                );
                drop(ctx);
                assert!(fired, "{event}");
            }
        }
        let (_t, mut r) = setup(b"a");
        r.snapshot = "/absent-read-input".into();
        let flag = AtomicBool::new(false);
        let mut ctx = Context::new(r.limits.inner.clone(), &flag);
        ctx.hook = Some(Box::new(|event| {
            if event == "operation_finished" {
                flag.store(true, Ordering::Relaxed);
                return true;
            }
            false
        }));
        assert!(matches!(operation(&r, &mut ctx), Err(Error("cancelled"))));
    }
}
