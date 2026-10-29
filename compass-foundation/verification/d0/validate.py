"""Strict transport, frozen-input integrity and independent report assertions.

Filesystem observations here concern controlled, static laboratory trees only.
They do not implement or prove the production openat2/TOCTOU guarantees.
"""
import base64
import copy
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import sys
from datetime import datetime
from functools import lru_cache
from urllib.parse import unquote, urlparse
import oracle

ROOT = Path(__file__).resolve().parents[2]
CONTRACT = ROOT / 'research/proposals/d0-c2'
FIX = CONTRACT / 'fixtures'
INDEX_HASH = '63168b321845393168134a30afa39811e702eecc678b31c06f3bc24d3c17208e'
POLICY_ID = '51d824e4f7e229cfeb61b63c33681fff65a879e173956bdb0029dd4473d358a5'
SIDES = ('baseline', 'current')
U64 = 2**64-1


class Rejected(AssertionError):
    def __init__(self, code, detail=''):
        self.code = code
        super().__init__(f'{code}: {detail}')


def need(ok, code, detail=''):
    if not ok:
        raise Rejected(code, detail)


def equal(actual, expected, code):
    need(actual == expected, code, f'actual={actual!r}; expected={expected!r}')


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def strict(raw):
    def pairs(items):
        out = {}
        for k, v in items:
            need(k not in out, 'json_duplicate')
            out[k] = v
        return out
    def bad(_):
        raise Rejected('json_number')
    try:
        need(not raw.startswith(b'\xef\xbb\xbf'), 'json_bom')
        parsed=json.loads(raw.decode('utf-8', 'strict'), object_pairs_hook=pairs,
                          parse_constant=bad, parse_float=bad)
        def unicode_strings(x):
            if isinstance(x,str): x.encode('utf-8','strict')
            elif isinstance(x,list):
                for y in x: unicode_strings(y)
            elif isinstance(x,dict):
                for k,y in x.items(): unicode_strings(k); unicode_strings(y)
        unicode_strings(parsed)
        return parsed
    except (UnicodeError, ValueError) as error:
        raise Rejected('json_transport', str(error)) from error


def load(path):
    return strict(Path(path).read_bytes())


def canonical(value):
    """JCS subset: string/null/bool/list/object; no numbers or lone surrogates.

    UTF-16 property ordering, UTF-8 strings and JSON control escaping. This is
    deliberately not an ECMAScript number formatter or a general JCS library.
    """
    if value is None:
        return b'null'
    if isinstance(value, bool):
        return b'true' if value else b'false'
    if isinstance(value, str):
        return json.dumps(value, ensure_ascii=False).encode('utf-8', 'strict')
    if isinstance(value, list):
        return b'[' + b','.join(canonical(v) for v in value) + b']'
    if isinstance(value, dict):
        return b'{' + b','.join(canonical(k)+b':'+canonical(value[k]) for k in
                                sorted(value, key=lambda s:s.encode('utf-16-be'))) + b'}'
    raise Rejected('canonical_domain', type(value).__name__)


def cached_dependencies(path=None):
    if path:
        for p in sorted(Path(path).iterdir()):
            if any((p/n).exists() for n in ('jsonschema','attrs','attr','referencing',
                                           'rpds','jsonschema_specifications','typing_extensions.py')):
                sys.path.insert(0, str(p))


@lru_cache(maxsize=None)
def validator(name):
    from jsonschema import Draft202012Validator, FormatChecker
    from referencing import Registry, Resource
    path = (ROOT/'research/contracts/transport.schema.json' if name == 'foundation'
            else CONTRACT/'schemas'/f'{name}.schema.json')
    def retrieve(uri):
        target = Path(unquote(urlparse(uri).path)).resolve()
        allowed = [ROOT/'research/contracts/transport.schema.json',
                   *sorted((CONTRACT/'schemas').glob('*.json'))]
        need(target in allowed, 'schema_external_reference')
        return Resource.from_contents(load(target))
    schema = load(path)
    if name == 'foundation':
        schema = {'$schema':schema['$schema'], '$ref':path.as_uri()+'#/$defs/snapshot'}
    else:
        Draft202012Validator.check_schema(schema)
        schema = dict(schema, **{'$id': path.as_uri()})
    return Draft202012Validator(schema, registry=Registry(retrieve=retrieve),
                                format_checker=FormatChecker())


