#!/usr/bin/env python3
"""BUG-005 public controls; intact RD/Foundation/D0 labs run only in a copy."""
import sys
sys.dont_write_bytecode = True
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
HERE = ROOT/'verification/rd-c01'

def sha(b): return hashlib.sha256(b).hexdigest()
def blob(b): return dict(base64=base64.b64encode(b).decode(),sha256=sha(b),bytes=len(b))
def load(p): return json.loads(p.read_bytes())
def save(p,v): p.write_text(json.dumps(v,indent=2,sort_keys=True,ensure_ascii=False)+'\n')
def compact(v): return json.dumps(v,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()
def pretty(v): return json.dumps(v,ensure_ascii=False,sort_keys=True,indent=2).encode()+b'\n'
def invoke(binary,args):
    p=subprocess.run([str(binary),*map(str,args)],capture_output=True,timeout=30)
    return dict(argv=[str(binary),*map(str,args)],exit=p.returncode,stdout=blob(p.stdout),stderr=blob(p.stderr)),p

def controls(binary):
    before=sha(binary.read_bytes());records=[];setup=[]
    def case(id,args,error='invalid_arguments',exit=2,json_mode=False,query=None):
        record,p=invoke(binary,args);record['id']=id;failures=[]
        expected=dict(error=dict(code=error,message=error)) if error else None
        try:v=json.loads(p.stdout)
        except (ValueError,UnicodeError):v=None;failures.append('BUG-005.json')
        if p.returncode!=exit:failures.append('BUG-005.exit')
        if p.stderr:failures.append('BUG-005.stderr')
        if expected is not None and v!=expected:failures.append('BUG-005.error_value')
        if query is not None:
            if not isinstance(v,dict) or v.get('query',{}).get('text')!=query:failures.append('BUG-005.literal_query')
            if not isinstance(v,dict) or [h['text'] for h in v.get('matches',[])]!=[query+'\n']:failures.append('BUG-005.literal_hit')
        expected_bytes=(compact if json_mode else pretty)(expected if expected is not None else v)
        if p.stdout!=expected_bytes:failures.append('BUG-005.format')
        record.update(expected=dict(exit=exit,error=error,json=json_mode,query=query),expected_stdout=blob(expected_bytes),failures=failures,passed=not failures)
        records.append(record)
    with tempfile.TemporaryDirectory(prefix='rd-c01-controls-') as td:
        tmp=Path(td);absent=tmp/'absent';pathargs=['--snapshot',str(absent),'--source',str(absent)]
        for form,queryargs in [('separate',['--query','--json']),('equals',['--query=--json'])]:
            for pos in ['none','global','before_query','after_error']:
                args=(['--json'] if pos=='global' else [])+['search',*pathargs]+(['--json'] if pos=='before_query' else [])+queryargs+['--max-matches','bad']+(['--json'] if pos=='after_error' else [])
                case('BUG-005.parse.'+form+'.'+pos,args,json_mode=pos!='none')
            # Domain failures with valid parsing must not change literal hyphen values.
            for flag in [False,True]:case('BUG-005.domain.'+form+'.'+str(flag),['search',*pathargs,*queryargs]+(['--json'] if flag else []),error='input_invalid',exit=1,json_mode=flag)
        case('BUG-005.missing_snapshot_value',['search','--snapshot','--json','--source',str(absent),'--query','ordinary'],json_mode=True)
        for flag in [False,True]:case('BUG-005.ordinary.'+str(flag),['search',*pathargs,'--query','ordinary','--max-matches','bad']+(['--json'] if flag else []),json_mode=flag)
        # Build one real synthetic package, independently bound to the snapshot bytes.
        tree=tmp/'tree';tree.mkdir();content=b'--json\n--needle\n-x\nd0\ninit\nsearch\n';(tree/'a').write_bytes(content)
        for args in [['init','--root',tree,'--json'],['snapshot','--root',tree,'--json']]:
            record,p=invoke(binary,args);setup.append(record)
            if p.returncode or p.stderr:raise AssertionError('control setup failed, not BUG-005')
            receipt=json.loads(p.stdout)
        manifest=Path(os.fsdecode(base64.urlsafe_b64decode(receipt['snapshot_path_base64']+'='*(-len(receipt['snapshot_path_base64'])%4))));raw=manifest.read_bytes();snap=json.loads(raw)
        package=tmp/'package';(package/'objects').mkdir(parents=True);(package/'objects'/sha(content)).write_bytes(content)
        descriptor=dict(source_schema_version='1',source_contract_version='0.1.0',**{k:snap[k] for k in ['workspace_id','snapshot_id','policy_id']},snapshot_file_sha256=sha(raw));(package/'source.json').write_bytes(compact(descriptor))
        for form,qa in [('separate',['--query','--json']),('equals',['--query=--json'])]:
            for flag in [False,True]:
                case('BUG-005.absent_source.'+form+'.'+str(flag),['search','--snapshot',str(manifest),'--source',str(absent),*qa]+(['--json'] if flag else []),error='source_invalid',exit=1,json_mode=flag)
        protected={p:sha(p.read_bytes()) for p in [manifest,tree/'a',package/'source.json',package/'objects'/sha(content)]}
        for q in ['--json','--needle','-x','d0','init','search']:
            for joined in [False,True]:
                for flag in [False,True]:
                    qa=['--query='+q] if joined else ['--query',q]
                    case('BUG-005.valid.'+q+'.'+str(joined)+'.'+str(flag),['search','--snapshot',str(manifest),'--source',str(package),*qa]+(['--json'] if flag else []),error=None,exit=0,json_mode=flag,query=q)
        for p,h in protected.items():
            if sha(p.read_bytes())!=h:raise AssertionError('control inputs mutated')
        for args in [['--help'],['--version'],['search','--help'],['init','--help'],['d0','--help']]:
            record,p=invoke(binary,args);record['id']='BUG-005.help_version.'+'.'.join(args);record['failures']=[] if p.returncode==0 and p.stdout and not p.stderr else ['BUG-005.help_version'];record['passed']=not record['failures'];records.append(record)
        inputs=dict(snapshot=blob(raw),descriptor=blob((package/'source.json').read_bytes()),object=blob(content))
    after=sha(binary.read_bytes())
    if after!=before:raise AssertionError('control binary changed')
    return dict(binary=str(binary),binary_before=before,binary_after=after,records=records,setup=setup,inputs=inputs,failed=[x['id'] for x in records if not x['passed']],passed_count=sum(x['passed'] for x in records),total=len(records))

def campaigns(binary,cache):
    before=sha(binary.read_bytes());runs=[]
    with tempfile.TemporaryDirectory(prefix='rd-c01-lab-') as td:
        lab=Path(td)/'compass-foundation'
        shutil.copytree(ROOT,lab,ignore=shutil.ignore_patterns('.git','target','__pycache__'))
        # Historical authentication also references this document, which remains read-only.
        shutil.copytree(ROOT.parent/'compass-documento-principal',Path(td)/'compass-documento-principal')
        originals={str(p.relative_to(ROOT)):sha(p.read_bytes()) for p in (ROOT/'verification/rd').rglob('*') if p.is_file()}
        copied={n:sha((lab/n).read_bytes()) for n in originals}
        if copied!=originals:raise AssertionError('RD copy differs before campaign')
        for mode in [[],['--regressions']]:
            args=[sys.executable,str(lab/'verification/rd/acceptance.py'),*mode,'--binary',str(binary)]
            if mode and cache:args+=['--cached-dependencies',str(cache)]
            p=subprocess.run(args,cwd=lab,capture_output=True,timeout=600,env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1'))
            runs.append(dict(argv=args,cwd=str(lab),exit=p.returncode,stdout=blob(p.stdout),stderr=blob(p.stderr)))
            if p.returncode:
                return dict(runs=runs,failed=True,binary_before=before,binary_after=sha(binary.read_bytes()))
        result=dict(runs=runs,rd=load(lab/'verification/rd/integration.json'),resource_profile=load(lab/'verification/rd/resource-profile.json'),regressions=load(lab/'verification/rd/regressions.json'),original_rd_hashes=originals,copied_rd_hashes_before=copied,failed=False,binary_before=before,binary_after=sha(binary.read_bytes()))
        if result['binary_after']!=before:raise AssertionError('campaign binary changed')
        return result

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',type=Path,required=True);p.add_argument('--control-only',action='store_true');p.add_argument('--cached-dependencies',type=Path);a=p.parse_args();binary=a.binary.resolve(strict=True)
    existing=load(HERE/'integration.json') if (HERE/'integration.json').exists() else dict(bug='BUG-005',bindings=['RD22','RD24','RD-AC-07'],permanent_tests=['bug005_query_json_is_a_value_and_real_flags_select_transport','bug005_valid_hyphenated_queries_keep_typed_transport'],controls=[])
    result=controls(binary);existing['controls'].append(result);save(HERE/'integration.json',existing)
    print('CONTROLS',result['passed_count'],'/',result['total'],'FAILED',result['failed'],flush=True)
    if result['failed']:return 1
    if not a.control_only:
        result=campaigns(binary,a.cached_dependencies);existing['campaign']=result.copy();reg=existing['campaign'].pop('regressions',None);save(HERE/'integration.json',existing)
        if reg is not None:save(HERE/'regressions.json',reg)
        print('CAMPAIGNS', 'FAILED' if result['failed'] else 'PASSED',flush=True)
        if result['failed']:return 1
    return 0
if __name__=='__main__':sys.exit(main())
