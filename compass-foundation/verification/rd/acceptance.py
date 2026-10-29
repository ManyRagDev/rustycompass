#!/usr/bin/env python3
"""Independent RD byte oracle and real CLI laboratory. Standard library only."""
import sys
sys.dont_write_bytecode = True
import argparse, base64, copy, hashlib, importlib.util, json, os, pathlib, platform, shutil, socket, subprocess, tempfile, time
ROOT = pathlib.Path(__file__).resolve().parents[2]
HERE = ROOT/'verification/rd'
def sha(b): return hashlib.sha256(b).hexdigest()
def save(name,v): (HERE/name).write_text(json.dumps(v,ensure_ascii=False,indent=2,sort_keys=True)+'\n')
def require(ok, tag):
    if not ok: raise AssertionError(tag)
def oracle(raw,query):
    text=raw.decode('utf-8')
    if '\0' in text: raise ValueError('nul')
    result=[]; start=0; line=0
    while start<len(raw):
        end=raw.find(b'\n',start);end=len(raw) if end<0 else end+1;line+=1
        if query.encode() in raw[start:end]: result.append(dict(line=str(line),start_byte=str(start),end_byte=str(end),text=raw[start:end].decode()))
        start=end
    return result

def validate(v, expected):
    hits=v['matches']; want=expected['matches']
    require([h['text'] for h in hits]==[h['text'] for h in want],'RD24.text')
    require([(h['line'],h['start_byte'],h['end_byte']) for h in hits]==[(h['line'],h['start_byte'],h['end_byte']) for h in want],'RD24.offsets')
    require([(h['path_key'],h['content_sha256'],h['size_bytes']) for h in hits]==[(h['path_key'],h['content_sha256'],h['size_bytes']) for h in want],'RD24.revision')
    require(v['scan']['status']==expected['scan']['status'] and v['scan']['reasons']==expected['scan']['reasons'],'RD24.coverage')
    require(all(v['scan'][k]==expected['scan'][k] for k in ['matching_lines_known','matches_returned','matches_truncated']),'RD24.truncation')

def selftest():
    f=json.loads((HERE/'fixtures.json').read_bytes())
    for x in f['literal']:
        raw=base64.b64decode(x['raw']); require(sha(raw)==x['sha256'],x['id']+'.hash');require(oracle(raw,x['query'])==x['hits'],x['id']+'.literal')
    require(8*1234+4096*5+65536+8*4096==128656,'RD17.S_manifest')
    require(128656+8*321==131224,'RD17.S');require(2*(8192+1)+65536==81922,'RD17.W')
    v=copy.deepcopy(f['mutation_control']);validate(v,v)
    mutations=[('text',lambda x:x['matches'][0].update(text='corrupt')),('offsets',lambda x:x['matches'][0].update(start_byte='1')),('revision',lambda x:x['matches'][0].update(content_sha256='0'*64)),('coverage',lambda x:x['scan'].update(status='complete')),('truncation',lambda x:x['scan'].update(matches_truncated=False))]
    for tag,mutate in mutations:
        bad=copy.deepcopy(v);mutate(bad)
        try: validate(bad,v)
        except AssertionError as e: require(str(e)=='RD24.'+tag,'wrong mutation assertion')
        else: raise AssertionError('accepted '+tag)
    bad=copy.deepcopy(f['literal'][0]);bad['hits'][0]['end_byte']='4'
    try: require(oracle(base64.b64decode(bad['raw']),bad['query'])==bad['hits'],'RD01.literal')
    except AssertionError as e: require(str(e)=='RD01.literal','specific expected rejection')
    else: raise AssertionError('bad expected accepted')
    print('SELF_TEST_OK: literal bytes, SHA256, S/W, five specific mutations; no production invoked')

