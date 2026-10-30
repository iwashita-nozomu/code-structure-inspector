# Code Structure Inspector

A Rust CLI and local Codex plugin that turns an explicitly authorized repository
into bounded, file/line-backed dependency evidence. MIT project code; dependency
licenses are preserved in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

## What it provides

- Rust, Python and C/C++ declaration names and source line ranges via Tree-sitter.
  CUDA files use the C++ grammar; CUDA syntax failures are visible.
- Literal C/C++ includes, Python imports, Rust `mod`/`use` declarations and
  Cargo/PEP 621/requirements dependency declarations with source line witnesses.
- Local file layout candidates, unresolved/ambiguous dependencies, strongly
  connected groups, direct related files and a Mermaid file dependency diagram.
- JSON schema v1, CLI and two read-only MCP tools: `inspect_repository`,
  `related_files`. An explicit-only Skill uses Codex GPT to explain responsibilities,
  dependency meaning and candidate change impact from selected source witnesses,
  then returns a native visualization. No external AI API or custom LLM runtime.

This is syntactic evidence, not a complete call graph. Responsibilities are
interpretations for Codex to support with declaration and source evidence. No
compiler, build, import, shell, compile database, renderer or repository-provided
command is run. See [analysis limits](docs/ANALYSIS.md) before drawing conclusions.

The intended execution target is **Codex Work with an enabled local stdio MCP
connection and authorized source reads**. Check the actual tools exposed to the
current turn; a Work label alone is insufficient. The new interactive flow has
not completed real-host acceptance even in that target.

Visualize display is a separate capability from local MCP execution. This
package's local marketplace/stdio availability in ordinary ChatGPT Chat (including
Mac Chat) is unverified; no permanent limitation is inferred. Viewing a Mac
conversation on Windows does not move or invoke the Mac's MCP process. Native
visualize and `sendFollowUpMessage` must be checked independently. A component API
or bundled Skill documenting that method does not guarantee it is exposed to a
native Visualize fragment. Detect it there and offer a copyable reference question
if absent/rejected; an API return does not prove confirmation, sending or arrival.
If local MCP is absent, use only already available evidence for a labeled fallback.
If MCP works but source reads are unavailable, permitted structural observations
may still be shown with meanings unconfirmed. A copied question supplies no
missing execution. Do not bypass missing/denied tools via CLI or another plugin.

With the plugin explicitly selected, 「このレポを見て」 defaults to short roles,
dependency reasons and conditional impact inside the native figure. Select a node
or edge, then use 詳しく聞く to pass its file/range/content-revision references
for a bounded explanation. Selecting alone sends nothing; unsupported hosts offer
a copyable reference question. Content revisions are file SHA-256, not Git commits.

For the meaning/diagram workflow, selected source budget, fallback behavior and
synthetic user-facing acceptance example, see [semantic workflow](docs/SEMANTIC_WORKFLOW.md).

## Use a release

Download the archive for your platform and `SHA256SUMS` from
[Releases](https://github.com/iwashita-nozomu/code-structure-inspector/releases).
Verify the archive SHA-256 against `SHA256SUMS`, then extract into a new directory.
Checksums detect changes against that release manifest; this is not an independent
signature or notarization. These binaries are not notarized/signed for OS trust.
Do not disable OS security features to run them; building from reviewed source
is an alternative if your OS blocks the download.

Each archive contains the binary, portable/compatibility plugin manifests,
Skill, README, schema and complete license notice bundle. Mac Apple Silicon
(`aarch64-apple-darwin`), Linux x64 GNU (`x86_64-unknown-linux-gnu`) and Windows x64
MSVC (`x86_64-pc-windows-msvc`) use standard GitHub-hosted runners. Linux requires
an OS compatible with the Ubuntu 24.04 runner's glibc. No updater is installed.

```sh
./code-structure-inspector/bin/code-structure-inspector --version
./code-structure-inspector/bin/code-structure-inspector inspect /absolute/repo
./code-structure-inspector/bin/code-structure-inspector inspect /absolute/repo --format mermaid
./code-structure-inspector/bin/code-structure-inspector related /absolute/repo src/lib.rs
```

On Windows use `bin/code-structure-inspector.exe`. The Windows archive's MCP
manifests reference that executable. The host launches stdio with the plugin root
as its working directory. Stdout contains only newline-delimited MCP messages.

To add the extracted plugin to Codex, append an AVAILABLE entry to the existing
personal marketplace using the host's Plugin Creator flow. Preserve all existing
entries; do not overwrite a shared marketplace. A local installation must verify
startup, tool discovery, and one synthetic read before being considered complete.
Source checkout alone does not include the compiled `bin/` entrypoint; use a
release archive or build/package it first. No account upload or public plugin
catalog submission is required. See [installation](docs/INSTALL.md).

## Build and check

Rust 1.99.1, a C/C++ build toolchain, Python 3.11+, and Node 22+ (helper tests only) are needed for development.
Tree-sitter grammars are compiled from locked registry sources. The analyzer never
compiles the repository it is inspecting.

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --offline
cargo build --locked --release --offline
node --test scripts/test-structure-selection.cjs
python3 scripts/licenses.py --check
python3 scripts/check_package.py
python3 scripts/package.py --target aarch64-apple-darwin --binary target/release/code-structure-inspector
```

First run `cargo fetch --locked` to obtain dependencies. Synthetic fixtures only;
no personal research code or data is included. Release archives carry version and
target names; CI packages are promoted into a release only after review/checks.

## Privacy and security

The analyzer has no network client and no write/execute tools. It reads only the
requested root through capability-relative filesystem operations. Hidden paths,
symlinks, secret/environment directories, vendor/generated paths and unsupported
files are excluded. It returns names and line ranges, not code snippets,
comments or docstrings. Repository names and paths can still be sensitive: MCP
results enter Codex's configured model context. Do not describe this as a promise
that source-derived metadata never reaches cloud services.

Read [SECURITY.md](SECURITY.md) for boundaries and dependency remediation policy.
