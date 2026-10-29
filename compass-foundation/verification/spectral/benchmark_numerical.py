#!/usr/bin/env python3
"""Bounded process-level cost curves; no efficacy or 100k-node extrapolation."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import time

import numpy as np

sys.dont_write_bytecode = True
from compare_numerical import HERE, error_components, validate_work


def fixtures():
    rows = []
    limits = dict(max_elapsed_ms=250, max_node_visits=20_000_000,
                  max_edge_visits=20_000_000, max_increments=8192,
                  max_operator_applications=1_000_000, max_samples=64)
    def add(family, n, edges, scales=None, seed=0):
        rows.append(dict(case_id=f'{family}-{n}', family=family, node_count=n, edges=edges,
                         seeds=[[seed, 1.]], scales=scales or [.5, 2., 8., 32., 128.], limits=limits))
    for n in [64, 512, 4096, 16384]:
        add('path', n, [[i, i + 1, 1.] for i in range(n - 1)])
        add('star', n, [[0, i, 1.] for i in range(1, n)], seed=1)
        add('isolates', n, [])
    for width in [8, 32, 64]:
        edges = []
        for x in range(width):
            for y in range(width):
                i = x * width + y
                if x + 1 < width: edges.append([i, i + width, 1.])
                if y + 1 < width: edges.append([i, i + 1, 1.])
        add('grid', width ** 2, edges)
    for n in [64, 256, 512]:
        add('clique', n, [[i, j, 1.] for i in range(n) for j in range(i + 1, n)])
    for half in [6, 32, 128]:
        edges = [[i, j, 1.] for start in (0, half) for i in range(start, start + half)
                 for j in range(i + 1, start + half)] + [[half - 1, half, .02]]
        add('weak-bridge', half * 2, edges, [1., 16., 256., 4096., 65536.])
    return rows


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--probe', type=Path, required=True)
    parser.add_argument('--output', type=Path, default=HERE / 'numerical-benchmarks.json')
    parser.add_argument('--repetitions', type=int, default=2)
    args = parser.parse_args()
    assert 1 <= args.repetitions <= 5
    cases = fixtures()
    results = []
    probe_hash = hashlib.sha256(args.probe.read_bytes()).hexdigest()
    for case in cases:
        family = case['family']
        request = {k:v for k,v in case.items() if k != 'family'}
        degree = np.zeros(case['node_count'])
        for i,j,w in case['edges']:
            degree[i] += w
            degree[j] += w
        for repetition in range(args.repetitions):
            with tempfile.NamedTemporaryFile() as metric:
                begin = time.perf_counter_ns()
                run = subprocess.run(['/usr/bin/time', '-f', '%M', '-o', metric.name,
                                      str(args.probe.resolve())], input=json.dumps(request)+'\n',
                                     text=True, capture_output=True, timeout=15)
                wall_ns = time.perf_counter_ns() - begin
                rss = int(Path(metric.name).read_text().strip().splitlines()[-1])
            assert run.returncode == 0, run.stderr
            output = json.loads(run.stdout)
            assert output['case_id'] == case['case_id']
            assert output['status'] in ('complete', 'time_limit', 'node_limit', 'edge_limit',
                                        'operator_limit', 'increment_limit', 'series_limit', 'sample_limit'), output['status']
            validate_work(output['work'])
            for field,counter in [('max_node_visits','node_visits'), ('max_edge_visits','edge_visits'),
                                  ('max_increments','increments'), ('max_operator_applications','operator_applications')]:
                assert output['work'][counter] <= case['limits'][field]
            assert output['work']['samples'] == len(output['samples'])
            if output['status'] == 'complete': assert len(output['samples']) == len(case['scales'])
            samples = []
            previous = None
            for sample in output['samples']:
                validate_work(sample['work'], previous)
                previous = sample['work']
                h, y = np.array(sample['h']), np.array(sample['y'])
                assert h.shape == y.shape == (case['node_count'],)
                assert np.all(np.isfinite(h)) and np.all(np.isfinite(y))
                heat_error = sum(error_components(sample['heat_error']))
                state_error = sum(error_components(sample['error']))
                assert abs(h.sum() - 1) <= np.sqrt(len(h)) * heat_error + 2e-12
                assert abs(y.sum() - 1) <= np.sqrt(len(y)) * state_error + 2e-12
                if family == 'isolates': assert sample['quality'] == 'no_dynamics'
                samples.append(dict(theta=sample['theta'], quality=sample['quality'],
                                    c_q=sample['c_q'], z_q=sample['z_q'], work=sample['work'],
                                    h_mass=float(h.sum()), y_mass=float(y.sum())))
            results.append(dict(case_id=case['case_id'], family=family, repetition=repetition,
                                nodes=case['node_count'], undirected_edges=len(case['edges']),
                                max_degree=float(degree.max()), average_degree=float(degree.mean()),
                                limits=case['limits'], requested_scales=case['scales'], status=output['status'],
                                samples=samples, work=output['work'], wall_ns=wall_ns,
                                peak_rss_kib=rss, stdout_utf8_bytes=len(run.stdout.encode())))
    assert hashlib.sha256(args.probe.read_bytes()).hexdigest() == probe_hash, 'binary changed during benchmark'
    report = dict(status='measured', probe_sha256=probe_hash, profile='debug binary; fixed 250ms internal compute budget',
                  runner_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                  repetitions=args.repetitions, cases=len(cases), measurements=len(results),
                  maximum_nodes=max(x['nodes'] for x in results),
                  conditions='Separate process per graph/repetition. Wall time and RSS include parsing, graph construction, trajectory, JSON serialization and process startup. Internal elapsed excludes graph construction and output. Uncontrolled page cache, shared host, no release or 100k-node extrapolation. Failed/limited endpoints remain in results.',
                  results=results)
    args.output.write_text(json.dumps(report, indent=2)+'\n')
    summary = dict(cases=len(cases), runs=len(results), complete=sum(x['status']=='complete' for x in results),
                   maximum_nodes=report['maximum_nodes'], maximum_wall_ms=max(x['wall_ns']/1e6 for x in results),
                   maximum_peak_rss_kib=max(x['peak_rss_kib'] for x in results))
    print(json.dumps(summary))


if __name__ == '__main__':
    main()
