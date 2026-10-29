#!/usr/bin/env python3
"""Aggregate the frozen matched experiment without tuning after reserved use."""
import json
from pathlib import Path
import statistics
import sys

sys.dont_write_bytecode = True
from acquisition_baseline import HERE, ROOT, digest


def main():
    freeze_path = HERE / 'profile-frozen.json'
    freeze = json.loads(freeze_path.read_text())
    for path, expected in freeze['source_sha256'].items():
        assert digest((ROOT/path).read_bytes()) == expected, ('source changed after freeze',path)
    assert digest((HERE/'frozen-cases.json').read_bytes()) == freeze['frozen_cases_sha256']
    paths = [HERE/'matched-dev-final.json', HERE/'matched-reserved-final.json']
    reports = [json.loads(path.read_text()) for path in paths]
    assert all(report['binary_sha256'] == freeze['binary_sha256'] and report['probe_sha256'] == freeze['probe_sha256'] for report in reports)
    assert reports[1]['profile_freeze_sha256'] == digest(freeze_path.read_bytes())
    assert all(report['fixed_theta_grid'] == freeze['fixed_theta_grid'] for report in reports)
    assert all(json.loads((HERE/'controls'/f"{report['split']}.json").read_text())['budgets'] == freeze['budgets'] for report in reports)
    preserved = dict(probe=ROOT/'.compass/spectral-evaluation-build/acquisition_probe',
                     binary=ROOT/'.compass/spectral-evaluation-build/compass')
    assert digest(preserved['probe'].read_bytes()) == freeze['probe_sha256']
    assert digest(preserved['binary'].read_bytes()) == freeze['binary_sha256']
    rows = [row for report in reports for row in report['results']]
    groups = []
    for budget in freeze['budgets']:
        for strategy,theta in [('none',None), ('bfs',None), *[('fixed_heat',t) for t in freeze['fixed_theta_grid']], ('spectral_window_v1',None)]:
            selected = [row for row in rows if row['budget_bytes']==budget and row['strategy']==strategy and row['fixed_theta']==theta]
            assert len(selected) == 14
            windows = [row for row in selected if row.get('acquisition',{}).get('reason')=='spectral_window']
            unsupported = [row for row in windows if not row['answerable'] or row['support_complete'] is not True]
            latencies = [row['wall_ns']/1e6 for row in selected if row['wall_ns'] is not None]
            known_work = [row['work'] for row in selected if row['work'] is not None]
            groups.append(dict(budget_bytes=budget,strategy=strategy,fixed_theta=theta,
                               acquisitions=len(selected),errors=sum(row['status']=='error' for row in selected),
                               delivered_units=sum(row['delivered_units'] for row in selected),required_units=sum(row['required_units'] for row in selected),
                               answerable_cases=sum(row['answerable'] for row in selected),
                               answerable_with_full_support=sum(row['answerable'] and row['support_complete'] is True for row in selected),
                               accepted_windows=len(windows),windows_without_answer_support=len(unsupported),
                               accepted_window_rate=len(windows)/len(selected),
                               windows_without_support_rate=len(unsupported)/len(windows) if windows else None,
                               serialized_payload_bytes_total=sum(row.get('payload_bytes',0) for row in selected),
                               per_acquisition_wall_ms_median=statistics.median(latencies) if latencies else None,
                               per_acquisition_wall_ms_total=sum(latencies) if latencies else None,
                               per_acquisition_timing_known=len(latencies),
                               core_work_known=len(known_work),
                               core_elapsed_ms_total=sum(w['elapsed_ms'] for w in known_work) if known_work else None,
                               source_revalidation_bytes_total=sum(w['source_revalidation_bytes'] for w in known_work) if known_work else None,
                               total_verified_document_reads=sum(w['total_verified_document_reads'] for w in known_work) if known_work else None,
                               scales_total=sum(w['scales'] for w in known_work) if known_work else None,
                               numeric_operator_applications_total=sum(w['numeric']['operator_applications'] for w in known_work) if known_work else None))
    fixed = next(x for x in groups if x['budget_bytes']==16384 and x['strategy']=='fixed_heat' and x['fixed_theta']==1.)
    adaptive = next(x for x in groups if x['budget_bytes']==16384 and x['strategy']=='spectral_window_v1')
    report = dict(status='experimental_only', profile=freeze['profile'],profile_freeze_sha256=digest(freeze_path.read_bytes()),
                  binary_sha256=freeze['binary_sha256'],probe_sha256=freeze['probe_sha256'],
                  preserved_evaluated_build={k:str(p.relative_to(ROOT)) for k,p in preserved.items()},
                  build_attribution='Measurements belong to the exact frozen CLI/probe hashes preserved under .compass/spectral-evaluation-build. A later all-examples build relinked target/debug/examples/acquisition_probe to a different hash without source/profile changes. No result is attributed to that relink or to a release build.',
                  source_identity_unchanged_since_freeze=True, reserved_opened_after_freeze=True,
                  artifacts_sha256={path.name:digest(path.read_bytes()) for path in paths},
                  groups=groups,
                  comparison_at_16kib=dict(fixed_theta_1_answerable_supported=fixed['answerable_with_full_support'],
                                          adaptive_answerable_supported=adaptive['answerable_with_full_support'],
                                          answerable_cases=adaptive['answerable_cases'],
                                          adaptive_over_fixed_total_wall_ratio=adaptive['per_acquisition_wall_ms_total']/fixed['per_acquisition_wall_ms_total'],
                                          adaptive_over_fixed_median_wall_ratio=adaptive['per_acquisition_wall_ms_median']/fixed['per_acquisition_wall_ms_median'],
                                          adaptive_minus_fixed_serialized_payload_bytes=adaptive['serialized_payload_bytes_total']-fixed['serialized_payload_bytes_total']),
                  cost_accounting=dict(total_matched_acquisitions=sum(r['acquisition_calls'] for r in reports),
                                       primary_none_bfs_fixed1_adaptive_acquisitions=112,
                                       extra_retrospective_fixed_scale_acquisitions=112,
                                       laboratory_adapter_calls=sum(r['laboratory_adapter_calls'] for r in reports),
                                       public_cli_or_mcp_acquisition_calls=0,
                                       collection_preparation_instrumentation_calls=sum(r['instrumentation_calls'] for r in reports),
                                       probe_wall_seconds_total=sum(r['wall_ns_total'] for r in reports)/1e9,
                                       preparation_wall_seconds_total=sum(r['instrumentation_prepare_wall_ns'] for r in reports)/1e9,
                                       whole_process_peak_rss_kib=[r['peak_rss_kib_total_process'] for r in reports],
                                       error_costs='Every 3-KiB request explicitly failed metadata fit. Per-request wall/core work are not returned on errors and remain unknown; whole-process time/RSS include all requests.'),
                  measurement_conditions='Sequential laboratory requests in one adapter process per split, with none/BFS/fixed-grid/adaptive order. Shared host and uncontrolled OS page cache. Request wall time includes collection open/preflight/acquisition, while core counters describe their own stages. Peak RSS belongs to the whole process and cannot be attributed to an individual arm. Two synthetic collections do not support general latency or efficacy claims.',
                  findings=['At 16 KiB every dev arm delivers all required support. The window rule adds no dev coverage.',
                            'In reserved controls adaptive and fixed theta=256 deliver one additional required unit over theta=1; BFS also does. The affected adaptive request ends no_window, so that gain is due to the larger-scale fallback, not acceptance of a spectral window.',
                            'The two accepted windows (one in each split) have the required support already at the first evaluated scale. None lacks required support, but this sample does not establish general reliability.',
                            'Five answerable cases per split are distinct from a dynamic/external case and a missing-symbol control. Delivering the dynamic function body does not make its external behavior answerable.',
                            'Both budgets and all errors remain in denominators. The 3-KiB budget cannot fit current metadata for any arm.',
                            'Matched arms use the same materializer, scope, seeds and projection identities. Legacy-retriever baseline results are separate and do not isolate scale selection.',
                            'The acquisition preflight revalidates/reparses the entire generation and is limited to 128 documents and 22,369,621 source bytes. Above the limit it returns an empty terminal structural_work_limit pack. Numerical solver benchmarks at 16,384 nodes do not demonstrate large-collection acquisition performance.',
                            'No production promotion threshold, paid LLM trial, token-economy claim or general superiority is established. Frozen policy and gabaritos were not adjusted after reserved measurement.'],
                  promotion=dict(status='experimental_only', quantitative_threshold_agreed=False))
    output = HERE/'evaluation-summary.json'
    output.write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(dict(status=report['status'],comparison_at_16kib=report['comparison_at_16kib'],costs=report['cost_accounting'])))


if __name__ == '__main__':main()
