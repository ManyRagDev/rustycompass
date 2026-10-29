#!/usr/bin/env python3
"""Independent numerical check: dense symmetric eigendecomposition, never Poisson.

Run with the bundled Python (NumPy required):
  <python> verification/recovery/heat_oracle.py
The Rust executable is a JSON-lines adapter over the public diffusion API.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

import numpy as np

ROOT = Path(__file__).resolve().parents[2]


def fixtures():
    cases = []
    for name, ids, edges, seeds in [
        ("two-node", ["a", "b"], [("a", "b", 1)], [("a", 1)]),
        ("chain", ["a", "b", "c", "d"], [("a", "b", 1), ("b", "c", 1), ("c", "d", 1)], [("a", 1)]),
        ("star", ["a", "b", "c", "d", "e"], [("a", "b", 1), ("a", "c", 1), ("a", "d", 1), ("a", "e", 1)], [("b", 1)]),
        ("disconnected", ["a", "b", "c", "d", "e"], [("a", "b", 1), ("c", "d", 1)], [("a", 1)]),
        ("weighted", ["a", "b", "c"], [("a", "b", 0.25), ("b", "c", 3.5)], [("a", 2), ("c", 1)]),
        ("duplicate-self", ["a", "b", "c"], [("a", "b", 1), ("b", "a", 1), ("b", "b", 5)], [("a", 1)]),
        ("isolated", ["a", "b"], [], [("a", 1)]),
        ("empty-seed", ["a", "b"], [("a", "b", 1)], []),
    ]:
        for theta in [0, 1e-6, 0.1, 1, 3, 10]:
            cases.append(dict(name=f"{name}@{theta}", ids=ids, edges=edges, seeds=seeds, theta=theta))
    rng = np.random.default_rng(2030917)
    for index in range(25):
        count = int(rng.integers(2, 25))
        ids = [f"v{i:02d}" for i in range(count)]
        edges = [(a, b, float(rng.uniform(0.01, 5))) for i, a in enumerate(ids) for b in ids[i + 1:] if rng.random() < 0.15]
        seeds = [(node, float(rng.uniform(0.01, 1))) for node in ids if rng.random() < 0.2] or [(ids[0], 1)]
        cases.append(dict(name=f"random-{index}", ids=ids, edges=edges, seeds=seeds, theta=float(rng.uniform(0, 10))))
    cases.append(dict(name="work-limit", ids=["a", "b"], edges=[("a", "b", 1)], seeds=[("a", 1)], theta=3, max_edge_visits=0))
    return cases


def oracle(case, ids):
    index = {name: i for i, name in enumerate(ids)}
    adjacency = np.zeros((len(ids), len(ids)))
    for a, b, weight in case["edges"]:
        if a != b:
            adjacency[index[a], index[b]] = weight
            adjacency[index[b], index[a]] = weight
    laplacian = np.diag(adjacency.sum(axis=1)) - adjacency
    seed = np.zeros(len(ids))
    for name, mass in case["seeds"]:
        seed[index[name]] += mass
    if seed.sum() > 0:
        seed /= seed.sum()
    dmax = float(adjacency.sum(axis=1).max(initial=0))
    if dmax == 0:
        return seed, laplacian
    eigenvalues, eigenvectors = np.linalg.eigh(laplacian)
    expected = eigenvectors @ (np.exp(-case["theta"] * eigenvalues / dmax) * (eigenvectors.T @ seed))
    return expected, laplacian


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--skip-build", action="store_true")
    parser.add_argument("--output", type=Path, default=ROOT / "verification/recovery/heat-results.json")
    args = parser.parse_args()
    if not args.skip_build:
        subprocess.run([str(ROOT / "scripts/cargo-local.sh"), "build", "-p", "compass-core", "--example", "heat_probe", "--locked", "--offline"], cwd=ROOT, check=True)
    driver = ROOT / "target/debug/examples/heat_probe"
    cases = fixtures()
    raw = subprocess.run([str(driver)], input="".join(json.dumps(c) + "\n" for c in cases), text=True, capture_output=True, check=True)
    outputs = [json.loads(line) for line in raw.stdout.splitlines()]
    assert len(outputs) == len(cases)
    checks = []
    for case, actual in zip(cases, outputs):
        assert actual["name"] == case["name"]
        heat = actual["heat"]
        scores = np.asarray(heat["scores"])
        expected, laplacian = oracle(case, actual["ids"])
        error = float(np.abs(scores - expected).sum())
        tail = heat["truncation_tail_bound"]
        assert error <= tail + 2e-13, (case["name"], error, tail, scores, expected)
        assert np.all(scores >= 0), case["name"]
        assert scores.sum() <= 1 + 2e-13, case["name"]
        expected_mass = 1 if sum(mass for _, mass in case["seeds"]) else 0
        assert abs(float(scores.sum()) - expected_mass) <= tail + 2e-13
        if case["name"].startswith("disconnected"):
            assert scores[2:].tolist() == [0, 0, 0]
        if case["name"] == "work-limit":
            assert heat["status"] == "work_limit"
        else:
            assert heat["status"] == "complete"
        checks.append(dict(name=case["name"], l1_error=error, analytic_tail_bound=tail,
                           mass=float(scores.sum()), energy=float(scores @ laplacian @ scores), status=heat["status"]))
    # Independent monotonic energy checks across each fixed fixture's theta sequence.
    for prefix in ["two-node", "chain", "star", "weighted"]:
        energy = [c["energy"] for c in checks if c["name"].startswith(prefix + "@")]
        assert all(a + 2e-13 >= b for a, b in zip(energy, energy[1:])), prefix
    result = dict(status="passed", oracle="numpy.linalg.eigh; exp(-theta * L / max_degree)",
                  numpy_version=np.__version__, fixture_count=len(checks),
                  binary_sha256=hashlib.sha256(driver.read_bytes()).hexdigest(),
                  source_sha256={str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
                                 for p in [ROOT / "Cargo.lock", ROOT / "crates/compass-core/src/diffusion.rs", ROOT / "crates/compass-core/examples/heat_probe.rs"]},
                  checks=checks)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(dict(status="passed", cases=len(checks),
                          worst_complete_l1_error=max(c["l1_error"] for c in checks if c["status"] == "complete"),
                          work_limit_l1_error=next(c["l1_error"] for c in checks if c["status"] == "work_limit"),
                          result=str(args.output))))


if __name__ == "__main__":
    main()
