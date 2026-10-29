use super::{
    resources::Context,
    types::{Error, Result, number},
};
use crate::model::{Entry, Snapshot, decode_path_key, parent};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(crate) struct View<'a> {
    pub entries: BTreeMap<Vec<u8>, &'a Entry>,
    coverage: BTreeMap<Vec<u8>, &'a crate::model::Coverage>,
}
impl<'a> View<'a> {
    pub fn new(snapshot: &'a Snapshot) -> Result<Self> {
        Ok(Self {
            entries: snapshot
                .entries
                .iter()
                .map(|e| {
                    Ok((
                        decode_path_key(&e.path_key).map_err(|_| Error("input_invalid"))?,
                        e,
                    ))
                })
                .collect::<Result<_>>()?,
            coverage: snapshot
                .directory_coverage
                .iter()
                .map(|c| {
                    Ok((
                        decode_path_key(&c.path_key).map_err(|_| Error("input_invalid"))?,
                        c,
                    ))
                })
                .collect::<Result<_>>()?,
        })
    }
    pub fn observation(&self, path: &[u8]) -> &'static str {
        let mut ancestor = path;
        loop {
            if self
                .coverage
                .get(ancestor)
                .is_some_and(|c| c.state != "complete")
            {
                return "unknown";
            }
            if ancestor.is_empty() {
                break;
            }
            ancestor = parent(ancestor);
        }
        for end in path
            .iter()
            .enumerate()
            .filter_map(|(i, b)| (*b == b'/').then_some(i))
            .chain(std::iter::once(path.len()))
        {
            let p = &path[..end];
            if let Some(e) = self.entries.get(p) {
                return match e.state.as_str() {
                    "observed" if p == path => "observed",
                    "observed" => "absent",
                    "excluded" => "excluded",
                    _ => "unknown",
                };
            }
            if !self.coverage.contains_key(p) {
                return "absent";
            }
        }
        "absent"
    }
}
pub(crate) fn normalize(raw: &[u8]) -> std::result::Result<Vec<String>, &'static str> {
    let text = std::str::from_utf8(raw).map_err(|_| "invalid_utf8")?;
    if text.contains('\0') {
        return Err("nul");
    }
    let text = text
        .strip_prefix('\u{feff}')
        .unwrap_or(text)
        .replace("\r\n", "\n");
    if text.is_empty() {
        return Ok(vec![]);
    }
    let mut lines: Vec<_> = text.split('\n').map(str::to_owned).collect();
    if text.ends_with('\n') {
        lines.pop();
    }
    Ok(lines)
}
pub(crate) fn ratio(c: u128, m: u128) -> Result<Value> {
    number(c)?;
    number(m)?;
    let d = if m == 0 { 1 } else { m };
    if c > d {
        return Err(Error("input_invalid"));
    }
    let scaled = c
        .checked_mul(1_000_000)
        .ok_or(Error("arithmetic_overflow"))?;
    let (mut q, r) = (scaled / d, scaled % d);
    if 2 * r > d || (2 * r == d && q % 2 == 1) {
        q += 1;
    }
    Ok(
        json!({"numerator":c.to_string(),"denominator":d.to_string(),"decimal":format!("{}.{:06}",q/1_000_000,q%1_000_000)}),
    )
}
pub(crate) fn calculation(n: u128, t: u128, l: u128) -> Result<Value> {
    if l > n.min(t) {
        return Err(Error("input_invalid"));
    }
    let a = t - l;
    let r = n - l;
    let c = a.checked_add(r).ok_or(Error("arithmetic_overflow"))?;
    let m = n.checked_add(t).ok_or(Error("arithmetic_overflow"))?;
    Ok(
        json!({"n0":number(n)?,"nt":number(t)?,"l":number(l)?,"a":number(a)?,"r":number(r)?,"c":number(c)?,"m":number(m)?,
        "delta":ratio(c,m)?,"zero_basis":if m==0{json!("zero_text_mass")}else{Value::Null}}),
    )
}
pub(crate) fn diff(a: &[String], b: &[String], ctx: &mut Context<'_>) -> Result<Option<Value>> {
    let mut same = a.len() == b.len();
    if same {
        for (i, (x, y)) in a.iter().zip(b).enumerate() {
            if i.is_multiple_of(4096) {
                ctx.check("lcs_checkpoint")?;
            }
            if x != y {
                same = false;
                break;
            }
        }
    }
    let n = a.len() as u128;
    let t = b.len() as u128;
    if same {
        return calculation(n, t, n).map(Some);
    }
    let cells = n.checked_mul(t).ok_or(Error("arithmetic_overflow"))?;
    if !ctx.reserve_cells(cells) {
        return Ok(None);
    }
    let (rows, cols) = if a.len() >= b.len() { (a, b) } else { (b, a) };
    let mut row = vec![0u64; cols.len() + 1];
    let mut work = 0u64;
    for x in rows {
        let mut diagonal = 0;
        for (j, y) in cols.iter().enumerate() {
            if work.is_multiple_of(4096) {
                ctx.check("lcs_checkpoint")?;
            }
            work += 1;
            let old = row[j + 1];
            row[j + 1] = if x == y {
                diagonal + 1
            } else {
                row[j + 1].max(row[j])
            };
            diagonal = old;
        }
    }
    ctx.check("lcs_finished")?;
    calculation(n, t, *row.last().unwrap() as u128).map(Some)
}

