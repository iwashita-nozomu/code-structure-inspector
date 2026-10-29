# Release notes

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
