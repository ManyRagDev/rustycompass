#!/usr/bin/env python3
"""D0 independent laboratory: offline self-test or a stable public CLI binary."""
import sys
sys.dont_write_bytecode = True
import argparse
import copy
import itertools
import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time
import traceback
import cases
import oracle
import validate as v

HERE=Path(__file__).resolve().parent


class Checks:
    def __init__(self):
        self.passed=[]
        self.failed=[]
    def run(self,name,fn,ids=()):
        try:
            detail=fn()
            self.passed.append(dict(test=name,ids=list(ids),detail=detail))
        except Exception as error:
            self.failed.append(dict(test=name,ids=list(ids),type=type(error).__name__,error=str(error),traceback=traceback.format_exc()))


def rejects(fn,code):
    try: fn()
    except v.Rejected as e:
        v.equal(e.code,code,'wrong_rejection')
        return code
    raise AssertionError('accepted negative '+code)


def mathematical():
    total=0
    aggregate=0
    for entry in v.load(v.FIX/'cases.json'):
        results=[]
        for c in entry['calculations']:
            result=oracle.calculation(c['baseline_lines'],c['current_lines'])
            for k,value in c['expected'].items(): v.equal(result[k],value,'literal_math')
            if max(len(c['baseline_lines']),len(c['current_lines']))<=12:
                v.equal(int(result['l']),oracle.exhaustive_lcs(c['baseline_lines'],c['current_lines']),'two_methods')
            results.append(result)
            total+=1
        exp=entry['expected']
        if results and 'C' in exp:
            c=sum(int(x['c']) for x in results); m=sum(int(x['m']) for x in results)
            v.equal(str(c),exp['C'],'literal_aggregate_c')
            v.equal(str(m),exp['M'],'literal_aggregate_m')
            v.equal(oracle.ratio(c,m)['decimal'],exp['decimal'],'literal_rounding')
            aggregate+=1
        if 'individual_ratios' in exp:
            for x,pair in zip(results,exp['individual_ratios']):
                v.equal([x['delta']['numerator'],x['delta']['denominator']],pair,'literal_individual')
    for c,m,decimal in next(e for e in v.load(v.FIX/'cases.json') if e['id']=='E49')['expected']['ratios']:
        v.equal(oracle.ratio(int(c),int(m))['decimal'],decimal,'rounding_tie')
    alphabet=['a','b']
    sequences=[list(p) for n in range(5) for p in itertools.product(alphabet,repeat=n)]
    pairs=0
    for x in sequences:
        for y in sequences:
            n=oracle.lcs(x,y)
            v.equal(n,oracle.exhaustive_lcs(x,y),'exhaustive_crosscheck')
            v.equal(n,oracle.lcs(y,x),'symmetry')
            v.need(0<=n<=min(len(x),len(y)),'lcs_bound')
            pairs+=1
    return dict(literal_calculations=total,literal_aggregates=aggregate,small_pairs=pairs)


def transport():
    for raw,code in [(b'{"x":1,"x":2}','json_duplicate'),(b'\xef\xbb\xbf{}','json_bom'),
                     (b'{} {}','json_transport'),(b'"\xff"','json_transport'),(b'NaN','json_number'),
                     (b'Infinity','json_number'),(b'-Infinity','json_number'),(b'1e999','json_number')]:
        rejects(lambda:v.strict(raw),code)
    # Independent fixed UTF-16 order/control escaping example; U+10000 precedes U+E000.
    v.equal(v.canonical({'\ue000':'\b\t\n\f\r\x00','\U00010000':'é','a':'"\\/'}),
            '{"a":"\\\"\\\\/","𐀀":"é","":"\\b\\t\\n\\f\\r\\u0000"}'.encode(),'canonical_vector')
    rejects(lambda:v.canonical({'number':1}), 'canonical_domain')
    for path in ('Li4','L2E','YQ==','YR','YQAvYg'):
        rejects(lambda:v.path_bytes(path),'path')
    v.equal(v.uint('18446744073709551615'),2**64-1,'u64_max')
    rejects(lambda:v.uint('18446744073709551616'),'u64')
    return dict(strict_negatives=8,path_negatives=5,canonical='string/null/bool subset; UTF-16 keys')


