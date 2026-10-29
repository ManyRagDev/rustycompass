#!/usr/bin/env python3
"""Compare JSON-lines probe states against an independently frozen reference.

No build is performed: coordinator selects the tested binary. All roundoff
allowances below are empirical; declared analytic truncation remains separate.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import time

import numpy as np

HERE = Path(__file__).resolve().parent


WORK_FIELDS = ('increments', 'operator_applications', 'node_visits', 'edge_visits', 'samples', 'elapsed_ms')


def error_components(error):
    assert error['roundoff_certified'] is False, ('uncertified roundoff required', error)
    values = []
    for field in ('truncation_l2', 'roundoff_l2_estimate'):
        value = error[field]
        assert isinstance(value, (float, int)) and not isinstance(value, bool)
        assert np.isfinite(value) and value >= 0, (field, value)
        values.append(float(value))
    return tuple(values)


def validate_work(work, previous=None):
    for field in WORK_FIELDS:
        assert type(work[field]) is int and work[field] >= 0, (field, work)
        if previous is not None:
            assert work[field] >= previous[field], ('work reset', field, work, previous)


def validate_interval(interval, value, truth, oracle_allowance, label):
    assert isinstance(interval, dict), (label, interval)
    lower, upper = interval['lower'], interval['upper']
    assert all(isinstance(x, (float, int)) and not isinstance(x, bool) for x in (lower, upper)), (label, interval)
    assert np.isfinite(lower) and np.isfinite(upper) and 0 <= lower <= upper, (label, interval)
    value_allowance = 8 * np.finfo(float).eps * max(abs(value), np.finfo(float).tiny)
    assert lower - value_allowance <= value <= upper + value_allowance, (label, value, interval)
    assert lower - oracle_allowance <= truth <= upper + oracle_allowance, (label, truth, interval, oracle_allowance)


def compare_case(request, expected, actual):
    assert request['case_id'] == expected['case_id'] == actual['case_id']
    assert actual['status'] == 'complete', (request['case_id'], 'incomplete trajectory', actual['status'])
    assert 'error' not in actual, (request['case_id'], actual.get('error'))
    assert len(actual['samples']) == len(expected['samples']), (request['case_id'], actual)
    n = request['node_count']
    case_checks = []
    previous_work = None
    for index, (truth, sample) in enumerate(zip(expected['samples'], actual['samples'])):
        theta = truth['theta']
        assert sample['theta'] == theta
        h, y = np.array(sample['h']), np.array(sample['y'])
        assert h.shape == y.shape == (n,)
        assert np.all(np.isfinite(h)) and np.all(np.isfinite(y))
        dh = float(np.linalg.norm(h - truth['h']))
        dy = float(np.linalg.norm(y - truth['y']))
        tail, roundoff = error_components(sample['error'])
        heat_tail, heat_roundoff = error_components(sample['heat_error'])
        # Eigh is a floating reference; this observed-scale allowance is
        # independent of the production solver's claimed analytic bound.
        oracle_allowance = 128 * np.finfo(float).eps * max(1, n) * max(1, theta)
        assert dh <= heat_tail + heat_roundoff + oracle_allowance, (request['case_id'], theta, 'h', dh)
        assert dy <= tail + roundoff + oracle_allowance, (request['case_id'], theta, 'y', dy)
        if truth['z_q'] is None:
            assert sample['z_q'] is None and sample['c_q'] is None, (request['case_id'], sample)
            assert sample['spectral_mean'] is None
            assert sample['z_q_interval_estimate'] is None and sample['c_q_interval_estimate'] is None
            assert sample['quality'] == 'no_seed', (request['case_id'], sample['quality'])
            zerr = cerr = None
        else:
            z, c = sample['z_q'], sample['c_q']
            assert z is not None and c is not None, (request['case_id'], sample)
            assert np.isfinite(z) and z > 0 and np.isfinite(c) and c >= 0
            seed = np.zeros(n)
            for seed_index, mass in request['seeds']:
                seed[seed_index] += mass
            seed /= seed.sum()
            normv2 = float(seed @ seed)
            normy = float(np.linalg.norm(y))
            delta = dy + oracle_allowance
            zbound = (2 * normy * delta + delta ** 2) / normv2
            # Normalized vector perturbation <= 2δ/||yhat||, ||A||<=2.
            # Rayleigh perturbation <=4ε and residual perturbation <=8ε.
            eta = 16 * delta / max(normy, np.finfo(float).tiny)
            rhat = np.sqrt(c) / theta if theta else 0.
            cbound = theta ** 2 * (2 * rhat * eta + eta ** 2)
            zerr, cerr = abs(z - truth['z_q']), abs(c - truth['c_q'])
            assert zerr <= zbound + 2e-13, (request['case_id'], theta, zerr, zbound)
            assert cerr <= cbound + 2e-13, (request['case_id'], theta, cerr, cbound)
            validate_interval(sample['z_q_interval_estimate'], z, truth['z_q'], zbound + 2e-13, 'Z')
            validate_interval(sample['c_q_interval_estimate'], c, truth['c_q'], cbound + 2e-13, 'C')
            mean = sample['spectral_mean']
            assert np.isfinite(mean) and -eta <= mean <= 2 + eta
            assert abs(mean - truth['mu']) <= eta + 2e-13
            if not expected['informative']:
                assert sample['quality'] == 'no_dynamics', (request['case_id'], sample['quality'])
            elif theta == 0 or c < np.finfo(float).tiny or sample['c_q_interval_estimate']['lower'] == 0:
                assert sample['quality'] == 'numerically_indeterminate', (request['case_id'], theta, sample['quality'])
            else:
                assert sample['quality'] == 'estimated', (request['case_id'], theta, sample['quality'])
                assert normy > tail + roundoff, ('unresolved denominator claims estimated', request['case_id'], theta)
            if request['case_id'] == 'two-node':
                exp = np.exp(-2 * theta)
                analytic = 4 * theta ** 2 * exp / (1 + exp) ** 2
                assert abs(c - analytic) <= cbound + 2e-13, (theta, c, analytic, cbound)
                if theta >= 64:
                    assert sample['quality'] == 'numerically_indeterminate'
        validate_work(sample['work'], previous_work)
        assert sample['work']['samples'] == index + 1
        previous_work = sample['work']
        if expected['dmax']:
            assert np.isclose(sample['physical_time'], theta / expected['dmax'], atol=0, rtol=2e-15)
        else:
            assert sample['physical_time'] is None
        case_checks.append(dict(theta=theta, h_l2_error=dh, y_l2_error=dy,
                                z_abs_error=zerr, c_abs_error=cerr,
                                analytic_truncation_l2=tail, empirical_roundoff_l2=roundoff,
                                heat_analytic_truncation_l2=heat_tail, heat_empirical_roundoff_l2=heat_roundoff,
                                oracle_empirical_allowance=oracle_allowance,
                                quality=sample['quality'], work=sample['work']))
    validate_work(actual['work'], previous_work)
    assert actual['work']['samples'] == len(expected['samples'])
    return dict(case_id=request['case_id'], status=actual['status'], work=actual['work'], checks=case_checks)


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--probe', type=Path, required=True)
    p.add_argument('--output', type=Path, default=HERE / 'numerical-comparison.json')
    a = p.parse_args()
    reference = json.loads((HERE / 'numerical-reference.json').read_text())
    requests = [json.loads(line) for line in (HERE / 'numerical-requests.jsonl').read_text().splitlines()]
    begin = time.perf_counter()
    run = subprocess.run([str(a.probe.resolve())], input=''.join(json.dumps(x)+'\n' for x in requests),
                         text=True, capture_output=True, timeout=180)
    assert run.returncode == 0, (run.returncode, run.stderr)
    outputs = [json.loads(x) for x in run.stdout.splitlines()]
    assert len(outputs) == len(requests)
    checks = []
    for request, expected, actual in zip(requests, reference['outputs'], outputs):
        checks.append(compare_case(request, expected, actual))
    result = dict(status='passed', cases=len(checks), samples=sum(len(x['checks']) for x in checks),
                  wall_seconds=time.perf_counter() - begin,
                  probe_sha256=hashlib.sha256(a.probe.read_bytes()).hexdigest(),
                  comparator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                  reference_sha256=hashlib.sha256((HERE / 'numerical-reference.json').read_bytes()).hexdigest(),
                  requests_sha256=hashlib.sha256((HERE / 'numerical-requests.jsonl').read_bytes()).hexdigest(),
                  uncertainty='Analytic truncation and empirical roundoff are recorded separately; oracle allowance is empirical, not a certificate.',
                  checks=checks, raw_probe_outputs=outputs)
    a.output.write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(dict(status='passed', cases=result['cases'], samples=result['samples'], wall_seconds=result['wall_seconds'])))


if __name__ == '__main__':
    main()
