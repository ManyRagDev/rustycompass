#!/usr/bin/env python3
"""Independent public CLI and real stdio MCP obligation-pack acceptance.

Synthetic manual controls are frozen in controls.json; no engine imports and no
private evaluation source. Source bytes, revisions and coverage checked locally.
"""
import argparse, base64, copy, hashlib, json, os, pathlib, selectors, shutil, subprocess, tempfile, time, sys
sys.dont_write_bytecode=True
HERE=pathlib.Path(__file__).resolve().parent
CONTROL=json.loads((HERE/'controls.json').read_text())

def check(ok, message):
    if not ok: raise AssertionError(message)
def canonical(v): return json.dumps(v,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()
def digest(raw): return hashlib.sha256(raw).hexdigest()
def key(path): return base64.urlsafe_b64encode(path.encode()).rstrip(b'=').decode()
def decode(path): return base64.urlsafe_b64decode(path+'='*(-len(path)%4)).decode()
def selector(symbol): return dict(name=symbol['name'],path_key=key(symbol['path']),**({'scope':symbol['scope']} if 'scope' in symbol else {}))
def definition(symbol,identity): return {'id':identity,'kind':'symbol_definition','symbol':selector(symbol)}
def line_range(raw,start,end):
    lines=raw.splitlines(keepends=True);a=sum(map(len,lines[:start-1]));b=sum(map(len,lines[:end]));return a,a+len(raw[a:b].rstrip(b'\r\n'))
def covered(intervals,a,b):
    cursor=a
    for x,y in sorted(intervals):
        if x<=cursor:cursor=max(cursor,y)
    return cursor>=b

def verify_items(pack, sources, generation):
    check(pack['generation']==generation,'pack generation drift')
    check(pack['payload_bytes']<=pack['budget_bytes'],'pack budget exceeded')
    intervals={};seen=set();redundant=0;delivered=0
    for item in pack['items']:
        path=decode(item['path_key']);raw=sources[path];a=int(item['start_byte']);b=int(item['end_byte'])
        check(0<=a<=b<=len(raw),'source range outside original')
        check(int(item['start_line'])==raw[:a].count(b'\n')+1,'start line mismatches source bytes')
        check(int(item['end_line'])==raw[:max(a,b-1)].count(b'\n')+1,'end line mismatches source bytes')
        check(raw[a:b].decode()==item['text'],'source text mismatch')
        check(digest(raw)==item['content_sha256'],'source hash mismatch')
        check(item['verification']=='full_object_sha256','unverified item bytes')
        check(item['span_id'] not in seen,'duplicate span ID');seen.add(item['span_id'])
        prior=intervals.setdefault(path,[])
        # Redundancy is measured in source bytes, separately from JSON overhead.
        points=sorted({a,b,*[max(a,min(b,p)) for pair in prior for p in pair]})
        redundant+=sum(y-x for x,y in zip(points,points[1:]) if covered(prior,x,y))
        delivered+=b-a;prior.append((a,b))
    if 'completion' in pack:
        check(pack['completion']['scope']=='current_pack','completion scope not current pack')
        if pack['completion']['status']=='SUFFICIENT_EVIDENCE_ACQUIRED':
            check(all(o['status']=='satisfied' and o['witnesses'] for o in pack['completion']['obligations']),'sufficient with unsatisfied/unwitnessed obligation')
            check(not pack['completion']['interpretation_pending'],'sufficient with interpretation pending')
        for omitted in pack['omitted']:
            check(set(omitted)=={'path_key','content_sha256','start_byte','end_byte','reason'},'omitted reference shape')
            path=decode(omitted['path_key']);raw=sources[path];a=int(omitted['start_byte']);b=int(omitted['end_byte'])
            check(digest(raw)==omitted['content_sha256'] and 0<=a<b<=len(raw),'invalid omitted source identity/range')
            check(all(max(a,x)>=min(b,y) for x,y in intervals.get(path,[])),'omitted bytes overlap delivered bytes')
        spans={i['span_id']:i for i in pack['items']}
        for obligation in pack['completion']['obligations']:
            for witness in obligation['witnesses']:
                check(witness['span_id'] in spans,'witness references absent current span')
                item=spans[witness['span_id']];a=int(witness['start_byte']);b=int(witness['end_byte'])
                check(int(item['start_byte'])<=a<=b<=int(item['end_byte']),'witness outside delivered span')
    return intervals,dict(source_bytes=delivered,redundant_source_bytes=redundant,items=len(pack['items']))

def symbol_covered(symbol,sources,intervals):
    raw=sources[symbol['path']];a,b=line_range(raw,symbol['start_line'],symbol['end_line'])
    # Indentation precedes the syntactic declaration, independently determined
    # from the manually selected first line; the complete declaration/body and
    # every interior indentation byte remain required. Literal witnesses use
    # their full original lines instead.
    first=raw[a:].split(b'\n',1)[0];a+=len(first)-len(first.lstrip(b' \t'))
    return covered(intervals.get(symbol['path'],[]),a,b)
def literal_covered(literal,sources,intervals):
    raw=sources[literal['path']];a,b=line_range(raw,literal['start_line'],literal['start_line'])
    check(literal['literal'].encode() in raw[a:b],'manual literal expectation wrong')
    return covered(intervals.get(literal['path'],[]),a,b)

class Lab:
    def __init__(self,binary):self.binary=pathlib.Path(binary).resolve();self.measurements=[]
    def cli(self,*args,success=True):
        with tempfile.NamedTemporaryFile() as metric:
            begin=time.perf_counter_ns()
            p=subprocess.run(['/usr/bin/time','-f','%M','-o',metric.name,str(self.binary),*map(str,args),'--json'],capture_output=True,timeout=120)
            elapsed=time.perf_counter_ns()-begin
            rss=pathlib.Path(metric.name).read_text().strip().splitlines()[-1]
            self.measurements.append(dict(operation=list(map(str,args[:2])),wall_ns=elapsed,peak_rss_kib=int(rss),stdout_bytes=len(p.stdout)))
        check(success is None or (p.returncode==0 if success else p.returncode!=0),f'CLI exit {p.returncode}: {p.stdout[:1200]!r} {p.stderr[:300]!r}')
        v=json.loads(p.stdout)
        if v.get('report_type')=='evidence_pack':check(v['payload_bytes']==len(p.stdout.rstrip(b'\n')),'canonical payload count')
        return v

class MCP:
    def __init__(self,binary,config):
        self.p=subprocess.Popen([str(binary),'mcp','--config',str(config)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,bufsize=0)
        self.selector=selectors.DefaultSelector();self.selector.register(self.p.stdout,selectors.EVENT_READ);self.id=0;self.buffer=b'';self.calls=0
        self.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'independent-acquisition-control','version':'1'}})
        self.send({'jsonrpc':'2.0','method':'notifications/initialized'})
    def send(self,v):self.p.stdin.write(canonical(v)+b'\n');self.p.stdin.flush()
    def request(self,method,params):
        self.id+=1;self.send({'jsonrpc':'2.0','id':self.id,'method':method,'params':params});end=time.monotonic()+120
        while b'\n' not in self.buffer:
            check(self.selector.select(max(0,end-time.monotonic())),'MCP timeout');raw=os.read(self.p.stdout.fileno(),65536);check(raw,'MCP EOF');self.buffer+=raw
        raw,self.buffer=self.buffer.split(b'\n',1);v=json.loads(raw);check(v.get('id')==self.id and 'error' not in v,f'MCP error {v}');return v['result']
    def tool(self,args,error=False):
        self.calls+=1;r=self.request('tools/call',{'name':'compass_retrieve','arguments':args})
        check(bool(r['isError'])==error,f'MCP tool error {r}')
        check(json.loads(r['content'][0]['text'])==r['structuredContent'],'MCP representation mismatch')
        return r['structuredContent']
    def close(self):
        self.p.stdin.close();check(self.p.wait(timeout=10)==0,'MCP shutdown');check(not self.p.stderr.read(),'MCP stderr');self.selector.close()

