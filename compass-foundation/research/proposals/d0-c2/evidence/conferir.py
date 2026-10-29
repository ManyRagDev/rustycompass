#!/usr/bin/env python3
"""Conferência documental C2: literais pequenos, transporte e hashes; não sensor/V1.

Uso: python3 .../conferir.py [--cached-dependencies /home/many/.cache/uv/archive-v0]
Não importa Rust, não captura acervos, não executa CLI nem altera esperados.
"""
import argparse
import base64
import copy
import hashlib
import itertools
import json
import sys
from decimal import Decimal, localcontext
from pathlib import Path
from urllib.parse import unquote, urlparse

parser = argparse.ArgumentParser()
parser.add_argument('--cached-dependencies', type=Path)
args = parser.parse_args()
if args.cached_dependencies:
    for p in sorted(args.cached_dependencies.iterdir()):
        if any((p / n).exists() for n in ('jsonschema', 'attrs', 'attr', 'referencing',
                                         'rpds', 'jsonschema_specifications', 'typing_extensions.py')):
            sys.path.insert(0, str(p))
from importlib.metadata import version
from jsonschema import Draft202012Validator, FormatChecker
from referencing import Registry, Resource

ROOT = Path(__file__).resolve().parents[1]
FIX = ROOT / 'fixtures'
checks = {}
def inc(name):
    checks[name] = checks.get(name, 0) + 1

def strict(b):
    def pairs(xs):
        d = {}
        for k, v in xs:
            if k in d:
                raise ValueError('duplicate key')
            d[k] = v
        return d
    return json.loads(b.decode('utf-8'), object_pairs_hook=pairs)

def load(p):
    return strict(p.read_bytes())

def canonical_fixture(o):
    # Restricted fixture set: all property names ASCII; no JSON numbers;
    # all serialized strings ASCII (Unicode only occurs in cases, not JCS vectors).
    def check(x):
        if isinstance(x, dict):
            for k, v in x.items():
                assert k.isascii()
                check(v)
        elif isinstance(x, list):
            for v in x: check(v)
        elif isinstance(x, str): assert x.isascii()
        else: assert x is None or isinstance(x, bool)
    check(o)
    return json.dumps(o, sort_keys=True, separators=(',', ':'), ensure_ascii=True).encode('ascii')

def sha(b): return hashlib.sha256(b).hexdigest()

def subsequences(xs):
    return {tuple(xs[i] for i in range(len(xs)) if mask & (1 << i))
            for mask in range(1 << len(xs))}

def lcs(x, y):
    # Enumeration of subsequences, no DP/diff library/production.
    return max(map(len, subsequences(x) & subsequences(y)))

def decimal_ratio(n, d):
    # Independent Decimal reference; production contract calls for integer division.
    with localcontext() as ctx:
        ctx.prec = 80
        return format((Decimal(n) / Decimal(d)).quantize(Decimal('0.000001')), 'f')

cases = load(FIX / 'cases.json')
assert [c['id'] for c in cases] == [f'E{i:02}' for i in range(1, 56)]
for c in cases:
    inc('E01_E55_catalogued')
    for calc in c['calculations']:
        x, y = calc['baseline_lines'], calc['current_lines']
        e = calc['expected']; n0, nt = len(x), len(y); ll = lcs(x, y)
        assert [int(e[k]) for k in ('n0','nt','l','a','r','c','m')] == [n0,nt,ll,nt-ll,n0-ll,n0+nt-2*ll,n0+nt]
        assert lcs(y, x) == ll
        inc('literal_lcs_calculations')
    e = c['expected']
    if 'C' in e and c['calculations']:
        C, M = int(e['C']), int(e['M'])
        assert C == sum(int(x['expected']['c']) for x in c['calculations'])
        assert M == sum(int(x['expected']['m']) for x in c['calculations'])
        assert e['decimal'] == decimal_ratio(C, M or 1)
        inc('literal_aggregates')
