#!/usr/bin/env python3
"""Specific schema-valid protocol negatives; generic errors never count as kills."""
import sys
sys.dont_write_bytecode=True
import argparse
import copy
import json
from pathlib import Path
import subprocess
import tempfile
import cases
import oracle
import validate as v

HERE=Path(__file__).resolve().parent


def synthetic(c):
    value=v.load(v.FIX/'reports/complete.json')
    value.update(v.model(c['paths'],c['packages'],c['limits']))
    return value


def specifications(directory):
    def make(name,a,b,**opts):
        return cases.make(directory/name,name,a,b,**opts)
    missing=make('missing',{b'p':b'a\n'},{b'p':b'a\n'})
    missing['packages'][0]=None
    aggregate=make('unchanged',{b'p':b'a\n',b'q':b'a\n'*9},{b'p':b'b\n',b'q':b'a\n'*9},literal={'C':'2','M':'20'})
    count=make('count-only',{b'p':b'a\n'*4000},{b'p':b'b\n'*4000},literal={'status':'unavailable'})
    equal=make('equal',{b'p':b'a\n'*4000},{b'p':b'a\n'*4000},literal={'C':'0','M':'8000','cells_reserved':'0'})
    partial=make('subset',{b'p':b'a\n'},{b'p':b'a\n'},cover1={b'private':'unreadable'},literal={'status':'partial'})
    def false_zero(r):
        u=r['units'][0]
        u.update(status='available',reasons=[],calculation=oracle.calculation(['a'],['a']))
        u['checks']['baseline'].update(state='verified',reasons=[],actual_revision=copy.deepcopy(u['checks']['baseline']['expected_revision']),hash_check=True,size_check=True,source_ref='0'*64)
        r['measurement']=oracle.aggregate([u])
    def drop_unchanged(r):
        r['measurement']['line_mass']='2'
        r['measurement']['value']=oracle.ratio(2,2)
        r['measurement']['known_subset'].update(line_mass='2',value=oracle.ratio(2,2))
    def count_equality(r):
        u=r['units'][0]
        u.update(status='available',reasons=[],calculation=oracle.calculation(['a']*4000,['a']*4000))
    def quadratic(r): r['resources']['cells_reserved']='16000000'
    def bad_policy(r): r['metric_policy_id']='0'*64
    def integral_subset(r):
        r['measurement'].update(status='complete',value=oracle.ratio(0,2),line_mass='2',zero_basis=None,reasons=[])
    def overflow(r): r['resources']['object_bytes_read']='18446744073709551616'
    def invalid_path(r): r['units'][0]['path_key']='Li4'
    def binding(r): r['sources']['baseline']['snapshot_file_sha256']='0'*64
    def memory(r): r['resources']['logical_memory_peak']='1'
    def order(r): r['units'].reverse()
    return [('false-zero-missing-source',missing,false_zero,'report_units'),
            ('aggregate-without-unchanged',aggregate,drop_unchanged,'report_measurement'),
            ('equality-from-line-count',count,count_equality,'report_units'),
            ('quadratic-debit-for-equality',equal,quadratic,'report_resources'),
            ('incoherent-policy-hash',equal,bad_policy,'report_metric_policy_id'),
            ('subset-as-integral',partial,integral_subset,'report_measurement'),
            ('u64-overflow',equal,overflow,'u64'),('invalid-native-path',equal,invalid_path,'path'),
            ('false-manifest-binding',equal,binding,'report_sources'),
            ('false-logical-memory',equal,memory,'report_resources'),
            ('wrong-native-unit-order',aggregate,order,'report_units')]


def run(binary=None):
    results=[]
    with tempfile.TemporaryDirectory(prefix='d0-mutants-') as tmp:
        for name,c,mutate,assertion in specifications(Path(tmp)):
            if binary:
                from acceptance import invoke
                args=invoke(binary,c)
                before=cases.inventory(Path(tmp))
                p=subprocess.run(args,capture_output=True,timeout=30)
                valid=v.strict(p.stdout)
                v.equal(p.stdout,v.canonical(valid),'mutation_control_canonical')
                v.report(valid,c['paths'],c['packages'],c['limits'],p.returncode)
                v.equal(cases.inventory(Path(tmp)),before,'mutation_input_immutability')
            else:
                valid=synthetic(c)
                v.report(valid,c['paths'],c['packages'],c['limits'])
            cases.assert_literal(v.model(c['paths'],c['packages'],c['limits']),c['literal'])
            mutant=copy.deepcopy(valid)
            mutate(mutant)
            # The schema must accept before the independent semantic assertion kills.
            v.schema(mutant,'report')
            try:
                v.report(mutant,c['paths'],c['packages'],c['limits'])
            except v.Rejected as error:
                v.equal(error.code,assertion,'mutation_wrong_assertion')
                results.append(dict(mutation=name,positive_control='PASS',schema='PASS',rejected_by=error.code))
            else:
                raise AssertionError('survived mutation '+name)
    return results


def offline():
    return run()


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    mode=parser.add_mutually_exclusive_group(required=True)
    mode.add_argument('--self-test',action='store_true')
    mode.add_argument('--binary',type=Path)
    parser.add_argument('--cached-dependencies',type=Path)
    args=parser.parse_args()
    v.cached_dependencies(args.cached_dependencies)
    result=dict(modality='binary' if args.binary else 'self-test',runtime_executed=False,failures=[],rejected=[],binary=None)
    try:
        result['integrity']=v.integrity()
        binary=args.binary.resolve() if args.binary else None
        if binary:
            before=v.sha(binary.read_bytes())
            result['binary']=dict(path=str(binary),sha256_before=before)
            probe=subprocess.run([str(binary),'d0','--help'],capture_output=True,timeout=10)
            v.need(probe.returncode==0 and b'--baseline' in probe.stdout,'binary_no_d0')
        try:
            result['rejected']=run(binary)
            result['runtime_executed']=bool(binary)
        finally:
            if binary:
                after=v.sha(binary.read_bytes())
                result['binary']['sha256_after']=after
                v.equal(before,after,'binary_changed')
    except Exception as error:
        result['failures'].append(dict(type=type(error).__name__,error=str(error)))
    result['state']='FAILED' if result['failures'] else 'PASSED'
    result['runtime_pending']=not result['runtime_executed']
    target=HERE/'results/mutations.json'
    target.parent.mkdir(exist_ok=True)
    target.write_text(json.dumps(result,indent=2,ensure_ascii=False)+'\n')
    print(json.dumps(dict(state=result['state'],rejected=len(result['rejected']),failures=result['failures'],evidence=str(target)),ensure_ascii=False))
    return 1 if result['failures'] else 0


if __name__=='__main__': sys.exit(main())
