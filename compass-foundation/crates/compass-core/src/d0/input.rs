use super::{
    resources::Context,
    source::{self, Root, Source},
    types::{Error, Limits, Result},
};
use crate::{
    codec,
    model::{Snapshot, digest},
};
use serde_json::{Value, json};
use std::{path::Path, sync::atomic::AtomicBool};

fn depth(raw: &[u8], limit: u64) -> Result<()> {
    let (mut level, mut string, mut escape) = (0u64, false, false);
    for &b in raw {
        if string {
            if escape {
                escape = false;
            } else if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                string = false;
            }
        } else {
            match b {
                b'"' => string = true,
                b'[' | b'{' => {
                    level += 1;
                    if level > limit {
                        return Err(Error("input_limit"));
                    }
                }
                b']' | b'}' => level = level.saturating_sub(1),
                _ => {}
            }
        }
    }
    Ok(())
}
pub(crate) fn parse(raw: &[u8], limit: u64, invalid: &'static str) -> Result<Value> {
    depth(raw, limit)?;
    codec::decode(raw).map_err(|_| Error(invalid))
}
pub fn limits(path: Option<&Path>, duration: Option<u64>) -> Result<Limits> {
    let cancel = AtomicBool::new(false);
    let mut ctx = Context::new(Limits::default(), &cancel);
    let profile = path
        .map(|p| {
            source::raw_file(p, 65536, &mut ctx, "invalid_arguments")
                .and_then(|raw| parse(&raw, 64, "invalid_arguments"))
        })
        .transpose()
        .map_err(|_| Error("invalid_arguments"))?;
    Limits::reduced(profile.as_ref(), duration)
}
pub(crate) struct Manifest {
    pub snapshot: Snapshot,
    pub raw: Vec<u8>,
}
pub(crate) fn manifests(a: &Path, b: &Path, ctx: &mut Context<'_>) -> Result<[Manifest; 2]> {
    let mut raws = vec![];
    let mut values = vec![];
    let (mut records, mut n) = (0usize, 0usize);
    for path in [a, b] {
        let raw = source::raw_file(
            path,
            ctx.limits.get("max_manifest_bytes"),
            ctx,
            "input_invalid",
        )?;
        let value = parse(&raw, ctx.limits.get("max_json_depth"), "input_invalid")?;
        for field in ["entries", "directory_coverage", "issues"] {
            let len = value[field].as_array().ok_or(Error("input_invalid"))?.len();
            n += len;
            if field != "issues" {
                records += len;
            }
        }
        raws.push(raw);
        values.push(value);
    }
    if records as u64 > ctx.limits.get("max_records") {
        return Err(Error("input_limit"));
    }
    ctx.add_structural(
        8 * raws.iter().map(|r| r.len() as u128).sum::<u128>() + 4096 * n as u128 + 65536,
    )?;
    ctx.check("before_domain")?;
    let mut result = vec![];
    for (value, raw) in values.into_iter().zip(raws) {
        let snapshot: Snapshot =
            serde_json::from_value(value).map_err(|_| Error("input_invalid"))?;
        snapshot.validate().map_err(|_| Error("input_invalid"))?;
        result.push(Manifest { snapshot, raw });
    }
    let a = &result[0].snapshot;
    let b = &result[1].snapshot;
    if a.workspace_id != b.workspace_id
        || a.path_namespace != b.path_namespace
        || a.policy_id != b.policy_id
    {
        return Err(Error("incompatible_inputs"));
    }
    Ok(result.try_into().ok().unwrap())
}
pub(crate) fn source(
    path: Option<&Path>,
    manifest: &Manifest,
    ctx: &mut Context<'_>,
) -> Result<Option<Source>> {
    let Some(path) = path else {
        return Ok(None);
    };
    let root = Root::at(path)?;
    let raw = root.read_input(
        b"source.json",
        ctx.limits.get("max_source_descriptor_bytes"),
        ctx,
        "source_invalid",
    )?;
    let d = parse(&raw, ctx.limits.get("max_json_depth"), "source_invalid")?;
    let s = &manifest.snapshot;
    let expected = json!({"source_schema_version":"1","source_contract_version":"0.1.0","workspace_id":s.workspace_id,"snapshot_id":s.snapshot_id,"policy_id":s.policy_id,"snapshot_file_sha256":digest(&manifest.raw)});
    let keys = [
        "source_schema_version",
        "source_contract_version",
        "workspace_id",
        "snapshot_id",
        "policy_id",
        "snapshot_file_sha256",
    ];
    if d.as_object().is_none_or(|o| o.len() != 6)
        || keys.iter().any(|k| !d[*k].is_string())
        || d["source_schema_version"] != "1"
        || d["source_contract_version"] != "0.1.0"
    {
        return Err(Error("source_invalid"));
    }
    // Shape/domain validation precedes logical binding, including empty universes.
    for k in ["workspace_id", "snapshot_id"] {
        let text = d[k].as_str().unwrap();
        let id = uuid::Uuid::parse_str(text).map_err(|_| Error("source_invalid"))?;
        if id.get_version_num() != 4
            || id.get_variant() != uuid::Variant::RFC4122
            || id.to_string() != text
        {
            return Err(Error("source_invalid"));
        }
    }
    for k in ["policy_id", "snapshot_file_sha256"] {
        let t = d[k].as_str().unwrap();
        if t.len() != 64
            || !t
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(Error("source_invalid"));
        }
    }
    if d != expected {
        return Err(Error("incompatible_inputs"));
    }
    ctx.add_structural(8 * raw.len() as u128)?;
    let mut descriptor = d;
    descriptor["source_ref"] = json!(digest(&raw));
    Ok(Some(Source {
        root,
        descriptor,
        raw,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strict_transport_and_depth() {
        assert!(parse(b" { \"a\": \"1\" } ", 2, "input_invalid").is_ok());
        for bytes in [
            b"{\"a\":1,\"a\":2}".as_slice(),
            b"\xef\xbb\xbf{}",
            b"{} {}",
            b"\xff",
            b"NaN",
            b"Infinity",
        ] {
            assert_eq!(
                parse(bytes, 64, "input_invalid").unwrap_err(),
                Error("input_invalid")
            );
        }
        assert_eq!(
            parse(b"[[[]]]", 2, "input_invalid").unwrap_err(),
            Error("input_limit")
        );
    }
    #[test]
    fn structural_preflight_precedes_domain_validation() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("snapshot");
        // Transport preflight can count records, but must refuse before Snapshot exists.
        std::fs::write(
            &path,
            b"{\"entries\":[],\"directory_coverage\":[],\"issues\":[]}",
        )
        .unwrap();
        let cancel = AtomicBool::new(false);
        let mut limits = Limits::default();
        limits.values.insert("max_logical_memory_bytes".into(), 1);
        let mut ctx = Context::new(limits, &cancel);
        ctx.hook = Some(Box::new(|event| {
            assert_ne!(event, "before_domain");
            false
        }));
        assert!(matches!(
            manifests(&path, &path, &mut ctx),
            Err(Error("input_limit"))
        ));
        let mut reached = false;
        let mut control = Context::new(Limits::default(), &cancel);
        control.hook = Some(Box::new(|event| {
            if event == "before_domain" {
                reached = true;
            }
            false
        }));
        assert!(matches!(
            manifests(&path, &path, &mut control),
            Err(Error("input_invalid"))
        ));
        drop(control);
        assert!(reached);
    }
}
