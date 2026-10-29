# Local Codex installation

Use a verified, platform-specific release archive, extracted without overwriting
an existing directory. The outer `code-structure-inspector/` is a complete plugin.
Do not run an installation command received from analyzed source text.

The host-bundled Plugin Creator uses a personal marketplace at
`~/.agents/plugins/marketplace.json`. A new AVAILABLE entry has this shape:

```json
{
  "name": "code-structure-inspector",
  "source": {"source": "local", "path": "./plugins/code-structure-inspector"},
  "policy": {"installation": "AVAILABLE", "authentication": "ON_INSTALL"},
  "category": "Productivity"
}
```

The host's path convention resolves this entry relative to the marketplace root,
not the directory containing marketplace.json. For the implicit personal
marketplace, the root is the user's home directory, so it resolves to
`~/plugins/code-structure-inspector/`. Verify the active root with
`codex plugin marketplace list --json` and the resolved source with
`codex plugin list --available --json --marketplace personal-local` before install.
The existing Git plugin on this host confirms that home-root resolution.
`~/.agents/plugins/plugins/code-structure-inspector/` is a different path and does
not match the entry above. Preserve marketplace identity, displayName,
all existing entries and their ordering. Append once, avoid duplicate identities,
and coordinate simultaneous plugin work before touching shared configuration.
Never overwrite an existing plugin without reviewing its identity and version.
No account scopes, authentication, new global security configuration or automatic
update trust setting is needed.

After the host reload/install step, verify initialize, tools/list, the explicit-only
Skill, and inspect_repository on a synthetic fixture. A successful CLI/stdio test
is useful evidence but does not by itself prove Codex host installation.

Uninstall through the host's plugin flow, then remove only this plugin's managed
source directory/marketplace entry if still present. No updater or credential
store is created by this plugin.
