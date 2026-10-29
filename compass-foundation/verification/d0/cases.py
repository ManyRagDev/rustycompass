"""Reproducible static runtime scenarios. Every write stays in caller's temp tree."""
import copy
import json
import os
from pathlib import Path
import shutil
import socket
import validate as v


def bytes_for(lines):
    # The extra terminal LF preserves even a last empty line.
    raw=('\n'.join(lines)+'\n').encode() if lines else b''
    return b'\xef\xbb\xbf'+raw if lines and lines[0].startswith('\ufeff') else raw


def manifest(directory, side, files, coverage=None, excluded=None, executable=False):
    obj=v.load(v.FIX/'manifests/empty.json')
    obj['snapshot_id']='00000000-0000-4000-8000-'+('000000000101' if side=='baseline' else '000000000102')
    entries=[]
    for raw,data in files.items():
        entries.append(dict(path_key=v.key(raw),state='observed',kind='file',
                            content_sha256=v.sha(data),size_bytes=str(len(data)),executable=executable))
    entries.extend(copy.deepcopy(excluded or []))
    entries.sort(key=lambda e:v.path_bytes(e['path_key']))
    obj['entries']=entries
    cov={b'':dict(path_key='',state='complete')}
    for p in files:
        parts=p.split(b'/')
        for i in range(1,len(parts)):
            raw=b'/'.join(parts[:i])
            cov[raw]=dict(path_key=v.key(raw),state='complete')
    if coverage:
        for raw,state in coverage.items():
            cov[raw]=dict(path_key=v.key(raw),state=state)
            if state!='complete':
                cov[raw]['reason']='directory_changed' if state=='unstable' else 'permission_denied'
    obj['directory_coverage']=[cov[p] for p in sorted(cov)]
    obj['quality']='partial' if any(c['state']!='complete' for c in cov.values()) else 'complete'
    obs=[e for e in entries if e['state']=='observed']
    obj['stats'].update(observed_files=str(len(obs)),excluded_entries=str(len(entries)-len(obs)),
                        unknown_entries='0',bytes_read=str(sum(len(d) for d in files.values())),read_attempts=str(len(obs)))
    obj['state_digest']=None
    if obj['quality']=='complete':
        payload={k:obj[k] for k in ('workspace_id','path_namespace','policy_id')}
        payload['files']=[{k:e[k] for k in ('path_key','content_sha256','size_bytes','executable')} for e in obs]
        obj['state_digest']=v.sha(b'compass-state-v1\0'+v.canonical(payload))
    path=directory/(side+'.json')
    path.write_bytes(v.canonical(obj))
    package=directory/(side+'-source')
    (package/'objects').mkdir(parents=True)
    for data in files.values():
        (package/'objects'/v.sha(data)).write_bytes(data)
    desc=dict(source_schema_version='1',source_contract_version='0.1.0',workspace_id=obj['workspace_id'],
              snapshot_id=obj['snapshot_id'],policy_id=obj['policy_id'],snapshot_file_sha256=v.sha(path.read_bytes()))
    (package/'source.json').write_bytes(v.canonical(desc))
    return path,package


def make(directory, name, files0, files1, *, limits=None, cover0=None, cover1=None,
         excluded0=None, excluded1=None, executable=False, ids=(), literal=None):
    directory.mkdir()
    a,sa=manifest(directory,'baseline',files0,cover0,excluded0)
    b,sb=manifest(directory,'current',files1,cover1,excluded1,executable)
    return dict(name=name,ids=list(ids),paths=[a,b],packages=[sa,sb],limits=limits or {},literal=literal or {})


def object_path(case, side=0, path=b'p'):
    obj=v.load(case['paths'][side])
    entry=next(e for e in obj['entries'] if e['path_key']==v.key(path))
    return case['packages'][side]/'objects'/entry['content_sha256']


def frozen(directory):
    shutil.copytree(v.FIX,directory/'frozen')
    base=directory/'frozen'
    out=[]
    mapping={'complete':['E02'],'created-text':['E05'],'empty-universe':['E12'],
             'non-text':['E22'],'partial':['E27'],'scope-transition':['E25'],
             'unavailable-hash':['E29'],'unavailable-source':['E28','E31'],
             'unknown':['E26','E40'],'zero-text-mass':['E10','E44']}
    for name,c in v.load(base/'report-inputs.json').items():
        out.append(dict(name='frozen-'+name,ids=mapping[name],
                        paths=[base/c[s+'_manifest'] for s in v.SIDES],
                        packages=[base/c[s+'_package'] if c[s+'_package'] else None for s in v.SIDES],
                        limits={},literal={},frozen_report=base/c['expected_report'],expected_exit=c['expected_exit']))
    return out


