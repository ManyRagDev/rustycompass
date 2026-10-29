#!/usr/bin/env python3
"""Independent dense controls, derived from plan 11A before solver inspection.

This is verification only. No production imports, uniformization, or Poisson
series. Eigh, spectral moments, and physical-time invariants cross-check one
another. Reference roundoff is an empirical allowance, never certification.
"""
import argparse
import hashlib
import json
from pathlib import Path

import numpy as np

HERE = Path(__file__).resolve().parent


def fixtures():
    scales = [0.0, 1e-6, .01, .1, .5, 1., 2., 4., 8., 16., 64., 256., 1807.2, 4096.]
    bridge = [[a, b, 1.] for start in (0, 6) for a in range(start, start + 6)
              for b in range(a + 1, start + 6)] + [[5, 6, .02]]
    cases = []
    def add(name, n, edges, seeds, theta=scales, **extra):
        cases.append(dict(case_id=name, node_count=n, edges=edges, seeds=seeds, scales=theta, **extra))
    add("two-node", 2, [[0, 1, 1.]], [[0, 1.]])
    add("two-node-two-seeds", 2, [[0, 1, 1.]], [[0, 1.], [1, 2.]])
    add("k6-bridge-one", 12, bridge, [[0, 1.]])
    add("k6-bridge-two-same", 12, bridge, [[0, 1.], [1, 1.]])
    add("k6-bridge-two-opposite", 12, bridge, [[0, 1.], [11, 1.]])
    add("k6-bridge-uniform", 12, bridge, [[i, 1.] for i in range(12)])
    add("k6-bridge-weight-rescale", 12, [[a, b, w * 17] for a, b, w in bridge], [[0, 1.]])
    add("k6-bridge-isolates", 21, bridge, [[0, 1.]])
    # Same physical times; dmax changes from 5.02 to 90.0.
    high_degree = [[12, i, 10.] for i in range(13, 22)]
    add("k6-bridge-unseeded-high-degree", 22, bridge + high_degree, [[0, 1.]],
        [x * 90 / 5.02 for x in scales])
    add("isolated-seed", 21, bridge, [[20, 1.]])
    add("no-edges", 7, [], [[0, 1.]])
    add("zero-seed", 12, bridge, [])
    add("zero-graph", 0, [], [])
    rng = np.random.default_rng(2718281828)
    for i in range(12):
        n = int(rng.integers(3, 22))
        edges = [[a, b, float(rng.uniform(.001, 7))] for a in range(n)
                 for b in range(a + 1, n) if rng.random() < .2]
        add(f"random-{i}", n, edges, [[0, 1.], [n - 1, .3]], [.02, 1., 23., 1700.])
    return cases


def reference(case):
    n = case['node_count']
    adjacency = np.zeros((n, n), dtype=np.float64)
    for a, b, w in case['edges']:
        assert a != b and w > 0 and np.isfinite(w)
        adjacency[a, b] += w
        adjacency[b, a] += w
    degree = adjacency.sum(axis=1)
    dmax = float(degree.max(initial=0))
    operator = (np.diag(degree) - adjacency) / (dmax or 1)
    v = np.zeros(n)
    for index, mass in case['seeds']:
        v[index] += mass
    if v.sum():
        v /= v.sum()
    norm2 = float(v @ v)
    values, vectors = np.linalg.eigh(operator)
    # Values below the backward-error scale are a known nullspace; clipping
    # avoids exponentially amplifying eigensolver roundoff at very large theta.
    cutoff = 64 * np.finfo(float).eps * max(1, n)
    assert np.all(values >= -cutoff)
    values[np.abs(values) <= cutoff] = 0.
    coefficients = vectors.T @ v
    samples = []
    for theta in case['scales']:
        h = vectors @ (np.exp(-theta * values) * coefficients)
        y = vectors @ (np.exp(-theta / 2 * values) * coefficients)
        denom = float(y @ y)
        if denom and norm2:
            ay = operator @ y
            mu = float(y @ ay / denom)
            residual = theta * (ay - mu * y)
            cq = float(residual @ residual / denom)
            weights = np.exp(-theta * values) * coefficients ** 2
            moment_mu = float(weights @ values / weights.sum())
            moment_cq = float(theta ** 2 * (weights @ (values - moment_mu) ** 2) / weights.sum())
            assert abs(cq - moment_cq) <= 1e-20 + 2e-10 * abs(moment_cq), (case['case_id'], theta, cq, moment_cq)
            z = denom / norm2
        else:
            mu = cq = moment_cq = z = None
        samples.append(dict(theta=theta, h=h.tolist(), y=y.tolist(), z_q=z,
                            c_q=cq, moment_c_q=moment_cq, mu=mu))
    return dict(case_id=case['case_id'], dmax=dmax, samples=samples,
                informative=bool(norm2 and np.linalg.norm(operator @ v) > cutoff))