def schema(value, name):
    errors = list(validator(name).iter_errors(value))
    need(not errors, 'schema_'+name, str(errors[0])[:250] if errors else '')


def integrity(base=CONTRACT):
    index = (base/'SHA256SUMS').read_bytes()
    equal(sha(index), INDEX_HASH, 'index_hash')
    names = []
    for line in index.decode().splitlines():
        digest, name = line.split('  ', 1)
        names.append(name)
        equal(sha((base/name).read_bytes()), digest, 'artifact_hash')
    equal(len(names),81,'index_count')
    equal(len(set(names)),81,'index_duplicates')
    equal(sha((base/'CONTRATO_CANDIDATO.md').read_bytes()),
          '884eecc521bfa123ad3d0cc608c7c7619ab63bb87affa2112be217ac47f7291e','contract_hash')
    equal(sha((ROOT/'research/audits/AUDITORIA-OS-D0-C2-A1.md').read_bytes()),
          '9e89ad9217b50f475fafd4dc74ce7edae37345187ca00225cfa8d8f8b9b51c20','audit_hash')
    equal(policy_id(load(base/'fixtures/metric-policy.json')), POLICY_ID,'policy_hash')
    for f in load(base/'evidence/preservacao.json')['protected_files']:
        equal(sha((ROOT/f['path']).read_bytes()), f['after_sha256'], 'protected_hash')
    return {'index_sha256':INDEX_HASH,'artifacts':81,'protected_files':34,'metric_policy_id':POLICY_ID}


def policy_id(policy):
    return sha(b'compass-d0-policy-v1\0'+canonical(policy))


def key(raw):
    return base64.urlsafe_b64encode(raw).rstrip(b'=').decode('ascii')


def path_bytes(value, root=False):
    need(isinstance(value,str) and re.fullmatch('[A-Za-z0-9_-]*',value), 'path')
    try:
        raw = base64.urlsafe_b64decode(value+'='*(-len(value)%4))
    except ValueError as e:
        raise Rejected('path') from e
    need(key(raw)==value and b'\0' not in raw, 'path')
    need((root and raw==b'') or all(x not in (b'',b'.',b'..') for x in raw.split(b'/')), 'path')
    return raw


def uint(value):
    need(isinstance(value,str) and re.fullmatch(r'0|[1-9][0-9]*',value) is not None
         and int(value)<=U64, 'u64')
    return int(value)


# Numeric field roles from the frozen Foundation/D0 schemas; identifiers and
# paths retain their own domains even when their string representation is digits.
U64_FIELDS = frozenset({
    'a', 'added', 'bytes_read', 'c', 'candidates', 'cells_reserved', 'denominator',
    'duration_ms', 'entered_scope', 'excluded_entries', 'l', 'left_scope',
    'line_mass', 'logical_memory_peak', 'm', 'max_cells_per_unit',
    'max_cells_total', 'max_duration_ms', 'max_entries', 'max_file_bytes',
    'max_json_depth', 'max_lines_per_side', 'max_logical_memory_bytes',
    'max_manifest_bytes', 'max_output_bytes', 'max_records',
    'max_source_descriptor_bytes', 'max_total_bytes_read', 'max_total_object_bytes',
    'metadata_changed', 'modified', 'n0', 'non_text_units', 'nt', 'numerator',
    'object_bytes_read', 'object_bytes_reserved', 'observed_files', 'ordinal',
    'r', 'read_attempts', 'removed', 'resolved_text_units', 'scope_transitions',
    'size_bytes', 'unavailable_units', 'unchanged', 'unknown', 'unknown_entries',
})


def domain(value, role=None):
    if isinstance(value,dict):
        for k,v in value.items():
            if k=='path_key':
                path_bytes(v,root=True)
            # 'unknown' is a counter only inside Comparison.counts; in the
            # metric policy it is the textual rule 'blocks_integral'.
            numeric = k in U64_FIELDS and (k != 'unknown' or role == 'counts')
            if numeric and v is not None:
                uint(v)
            domain(v, k)
    elif isinstance(value,list):
        for v in value:
            domain(v, role)