def contract(obligations,pending=None):return {'schema_version':'1','intent':'Acquire the exact requested source evidence.','obligations':obligations,'interpretation_pending':pending or []}
def cases(control):
    syms=control['symbols'];lit=control['literal'];rel=control['relation']
    literal={'id':'literal','kind':'literal_occurrence','literal':lit['literal'],'path_key':key(lit['path'])}
    relation={'id':'relation','kind':'resolved_relation','source':{'name':rel['source_name'],'path_key':key(rel['source_path'])},'target':{'name':rel['target_name'],'path_key':key(rel['target_path'])},'relation':rel['relation']}
    positives=[(f'definition-{i}',contract([definition(s,'definition')]),[s],[]) for i,s in enumerate(syms)]
    positives += [('literal',contract([literal]),[],[lit]),('direct-call',contract([relation]),[dict(s,end_line=s['start_line']) for s in syms[:2]],[dict(path=rel['source_path'],literal=rel['witness'],start_line=rel['witness_line'])]),('all-mandatory',contract([*[definition(s,f'definition-{i}') for i,s in enumerate(syms)],literal,relation]),syms,[lit])]
    negative=[('missing',contract([{'id':'missing','kind':'symbol_definition','symbol':{'name':control['missing_symbol']}}])),('ambiguous',contract([{'id':'ambiguous','kind':'symbol_definition','symbol':selector(control['ambiguous'])}])),('reverse-call',contract([dict(relation,source=relation['target'],target=relation['source'])])),('dynamic-call',contract([dict(relation,source={'name':control['dynamic']['source'],'path_key':key(control['dynamic']['path'])})])),('missing-literal',contract([dict(literal,literal='NEVER_PRESENT_LITERAL')])),('interpretation-pending',contract([literal],['Choose which behavior should change.']))]
    positives += [('scoped-homonym',contract([definition(control['scoped_symbol'],'scoped')]),[control['scoped_symbol']],[]),('sixteen',contract([definition(syms[0],f'same-symbol-{i}') for i in range(16)]),[syms[0]],[]),('symbol-scoped-literal',contract([dict(id='symbol-literal',kind='literal_occurrence',literal=lit['literal'],symbol=selector(syms[0]))]),[],[lit])]
    return positives,negative