def canonical(v): return json.dumps(v,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()
def key(b): return base64.urlsafe_b64encode(b).rstrip(b'=').decode()
def unkey(s): return base64.urlsafe_b64decode(s+'='*(-len(s)%4))
def blob(b): return dict(base64=base64.b64encode(b).decode(),sha256=sha(b),bytes=len(b))
def inventory(p): return {str(q.relative_to(p)):sha(q.read_bytes()) for q in p.rglob('*') if q.is_file() and not q.is_symlink()}

def campaign(binary):
    binary=pathlib.Path(binary).resolve(strict=True);before=sha(binary.read_bytes());records=[];checks=[]
    def check(ok,tag):require(ok,tag);checks.append(tag)
    def invoke(args,code=0):
        p=subprocess.run([str(binary),*map(str,args)],capture_output=True,timeout=30)
        records.append(dict(command=[str(binary),*map(str,args)],exit=p.returncode,stdout=blob(p.stdout),stderr=blob(p.stderr)))
        check(p.returncode==code,'exit:'+str(args[:2]));check(not p.stderr,'stderr')
        v=json.loads(p.stdout);return v,p.stdout
    def make(base,files,partial=False,identity=None):
        tree=base/'tree';tree.mkdir(parents=True)
        for name,data in files.items():
            p=os.fsencode(tree)+b'/'+name;os.makedirs(os.path.dirname(p),exist_ok=True)
            with open(p,'wb') as f:f.write(data)
        invoke(['init','--root',tree,'--json'])
        if identity:shutil.copyfile(identity,tree/'.compass/workspace.json')
        receipt,_=invoke(['snapshot','--root',tree,'--json']);manifest=pathlib.Path(os.fsdecode(unkey(receipt['snapshot_path_base64'])))
        raw=manifest.read_bytes();manifest=base/'snapshot.json';manifest.write_bytes(raw);s=json.loads(raw)
        if partial:
            s['directory_coverage'][0]['state']='unreadable';s['directory_coverage'][0]['reason']='permission_denied';s['quality']='partial';s['state_digest']=None
            # Foundation incomplete coverage is explicitly permission_denied.
            raw=canonical(s);manifest.write_bytes(raw)
        package=base/'package';(package/'objects').mkdir(parents=True)
        for e in s['entries']:
            if e['state']=='observed':(package/'objects'/e['content_sha256']).write_bytes(files[unkey(e['path_key'])])
        descriptor=dict(source_schema_version='1',source_contract_version='0.1.0',**{k:s[k] for k in ['workspace_id','snapshot_id','policy_id']},snapshot_file_sha256=sha(raw))
        (package/'source.json').write_bytes(canonical(descriptor))
        return dict(tree=tree,manifest=manifest,package=package,files=files,snapshot=s)
    def query(c,q='alvo',max_matches=20,limits=None,code=0,extra=(),as_json=True):
        args=['search','--snapshot',c['manifest'],'--source',c['package'],'--query',q,'--max-matches',str(max_matches)]
        if limits is not None:
            p=c['package'].parent/'limits.json';p.write_bytes(canonical(limits));args+=['--limits',p]
        args+=list(extra)
        if as_json:args+=['--json']
        before_inputs={str(p):sha(p.read_bytes()) for p in [c['manifest'],c['package']/'source.json'] if p.is_file()}
        v,b=invoke(args,code)
        check(all(sha(pathlib.Path(p).read_bytes())==h for p,h in before_inputs.items()),'inputs_unchanged')
        if as_json:check(b==canonical(v),'canonical')
        else:check(b.endswith(b'\n'),'pretty_newline')
        if 'scan' in v:
            check(set(v)==set('schema_version retrieval_contract_version report_type producer_version producer_build scope snapshot source query limits scan files matches resources'.split()),'closed_report')
            check(set(v['scan'])==set('status reasons observed_files verified_text_files non_text_files unavailable_files excluded_entries matching_lines_known matches_returned matches_truncated'.split()),'closed_scan')
            check(v['scope']=='declared_snapshot_only' and v['snapshot']['snapshot_file_sha256']==sha(c['manifest'].read_bytes()),'declared_snapshot_binding')
            check(v['source']['source_ref']==sha((c['package']/'source.json').read_bytes()),'source_ref')
            for f in v['files']:
                check(set(f)==set('path_key expected_revision state reasons check'.split()),'closed_file')
                check(set(f['check'])==set('state source_ref actual_revision hash_check size_check'.split()),'closed_check')
                check(f['check']['source_ref']==v['source']['source_ref'],'check.source_ref')
                if f['check']['state']=='verified':check(f['check']['actual_revision']==f['expected_revision'] and f['check']['hash_check'] is True and f['check']['size_check'] is True,'verified_revision')
            for h in v['matches']:
                check(set(h)==set('path_key content_sha256 size_bytes line start_byte end_byte text'.split()),'closed_match')
            for k in ['observed_files','verified_text_files','non_text_files','unavailable_files','excluded_entries','matching_lines_known','matches_returned']:
                n=v['scan'][k];check(isinstance(n,str) and str(int(n))==n and 0<=int(n)<2**64,'canonical_u64:'+k)
            scan=v['scan'];check(int(scan['observed_files'])==sum(int(scan[k]) for k in ['verified_text_files','non_text_files','unavailable_files']),'scan_partition')
            check(int(scan['matches_returned'])==len(v['matches']) and scan['matches_truncated']==(int(scan['matching_lines_known'])>len(v['matches'])),'selection_identity')
            lim=v['limits'];snap=json.loads(c['manifest'].read_bytes());n=sum(len(snap[k]) for k in ['entries','directory_coverage','issues']);sm=8*c['manifest'].stat().st_size+4096*n+65536+8*int(lim['max_output_bytes']);structural=sm+8*(c['package']/'source.json').stat().st_size
            working=[2*(int(f['expected_revision']['size_bytes'])+1)+65536 for f in v['files'] if not set(f['reasons'])&{'memory_budget','file_bytes_limit'}]
            check(v['resources']['logical_memory_peak']==str(structural+max(working,default=0)),'RD17.independent_peak')
        return v,b
    def expected(c,q,max_matches=20,status='complete',reasons=()):
        hits=[]
        for name,raw in sorted(c['files'].items()):
            if name.startswith(b'.env'):continue
            try: lines=oracle(raw,q)
            except (UnicodeError,ValueError):continue
            # LF only and entire-file preflight independently.
            lens=[len(p)+1 for p in raw.split(b'\n')[:-1]]+([len(raw.split(b'\n')[-1])] if raw.split(b'\n')[-1] else [])
            if any(n>4096 for n in lens):continue
            hits.extend(dict(h,path_key=key(name),content_sha256=sha(raw),size_bytes=str(len(raw))) for h in lines)
        known=len(hits);hits=hits[:max_matches]
        return dict(matches=hits,scan=dict(status=status,reasons=list(reasons),matching_lines_known=str(known),matches_returned=str(len(hits)),matches_truncated=known>len(hits)))
    def error(c,tag,code=1,**kwargs):
        v,b=query(c,code=code,**kwargs);check(v==dict(error=dict(code=tag,message=tag)),tag);return v
    fixture=json.loads((HERE/'fixtures.json').read_bytes())
    defaults=dict(zip(['max_manifest_bytes','max_source_descriptor_bytes','max_records','max_json_depth','max_file_bytes','max_total_object_bytes','max_output_bytes','max_logical_memory_bytes','max_line_bytes'],map(str,[16777216,65536,10000,64,4194304,67108864,1048576,134217728,4096])))
    with tempfile.TemporaryDirectory(prefix='rd-') as tmp:
        tmp=pathlib.Path(tmp)
        for i,x in enumerate(fixture['literal']):
            c=make(tmp/f'literal-{i}',{b'a.txt':base64.b64decode(x['raw'])});v,b=query(c,x['query']);validate(v,expected(c,x['query']));check(v['files'][0]['check']['state']=='verified',x['id']+'.verified');check(v['resources']['object_bytes_reserved']==str(len(base64.b64decode(x['raw']))+1),x['id']+'.reserved')
        c=make(tmp/'native',{b'\xff':b'alvo\n',b'10':b'alvo\n',b'2':b'alvo\n',b'nul':b'\0',b'bad':b'\xff\0',b'.env':b'alvo secret'})
        v,b=query(c);validate(v,expected(c,'alvo'));check([unkey(h['path_key']) for h in v['matches']]==[b'10',b'2',b'\xff'],'RD07.native');check([f['reasons'] for f in v['files'] if f['state']=='non_text']==[['invalid_utf8'],['nul']],'RD08.precedence')
        check(v['scan']['non_text_files']=='2' and v['scan']['excluded_entries']=='2','RD08.excluded') # .compass and .env
        # max_matches does not stop scanning; a later failure changes coverage.
        c=make(tmp/'late',{b'a':b'alvo\nalvo\nalvo\n',b'z':b'alvo later\n'});late=c['package']/'objects'/sha(c['files'][b'z']);late.unlink();v,b=query(c,max_matches=1,code=3)
        e=expected(c,'alvo',1,'partial',['unresolved_files']);e['scan']['matching_lines_known']='3';validate(v,e);check(v['files'][-1]['reasons']==['object_missing'],'RD16.late_failure')
        mutation_results=[]
        for tag,change in [('text',lambda x:x['matches'][0].update(text='bad')),('offsets',lambda x:x['matches'][0].update(end_byte='1')),('revision',lambda x:x['matches'][0].update(content_sha256='0'*64)),('coverage',lambda x:x['scan'].update(status='complete')),('truncation',lambda x:x['scan'].update(matches_truncated=False))]:
            bad=copy.deepcopy(v);change(bad)
            try:validate(bad,e)
            except AssertionError as ex:check(str(ex)=='RD24.'+tag,'RD24.runtime.'+tag);mutation_results.append(dict(mutation=tag,rejection=str(ex)))
            else:raise AssertionError('runtime mutation accepted '+tag)
        c['files'].pop(b'z');(c['package']/'objects'/sha(c['files'][b'a'])).unlink();v,b=query(c,code=1);check(v['scan']['status']=='unavailable' and not v['matches'],'RD09.unavailable')
        # Every EOF distinction, including a prefix at s+1, is retained.
        for label,new in [('same',b'xxxx\n'),('short',b'a'),('long',b'alvo\nxx')]:
            c=make(tmp/('tamper-'+label),{b'a':b'alvo\n'});(c['package']/'objects'/sha(b'alvo\n')).write_bytes(new);v,b=query(c,code=1);f=v['files'][0];ck=f['check'];check(not v['matches'],'RD10.no_unverified_hit');check(ck['size_check']==(label=='same'),'RD10.size.'+label)
            if label=='long':check(ck['actual_revision'] is None and ck['hash_check'] is None,'RD10.prefix_no_hash')
            else:check(ck['actual_revision']==dict(content_sha256=sha(new),size_bytes=str(len(new))) and ck['hash_check'] is False,'RD10.EOF.'+label)
        # Static prohibited kinds at object/components/root; no outside bytes.
        for ki,kind in enumerate(['symlink','dangling','fifo','socket','objects_link','root_link']):
            c=make(tmp/('k'+str(ki)),{b'a':b'alvo\n'});obj=c['package']/'objects'/sha(b'alvo\n');outside=tmp/('external-'+kind);outside.write_bytes(b'alvo outside')
            listener=None
            if kind in ['objects_link','root_link']:
                target=c['package']/'objects' if kind=='objects_link' else c['package'];old=target.with_name(target.name+'-saved');target.rename(old);target.symlink_to(old,target_is_directory=True)
            else:
                obj.unlink()
                if kind=='symlink':obj.symlink_to(outside)
                elif kind=='dangling':obj.symlink_to(tmp/'absent')
                elif kind=='fifo':os.mkfifo(obj)
                else:listener=socket.socket(socket.AF_UNIX);listener.bind(str(obj))
            try:
                if kind=='root_link':error(c,'source_invalid')
                else:
                    v,b=query(c,code=1);check(v['files'][0]['reasons']==['source_invalid_kind'],'RD11.'+kind);check(v['resources']['object_bytes_read']=='0','RD11.no_read')
            finally:
                if listener:listener.close()
            check(outside.read_bytes()==b'alvo outside','RD11.external_unchanged')
        # Partial observation stays partial even when an observed object is verified.
        for content in [{b'a':b'alvo\n'},{}]:
            c=make(tmp/('partial-'+str(len(content))),content,partial=True);v,b=query(c,code=3 if content else 1);validate(v,expected(c,'alvo',status='partial' if content else 'unavailable',reasons=['snapshot_partial']));check(v['scan']['status']!='complete','RD13.partial')
        # Read budget is per path, s+1, never refunded, rejected big entries allow small later.
        c=make(tmp/'budget',{b'a':b'alvo\n',b'b':b'alvo\n',b'z':b''})
        for budget,states,reserved in [(6,['searched','unavailable','unavailable'],6),(7,['searched','unavailable','searched'],7),(13,['searched','searched','searched'],13)]:
            l=dict(defaults,max_total_object_bytes=str(budget));v,b=query(c,limits=l,code=0 if budget==13 else 3);check([f['state'] for f in v['files']]==states,'RD14.states');check(v['resources']['object_bytes_reserved']==str(reserved),'RD14.reserved')
        c=make(tmp/'smaller',{b'a':b'x'*50,b'z':b'alvo\n'});v,b=query(c,limits=dict(defaults,max_total_object_bytes='6'),code=3);check([f['state'] for f in v['files']]==['unavailable','searched'],'RD14.small_later')
        # File, line and query byte boundaries.
        for n in [4096,4097]:
            c=make(tmp/f'line-{n}',{b'a':b'alvo\n'+b'x'*(n-1)+b'\n'});v,b=query(c,code=0 if n==4096 else 1);check(len(v['matches'])==(1 if n==4096 else 0),'RD15.whole_file_preflight')
            if n==4097:check(v['files'][0]['reasons']==['line_bytes_limit'] and v['files'][0]['check']['state']=='verified','RD15.line_reason')
        c=make(tmp/'query',{b'a':('é'*128+'\n').encode()});v,b=query(c,'é'*128);validate(v,expected(c,'é'*128));error(c,'invalid_arguments',code=2,q='é'*128+'a')
        v,b=query(c,'a',limits=dict(defaults,max_file_bytes='1'),code=1);check(v['files'][0]['reasons']==['file_bytes_limit'] and v['resources']['object_bytes_reserved']=='0','RD14.file_limit')
        # Empty universe descriptor is still validated, including valid but wrong binding.
        c=make(tmp/'empty',{});v,b=query(c);check(v['scan']['status']=='complete','RD21.empty_complete');desc=c['package']/'source.json';original=desc.read_bytes()
        for raw,err,exit in [(b'{}','source_invalid',1),(original.replace(b'"source_schema_version":"1"',b'"source_schema_version":"1","extra":"x"'),'source_invalid',1),(canonical(dict(json.loads(original),snapshot_file_sha256='0'*64)),'incompatible_inputs',2)]:desc.write_bytes(raw);error(c,err,code=exit)
        desc.write_bytes(original)
        # Strict snapshot transport and profile domains.
        mraw=c['manifest'].read_bytes()
        for raw in [b'{"entries":[],"entries":[]}',mraw[:-1]+b',"extra":"x"}',mraw.replace(b'"observed_files":"0"',b'"observed_files":0')]:c['manifest'].write_bytes(raw);error(c,'input_invalid')
        c['manifest'].write_bytes(mraw)
        for k,val in [('max_records',0),('max_records','0'),('max_records','01'),('max_records','18446744073709551616'),('max_records','-1'),('max_records',1.5),('extra','1')]:
            l=dict(defaults);l[k]=val;error(c,'invalid_arguments',code=2,limits=l)
        error(c,'invalid_arguments',code=2,limits={})
        rawprofile=c['package'].parent/'bad-limits.json'
        for raw in [b'{"max_records":"1","max_records":"2"}', b'['*65+b'0'+b']'*65, b' '*65537]:
            rawprofile.write_bytes(raw);error(c,'invalid_arguments',code=2,extra=['--limits',rawprofile])
        query(c,limits=dict(defaults,max_manifest_bytes=str(len(mraw)),max_source_descriptor_bytes=str(len(original))))
        for k,val in [('max_manifest_bytes' ,len(mraw)-1),('max_source_descriptor_bytes',len(original)-1),('max_json_depth',1)]:error(c,'input_limit',limits=dict(defaults,**{k:str(val)}))
        # Record cap includes directory coverage and issues. Valid snapshot has root coverage.
        c=make(tmp/'record-limit',{b'a':b'alvo'});error(c,'input_limit',limits=dict(defaults,max_records='1'))
        for args in [('--max-matches','-1'),('--max-matches','1.5'),('--max-matches','18446744073709551616'),('--deadline-ms','0')]:error(c,'invalid_arguments',code=2,extra=args)
        for q in ['','a\nb','a\rb']:error(c,'invalid_arguments',code=2,q=q)
        # Cap exact equality and cap-1; no hidden trimming, fixed envelope exempt.
        v,b=query(c);n=len(b);v2,b2=query(c,limits=dict(defaults,max_output_bytes=str(n)))
        # Limits and logical peak strings may change serialized length: solve fixed point independently.
        cap=len(b2)
        for _ in range(8):
            v3,b3=query(c,limits=dict(defaults,max_output_bytes=str(max(cap,n))))
            if len(b3)==cap:break
            cap=len(b3)
        # Obtain exact output size without depending on successful production under a too-small cap.
        vv=copy.deepcopy(v);vv['limits']['max_output_bytes']=str(cap)
        sm=8*len(c['manifest'].read_bytes())+4096*2+65536+8*cap;s=sm+8*len((c['package']/'source.json').read_bytes());vv['resources']['logical_memory_peak']=str(s+65536+2*5)
        cap=len(canonical(vv));v,b=query(c,limits=dict(defaults,max_output_bytes=str(cap)));check(len(b)==cap,'RD22.exact_cap');error(c,'output_limit',limits=dict(defaults,max_output_bytes=str(cap-1)));error(c,'output_limit',limits=dict(defaults,max_output_bytes='1'))
        for q in ['d0','init','search']:query(c,q)
        query(c,as_json=False)
        # Full CLI flow, immutable inputs, absent live tree, revision separation.
        c0=make(tmp/'revision0',{b'a':b'\xef\xbb\xbfalvo\r\n'});c1=make(tmp/'revision1',{b'a':b'alvo\n'},identity=c0['tree']/'.compass/workspace.json')
        for c in [c0,c1]:
            inv=inventory(c['package']);v,b=query(c);validate(v,expected(c,'alvo'));shutil.rmtree(c['tree']);v2,b2=query(c);check(b==b2,'RD20.no_live_tree');check(inventory(c['package'])==inv,'RD20.package_unchanged')
        d0,_=invoke(['d0','--baseline',c0['manifest'],'--current',c1['manifest'],'--baseline-source',c0['package'],'--current-source',c1['package'],'--json'])
        check(d0['measurement']['value']['numerator']=='0','RD23.D0_zero_does_not_mix_revisions')
        other=dict(c0,package=c1['package']);error(other,'incompatible_inputs',code=2)
        # Persist representative raw inputs; all stdout/error objects above are recoverable.
        inputs={name:blob(pathlib.Path(path).read_bytes()) for name,path in [('snapshot',c0['manifest']),('descriptor',c0['package']/'source.json')]}
        # Corpus exactly as R1/S0 recipe with BASE only at j=31.
        files={f'dir-{i//10:03d}/file-{i:04d}.txt'.encode():''.join((f'F{i:04d}:L{j:02d}:'+('BASE' if j==31 else 'NONE')).ljust(63,'x')+'\n' for j in range(64)).encode() for i in range(1000)}
        check(sum(map(len,files.values()))==4096000,'measurement.recipe_bytes');c=make(tmp/'measurement',files);shutil.rmtree(c['tree']);samples=[];outputs=[]
        args=[str(binary),'search','--snapshot',str(c['manifest']),'--source',str(c['package']),'--query','BASE','--max-matches','20','--json']
        for i in range(3):
            measure=tmp/f'time-{i}';command=['/usr/bin/time','-q','-f','%U %S %M %e %x','-o',str(measure),*args];start=time.perf_counter_ns();p=subprocess.run(command,capture_output=True,timeout=60);wall=time.perf_counter_ns()-start
            check(p.returncode==0 and not p.stderr,'measurement.exit');v=json.loads(p.stdout);validate(v,expected(c,'BASE'));check(v['resources']['object_bytes_read']=='4096000' and v['resources']['object_bytes_reserved']=='4097000','measurement.ledger')
            fields=measure.read_text().split();samples.append(dict(command=command,raw=measure.read_text(),user_s=fields[0],system_s=fields[1],max_rss_kib=int(fields[2]),elapsed_s=fields[3],exit=int(fields[4]),wall_ns=wall,stdout=blob(p.stdout),ledger=v['resources']));outputs.append(p.stdout)
        check(outputs[0]==outputs[1]==outputs[2],'measurement.repeatability')
        save('resource-profile.json',dict(binary_sha256=before,producer_build=v['producer_build'],samples=samples,environment=dict(uname=platform.uname()._asdict(),uid=os.getuid(),cpu=os.cpu_count(),cache='uncontrolled; no flush; sequential; no compilation',time_version=subprocess.check_output(['/usr/bin/time','--version']).decode(),disk_logical_bytes=sum(p.stat().st_size for p in c['package'].rglob('*') if p.is_file()),disk_allocated_bytes=sum(p.stat().st_blocks*512 for p in c['package'].rglob('*') if p.is_file()),manifest_bytes=c['manifest'].stat().st_size),limitations=['descriptive local measurements; no SLO','logical ledger is not RSS or allocator control']))
    check(sha(binary.read_bytes())==before,'binary_stability')
    save('integration.json',dict(binary_sha256=before,checks=checks,commands=records,inputs=inputs,runtime_executed=True,mutations=mutation_results))
    print('RD_CAMPAIGN_OK',len(checks),'checks',len(records),'CLI commands; three native measurements')

def regressions(binary,cache):
    binary=pathlib.Path(binary).resolve(strict=True);before=sha(binary.read_bytes());runs=[]
    for script in ['verification/reference/acceptance.py','verification/reference/mutations.py']:
        cmd=[sys.executable,str(ROOT/script),'--binary',str(binary)];p=subprocess.run(cmd,capture_output=True,timeout=600,env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1'));runs.append(dict(command=cmd,exit=p.returncode,stdout=blob(p.stdout),stderr=blob(p.stderr)));require(p.returncode==0,'Foundation regression '+script)
    spec=importlib.util.spec_from_file_location('rd_historical_mount',ROOT/'verification/d0-i1/run-v1.py');adapter=importlib.util.module_from_spec(spec);spec.loader.exec_module(adapter);adapter.verify()
    with tempfile.TemporaryDirectory(prefix='compass-rd-regressions-') as tmp:
        lab=adapter.mount(pathlib.Path(tmp))
        for name in ['acceptance.py','mutations.py']:
            cmd=[sys.executable,str(lab/'verification/d0'/name),'--binary',str(binary)]
            if cache:cmd+=['--cached-dependencies',str(cache)]
            p=subprocess.run(cmd,capture_output=True,timeout=600,env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1'));result=lab/'verification/d0/results'/('acceptance.json' if name=='acceptance.py' else 'mutations.json')
            runs.append(dict(command=cmd,exit=p.returncode,stdout=blob(p.stdout),stderr=blob(p.stderr),full_result=json.loads(result.read_bytes()) if result.exists() else None))
    save('regressions.json',dict(binary_before=before,binary_after=sha(binary.read_bytes()),runs=runs))
    require(all(x['exit']==0 for x in runs),'D0 regression failed');require(sha(binary.read_bytes())==before,'regression binary changed');print('REGRESSIONS_OK')

def main():
    p=argparse.ArgumentParser();p.add_argument('--self-test',action='store_true');p.add_argument('--binary',type=pathlib.Path);p.add_argument('--regressions',action='store_true');p.add_argument('--cached-dependencies',type=pathlib.Path);a=p.parse_args()
    if a.self_test:selftest()
    elif a.binary:
        if a.regressions:regressions(a.binary,a.cached_dependencies)
        else:selftest();campaign(a.binary)
    else:p.error('--self-test or --binary required')
if __name__=='__main__':main()
