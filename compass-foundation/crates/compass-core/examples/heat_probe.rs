//! JSON-lines probe for an independent numerical oracle in verification/recovery.
use compass_core::diffusion::Graph;
use serde::Deserialize;
use serde_json::json;
use std::io::{self, BufRead};
use std::sync::atomic::AtomicBool;

#[derive(Deserialize)]
struct Case {
    name: String,
    ids: Vec<String>,
    edges: Vec<(String, String, f64)>,
    seeds: Vec<(String, f64)>,
    theta: f64,
    #[serde(default = "work")]
    max_edge_visits: usize,
}
fn work() -> usize {
    20_000_000
}
fn main() {
    for line in io::stdin().lock().lines() {
        let case: Case = serde_json::from_str(&line.expect("input line")).expect("valid case JSON");
        let output = Graph::weighted(case.ids, &case.edges).and_then(|g| {
            let heat = g.heat(
                &case.seeds,
                case.theta,
                case.max_edge_visits,
                &AtomicBool::new(false),
            )?;
            Ok(json!({"name":case.name,"ids":g.ids,"heat":heat}))
        });
        println!(
            "{}",
            match output {
                Ok(value) => value,
                Err(error) => json!({"name":case.name,"error":error.code,"message":error.message}),
            }
        );
    }
}