def ordered(records, allow_root=False):
    values=[path_bytes(e['path_key'],allow_root) for e in records]
    need(values==sorted(set(values)), 'native_order')


def snapshot(path):
    raw=Path(path).read_bytes()
    obj=strict(raw)
    schema(obj,'foundation')
    domain(obj)
    equal(sha(b'compass-policy-v1\0'+canonical(obj['policy'])),obj['policy_id'],'foundation_policy')
    for k in ('exclude_directory_names','exclude_file_names','exclude_file_prefixes','exclude_paths'):
        xs=obj['policy'][k]
        equal(xs,sorted(set(xs)), 'foundation_policy_order')
    ordered(obj['entries'])
    ordered(obj['directory_coverage'],True)
    entries={path_bytes(e['path_key']):e for e in obj['entries']}
    coverage={path_bytes(e['path_key'],True):e for e in obj['directory_coverage']}
    need(not entries.keys() & coverage.keys(), 'tree_overlap')
    need(b'' in coverage, 'tree_root')
    for p in entries.keys() | coverage.keys():
        if p:
            need(p.rpartition(b'/')[0] in coverage,'tree_parent')
    observed=[e for e in obj['entries'] if e['state']=='observed']
    unknown=[e for e in obj['entries'] if e['state'] not in ('observed','excluded')]
    quality='partial' if unknown or any(c['state']!='complete' for c in coverage.values()) else 'complete'
    equal(obj['quality'],quality,'snapshot_quality')
    for field,n in [('observed_files',len(observed)),('excluded_entries',sum(e['state']=='excluded' for e in entries.values())),('unknown_entries',len(unknown))]:
        equal(obj['stats'][field],str(n),'snapshot_counts')
    need(int(obj['stats']['bytes_read'])>=sum(int(e['size_bytes']) for e in observed),'snapshot_bytes')
    need(int(obj['stats']['read_attempts'])>=len(observed),'snapshot_attempts')
    start,end=[datetime.fromisoformat(obj[k].replace('Z','+00:00')) for k in ('started_at','finished_at')]
    need(start.utcoffset().total_seconds()==end.utcoffset().total_seconds()==0 and end>=start,'snapshot_window')
    issue_order=[(path_bytes(i['path_key'],True),int(i['ordinal'])) for i in obj['issues']]
    equal(issue_order,sorted(set(issue_order)),'issues_order')
    digest=None
    if quality=='complete':
        payload={k:obj[k] for k in ('workspace_id','path_namespace','policy_id')}
        payload['files']=[{k:e[k] for k in ('path_key','content_sha256','size_bytes','executable')} for e in observed]
        digest=sha(b'compass-state-v1\0'+canonical(payload))
    equal(obj['state_digest'],digest,'state_digest')
    return obj,raw


def effective(obj,p):
    entries={path_bytes(e['path_key']):e for e in obj['entries']}
    cov={path_bytes(e['path_key'],True):e for e in obj['directory_coverage']}
    chain=[b'']+[b'/'.join(p.split(b'/')[:i]) for i in range(1,len(p.split(b'/'))+1)]
    if any(x in cov and cov[x]['state']!='complete' for x in chain):
        return 'unknown',None
    for x in chain:
        e=entries.get(x)
        if e:
            if e['state'] not in ('observed','excluded'):
                return 'unknown',e
            if e['state']=='excluded':
                return 'excluded',e
            return ('observed',e) if x==p else ('absent',None)
    if p in cov:
        return 'absent',None
    for x in chain[1:]:
        if x not in entries and x not in cov:
            parent=x.rpartition(b'/')[0]
            return ('absent',None) if parent in cov and cov[parent]['state']=='complete' else ('unknown',None)
    return 'unknown',None


