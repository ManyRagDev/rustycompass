#!/usr/bin/env python3
"""Real local stdio MCP client against the executable; no host settings edited."""
import sys
sys.dont_write_bytecode=True
import argparse,base64,hashlib,importlib.util,json,os,selectors,subprocess,tempfile,time,signal
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('reading',ROOT/'verification/reading/acceptance.py');reading=importlib.util.module_from_spec(spec);spec.loader.exec_module(reading)

def require(ok,tag):
    if not ok:raise AssertionError(tag)
class Client:
    def __init__(self,binary,config,cwd=None):
        self.p=subprocess.Popen([str(binary),'mcp','--config',str(config)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,cwd=cwd,bufsize=0)
        self.sel=selectors.DefaultSelector();self.sel.register(self.p.stdout,selectors.EVENT_READ);self.events=[];self.nextid=1;self.buffer=b''
    def send(self,v):
        raw=json.dumps(v,ensure_ascii=False,separators=(',',':')).encode()+b'\n';self.p.stdin.write(raw);self.p.stdin.flush();self.events.append(dict(direction='client',message=v))
    def receive(self,timeout=5):
        end=time.monotonic()+timeout
        while b'\n' not in self.buffer:
            remaining=end-time.monotonic();require(remaining>0,'client timeout')
            require(bool(self.sel.select(remaining)),'client timeout')
            b=os.read(self.p.stdout.fileno(),65536);require(bool(b),'unexpected MCP EOF');self.buffer+=b
        raw,self.buffer=self.buffer.split(b'\n',1);v=json.loads(raw);self.events.append(dict(direction='server',message=v));return v
    def request(self,method,params=None):
        id=self.nextid;self.nextid+=1;v=dict(jsonrpc='2.0',id=id,method=method)
        if params is not None:v['params']=params
        self.send(v);r=self.receive();require(r.get('id')==id and r.get('jsonrpc')=='2.0','request response');return r
    def initialize(self):
        v=self.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='compass-verification-client',version='1')))
        require(v['result']['protocolVersion']=='2025-11-25','protocol negotiated');require('tools' in v['result']['capabilities'],'tools capability');self.send(dict(jsonrpc='2.0',method='notifications/initialized'))
    def call(self,name,arguments):
        r=self.request('tools/call',dict(name=name,arguments=arguments));require('result' in r,'tool result');v=r['result'];require(json.loads(v['content'][0]['text'])==v['structuredContent'],'structured and text equivalence');return v
    def close(self):
        self.p.stdin.close();self.p.wait(timeout=5);require(self.p.returncode==0,'server shutdown');require(not self.p.stderr.read(),'unexpected stderr');self.sel.close()

