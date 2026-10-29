#!/usr/bin/env python3
"""Separate, synthetic relation-weight experiment; no production policy changes.

Fixtures, queries and relevance below are declared before invoking the public
Rust kernel probe. Each graph represents a function that calls a helper,
imports a module locally and contains a nested declaration. Two label orders
control the tie-breaking effect. The same graph has both call and import tasks.
These controls demonstrate policy tradeoffs; they are not a relevance benchmark.
"""
import argparse
import hashlib
import json
from pathlib import Path
import statistics
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
THETA = 1.0
TOP_K = 1
SEEDS = [("entry", 1.0)]
REPETITIONS = 7
POLICIES = {
    "unit": {"contains": 1.0, "imports": 1.0, "calls": 1.0},
    "calls_favored": {"contains": 1.0, "imports": 0.25, "calls": 1.0},
    "imports_favored": {"contains": 1.0, "imports": 1.0, "calls": 0.25},
}


def fixtures():
    # Relevance is specified from task intent, independently of weight or score.
    cases = []
    for name, call, imported in [
        ("call-id-first", "a_call_helper", "b_imported_module"),
        ("import-id-first", "b_call_helper", "a_imported_module"),
    ]:
        cases.append({
            "name": name,
            "ids": ["entry", call, imported, "c_nested_declaration"],
            "typed_edges": [
                ["entry", call, "calls"],
                ["entry", imported, "imports"],
                ["entry", "c_nested_declaration", "contains"],
            ],
            "queries": [
                {"name": "follow-call", "text": "Find the helper implementation called by entry.", "intent": "calls", "relevant": [call]},
                {"name": "follow-import", "text": "Find the module imported locally by entry.", "intent": "imports", "relevant": [imported]},
            ],
        })
    return cases


def invoke(driver, request):
    start = time.perf_counter_ns()
    run = subprocess.run([str(driver)], input=json.dumps(request) + "\n", text=True,
                         capture_output=True, check=True)
    elapsed = time.perf_counter_ns() - start
    output = json.loads(run.stdout)
    assert "error" not in output, output
    assert output["heat"]["status"] == "complete", output
    return output, elapsed


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--driver", type=Path, default=ROOT / "target/debug/examples/heat_probe")
    parser.add_argument("--output", type=Path, default=ROOT / "verification/recovery/weights-results.json")
    args = parser.parse_args()
    controls = fixtures()
    measurements, evaluations = [], []
    for fixture in controls:
        for policy, weights in POLICIES.items():
            request = dict(name=f"{fixture['name']}/{policy}", ids=fixture["ids"],
                           edges=[(a, b, weights[kind]) for a, b, kind in fixture["typed_edges"]],
                           seeds=SEEDS, theta=THETA, max_edge_visits=20_000_000)
            invoke(args.driver, request)  # one warm-up process, excluded from timing
            runs = [invoke(args.driver, request) for _ in range(REPETITIONS)]
            output = runs[0][0]
            assert all(run == output for run, _ in runs), "kernel output must be deterministic"
            heat = output["heat"]
            scores = dict(zip(output["ids"], heat["scores"]))
            ranking = sorted((node for node in fixture["ids"] if node != "entry"),
                             key=lambda node: (-scores[node], node))
            selected = ranking[:TOP_K]
            measurements.append(dict(fixture=fixture["name"], policy=policy,
                                     selected=selected, ranking=ranking, scores=scores,
                                     iterations=heat["iterations"], edge_visits=heat["edge_visits"],
                                     analytic_tail_bound=heat["truncation_tail_bound"],
                                     elapsed_subprocess_ns=[elapsed for _, elapsed in runs],
                                     median_elapsed_subprocess_ns=statistics.median(elapsed for _, elapsed in runs)))
            for query in fixture["queries"]:
                relevant = set(query["relevant"])
                hits = len(set(selected) & relevant)
                evaluations.append(dict(fixture=fixture["name"], policy=policy,
                                        query=query["name"], intent=query["intent"],
                                        selected=selected, relevant=sorted(relevant),
                                        precision_at_k=hits / TOP_K,
                                        recall_at_k=hits / len(relevant)))
    summary = []
    for policy in POLICIES:
        group = [e for e in evaluations if e["policy"] == policy]
        summary.append(dict(policy=policy, query_count=len(group),
                            macro_precision_at_k=statistics.mean(e["precision_at_k"] for e in group),
                            macro_recall_at_k=statistics.mean(e["recall_at_k"] for e in group),
                            by_intent={intent: dict(
                                precision_at_k=statistics.mean(e["precision_at_k"] for e in group if e["intent"] == intent),
                                recall_at_k=statistics.mean(e["recall_at_k"] for e in group if e["intent"] == intent))
                                for intent in ["calls", "imports"]}))
    # Named controls require the expected tradeoff rather than a tuned winner.
    by_policy = {s["policy"]: s for s in summary}
    assert by_policy["calls_favored"]["by_intent"]["calls"]["recall_at_k"] == 1
    assert by_policy["calls_favored"]["by_intent"]["imports"]["recall_at_k"] == 0
    assert by_policy["imports_favored"]["by_intent"]["imports"]["recall_at_k"] == 1
    assert by_policy["imports_favored"]["by_intent"]["calls"]["recall_at_k"] == 0
    result = dict(
        status="completed", experiment="synthetic_relation_weights_v1",
        protocol=dict(theta=THETA, top_k=TOP_K, seeds=SEEDS, repetitions=REPETITIONS,
                      ranking="heat descending; node ID ascending ties; seed excluded",
                      timing="external subprocess wall time includes startup, JSON, graph construction and heat; no kernel-only latency claim",
                      relevance="predeclared from query intent; same graph/seeds/theta/top-k for every policy",
                      limitations="two tiny label-permuted synthetic graphs; no held-out task corpus or statistical/generalization claim; not PostSpark"),
        policies=POLICIES, fixtures=controls, measurements=measurements,
        evaluations=evaluations, summary=summary,
        conclusion="Keep unit weights in production. These balanced synthetic controls show that favoring calls helps call tasks and hurts import tasks, while favoring imports reverses the outcome. They supply no generalizable evidence for a fixed non-unit policy. Timing is descriptive and process-dominated.",
        binary_sha256=hashlib.sha256(args.driver.read_bytes()).hexdigest(),
        source_sha256={str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
                       for p in [Path(__file__).resolve(), ROOT / "crates/compass-core/src/diffusion.rs",
                                 ROOT / "crates/compass-core/examples/heat_probe.rs", ROOT / "Cargo.lock"]},
    )
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(dict(status="completed", graphs=len(controls), queries=len(controls)*2,
                         policies=len(POLICIES), summary=summary, result=str(args.output))))


if __name__ == "__main__":
    main()