def vectors():
    count=0
    for vector in v.load(v.FIX/'byte-vectors.json'):
        for side in v.SIDES:
            raw=bytes.fromhex(vector[side+'_hex'])
            v.equal(v.sha(raw),vector[side+'_sha256'],'raw_hash')
            v.equal(oracle.lines(raw),vector[side+'_lines'],'normalized_lines')
            count+=1
    h=v.load(v.FIX/'hash-vectors.json')
    for e in h['sha256_vectors']:
        raw=bytes.fromhex(e['hex'])
        v.equal(v.sha(raw),e['sha256'],'hash_vector')
        v.equal(str(len(raw)),e['size'],'size_vector')
    policy=v.load(v.FIX/'metric-policy.json')
    v.equal(v.canonical(policy),(v.FIX/'metric-policy.jcs').read_bytes(),'policy_canonical')
    v.equal(v.sha(v.canonical(policy)),h['metric_policy_jcs_sha256'],'policy_jcs_hash')
    v.equal((b'compass-d0-policy-v1\0').hex(),h['metric_policy_prefix_hex'],'policy_prefix')
    for n in ('report','source','metric-policy','limits','error'): v.validator(n)
    for p in (v.FIX/'errors').glob('*.json'): v.schema(v.load(p),'error')
    return dict(byte_sides=count,sha_vectors=len(h['sha256_vectors']),schemas=5,error_envelopes=10)


def integrity_negative():
    with tempfile.TemporaryDirectory(prefix='d0-integrity-') as temp:
        import shutil
        base=Path(temp)/'d0-c2'
        shutil.copytree(v.CONTRACT,base)
        # SHA256SUMS also references the sibling C2 review.
        shutil.copyfile(v.CONTRACT.parent/'D0_C2_REVISAO.md',Path(temp)/'D0_C2_REVISAO.md')
        p=base/'fixtures/cases.json'; p.write_bytes(p.read_bytes()+b' ')
        return rejects(lambda:v.integrity(base),'artifact_hash')


def frozen_reports():
    names=[]
    for name,c in v.load(v.FIX/'report-inputs.json').items():
        value=v.load(v.FIX/c['expected_report'])
        paths=[v.FIX/c[s+'_manifest'] for s in v.SIDES]
        packages=[v.FIX/c[s+'_package'] if c[s+'_package'] else None for s in v.SIDES]
        v.report(value,paths,packages,exit_code=c['expected_exit'])
        v.equal(v.canonical(value),(v.FIX/c['expected_report']).with_suffix('.jcs').read_bytes(),'frozen_jcs')
        names.append(name)
    return names


def frontier_vectors():
    f=v.load(v.FIX/'resource-frontiers.json')
    for x in f['file_bytes']:
        allowed=int(x['declared'])<=4194304
        v.equal(allowed,x['file_preflight']=='admitted','file_frontier')
        if allowed:v.equal(str(int(x['declared'])+1),x['object_reservation'],'file_reservation')
    for x in f['lines']:
        v.equal(max(int(x['n0']),int(x['nt']))<=100000,x['line_preflight']=='admitted','line_frontier')
    for x in f['cells_per_unit']:
        n,t=int(x['n0']),int(x['nt'])
        a=['a']*n; b=['a' if x['normalized_sequences_exactly_equal'] else 'b']*t
        v.equal(str(oracle.cell_cost(a,b)),x['cells'],'cell_frontier')
    for x in f['cell_schedule']:
        remaining=int(x['max_cells_total'])
        for u in x['units']:
            a=['a']*int(u['n0']); b=['a' if u['equal'] else 'b']*int(u['nt'])
            cost=oracle.cell_cost(a,b)
            v.equal(str(cost),u['cells'],'cell_schedule'); remaining-=cost
        v.equal(str(remaining),x['remaining'],'cell_balance')
    for x in f['logical_memory']:
        v.equal(int(x['S'])+int(x['W'])<=int(x['budget']),x['preflight']=='admitted','memory_frontier')
    return {k:len(x) for k,x in f.items() if isinstance(x,list)}


