# DF-presence-or-refusal: a presence `NAME` holding `or` between two words is refused for the `or`, corrected in place

**Date:** 2026-11-16

## status: Accepted

## 1. Context

A presence rule counts one chapter `NAME` ([§FS-rules.3.1](../../functional-spec/FS-rules.md#31-chapter-presence)). An author who wants
each `FS` to carry one of two chapters writes
`Each FS must have exactly one dearer or unsold chapter.`, and grund 0.22.0
refuses it with this reason (agent-grounds/grund#1374):

```text
chapter name must be a non-empty NAME with no surrounding whitespace; NAME forbids whitespace anywhere.
```

The parser read `dearer or unsold` as one `NAME` and refused the spaces in it
([§FS-rule-refusals.sentences.whitespace](../../functional-spec/FS-rule-refusals.md#sentenceswhitespace-presence-name-whitespace-refusal)). The author wrote two one-word names
and a connective, and the reason sends them to retitle a chapter called
"dearer or unsold" that does not exist. `at least one`, a plural count and
`dearer chapter or unsold chapter` get the same reason. The rule language has
no chapter disjunction, and nothing in the reason says so.

The reason is message text on both rule surfaces, `check --rule` and a
configured rule declaration's `invalid-rule` finding, and tools may match it
([§FS-errors.3](../../functional-spec/FS-errors.md#3-message-text), [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered)). So changing it is a decision
rather than a fix, as it was for the chapter-path reasons of
[§DF-rule-refusal-reasons](DF-rule-refusal-reasons.md#df-rule-refusal-reasons-a-chapter-path-is-refused-for-the-component-that-failed-corrected-in-place).

## 2. Decision

A presence `NAME` that, split at whitespace, holds the lower-case word `or`
with a word before it and a word after it is refused for the `or`, and no form
is offered ([§FS-rule-refusals.sentences.presence-or](../../functional-spec/FS-rule-refusals.md#sentencespresence-or-or-between-chapter-names-is-refused-for-the-or)):

```text
"or" between chapter names is not accepted; a presence rule counts one NAME.
```

The new reason replaces the old one in place, in one release, with no window,
under the pre-1.0 licence of [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise), as a named
exception of [§FS-errors.requirements.prefix-windows](../../functional-spec/FS-errors.md#requirementsprefix-windows-a-wording-change-keeps-the-old-message-whole-until-its-window-closes).

- The old reason named a fault the sentence does not have. A window would keep
  sending the author, on purpose, to retitle a chapter that does not exist.
- Every sentence whose reason changes was refused before and stays refused, so
  nothing that worked can have depended on the old reason.
- A consumer of the `invalid-rule` finding keeps its code, its path, line,
  severity and authority, the exit code, and selection by
  `--only invalid-rule`. Only the text after `is not a valid rule: ` moves.
- `check --rule` has no code. A consumer of it keys on the exit code, which
  stays `2`, and on the sentence it passed, and this change moves neither.
- The refusal stands where the whitespace refusal stood, in the predicate stage
  of [§FS-rule-refusals.sentences.order](../../functional-spec/FS-rule-refusals.md#sentencesorder-a-sentence-is-refused-for-the-first-stage-it-fails), so every earlier stage still wins.
- Every `NAME` that does not hold `or` between two words keeps today's bytes.
  `Goal and hypothesis`, an empty `NAME`, `or` alone, an `or` with no word on
  one side, a capitalised `OR` and `order` are still answered by
  [§FS-rule-refusals.sentences.whitespace](../../functional-spec/FS-rule-refusals.md#sentenceswhitespace-presence-name-whitespace-refusal).
- No verdict moves: the accepted and refused sentences, the exit codes and the
  finding code stay as they are. So this is not a verdict correction, and it
  needs none of that route's proof.

No form is offered because the language has none to give. One presence rule
per name requires every chapter rather than any one of them, which judges
other units than the typed sentence named
([§FS-rule-refusals.sentences.rewrite.same-units](../../functional-spec/FS-rule-refusals.md#sentencesrewritesame-units-a-form-judges-the-units-the-typed-sentence-named-or-none-is-offered)).

## 3. Consequences

- A script that matches the exact stderr of `check --rule`, or the exact
  `message` of an `invalid-rule` finding, reads a new reason for a presence
  `NAME` holding `or` between two words. No part of the old line survives:
  its explanation after `; ` was about whitespace too.
- The whitespace refusal keeps every byte for the `NAME`s it still answers, and
  its promise to keep the released reason as a prefix now covers only those.
- A presence rule still cannot accept a choice between chapters. Whether it
  should is a grammar question this record does not decide. If such a choice
  is ever accepted, this refusal goes with it.
- Exit codes, stdout, finding codes, locations and the set of accepted
  sentences do not move. No repository that passed starts failing, and none
  that failed starts passing.

## 4. Alternatives considered

| Approach | Why rejected |
|---|---|
| Keep the released reason as a prefix and append the `or` reason after it, the [§FS-errors.3](../../functional-spec/FS-errors.md#3-message-text) route | The whitespace reason would still come first, which is what agent-grounds/grund#1374 reports. [§DF-rule-refusal-reasons.4](DF-rule-refusal-reasons.md#4-alternatives-considered) rejected the same route for the same reason. |
| Open a window: print the old reason for a release, then drop it | A window lets exact-line consumers move to a stable code. The finding's code already exists and does not move, and `check --rule` has none, so a window gives them nothing new. It would print the false reason, on purpose, for at least one more release. |
| Correct `check --rule` only and keep the finding's bytes | One function renders both surfaces. Keeping the finding would need a second renderer whose only job is to print a false reason, on the surface a repository's `grund check` fails on. |
| Offer one presence rule per name as the accepted form | Together they require both chapters rather than either one, so they judge other units than the typed sentence named ([§FS-rule-refusals.sentences.rewrite.same-units](../../functional-spec/FS-rule-refusals.md#sentencesrewritesame-units-a-form-judges-the-units-the-typed-sentence-named-or-none-is-offered)). |
| Accept the disjunction | That is a change to the grammar, not to a message, and agent-grounds/grund#1374 asks only for the message. |

## release-note: Release note

- [§FS-rule-refusals.sentences.presence-or](../../functional-spec/FS-rule-refusals.md#sentencespresence-or-or-between-chapter-names-is-refused-for-the-or): **a presence rule that joins two
  chapter names with `or` is refused for the `or`.** `grund check --rule` and a
  configured rule declaration's `invalid-rule` finding now refuse
  `Each FS must have exactly one dearer or unsold chapter.` with
  `"or" between chapter names is not accepted; a presence rule counts one NAME.`,
  no longer with
  `chapter name must be a non-empty NAME with no surrounding whitespace; NAME forbids whitespace anywhere.`
  The same holds for any count, and for `dearer chapter or unsold chapter`.
  **Who this breaks:** a script matching, for such a sentence, the exact stderr
  of `check --rule`, the exact `message` of an `invalid-rule` finding, or the
  exact failure message of the Python `check(rule=…)` or Node
  `check(root, { rule })`. Exit codes, stdout, the finding code and location,
  and every sentence's verdict do not move. Closes
  [issue #1374](https://github.com/agent-grounds/grund/issues/1374).