def campaign(directory):
    out=frozen(directory)
    cases=v.load(v.FIX/'cases.json')
    def add(name,a,b,**kwargs):
        c=make(directory/name,name,a,b,**kwargs)
        out.append(c)
        return c
    # All literal line pairs become physical manifests and verified byte packages.
    for entry in cases:
        calcs=entry['calculations']
        if not calcs:
            continue
        a={}
        b={}
        for i,calc in enumerate(calcs):
            p=b'p'+str(i).encode()
            # E05/E06/E09/E11: actual absence, rather than an empty file side.
            x,y=calc['baseline_lines'],calc['current_lines']
            if not (entry['id'] in ('E05','E09','E11') and not x):
                a[p]=bytes_for(x)
            if not (entry['id'] in ('E06','E09') and not y):
                b[p]=bytes_for(y)
        # E52 exercises raw CRLF/LF identity before normalization, not only
        # its already-normalized mathematical line lists.
        if entry['id']=='E52':
            a={b'p0':bytes.fromhex(entry['input']['baseline_hex'])}
            b={b'p0':bytes.fromhex(entry['input']['current_hex'])}
        opts={}
        if entry['id'] in ('E26','E27','E38'):
            opts['cover1']={b'private':'unreadable'}
        if entry['id'] in ('E33','E37','E51','E55'):
            opts['limits']={'max_cells_total':'4'}
        literal={k:entry['expected'][k] for k in ('C','M','decimal','status','zero_basis','cells_reserved') if k in entry['expected']}
        if entry['id']=='E24':
            # Literal C/M corresponds to the frozen pairs; native names tested below.
            pass
        if entry['id']=='E37':
            # Insert blocked q between admitted p and r, preserving native order.
            a={b'p':bytes_for(['a','b']),b'q':b'a\n',b'r':b''}
            b={b'p':bytes_for(['a','c']),b'q':b'b\n',b'r':b'a\n'}
        if entry['id']=='E33':
            a[b'q']=b'a\n'; b[b'q']=b'b\n'
        add(entry['id'],a,b,ids=[entry['id']],literal=literal,**opts)
    for vector in v.load(v.FIX/'byte-vectors.json'):
        add('bytes-'+vector['label'],{b'p':bytes.fromhex(vector['baseline_hex'])},
            {b'p':bytes.fromhex(vector['current_hex'])},ids=[vector['label'][:3]])
    for ident in ('E20','E22','E23'):
        e=next(c for c in cases if c['id']==ident)
        add(ident,{b'p':bytes.fromhex(e['input']['baseline_hex'])},
            {b'p':bytes.fromhex(e['input']['current_hex'])},ids=[ident],
            literal={'status':'unavailable','local':['non_text']})
    c=add('E21-non-text',{b'p':bytes_for(list(map(str,range(10)))),b'q':b'\0\xff'},
          {b'p':bytes_for(['x']+list(map(str,range(1,10)))),b'q':b'\0\xfe'},ids=['E21'],literal={'C':'2','M':'20','status':'complete'})
    for path_key in ('1111','1'*24):
        raw=v.path_bytes(path_key)
        add('BUG-001-path-'+str(len(path_key)),{raw:b'a\n'},{raw:b'a\n'},ids=['E24','E50'],
            literal={'status':'complete','C':'0','M':'2','decimal':'0.000000','cells_reserved':'0'})
    add('E24-native',{b'A':b'a\n',b'a':b'a\n',b'\xff':b'a\n'},
        {b'A':b'a\n',b'a':b'a\n',b'\xff':b'a\n'},executable=True,ids=['E24'],literal={'C':'0','M':'6'})
    add('E26-ancestor',{b'd/p':b'a\n',b'q':b'a\n'},
        {b'd/p':b'a\n',b'q':b'a\nb\n'},cover1={b'd':'unstable'},ids=['E26'],literal={'C':'1','M':'3','status':'partial'})
    add('E39',{}, {},cover1={b'private':'unreadable'},ids=['E39'],literal={'status':'unavailable'})
    excluded=dict(path_key=v.key(b'p'),kind='symlink',state='excluded',reason='symlink_not_followed')
    add('E40',{b'p':b'a\n'}, {},cover0={b'':'unstable'},excluded1=[excluded],ids=['E40'],literal={'local':['observation_unknown'],'object_bytes_read':'0'})
    add('E25-entered',{}, {b'p':b'a\n'},excluded0=[excluded],ids=['E25'],literal={'local':['scope_transition']})
    add('E25-directory',{b'p':b'a\n'}, {},cover1={b'p':'complete'},ids=['E25'],literal={'C':'1','M':'1'})
    for name,ids,kind in [('E28-missing-object',['E28'],'missing'),('E30',['E30'],'prefix'),
                          ('E43-symlink',['E43'],'symlink'),('E43-fifo',['E43'],'fifo'),
                          ('E43-directory',['E43'],'directory'),('E43-socket',['E43'],'socket')]:
        c=add(name,{b'p':b'abc'},{b'p':b'abc'},ids=ids)
        p=object_path(c)
        p.unlink()
        if kind=='prefix': p.write_bytes(b'abcd')
        elif kind=='symlink':
            target=directory/(name+'-target'); target.write_bytes(b'abc'); p.symlink_to(target)
        elif kind=='fifo': os.mkfifo(p)
        elif kind=='directory': p.mkdir()
        elif kind=='socket':
            # A pathname socket remains after closing the listening handle.
            sock=socket.socket(socket.AF_UNIX)
            short=directory/(name+'-sock')
            sock.bind(str(short)); sock.close(); short.rename(p)
        c['literal']={'local':[{'missing':'object_missing','prefix':'size_mismatch'}.get(kind,'source_invalid_kind')],'status':'unavailable'}
    c=add('E41',{b'p':b'a\n'},{b'p':b'\0a'},ids=['E41'],literal={'local':['source_missing'],'status':'unavailable'})
    c['packages'][0]=None
    for name,shared,hardlink in [('E44-shared',True,False),('E44-distinct',False,False),('E44-hardlink',False,True)]:
        c=add(name,{b'p':b''},{b'p':b''},ids=['E44'],literal={'object_bytes_reserved':'1' if shared else '2'})
        c['paths'][1]=c['paths'][0]
        if shared:
            c['packages'][1]=c['packages'][0]
        else:
            shutil.copyfile(c['packages'][0]/'source.json',c['packages'][1]/'source.json')
            if hardlink:
                p=object_path(c,1); p.unlink(); os.link(object_path(c),p)
    for reverse in (False,True):
        c=add('E45-'+str(reverse),{b'p':b'a\n',b'q':b''},{b'p':b'',b'q':b''},ids=['E45'],
              limits={'max_total_object_bytes':'3'},literal={'status':'partial' if reverse else 'unavailable','object_bytes_reserved':'3'})
        if reverse:
            c['paths'].reverse(); c['packages'].reverse()
    c=add('E47',{b'p':b'a\n',b'q':b''},{b'p':b'',b'q':b''},ids=['E47'],
          limits={'max_total_object_bytes':'3'},literal={'object_bytes_reserved':'3','object_bytes_read':'0','status':'unavailable'})
    object_path(c).unlink()
    for name,ident,equal in [('E36','E36',True),('E54','E54',False)]:
        add(name,{b'p':b'a\n'*4000},{b'p':(b'a\n' if equal else b'b\n')*4000},ids=[ident],
            literal={'C':'0','M':'8000','cells_reserved':'0','status':'complete'} if equal else {'local':['diff_work_limit'],'status':'unavailable','cells_reserved':'0'})
    # A→B→A uses only the explicitly supplied endpoints.
    add('E08-AA',{b'p':b'a\nb\n'},{b'p':b'a\nb\n'},ids=['E08'],literal={'C':'0','M':'4'})
    add('E08-AB',{b'p':b'a\nb\n'},{b'p':b'a\nc\n'},ids=['E08'],literal={'C':'2','M':'4'})
    c=add('E28-partial',{b'p':b'abc',b'q':bytes_for(list(map(str,range(10))))},
          {b'p':b'abc',b'q':bytes_for(list(map(str,range(11))))},ids=['E28'],literal={'C':'1','M':'21','status':'partial'})
    object_path(c).unlink()
    c=add('E31-empty-missing',{b'p':b''},{b'p':b''},ids=['E31'],literal={'local':['source_missing'],'status':'unavailable'})
    c['packages']=[None,None]
    for variant in ('missing','adulterated'):
        baseline=(b'a\r\n'+b'a\n')*2000 if variant=='adulterated' else b'a\n'*4000
        c=add('E53-'+variant,{b'p':baseline},{b'p':b'a\n'*4000},ids=['E53'],
              literal={'local':['source_missing' if variant=='missing' else 'hash_mismatch'],'status':'unavailable','cells_reserved':'0'})
        if variant=='missing':
            c['packages'][0]=None
        else:
            # Same size and normalized sequence, different raw hash; still forbidden.
            object_path(c).write_bytes((b'a\n'+b'a\r\n')*2000)
    # Static, exact boundaries and precedence. No giant allocations required.
    for limit,at,over in [('max_file_bytes','2','1'),('max_lines_per_side','2','1'),('max_cells_per_unit','4','3')]:
        for value in (at,over):
            data0,data1=(b'a\n',b'b\n') if limit=='max_file_bytes' else (b'a\nb\n',b'a\nc\n')
            reason={'max_file_bytes':'file_bytes_limit','max_lines_per_side':'line_count_limit','max_cells_per_unit':'diff_work_limit'}[limit]
            add('boundary-'+limit+'-'+value,{b'p':data0},{b'p':data1},limits={limit:value},ids=['E32','E46'],
                literal={'status':'complete'} if value==at else {'local':[reason],'status':'unavailable'})
    # Descriptor whitespace is accepted and changes only raw source_ref.
    c=add('E42-whitespace',{b'p':b'a\n'},{b'p':b'a\n'},ids=['E42'],literal={'C':'0','M':'2'})
    p=c['packages'][0]/'source.json'; p.write_text(json.dumps(v.load(p),indent=2))
    # Structural errors have fixed envelopes and exit codes.
    for name,error,code in [('descriptor-binding','incompatible_inputs',2),('descriptor-duplicate','source_invalid',1),
                            ('manifest-duplicate','input_invalid',1),('manifest-digest','input_invalid',1),
                            ('manifest-whitespace-binding','incompatible_inputs',2),('descriptor-root-link','source_invalid',1),
                            ('source-link','source_invalid',1),('manifest-link','input_invalid',1)]:
        c=add(name,{b'p':b'a\n'},{b'p':b'b\n'},ids=['E34','E42','E43'])
        c['error']=(error,code)
        if name=='descriptor-binding':
            p=c['packages'][0]/'source.json'; d=v.load(p); d['snapshot_id']='00000000-0000-4000-8000-000000000999'; p.write_bytes(v.canonical(d))
        elif name=='descriptor-duplicate':
            (c['packages'][0]/'source.json').write_bytes(b'{"snapshot_id":"x","snapshot_id":"y"}')
        elif name=='manifest-duplicate': c['paths'][0].write_bytes(b'{"x":1,"x":2}')
        elif name=='manifest-digest':
            p=c['paths'][0]; d=v.load(p); d['state_digest']='0'*64; p.write_bytes(v.canonical(d))
        elif name=='manifest-whitespace-binding':
            p=c['paths'][0]; p.write_text(json.dumps(v.load(p),indent=2))
        elif name=='descriptor-root-link':
            p=c['packages'][0]; link=p.with_name('root-link'); link.symlink_to(p,target_is_directory=True); c['packages'][0]=link
        else:
            p=c['packages'][0]/'source.json' if name=='source-link' else c['paths'][0]
            target=p.with_name('original'); p.rename(target); p.symlink_to(target)
    for name,limit,value,code in [('output-limit','max_output_bytes','1','output_limit'),
                                  ('manifest-limit','max_manifest_bytes','1','input_limit'),
                                  ('descriptor-limit','max_source_descriptor_bytes','1','input_limit'),
                                  ('records-limit','max_records','1','input_limit'),
                                  ('memory-structural','max_logical_memory_bytes','1','input_limit')]:
        c=add(name,{b'p':b'a\n'},{b'p':b'b\n'},ids=['E46'],limits={limit:value})
        c['error']=(code,1)
    # Wb and W exact admission boundaries are computed from input sizes, not production.
    for phase in ('Wb','W'):
        for offset in (0,-1):
            name='memory-'+phase+'-'+str(offset)
            c=add(name,{b'p':b'a\nb\n'},{b'p':b'a\nc\n'},ids=['E46'])
            base=v.model(c['paths'],c['packages'])
            rawbytes=sum(p.stat().st_size for p in c['paths'])+sum((p/'source.json').stat().st_size for p in c['packages'])
            structural=8*rawbytes+4096*4+65536
            wb=2*(4+4+2)+65536
            w=wb+32*4+8*3
            budget=structural+(wb if phase=='Wb' else w)+offset
            c['limits']={'max_logical_memory_bytes':str(budget)}
            if phase=='Wb' or offset==-1:
                c['literal']={'local':['memory_budget'],'status':'unavailable'}
            else: c['literal']={'status':'complete'}
    for variant in ('objects-link','objects-dangling-link','root-missing'):
        c=add('E43-'+variant,{b'p':b'abc'},{b'p':b'abc'},ids=['E43'])
        if variant in ('objects-link','objects-dangling-link'):
            p=c['packages'][0]/'objects'; target=p.with_name('stored'); p.rename(target)
            p.symlink_to(target if variant=='objects-link' else p.with_name('absent-target'),target_is_directory=True)
            c['literal']={'local':['source_invalid_kind'],'status':'unavailable','object_bytes_reserved':'8','object_bytes_read':'3'}
        else:
            c['packages'][0]=c['paths'][0].parent/'absent-source'; c['error']=('source_invalid',1)
    # Competing failures assert precedence, rather than only isolated frontiers.
    for phase in ('file-before-memory','line-before-memory'):
        c=add(phase,{b'p':b'a\nb\n'},{b'p':b'a\nc\n'},ids=['E46'])
        rawbytes=sum(p.stat().st_size for p in c['paths'])+sum((p/'source.json').stat().st_size for p in c['packages'])
        structural=8*rawbytes+4096*4+65536
        wb=2*(4+4+2)+65536
        if phase=='file-before-memory':
            c['limits']={'max_file_bytes':'1','max_logical_memory_bytes':str(structural+1)}
            c['literal']={'local':['file_bytes_limit'],'object_bytes_read':'0'}
        else:
            c['limits']={'max_lines_per_side':'1','max_logical_memory_bytes':str(structural+wb)}
            c['literal']={'local':['line_count_limit']}
    c=add('file-before-source',{b'p':b'abc'},{b'p':b'abc'},ids=['E32','E46'],limits={'max_file_bytes':'2'},literal={'local':['file_bytes_limit'],'object_bytes_reserved':'0'})
    c['packages']=[None,None]
    c=add('manifest-memory-before-digest',{b'p':b'a\n'},{b'p':b'a\n'},ids=['E46'],limits={'max_logical_memory_bytes':'1'})
    p=c['paths'][0]; obj=v.load(p); obj['state_digest']='0'*64; p.write_bytes(v.canonical(obj)); c['error']=('input_limit',1)
    c=add('compatibility-before-source',{b'p':b'a\n'},{b'p':b'a\n'},ids=['E34'])
    p=c['paths'][1]; obj=v.load(p); obj['workspace_id']='00000000-0000-4000-8000-000000000999'
    payload={k:obj[k] for k in ('workspace_id','path_namespace','policy_id')}
    payload['files']=[{k:e[k] for k in ('path_key','content_sha256','size_bytes','executable')} for e in obj['entries']]
    obj['state_digest']=v.sha(b'compass-state-v1\0'+v.canonical(payload)); p.write_bytes(v.canonical(obj))
    c['packages'][0]=c['paths'][0].parent/'missing-source'; c['error']=('incompatible_inputs',2)
    for variant in ('invalid-path','u64-overflow'):
        c=add('E50-'+variant,{b'p':b'a\n'},{b'p':b'b\n'},ids=['E50'])
        p=c['paths'][0]; obj=v.load(p)
        if variant=='invalid-path': obj['entries'][0]['path_key']='Li4'
        else: obj['entries'][0]['size_bytes']='18446744073709551616'
        p.write_bytes(v.canonical(obj)); c['error']=('input_invalid',1)
    c=add('deadline-public',{b'p':b'a\n'*2500},{b'p':b'b\n'*4000},ids=['E35'])
    c['deadline_ms']='1'
    c['error']=('deadline_exceeded',1)
    return out


