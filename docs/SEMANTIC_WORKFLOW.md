# Codex GPT semantic workflow

The Rust tools supply source-indexed observations. Codex GPT, in the current
conversation, reads selected authorized source and explains what components own,
why a dependency matters and which changed contracts could affect a caller.
The package adds no model API, custom LLM runtime, shared orchestrator or new
file-reading tool. Other inspector plugins are not required.

The workflow is bundled in
[the Skill](../plugins/code-structure-inspector/skills/inspect-code-structure/SKILL.md),
with a [source-selection boundary](../plugins/code-structure-inspector/skills/inspect-code-structure/references/source-evidence.md)
and [user-facing acceptance example](../plugins/code-structure-inspector/skills/inspect-code-structure/references/acceptance.md).
A user's normal request should produce a meaning-first explanation with cited
relative source ranges, a useful native visualization, and explicit uncertainty.
The raw machine graph is an input to that answer, not the entire answer.

Before rendering, prepare a small [evidence-linked figure plan](../plugins/code-structure-inspector/skills/inspect-code-structure/references/figure-contract.md).
GPT supplies meaning and citations; supported native layout supplies geometry.
Stable response-local IDs connect prose claims to nodes/edges and preserve
unrelated content during a focused follow-up. This adds no renderer or patch API.
If native partial update is unavailable, rerender with the same IDs rather than
inventing incremental-update support.

Source names/ranges alone cannot establish responsibility. Select up to four
reported files and 240 lines initially (80 per range) through existing permitted
host reads. Respect the same root and exclusions. If source access is missing,
state that meanings are unconfirmed rather than guessing from directory names.
A report and later source view are not an atomic snapshot.

The native visualize skill determines the presentation: selected component detail
and candidate impact can use an interactive view; a small static structure may
route to Mermaid. Missing native visualize gets a labeled fallback, not another
installation or a new rendering service. Keep source snippets, private values,
absolute roots and data out of the visual payload and public examples.

The synthetic fixture deliberately uses misleading names. `train.py` coordinates
count conversion and summation, `cache.py` validates a string, and `renderer.py`
sums values. The selected source, not those names, supports the role descriptions.
CI verifies the fact/range/selection/diagram witness contract and package content;
it does not call GPT or prove the quality of every future semantic answer.

Runtime remains read-only with explicit inputs and results, no persistent state,
network calls, command execution or cross-plugin calls. Install/update is the only
shared-setting operation and remains coordinated separately.
