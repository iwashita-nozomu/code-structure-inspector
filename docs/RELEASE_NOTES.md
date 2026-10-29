# Release notes

## 0.2.0

Add an explicit Codex GPT semantic workflow: observe structural facts, read a
small selection of already-authorized source through the host, explain component
responsibilities/dependency meaning/candidate impact with witnesses, and return a
native visualization. Missing source yields an explicit evidence gap; misleading
file names are not substituted for semantic understanding.

Bundle selection and diagram acceptance references and synthetic misleading-name
fixtures. Keep machine facts, GPT interpretations and unknowns distinct. The
runtime adds no model API, reader tool, network, persistent state or orchestrator.
CI checks the fixture evidence contract; it does not call an AI service or claim
to prove every future GPT answer. Existing protocol/schema/tool surface is retained.

## 0.1.2

Use the host-compatible relative MCP executable `./bin/code-structure-inspector`
(`.exe` on Windows) with `cwd: ${PLUGIN_ROOT}` in both portable and compatibility
MCP manifests. The executable command no longer depends on variable expansion
inside `command`. This matches the working Git plugin on the current Mac host.

Packaging now resolves and executes both manifests from the staged plugin root,
checking MCP initialize and tool discovery. That packaging check is distinct from
discovery in the Codex host, which must be verified after installation.

## 0.1.1

Fix omitted declaration names when Tree-sitter represents them as
`type_identifier` or `namespace_identifier`. Rust structs/enums/traits/type aliases
and C++ classes/structs/enums/namespaces now retain their declaration witnesses.
The same correction applies to the C++ syntax fallback for CUDA.

Language-specific synthetic regression cases assert every recognized
declaration's name, node kind and exact source line range, including Rust module
and function, Python class and methods, C++ methods/functions, and a CUDA kernel.
The new tests reproduce the omission before the fix; all pass after it.

Also fix Rust `use` candidate selection: determine ambiguity among the two
file layouts at the longest matching module prefix, before considering a
shallower symbol-suffix fallback. Two same-depth candidates now stay unresolved
with `resolution: ambiguous`, consistently with out-of-line `mod` declarations.

Serialize file IDs from OS-native path components. Literal backslashes in Unix
filenames are explicitly skipped with `unsupported_path_encoding` coverage
rather than being rewritten into separators and colliding with another file.
Regression tests check distinct IDs, include resolution and source-path round trips.

The analysis remains syntactic and bounded. Read-only filesystem/execution
boundaries are unchanged.