# Small exhaustive mathematical properties, not a domain/IO verifier.
seqs = [list(x) for n in range(5) for x in itertools.product(('a','b'), repeat=n)]
for x in seqs:
    for y in seqs:
        ll = lcs(x, y); m = len(x)+len(y); c = m-2*ll
        assert ll == lcs(y, x) and 0 <= c <= m
        assert lcs(x, x) == len(x)
        inc('small_property_pairs')
assert lcs(['a','b'],['a','c']) == 1
assert lcs(['a','b'],['a','b']) == 2
inc('A_B_A_extremes')
for n, d, expected in cases[48]['expected']['ratios']:
    assert decimal_ratio(int(n), int(d)) == expected
    inc('rounding_boundary_vectors')
# Historical difference only; no scientific efficacy inference.
with localcontext() as ctx:
    ctx.prec = 60
    log_example = Decimal(2).ln() / (Decimal(2).ln() + Decimal(10).ln())
    historical_decimal = format(log_example.quantize(Decimal('0.000001')), 'f')
assert historical_decimal == '0.231378'
inc('historical_weight_comparison')

def lines(b):
    s = b.decode('utf-8', errors='strict')
    assert '\0' not in s
    if s.startswith('\ufeff'): s=s[1:]
    s=s.replace('\r\n','\n')
    if not s: return []
    xs=s.split('\n')
    return xs[:-1] if s.endswith('\n') else xs
for vec in load(FIX / 'byte-vectors.json'):
    for side in ('baseline','current'):
        b=bytes.fromhex(vec[side+'_hex'])
        assert lines(b) == vec[side+'_lines']
        assert sha(b) == vec[side+'_sha256']
        inc('literal_normalization_hash_sides')
hashes=load(FIX/'hash-vectors.json')
for vec in hashes['sha256_vectors']:
    b=bytes.fromhex(vec['hex'])
    assert len(b)==int(vec['size']) and sha(b)==vec['sha256']
    inc('raw_sha256_vectors')
pb=(FIX/'metric-policy.jcs').read_bytes();policy=load(FIX/'metric-policy.json')
assert pb==canonical_fixture(policy)
assert sha(pb)==hashes['metric_policy_jcs_sha256']
assert sha(bytes.fromhex(hashes['metric_policy_prefix_hex'])+pb)==hashes['metric_policy_id']
inc('metric_policy_hash')
# Local official JSON Schema validator with reference resolution restricted to files.
def retrieve(uri):
    u=urlparse(uri)
    assert u.scheme=='file'
    return Resource.from_contents(load(Path(unquote(u.path))))
registry=Registry(retrieve=retrieve)
validators={}
for p in sorted((ROOT/'schemas').glob('*.schema.json')):
    schema=load(p);Draft202012Validator.check_schema(schema)
    uri=p.as_uri();reg=registry.with_resource(uri,Resource.from_contents(schema))
    validators[p.stem.split('.')[0]]=Draft202012Validator({'$ref':uri},registry=reg,format_checker=FormatChecker())
    inc('schemas_metaschema')
validators['limits'].validate(load(FIX/'limits-default.json'))
validators['limits'].validate(load(FIX/'limits-reduced.json'))
validators['metric-policy'].validate(policy)
checks['limits_and_policy_fixtures']=3
for p in sorted((FIX/'packages').glob('*/source.json')):
    src=load(p);validators['source'].validate(src)
    manifest=next(q for q in (FIX/'manifests').glob('*.json') if load(q)['snapshot_id']==src['snapshot_id'])
    m=load(manifest)
    assert src['snapshot_file_sha256']==sha(manifest.read_bytes())
    assert all(src[k]==m[k] for k in ('snapshot_id','workspace_id','policy_id'))
    inc('descriptor_bindings')
