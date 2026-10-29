use super::{
    input::Manifest,
    metric,
    resources::Context,
    source::Source,
    types::{Error, Result, canon, uint},
};
use crate::{
    compare::Comparison,
    model::{Entry, digest, path_key},
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

fn revision(entry: Option<&&Entry>) -> Value {
    match entry {
        Some(e) if e.state == "observed" => {
            json!({"content_sha256":e.content_sha256,"size_bytes":e.size_bytes})
        }
        _ => Value::Null,
    }
}
fn check(observation: &str, revision: Value, source: Option<&Source>) -> Value {
    let reference = if !revision.is_null() {
        source
            .map(|s| s.descriptor["source_ref"].clone())
            .unwrap_or(Value::Null)
    } else {
        Value::Null
    };
    json!({"observation":observation,"state":if ["absent","excluded"].contains(&observation){observation}else{"not_attempted"},"expected_revision":revision,
        "source_ref":reference,"actual_revision":null,"hash_check":null,"size_check":null,"reasons":[]})
}
fn reason(check: &mut Value, why: &str) {
    check["reasons"] = json!([why]);
}
fn reasons(checks: &[Value; 2]) -> Vec<String> {
    checks
        .iter()
        .flat_map(|c| {
            c["reasons"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| r.as_str().unwrap().to_owned())
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
fn unit(
    path: &[u8],
    classification: &str,
    views: &[metric::View<'_>; 2],
    sources: &[Option<Source>; 2],
    ctx: &mut Context<'_>,
) -> Result<Value> {
    let entries = [views[0].entries.get(path), views[1].entries.get(path)];
    let observations = [views[0].observation(path), views[1].observation(path)];
    let mut checks = std::array::from_fn::<_, 2, _>(|i| {
        check(observations[i], revision(entries[i]), sources[i].as_ref())
    });
    let mut status = "unavailable";
    let mut why = vec![];
    let mut non_text = vec![];
    let mut calc = None;
    if observations.contains(&"unknown") {
        why.push("observation_unknown".to_owned());
        for check in &mut checks {
            check["state"] = json!("not_attempted");
            reason(check, "observation_unknown");
        }
    } else if ["entered_scope", "left_scope"].contains(&classification) {
        status = "out_of_scope";
        why.push("scope_transition".into());
    } else {
        let sizes = std::array::from_fn::<_, 2, _>(|i| {
            entries[i]
                .filter(|e| e.state == "observed")
                .map(|e| e.size_bytes.as_ref().unwrap().parse::<u64>().unwrap())
                .unwrap_or(0)
        });
        let wb = 2 * (sizes[0] as u128 + sizes[1] as u128 + 2) + 65536;
        if sizes.iter().any(|s| *s > ctx.limits.get("max_file_bytes")) {
            for i in 0..2 {
                if sizes[i] > ctx.limits.get("max_file_bytes") {
                    reason(&mut checks[i], "file_bytes_limit");
                }
            }
            why.push("file_bytes_limit".into());
        } else if !ctx.memory(wb) {
            for i in 0..2 {
                if observations[i] == "observed" {
                    reason(&mut checks[i], "memory_budget");
                }
            }
            why.push("memory_budget".into());
        } else {
            let mut data: [Vec<u8>; 2] = [vec![], vec![]];
            for i in 0..2 {
                if observations[i] == "absent" {
                    continue;
                }
                let Some(source) = &sources[i] else {
                    reason(&mut checks[i], "source_missing");
                    continue;
                };
                let shared = i == 1
                    && sources[0].as_ref().is_some_and(|a| {
                        a.shared(
                            source,
                            &checks[0]["expected_revision"],
                            &checks[1]["expected_revision"],
                        )
                    });
                if shared {
                    for key in [
                        "state",
                        "actual_revision",
                        "hash_check",
                        "size_check",
                        "reasons",
                    ] {
                        checks[1][key] = checks[0][key].clone();
                    }
                    if !source.reference_stable() {
                        let mut why: Vec<_> = checks[1]["reasons"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|r| r.as_str().unwrap().to_owned())
                            .collect();
                        why.push("source_unstable".into());
                        why.sort();
                        why.dedup();
                        checks[1]["state"] = json!("failed");
                        checks[1]["reasons"] = json!(why);
                    }
                    data[1] = data[0].clone();
                } else {
                    let rev = checks[i]["expected_revision"].clone();
                    data[i] = source.object(&rev, ctx, &mut checks[i])?;
                }
            }
            why = reasons(&checks);
            if why.is_empty() {
                let mut lines = [vec![], vec![]];
                for i in 0..2 {
                    match metric::normalize(&data[i]) {
                        Ok(text) => lines[i] = text,
                        Err(r) => non_text
                            .push(json!({"side":if i==0{"baseline"}else{"current"},"reason":r})),
                    }
                }
                if !non_text.is_empty() {
                    status = "out_of_scope";
                    why.push("non_text".into());
                } else if lines
                    .iter()
                    .any(|l| l.len() as u64 > ctx.limits.get("max_lines_per_side"))
                {
                    why.push("line_count_limit".into());
                } else {
                    let (n, t) = (lines[0].len() as u128, lines[1].len() as u128);
                    let w = wb + 32 * (n + t) + 8 * (n.min(t) + 1);
                    if !ctx.memory(w) {
                        why.push("memory_budget".into());
                    } else {
                        calc = metric::diff(&lines[0], &lines[1], ctx)?;
                        if calc.is_some() {
                            status = "available";
                        } else {
                            why.push("diff_work_limit".into());
                        }
                    }
                }
            }
        }
    }
    let mut result = json!({"path_key":path_key(path),"classification":classification,"before":entries[0],"after":entries[1],
        "status":status,"reasons":why,"non_text":non_text,"checks":{"baseline":checks[0],"current":checks[1]}});
    if let Some(calc) = calc {
        result["calculation"] = calc;
    }
    validate_unit(&result)?;
    Ok(result)
}
fn validate_unit(unit: &Value) -> Result<()> {
    if unit["status"] == "available" {
        if unit["reasons"] != json!([]) || unit["non_text"] != json!([]) {
            return Err(Error("input_invalid"));
        }
        for side in ["baseline", "current"] {
            let check = &unit["checks"][side];
            if check["state"] == "verified" {
                if check["actual_revision"] != check["expected_revision"]
                    || check["hash_check"] != true
                    || check["size_check"] != true
                    || check["source_ref"].is_null()
                {
                    return Err(Error("input_invalid"));
                }
            } else if check["state"] != "absent" {
                return Err(Error("input_invalid"));
            }
        }
        let c = &unit["calculation"];
        if metric::calculation(
            uint(&c["n0"])? as u128,
            uint(&c["nt"])? as u128,
            uint(&c["l"])? as u128,
        )? != *c
        {
            return Err(Error("input_invalid"));
        }
    } else if unit.get("calculation").is_some()
        || unit["reasons"].as_array().is_none_or(|r| r.is_empty())
    {
        return Err(Error("input_invalid"));
    }
    Ok(())
}
pub(crate) fn assemble(
    manifests: &[Manifest; 2],
    sources: &[Option<Source>; 2],
    comparison: Comparison,
    ctx: &mut Context<'_>,
) -> Result<Value> {
    let views = [
        metric::View::new(&manifests[0].snapshot)?,
        metric::View::new(&manifests[1].snapshot)?,
    ];
    let classes: BTreeMap<_, _> = comparison
        .changes
        .iter()
        .map(|c| (c.path_key.as_str(), c.classification.as_str()))
        .collect();
    let mut paths = BTreeSet::new();
    for view in &views {
        for (p, e) in &view.entries {
            if e.state == "observed" {
                paths.insert(p.clone());
            }
        }
    }
    for change in &comparison.changes {
        if change.classification == "unknown" {
            paths.insert(
                crate::model::decode_path_key(&change.path_key)
                    .map_err(|_| Error("input_invalid"))?,
            );
        }
    }
    let mut units = vec![];
    for path in paths {
        ctx.check("before_unit")?;
        let key = path_key(&path);
        let class = classes.get(key.as_str()).copied().unwrap_or("unchanged");
        units.push(unit(&path, class, &views, sources, ctx)?);
        ctx.check("after_unit")?;
    }
    let available: Vec<_> = units
        .iter()
        .filter(|u| u["status"] == "available")
        .collect();
    let t = available.len();
    let k = units.len();
    let unresolved = units
        .iter()
        .filter(|u| u["status"] == "unavailable")
        .count();
    let blocked = comparison.quality == "partial" || unresolved > 0;
    let mut c = 0u128;
    let mut m = 0u128;
    for unit in &available {
        c = c
            .checked_add(uint(&unit["calculation"]["c"])? as u128)
            .ok_or(Error("arithmetic_overflow"))?;
        m = m
            .checked_add(uint(&unit["calculation"]["m"])? as u128)
            .ok_or(Error("arithmetic_overflow"))?;
    }
    let value = metric::ratio(c, m)?;
    let zero = if m == 0 {
        json!("zero_text_mass")
    } else {
        Value::Null
    };
    let status = if blocked {
        if t > 0 { "partial" } else { "unavailable" }
    } else if t > 0 || k == 0 {
        "complete"
    } else {
        "unavailable"
    };
    let mut reasons = vec![];
    if comparison.quality == "partial" {
        reasons.push("foundation_partial");
    }
    if t == 0 && status != "complete" {
        reasons.push("no_text_units");
    }
    if unresolved > 0 {
        reasons.push("unresolved_units");
    }
    let subset = if t > 0 {
        json!({"scope":"resolved_text_units_only","value":value,"line_mass":m.to_string(),"zero_basis":zero,"resolved_text_units":t.to_string()})
    } else {
        Value::Null
    };
    let measurement = json!({"status":status,"value":if status=="complete"{value}else{Value::Null},"line_mass":if status=="complete"{json!(m.to_string())}else{Value::Null},
        "zero_basis":if status=="complete"{if t==0{json!("empty_universe")}else{zero}}else{Value::Null},"reasons":reasons,"known_subset":subset});
    let mut exclusions = vec![];
    for (i, manifest) in manifests.iter().enumerate() {
        for e in &manifest.snapshot.entries {
            if e.state == "excluded" {
                exclusions.push((
                    crate::model::decode_path_key(&e.path_key).unwrap(),
                    i,
                    json!({"side":if i==0{"baseline"}else{"current"},"entry":e}),
                ));
            }
        }
    }
    exclusions.sort_by(|a, b| (&a.0, a.1).cmp(&(&b.0, b.1)));
    let mut summaries = vec![];
    for manifest in manifests {
        let original =
            serde_json::to_value(&manifest.snapshot).map_err(|_| Error("input_invalid"))?;
        let mut s = json!({});
        for key in [
            "snapshot_id",
            "schema_version",
            "contract_version",
            "producer_version",
            "producer_build",
            "started_at",
            "finished_at",
            "consistency",
            "quality",
            "state_digest",
        ] {
            s[key] = original[key].clone();
        }
        s["snapshot_file_sha256"] = json!(digest(&manifest.raw));
        summaries.push(s);
    }
    let policy = ctx.limits.policy();
    let mut policy_bytes = b"compass-d0-policy-v1\0".to_vec();
    policy_bytes.extend(canon(&policy)?);
    let a = &manifests[0].snapshot;
    Ok(
        json!({"report_type":"d0_measurement","schema_version":"1","d0_contract_version":"0.1.0","producer_version":env!("CARGO_PKG_VERSION"),"producer_build":crate::BUILD_ID,
        "sensor_id":"d0_line_mass_v1","metric_version":"1","metric_policy":policy,"metric_policy_id":digest(&policy_bytes),
        "workspace_id":a.workspace_id,"path_namespace":a.path_namespace,"policy_id":a.policy_id,"baseline":summaries[0],"current":summaries[1],
        "sources":{"baseline":sources[0].as_ref().map(|s|&s.descriptor),"current":sources[1].as_ref().map(|s|&s.descriptor)},
        "foundation_quality":comparison.quality,"foundation_relation":comparison.relation,"comparison":comparison,"measurement":measurement,"units":units,
        "scope":{"candidates":k.to_string(),"resolved_text_units":t.to_string(),"unavailable_units":unresolved.to_string(),"non_text_units":units.iter().filter(|u|!u["non_text"].as_array().unwrap().is_empty()).count().to_string(),
            "scope_transitions":units.iter().filter(|u|u["classification"]=="entered_scope"||u["classification"]=="left_scope").count().to_string(),"exclusions":exclusions.into_iter().map(|(_,_,e)|e).collect::<Vec<_>>()},"resources":ctx.value()}),
    )
}
