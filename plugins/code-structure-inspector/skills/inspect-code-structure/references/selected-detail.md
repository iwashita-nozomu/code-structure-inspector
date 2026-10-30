# In-figure overview → explicit selected detail

With this plugin explicitly selected, **「このレポを見て」** means the two stages
below. Use the current authorized project when unambiguous; otherwise clarify
the root before reading. Do not require a longer visualization prompt. Respect
「一覧だけ」/「図だけ」/「詳細不要」 and other narrower user instructions. Explicit
invocation and a button do not grant new tool or source-read permissions.

## Capability prerequisites are independent

The intended target is Codex Work with enabled local stdio MCP and authorized
source reads, with real-host acceptance still pending. Visualization and the follow-up bridge are separate capabilities.
This package's local marketplace/stdio exposure in normal ChatGPT Chat (including
Mac Chat) has not been verified; that does not rule out future host support. Viewing the same
conversation on Windows does not run or transfer the Mac MCP process; neither a
device name nor Work mode alone establishes local tools, source access or a host
bridge. Check the tools actually exposed to this turn. If absent, report the gap
and use only already available evidence for static/text fallback. Available MCP
without source reads can still supply permitted structural facts, with meanings
unconfirmed. Never fabricate
revision references, imply a copyable question restores execution, or bypass the
missing/denied tool through CLI or another plugin.

## First: concise meaning and evidence inside the figure

Use the normal bounded inspect + selected source workflow (4 files / 240 lines,
80 per range). Show responsibilities on nodes, dependency reasons on edges and a
short conditional candidate-impact label beside the relevant relationship. Include
relative file/range witnesses and observed / GPT interpretation / unknown labels
inside the figure, not only in a prose paragraph or a filename inventory. File
names remain secondary. Missing source means an unknown responsibility/impact.
Do not automatically open a representative component's long explanation.

Read [structure-selection.js](structure-selection.js) and inline it into the
host-authored native fragment; it binds controls, not layout or an MCP client.
Give the available native renderer the small declarative graph. No plugin server,
custom renderer, Node runtime, AI API, shared storage or other plugin is needed.
Node's built-in test runner is used only during development/CI.

Register `CodeStructureSelection.create({view_id, files, targets})` from the current
report and figure ledger. `view_id` is a response-local opaque ASCII ID bound to
the authorized root in the conversation; it is not a pathname or a Git revision.

- `files`: up to 500 reported `{file_index, path, content_revision, lines}` entries;
  use their original report indices. The initial displayed subset stays small.
- `targets`: at most 200 displayed nodes/edges. Each has the existing figure `id`,
  `kind: node|edge`, `basis`, `claim_ids`, `evidence_ids`, and `anchors` containing
  reported `file_index`, `start_line`, `end_line`. The helper resolves the path and
  full content revision from the registry; it ignores input labels/comments.
- Keep anchor ranges within observed declarations or import witnesses. A selected
  target has at most 2 files / 4 ranges / 160 total lines / 80 per range. An edge
  can retain its import line and bounded caller/callee source witnesses.
- `content_revision` is `sha256:` plus 64 hex digits for the **exact file bytes
  parsed by Rust**. It is not a Git commit, historical source store or atomic
  repository snapshot. Display its shortened form with an explicit content label;
  keep the full value in the selected details and question. Never attach a Git
  permalink to a content digest or pretend a dirty worktree equals HEAD.

Each displayed selectable mark is a real button with `data-structure-id` matching
a registered target. `CodeStructureSelection.bind(root, view)` expects these
buttons plus `[data-structure-selection]`, a clearly labeled **詳しく聞く** button
`[data-structure-ask]`, a readonly textarea `[data-structure-fallback]`, and an
`aria-live="polite"` element `[data-structure-status]` in that root. Keep the node/
edge meaning visible when unselected and selected. Use keyboard-accessible native
controls, textContent for source-derived text, and safely escaped inline JSON
(`<`, `>`, `&`, U+2028/U+2029); never interpolate external strings as HTML or code.

## Second: ask, verify the reference, then read just the selected evidence