def assert_e52_stimulus(c):
    """Assert frozen raw bytes before granting E52 any normalization coverage."""
    fixture=next(e for e in v.load(v.FIX/'cases.json') if e['id']=='E52')
    physical=[]
    normalized=[]
    for side,path,package in zip(v.SIDES,c['paths'],c['packages']):
        obj,manifest_raw=v.snapshot(path)
        v.equal(len(obj['entries']),1,'bug003_single_entry')
        entry=obj['entries'][0]
        data=(package/'objects'/entry['content_sha256']).read_bytes()
        frozen=bytes.fromhex(fixture['input'][side+'_hex'])
        v.equal(data,frozen,'bug003_stimulus_bytes')
        v.equal(entry['content_sha256'],v.sha(data),'bug003_revision_hash')
        v.equal(entry['size_bytes'],str(len(data)),'bug003_revision_size')
        descriptor,_=v.descriptor(package,obj,manifest_raw)
        v.equal(descriptor['snapshot_file_sha256'],v.sha(manifest_raw),'bug003_descriptor_binding')
        physical.append(dict(side=side,hex=data.hex(),sha256=v.sha(data),size_bytes=str(len(data)),
                             snapshot_file_sha256=descriptor['snapshot_file_sha256']))
        normalized.append(oracle.lines(data))
    v.need(physical[0]['sha256']!=physical[1]['sha256'],'bug003_distinct_raw_hashes')
    v.equal(normalized,[['a'],['a']],'bug003_equal_normalized_lines')
    expected=v.model(c['paths'],c['packages'],c['limits'])
    cases.assert_literal(expected,{'status':'complete','C':'0','M':'2','decimal':'0.000000','cells_reserved':'0'})
    v.equal(expected['units'][0]['classification'],'modified','bug003_foundation_modified')
    for check in expected['units'][0]['checks'].values():
        v.equal(check['state'],'verified','bug003_verified_bytes')
    return dict(assertion='bug003_stimulus_bytes',physical=physical,normalized_lines=normalized,
                foundation_classification='modified',value=expected['measurement']['value'],
                line_mass='2',cells_reserved='0')


def regression_bug003(c):
    observed=assert_e52_stimulus(c)
    fixture=next(e for e in v.load(v.FIX/'cases.json') if e['id']=='E52')
    calc=fixture['calculations'][0]
    with tempfile.TemporaryDirectory(prefix='d0-bug003-negative-') as temp:
        # Recreate the audited defect: discard the frozen raw bytes and rebuild
        # both revisions from already-normalized lists. Mathematics still passes.
        wrong=cases.make(Path(temp)/'normalized-only','wrong-e52',
                         {b'p':cases.bytes_for(calc['baseline_lines'])},
                         {b'p':cases.bytes_for(calc['current_lines'])})
        expected=v.model(wrong['paths'],wrong['packages'])
        cases.assert_literal(expected,{'status':'complete','C':'0','M':'2','cells_reserved':'0'})
        envelope=v.load(v.FIX/'reports/complete.json');envelope.update(expected)
        v.schema(envelope,'report');v.report(envelope,wrong['paths'],wrong['packages'])
        rejected=rejects(lambda:assert_e52_stimulus(wrong),'bug003_stimulus_bytes')
    return dict(bug='BUG-003',positive=observed,normalized_only_control_schema='PASS',
                normalized_only_control_value=expected['measurement']['value'],negative_rejection=rejected)


