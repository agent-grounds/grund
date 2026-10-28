# DF-whose-clause-window-closes-in-0-22: The whose-clause wording window closes in 0.22.0

**Date:** 2026-10-28

## status: Accepted

## 1. Context

[§DF-field-value-subjects.2.4](DF-field-value-subjects.md#24-refused-for-the-part-that-failed-through-a-window) let a rule subject carry a whose clause and answered a refused
clause for the part that failed. Before that, every such subject had been refused as an unknown
kind, so `0.21.0` printed the legacy `unknown kind "<whole text>"` reason as a prefix, then
` — `, the new reason and `; this wording changes in grund 0.22.0`, and kept the `known kinds:`
line on `list --selector` ([§FS-errors.3.10](../../functional-spec/FS-errors.md#310-the-whose-clause-wording-migration)). That is the warning release of
[§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path), under D10 of [§DF-one-answer-rulings-grammar](DF-one-answer-rulings-grammar.md#df-one-answer-rulings-grammar-one-grammar-keeps-the-documented-refusal-bytes-and-order-and-one-kind-universe-keeps-each-commands-vocabulary). The suffix named
`0.22.0` as the release the bytes change in, and the release guard refuses a version that
reaches a promise it does not keep ([§FS-distribution.4.2](../../functional-spec/FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)).

## 2. Decision

In `0.22.0` (agent-grounds/grund#1063):

- the prefix, the ` — ` and the suffix go, so every refusal of a sentence whose subject carries
  a whose clause, from the subject on, prints the reason of [§FS-rule-refusals.sentences.whose](../../functional-spec/FS-rule-refusals.md#sentenceswhose-a-whose-clause-is-refused-for-the-part-that-failed),
  or of the later production that refused it, alone, on `check --rule`, on a configured rule's
  `invalid-rule` finding and on every `list --selector` mode;
- the `known kinds:` line goes after a refused clause, because the kind was recognised, and
  stays after a kind that is not configured;
- the three spellings that never took the window keep their bytes: an exact ID followed by a
  clause, an object kind followed by one, and `KIND.NAME` followed by one;
- the exit code `2`, the `invalid-rule` code and its severity, and what is refused do not move.

## 3. Rejected alternatives

- **Keep the window for another release.** `0.21.0` printed `0.22.0` as the release it closes
  in, and [§FS-distribution.4.2](../../functional-spec/FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name) refuses a release that reaches a promise it does not keep.
- **Keep `known kinds:` after a refused clause.** It lists the kinds for an author who named
  one that is not configured; after a clause the kind was recognised, so the list answers a
  question nobody asked.

## release-note: Release note

- [§DF-whose-clause-window-closes-in-0-22](DF-whose-clause-window-closes-in-0-22.md#df-whose-clause-window-closes-in-0-22-the-whose-clause-wording-window-closes-in-0220), [§FS-errors.3.10](../../functional-spec/FS-errors.md#310-the-whose-clause-wording-migration), [§FS-rule-refusals.sentences.whose](../../functional-spec/FS-rule-refusals.md#sentenceswhose-a-whose-clause-is-refused-for-the-part-that-failed):
  **a refused whose clause is answered with its reason alone.** What grund 0.21.0 announced
  lands. `check --rule 'Each SWR whose verfication is Test must be cited by at least one TC.'`
  printed `error: unknown kind "SWR whose verfication is Test" — "verfication" is not a field of
  kind SWR; its scalar fields: verification; this wording changes in grund 0.22.0` and now
  prints `error: "verfication" is not a field of kind SWR; its scalar fields: verification`, on
  a configured rule and `list --selector` alike, and a sentence whose clause is accepted but
  whose quantifier, object or predicate is refused now prints that reason alone. On
  `list --selector` the `known kinds:` line no longer follows a refused clause; it still follows
  a kind that is not configured. **Who this affects:** a script that matches the legacy
  `unknown kind "KIND whose …"` prefix or the `known kinds:` line after a refused clause, which
  should match the reason after the ` — ` it printed in 0.21.0. Closes
  [issue #1063](https://github.com/agent-grounds/grund/issues/1063).
