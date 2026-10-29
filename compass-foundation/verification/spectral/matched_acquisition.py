#!/usr/bin/env python3
"""Compare strategies through the SAME production materializer probe.

The adapter is supplied by acquisition engineering; this file only builds its
source collections, issues requests and measures verified delivered support.
Default development-only. Reserved use requires a previously frozen profile.
"""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time

sys.dont_write_bytecode = True
from acquisition_baseline import HERE, ROOT, digest, support, verify_pack


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, default=ROOT / 'target/release/compass')
    parser.add_argument('--probe', type=Path, required=True)
    parser.add_argument('--split', choices=['dev', 'reserved'], default='dev')
    parser.add_argument('--fixed-theta', type=float, default=1.)
    parser.add_argument('--fixed-theta-grid', type=float, nargs='+')
    parser.add_argument('--strategies', nargs='+', choices=['none', 'bfs', 'fixed_heat', 'spectral_window_v1'],
                        default=['none', 'bfs', 'fixed_heat', 'spectral_window_v1'])
    parser.add_argument('--frozen-profile', type=Path)
    parser.add_argument('--output', type=Path, default=HERE / 'matched-dev.json')
    args = parser.parse_args()
    binary_hash, probe_hash = digest(args.binary.read_bytes()), digest(args.probe.read_bytes())
    if args.split == 'reserved':
        assert args.frozen_profile is not None, 'reserved measurement requires the explicitly frozen profile'
        frozen_profile = json.loads(args.frozen_profile.read_text())
        for path, expected_hash in frozen_profile['source_sha256'].items():
            assert digest((ROOT / path).read_bytes()) == expected_hash, ('profile source changed', path)
        assert frozen_profile['probe_sha256'] == probe_hash and frozen_profile['binary_sha256'] == binary_hash
        assert frozen_profile['frozen_cases_sha256'] == digest((HERE/'frozen-cases.json').read_bytes())
    fixed_scales = args.fixed_theta_grid or (frozen_profile['fixed_theta_grid'] if args.split == 'reserved' else [args.fixed_theta])
    if args.split == 'reserved':assert fixed_scales == frozen_profile['fixed_theta_grid']
    assert all(0 <= x <= 256 for x in fixed_scales) and len(set(fixed_scales)) == len(fixed_scales)
    control_file = HERE / 'controls' / f'{args.split}.json'
    controls = json.loads(control_file.read_text())
    frozen = json.loads((HERE / 'frozen-cases.json').read_text())['splits'][args.split]
    assert digest(control_file.read_bytes()) == frozen['control_sha256']
    source_dir = HERE / 'fixtures' / args.split
    sources = {p.name:p.read_bytes() for p in source_dir.iterdir() if p.is_file()}
    assert {p:digest(v) for p,v in sources.items()} == frozen['file_sha256']
    requests, results = [], []
    with tempfile.TemporaryDirectory(prefix='compass-matched-') as temporary:
        tmp = Path(temporary)
        tree, store = tmp / 'source', tmp / 'store'
        shutil.copytree(source_dir, tree)
        prepare_begin = time.perf_counter_ns()
        prepared = subprocess.run([str(args.binary.resolve()), 'collection', 'prepare', '--root', str(tree),
                                   '--store', str(store), '--json'], capture_output=True, check=True, timeout=120)
        prepare_wall_ns = time.perf_counter_ns() - prepare_begin
        generation = json.loads(prepared.stdout)['generation']
        for case in controls['cases']:
            for budget in controls['budgets']:
                for strategy in args.strategies:
                    for theta in (fixed_scales if strategy == 'fixed_heat' else [None]):
                        suffix = f':{theta}' if theta is not None else ''
                        request = dict(case_id=f"{case['id']}:{budget}:{strategy}{suffix}", store=str(store),
                                       generation=generation, query=case['query'], scope=case['scope'],
                                       budget_bytes=budget, strategy=strategy)
                        if theta is not None:
                            request['theta'] = theta
                        requests.append(request)
        with tempfile.NamedTemporaryFile() as metric:
            begin = time.perf_counter_ns()
            response = subprocess.run(['/usr/bin/time', '-f', '%M', '-o', metric.name, str(args.probe.resolve())], text=True,
                                      input=''.join(json.dumps(x)+'\n' for x in requests),
                                      capture_output=True, timeout=180)
            wall_ns = time.perf_counter_ns() - begin
            peak_rss_kib = int(Path(metric.name).read_text().strip().splitlines()[-1])
        assert response.returncode == 0, response.stderr
        outputs = [json.loads(x) for x in response.stdout.splitlines()]
        assert len(outputs) == len(requests)
        lookup = {x['id']:x for x in controls['cases']}
        for request, output in zip(requests, outputs):
            assert output['case_id'] == request['case_id']
            case_id = request['case_id'].split(':', 1)[0]
            case = lookup[case_id]
            result = dict(case_id=case_id, strategy=request['strategy'], scope=request['scope'],
                          budget_bytes=request['budget_bytes'], fixed_theta=request.get('theta'),
                          acquisition_calls=1, laboratory_adapter_calls=1, public_cli_or_mcp_calls=0,
                          work=output.get('work'), wall_ns=output.get('wall_ns'))
            if 'error' in output:
                result.update(status='error', error=output['error'], support_complete=False if case['required'] else None,
                              answerable=case['answerable'],
                              required_units=len(case['required']), delivered_units=0)
            else:
                pack = output['pack']
                # The probe returns the exact canonical public pack bytes as
                # JSON text; Python re-serialization cannot certify byte size.
                raw = output['pack_json'].encode()
                assert json.loads(raw) == pack
                intervals, metrics = verify_pack(pack, sources, generation, raw)
                delivered_intervals = sorted([path, a, b] for path,pairs in intervals.items() for a,b in pairs)
                result.update(status='pack', **metrics, **support(case, sources, intervals),
                              delivered_intervals=delivered_intervals,
                              structural=pack.get('structural'), acquisition=pack.get('acquisition'),
                              coverage=pack.get('coverage'), trajectory=output.get('trajectory'),
                              diagnostics=pack.get('diagnostics'))
                assert pack['query'] == case['query'] and pack['scope']['kind'] == case['scope']
                if request['strategy'] == 'spectral_window_v1':
                    assert pack['schema_version'] == '3'
                    assert pack['next_cursor'] is None
                    assert pack['acquisition']['state'] == 'terminal'
                    assert pack.get('completion', {}).get('status') != 'SUFFICIENT_EVIDENCE_ACQUIRED'
            results.append(result)
    assert digest(args.binary.read_bytes()) == binary_hash and digest(args.probe.read_bytes()) == probe_hash, 'binary changed during measurement'
    for case in controls['cases']:
        identities = {(x['diagnostics'].get('seed_sha256'), x['diagnostics'].get('projection_sha256'))
                      for x in results if x['case_id'] == case['id'] and x['status'] == 'pack'}
        assert len(identities) <= 1, ('matched arms changed seed/projection', case['id'], identities)
    report = dict(split=args.split, generation=generation,
                  probe_sha256=probe_hash, binary_sha256=binary_hash,
                  runner_sha256=digest(Path(__file__).read_bytes()),
                  controls_sha256=digest(control_file.read_bytes()), fixed_theta_grid=fixed_scales,
                  wall_ns_total=wall_ns, peak_rss_kib_total_process=peak_rss_kib,
                  response_utf8_bytes=len(response.stdout.encode()), instrumentation_calls=1,
                  instrumentation_prepare_wall_ns=prepare_wall_ns,
                  acquisition_calls=len(requests), laboratory_adapter_calls=len(requests), public_cli_or_mcp_calls=0,
                  profile_freeze_sha256=digest(args.frozen_profile.read_bytes()) if args.frozen_profile else None,
                  results=results,
                  conditions='One prepared generation, identical seed/projection identities checked across arms and budgets. Shared closure/materializer supplied by production. One lab adapter request produces one pack; no public CLI/MCP acquisition is claimed. Per-request wall_ns includes collection open and acquisition; process total time/RSS also include JSON adapter overhead. Core work is counted separately. Collection preparation is one explicit instrumentation call. Retrospective fixed scales are extra acquisitions and never an adaptive single-call cost.')
    args.output.write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(dict(status='measured', split=args.split, requests=len(results), output=str(args.output))))


if __name__ == '__main__':
    main()
