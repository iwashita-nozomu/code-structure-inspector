# Evidence-linked figure plan

Prepare a small declarative plan before rendering the answer. This is a host-side
reasoning/presentation contract, not a new MCP result schema, reader, patch API,
renderer or saved database. Keep it in the current response context and follow
the available native visualize skill. Do not print the plan instead of answering.

## Content and layout responsibilities

Codex GPT chooses the concise explanation, responsibilities, relationships,
uncertainty and source citations. Describe nodes/edges and optional logical panels;
do not guess pixel coordinates, SVG paths or panel sizes. Prefer the host's
supported declarative graph/automatic layout, including Mermaid for static
structure. Do not install Node, dagre, a renderer or another plugin. If the native
host cannot express the view, use its supported fallback and label the limitation.
No layout or Chat/Work display is verified merely because a plan or HTML exists.

## Small plan

Use only the sections that answer the question; logical panels need not become
visible cards. A typical view has `responsibilities`, `dependencies` and `impact`.

- `view_id`: a stable response-local key, such as `code-responsibilities`.
- `panels`: explicit `id`, changeable `title`, and the IDs of relevant claims,
  nodes and edges. An ID is never a title, array position or automatically assigned
  letter. Keep IDs unique; do not reuse a removed ID for a different component.
- `claims`: stable `id`, short `text`, `status` (`observed`, `gpt_interpretation`,
  `unknown`) and `evidence` IDs. Observed/interpretation claims need actual witnesses;
  an unknown can name the evidence gap instead. Never invent a citation to fill it.
- `nodes`: stable `id`, relative `component`, short responsibility `label` and
  `claim_ids`. File names are secondary to the meaning.
- `edges`: stable `id`, `from`, `to`, `claim_ids` and `basis`. A literal include or
  layout-candidate import retains its machine observation ID and exact basis.
  A candidate impact overlay uses `gpt_interpretation`, never a fabricated fact.
- `evidence`: source-selection or observed-import IDs tied to the current root,
  selected source ranges and report. Keep the same citation IDs in the prose and
  node/edge details. A role sentence and its figure label must not cite different
  evidence for the same claim.
- `layout`: a host-supported strategy/direction only. Do not place source snippets,
  literal/private values, absolute roots, HTML/scripts or geometry in this plan.

The synthetic fixture `tests/figure_contract.json` is a design/acceptance example,
not input to the Rust CLI. Its citation registry points to the existing semantic
fixture. The backend continues to expose only observed structural facts.

## Focused changes

When the user asks to revise one part, locate it by explicit ID, even if a title
changed or another panel was inserted. Preserve unrelated panel/claim/evidence IDs
and content. Update only supported claims; retain the same source-selection budget
and do not reread the repository merely to relayout a figure.

A changed source/report invalidates affected witnesses. Reinspect within the
existing scope and refresh dependent claims before reusing them. IDs express
response identity, not a promise that line references remain valid forever.
If the native host cannot patch a panel, rerender the current plan with the same
IDs; do not promise incremental DOM updates or add a persistent patch service.

## Before rendering

Check unique IDs, existing edge endpoints, valid claim references and real source
citations. Reject a dangling link or missing witness; keep it unknown rather than
silently repairing it with a guess. Retain ambiguity/layout-candidate labels.
Then render through the host and perform actual display QA separately. The plan
and fixture tests do not prove model reasoning quality, layout or screen behavior.