def assert_literal(expected, literal):
    m=expected['measurement']
    agg=m['known_subset'] if m['status']=='partial' else m
    for k,value in literal.items():
        if k=='status': v.equal(m['status'],value,'literal_status')
        elif k in ('C','M','decimal'):
            actual=(agg['value']['numerator'] if k=='C' else agg['line_mass'] if k=='M' else agg['value']['decimal'])
            v.equal(actual,value,'literal_'+k)
        elif k=='zero_basis': v.equal(agg['zero_basis'],value,'literal_zero')
        elif k=='local':
            reasons={r for u in expected['units'] for r in u['reasons']}
            v.need(set(value)<=reasons,'literal_local',str(reasons))
        elif k in expected['resources']: v.equal(expected['resources'][k],value,'literal_'+k)
        else: raise AssertionError('unknown literal '+k)


def inventory(directory):
    result={}
    for p in sorted(directory.rglob('*')):
        rel=str(p.relative_to(directory))
        mode=p.lstat().st_mode
        import stat
        if p.is_symlink(): result[rel]=['link',os.readlink(p)]
        elif stat.S_ISREG(mode): result[rel]=['file',v.sha(p.read_bytes()),mode]
        elif stat.S_ISDIR(mode): result[rel]=['directory',mode]
        else: result[rel]=['special',mode]
    return result