def prepared_case(c):
    physical=assert_e52_stimulus(c) if c['name']=='E52' else None
    if 'error' in c:
        code,exit_code=c['error']
        # Constructed errors are validated as frozen envelopes offline, not claimed
        # to have been emitted by a sensor. Their runtime assertions run in --binary.
        v.schema(v.load(v.FIX/'errors'/(code+'.json')),'error')
        return dict(state='PREPARADA',error=code,exit=exit_code)
    expected=v.model(c['paths'],c['packages'],c['limits'])
    cases.assert_literal(expected,c['literal'])
    envelope=v.load(v.FIX/'reports/complete.json')
    envelope.update(expected)
    v.schema(envelope,'report')
    additional=v.load(HERE/'fixtures.json')['additional_literals']
    if c['name']=='E24-native':
        cases.assert_literal(expected,{k:additional['native_metadata'][k] for k in ('C','M')})
        v.equal([v.path_bytes(u['path_key']).hex() for u in expected['units']],additional['native_metadata']['path_hex'],'additional_native_paths')
    if c['name'] in ('E44-shared','E44-distinct'):
        fixture='same_root_empty' if c['name']=='E44-shared' else 'distinct_roots_empty'
        v.equal(expected['resources']['object_bytes_reserved'],additional[fixture]['reserved'],'additional_sharing')
    if c['name']=='E30':
        check=expected['units'][0]['checks']['baseline']
        for k,value in additional['prefix_abc_abcd'].items(): v.equal(check[k],value,'additional_prefix')
    if 'frozen_report' in c:
        v.report(v.load(c['frozen_report']),c['paths'],c['packages'],exit_code=c['expected_exit'])
    result=dict(state='MODELO_OFFLINE_CONFERIDO',status=expected['measurement']['status'],resources=expected['resources'])
    if physical is not None: result['E52_physical_stimulus']=physical
    return result


def regression_bug001():
    """C01: native identifiers are not counters; actual numeric fields remain u64."""
    observed=[]
    with tempfile.TemporaryDirectory(prefix='d0-bug001-') as temp:
        for path_key in ('1111','1'*24):
            raw=v.path_bytes(path_key)
            v.equal(v.key(raw),path_key,'bug001_native_path')
            v.domain({'path_key':path_key,'producer_version':'1'*24,
                      'producer_build':'1'*24,'content_sha256':'1'*64,
                      'metric_policy':{'unknown':'blocks_integral'}})
            c=cases.make(Path(temp)/str(len(path_key)),'bug001',{raw:b'a\n'},{raw:b'a\n'})
            for p in c['paths']: v.snapshot(p)
            expected=v.model(c['paths'],c['packages'])
            cases.assert_literal(expected,{'status':'complete','C':'0','M':'2','decimal':'0.000000','cells_reserved':'0'})
            envelope=v.load(v.FIX/'reports/complete.json'); envelope.update(expected)
            envelope['producer_version']='1'*24
            v.report(envelope,c['paths'],c['packages'],exit_code=0)
            observed.append(dict(path_key=path_key,path_hex=raw.hex(),value=expected['measurement']['value'],cells_reserved=expected['resources']['cells_reserved']))
    # Inspect field roles in the existing schemas, never the lexical form of values.
    fields=set()
    def numeric(node):
        return isinstance(node,dict) and (
            node.get('type')=='string' and node.get('pattern') in
            (r'^(0|[1-9][0-9]*)$',r'^[1-9][0-9]*$') or
            any(numeric(x) for x in node.get('anyOf',[])))
    def walk(node):
        if isinstance(node,dict):
            for name,definition in node.get('properties',{}).items():
                if numeric(definition): fields.add(name)
            for child in node.values(): walk(child)
        elif isinstance(node,list):
            for child in node: walk(child)
    for p in [v.ROOT/'research/contracts/transport.schema.json',*(v.CONTRACT/'schemas').glob('*.json')]: walk(v.load(p))
    for name in fields:
        # The overloaded name 'unknown' has its numeric role in counts.
        v.domain({'counts':{name:str(2**64-1)}})
        rejects(lambda name=name:v.domain({'counts':{name:str(2**64)}}),'u64')
    return dict(bug='BUG-001',positive_paths=observed,u64_fields_checked=sorted(fields),overflow_rejection='u64')


