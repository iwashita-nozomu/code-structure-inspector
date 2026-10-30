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
The intended target is Codex Work; the execution host must actually expose
enabled local stdio MCP tools and
permitted source reads. Native visualization and its optional follow-up bridge
are independent. This package's local marketplace/stdio exposure in ordinary
ChatGPT Chat, including Mac Chat, is unverified (not permanently ruled out); a
Windows conversation viewer does not execute the Mac plugin. Check available
tools instead of inferring execution from OS, a displayed conversation or a Work
label. Missing execution/source evidence gets a labeled capability gap and only
an already-supported fallback, without CLI or another-plugin bypass. A copied
question alone cannot provide missing local tools. A component API or bundled
Skill documenting sendFollowUpMessage does not prove native-fragment exposure;
feature-detect it there and use copy fallback when unavailable. Its return does
not establish confirmation, sending or arrival. Real-host Work acceptance is also
still pending.

A brief **「このレポを見て」** with this plugin explicitly selected starts
[an in-figure overview then explicit selected detail](../plugins/code-structure-inspector/skills/inspect-code-structure/references/selected-detail.md).
The figure contains short responsibility, dependency-reason and conditional-impact
labels with witnesses. Selecting stays local; 詳しく聞く sends exact references
through the host bridge or provides a manual-copy question. User scope overrides
this default, and an ambiguous root must be clarified.
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
The Rust report now includes `files[].content_revision` (SHA-256 of the exact
parsed bytes), an additive report-v1 file field. It is not a Git commit or stored
history. Selected questions carry full content revisions plus relative ranges;
stale/unverifiable bytes require refreshed evidence, not silent substitution.
The scanner reads no additional file to create this digest. It uses RustCrypto
[sha2 0.10.9](https://docs.rs/sha2/0.10.9/sha2/) with preserved dependency notices.
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

The native selection helper is host-fragment JavaScript only, adapted from the
MIT Git inspector pattern at `6ccb4f19a3de701f889d8d8c7824c431d110ec5e`. No Node
package or runtime is shipped/required by the Rust tools; Node's built-in test
runner checks the helper in development/CI. All host UI/click/confirmation and GPT
quality acceptance remain unverified by those synthetic tests.