Selecting only highlights existing meaning/evidence locally. It does not fetch,
run a tool or start a turn. Best-effort `setWidgetState` saves only the small
selection; it cannot be the sole delivery path and never proves delivery.
Unknown/stale/tampered restored references are rejected against the registered
view, paths, hashes, ranges and IDs. Do not trust persisted fields as authority.

Only the explicit **詳しく聞く** action calls feature-detected
`await window.openai.sendFollowUpMessage({prompt,title})`, directly under user
activation. The helper serializes view ID, node/edge ID, claim/evidence IDs and
every selected file/range/full content revision into the question. It sends no
source snippets, comments, inferred instruction, absolute root or private value.
A component API or bundled Skill documenting this method does not establish its
exposure in a native Visualize fragment. Detect it in the actual fragment and
keep the copy fallback. A resolved Promise means only that the invocation returned;
it does not prove a confirmation exists, sending/approval/arrival, evidence read
or answer rendering.
Never automate a host confirmation or an app UI.

When the question arrives, match all references to the conversation's authorized
view and evidence ledger before any I/O. Reinspect using the same authorized root
only if the existing tool is permitted. Match each index/path/content_revision;
do not swap in another file or a newer revision. Use the existing authorized host
reader only when it can bind the selected ranges to those exact file bytes
(revision-bound read or a verified full-file digest with the same read). Hashing
may remain local; expose only the selected lines to the model. Scanner and host
reads are not atomic. Rechecking only line counts, names or timestamps is not a
content check. Never read `.git`, history, extra files or excluded paths to repair
a selection. The Rust scanner retains no old source.

If the file changed or exact-byte verification is unavailable, explain that gap
and ask to refresh the selection rather than claiming to have read the old
revision. Reuse already verified witnesses for a narrower answer when possible.
Do not add a reader tool or bypass denied/absent tools via CLI or another plugin.

After the explicit question, add only the registered ranges, within the selected
2-file/160-line budget. Explain responsibility, why this dependency matters and
conditional candidate impact from those source operations. Update the same figure
IDs and evidence ledger, preserving unrelated content. A literal include/import
is a syntax fact; neither it nor graph reachability is proof of a runtime call,
executed path, cause or guaranteed breakage. Keep GPT meaning separate from facts.

## Honest fallback and acceptance

If the host API is absent or rejects, show the helper's exact reference question
in the readonly field for the user to copy. Do not display raw host error strings.
If native interactive visualization is unavailable, label the static/text fallback
and provide registered IDs plus their reference questions. Mermaid IDs alone are
not clickable controls or automatic follow-up delivery. Never silently replace
the selected target. Missing source, unresolved imports, coverage gaps and hash
changes remain visible. Private repository data stays out of public fixtures/logs.

Synthetic acceptance: ask 「このレポを見て」 on `tests/semantic-fixtures`.
The first figure shows **検証・整数化**, **値を合計**, **行の変換をまとめる**,
dependency reasons and the conditional validation-contract impact with source
references. Select `validator` then press 詳しく聞く: the question must retain
`cache.py:1-5`, its actual content revision and `role-validator` / `S1`. Selecting
`dep-validator` carries the import witness and selected source, with
`layout_candidate` retained. No question is sent before the button. Revision
drift, injected labels, host rejection and selection changes during a pending
confirmation must not change the target or be reported as success.

Tests validate references, evidence bounds and a mocked host bridge. They do not
call GPT or establish real Chat/Work rendering, clicks, confirmation or message
arrival. Real host acceptance remains a separate check; do not retry denied UI
automation. Check readable in-figure meanings/references at 320/768/1024 px and
light/dark only through an authorized QA route.

Reference pattern: [Git inspector selection helper](https://github.com/iwashita-nozomu/github-work-inspector/blob/6ccb4f19a3de701f889d8d8c7824c431d110ec5e/.agents/skills/github-work-inspector/references/review-selection.js)
(MIT, same project author). The bundled visualize 1.0.66 composition text documents
`sendFollowUpMessage({prompt,title})`, but native-fragment exposure is unverified;
its widget state is best-effort.
