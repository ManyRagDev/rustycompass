//! JSON-lines adapter for an independent spectral oracle. No dense operator,
//! policy, or expected value is embedded here.
use compass_core::diffusion::{Graph, SpectralConfig, SpectralLimits, SpectralSample};
use serde::Deserialize;
use serde_json::json;
use std::io::{self, BufRead};
use std::sync::atomic::AtomicBool;

#[derive(Deserialize)]
struct Case {
    case_id: String,
    node_count: usize,
    edges: Vec<(usize, usize, f64)>,
    seeds: Vec<(usize, f64)>,
    scales: Vec<f64>,
    #[serde(default)]
    config: SpectralConfig,
    #[serde(default)]
    limits: SpectralLimits,
    #[serde(default)]
    cancelled: bool,
}

fn id(index: usize) -> String {
    format!("{index:06}")
}

fn run(case: Case) -> serde_json::Value {
    let cancel = AtomicBool::new(case.cancelled);
    let mut samples: Vec<SpectralSample> = Vec::new();
    let result = Graph::weighted(
        (0..case.node_count).map(id).collect(),
        &case
            .edges
            .into_iter()
            .map(|(a, b, weight)| (id(a), id(b), weight))
            .collect::<Vec<_>>(),
    )
    .and_then(|graph| {
        let mut trajectory = graph.spectral_trajectory(
            &case
                .seeds
                .into_iter()
                .map(|(index, mass)| (id(index), mass))
                .collect::<Vec<_>>(),
            case.config,
            case.limits,
            &cancel,
        )?;
        for theta in case.scales {
            match trajectory.sample(theta, &cancel)? {
                Some(sample) => samples.push(sample),
                None => break,
            }
        }
        Ok(json!({
            "case_id": case.case_id,
            "status": trajectory.status(),
            "samples": samples,
            "work": trajectory.work(),
        }))
    });
    match result {
        Ok(value) => value,
        Err(error) => json!({
            "case_id": case.case_id,
            "status": if case.cancelled { "cancelled" } else { "error" },
            "samples": samples,
            "error": error.code,
            "reason": error.message,
        }),
    }
}

fn main() {
    for line in io::stdin().lock().lines() {
        let output = match line {
            Ok(line) => match serde_json::from_str::<Case>(&line) {
                Ok(case) => run(case),
                Err(error) => json!({"status":"invalid_input", "reason":error.to_string()}),
            },
            Err(error) => json!({"status":"input_error", "reason":error.to_string()}),
        };
        println!("{output}");
    }
}
