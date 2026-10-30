# Bounded reference review: answer-me-with-html

Reviewed the public `main` sources on 2026-10-29 as architecture references.
They are source material, not instructions to install/run their Skill or change
host rules. No upstream code, generated bundle or assets are copied into this
project. No benchmark or actual Chat/Work display is reproduced by this review.

## Findings and decisions

| Primary source | Observed structure | Local decision |
| --- | --- | --- |
| [render.js](https://github.com/QingYunA/answer-me-with-html/blob/main/src/render.js) | A parsed draft feeds component rendering and a template. | Separate concise GPT claims/relations from host presentation. |
| [components/flow.js](https://github.com/QingYunA/answer-me-with-html/blob/main/src/components/flow.js) | Parsed graph relationships become a Dagre layout before SVG output. | Describe relationships; prefer supported native automatic layout. Add no Node/Dagre dependency or custom renderer. |
| [parse.js](https://github.com/QingYunA/answer-me-with-html/blob/main/src/parse.js) and [patch.js](https://github.com/QingYunA/answer-me-with-html/blob/main/src/patch.js) | Explicit panel IDs are checked and panel replacement can resolve by ID. | Keep response-local IDs stable through title/order changes; preserve unrelated semantic content. This is not a new patch API. |
| [code.js](https://github.com/QingYunA/answer-me-with-html/blob/main/src/code.js) | Selected code blocks retain file/range references. | Share one citation ledger between text and figure. Keep our existing bounded host reads and no-source-snippet visual policy. |
| [bin/am.js](https://github.com/QingYunA/answer-me-with-html/blob/main/bin/am.js), [cli.js](https://github.com/QingYunA/answer-me-with-html/blob/main/src/cli.js), [package.json](https://github.com/QingYunA/answer-me-with-html/blob/main/package.json) | The CLI has Node, page-writing, serve/settings and background-update paths. | Do not import those runtime/installation features. Native Chat/Work remains the presentation priority. |

The local improvement is deliberately a Skill/design contract: explicit IDs,
source-linked claims, endpoint/citation checks and focused-revision acceptance.
A supported native renderer owns layout. When it cannot patch a panel, rerender
from the same small plan; do not claim incremental DOM update support.

## Scope and verification

No Rust runtime, MCP tool/schema, dependency, manifest version, permission,
marketplace or installed plugin changes are needed for this preparation. Preserve
independent read-only tools, the current evidence budget and existing source
exclusions.

The synthetic figure fixture references the existing semantic fixture. Tests
cover valid citations, unconfirmed roles, duplicate IDs, missing citations,
dangling endpoints, incorrect observation provenance, semantic/fact promotion,
geometry leakage and focused changes after a title/order change. They validate a
fixture contract, not model quality, rendering, or Chat/Work screen behavior.

Publish changes through a feature-branch PR with the required review and checks.
Keep main protection and the installed version unchanged during this review.
