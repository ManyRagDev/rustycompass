#!/usr/bin/env python3
"""Independent public CLI/MCP controls. Expectations predate measured outputs."""
import argparse, base64, hashlib, json, os, pathlib, selectors, shutil, subprocess, tempfile, time
ROOT=pathlib.Path(__file__).resolve().parents[2]
CONTROL=json.loads((ROOT/'verification/recovery/controls.json').read_text())
MEASURES=[]
def canonical(v):
    # Payload length is checked against bytes emitted by the Rust canonical CLI;
    # JSON floats here are only decoded for semantic assertions.
    return json.dumps(v,ensure_ascii=False,separators=(',',':'),sort_keys=True).encode()
def sha(p): return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
def check(ok,message):
    if not ok: raise AssertionError(message)
def cli(binary,*args,success=True):
    with tempfile.NamedTemporaryFile() as metric:
        start=time.perf_counter()
        p=subprocess.run(['/usr/bin/time','-f','%M','-o',metric.name,str(binary),*map(str,args),'--json'],capture_output=True,timeout=300)
        duration=time.perf_counter()-start
        rss=pathlib.Path(metric.name).read_text().splitlines()[-1]
        MEASURES.append({'operation':' '.join(str(x) for x in args[:2]),'seconds':duration,'peak_rss_kib':int(rss),'stdout_bytes':len(p.stdout)})
    if success: check(p.returncode==0,f'CLI {args}: {p.returncode} {p.stdout[:1000]!r} {p.stderr[:500]!r}')
    else: check(p.returncode!=0,'expected rejected CLI request')
    value=json.loads(p.stdout)
    if value.get('report_type')=='evidence_pack': check(value['payload_bytes']==len(p.stdout.rstrip(b'\n')),'pack exact canonical length')
    return value
