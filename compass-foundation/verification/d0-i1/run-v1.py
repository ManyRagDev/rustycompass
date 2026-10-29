#!/usr/bin/env python3
"""Historical, authenticated lab environment; always execute the real I1 binary."""
import sys
sys.dont_write_bytecode = True
import argparse
import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import shutil
import subprocess
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parents[2]
HERE = ROOT/'verification/d0-i1'
INDEX = '63168b321845393168134a30afa39811e702eecc678b31c06f3bc24d3c17208e'


def sha(data): return hashlib.sha256(data).hexdigest()

def load(p): return json.loads(p.read_bytes())

def require(ok, reason):
    if not ok: raise ValueError(reason)


def historical(root):
    result={}
    for record in load(root/'research/proposals/d0-c2/evidence/preservacao.json')['protected_files']:
        relative=record['path']
        name=('compass-documento-principal/'+relative.split('/')[-1] if relative.startswith('../') else 'compass-foundation/'+relative)
        result[name]=(root/relative,record['after_sha256'])
    require(len(result)==34,'historical count')
    return result


def v1_hashes(root):
    return {str(p.relative_to(root)):sha(p.read_bytes()) for p in sorted((root/'verification/d0').rglob('*'))
            if p.is_file() and not {'results','__pycache__'} & set(p.parts)}


def frozen(root):
    base=root/'research/proposals/d0-c2'
    raw=(base/'SHA256SUMS').read_bytes()
    require(sha(raw)==INDEX,'index hash')
    lines=raw.decode().splitlines();require(len(lines)==81,'artifact count')
    for line in lines:
        digest,name=line.split('  ',1)
        require(sha((base/name).read_bytes())==digest,'artifact hash: '+name)


def verify(root=ROOT):
    meta=load(root/'verification/d0-i1/baseline.json')
    raw=(root/'verification/d0-i1/baseline-foundation.tar').read_bytes()
    require(sha(raw)==meta['tar_sha256'],'tar hash')
    require(v1_hashes(root)==meta['v1_sha256'],'V1 changed')
    require(sha((root/'docs/entregas/d0/OS-D0-I1.md').read_bytes())==meta['os_sha256'],'OS changed')
    frozen(root)
    expected=historical(root)
    require(meta['members_sha256']=={k:h for k,(_,h) in expected.items()},'historical inventory mismatch')
    contents={}
    with tarfile.open(fileobj=io.BytesIO(raw),mode='r:') as tar:
        for member in tar.getmembers():
            name=member.name;p=PurePosixPath(name)
            require(member.isfile() and not p.is_absolute() and '..' not in p.parts
                    and str(p)==name and name in expected and name not in contents,'forbidden tar member: '+name)
            data=tar.extractfile(member).read()
            require(sha(data)==expected[name][1],'member hash: '+name)
            contents[name]=data
    require(set(contents)==set(expected),'missing tar member')
    return contents,meta


def prepare(root=ROOT):
    here=root/'verification/d0-i1'
    if (here/'baseline.json').exists() or (here/'baseline-foundation.tar').exists():
        verify(root);print('BASELINE_REUSED');return
    frozen(root)
    expected=historical(root)
    for name,(path,digest) in expected.items():
        require(path.is_file() and not path.is_symlink() and sha(path.read_bytes())==digest,'historical mismatch: '+name)
    stream=io.BytesIO()
    with tarfile.open(fileobj=stream,mode='w:') as tar:
        for name,(path,_) in sorted(expected.items()):
            raw=path.read_bytes();entry=tarfile.TarInfo(name);entry.size=len(raw);entry.mode=0o644
            tar.addfile(entry,io.BytesIO(raw))
    data=stream.getvalue()
    meta=dict(tar_sha256=sha(data),members_sha256={k:h for k,(_,h) in expected.items()},
              v1_sha256=v1_hashes(root),
              v1_existing_sha256={str(p.relative_to(root)):sha(p.read_bytes()) for p in (root/'verification/d0').rglob('*') if p.is_file()},
              os_sha256=sha((root/'docs/entregas/d0/OS-D0-I1.md').read_bytes()))
    (here/'baseline-foundation.tar').write_bytes(data)
    (here/'baseline.json').write_text(json.dumps(meta,indent=2,sort_keys=True)+'\n')
    verify(root);print('BASELINE_PREPARED',meta['tar_sha256'])


def mount(destination, root=ROOT):
    contents,_=verify(root)
    for name,data in contents.items():
        p=destination/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(data)
    lab=destination/'compass-foundation'
    shutil.copytree(root/'verification/d0',lab/'verification/d0',ignore=shutil.ignore_patterns('results','__pycache__'))
    shutil.copytree(root/'research/proposals/d0-c2',lab/'research/proposals/d0-c2')
    for name in ('research/proposals/D0_C2_REVISAO.md','research/audits/AUDITORIA-OS-D0-C2-A1.md'):
        target=lab/name;target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(root/name,target)
    return lab


def run(binary, cache=None, root=ROOT):
    binary=Path(binary).resolve(strict=True)
    before=sha(binary.read_bytes())
    codes=[]
    with tempfile.TemporaryDirectory(prefix='compass-i1-v1-') as tmp:
        lab=mount(Path(tmp),root)
        print('LAB',lab,'SOURCE_I1',root,'REAL_BINARY',binary,'SHA256',before,flush=True)
        try:
            for command,result in [('acceptance.py','acceptance.json'),('mutations.py','mutations.json')]:
                args=[sys.executable,str(lab/'verification/d0'/command),'--binary',str(binary)]
                if cache: args.extend(['--cached-dependencies',str(cache)])
                process=subprocess.run(args,timeout=600)
                codes.append(process.returncode)
                output=lab/'verification/d0/results'/result
                if output.exists():shutil.copyfile(output,root/'verification/d0-i1'/result)
                else:require(process.returncode!=0,'missing result from successful lab')
                print('V1_EXIT',command,process.returncode,flush=True)
        finally:
            after=sha(binary.read_bytes());print('BINARY_AFTER',after,flush=True)
            require(after==before,'binary changed during campaign')
    return next((c if 0<c<256 else 1 for c in codes if c!=0),0)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    mode=parser.add_mutually_exclusive_group(required=True)
    mode.add_argument('--prepare-baseline',action='store_true')
    mode.add_argument('--binary',type=Path)
    parser.add_argument('--cached-dependencies',type=Path)
    args=parser.parse_args()
    try:
        if args.prepare_baseline:prepare();return 0
        return run(args.binary,args.cached_dependencies)
    except (OSError,ValueError,subprocess.TimeoutExpired,tarfile.TarError) as error:
        print('FAILED:',error,file=sys.stderr);return 1


if __name__=='__main__':sys.exit(main())