def regression_bug002():
    """C02: static component rejection precedes final-path nonexistence."""
    evidence=[]
    with tempfile.TemporaryDirectory(prefix='d0-bug002-') as temp:
        for variant,reason in [('regular',None),('missing','object_missing'),
                               ('existing-link','source_invalid_kind'),
                               ('dangling-link','source_invalid_kind')]:
            c=cases.make(Path(temp)/variant,'bug002',{b'p':b'abc'},{b'p':b'abc'})
            objects=c['packages'][0]/'objects'
            if variant=='missing': cases.object_path(c).unlink()
            elif variant in ('existing-link','dangling-link'):
                kept=objects.with_name('kept'); objects.rename(kept)
                objects.symlink_to(kept if variant=='existing-link' else objects.with_name('absent-target'),target_is_directory=True)
            expected=v.model(c['paths'],c['packages'])
            unit=expected['units'][0]; check=unit['checks']['baseline']
            if reason:
                v.equal(unit['reasons'],[reason],'bug002_literal_reason')
                v.equal(check['reasons'],[reason],'bug002_literal_check_reason')
                v.equal(check['state'],'failed','bug002_failed_state')
                for field in ('actual_revision','hash_check','size_check'):
                    v.equal(check[field],None,'bug002_no_fabricated_revision')
                v.equal(expected['measurement']['status'],'unavailable','bug002_unavailable')
                v.need('calculation' not in unit,'bug002_no_calculation')
            else:
                cases.assert_literal(expected,{'status':'complete','C':'0','M':'2','cells_reserved':'0'})
            v.equal(unit['checks']['current']['state'],'verified','bug002_current_still_verified')
            v.equal(expected['resources']['object_bytes_reserved'],'8','bug002_reservation')
            v.equal(expected['resources']['object_bytes_read'],'6' if reason is None else '3','bug002_physical_read')
            envelope=v.load(v.FIX/'reports/complete.json'); envelope.update(expected)
            v.schema(envelope,'report'); v.report(envelope,c['paths'],c['packages'])
            rejected=None
            if variant=='dangling-link':
                wrong=copy.deepcopy(envelope)
                wrong['units'][0]['reasons']=['object_missing']
                wrong['units'][0]['checks']['baseline']['reasons']=['object_missing']
                v.schema(wrong,'report')
                rejected=rejects(lambda:v.report(wrong,c['paths'],c['packages']),'report_units')
            evidence.append(dict(variant=variant,literal_reason=reason,
                                 object_bytes_reserved=expected['resources']['object_bytes_reserved'],
                                 object_bytes_read=expected['resources']['object_bytes_read'],
                                 current_state=unit['checks']['current']['state'],negative_rejection=rejected))
    return dict(bug='BUG-002',variants=evidence,concurrency_proof='PENDENTE_I1_R1')


def coverage_consistency(built,checks):
    matrix=v.load(HERE/'coverage.json')
    entries=matrix['entries']
    v.equal([e['id'] for e in entries],[f'E{i:02}' for i in range(1,56)],'coverage_ids')
    actual={p['test']:p for p in checks.passed}
    scenarios={c['name']:c for c in built}
    for e in entries:
        v.need(bool(e['offline']),'coverage_offline')
        for proof in e['offline']:
            v.need(proof['test'] in actual and e['id'] in actual[proof['test']]['ids'],'coverage_actual_test')
        for proof in e['runtime']:
            name=proof['scenario']
            v.need(name=='public-cancellation-smoke' or name in scenarios and e['id'] in scenarios[name]['ids'],'coverage_prepared_scenario')
        declared={p['scenario'] for p in e['runtime']}
        required={c['name'] for c in built if e['id'] in c['ids']}
        v.need(required<=declared,'coverage_missing_scenario')
    return dict(ids=55,runtime_scenarios=len(built),runtime_state='NAO_EXECUTADO')


def invoke(binary,c,extra=()):
    args=[str(binary),'d0','--baseline',str(c['paths'][0]),'--current',str(c['paths'][1])]
    for side,p in zip(v.SIDES,c['packages']):
        if p is not None: args.extend(['--'+side+'-source',str(p)])
    if c['limits']:
        limits=v.load(v.FIX/'limits-default.json')
        limits.update({k:value for k,value in c['limits'].items() if k!='max_duration_ms'})
        p=c['paths'][0].parent/('limits-'+c['name']+'.json')
        p.write_bytes(v.canonical(limits))
        args.extend(['--limits',str(p)])
    if c.get('deadline_ms'):
        args.extend(['--deadline-ms',c['deadline_ms']])
    args.extend(extra)
    args.append('--json')
    return args