def comparison(a,b):
    for k in ('workspace_id','path_namespace','policy_id'):
        equal(a[k],b[k],'incompatible_inputs')
    maps=[{e['path_key']:e for e in obj['entries']} for obj in (a,b)]
    counts=dict.fromkeys(('unchanged','added','removed','modified','metadata_changed','left_scope','entered_scope','unknown'),0)
    changes=[]
    classes={}
    for k in sorted(maps[0].keys()|maps[1].keys(), key=path_bytes):
        p=path_bytes(k)
        (s,e),(t,f)=[effective(o,p) for o in (a,b)]
        before,after=[m.get(k) for m in maps]
        if 'unknown' in (s,t): c='unknown'
        elif s==t=='observed':
            c=('modified' if (e['content_sha256'],e['size_bytes'])!=(f['content_sha256'],f['size_bytes']) else
               'metadata_changed' if e['executable']!=f['executable'] else 'unchanged')
        elif s=='observed': c='left_scope' if t=='excluded' else 'removed'
        elif t=='observed': c='entered_scope' if s=='excluded' else 'added'
        else: continue
        counts[c]+=1
        classes[k]=c
        if c=='unchanged': continue
        both=before is not None and after is not None and before['state']==after['state']=='observed'
        change=dict(path_key=k,classification=c,before=before,after=after,
                    content_changed=(before['content_sha256'],before['size_bytes'])!=(after['content_sha256'],after['size_bytes']) if both else c!='unknown',
                    executable_changed=before['executable']!=after['executable'] if both else False)
        if c in ('left_scope','entered_scope'):
            change['reason']=(f if c=='left_scope' else e)['reason']
        changes.append(change)
    regions=[]
    for side,o in zip(SIDES,(a,b)):
        for v in o['directory_coverage']:
            if v['state']!='complete':
                regions.append(dict(v,side=side))
    regions.sort(key=lambda r:(path_bytes(r['path_key'],True),r['side']))
    partial=a['quality']=='partial' or b['quality']=='partial'
    result={k:a[k] for k in ('schema_version','contract_version','workspace_id','path_namespace','policy_id')}
    result.update(baseline_snapshot_id=a['snapshot_id'],current_snapshot_id=b['snapshot_id'],
                  quality='partial' if partial else 'complete',relation='undetermined' if partial else 'different' if changes else 'equal',
                  changes=changes,counts={k:str(v) for k,v in counts.items()},unknown_regions=regions)
    return result,classes


def static_path_kind(path):
    """Inspect each static component before concluding that the object is absent.

    lstat preserves dangling-link evidence; this is not a secure production open.
    """
    path=Path(path).absolute()
    for component in [*reversed(path.parents),path]:
        try:
            mode=component.lstat().st_mode
        except FileNotFoundError:
            return 'missing'
        if stat.S_ISLNK(mode) or (component != path and not stat.S_ISDIR(mode)):
            return 'invalid'
    return 'regular' if stat.S_ISREG(mode) else 'invalid'


def regular_path(path):
    """Static-tree kind check, never used as a production secure-open substitute."""
    return static_path_kind(path) == 'regular'


def descriptor(package, obj, raw):
    if package is None:
        return None,None
    p=Path(package)/'source.json'
    need(regular_path(p),'source_invalid')
    data=p.read_bytes()
    d=strict(data)
    schema(d,'source')
    expected=dict(source_schema_version='1',source_contract_version='0.1.0',
                  workspace_id=obj['workspace_id'],snapshot_id=obj['snapshot_id'],
                  policy_id=obj['policy_id'],snapshot_file_sha256=sha(raw))
    equal(d,expected,'incompatible_inputs')
    return dict(d,source_ref=sha(data)),data