# Noncanonical but valid descriptor has a different raw hash, same logical binding.
a=(FIX/'packages/a/source.json').read_bytes();w=(FIX/'packages/a-whitespace/source.json').read_bytes()
assert strict(a)==strict(w) and sha(a)!=sha(w)
inc('noncanonical_descriptor_accepted')
try: strict(b'{"a":1,"\\u0061":2}')
except ValueError: inc('duplicate_key_rejected')
else: raise AssertionError('duplicate key accepted')
foundation=ROOT.parents[1]/'contracts/transport.schema.json'
fv=Draft202012Validator(load(foundation),format_checker=FormatChecker())
for p in sorted((FIX/'manifests').glob('*.json')):
    m=load(p);fv.validate(m)
    fp=canonical_fixture(m['policy'])
    assert sha(b'compass-policy-v1\0'+fp)==m['policy_id']
    if m['quality']=='complete':
        files=[{k:e[k] for k in ('path_key','content_sha256','size_bytes','executable')} for e in m['entries'] if e['state']=='observed']
        state={k:m[k] for k in ('workspace_id','path_namespace','policy_id')};state['files']=files
        assert sha(b'compass-state-v1\0'+canonical_fixture(state))==m['state_digest']
    else: assert m['state_digest'] is None
    inc('candidate_manifests_schema_hashes')
# Minimal semantic cross-checks for reviewed full reports. This is not the future V1.
def report_semantics(r):
    assert r['foundation_quality']==r['comparison']['quality']
    assert r['foundation_relation']==r['comparison']['relation']
    assert r['metric_policy_id']==sha(b'compass-d0-policy-v1\0'+canonical_fixture(r['metric_policy']))
    available=[u for u in r['units'] if u['status']=='available']
    for u in available:
        c=u['calculation'];n0,nt,ll,a,rr,cc,m=[int(c[k]) for k in ('n0','nt','l','a','r','c','m')]
        assert ll<=min(n0,nt) and a==nt-ll and rr==n0-ll and cc==a+rr and m==n0+nt
        assert c['delta']=={'numerator':str(cc),'denominator':str(m or 1),'decimal':decimal_ratio(cc,m or 1)}
        assert all(s['state'] in ('verified','absent') for s in u['checks'].values())
    C=sum(int(u['calculation']['c']) for u in available);M=sum(int(u['calculation']['m']) for u in available)
    meas=r['measurement'];v={'numerator':str(C),'denominator':str(M or 1),'decimal':decimal_ratio(C,M or 1)}
    blocked=r['foundation_quality']=='partial' or any(u['status']=='unavailable' for u in r['units'])
    status='partial' if blocked and available else 'unavailable' if blocked or r['units'] and not available else 'complete'
    assert meas['status']==status
    assert meas['value']==(v if status=='complete' else None)
    assert meas['line_mass']==(str(M) if status=='complete' else None)
    reasons=sorted((['foundation_partial'] if r['foundation_quality']=='partial' else [])+(['unresolved_units'] if any(u['status']=='unavailable' for u in r['units']) else [])+(['no_text_units'] if not available and status!='complete' else []))
    assert meas['reasons']==reasons
    if available: assert meas['known_subset']['value']==v
    else: assert meas['known_subset'] is None
    assert int(r['scope']['candidates'])==len(r['units']) and int(r['scope']['resolved_text_units'])==len(available)
    assert int(r['resources']['object_bytes_read'])<=int(r['resources']['object_bytes_reserved'])<=int(r['metric_policy']['limits']['max_total_object_bytes'])
    for u in r['units']:
        assert ('calculation' in u)==(u['status']=='available')
        for side,s in u['checks'].items():
            if s['state']=='verified':
                assert s['expected_revision']==s['actual_revision'] and s['hash_check'] and s['size_check']
            if s['source_ref'] is not None: assert s['source_ref']==r['sources'][side]['source_ref']
    assert [u['path_key'] for u in r['units']]==sorted((u['path_key'] for u in r['units']),key=lambda k:base64.urlsafe_b64decode(k+'='*((-len(k))%4)))
for p in sorted((FIX/'reports').glob('*.json')):
    r=load(p);validators['report'].validate(r);report_semantics(r)
    assert p.with_suffix('.jcs').read_bytes()==canonical_fixture(r)
    inc('reports_schema_semantics_canonical')