#[cfg(test)]
mod tests {
    use super::super::types::Limits;
    use super::*;
    use std::sync::atomic::AtomicBool;
    #[test]
    fn frozen_lcs_counts_aggregates_and_byte_vectors() {
        let cases: Value = serde_json::from_str(include_str!(
            "../../../../research/proposals/d0-c2/fixtures/cases.json"
        ))
        .unwrap();
        let flag = AtomicBool::new(false);
        let mut count = 0;
        for case in cases.as_array().unwrap() {
            let (mut c, mut m) = (0u128, 0u128);
            let mut n = 0;
            for literal in case["calculations"].as_array().unwrap() {
                let a: Vec<String> =
                    serde_json::from_value(literal["baseline_lines"].clone()).unwrap();
                let b: Vec<String> =
                    serde_json::from_value(literal["current_lines"].clone()).unwrap();
                let mut ctx = Context::new(Limits::default(), &flag);
                let result = diff(&a, &b, &mut ctx).unwrap().unwrap();
                for (key, expected) in literal["expected"].as_object().unwrap() {
                    assert_eq!(&result[key], expected, "{} {key}", case["id"]);
                }
                c += result["c"].as_str().unwrap().parse::<u128>().unwrap();
                m += result["m"].as_str().unwrap().parse::<u128>().unwrap();
                count += 1;
                n += 1;
            }
            if n > 0 && case["expected"].get("C").is_some() {
                assert_eq!(c.to_string(), case["expected"]["C"]);
                assert_eq!(m.to_string(), case["expected"]["M"]);
                assert_eq!(ratio(c, m).unwrap()["decimal"], case["expected"]["decimal"]);
            }
        }
        assert_eq!(count, 44);
        assert_eq!(normalize(b"\xef\xbb\xbfa\r\n").unwrap(), vec!["a"]);
        assert_eq!(
            normalize(b"\xef\xbb\xbf\xef\xbb\xbfa").unwrap(),
            vec!["\u{feff}a"]
        );
        assert_eq!(normalize(b"\0\xff").unwrap_err(), "invalid_utf8");
        assert_eq!(normalize(b"\0a").unwrap_err(), "nul");
        assert_eq!(normalize(b"a\n\n").unwrap(), vec!["a", ""]);
    }
    #[test]
    fn e36_e51_e52_e54_e55_equality_and_balances() {
        let flag = AtomicBool::new(false);
        let equal = vec!["a".to_owned(); 4000];
        let different = vec!["b".to_owned(); 4000];
        let mut ctx = Context::new(Limits::default(), &flag);
        assert_eq!(
            diff(&equal, &equal, &mut ctx).unwrap().unwrap()["m"],
            "8000"
        );
        assert_eq!(ctx.cells, 0);
        assert!(diff(&equal, &different, &mut ctx).unwrap().is_none());
        assert_eq!(ctx.cells, 0);
        let a = vec!["a".to_owned(), "b".to_owned()];
        let b = vec!["a".to_owned(), "c".to_owned()];
        for first_equal in [true, false] {
            let mut limits = Limits::default();
            limits.values.insert("max_cells_total".into(), 4);
            let mut ctx = Context::new(limits, &flag);
            let pairs = if first_equal {
                vec![(&a, &a), (&a, &b)]
            } else {
                vec![(&a, &b), (&a, &a)]
            };
            for (x, y) in pairs {
                assert!(diff(x, y, &mut ctx).unwrap().is_some());
            }
            assert_eq!(ctx.cells, 4);
        }
        let mut ctx = Context::new(Limits::default(), &flag);
        assert_eq!(
            diff(
                &normalize(b"a\r\n").unwrap(),
                &normalize(b"a\n").unwrap(),
                &mut ctx
            )
            .unwrap()
            .unwrap()["delta"],
            ratio(0, 2).unwrap()
        );
        assert_eq!(ctx.cells, 0);
    }
    #[test]
    fn e49_u64_and_checked_equations() {
        for (c, m, s) in [
            (1, 2_000_000, "0.000000"),
            (3, 2_000_000, "0.000002"),
            (1, 10_000_000, "0.000000"),
            (1_999_999, 2_000_000, "1.000000"),
        ] {
            assert_eq!(ratio(c, m).unwrap()["decimal"], s);
        }
        assert!(ratio(u64::MAX as u128, u64::MAX as u128).is_ok());
        assert_eq!(
            ratio(u64::MAX as u128 + 1, u64::MAX as u128 + 1).unwrap_err(),
            Error("arithmetic_overflow")
        );
        assert_eq!(
            calculation(u64::MAX as u128, u64::MAX as u128, 0).unwrap_err(),
            Error("arithmetic_overflow")
        );
        assert_eq!(calculation(1, 1, 2).unwrap_err(), Error("input_invalid"));
    }
    #[test]
    fn lcs_checkpoints_bound_work_and_cancel() {
        let flag = AtomicBool::new(false);
        let mut ctx = Context::new(Limits::default(), &flag);
        let mut calls = 0;
        ctx.hook = Some(Box::new(|event| {
            if event == "lcs_checkpoint" {
                calls += 1;
                if calls == 2 {
                    flag.store(true, std::sync::atomic::Ordering::Relaxed);
                }
            }
            false
        }));
        assert_eq!(
            diff(&vec!["a".into(); 100], &vec!["b".into(); 100], &mut ctx).unwrap_err(),
            Error("cancelled")
        );
        drop(ctx);
        assert_eq!(calls, 2);
    }
}
