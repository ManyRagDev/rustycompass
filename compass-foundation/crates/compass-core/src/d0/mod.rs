//! Exact D0 sensor. Foundation remains the authority for saved-state validation.
pub(crate) mod input;
pub(crate) mod resources;
pub(crate) mod source;
mod types;
pub use input::limits;
pub use types::{Error, Limits, Output, Request, Result};
mod metric;
mod report;
use std::sync::atomic::AtomicBool;

pub fn measure(request: &Request, cancel: &AtomicBool) -> Result<Output> {
    let mut ctx = resources::Context::new(request.limits.clone(), cancel);
    operation(request, &mut ctx)
}
fn operation(request: &Request, ctx: &mut resources::Context<'_>) -> Result<Output> {
    let result = (|| {
        ctx.check("operation_started")?;
        let manifests = input::manifests(&request.baseline, &request.current, ctx)?;
        let a = input::source(request.baseline_source.as_deref(), &manifests[0], ctx)?;
        let b = input::source(request.current_source.as_deref(), &manifests[1], ctx)?;
        ctx.check("before_comparison")?;
        let comparison = crate::compare::compare(&manifests[0].snapshot, &manifests[1].snapshot)
            .map_err(|_| Error("input_invalid"))?;
        let value = report::assemble(&manifests, &[a, b], comparison, ctx)?;
        let bytes = types::canon(&value)?;
        if bytes.len() as u64 > ctx.limits.get("max_output_bytes") {
            return Err(Error("output_limit"));
        }
        ctx.check("before_publication")?;
        let exit_code = match value["measurement"]["status"].as_str().unwrap() {
            "complete" => 0,
            "partial" => 3,
            _ => 1,
        };
        Ok(Output {
            value,
            bytes,
            exit_code,
        })
    })();
    ctx.check("operation_finished")?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        codec,
        model::{Snapshot, digest, path_key},
    };
    use serde_json::{Value, json};
    use std::{
        fs,
        path::Path,
        sync::{Arc, Barrier, atomic::Ordering},
        thread,
    };
    fn two_units() -> (tempfile::TempDir, Request) {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("snapshot.json");
        let mut snap: Snapshot = codec::decode(include_bytes!(
            "../../../../research/proposals/d0-c2/fixtures/manifests/a.json"
        ))
        .unwrap();
        let mut q = snap.entries[0].clone();
        q.path_key = path_key(b"q");
        snap.entries.push(q);
        snap.stats.observed_files = "2".into();
        snap.stats.bytes_read = "4".into();
        snap.stats.read_attempts = "2".into();
        snap.state_digest = Some(snap.compute_state_digest().unwrap());
        let raw = codec::canonical(&snap).unwrap();
        fs::write(&path, &raw).unwrap();
        fs::create_dir(temp.path().join("objects")).unwrap();
        fs::write(temp.path().join("objects").join(digest(b"a\n")), b"a\n").unwrap();
        let desc = json!({"source_schema_version":"1","source_contract_version":"0.1.0","workspace_id":snap.workspace_id,"snapshot_id":snap.snapshot_id,"policy_id":snap.policy_id,"snapshot_file_sha256":digest(&raw)});
        fs::write(
            temp.path().join("source.json"),
            codec::canonical(&desc).unwrap(),
        )
        .unwrap();
        let req = Request {
            baseline: path.clone(),
            current: path,
            baseline_source: Some(temp.path().into()),
            current_source: Some(temp.path().into()),
            limits: Limits::default(),
        };
        (temp, req)
    }
    // Real manifests/packages and the supplied (non-canonicalized) root references.
    fn bug004_operation(mode: &str) -> Output {
        let temp = tempfile::tempdir().unwrap();
        let manifest = temp.path().join("snapshot.json");
        let raw = include_bytes!("../../../../research/proposals/d0-c2/fixtures/manifests/a.json");
        fs::write(&manifest, raw).unwrap();
        let snap: Snapshot = codec::decode(raw).unwrap();
        let package = temp.path().join("package");
        let alias = temp.path().join("alias-component");
        fs::create_dir_all(package.join("objects")).unwrap();
        fs::create_dir(&alias).unwrap();
        fs::write(package.join("objects").join(digest(b"a\n")), b"a\n").unwrap();
        let descriptor = codec::canonical(&json!({"source_schema_version":"1",
            "source_contract_version":"0.1.0","workspace_id":snap.workspace_id,
            "snapshot_id":snap.snapshot_id,"policy_id":snap.policy_id,
            "snapshot_file_sha256":digest(raw)}))
        .unwrap();
        fs::write(package.join("source.json"), &descriptor).unwrap();
        let indirect = alias.join("..").join("package");
        let req = Request {
            baseline: manifest.clone(),
            current: manifest,
            baseline_source: Some(if mode == "baseline" {
                indirect.clone()
            } else {
                package.clone()
            }),
            current_source: Some(if mode == "same" || mode == "baseline" {
                package
            } else {
                indirect
            }),
            limits: Limits::default(),
        };
        let intervention = mode == "current" || mode == "baseline";
        let gate = Arc::new(Barrier::new(2));
        let other = gate.clone();
        let worker = intervention.then(|| {
            thread::spawn(move || {
                other.wait();
                fs::rename(&alias, alias.with_file_name("old-component")).unwrap();
                fs::create_dir(&alias).unwrap();
                other.wait();
            })
        });
        let flag = AtomicBool::new(false);
        let mut ctx = resources::Context::new(req.limits.clone(), &flag);
        let mut opened = 0;
        ctx.hook = Some(Box::new(|event| {
            if event == "object_opened" {
                opened += 1;
                if intervention && opened == 1 {
                    gate.wait();
                    gate.wait();
                }
            }
            false
        }));
        let result = operation(&req, &mut ctx);
        if let Some(worker) = worker {
            worker.join().unwrap();
        }
        let out = result.expect("BUG-004 requires a measurement, not a generic error");
        drop(ctx);
        assert_eq!(opened, 1, "shared unit must open/read only one object");
        let v = &out.value;
        eprintln!(
            "bug004 {mode}: exit={} measurement={} unit={} resources={}",
            out.exit_code, v["measurement"], v["units"][0], v["resources"]
        );
        assert_eq!(v["units"].as_array().unwrap().len(), 1);
        assert_eq!(v["resources"]["object_bytes_reserved"], "3");
        assert_eq!(v["resources"]["object_bytes_read"], "2");
        assert_eq!(v["resources"]["cells_reserved"], "0");
        let revision = json!({"content_sha256":digest(b"a\n"),"size_bytes":"2"});
        for side in ["baseline", "current"] {
            let check = &v["units"][0]["checks"][side];
            assert_eq!(check["observation"], "observed");
            assert_eq!(check["source_ref"], digest(&descriptor));
            assert_eq!(check["expected_revision"], revision);
            // The one physical read reached EOF even if a reference became unstable.
            assert_eq!(check["actual_revision"], revision);
            assert_eq!(check["hash_check"], true);
            assert_eq!(check["size_check"], true);
        }
        out
    }
    fn bug004_unavailable(out: &Output) {
        let v = &out.value;
        assert_eq!(out.exit_code, 1);
        assert_eq!(v["measurement"]["status"], "unavailable");
        for field in ["value", "line_mass", "zero_basis", "known_subset"] {
            assert_eq!(v["measurement"][field], Value::Null);
        }
        assert_eq!(
            v["measurement"]["reasons"],
            json!(["no_text_units", "unresolved_units"])
        );
        assert_eq!(v["units"][0]["status"], "unavailable");
        assert_eq!(v["units"][0]["reasons"], json!(["source_unstable"]));
        assert!(v["units"][0].get("calculation").is_none());
        assert_eq!(v["scope"]["resolved_text_units"], "0");
        assert_eq!(v["scope"]["unavailable_units"], "1");
    }
    #[test]
    fn bug004_stable_same_and_distinct_references_share_one_read() {
        for mode in ["same", "distinct"] {
            let out = bug004_operation(mode);
            let v = &out.value;
            assert_eq!(out.exit_code, 0);
            assert_eq!(v["measurement"]["status"], "complete");
            assert_eq!(v["measurement"]["value"]["numerator"], "0");
            assert_eq!(v["measurement"]["value"]["denominator"], "2");
            assert_eq!(v["measurement"]["line_mass"], "2");
            assert_eq!(v["measurement"]["reasons"], json!([]));
            assert_eq!(v["units"][0]["status"], "available");
            assert_eq!(v["units"][0]["calculation"]["c"], "0");
            assert_eq!(v["units"][0]["calculation"]["m"], "2");
            for side in ["baseline", "current"] {
                assert_eq!(v["units"][0]["checks"][side]["state"], "verified");
                assert_eq!(v["units"][0]["checks"][side]["reasons"], json!([]));
            }
        }
    }
    #[test]
    fn bug004_current_component_replaced_during_shared_read() {
        let out = bug004_operation("current");
        let checks = &out.value["units"][0]["checks"];
        assert_eq!(checks["baseline"]["state"], "verified");
        assert_eq!(checks["baseline"]["reasons"], json!([]));
        assert_eq!(
            checks["current"]["state"], "failed",
            "BUG-004: unstable current was declared verified"
        );
        assert_eq!(checks["current"]["reasons"], json!(["source_unstable"]));
        bug004_unavailable(&out);
    }
    #[test]
    fn bug004_baseline_component_replacement_remains_detected() {
        let out = bug004_operation("baseline");
        let check = &out.value["units"][0]["checks"]["baseline"];
        assert_eq!(check["state"], "failed");
        assert_eq!(check["reasons"], json!(["source_unstable"]));
        bug004_unavailable(&out);
    }
    #[test]
    fn same_root_shares_only_within_unit() {
        let (_temp, req) = two_units();
        let out = measure(&req, &AtomicBool::new(false)).unwrap();
        assert_eq!(out.value["resources"]["object_bytes_reserved"], "6");
        assert_eq!(out.value["resources"]["object_bytes_read"], "4");
        assert_eq!(out.value["measurement"]["value"]["denominator"], "4");
    }
    #[test]
    fn synchronized_sigint_after_first_unit_before_next_and_before_publication() {
        for event in ["after_unit", "before_publication", "read_chunk"] {
            let (_temp, req) = two_units();
            let flag = Arc::new(AtomicBool::new(false));
            let worker_flag = flag.clone();
            let barrier = Arc::new(Barrier::new(2));
            let other = barrier.clone();
            let worker = thread::spawn(move || {
                other.wait();
                worker_flag.store(true, Ordering::Relaxed);
                other.wait();
            });
            let mut ctx = resources::Context::new(req.limits.clone(), &flag);
            let mut units = 0;
            let mut fired = false;
            ctx.hook = Some(Box::new(move |name| {
                if name == "before_unit" {
                    units += 1;
                }
                if !fired && name == event {
                    if event == "after_unit" {
                        assert_eq!(units, 1);
                    }
                    fired = true;
                    barrier.wait();
                    barrier.wait();
                }
                false
            }));
            let out = operation(&req, &mut ctx);
            worker.join().unwrap();
            assert!(matches!(out, Err(Error("cancelled"))));
        }
    }
    #[test]
    fn deadline_before_publication_and_priority_over_error() {
        let (_temp, req) = two_units();
        let flag = AtomicBool::new(false);
        let mut ctx = resources::Context::new(req.limits.clone(), &flag);
        ctx.hook = Some(Box::new(|event| event == "before_publication"));
        assert!(matches!(
            operation(&req, &mut ctx),
            Err(Error("deadline_exceeded"))
        ));
        for cancelled in [false, true] {
            let mut req = req.clone();
            req.baseline = Path::new("/nonexistent-compass-i1-input").into();
            flag.store(false, Ordering::Relaxed);
            let mut ctx = resources::Context::new(req.limits.clone(), &flag);
            ctx.hook = Some(Box::new(|event| {
                if event == "operation_finished" {
                    flag.store(cancelled, Ordering::Relaxed);
                    return true;
                }
                false
            }));
            assert!(
                matches!(operation(&req,&mut ctx),Err(e) if e==Error(if cancelled{"cancelled"}else{"deadline_exceeded"}))
            );
        }
    }
    #[test]
    fn descriptor_binding_and_present_invalid_on_empty_universe() {
        let (temp, mut req) = two_units();
        let mut snap: Value = codec::decode(&fs::read(&req.baseline).unwrap()).unwrap();
        snap["entries"] = json!([]);
        snap["stats"]["observed_files"] = json!("0");
        snap["stats"]["bytes_read"] = json!("0");
        snap["stats"]["read_attempts"] = json!("0");
        let mut typed: Snapshot = serde_json::from_value(snap).unwrap();
        typed.state_digest = Some(typed.compute_state_digest().unwrap());
        fs::write(&req.baseline, codec::canonical(&typed).unwrap()).unwrap();
        assert!(matches!(
            measure(&req, &AtomicBool::new(false)),
            Err(Error("incompatible_inputs"))
        ));
        req.baseline_source = Some(temp.path().join("absent"));
        assert!(matches!(
            measure(&req, &AtomicBool::new(false)),
            Err(Error("source_invalid"))
        ));
        req.baseline_source = None;
        req.current_source = None;
        assert_eq!(
            measure(&req, &AtomicBool::new(false)).unwrap().value["measurement"]["zero_basis"],
            "empty_universe"
        );
    }
}
