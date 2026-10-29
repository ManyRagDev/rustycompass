#!/usr/bin/env python3
"""Measure delivered support, windows and retrospective scale opportunity."""
import argparse
from collections import defaultdict
import json
from pathlib import Path
import statistics
import sys

sys.dont_write_bytecode = True
from acquisition_baseline import HERE, decode, digest, support


def summarize(path):
    report = json.loads(path.read_text())
    controls_path = HERE / 'controls' / f"{report['split']}.json"
    assert digest(controls_path.read_bytes()) == report['controls_sha256']
    controls = json.loads(controls_path.read_text())
    lookup = {case['id']:case for case in controls['cases']}
    sources = {p.name:p.read_bytes() for p in (HERE/'fixtures'/report['split']).iterdir() if p.is_file()}
    groups = defaultdict(list)
    for row in report['results']:
        groups[(row['budget_bytes'], row['strategy'], row.get('fixed_theta'))].append(row)
    aggregates = []
    for (budget,strategy,theta), rows in groups.items():
        answerable = [row for row in rows if row['answerable']]
        required = [row for row in rows if row['required_units']]
        windows = [row for row in rows if row.get('acquisition',{}).get('reason') == 'spectral_window']
        without_support = [row for row in windows if not row['answerable'] or row['support_complete'] is not True]
        latencies = [row['wall_ns']/1e6 for row in rows if row.get('wall_ns') is not None]
        numeric_ops = sum((row.get('work') or {}).get('numeric',{}).get('operator_applications',0) for row in rows)
        aggregates.append(dict(budget_bytes=budget, strategy=strategy, fixed_theta=theta,
                               cases=len(rows), errors=sum(row['status']=='error' for row in rows),
                               evidence_cases=len(required), complete_evidence_cases=sum(row['support_complete'] is True for row in required),
                               delivered_units=sum(row['delivered_units'] for row in rows), required_units=sum(row['required_units'] for row in rows),
                               answerable_cases=len(answerable), answerable_with_full_support=sum(row['support_complete'] is True for row in answerable),
                               accepted_windows=len(windows), windows_without_answer_support=len(without_support),
                               acceptance_rate=len(windows)/len(rows),
                               missing_support_among_accepted=len(without_support)/len(windows) if windows else None,
                               serialized_payload_bytes_total=sum(row.get('payload_bytes',0) for row in rows),
                               source_bytes_total=sum(row.get('source_bytes',0) for row in rows),
                               acquired_scales_total=sum((row.get('work') or {}).get('scales',0) for row in rows),
                               numeric_operator_applications_total=numeric_ops,
                               per_request_wall_ms_median=statistics.median(latencies) if latencies else None,
                               per_request_wall_ms_max=max(latencies) if latencies else None))
    retrospective = []
    for row in report['results']:
        if row['strategy'] != 'spectral_window_v1':continue
        case = lookup[row['case_id']]
        candidates = []
        for step in row.get('trajectory') or []:
            intervals = defaultdict(list)
            for path_key,a,b in step['evidence_intervals']:
                source_path = decode(path_key)
                assert 0 <= a <= b <= len(sources[source_path])
                intervals[source_path].append((a,b))
            observed = support(case, sources, intervals)
            candidates.append(dict(theta=step['theta'], support_complete=observed['support_complete'],
                                   delivered_units=observed['delivered_units'], window_eligible=step['window_eligible'],
                                   gaps=step['gaps']))
        fixed = [other for other in report['results'] if other['case_id'] == row['case_id']
                 and other['budget_bytes'] == row['budget_bytes'] and other['strategy'] == 'fixed_heat']
        good_fixed = [other['fixed_theta'] for other in fixed if other['support_complete'] is True]
        content = {json.dumps(other.get('delivered_intervals'),sort_keys=True) for other in fixed if other['status']=='pack'}
        retrospective.append(dict(case_id=row['case_id'], budget_bytes=row['budget_bytes'],
                                  answerable=case['answerable'], final_support_complete=row['support_complete'],
                                  final_reason=row.get('acquisition',{}).get('reason'),
                                  selected_theta=row.get('diagnostics',{}).get('selected_theta'),
                                  max_evaluated_theta=row.get('diagnostics',{}).get('max_evaluated_theta'),
                                  any_trajectory_candidate_has_support=any(x['support_complete'] is True for x in candidates),
                                  first_supported_trajectory_theta=next((x['theta'] for x in candidates if x['support_complete'] is True),None),
                                  any_eligible_candidate_has_support=any(x['support_complete'] is True and x['window_eligible'] for x in candidates),
                                  fixed_scales_with_full_support=good_fixed,
                                  fixed_scale_changes_delivered_intervals=len(content)>1,
                                  candidates=candidates))
    return dict(split=report['split'], input_sha256=digest(path.read_bytes()),
                binary_sha256=report['binary_sha256'], probe_sha256=report['probe_sha256'],
                aggregates=aggregates, retrospective=retrospective,
                costs=dict(acquisitions=report['acquisition_calls'], laboratory_adapter_calls=report['laboratory_adapter_calls'],
                           public_cli_or_mcp_calls=report['public_cli_or_mcp_calls'], instrumentation_prepare_calls=report['instrumentation_calls'],
                           instrumentation_prepare_wall_ns=report['instrumentation_prepare_wall_ns'],
                           probe_wall_ns_total=report['wall_ns_total'], peak_rss_kib_whole_probe=report['peak_rss_kib_total_process']),
                interpretation='Full support of required intervals is distinct from answerability and window eligibility. Retrospective fixed-scale calls use evaluator labels only after each acquisition; they are extra instrumented acquisitions, not an adaptive selection rule. Errors remain in every applicable denominator; no production promotion threshold is claimed.')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--input', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    result = summarize(args.input)
    args.output.write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(dict(split=result['split'], groups=len(result['aggregates']),
                          retrospective_cases=len(result['retrospective']), costs=result['costs'])))


if __name__ == '__main__':main()