def runtime_case(binary,c):
    if c['name']=='E52': assert_e52_stimulus(c)
    args=invoke(binary,c)
    # Limits preparation is part of setup, before immutable inventory.
    directory=Path(os.path.commonpath([str(p.parent) for p in c['paths']]+[str(p) for p in c['packages'] if p is not None]))
    before=cases.inventory(directory)
    expected=None if 'error' in c else v.model(c['paths'],c['packages'],c['limits'])
    outputs=[]
    for _ in range(2):
        p=subprocess.run(args,capture_output=True,timeout=30)
        need_output=p.stdout
        v.need(bool(need_output),'cli_no_json',p.stderr.decode(errors='replace')[:300])
        value=v.strict(need_output)
        v.equal(need_output,v.canonical(value),'cli_canonical_bytes')
        if 'error' in c:
            code,status=c['error']
            v.equal(value,dict(error=dict(code=code,message=code)),'cli_error')
            v.equal(p.returncode,status,'cli_error_exit')
        else:
            v.report(value,c['paths'],c['packages'],c['limits'],p.returncode)
            cases.assert_literal(expected,c['literal'])
            v.need(value['producer_build']!='fixture:no-production-build','runtime_producer')
        outputs.append(need_output)
    v.equal(outputs[0],outputs[1],'cli_repeatability')
    v.equal(cases.inventory(directory),before,'cli_input_immutability')
    return dict(exit=p.returncode,stdout_sha256=v.sha(outputs[0]),
                producer_build=value.get('producer_build'),producer_version=value.get('producer_version'),repetitions=2)