def selftest():
    frozen=json.loads((HERE/'frozen.json').read_text())
    check(digest((HERE/'controls.json').read_bytes())==frozen['controls_sha256'],'frozen controls modified')
    count=0
    for split,c in CONTROL['splits'].items():
        sources={p.name:p.read_bytes() for p in (HERE/'fixtures'/split).iterdir()}
        for name,expected in c['file_sha256'].items():check(digest(sources[name])==expected,'fixture frozen hash')
        for s in [*c['symbols'],c['scoped_symbol']]:
            a,b=line_range(sources[s['path']],s['start_line'],s['end_line']);check(s['name'].encode() in sources[s['path']][a:b],'manual definition range');check(s['end_line']<=len(sources[s['path']].splitlines()),'manual range beyond file')
            check(symbol_covered(s,sources,{s['path']:[(a,b)]}),'complete span rejected');check(not symbol_covered(s,sources,{s['path']:[(a,b-1)]}),'truncated span accepted');count+=1
        check(literal_covered(c['literal'],sources,{c['literal']['path']:[(0,len(sources[c['literal']['path']]))]}),'literal oracle')
    return {'manual_symbols':count,'truncated_mutations_rejected':count,'fixture_hashes':'verified'}

def run(binary):
    lab=Lab(binary);results=[];mcp_tool_calls=0;binary_hash=digest(lab.binary.read_bytes())
    with tempfile.TemporaryDirectory(prefix='compass-acquisition-') as td:
      tmp=pathlib.Path(td)
      for split,c in CONTROL['splits'].items():
        tree=tmp/split;shutil.copytree(HERE/'fixtures'/split,tree);sources={p.name:p.read_bytes() for p in tree.iterdir()};store=tmp/(split+'-store')
        prepared=lab.cli('collection','prepare','--root',tree,'--store',store);generation=prepared['generation']
        cfg=tmp/(split+'-mcp.json');cfg.write_bytes(canonical({'sources':{'control':{'collection':str(store)}}}));client=MCP(lab.binary,cfg)
        positive,negative=cases(c)
        def retrieve(ctr,budget=65536,gen=generation,mcp=True,allow_budget_refusal=False,mode='literal',cursor=None):
            file=tmp/'contract.json';file.write_bytes(canonical(ctr));args=['retrieve','--store',store,'--generation',gen,'--query',c['query'],'--mode',mode,'--expansion','none','--budget-bytes',budget,'--contract',file]
            if cursor is not None:args+=['--cursor',cursor]
            v=lab.cli(*args,success=None if allow_budget_refusal else True)
            if 'error' in v:
                check(allow_budget_refusal and v['error']['code']=='invalid_input' and 'budget' in v['error']['message'],'unexpected budget refusal')
                if mcp:
                    actual=client.tool({'source':'control','generation':gen,'query':c['query'],'mode':mode,'expansion':'none','budget_bytes':budget,'contract':ctr,**({'cursor':cursor} if cursor is not None else {})},error=True)
                    check(actual==v,'CLI/MCP budget refusal mismatch')
                return v
            obligations={o['id']:o for o in ctr['obligations']}
            check({o['id'] for o in v['completion']['obligations']}==set(obligations),'completion obligation identity mismatch')
            for outcome in v['completion']['obligations']:
                spec=obligations[outcome['id']]
                if outcome['status']=='satisfied' and spec['kind']=='resolved_relation':
                    resolution=outcome['resolution'];check(resolution['relation']==spec['relation'],'relation direction/type mismatch')
                    for endpoint in ['source','target']:
                        expected=spec[endpoint];actual=resolution[endpoint]
                        check(actual['node_id']==expected['node_id'] if 'node_id' in expected else actual['name']==expected['name'] and actual['path_key']==expected['path_key'],'resolved relation endpoint mismatch')
            if mcp:
                actual=client.tool({'source':'control','generation':gen,'query':c['query'],'mode':mode,'expansion':'none','budget_bytes':budget,'contract':ctr,**({'cursor':cursor} if cursor is not None else {})});check(actual==v,'CLI/MCP contract pack mismatch')
            return v
        # Supplemental import control: expected direction follows the authored
        # `from engine/flow import leaf/finish` fixture, never returned edges.
        import_source='bridge.py' if split=='dev' else 'adapter.py'
        import_target=c['symbols'][0]['path']
        node_ids={}
        for filename in [import_source,import_target]:
            discovery=lab.cli('neighbors','--store',store,'--generation',generation,'--query',filename)
            matches=[n for n in discovery['nodes'] if n['kind']=='file' and n['path_key']==key(filename)]
            check(len(matches)==1,'file endpoint public discovery')
            node_ids[filename]=matches[0]['id']
        import_spec={'id':'import','kind':'resolved_relation','source':{'node_id':node_ids[import_source]},'target':{'node_id':node_ids[import_target]},'relation':'imports'}
        import_witness={'path':import_source,'literal':'from '+import_target.removesuffix('.py')+' import '+c['symbols'][0]['name'],'start_line':1}
        file_anchors=[dict(path=name,start_line=1,end_line=1) for name in [import_source,import_target]]
        positive.append(('direct-import',contract([import_spec]),file_anchors,[import_witness]))
        negative.append(('reverse-import',contract([dict(import_spec,source=import_spec['target'],target=import_spec['source'])])))
        for name,ctr,symbols,literals in positive:
            p=retrieve(ctr);intervals,metrics=verify_items(p,sources,generation)
            actual_coverage=all(symbol_covered(s,sources,intervals) for s in symbols) and all(literal_covered(l,sources,intervals) for l in literals)
            check(actual_coverage,f'{split}/{name}: first pack omits mandatory source')
            check(p['completion']['status']=='SUFFICIENT_EVIDENCE_ACQUIRED',f'{split}/{name}: complete evidence not sufficient: {p["completion"]}')
            check(all(o['status']=='satisfied' for o in p['completion']['obligations']),f'{name}: obligation mismatch')
            results.append(dict(split=split,case=name,status=p['completion']['status'],first_pack_coverage=1,payload_bytes=p['payload_bytes'],wall_ns=lab.measurements[-1]['wall_ns'],peak_rss_kib=lab.measurements[-1]['peak_rss_kib'],**metrics))
        for name,ctr in negative:
            p=retrieve(ctr);verify_items(p,sources,generation);check(p['completion']['status']!='SUFFICIENT_EVIDENCE_ACQUIRED',f'false sufficient {split}/{name}')
            results.append(dict(split=split,case=name,status=p['completion']['status'],false_sufficient=False))
        # Current-pack sufficiency may never borrow earlier calls at smaller budgets.
        all_ctr=next(case[1] for case in positive if case[0]=='all-mandatory')
        for budget in [3000,8000,16384]:
            p=retrieve(all_ctr,budget,allow_budget_refusal=True)
            if 'error' in p:
                results.append(dict(split=split,case=f'budget-{budget}',status='budget_refused',error=p['error']));continue
            intervals,metrics=verify_items(p,sources,generation)
            covered_count=sum(symbol_covered(s,sources,intervals) for s in c['symbols'])+literal_covered(c['literal'],sources,intervals)
            if covered_count<len(c['symbols'])+1:check(p['completion']['status']!='SUFFICIENT_EVIDENCE_ACQUIRED','budget truncation false sufficient')
            results.append(dict(split=split,case=f'budget-{budget}',status=p['completion']['status'],wall_ns=lab.measurements[-1]['wall_ns'],peak_rss_kib=lab.measurements[-1]['peak_rss_kib'],mandatory_source_coverage=covered_count/(len(c['symbols'])+1),payload_bytes=p['payload_bytes'],**metrics))
        first=retrieve(all_ctr,8000)
        check(first['next_cursor'] is not None,'partial first pack lacks continuation')
        second=retrieve(all_ctr,8000,cursor=first['next_cursor'])
        intervals,metrics=verify_items(second,sources,generation)
        current_complete=all(symbol_covered(s,sources,intervals) for s in c['symbols']) and literal_covered(c['literal'],sources,intervals)
        if not current_complete:check(second['completion']['status']!='SUFFICIENT_EVIDENCE_ACQUIRED','continuation borrowed previous-page evidence')
        results.append(dict(split=split,case='continuation-current-pack-only',status=second['completion']['status'],current_complete=current_complete,payload_bytes=second['payload_bytes'],**metrics))
        hybrid=retrieve(all_ctr,mode='hybrid')
        intervals,metrics=verify_items(hybrid,sources,generation)
        check(hybrid['completion']['status']=='SUFFICIENT_EVIDENCE_ACQUIRED','hybrid all-fit not sufficient')
        check(all(symbol_covered(s,sources,intervals) for s in c['symbols']) and literal_covered(c['literal'],sources,intervals),'hybrid omitted mandatory bytes')
        results.append(dict(split=split,case='hybrid-all-mandatory',status=hybrid['completion']['status'],payload_bytes=hybrid['payload_bytes'],wall_ns=lab.measurements[-1]['wall_ns'],peak_rss_kib=lab.measurements[-1]['peak_rss_kib'],**metrics))
        normal=lab.cli('retrieve','--store',store,'--generation',generation,'--query',c['query'],'--mode','hybrid','--expansion','none','--budget-bytes',65536)
        intervals,metrics=verify_legacy(normal,sources,generation)
        results.append(dict(split=split,case='normal-hybrid-baseline',payload_bytes=normal['payload_bytes'],symbol_source_coverage=sum(symbol_covered(s,sources,intervals) for s in c['symbols'])/len(c['symbols']),wall_ns=lab.measurements[-1]['wall_ns'],peak_rss_kib=lab.measurements[-1]['peak_rss_kib'],**metrics))
        boundary=contract([definition(c['symbols'][0],'boundary')])
        boundary['intent']+='x'*(16384-len(canonical(boundary)))
        check(len(canonical(boundary))==16384,'canonical contract boundary construction')
        boundary_pack=retrieve(boundary);verify_items(boundary_pack,sources,generation)
        check(boundary_pack['completion']['status']=='SUFFICIENT_EVIDENCE_ACQUIRED','exact16KiB contract rejected')
        results.append(dict(split=split,case='canonical-contract-exact16384',accepted=True))
        # Closed input limits, independently generated; do not loosen accepted limits.
        for label,ctr in [('zero',contract([])),('seventeen',contract([definition(c['symbols'][0],str(i)) for i in range(17)])),('oversized',dict(all_ctr,intent='x'*16385)),('duplicate-id',contract([definition(c['symbols'][0],'dup')]*2)),('exact16385',dict(boundary,intent=boundary['intent']+'x'))]:
            f=tmp/'invalid.json';f.write_bytes(canonical(ctr));lab.cli('retrieve','--store',store,'--query',c['query'],'--contract',f,success=False)
            client.tool({'source':'control','query':c['query'],'contract':ctr},error=True)
            results.append(dict(split=split,case='invalid-'+label,rejected=True))
        # Paired no-contract baseline on the identical query/budgets; efficacy is not inferred.
        for budget in [3000,8000,16384,65536]:
            normal=lab.cli('retrieve','--store',store,'--generation',generation,'--query',c['query'],'--mode','literal','--expansion','none','--budget-bytes',budget)
            intervals,metrics=verify_legacy(normal,sources,generation)
            observed=lab.measurements[-1]
            results.append(dict(split=split,case=f'normal-baseline-{budget}',payload_bytes=normal['payload_bytes'],wall_ns=observed['wall_ns'],peak_rss_kib=observed['peak_rss_kib'],symbol_source_coverage=sum(symbol_covered(s,sources,intervals) for s in c['symbols'])/len(c['symbols']),**metrics))
        # Legacy path remains without a completion claim; preserve direct retrieval.
        legacy=lab.cli('retrieve','--store',store,'--query',c['query'],'--mode','literal','--budget-bytes',16384)
        check('completion' not in legacy,'no-contract completion changed');verify_legacy(legacy,sources,generation)
        old=retrieve(all_ctr);mcp_tool_calls+=client.calls;client.close()
        target=tree/c['update']['path'];target.write_bytes(target.read_bytes().replace(c['update']['old_literal'].encode(),c['update']['new_literal'].encode()))
        update=lab.cli('collection','update','--store',store);check(update['generation']!=generation,'generation did not advance')
        client=MCP(lab.binary,cfg) # restart plus immutable old generation
        preserved=retrieve(all_ctr);check(old==preserved,'old generation contract evidence changed after update/restart')
        stale=retrieve(contract([all_ctr['obligations'][-2]]),gen=update['generation']);check(stale['completion']['status']!='SUFFICIENT_EVIDENCE_ACQUIRED','new generation missing literal sufficient')
        shutil.rmtree(tree);offline=retrieve(positive[0][1]);verify_items(offline,sources,generation);mcp_tool_calls+=client.calls;client.close()
        results.append(dict(split=split,case='update-restart-old-generation-offline',preserved=True))
      # Supplemental precision discriminator authored from source syntax:
      # an unrelated same-line constant is outside the exported declaration.
      inline=tmp/'inline';inline.mkdir();declaration='export function tiny() { return 7; }'
      raw=(declaration+' const unrelated = "'+'z'*20000+'";\n').encode();(inline/'inline.ts').write_bytes(raw)
      store=tmp/'inline-store';generation=lab.cli('collection','prepare','--root',inline,'--store',store)['generation']
      cfg=tmp/'inline-mcp.json';cfg.write_bytes(canonical({'sources':{'inline':{'collection':str(store)}}}));client=MCP(lab.binary,cfg)
      for kind in ['symbol_definition','literal_occurrence']:
        obligation={'id':'inline','kind':kind,'symbol':{'name':'tiny','path_key':key('inline.ts')}}
        if kind=='literal_occurrence':obligation['literal']='return 7'
        ctr=contract([obligation]);file=tmp/'inline-contract.json';file.write_bytes(canonical(ctr))
        pack=lab.cli('retrieve','--store',store,'--generation',generation,'--query','tiny','--mode','literal','--budget-bytes',4096,'--contract',file)
        actual=client.tool({'source':'inline','generation':generation,'query':'tiny','mode':'literal','budget_bytes':4096,'contract':ctr});check(actual==pack,'inline CLI/MCP mismatch')
        intervals,metrics=verify_items(pack,{'inline.ts':raw},generation)
        if kind=='symbol_definition':
            check(pack['completion']['status']=='SUFFICIENT_EVIDENCE_ACQUIRED','tiny definition blocked by unrelated same-line source')
            check(covered(intervals.get('inline.ts',[]),0,len(declaration.encode())),'tiny exported declaration incomplete')
        else:check(pack['completion']['status']!='SUFFICIENT_EVIDENCE_ACQUIRED','literal full-line requirement falsely sufficient')
        results.append(dict(split='supplemental',case='inline-'+kind,status=pack['completion']['status'],payload_bytes=pack['payload_bytes'],**metrics))
      mcp_tool_calls+=client.calls;client.close()
    check(digest(lab.binary.read_bytes())==binary_hash,'binary changed during acceptance')
    return dict(binary_sha256=binary_hash,acceptance_sha256=digest((HERE/'acceptance.py').read_bytes()),fixture_freeze_sha256=digest((HERE/'frozen.json').read_bytes()),supplemental_fixture_sha256=digest(raw),mcp_tool_calls=mcp_tool_calls,controls_sha256=digest((HERE/'controls.json').read_bytes()),results=results,measurements=lab.measurements,limitations=['Synthetic DEV and heldout sources; no general task efficacy claim.','RSS from GNU time per CLI process; MCP latency/RSS not separately instrumented.','OS page cache uncontrolled; no paid LLM calls.'])

def verify_legacy(pack,sources,generation):
    compatible=copy.deepcopy(pack)
    for i,item in enumerate(compatible['items']):item.setdefault('span_id',str(i))
    return verify_items(compatible,sources,generation)

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--self-test',action='store_true');parser.add_argument('--binary',type=pathlib.Path);parser.add_argument('--output',type=pathlib.Path);args=parser.parse_args()
    report={'self_test':selftest()}
    if args.binary:report['campaign']=run(args.binary)
    elif not args.self_test:parser.error('provide --self-test or --binary')
    if args.output:args.output.write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({'self_test':report['self_test'],'cases':len(report.get('campaign',{}).get('results',[]))}))
