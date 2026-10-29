#!/usr/bin/env python3
"""Snapshot an explicitly reviewed experimental profile before reserved use.

This command only freezes identity and evaluation conditions. It does not run
reserved cases, certify promotion, or authorize a production deployment.
"""
import argparse
import datetime
import json
from pathlib import Path
import sys

sys.dont_write_bytecode = True
from acquisition_baseline import HERE, ROOT, digest


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--dev-results', type=Path, required=True)
    parser.add_argument('--probe', type=Path, required=True)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--review-reference', required=True)
    parser.add_argument('--output', type=Path, default=HERE / 'profile-frozen.json')
    args = parser.parse_args()
    assert not args.output.exists(), 'freeze already exists; preserve it and choose a new filename for a new profile epoch'
    results = json.loads(args.dev_results.read_text())
    assert results['split'] == 'dev'
    assert results['probe_sha256'] == digest(args.probe.read_bytes())
    assert results['binary_sha256'] == digest(args.binary.read_bytes())
    diagnostics = [x['diagnostics'] for x in results['results'] if x['status']=='pack']
    assert diagnostics
    profiles = {(d['profile'],json.dumps(d['profile_parameters'],sort_keys=True),d['seed_policy'],d['seed_limit'],d['projection']) for d in diagnostics}
    assert len(profiles) == 1, 'development report mixes profiles'
    profile,parameters,seed_policy,seed_limit,projection = profiles.pop()
    shared_limits = [{k:v for k,v in d['limits'].items() if k != 'numeric'} for d in diagnostics]
    assert all(item == shared_limits[0] for item in shared_limits)
    files = sorted((ROOT/'crates/compass-core/src').rglob('*.rs'))
    files += [ROOT/'crates/compass-core/examples/acquisition_probe.rs', ROOT/'Cargo.lock']
    control = json.loads((HERE/'controls/dev.json').read_text())
    record = dict(schema_version=1, frozen_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
                  review_reference=args.review_reference, profile=profile,
                  parameters=json.loads(parameters), seed_policy=seed_policy, seed_limit=seed_limit, projection=projection,
                  acquisition_limits=shared_limits[0],
                  generation_revalidation=dict(universe='complete_generation',
                                              maximum_documents=shared_limits[0]['cache_documents'],
                                              maximum_source_bytes=shared_limits[0]['generation_revalidation_bytes'],
                                              over_limit_reason='structural_work_limit',
                                              over_limit_behavior='empty terminal pack; structural not_revalidated'),
                  source_sha256={str(p.relative_to(ROOT)):digest(p.read_bytes()) for p in files},
                  probe_sha256=results['probe_sha256'], binary_sha256=results['binary_sha256'],
                  dev_results_sha256=digest(args.dev_results.read_bytes()),
                  frozen_cases_sha256=digest((HERE/'frozen-cases.json').read_bytes()),
                  budgets=control['budgets'], fixed_theta_grid=results['fixed_theta_grid'],
                  query_classes=['local_definition', 'same_seed_different_scope', 'cross_module_static',
                                 'negative_absence', 'dynamic_or_external', 'coherent_unit_budget'],
                  measurement_rules=dict(errors_in_denominators=True, alternatives_require_delivered_verified_intervals=True,
                                         window_without_support_includes_unanswerable=True,
                                         retrospective_scale_calls_are_extra_acquisitions=True,
                                         distinguish_core_work_request_wall_and_process_rss=True),
                  evaluation_goals=['Measure support delivery, omissions and accepted windows without required support.',
                                    'Measure termination with gaps and explicit numerical/work limits.',
                                    'Compare fixed/adaptive with identical generation, seeds, scope, budget and materializer.'],
                  applicability_limit='Numerical solver benchmarks up to 16,384 nodes do not demonstrate acquisition for large collections. Acquisition reparses the complete generation under a 128-document/about-21.3-MiB preflight limit.',
                  promotion=dict(status='experimental_only', production_threshold_agreed=False,
                                 meaning='No quantitative promotion target was agreed. Reserved results assess this frozen profile; no tuning after opening them may claim reserved generalization.'))
    args.output.write_text(json.dumps(record,indent=2)+'\n')
    print(json.dumps(dict(status='frozen',profile=profile,source_files=len(files),output=str(args.output))))


if __name__ == '__main__':main()