# Every report has concrete manifest/package refs, and physical buffers explain checks.
for name, setup in load(FIX/'report-inputs.json').items():
    r=load(FIX/setup['expected_report']);seen=set();physical=0;reserved=0;report_cells_reserved=0
    manifests={side:load(FIX/setup[side+'_manifest']) for side in ('baseline','current')}
    S=8*sum((FIX/setup[side+'_manifest']).stat().st_size for side in ('baseline','current'))
    N=sum(len(m['entries'])+len(m['directory_coverage'])+len(m['issues']) for m in manifests.values())
    for side in ('baseline','current'):
        m=manifests[side];summary=r[side]
        assert summary['snapshot_file_sha256']==sha((FIX/setup[side+'_manifest']).read_bytes())
        for k in ('snapshot_id','state_digest','quality','started_at','finished_at'):
            assert summary[k]==m[k]
        package=setup[side+'_package']
        if package:
            db=(FIX/package/'source.json').read_bytes();ds=strict(db)
            S+=8*len(db)
            assert r['sources'][side]=={**ds,'source_ref':sha(db)}
        else: assert r['sources'][side] is None
    S+=4096*N+65536;peak=S
    for u in r['units']:
        if u['classification'] not in ('unknown','left_scope','entered_scope'):
            sizes=[int(c['expected_revision']['size_bytes']) if c['expected_revision'] else 0 for c in u['checks'].values()]
            Wb=2*(sum(sizes)+2)+65536;peak=max(peak,S+Wb)
            if u['status']=='available':
                c=u['calculation'];nn0=int(c['n0']);nnt=int(c['nt'])
                peak=max(peak,S+Wb+32*(nn0+nnt)+8*(min(nn0,nnt)+1))
        for side,c in u['checks'].items():
            if c['state'] not in ('verified','failed'): continue
            package=setup[side+'_package'];e=c['expected_revision']
            path=FIX/package/'objects'/e['content_sha256'];b=path.read_bytes()
            assert len(b)<=int(e['size_bytes']) # these EOF-complete fixtures only
            actual={'content_sha256':sha(b),'size_bytes':str(len(b))}
            assert actual==c['actual_revision']
            assert c['hash_check']==(actual['content_sha256']==e['content_sha256'])
            assert c['size_check']==(actual['size_bytes']==e['size_bytes'])
            if str(path) not in seen:
                seen.add(str(path));physical+=len(b);reserved+=int(e['size_bytes'])+1
        if u['status']=='available':
            xs=[]
            for side,c in u['checks'].items():
                if c['state']=='absent': xs.append([])
                else: xs.append(lines((FIX/setup[side+'_package']/'objects'/c['expected_revision']['content_sha256']).read_bytes()))
            assert lcs(*xs)==int(u['calculation']['l'])
            report_cells_reserved += 0 if xs[0] == xs[1] else len(xs[0]) * len(xs[1])
    assert r['resources']['object_bytes_read']==str(physical)
    assert r['resources']['object_bytes_reserved']==str(reserved)
    assert r['resources']['logical_memory_peak']==str(peak)
    assert r['resources']['cells_reserved']==str(report_cells_reserved)
    assert setup['expected_exit']=={'complete':0,'partial':3,'unavailable':1}[r['measurement']['status']]
    inc('report_inputs_same_buffer_budget_checks')
# Deliberate asymmetric admission counterexample E45, no filesystem timing involved.
def two_reservations(sizes, budget):
    successes=[]
    for size in sizes:
        cost=size+1
        ok=cost<=budget
        if ok: budget-=cost
        successes.append(ok)
    return successes,budget
