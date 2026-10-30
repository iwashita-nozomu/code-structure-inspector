#!/usr/bin/env python3
"""Verify manifests, schema contract, stdio discovery and synthetic read without network."""
import argparse
import json
import re
from pathlib import Path
import subprocess
import tomllib
ROOT = Path(__file__).resolve().parents[1]
PLUGIN = ROOT / 'plugins/code-structure-inspector'

def validate(value, schema, document):
    if '$ref' in schema:
        resolved = document
        for part in schema['$ref'].removeprefix('#/').split('/'):
            resolved = resolved[part]
        return validate(value, resolved, document)
    if 'const' in schema:
        assert value == schema['const']
    if 'pattern' in schema and isinstance(value, str):
        assert re.fullmatch(schema['pattern'], value)
    if 'enum' in schema:
        assert value in schema['enum']
    kind = schema.get('type')
    matches = {'object': isinstance(value, dict), 'array': isinstance(value, list),
               'string': isinstance(value, str), 'integer': isinstance(value, int) and not isinstance(value, bool),
               'boolean': isinstance(value, bool), 'null': value is None}
    if kind:
        assert any(matches[k] for k in ([kind] if isinstance(kind, str) else kind)), (kind, type(value))
    if isinstance(value, dict):
        assert set(schema.get('required', [])) <= value.keys()
        if schema.get('additionalProperties') is False:
            assert value.keys() <= schema.get('properties', {}).keys()
        for key, child in value.items():
            if key in schema.get('properties', {}):
                validate(child, schema['properties'][key], document)
    if isinstance(value, list):
        for child in value:
            validate(child, schema.get('items', {}), document)
    if isinstance(value, int) and not isinstance(value, bool) and 'minimum' in schema:
        assert value >= schema['minimum']

def check(binary=None):
    package = tomllib.loads((ROOT/'Cargo.toml').read_text(encoding="utf-8"))['package']
    portable = json.loads((PLUGIN/'plugin.json').read_text(encoding="utf-8"))
    legacy = json.loads((PLUGIN/'.codex-plugin/plugin.json').read_text(encoding="utf-8"))
    assert portable['$schema'] == 'https://agent-plugins.org/schemas/1.0.0/plugin.schema.json'
    assert portable['name'] == legacy['name'] == PLUGIN.name == package['name']
    assert portable['version'] == legacy['version'] == package['version']
    ui = portable['extensions']['com.openai']['interface']
    assert ui == legacy['interface'] and len(ui['shortDescription']) <= 30
    assert isinstance(ui['capabilities'], list) and ui['category'] == 'Productivity'
    new_mcp = json.loads((PLUGIN/'mcp.json').read_text(encoding="utf-8"))
    old_mcp = json.loads((PLUGIN/'.mcp.json').read_text(encoding="utf-8"))
    assert new_mcp['$schema'] == 'https://agent-plugins.org/schemas/1.0.0/mcp.schema.json'
    server = new_mcp['mcpServers']['code-structure-inspector'].copy()
    assert server.pop('type') == 'stdio'
    assert server == old_mcp['mcpServers']['code-structure-inspector']
    assert server['command'] == './bin/code-structure-inspector' and server['args'] == ['mcp']
    assert server['cwd'] == '${PLUGIN_ROOT}' and '${' not in server['command']
    skill = PLUGIN/'skills/inspect-code-structure'
    assert (skill/'SKILL.md').read_text(encoding="utf-8").startswith('---\nname: inspect-code-structure\n')
    assert 'allow_implicit_invocation: false' in (skill/'agents/openai.yaml').read_text(encoding="utf-8")
    workflow=(skill/'SKILL.md').read_text(encoding="utf-8")
    for reference in ['source-evidence.md','acceptance.md','figure-contract.md','selected-detail.md','structure-selection.js']:
        assert ('references/'+reference) in workflow
        text=(skill/'references'/reference).read_text(encoding="utf-8")
        assert len(text)>500
    assert 'このレポを見て' in workflow and '詳しく聞く' in workflow and 'sendFollowUpMessage' in workflow
    assert 'Codex GPT' in workflow and 'native visualize' in workflow
    assert not any(p.is_symlink() for p in PLUGIN.rglob('*'))
    for p in PLUGIN.rglob('*.json'):
        json.loads(p.read_text(encoding="utf-8"))
    if binary:
        schema = json.loads((ROOT/'schemas/report-v1.schema.json').read_text(encoding="utf-8"))
        fixture = ROOT/'tests/fixtures'
        report = json.loads(subprocess.check_output([str(binary),'inspect',str(fixture)], text=True, timeout=15))
        validate(report, schema, schema)
        assert report['coverage']['syntax_error_files'] <= 1  # CUDA uses C++ fallback.
        declarations = json.loads(subprocess.check_output([str(binary),'inspect',str(ROOT/'tests/declaration-fixtures')], text=True, timeout=15))
        validate(declarations, schema, schema)
        witnesses = {file['path']: {(s['name'],s['kind'],s['start_line'],s['end_line']) for s in file['symbols']} for file in declarations['files']}
        for path, expected in {
            'boundaries.rs': {('Worker','struct_item',2,4),('State','enum_item',5,7),('Runner','trait_item',8,10),('Count','type_item',11,11)},
            'boundaries.hpp': {('analysis','namespace_definition',1,17),('Model','class_specifier',2,7),('Data','struct_specifier',8,10),('Mode','enum_specifier',11,13),('Box','class_specifier',22,23),('outer::inner','namespace_definition',30,34)},
            'boundaries.cu': {('device','namespace_definition',1,8),('Buffer','struct_specifier',2,4),('update','function_definition',5,7)},
            'boundaries.py': {('Experiment','class_definition',1,3),('run','function_definition',2,3)},
        }.items():
            assert expected <= witnesses[path], ('Missing release declaration witnesses',path)
        semantics=json.loads(subprocess.check_output([str(binary),'inspect',str(ROOT/'tests/semantic-fixtures')], text=True, timeout=15))
        contract=json.loads((ROOT/'tests/semantic_contract.json').read_text(encoding="utf-8"))
        validate(semantics,schema,schema)
        assert semantics['dependencies']==contract['import_facts']
        import hashlib
        for file in semantics['files']:
            assert file['content_revision']=='sha256:'+hashlib.sha256((ROOT/'tests/semantic-fixtures'/file['path']).read_bytes()).hexdigest()
        for selected in contract['selections']:
            file=next(f for f in semantics['files'] if f['path']==selected['path'])
            assert any(s['name']==selected['symbol'] and s['start_line']==selected['start_line'] and s['end_line']==selected['end_line'] for s in file['symbols'])
        assert 'interpretations' not in semantics and 'responsibilities' not in semantics
        requests = [
            {'jsonrpc':'2.0','id':1,'method':'initialize','params':{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'package-check','version':'1'}}},
            {'jsonrpc':'2.0','method':'notifications/initialized'},
            {'jsonrpc':'2.0','id':2,'method':'tools/list'},
            {'jsonrpc':'2.0','id':3,'method':'tools/call','params':{'name':'inspect_repository','arguments':{'root':str(fixture)}}},
        ]
        out = subprocess.run([str(binary),'mcp'],input=''.join(json.dumps(r)+'\n' for r in requests), text=True,capture_output=True,check=True,timeout=20)
        messages = [json.loads(line) for line in out.stdout.splitlines()]
        assert not out.stderr and len(messages) == 3
        assert [t['name'] for t in messages[1]['result']['tools']] == ['inspect_repository','related_files']
        result = messages[2]['result']
        assert result['isError'] is False
        validate(json.loads(result['content'][0]['text']), schema, schema)
    print(json.dumps({'plugin':'verified','schema_version':1,'binary_smoke':bool(binary)}))

