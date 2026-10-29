# Release notes

## 0.1.1

Fix omitted declaration names when Tree-sitter represents them as
`type_identifier` or `namespace_identifier`. Rust structs/enums/traits/type aliases
and C++ classes/structs/enums/namespaces now retain their declaration witnesses.
The same correction applies to the C++ syntax fallback for CUDA.

Four language-specific synthetic regression cases assert every recognized
declaration's name, node kind and exact source line range, including Rust module
and function, Python class and methods, C++ methods/functions, and a CUDA kernel.
The new tests reproduce the omission before the fix; all pass after it.

The analysis remains syntactic and bounded. Dependency resolution rules and
read-only filesystem/execution boundaries are unchanged.
