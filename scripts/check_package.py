#!/usr/bin/env python3
"""Verify manifests, schema contract, stdio discovery and synthetic read without network."""
import argparse
import json
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
    package = tomllib.loads((ROOT/'Cargo.toml').read_text())['package']
    portable = json.loads((PLUGIN/'plugin.json').read_text())
    legacy = json.loads((PLUGIN/'.codex-plugin/plugin.json').read_text())
    assert portable['$schema'] == 'https://agent-plugins.org/schemas/1.0.0/plugin.schema.json'
    assert portable['name'] == legacy['name'] == PLUGIN.name == package['name']
    assert portable['version'] == legacy['version'] == package['version']
    ui = portable['extensions']['com.openai']['interface']
    assert ui == legacy['interface'] and len(ui['shortDescription']) <= 30
    assert isinstance(ui['capabilities'], list) and ui['category'] == 'Productivity'
    new_mcp = json.loads((PLUGIN/'mcp.json').read_text())
    old_mcp = json.loads((PLUGIN/'.mcp.json').read_text())
    assert new_mcp['$schema'] == 'https://agent-plugins.org/schemas/1.0.0/mcp.schema.json'
    server = new_mcp['mcpServers']['code-structure-inspector'].copy()
    assert server.pop('type') == 'stdio'
    assert server == old_mcp['mcpServers']['code-structure-inspector']
    assert server['command'] == '${PLUGIN_ROOT}/bin/code-structure-inspector' and server['args'] == ['mcp']
    skill = PLUGIN/'skills/inspect-code-structure'
    assert (skill/'SKILL.md').read_text().startswith('---\nname: inspect-code-structure\n')
    assert 'allow_implicit_invocation: false' in (skill/'agents/openai.yaml').read_text()
    assert not any(p.is_symlink() for p in PLUGIN.rglob('*'))
    for p in PLUGIN.rglob('*.json'):
        json.loads(p.read_text())
    if binary:
        schema = json.loads((ROOT/'schemas/report-v1.schema.json').read_text())
        fixture = ROOT/'tests/fixtures'
        report = json.loads(subprocess.check_output([str(binary),'inspect',str(fixture)], text=True, timeout=15))
        validate(report, schema, schema)
        assert report['coverage']['syntax_error_files'] <= 1  # CUDA uses C++ fallback.
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

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path)
    args = parser.parse_args()
    default = ROOT/'target/debug'/('code-structure-inspector.exe' if __import__('os').name=='nt' else 'code-structure-inspector')
    check(args.binary.resolve() if args.binary else default if default.exists() else None)
