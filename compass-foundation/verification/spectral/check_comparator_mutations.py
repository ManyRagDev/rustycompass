#!/usr/bin/env python3
"""Reject independent protocol/numerical-report mutations without changing truth."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import sys

sys.dont_write_bytecode = True
from compare_numerical import HERE, compare_case


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--actual', type=Path, default=HERE / 'numerical-comparison-final.json')
    p.add_argument('--output', type=Path, default=HERE / 'comparator-mutations.json')
    args = p.parse_args()
    inputs = [json.loads(x) for x in (HERE / 'numerical-requests.jsonl').read_text().splitlines()]
    truth = json.loads((HERE / 'numerical-reference.json').read_text())['outputs']
    actual = json.loads(args.actual.read_text())['raw_probe_outputs']
    lookup = {r['case_id']:(r, t, v) for r,t,v in zip(inputs, truth, actual)}
    def sample(output): return output['samples'][5]  # two-node theta=1
    def mutate_error(output, name, field, value): sample(output)[name][field] = value
    mutations = [
        ('incomplete_status', 'two-node', lambda x:x.update(status='time_limit')),
        ('no_seed_claims_estimated', 'zero-seed', lambda x:x['samples'][0].update(quality='estimated')),
        ('stationary_claims_estimated', 'k6-bridge-uniform', lambda x:x['samples'][0].update(quality='estimated')),
        ('zero_scale_claims_estimated', 'two-node', lambda x:x['samples'][0].update(quality='estimated')),
        ('late_zero_claims_estimated', 'two-node', lambda x:x['samples'][-1].update(quality='estimated')),
        ('positive_signal_claims_indeterminate', 'two-node', lambda x:sample(x).update(quality='numerically_indeterminate')),
        ('heat_negative_tail', 'two-node', lambda x:mutate_error(x, 'heat_error', 'truncation_l2', -1.)),
        ('heat_negative_roundoff', 'two-node', lambda x:mutate_error(x, 'heat_error', 'roundoff_l2_estimate', -1.)),
        ('heat_nan_roundoff', 'two-node', lambda x:mutate_error(x, 'heat_error', 'roundoff_l2_estimate', float('nan'))),
        ('heat_certified', 'two-node', lambda x:mutate_error(x, 'heat_error', 'roundoff_certified', True)),
        ('observable_negative_tail', 'two-node', lambda x:mutate_error(x, 'error', 'truncation_l2', -1.)),
        ('observable_infinite_roundoff', 'two-node', lambda x:mutate_error(x, 'error', 'roundoff_l2_estimate', float('inf'))),
        ('observable_certified', 'two-node', lambda x:mutate_error(x, 'error', 'roundoff_certified', True)),
        ('empty_c_interval', 'two-node', lambda x:sample(x).update(c_q_interval_estimate=dict(lower=0., upper=0.))),
        ('empty_z_interval', 'two-node', lambda x:sample(x).update(z_q_interval_estimate=dict(lower=0., upper=0.))),
        ('reversed_interval', 'two-node', lambda x:sample(x).update(c_q_interval_estimate=dict(lower=2., upper=1.))),
        ('infinite_interval', 'two-node', lambda x:sample(x).update(c_q_interval_estimate=dict(lower=0., upper=float('inf')))),
        ('missing_interval', 'two-node', lambda x:sample(x).update(c_q_interval_estimate=None)),
        ('work_reset', 'two-node', lambda x:sample(x)['work'].update(node_visits=0)),
    ]
    # A large observable-y allowance must not hide an incorrect selection-h.
    def separate_heat(output):
        target = sample(output)
        target['h'][0] += 1e-4
        target['error']['truncation_l2'] = 1.
        target['heat_error']['truncation_l2'] = 0.
        target['heat_error']['roundoff_l2_estimate'] = 0.
    mutations.append(('heat_uses_own_error', 'two-node', separate_heat))
    def original_reviewer_mutant(output):
        output['status'] = 'time_limit'
        for target in output['samples']:
            target['quality'] = 'estimated'
            if target['c_q'] is not None:
                target['c_q_interval_estimate'] = dict(lower=0., upper=0.)
                target['z_q_interval_estimate'] = dict(lower=0., upper=0.)
            target['heat_error'] = dict(truncation_l2=-1., roundoff_l2_estimate=-1., roundoff_certified=True)
    mutations.append(('original_reviewer_combined', 'two-node', original_reviewer_mutant))
    checks = []
    for name, case_id, mutation in mutations:
        request, expected, original = lookup[case_id]
        compare_case(request, expected, original)
        changed = copy.deepcopy(original)
        mutation(changed)
        rejected = False
        try:
            compare_case(request, expected, changed)
        except AssertionError:
            rejected = True
        assert rejected, ('survived comparator mutation', name)
        checks.append(dict(mutation=name, rejected=True))
    args.output.write_text(json.dumps(dict(status='passed', mutations=len(checks), checks=checks,
                                         actual_sha256=hashlib.sha256(args.actual.read_bytes()).hexdigest(),
                                         comparator_sha256=hashlib.sha256((HERE/'compare_numerical.py').read_bytes()).hexdigest(),
                                         runner_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest()), indent=2)+'\n')
    print(json.dumps(dict(status='passed', mutations_rejected=len(checks))))


if __name__ == '__main__':
    main()