def run(binary):
    binary=Path(binary).resolve();lab=reading.Lab(binary);checks=[];events=[];before=reading.sha(binary.read_bytes())
    with tempfile.TemporaryDirectory(prefix='compass-mcp-client-') as td:
        root=Path(td);c=lab.make(root/'data',{b'a.txt':b'before\nalvo\nafter\ntail\n'})
        config=root/'mcp.json';config.write_text(json.dumps(dict(sources=dict(demo=dict(snapshot='data/snapshot.json',source='data/package')))))
        client=Client(binary,'mcp.json',cwd=root)
        require(client.request('tools/list')['error']['code']==-32002,'initialization required');client.initialize()
        listed=client.request('tools/list')['result']['tools'];require([x['name'] for x in listed]==['compass_search','compass_read'],'tool discovery')
        require(listed[0]['inputSchema']['properties']['source']['enum']==['demo'],'configured names')
        client.p.stdin.write(b'{invalid-json}\n');require(client.receive()['error']['code']==-32700,'parse error')
        client.send(dict(jsonrpc='2.0',id=None,method='ping'));require(client.receive()['error']['code']==-32600,'invalid id')
        require(client.request('ping')['result']=={},'ping');require(client.request('no/such/method')['error']['code']==-32601,'unknown method')
        require(client.request('tools/call',dict(name='shell',arguments={}))['error']['code']==-32602,'unknown tool')
        search=client.call('compass_search',dict(source='demo',query='alvo'));require(not search['isError'],'search success')
        cli,_=lab.invoke(['search','--snapshot',c['manifest'],'--source',c['package'],'--query','alvo','--deadline-ms','30000','--json']);require(search['structuredContent']==cli,'CLI search equivalence')
        hit=search['structuredContent']['matches'][0]
        read=client.call('compass_read',dict(source='demo',path_key=hit['path_key'],start_line='1',max_lines=3));require(not read['isError'],'read success')
        cli,_=lab.invoke(['read','--snapshot',c['manifest'],'--source',c['package'],'--path-key',hit['path_key'],'--start-line','1','--max-lines','3','--deadline-ms','30000','--json']);require(read['structuredContent']==cli,'CLI read equivalence')
        next_line=read['structuredContent']['excerpt']['next_start_line'];continued=client.call('compass_read',dict(source='demo',path_key=hit['path_key'],start_line=next_line,max_lines=3));require(continued['structuredContent']['excerpt']['text']=='tail\n','continuation')
        checks+=['lifecycle','discovery','ping','errors','search-read-continuation','CLI-equivalence']
        for name,args,code in [
            ('compass_search',dict(source='/etc',query='x'),'source_not_configured'),
            ('compass_search',dict(source='demo',query='x',snapshot='/etc/passwd'),'invalid_arguments'),
            ('compass_read',dict(source='demo',path_key='Li4'),'invalid_arguments'),
            ('compass_read',dict(source='demo',path_key=hit['path_key'],start_line='01'),'invalid_arguments'),
            ('compass_read',dict(source='demo',path_key=hit['path_key'],start_line=1),'invalid_arguments'),
            ('compass_read',dict(source='demo',path_key=hit['path_key'],max_lines=1001),'invalid_arguments')]:
            v=client.call(name,args);require(v['isError'] and v['structuredContent']['error']['code']==code,'argument boundary')
        obj=c['package']/'objects'/reading.sha(c['files'][b'a.txt']);obj.write_bytes(b'x'*len(c['files'][b'a.txt']))
        failed=client.call('compass_read',dict(source='demo',path_key=hit['path_key']));require(failed['isError'] and failed['structuredContent']['excerpt']['text'] is None,'failed object no text');checks+=['source-boundary','strict-arguments','unavailable']
        client.send(dict(jsonrpc='2.0',method='notifications/cancelled',params=dict(requestId='unknown')))
        require(client.request('ping')['result']=={},'unknown cancellation ignored')
        client.close();events+=client.events
        # Long scanning input makes a real cancellation observable. Pipeline cancel
        # immediately after the call and then ping; no tool response is expected.
        files={f'f{i}'.encode():(b'x'*4095+b'\n')*1024 for i in range(4)}
        c=lab.make(root/'large',files);config.write_text(json.dumps(dict(sources=dict(large=dict(snapshot=str(c['manifest']),source=str(c['package']))))))
        client=Client(binary,config);client.initialize()
        client.send(dict(jsonrpc='2.0',id='slow',method='tools/call',params=dict(name='compass_search',arguments=dict(source='large',query='z'*256))))
        client.send(dict(jsonrpc='2.0',method='notifications/cancelled',params=dict(requestId='slow')))
        client.send(dict(jsonrpc='2.0',id='after-cancel',method='ping'))
        require(client.receive()['id']=='after-cancel','cancel responsive ping')
        time.sleep(0.1)
        v=client.call('compass_read',dict(source='large',path_key=reading.key(b'f0'),max_lines=1));require(not v['isError'],'post-cancel request')
        client.close();events+=client.events;checks.append('real-request-cancellation')
        # Resource cap and interrupted idle server do not block shutdown forever.
        client=Client(binary,config);client.initialize();client.p.send_signal(signal.SIGINT);client.p.wait(timeout=5);require(client.p.returncode==130,'idle SIGINT');client.p.stdin.close();client.sel.close();checks.append('SIGINT')
        client=Client(binary,config);client.p.stdin.write(b'x'*65537+b'\n');client.p.stdin.close();client.p.wait(timeout=5);require(client.p.returncode==1,'frame cap');client.sel.close();checks.append('frame-cap')
    require(reading.sha(binary.read_bytes())==before,'binary stable')
    return dict(binary_sha256=before,protocol_version='2025-11-25',checks=checks,transcript=events,client='verification/mcp/acceptance.py; real subprocess stdio, not a configured agent host')

def main():
    p=argparse.ArgumentParser();p.add_argument('--binary',required=True);p.add_argument('--output',type=Path);a=p.parse_args();r=run(a.binary)
    if a.output:a.output.write_text(json.dumps(r,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps(dict(checks=r['checks'],messages=len(r['transcript']))))
if __name__=='__main__':main()
