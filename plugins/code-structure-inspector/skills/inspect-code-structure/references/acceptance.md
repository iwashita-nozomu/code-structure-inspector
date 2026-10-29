# User-facing acceptance

## Misleading names, source-backed meaning

User: "このコード、何がどの責務を持つかと、入力仕様を変える影響を図で説明して。"

Use the synthetic public acceptance repository in `tests/semantic-fixtures` when
validating this plugin from source. A normal installed use chooses the user's
explicitly authorized repository instead. The fixture is deliberately named
`train.py`, `cache.py` and `renderer.py`; no model is trained, cache persisted, or
GUI rendered. Do not execute or import the fixture.

Observed machine facts: `train.py:1` imports `cache`, and `train.py:2` imports
`renderer`; both file resolutions are layout candidates. Selected source:
`train.py:4-8` / `summarize_rows`, `cache.py:1-5` / `parse_count`,
`renderer.py:1-5` / `total`.

A useful Codex GPT answer, after reading those selected ranges, is:

> ファイル名と異なり、ここは文字列の件数を検証して合計する処理です。
> `cache.py`は前後の空白を取り除き、十進数字か検証して整数へ変換する境界です
> (`cache.py:1-5`, `parse_count`)。`renderer.py`は渡された値の総和を求めます
> (`renderer.py:1-5`, `total`)。`train.py`は各行をその検証へ渡してから合計を依頼します
> (`train.py:4-8`, `summarize_rows`)。選択範囲にはエラーをcatchして行を飛ばす
> 処理がなく、検証エラーは上位へ伝わる構成と読めます (`cache.py:3-4`, `train.py:4-8`)。
>
> 入力許容範囲を変えるなら、まず`parse_count`の契約が変わります。その結果、
> `summarize_rows`が受け付ける行や失敗条件、最終的な合計も変わり得ます。
> これは選択したsourceが支える候補影響で、全実行経路の保証ではありません。

Then show a native visualization whose nodes say **行の変換をまとめる**,
**十進数字を検証・整数化**, **整数列を合計**, keeping file names secondary.
Selecting the validation node should reveal its relative source witness and
highlight the candidate caller impact, with no copied code or literal data values.
Follow the host visualize skill: an interactive selected-detail graph fits native
visualize, while a small static graph may be routed to Mermaid.

## Diagram contract

- Include the three responsibility nodes and the two observed import relationships.
  Label their file resolution as `layout_candidate`, not compiler-proven binding.
- Explain the first relationship as "各行の正規化・検証を依頼" with `train.py:7`
  evidence and the second as "変換した値の合計を依頼" with `train.py:8` evidence.
- Mark node responsibilities and behavioral edge meanings as **GPT interpretations**
  supported by selected source. Separate machine observations and unknowns.
- Candidate impact starts from the selected validator contract and points to its
  caller. Never reverse this into a claim that unrelated aggregation code must break.
- Keep all references relative. No source snippets, private values, absolute user
  roots, automatic file operations, hidden callbacks or cross-plugin calls.
- If host source access is unavailable, state that meanings are unconfirmed and
  show the import facts only. If native visualize is unavailable, label the
  Mermaid/text fallback. A raw Mermaid/JSON dump without explanation is a failure.

## Acceptance checks and limits

The automated fixtures check that the machine facts, selected declaration ranges,
source selection budget and cited source operations agree. CI also checks that
these workflow/reference files are packaged and explicit invocation is retained.
They do not call an AI API or assert that a model always generates the golden
wording. Evaluate a live Codex response against the meaning, evidence, uncertainty
and diagram contract; fluent prose, a graph alone or path-name guesses do not pass.