forward,left=two_reservations([2,0],3);reverse,right=two_reservations([0,2],3)
assert forward==[True,False] and reverse==[True,False] and left==0 and right==2
assert two_reservations([0,0],left)[0]==[False,False]
assert two_reservations([0,0],right)[0]==[True,True]
inc('byte_budget_subset_asymmetry')
# Literal nontext precedence: strict UTF8 first, then NUL.
for b,expected in [(b'\x00\xff','invalid_utf8'),(b'\x00a','nul'),(b'\xff\xfea\x00','invalid_utf8')]:
    try:
        decoded=b.decode('utf-8');reason='nul' if '\0' in decoded else None
    except UnicodeDecodeError:reason='invalid_utf8'
    assert reason==expected
    inc('non_text_literal_classification')
for p in sorted((FIX/'errors').glob('*.json')):
    validators['error'].validate(load(p));inc('error_fixtures')
# Negative transport checks and a deliberately schema-valid semantic error.
good=load(FIX/'reports/complete.json')
negative=[]
x=copy.deepcopy(good);del x['measurement']['zero_basis'];negative.append(x)
x=copy.deepcopy(good);x['units'][0]['status']='mystery';negative.append(x)
x=copy.deepcopy(good);x['units'][0]['calculation']['m']='02';negative.append(x)
x=load(FIX/'reports/unavailable-source.json');x['units'][0]['calculation']=good['units'][0]['calculation'];negative.append(x)
x=copy.deepcopy(good);x['unapproved']=True;negative.append(x)
x=copy.deepcopy(good);x['measurement']['value']=None;negative.append(x)
for x in negative:
    assert list(validators['report'].iter_errors(x));inc('negative_schema_rejections')
x=copy.deepcopy(good);x['units'][0]['calculation']['m']='3';validators['report'].validate(x)
try: report_semantics(x)
except AssertionError: inc('schema_valid_semantics_rejected')
else: raise AssertionError('invalid semantics accepted')
# Closed-form budget fronts, independent of allocator/production.
assert 8*(100+100+10+10)+4096*4+65536==83680
assert 2*(2+2+2)+65536==65548
assert 65548+32*(1+1)+8*(1+1)==65628
assert 83680+65628==149308
inc('logical_memory_literal_front')
def cell_reservation(x, y):
    # This is the D0-C2-A1 policy, reached only after source/text/normalization/
    # line-limit/memory admission. It does not infer equality from metadata.
    return 0 if x == y else len(x) * len(y)

equal_4k = ['same'] * 4000
different_4k = ['same'] * 3999 + ['other']
assert cell_reservation(equal_4k, equal_4k) == 0
assert len(equal_4k) + len(equal_4k) == 8000
e36 = next(c for c in cases if c['id'] == 'E36')['expected']
assert e36 == {'C':'0','M':'8000','cells_reserved':'0','decimal':'0.000000','l':'4000','n0':'4000','nt':'4000','status':'complete'}
assert 4000 * 4000 > int(policy['limits']['max_cells_per_unit'])
assert cell_reservation(equal_4k, different_4k) == 16000000
inc('exact_equal_over_cell_cap_zero_reservation')
assert lines(b'a\r\n') == lines(b'a\n') == ['a']
assert sha(b'a\r\n') != sha(b'a\n')
assert cell_reservation(lines(b'a\r\n'), lines(b'a\n')) == 0
inc('normalization_equal_bytes_differ_zero_reservation')
remaining=4
for x,y in ((['a','b'],['a','c']), (['z'],['z'])):
    reservation=cell_reservation(x,y)
    assert reservation <= remaining
    remaining -= reservation
assert remaining == 0
inc('equal_after_zero_balance_zero_reservation')
remaining=4
for x,y in ((['a','b'],['a','b']), (['a','b'],['a','c'])):
    reservation=cell_reservation(x,y)
    assert reservation <= remaining
    remaining -= reservation
assert remaining == 0
inc('equal_does_not_consume_following_balance')
for report_name, reason in (('unavailable-source','source_missing'), ('unavailable-hash','hash_mismatch')):
    report=load(FIX/'reports'/(report_name+'.json'))
    assert report['units'][0]['status']=='unavailable'
    assert reason in report['units'][0]['reasons']
    assert report['resources']['cells_reserved']=='0'
