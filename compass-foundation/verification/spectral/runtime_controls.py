#!/usr/bin/env python3
"""Independent resource, invalid-input and semigroup controls via JSON-lines."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time

import numpy as np

sys.dont_write_bytecode = True
from compare_numerical import HERE, compare_case, error_components, validate_work
from numerical_reference import reference


def cases():
    base = dict(node_count=2, edges=[[0, 1, 1.]], seeds=[[0, 1.]], scales=[0., 1., 2.])
    controls = []
    def add(name, status='complete', **changes):
        value = copy.deepcopy(base)
        value.update(case_id=name, **changes)
        controls.append((value, status))
    for field,status in [('max_node_visits','node_limit'), ('max_edge_visits','edge_limit'),
                         ('max_operator_applications','operator_limit'), ('max_samples','sample_limit'),
                         ('max_elapsed_ms','time_limit')]:
        add(field, status, limits={field:0})
    add('accumulated-increments', 'increment_limit', limits=dict(max_increments=3))
    add('accumulated-samples', 'sample_limit', limits=dict(max_samples=1))
    add('cancelled', 'cancelled', cancelled=True)
    add('many-isolates-node-budget', 'node_limit', node_count=10000, edges=[], limits=dict(max_node_visits=100))
    add('precision-series', 'series_limit', scales=[10.], config=dict(poisson_tail_tolerance=1e-300))
    add('zero-seed', seeds=[])
    add('uniform', seeds=[[0, 1.], [1, 1.]])
    add('no-edges', node_count=5, edges=[])
    add('subnormal-theta', scales=[1e-200])
    add('decreasing-scale', 'error', scales=[1., .5])
    add('negative-scale', 'error', scales=[-1.])
    add('invalid-increment', 'error', config=dict(max_increment=0.))
    add('negative-seed', 'error', seeds=[[0, -1.]])
    add('seed-overflow', 'error', seeds=[[0, 1e308], [1, 1e308]])
    add('degree-overflow', 'error', node_count=3, edges=[[0, 1, 1e308], [0, 2, 1e308]])
    add('invalid-weight', 'error', edges=[[0, 1, -1.]])
    chain = dict(node_count=4, edges=[[0, 1, 1.], [1, 2, .3], [2, 3, 2.]], seeds=[[0, 1.]], scales=[7.])
    add('direct', **chain)
    chain['scales'] = [.25, .75, 1.5, 3., 7.]
    add('composed', **chain, config=dict(max_increment=.5))
    add('loose-truncation', **{**chain, 'scales':[1., 2., 3., 4., 5., 6., 7.]},
        config=dict(max_increment=.5, poisson_tail_tolerance=1e-3))
    return controls


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--probe', type=Path, required=True)
    p.add_argument('--output', type=Path, default=HERE / 'runtime-controls.json')
    args = p.parse_args()
    controls = cases()
    requests = ''.join(json.dumps(case)+'\n' for case,_ in controls)
    # Transport-level non-finite JSON must be rejected, not interpreted as a
    # finite parameter or allowed to contaminate an observable.
    invalid_wire = ['{"case_id":"nan","node_count":2,"edges":[[0,1,1]],"seeds":[[0,1]],"scales":[NaN]}',
                    '{"case_id":"infinite","node_count":2,"edges":[[0,1,1]],"seeds":[[0,1]],"scales":[1e999]}']
    begin = time.perf_counter_ns()
    run = subprocess.run([str(args.probe.resolve())], input=requests+'\n'.join(invalid_wire)+'\n',
                         text=True, capture_output=True, timeout=60)
    assert run.returncode == 0, run.stderr
    outputs = [json.loads(x) for x in run.stdout.splitlines()]
    assert len(outputs) == len(controls) + len(invalid_wire)
    checks = []
    counter_fields = dict(max_increments='increments', max_operator_applications='operator_applications',
                          max_node_visits='node_visits', max_edge_visits='edge_visits', max_samples='samples')
    for (case,status), actual in zip(controls, outputs):
        assert actual['case_id'] == case['case_id'] and actual['status'] == status, (case, actual)
        if status == 'complete':
            compare_case(case, reference(case), actual)
        elif status in ('error', 'cancelled'):
            assert actual['error'] == ('cancelled' if status == 'cancelled' else 'invalid_input'), actual
        else:
            validate_work(actual['work'])
            assert actual['work']['samples'] == len(actual['samples']) < len(case['scales'])
            for field,counter in counter_fields.items():
                if field in case.get('limits', {}):
                    assert actual['work'][counter] <= case['limits'][field]
            if actual['samples']:
                partial = {**case, 'scales':case['scales'][:len(actual['samples'])]}
                synthetic_complete = {**actual, 'status':'complete', 'work':actual['samples'][-1]['work']}
                compare_case(partial, reference(partial), synthetic_complete)
        checks.append(dict(case_id=case['case_id'], expected_status=status, status=actual['status'],
                           work=actual.get('work'), samples=len(actual['samples'])))
    for output in outputs[len(controls):]:
        assert output['status'] == 'invalid_input', output
    lookup = {x['case_id']:x for x in outputs if 'case_id' in x}
    direct, composed = lookup['direct']['samples'][-1], lookup['composed']['samples'][-1]
    composition = {}
    for vector, error in [('h','heat_error'), ('y','error')]:
        difference = float(np.linalg.norm(np.array(direct[vector]) - composed[vector]))
        allowance = sum(error_components(direct[error])) + sum(error_components(composed[error]))
        assert difference <= allowance + 2e-14, (vector, difference, allowance)
        composition[vector] = dict(l2_difference=difference, combined_error_estimate=allowance)
    assert composed['work']['increments'] > direct['work']['increments']
    loose = lookup['loose-truncation']['samples']
    masses = [float(sum(s['h'])) for s in loose]
    assert all(a >= b for a,b in zip(masses, masses[1:])), masses
    assert 0 < 1 - masses[-1] < .1, masses
    assert 1 - masses[-1] > 1e-8, 'loose-tail control did not exercise visible mass loss'
    seed = np.zeros(4); seed[0] = 1
    for sample in loose:
        y = np.array(sample['y'])
        assert abs(sample['z_q'] - y @ y / (seed @ seed)) < 2e-15
        assert sum(sample['h']) <= sum(sample['y']) + 2e-14
    result = dict(status='passed', controls=len(controls), malformed_json_controls=len(invalid_wire),
                  probe_sha256=hashlib.sha256(args.probe.read_bytes()).hexdigest(),
                  runner_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                  wall_ns=time.perf_counter_ns()-begin, checks=checks,
                  direct_vs_composed=composition, loose_tail_mass=masses,
                  raw_probe_outputs=outputs)
    args.output.write_text(json.dumps(result, indent=2)+'\n')
    (HERE / 'runtime-requests.jsonl').write_text(requests+'\n'.join(invalid_wire)+'\n')
    print(json.dumps(dict(status='passed', controls=len(controls), malformed_json=len(invalid_wire),
                         loose_tail_final_mass=masses[-1], direct_vs_composed=composition)))


if __name__ == '__main__':
    main()
