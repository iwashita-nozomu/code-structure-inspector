#!/usr/bin/env python3
"""Create a self-contained plugin archive; no install, publish, or network."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
import tomllib
import zipfile
from check_package import check, check_packaged_launch
ROOT = Path(__file__).resolve().parents[1]
NAME = 'code-structure-inspector'
TARGETS = {'aarch64-apple-darwin','x86_64-unknown-linux-gnu','x86_64-pc-windows-msvc'}

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--target', required=True, choices=sorted(TARGETS))
    parser.add_argument('--binary', required=True, type=Path)
    args=parser.parse_args()
    binary=args.binary.resolve()
    check(binary)
    subprocess.run(['python3' if os.name!='nt' else 'python',str(ROOT/'scripts/licenses.py'),'--check'],check=True)
    version=tomllib.loads((ROOT/'Cargo.toml').read_text(encoding="utf-8"))['package']['version']
    assert subprocess.check_output([str(binary),'--version'],text=True).strip()==f'{NAME} {version}'
    commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
    compiler=subprocess.check_output(['rustc','--version'],text=True).strip()
    dist=ROOT/'dist';dist.mkdir(exist_ok=True)
    suffix='.zip' if args.target.endswith('msvc') else '.tar.gz'
    output=dist/f'{NAME}-v{version}-{args.target}{suffix}'
    assert not output.exists(), 'Do not overwrite an existing release archive'
    with tempfile.TemporaryDirectory() as temp:
        stage=Path(temp)/NAME
        shutil.copytree(ROOT/'plugins'/NAME,stage)
        (stage/'bin').mkdir()
        executable=NAME+('.exe' if args.target.endswith('msvc') else '')
        shutil.copy2(binary,stage/'bin'/executable)
        (stage/'bin'/executable).chmod(0o755)
        if executable.endswith('.exe'):
            for p in [stage/'mcp.json',stage/'.mcp.json']:
                doc=json.loads(p.read_text(encoding="utf-8"))
                doc['mcpServers'][NAME]['command'] += '.exe'
                p.write_text(json.dumps(doc,indent=2)+'\n')
        for file in ['LICENSE','THIRD_PARTY_NOTICES.md','README.md','SECURITY.md']:
            shutil.copy2(ROOT/file,stage/file)
        for directory in ['third-party','schemas','docs']:
            shutil.copytree(ROOT/directory,stage/directory)
        # Repository links become package-root-relative in the shipped guide.
        semantic_guide=stage/'docs/SEMANTIC_WORKFLOW.md'
        semantic_guide.write_text(semantic_guide.read_text(encoding="utf-8").replace('../plugins/'+NAME+'/', '../'),encoding="utf-8")
        binary_hash=hashlib.sha256(binary.read_bytes()).hexdigest()
        (stage/'binary.sha256').write_text(f'{binary_hash}  bin/{executable}\n')
        (stage/'build-info.json').write_text(json.dumps({'version':version,'target':args.target,'source_commit':commit,'compiler':compiler,'binary_sha256':binary_hash},indent=2)+'\n')
        check_packaged_launch(stage)
        files=sorted(p for p in stage.rglob('*') if p.is_file())
        assert not any(p.is_symlink() for p in stage.rglob('*'))
        if suffix=='.zip':
            with zipfile.ZipFile(output,'w',compression=zipfile.ZIP_DEFLATED) as archive:
                for p in files: archive.write(p,p.relative_to(Path(temp)).as_posix())
            with zipfile.ZipFile(output) as archive: names=archive.namelist()
        else:
            with tarfile.open(output,'w:gz') as archive:
                for p in files: archive.add(p,p.relative_to(Path(temp)).as_posix(),recursive=False)
            with tarfile.open(output) as archive:
                assert all(m.isfile() for m in archive.getmembers())
                names=archive.getnames()
        assert f'{NAME}/.codex-plugin/plugin.json' in names and f'{NAME}/plugin.json' in names
        assert f'{NAME}/bin/{executable}' in names and f'{NAME}/third-party/manifest.json' in names
        assert not any(n.startswith('/') or '..' in Path(n).parts for n in names)
    digest=hashlib.sha256(output.read_bytes()).hexdigest()
    (dist/(output.name+'.sha256')).write_text(f'{digest}  {output.name}\n')
    print(json.dumps({'archive':output.name,'sha256':digest,'source_commit':commit}))

if __name__=='__main__': main()