def cancellation_smoke(binary,directory):
    c=cases.make(directory/'signals','signals',{b'p':b'a\n'*2500},{b'p':b'b\n'*4000},ids=['E35'])
    args=invoke(binary,c)
    start=time.monotonic()
    p=subprocess.Popen(args,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    time.sleep(0.01)
    if p.poll() is not None:
        out,err=p.communicate()
        return dict(state='NAO_VERIFICADO',reason='processo terminou antes do sinal; não prova checkpoint',exit=p.returncode)
    p.send_signal(signal.SIGINT)
    try:
        out,err=p.communicate(timeout=30)
    except subprocess.TimeoutExpired:
        p.kill(); p.communicate()
        raise
    if p.returncode!=130:
        return dict(state='NAO_VERIFICADO',reason='sem confirmação de handler/checkpoint',exit=p.returncode)
    value=v.strict(out)
    v.equal(value,dict(error=dict(code='cancelled',message='cancelled')),'cancelled_envelope')
    v.equal(out,v.canonical(value),'cancelled_canonical')
    # This tests public cancellation only; exact after-p/before-q precedence remains pending.
    return dict(state='SMOKE_EXECUTADO',exit=130,seconds=time.monotonic()-start,
                synchronized_checkpoint='PENDENTE_I1_R1')


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    mode=parser.add_mutually_exclusive_group(required=True)
    mode.add_argument('--self-test',action='store_true')
    mode.add_argument('--binary',type=Path)
    parser.add_argument('--cached-dependencies',type=Path)
    args=parser.parse_args()
    v.cached_dependencies(args.cached_dependencies)
    checks=Checks()
    result=dict(modality='binary' if args.binary else 'self-test',runtime_executed=False,
                runtime_pending=['CLI D0 stable release','synchronized TOCTOU/substitution','SIGINT/deadline exact checkpoints','RSS measurement belongs to R1'],
                skips=[],binary=None)
    checks.run('authorized-integrity',v.integrity)
    if checks.failed:
        result['blocked']='authorized inputs failed; no campaign'
    else:
        checks.run('fixture-tamper-copy',integrity_negative)
        checks.run('regression:BUG-001',regression_bug001,['E24','E50'])
        checks.run('regression:BUG-002',regression_bug002,['E43'])
        checks.run('literal-math-two-methods',mathematical,[f'E{i:02}' for i in range(1,56) if next(e for e in v.load(v.FIX/'cases.json') if e['id']==f'E{i:02}')['calculations']]+['E49'])
        checks.run('strict-transport-canonical-path-u64',transport,['E42','E49','E50'])
        checks.run('byte-hash-schema-vectors',vectors,['E15','E16','E17','E18','E19','E48'])
        checks.run('frozen-reports-inputs',frozen_reports,['E02','E05','E10','E12','E22','E25','E26','E27','E28','E29','E31','E40','E44'])
        checks.run('resource-frontiers',frontier_vectors,['E32','E33','E36','E45','E46','E47','E50','E51','E54','E55'])
        import mutations
        checks.run('offline-specific-mutations',mutations.offline,['E07','E28','E31','E36','E50','E53','E54'])
        with tempfile.TemporaryDirectory(prefix='compass-d0-v1-') as temp:
            directory=Path(temp)
            built=[]
            def prepare():
                built.extend(cases.campaign(directory))
                return len(built)
            checks.run('campaign-construction',prepare)
            if built:
                checks.run('regression:BUG-003',lambda:regression_bug003(next(c for c in built if c['name']=='E52')),['E52'])
            for c in built:
                checks.run('prepare:'+c['name'],lambda c=c:prepared_case(c),c['ids'])
            checks.run('coverage-map',lambda:coverage_consistency(built,checks))
            result['campaign_prepared']=len(built)
            if args.binary:
                binary=args.binary.resolve()
                if not binary.is_file():
                    checks.failed.append(dict(test='binary-precondition',error='stable binary missing'))
                else:
                    before=v.sha(binary.read_bytes())
                    result['binary']={'path':str(binary),'sha256_before':before}
                    probe=None
                    try:
                        probe=subprocess.run([str(binary),'d0','--help'],capture_output=True,timeout=10)
                    except (OSError,subprocess.TimeoutExpired) as error:
                        checks.failed.append(dict(test='binary-precondition',error=str(error)))
                    if probe is not None and (probe.returncode!=0 or b'--baseline' not in probe.stdout):
                        checks.failed.append(dict(test='binary-precondition',error='binário sem interface D0 pública',exit=probe.returncode))
                    elif probe is not None and not checks.failed:
                        for c in built:
                            checks.run('runtime:'+c['name'],lambda c=c:runtime_case(binary,c),c['ids'])
                        checks.run('public-cancellation-smoke',lambda:cancellation_smoke(binary,directory),['E35'])
                        result['runtime_executed']=True
                        result['runtime_pending'].remove('CLI D0 stable release')
                    after=v.sha(binary.read_bytes())
                    result['binary']['sha256_after']=after
                    checks.run('binary-stability',lambda:v.equal(before,after,'binary_changed'))
    result.update(passed=checks.passed,failures=checks.failed,passed_count=len(checks.passed),failure_count=len(checks.failed),
                  state='FAILED' if checks.failed else 'SELF_TEST_PASSED_RUNTIME_PENDING' if not args.binary else 'RUNTIME_CAMPAIGN_EXECUTED')
    result['fixtures_sha256']={str(p.relative_to(v.ROOT)):v.sha(p.read_bytes()) for p in [v.CONTRACT/'SHA256SUMS',HERE/'fixtures.json',HERE/'coverage.json'] if p.exists()}
    target=HERE/'results'/('acceptance.json' if args.binary else 'self-test.json')
    target.parent.mkdir(exist_ok=True)
    target.write_text(json.dumps(result,indent=2,ensure_ascii=False)+'\n')
    print(json.dumps(dict(state=result['state'],passed=result['passed_count'],failures=result['failure_count'],evidence=str(target)),ensure_ascii=False))
    for f in checks.failed: print(f['test']+': '+f['error'][:700],file=sys.stderr)
    return 1 if checks.failed else 0


if __name__=='__main__':
    sys.exit(main())