def self_check(cases, outputs):
    by_id = {x['case_id']: x for x in outputs}
    two = by_id['two-node']
    worst = 0.
    for s in two['samples']:
        t = s['theta']
        exp = np.exp(-2 * t)
        expected_h = [(1 + exp) / 2, (1 - exp) / 2]
        # Stable analytic form of t²/cosh²(t), including theta=4096.
        expected_c = 4 * t ** 2 * exp / (1 + exp) ** 2
        expected_z = (1 + exp) / 2
        assert np.max(np.abs(np.array(s['h']) - expected_h)) < 5e-15
        assert abs(s['z_q'] - expected_z) < 5e-15
        err = abs(s['c_q'] - expected_c)
        # Residual subtraction loses relative accuracy near stationarity. An
        # empirical operator-roundoff allowance scales with theta and sqrt(C).
        residual_allowance = 16 * t * np.finfo(float).eps
        assert err <= 2 * np.sqrt(expected_c) * residual_allowance + residual_allowance ** 2 + 1e-28
        worst = max(worst, err)
    base = by_id['k6-bridge-one']['samples']
    for other in ('k6-bridge-weight-rescale', 'k6-bridge-isolates', 'k6-bridge-unseeded-high-degree'):
        for a, b in zip(base, by_id[other]['samples']):
            assert np.linalg.norm(np.array(a['h']) - np.array(b['h'][:12])) < 2e-11
            assert abs(a['z_q'] - b['z_q']) < 2e-11
            assert abs(a['c_q'] - b['c_q']) < 1e-10
            assert np.linalg.norm(b['h'][12:]) < 1e-14
    for output in outputs:
        for sample in output['samples']:
            if sample['z_q'] is not None:
                assert 0 < sample['z_q'] <= 1 + 2e-12
                assert abs(sum(sample['h']) - 1) < 2e-11
    return dict(analytic_two_node_worst_abs_c_error=worst,
                physical_time_disconnected='passed', weight_rescale='passed',
                independent_residual_and_moments='passed')


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--output', type=Path, default=HERE / 'numerical-reference.json')
    p.add_argument('--requests', type=Path, default=HERE / 'numerical-requests.jsonl')
    a = p.parse_args()
    cases = fixtures()
    outputs = [reference(case) for case in cases]
    checks = self_check(cases, outputs)
    a.requests.write_text(''.join(json.dumps(x, separators=(',', ':')) + '\n' for x in cases))
    result = dict(status='passed', oracle='numpy.linalg.eigh; independent spectral moments and residual',
                  numpy_version=np.__version__, cases=len(cases), checks=checks,
                  specification='docs/PLANO-IMPLEMENTACAO.md section 11A, 2026-10-28',
                  oracle_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(), outputs=outputs)
    a.output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(dict(status='passed', cases=len(cases), samples=sum(len(x['samples']) for x in outputs), checks=checks)))


if __name__ == '__main__':
    main()
