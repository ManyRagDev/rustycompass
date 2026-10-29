//! JSON-lines laboratory adapter. Repository text is input data only.
use compass_core::{
    codec,
    collection::Collection,
    spectral_acquisition::{self, Request},
};
use serde_json::{Value, json};
use std::{
    io::{self, BufRead},
    path::Path,
    sync::atomic::AtomicBool,
    time::Instant,
};
fn main() {
    for line in io::stdin().lock().lines() {
        let result = (|| -> compass_core::Result<Value> {
            let line = line.map_err(|e| compass_core::Error::io("stdin", e))?;
            let v: Value = serde_json::from_str(&line)
                .map_err(|e| compass_core::Error::invalid(e.to_string()))?;
            let case = v["case_id"].clone();
            let start = Instant::now();
            let result = (|| -> compass_core::Result<Value> {
                let string = |k: &str| {
                    v[k].as_str()
                        .ok_or_else(|| compass_core::Error::invalid(format!("missing {k}")))
                };
                let c = Collection::open(Path::new(string("store")?), Some(string("generation")?))?;
                let r = Request {
                    query: string("query")?.into(),
                    mode: "hybrid".into(),
                    scope: string("scope")?.into(),
                    budget_bytes: v["budget_bytes"]
                        .as_u64()
                        .ok_or_else(|| compass_core::Error::invalid("budget required"))?
                        as usize,
                };
                let (pack, work, trajectory) = spectral_acquisition::laboratory(
                    &c,
                    &r,
                    string("strategy")?,
                    v["theta"].as_f64(),
                    &AtomicBool::new(false),
                )?;
                let pack_json = String::from_utf8(codec::canonical(&pack)?).unwrap();
                Ok(
                    json!({"case_id":case,"pack":pack,"pack_json":pack_json,"work":work,"trajectory":trajectory,"wall_ns":start.elapsed().as_nanos()}),
                )
            })();
            Ok(match result {
                Ok(v) => v,
                Err(e) => json!({"case_id":case,"error":{"code":e.code,"message":e.message}}),
            })
        })();
        println!(
            "{}",
            match result {
                Ok(v) => v,
                Err(e) => json!({"error":{"code":e.code,"message":e.message}}),
            }
        );
    }
}