inc('invalid_source_blocks_before_equality')
# Byte reservation example: s+1 is never refunded even after missing object.
remaining=3;remaining-=2+1;assert remaining==0 and 0+1>remaining
inc('object_reservation_nonrefundable')
paths=[b'\xff',b'\xfb',b'a',b'A']
assert sorted(paths)==[b'A',b'a',b'\xfb',b'\xff']
assert sorted(base64.urlsafe_b64encode(p).decode().rstrip('=') for p in paths)!=[base64.urlsafe_b64encode(p).decode().rstrip('=') for p in sorted(paths)]
inc('native_order_not_base64')
front=load(FIX/'resource-frontiers.json')
for f in front['file_bytes']:
    admitted=int(f['declared'])<=4194304
    assert f['file_preflight']==('admitted' if admitted else 'file_bytes_limit')
    assert f['object_reservation']==(str(int(f['declared'])+1) if admitted else None)
    inc('resource_frontiers')
for f in front['lines']:
    admitted=max(int(f['n0']),int(f['nt']))<=100000
    assert f['line_preflight']==('admitted' if admitted else 'line_count_limit')
    assert f['cells']==(str(int(f['n0'])*int(f['nt'])) if admitted else None)
    inc('resource_frontiers')
for f in front['cells_per_unit']:
    cells=int(f['n0'])*int(f['nt'])
    if f['normalized_sequences_exactly_equal']:
        assert f['cells']=='0' and f['l']==f['n0']
        assert f['preflight']=='exact_equal_zero_cells'
    else:
        assert f['cells']==str(cells)
        assert f['preflight']==('admitted' if cells<=10000000 else 'diff_work_limit')
    inc('resource_frontiers')
for schedule in front['cell_schedule']:
    remaining=int(schedule['max_cells_total'])
    for unit in schedule['units']:
        cells=0 if unit['equal'] else int(unit['n0'])*int(unit['nt'])
        assert unit['cells']==str(cells) and cells<=remaining
        remaining-=cells
    assert remaining==int(schedule['remaining'])
    inc('equal_cell_schedules')
for f in front['logical_memory']:
    assert f['preflight']==('admitted' if int(f['S'])+int(f['W'])<=int(f['budget']) else 'memory_budget')
    inc('resource_frontiers')
for f in front['u64']:
    assert f['semantic_domain']==('admitted' if int(f['literal'])<=2**64-1 else 'rejected')
    inc('resource_frontiers')
# Build-input identity, without running/building production.
project=ROOT.parents[2];files=[]
for n in ('Cargo.toml','Cargo.lock','rust-toolchain.toml','crates','research/contracts'):
    p=project/n;files.extend([p] if p.is_file() else (q for q in p.rglob('*') if q.is_file()))
h=hashlib.sha256(b'compass-build-v1\0')
for p in sorted(files):
    path=p.relative_to(project).as_posix().encode();b=p.read_bytes()
    h.update(len(path).to_bytes(8,'little'));h.update(path);h.update(len(b).to_bytes(8,'little'));h.update(b)
build_id='sha256:'+h.hexdigest()
release=load(project/'verification/reference/release-verification.json')
assert build_id in json.dumps(release)
inc('foundation_build_inputs_identity')
print(json.dumps({'status':'passed','python':sys.version.split()[0],'jsonschema':version('jsonschema'),
                  'checks':checks,'historical_E07_decimal':historical_decimal,
                  'metric_policy_id':hashes['metric_policy_id'],'foundation_build_input_count':len(files),
                  'foundation_build_id':build_id,
                  'limitations':['No production tests/build/CLI/sensor execution',
                                 'No filesystem race/FIFO/deadline measurements: V1/I1',
                                 'Only restricted ASCII/string-only canonical fixture vectors; not general JCS',
                                 'Semantic report checks cover these small fixtures, not a full independent D0 verifier']},indent=2,sort_keys=True))