class MCP:
    def __init__(self,binary,config):
        self.p=subprocess.Popen([str(binary),'mcp','--config',str(config)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        self.selector=selectors.DefaultSelector();self.selector.register(self.p.stdout,selectors.EVENT_READ);self.id=0
        self.call('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'independent-recovery-verifier','version':'1'}})
        self.send({'jsonrpc':'2.0','method':'notifications/initialized'})
    def send(self,x): self.p.stdin.write(canonical(x)+b'\n');self.p.stdin.flush()
    def call(self,method,params):
        self.id+=1;self.send({'jsonrpc':'2.0','id':self.id,'method':method,'params':params})
        check(self.selector.select(120),'MCP response timeout')
        x=json.loads(self.p.stdout.readline());check(x.get('id')==self.id,f'wrong MCP id {x}')
        check('error' not in x,f'JSONRPC error {x}');return x['result']
    def tool(self,name,args,error=False):
        x=self.call('tools/call',{'name':name,'arguments':args})
        check(bool(x['isError'])==error,f'MCP {name}: {x}')
        check(json.loads(x['content'][0]['text'])==x['structuredContent'],'MCP representations disagree')
        return x['structuredContent']
    def close(self):
        self.p.stdin.close();check(self.p.wait(timeout=10)==0,'MCP did not close cleanly on EOF');check(not self.p.stderr.read(),'unexpected diagnostics')
def verify_items(pack,tree,generation):
    check(pack['generation']==generation,'pack changed generation')
    check(pack['payload_bytes']<=pack['budget_bytes'],'pack budget exceeded')
    for x in pack['items']:
        path=os.fsdecode(base64.urlsafe_b64decode(x['path_key']+'='*((-len(x['path_key']))%4)))
        raw=(tree/path).read_bytes();a=int(x['start_byte']);b=int(x['end_byte'])
        check(hashlib.sha256(raw).hexdigest()==x['content_sha256'],'wrong source hash')
        check(raw[a:b].decode()==x['text'],'pack bytes differ from original')
        check(x['verification']=='full_object_sha256','missing full verification')
def run(binary,model):
    results=[]
    with tempfile.TemporaryDirectory(prefix='compass-recovery-') as tmp:
      tmp=pathlib.Path(tmp)
      for split in ['dev','holdout']:
        tree=tmp/split;shutil.copytree(ROOT/'verification/recovery/fixtures'/split,tree)
        (tree/'.env').write_text('DO_NOT_CAPTURE=private_fixture\n')
        store=tmp/(split+'-store');control=CONTROL[split]
        prepared=cli(binary,'collection','prepare','--root',tree,'--store',store,'--semantic-model',model)
        generation=prepared['generation']
        files=cli(binary,'files','--store',store,'--limit',100)
        check(not any(x['path']=='.env' for x in files['results']),'secret exclusion changed')
        direct=cli(binary,'query','--store',store,'--mode','literal','--query',control['seed_query'])
        check(any(base64.urlsafe_b64decode(x['path_key']+'='*((-len(x['path_key']))%4)).decode()==control['direct'] for x in direct['results']),'seed not found')
        semantic=cli(binary,'query','--store',store,'--mode','semantic','--query',control['semantic_query'],'--limit',3)
        semantic_paths=[base64.urlsafe_b64decode(x['path_key']+'='*((-len(x['path_key']))%4)).decode() for x in semantic['results']]
        check(bool(set(semantic_paths)&set(control['semantic_relevant'])),'semantic relevant absent top3')
        variants={}
        for expansion in ['none','bfs','heat']:
            p=cli(binary,'retrieve','--store',store,'--query',control['seed_query'],'--mode','literal','--expansion',expansion,'--budget-bytes',16384)
            verify_items(p,tree,generation)
            paths={base64.urlsafe_b64decode(x['path_key']+'='*((-len(x['path_key']))%4)).decode() for x in p['items']}
            check(control['direct'] in paths,'direct seed lost in pack')
            if expansion=='heat': check(control['related'] in paths,'heat did not reach known relation')
            variants[expansion]={'files':sorted(paths),'payload_bytes':p['payload_bytes'],'expected_recall':len(paths&{control['direct'],control['related']})/2,'path_precision':len(paths&{control['direct'],control['related']})/max(1,len(paths))}
        config=tmp/(split+'-mcp.json');config.write_text(json.dumps({'sources':{'control':{'collection':str(store)}}}))
        client=MCP(binary,config)
        tools=client.call('tools/list',{})['tools'];check(len(tools)==6,'MCP tools missing')
        actual=client.tool('compass_query',{'source':'control','generation':generation,'query':control['seed_query'],'mode':'literal'})
        check(actual==direct,'CLI/MCP query not equivalent')
        for expansion in ['none','bfs','heat']:
            args={'source':'control','generation':generation,'query':control['seed_query'],'mode':'literal','expansion':expansion,'budget_bytes':16384}
            actual=client.tool('compass_retrieve',args);verify_items(actual,tree,generation)
            expected=cli(binary,'retrieve','--store',store,'--generation',generation,'--query',control['seed_query'],'--mode','literal','--expansion',expansion,'--budget-bytes',16384)
            check(actual==expected,'CLI/MCP pack not equivalent')
        key=direct['results'][0]['path_key']
        read=client.tool('compass_read',{'source':'control','generation':generation,'path_key':key,'max_lines':1})
        check(read['status']=='available','MCP managed read unavailable')
        if read['excerpt']['next_start_line']:
            continued=client.tool('compass_read',{'source':'control','generation':generation,'path_key':key,'start_line':read['excerpt']['next_start_line'],'max_lines':1})
            check(continued['snapshot']['snapshot_id']==generation,'read continuation revision changed')
        client.tool('compass_read',{'source':'control','path_key':key},error=True)
        client.tool('compass_files',{'source':'../../etc','query':''},error=True)
        client.tool('compass_query',{'source':'control','query':'x','path':'/etc/passwd'},error=True)
        client.tool('compass_query',{'source':'control','query':'x','offset':1},error=True)
        client.close()
        saved=tmp/(split+'-saved');shutil.copytree(tree,saved)
        source_file=tree/control['direct'];source_file.write_text(source_file.read_text().replace(control['seed_query'],'updatedMarker'))
        updated=cli(binary,'collection','update','--store',store)
        check(updated['generation']!=generation,'update did not publish generation')
        check(updated['derived']['semantic']['reused_files']>0,'unchanged semantic files not reused')
        old=cli(binary,'query','--store',store,'--generation',generation,'--mode','literal','--query',control['seed_query'])
        check(old['results']==direct['results'],'old revision changed')
        new=cli(binary,'query','--store',store,'--mode','literal','--query',control['seed_query'])
        check(not new['results'],'old match leaked into new generation')
        shutil.rmtree(tree)
        detached=cli(binary,'retrieve','--store',store,'--generation',generation,'--query',control['seed_query'],'--mode','literal','--expansion','heat')
        verify_items(detached,saved,generation)
        status=cli(binary,'collection','status','--store',store)
        results.append({'split':split,'generation':generation,'updated_generation':updated['generation'],'variants':variants,'semantic_top3':semantic_paths,'disk_bytes':status['disk_bytes'],'semantic_reused_files':updated['derived']['semantic']['reused_files'],'checks':'passed'})
    return {'binary_sha256':sha(binary),'controls_sha256':sha(ROOT/'verification/recovery/controls.json'),'results':results,'measurements':MEASURES,'measurement_conditions':'Separate CLI processes, uncontrolled OS page cache; peak RSS from /usr/bin/time. Synthetic fixtures, no efficacy generalization.'}
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=pathlib.Path,default=ROOT/'target/release/compass');p.add_argument('--model',type=pathlib.Path,default=ROOT/'.compass/models/multilingual-e5-small');p.add_argument('--output',type=pathlib.Path,default=ROOT/'verification/recovery/results.json');a=p.parse_args()
    result=run(a.binary.resolve(),a.model.resolve());a.output.write_text(json.dumps(result,indent=2,ensure_ascii=False)+'\n');print(json.dumps({'status':'passed','splits':len(result['results']),'measurements':len(MEASURES),'output':str(a.output)},ensure_ascii=False))
