#!/usr/bin/env python3
"""Independent line oracle and public read journey; synthetic inputs only."""
import sys
sys.dont_write_bytecode = True
import argparse, base64, copy, hashlib, json, os, platform, shutil, socket, subprocess, tempfile, time
from pathlib import Path
HERE = Path(__file__).resolve().parent

def sha(raw): return hashlib.sha256(raw).hexdigest()
def key(raw): return base64.urlsafe_b64encode(raw).rstrip(b'=').decode()
def unkey(s): return base64.urlsafe_b64decode(s+'='*(-len(s)%4))
def canonical(v): return json.dumps(v,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()
def require(ok, tag):
    if not ok: raise AssertionError(tag)
def oracle(raw, start, maximum):
    # LF is the only separator; offsets are counted in bytes, not characters.
    ends=[i+1 for i,b in enumerate(raw) if b==10]
    if raw and (not ends or ends[-1]!=len(raw)): ends.append(len(raw))
    total=len(ends);first=min(start-1,total);last=min(first+maximum,total)
    begin=ends[first-1] if first else 0;end=ends[last-1] if last else 0
    return dict(total_lines=str(total),returned_lines=str(last-first),start_byte=str(begin),end_byte=str(end),text=raw[begin:end].decode(),next_start_line=str(last+1) if last<total else None,truncated=last<total)
def validate(v, expected, raw, path):
    require(v['status']=='available','read.availability')
    require(v['excerpt']['text']==expected['text'],'read.text')
    require(all(v['excerpt'][k]==expected[k] for k in ['start_byte','end_byte','total_lines','returned_lines']),'read.offsets')
    require(v['file']['path_key']==key(path) and v['file']['expected_revision']==dict(content_sha256=sha(raw),size_bytes=str(len(raw))),'read.revision')
    require(all(v['excerpt'][k]==expected[k] for k in ['next_start_line','truncated']),'read.continuation')

def validate_unavailable(v):
    require(v['status']=='unavailable' and v['excerpt']==dict(total_lines=None,returned_lines='0',start_byte=None,end_byte=None,text=None,next_start_line=None,truncated=False),'read.failed_availability')

def selftest():
    cases=json.loads((HERE/'fixtures.json').read_bytes())['cases']
    for c in cases: require(oracle(c['text'].encode(),c['start'],c['max'])==c['expected'],'fixture:'+c['name'])
    raw=cases[0]['text'].encode();expected=cases[0]['expected'];path=b'a.txt'
    valid=dict(status='available',excerpt=copy.deepcopy(expected),file=dict(path_key=key(path),expected_revision=dict(content_sha256=sha(raw),size_bytes=str(len(raw)))))
    validate(valid,expected,raw,path)
    mutations=[('text',lambda v:v['excerpt'].update(text='wrong')),('offsets',lambda v:v['excerpt'].update(start_byte='3')),('revision',lambda v:v['file']['expected_revision'].update(content_sha256='0'*64)),('continuation',lambda v:v['excerpt'].update(next_start_line=None)),('availability',lambda v:v.update(status='unavailable'))]
    for tag,mutate in mutations:
        bad=copy.deepcopy(valid);mutate(bad)
        try: validate(bad,expected,raw,path)
        except AssertionError as e: require(str(e)=='read.'+tag,'mutation-specific:'+tag)
        else: raise AssertionError('mutation accepted:'+tag)
    failed=dict(status='unavailable',excerpt=dict(total_lines=None,returned_lines='0',start_byte=None,end_byte=None,text=None,next_start_line=None,truncated=False))
    validate_unavailable(failed)
    promoted=copy.deepcopy(failed);promoted.update(status='available');promoted['excerpt']=copy.deepcopy(expected)
    try: validate_unavailable(promoted)
    except AssertionError as e: require(str(e)=='read.failed_availability','specific false-availability rejection')
    else: raise AssertionError('failed source promoted to available')
    bad=copy.deepcopy(cases[0]);bad['expected']['end_byte']='9' 
    require(oracle(bad['text'].encode(),bad['start'],bad['max'])!=bad['expected'],'corrupt-fixture-rejected')
    return dict(fixtures=len(cases),specific_mutations=len(mutations)+1,fixture_tampering_rejected=True)

class Lab:
    def __init__(self,binary): self.binary=Path(binary).resolve();self.commands=[]
    def invoke(self,args,code=0):
        p=subprocess.run([str(self.binary),*map(str,args)],capture_output=True,timeout=30)
        self.commands.append(dict(argv=list(map(str,args)),exit=p.returncode,stdout_base64=base64.b64encode(p.stdout).decode(),stderr_base64=base64.b64encode(p.stderr).decode()))
        require(p.returncode==code,('exit',args,p.returncode,p.stdout,p.stderr));require(not p.stderr,'stderr')
        return json.loads(p.stdout),p.stdout
    def make(self,base,files,partial=False,identity=None):
        tree=base/'tree';tree.mkdir(parents=True)
        for name,raw in files.items():
            p=os.fsencode(tree)+b'/'+name;os.makedirs(os.path.dirname(p),exist_ok=True)
            with open(p,'wb') as f:f.write(raw)
        self.invoke(['init','--root',tree,'--json'])
        if identity is not None:(tree/'.compass/workspace.json').write_bytes(identity)
        receipt,_=self.invoke(['snapshot','--root',tree,'--json'])
        original=Path(os.fsdecode(unkey(receipt['snapshot_path_base64'])))
        snap=json.loads(original.read_bytes());manifest=base/'snapshot.json'
        if partial:
            snap['quality']='partial';snap['state_digest']=None;snap['directory_coverage'][0].update(state='unreadable',reason='permission_denied')
        manifest.write_bytes(canonical(snap))
        package=base/'package';(package/'objects').mkdir(parents=True)
        for e in snap['entries']:
            if e['state']=='observed':(package/'objects'/e['content_sha256']).write_bytes(files[unkey(e['path_key'])])
        (package/'source.json').write_bytes(canonical(dict(source_schema_version='1',source_contract_version='0.1.0',**{k:snap[k] for k in ['workspace_id','snapshot_id','policy_id']},snapshot_file_sha256=sha(manifest.read_bytes()))))
        return dict(tree=tree,manifest=manifest,package=package,files=files,snapshot=snap)
    def read(self,c,path=b'a.txt',start=1,maximum=20,code=0,limits=None,pretty=False):
        args=['read','--snapshot',c['manifest'],'--source',c['package'],'--path-key',key(path),'--start-line',start,'--max-lines',maximum]
        if limits is not None:
            p=c['package'].parent/'limits.json';p.write_bytes(canonical(limits));args+=['--limits',p]
        if not pretty:args+=['--json']
        watched=[c['manifest'],c['package']/'source.json']+[p for p in (c['package']/'objects').iterdir() if p.is_file() and not p.is_symlink()]
        before={p:sha(p.read_bytes()) for p in watched}
        v,b=self.invoke(args,code)
        require(all(sha(p.read_bytes())==h for p,h in before.items()),'inputs unchanged')
        require(b==(json.dumps(v,ensure_ascii=False,sort_keys=True,indent=2).encode()+b'\n' if pretty else canonical(v)),'transport')
        if 'status' in v:
            require(set(v)==set('schema_version read_contract_version report_type producer_version producer_build scope snapshot source request limits status reasons file excerpt resources'.split()),'closed fields')
            require(v['snapshot']['snapshot_file_sha256']==sha(c['manifest'].read_bytes()),'snapshot identity')
            if v['status']!='available':
                require(v['excerpt']==dict(total_lines=None,returned_lines='0',start_byte=None,end_byte=None,text=None,next_start_line=None,truncated=False),'unavailable excerpt')
        return v,b

def campaign(binary):
    lab=Lab(binary);before=sha(lab.binary.read_bytes());checks=[];measurements=[]
    with tempfile.TemporaryDirectory(prefix='compass-read-') as td:
        root=Path(td)
        for i,c in enumerate(json.loads((HERE/'fixtures.json').read_bytes())['cases']):
            raw=c['text'].encode();case=lab.make(root/str(i),{b'a.txt':raw});v,_=lab.read(case,start=c['start'],maximum=c['max'])
            validate(v,c['expected'],raw,b'a.txt');checks.append(c['name'])
        c=lab.make(root/'flow',{b'a.txt':b'before\nalvo\nafter\ntail\n',b'other':b'other\n',b'.env':b'excluded',b'dir/x':b'x'})
        found,_=lab.invoke(['search','--snapshot',c['manifest'],'--source',c['package'],'--query','alvo','--json'])
        hit=found['matches'][0];require(hit['line']=='2','search line')
        v,raw_out=lab.read(c,start=1,maximum=3);validate(v,oracle(c['files'][b'a.txt'],1,3),c['files'][b'a.txt'],b'a.txt')
        nextv,_=lab.read(c,start=int(v['excerpt']['next_start_line']),maximum=3);require(nextv['excerpt']['text']=='tail\n','continuation')
        identity=(c['tree']/'.compass/workspace.json').read_bytes()
        shutil.rmtree(c['tree']);require(lab.read(c,start=1,maximum=3)[1]==raw_out,'offline same bytes')
        c1=lab.make(root/'s1',{b'a.txt':b'changed\nalvo\n'},identity=identity);require(c1['snapshot']['workspace_id']==c['snapshot']['workspace_id'],'same workspace');v1,_=lab.read(c1);require(v1['file']['expected_revision']!=v['file']['expected_revision'],'different revisions')
        swapped=dict(c,package=c1['package']);require(lab.read(swapped,code=2)[0]['error']['code']=='incompatible_inputs','swapped package')
        for name in [b'missing',b'.env',b'dir']:
            require(lab.read(c,path=name,code=1)[0]['error']['code']=='path_not_observed','not observed')
        (c['package']/'objects'/sha(b'other\n')).unlink();require(lab.read(c)[0]['status']=='available','unselected missing')
        (c['package']/'objects'/sha(b'x')).write_bytes(b'y');require(lab.read(c)[0]['status']=='available','unselected corrupt')
        lab.read(c,pretty=True);checks+=['search-read-continuation','offline','revisions','binding','selection-only','pretty']
        c=lab.make(root/'partial',{b'a.txt':b'a\n'},partial=True);v,_=lab.read(c);require(v['status']=='available' and v['snapshot']['quality']=='partial','partial observed')
        c=lab.make(root/'native',{b'\xfb\xef':b'native\n'});v,_=lab.read(c,path=b'\xfb\xef');require(v['excerpt']['text']=='native\n','hyphenated native key');checks+=['partial','native']
        for kind in ['missing','same-size','short','long','link','fifo','socket','directory','objects-link','root-link']:
            c=lab.make(root/('bad-'+kind),{b'a.txt':b'a\n'});obj=c['package']/'objects'/sha(b'a\n');obj.unlink();sock=None
            if kind=='same-size':obj.write_bytes(b'b\n')
            elif kind=='short':obj.write_bytes(b'b')
            elif kind=='long':obj.write_bytes(b'xxxx')
            elif kind=='link':obj.symlink_to('/etc/passwd')
            elif kind=='fifo':os.mkfifo(obj)
            elif kind=='socket':
                sock=socket.socket(socket.AF_UNIX);short=root/'s';sock.bind(str(short));short.rename(obj)
            elif kind=='directory':obj.mkdir()
            elif kind in ['objects-link','root-link']:
                obj.write_bytes(b'a\n');p=c['package']/'objects' if kind=='objects-link' else c['package'];old=p.with_name(p.name+'-old');p.rename(old);p.symlink_to(old)
            v,_=lab.read(c,start=18446744073709551615,code=1)
            if sock:sock.close()
            if kind=='root-link':require('error' in v,'root link error')
            else:
                require(v['status']=='unavailable','bad object unavailable')
                if kind in ['link','fifo','socket','directory','objects-link']:require(v['resources']['object_bytes_read']=='0','no external/special bytes')
            checks.append('object-'+kind)
        for i,data in enumerate([b'good\n\0',b'\xff\0',b'good\n'+b'x'*4097]):
            c=lab.make(root/('text-'+str(i)),{b'a.txt':data});v,_=lab.read(c,maximum=1,code=1);require(v['file']['check']['state']=='verified','text after verification');checks.append('text-'+str(i))
        c=lab.make(root/'caps',{b'a.txt':b'a\n'});v,_=lab.read(c);limits={k:n for k,n in v['limits'].items() if k!='max_duration_ms'}
        for name in ['max_manifest_bytes','max_source_descriptor_bytes','max_records','max_json_depth','max_logical_memory_bytes']:
            profile=dict(limits,**{name:'1'});bad,_=lab.read(c,code=1,limits=profile);require('error' in bad,'input limit '+name);checks.append(name)
        # Canonical cap includes the cap itself: solve its stable encoded length.
        cap=65536
        for _ in range(8):
            profile=dict(limits,max_output_bytes=str(cap));candidate,encoded=lab.read(c,limits=profile)
            if len(encoded)==cap:break
            cap=len(encoded)
        require(len(encoded)==cap,'fixed output cap');bad,_=lab.read(c,code=1,limits=dict(limits,max_output_bytes=str(cap-1)));require(bad['error']['code']=='output_limit','cap-1');checks.append('exact-output-cap')
        files={f'dir-{i//10:03d}/file-{i:04d}.txt'.encode():''.join((f'F{i:04d}:L{j:02d}:'+('BASE' if j==31 else 'NONE')).ljust(63,'x')+'\n' for j in range(64)).encode() for i in range(1000)}
        c=lab.make(root/'load',files);path=b'dir-000/file-0000.txt';expected=oracle(files[path],30,5)
        args=[str(lab.binary),'read','--snapshot',str(c['manifest']),'--source',str(c['package']),'--path-key',key(path),'--start-line','30','--max-lines','5','--json'];outputs=[]
        for i in range(3):
            stats=root/f'time-{i}';start=time.perf_counter_ns()
            p=subprocess.run(['/usr/bin/time','-f','%U %S %e %M','-o',str(stats),*args],capture_output=True,timeout=30);wall=time.perf_counter_ns()-start
            require(p.returncode==0 and not p.stderr,'native measurement');v=json.loads(p.stdout);validate(v,expected,files[path],path)
            require(v['resources']['object_bytes_read']=='4096' and v['resources']['object_bytes_reserved']=='4097','selected object ledger')
            outputs.append(p.stdout);measurements.append(dict(command=args,gnu_time=stats.read_text().strip(),gnu_time_fields=['user_s','system_s','elapsed_s','max_rss_kib'],wall_ns=wall,resources=v['resources'],stdout_sha256=sha(p.stdout)))
        require(outputs[0]==outputs[1]==outputs[2],'stable measurement output');checks.append('native-resource-profile')
    require(sha(lab.binary.read_bytes())==before,'binary stable')
    return dict(binary=str(lab.binary),binary_sha256=before,fixtures_sha256=sha((HERE/'fixtures.json').read_bytes()),environment=platform.platform(),checks=checks,commands=lab.commands,resource_profile=measurements,limitations=['Linux/openat2; synthetic inputs','cache uncontrolled; no SLO','logical ledger is not RSS','synchronized races/cancellation covered by Rust tests'])

def main():
    p=argparse.ArgumentParser();p.add_argument('--self-test',action='store_true');p.add_argument('--binary');p.add_argument('--output',type=Path);a=p.parse_args();result=dict(self_test=selftest())
    if a.binary:result['campaign']=campaign(a.binary)
    elif not a.self_test:p.error('provide --self-test or --binary')
    if a.output:a.output.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps(dict(self_test=result['self_test'],commands=len(result.get('campaign',{}).get('commands',[])),checks=result.get('campaign',{}).get('checks',[]))))
if __name__=='__main__':main()