def model(paths, packages, limits=None):
    """Independent deterministic expected report components for static lab inputs."""
    parsed=[snapshot(p) for p in paths]
    objects=[o for o,_ in parsed]
    raws=[r for _,r in parsed]
    comp,classes=comparison(*objects)
    policy=load(FIX/'metric-policy.json')
    if limits:
        policy['limits'].update(limits)
    schema(policy,'metric-policy')
    lim={k:int(v) if v is not None else None for k,v in policy['limits'].items()}
    defaults=load(FIX/'metric-policy.json')['limits']
    for k,v in lim.items():
        if k=='max_duration_ms':
            need(v is None or 1<=v<=3600000,'invalid_arguments')
        else:
            need(0<v<=int(defaults[k]),'invalid_arguments')
    n=sum(len(o[k]) for o in objects for k in ('entries','directory_coverage','issues'))
    s=8*sum(map(len,raws))+4096*n+65536
    need(s<=lim['max_logical_memory_bytes'],'input_limit')
    sources=[]
    for package,o,raw in zip(packages,objects,raws):
        d,data=descriptor(package,o,raw)
        if data:
            s+=8*len(data)
            need(s<=lim['max_logical_memory_bytes'],'input_limit')
        sources.append(d)
    resources=dict(object_bytes_read=0,object_bytes_reserved=0,cells_reserved=0,logical_memory_peak=s)
    maps=[{e['path_key']:e for e in o['entries']} for o in objects]
    candidates={e['path_key'] for o in objects for e in o['entries'] if e['state']=='observed'}
    candidates|={k for k,c in classes.items() if c=='unknown'}
    units=[]
    for k in sorted(candidates,key=path_bytes):
        entries=[m.get(k) for m in maps]
        states=[effective(o,path_bytes(k))[0] for o in objects]
        checks=[]
        for state,e,source in zip(states,entries,sources):
            rev={f:e[f] for f in ('content_sha256','size_bytes')} if e and e['state']=='observed' else None
            checks.append(dict(observation=state,state=state if state in ('absent','excluded') else 'not_attempted',
                               expected_revision=rev,source_ref=source['source_ref'] if source and rev else None,
                               actual_revision=None,hash_check=None,size_check=None,reasons=[]))
        u=dict(path_key=k,classification=classes[k],before=entries[0],after=entries[1],
               status='unavailable',reasons=[],non_text=[],checks=dict(zip(SIDES,checks)))
        units.append(u)
        if 'unknown' in states:
            u['reasons']=['observation_unknown']
            for c in checks:
                c['state']='not_attempted'
                c['reasons']=['observation_unknown']
            continue
        if classes[k] in ('entered_scope','left_scope'):
            u.update(status='out_of_scope',reasons=['scope_transition'])
            continue
        sizes=[int(c['expected_revision']['size_bytes']) if c['expected_revision'] else 0 for c in checks]
        if any(size>lim['max_file_bytes'] for size in sizes):
            for c,size in zip(checks,sizes):
                if size>lim['max_file_bytes']:
                    c['reasons']=['file_bytes_limit']
            u['reasons']=['file_bytes_limit']
            continue
        wb=2*(sum(sizes)+2)+65536
        if s+wb>lim['max_logical_memory_bytes']:
            for c in checks:
                if c['observation']=='observed': c['reasons']=['memory_budget']
            u['reasons']=['memory_budget']
            continue
        resources['logical_memory_peak']=max(resources['logical_memory_peak'],s+wb)
        contents=[b'',b'']
        shared={}
        for i,(c,size,package,source) in enumerate(zip(checks,sizes,packages,sources)):
            if c['observation']=='absent': continue
            if source is None:
                c['reasons']=['source_missing']
                continue
            root=Path(package)
            identity=(root.stat().st_dev,root.stat().st_ino,source['source_ref'],c['expected_revision']['content_sha256'])
            if identity in shared:
                old,data=shared[identity]
                checks[i].update(copy.deepcopy(old))
                contents[i]=data
                continue
            if resources['object_bytes_reserved']+size+1>lim['max_total_object_bytes']:
                c['reasons']=['total_bytes_limit']
                continue
            resources['object_bytes_reserved']+=size+1
            obj=root/'objects'/c['expected_revision']['content_sha256']
            data=b''
            kind=static_path_kind(obj)
            if kind == 'missing':
                c.update(state='failed',reasons=['object_missing'])
            elif kind != 'regular':
                c.update(state='failed',reasons=['source_invalid_kind'])
            else:
                try:
                    with obj.open('rb') as f:
                        data=f.read(size+1)
                    resources['object_bytes_read']+=len(data)
                    if len(data)==size+1:
                        c.update(state='failed',size_check=False,reasons=['size_mismatch'])
                    else:
                        actual=dict(content_sha256=sha(data),size_bytes=str(len(data)))
                        hc=actual['content_sha256']==c['expected_revision']['content_sha256']
                        sc=len(data)==size
                        c.update(actual_revision=actual,hash_check=hc,size_check=sc,
                                 state='verified' if hc and sc else 'failed',
                                 reasons=sorted(([] if hc else ['hash_mismatch'])+([] if sc else ['size_mismatch'])))
                except PermissionError:
                    c.update(state='failed',reasons=['source_unreadable'])
            contents[i]=data
            shared[identity]=(copy.deepcopy(c),data)
        reasons=sorted({r for c in checks for r in c['reasons']})
        if reasons:
            u['reasons']=reasons
            continue
        normalized=[]
        for side,data in zip(SIDES,contents):
            try:
                normalized.append(oracle.lines(data))
            except UnicodeDecodeError:
                u['non_text'].append(dict(side=side,reason='invalid_utf8'))
            except ValueError:
                u['non_text'].append(dict(side=side,reason='nul'))
        if u['non_text']:
            u.update(status='out_of_scope',reasons=['non_text'])
            continue
        n0,nt=map(len,normalized)
        if max(n0,nt)>lim['max_lines_per_side']:
            u['reasons']=['line_count_limit']
            continue
        w=wb+32*(n0+nt)+8*(min(n0,nt)+1)
        if s+w>lim['max_logical_memory_bytes']:
            u['reasons']=['memory_budget']
            continue
        resources['logical_memory_peak']=max(resources['logical_memory_peak'],s+w)
        cells=oracle.cell_cost(*normalized)
        if cells>lim['max_cells_per_unit'] or resources['cells_reserved']+cells>lim['max_cells_total']:
            u['reasons']=['diff_work_limit']
            continue
        resources['cells_reserved']+=cells
        u.update(status='available',calculation=oracle.calculation(*normalized))
    exclusions=[dict(side=side,entry=e) for side,o in zip(SIDES,objects) for e in o['entries'] if e['state']=='excluded']
    exclusions.sort(key=lambda x:(path_bytes(x['entry']['path_key']),x['side']))
    scope=dict(candidates=str(len(units)),resolved_text_units=str(sum(u['status']=='available' for u in units)),
               unavailable_units=str(sum(u['status']=='unavailable' for u in units)),
               non_text_units=str(sum(bool(u['non_text']) for u in units)),
               scope_transitions=str(sum(u['classification'] in ('left_scope','entered_scope') for u in units)),exclusions=exclusions)
    summaries=[]
    for o,raw in parsed:
        summary={k:o[k] for k in ('snapshot_id','schema_version','contract_version','producer_version','producer_build','started_at','finished_at','consistency','quality','state_digest')}
        summary['snapshot_file_sha256']=sha(raw)
        summaries.append(summary)
    return dict(comparison=comp,units=units,scope=scope,resources={k:str(v) for k,v in resources.items()},
                measurement=oracle.aggregate(units,comp['quality']=='partial'),sources=dict(zip(SIDES,sources)),
                baseline=summaries[0],current=summaries[1],metric_policy=policy,metric_policy_id=policy_id(policy),
                foundation_quality=comp['quality'],foundation_relation=comp['relation'],
                **{k:objects[0][k] for k in ('workspace_id','path_namespace','policy_id')})


def report(value, paths, packages, limits=None, exit_code=None):
    schema(value,'report')
    domain(value)
    expected=model(paths,packages,limits)
    # Specific failure categories make mutation rejection attributable.
    for k in ('metric_policy_id','metric_policy','baseline','current','sources','comparison',
              'foundation_quality','foundation_relation','workspace_id','path_namespace','policy_id',
              'units','scope','measurement','resources'):
        equal(value[k],expected[k],'report_'+k)
    if exit_code is not None:
        equal(exit_code,{'complete':0,'partial':3,'unavailable':1}[expected['measurement']['status']],'report_exit')
    return expected
