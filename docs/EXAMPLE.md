# Synthetic example

This diagram comes from the committed synthetic fixture. Only resolved file layout
candidates and literal includes appear; unresolved declarations stay in JSON.

```mermaid
flowchart LR
  n0["Cargo.toml"]
  n1["cuda/kernel.cu"]
  n2["include/a.hpp"]
  n3["include/b.hpp"]
  n4["pkg/__init__.py"]
  n5["pkg/a.py"]
  n6["pkg/b.py"]
  n7["pyproject.toml"]
  n8["requirements.txt"]
  n9["src/main.rs"]
  n10["src/worker.rs"]
  n1 --> n2
  n2 --> n3
  n3 --> n2
  n5 --> n4
  n5 --> n6
  n6 --> n5
  n9 --> n10
```

| Dependency witness | Interpretation boundary |
| --- | --- |
| `src/main.rs:1` → `src/worker.rs` | Out-of-line Rust module layout candidate. |
| `pkg/a.py:2` → `pkg/b.py` | Relative Python module layout candidate. |
| `include/a.hpp:1` → `include/b.hpp` | Literal quoted include path. |

Strongly connected file groups: `include/a.hpp ↔ include/b.hpp`, `pkg/a.py ↔ pkg/b.py`.
