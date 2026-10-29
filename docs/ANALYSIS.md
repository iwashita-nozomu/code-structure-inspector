# Analysis contract v1

`analysis: syntactic_evidence` distinguishes observations from interpretations.
`files` contains relative paths, parser status and declaration name/range witnesses.
`dependencies` contains source file, line, declared target, kind, nullable resolved
file and resolution. `layout_candidate` means that a known file matches a simple
layout rule, not that the compiler/runtime selects it. `literal_relative_path`
means a quoted C/C++ include names one scanned file relative to its containing
file. Conditional compilation is not evaluated. `cycles` contains strongly
connected file groups in that candidate graph. Mermaid includes every scanned
file and deduplicated resolved file edges; evidence remains in JSON.

Rust resolution handles out-of-line `mod` and simple `crate`/`self`/`super` use
paths under the nearest scanned crate root. For `use`, the longest matching
module prefix is considered first; `prefix.rs` and `prefix/mod.rs` at that depth
must yield exactly one file, otherwise the dependency is ambiguous. A shallower
symbol-suffix fallback does not override that ambiguity. Grouped uses, macros, external crates,
path attributes and inline module namespaces are unresolved. Features and cfg
are not evaluated. Symbol suffix removal is only a file-layout candidate.

Python absolute imports use repository-root layout; relative imports use the
containing package directory. `from a import b` records the dependency on `a`;
it does not infer that `b` is a module. A leading-dot-only import refers to the
package initializer. `sys.path`, src layouts, namespace packages, dynamic imports,
installed environments and star exports are not inferred. C++ resolution does
not inspect system headers, include directories or macro expansions. CUDA has
no dedicated semantic analysis. No cross-file symbol or runtime call graph exists.

Cargo dependencies include target-specific, build, dev and workspace declarations.
They do not represent resolved lockfiles, enabled features or selected targets.
PEP 621 arrays and simple named requirements are recorded; Poetry tables,
requirements options, nested requirement files, URL/path requirements, markers
and extras resolution are not implemented. Manifests are never installed.

Facts are returned even when a parser reports `syntax_errors`; treat those files
as incomplete. Coverage counts unsupported/excluded/skipped entries and unresolved
dependencies. Limits: 500 read files, 10,000 visited entries, 1 MiB per file,
16 MiB total, directory depth 32, 100,000 AST nodes/file, 1,000 symbols/file,
10,000 dependencies and a cooperative five-second scan budget (500 ms/parser).
File IDs serialize native path components with `/` separators. Unix filenames
containing literal backslashes are explicitly skipped (`unsupported_path_encoding`)
so distinct filesystem names cannot collapse to the same ID.
The filesystem snapshot is not atomic. Enumeration and bounded reads are
synchronous; OS I/O can exceed the cooperative budget. Do not scan a filesystem
that is being concurrently mutated. Capability-based opens prevent path escape;
no security claim relies on MCP annotation hints.

This version deliberately avoids running clang-uml. Its Apache-2.0 upstream is
promising for future C++/CUDA semantic adapters, but compiler configuration,
query drivers, renderers, compilation databases and missing aggregate edge line
witnesses require a separate reviewed boundary. cargo-modules and cargo check
can run build scripts/procedural macros and are not part of this read-only path.

An unsupported or missing edge is not evidence of independence. Responsibility
explanations are model interpretations; cite actual file/line witnesses and
inspect additional authorized source where declaration names are insufficient.