def check_packaged_launch(stage):
    stage=stage.resolve()
    for reference in ['SKILL.md','references/source-evidence.md','references/acceptance.md','references/figure-contract.md','references/selected-detail.md','references/structure-selection.js']:
        relative=Path('skills/inspect-code-structure')/reference
        assert (stage/relative).read_bytes()==(PLUGIN/relative).read_bytes()
    for filename in ['mcp.json','.mcp.json']:
        doc=json.loads((stage/filename).read_text(encoding="utf-8"))
        server=doc['mcpServers']['code-structure-inspector']
        command=server['command']
        assert command.startswith('./bin/') and '${' not in command
        assert server['cwd']=='${PLUGIN_ROOT}' and server['args']==['mcp']
        resolved=(stage/command).resolve(strict=True)
        assert resolved.is_relative_to(stage) and resolved.is_file() and not (stage/command).is_symlink()
        requests=[
            {'jsonrpc':'2.0','id':1,'method':'initialize','params':{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'manifest-launch-check','version':'1'}}},
            {'jsonrpc':'2.0','method':'notifications/initialized'},
            {'jsonrpc':'2.0','id':2,'method':'tools/list'},
        ]
        out=subprocess.run([str(resolved),*server['args']],cwd=stage,input=''.join(json.dumps(r)+'\n' for r in requests),text=True,capture_output=True,check=True,timeout=20)
        messages=[json.loads(line) for line in out.stdout.splitlines()]
        assert not out.stderr and len(messages)==2
        assert [t['name'] for t in messages[1]['result']['tools']]==['inspect_repository','related_files']
    print(json.dumps({'packaged_manifest_launch':'verified','host_discovery':'separate installation check required'}))

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path)
    args = parser.parse_args()
    default = ROOT/'target/debug'/('code-structure-inspector.exe' if __import__('os').name=='nt' else 'code-structure-inspector')
    check(args.binary.resolve() if args.binary else default if default.exists() else None)
