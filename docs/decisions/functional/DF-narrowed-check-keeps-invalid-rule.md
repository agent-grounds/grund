# DF-narrowed-check-keeps-invalid-rule: a check narrowed to a rule's code keeps the error that says the rule could not run

**Status:** Accepted
**Date:** 2026-10-04

## 1. Context

A rule grund cannot accept is skipped, and the full check says so with an
`invalid-rule` error at the rule's heading
([§FS-rules.7.1](../../functional-spec/FS-rules.md#71-invalid-rule)). Selection was
an exact-code filter over the complete report
([§FS-check.1.4](../../functional-spec/FS-check.md#14-selecting-findings-with---only-and---ignore)),
so narrowing the check to the code that rule would have produced dropped that
error as an unselected code. Over a tree whose only Terms rule is its heading
line alone, with no rationale:

```console
$ grund check .
docs/rules/RULE-chapters.md:1: error: RULE-chapters is not a valid rule: rule rationale is empty
...
$ grund check . --only chapter-cardinality
success
$ echo $?
0
$ grund check . --only chapter-cardinality --format json   # prints nothing, exit 0
```

`grund check --only <code>` is how a person, a CI step or an agent asks whether
one check passes. When every rule behind that code was skipped, the answer was
the same bytes as a real pass, and nothing in the output said the selected check
had never run. The tree held `AR-none`, which a valid rule refuses, so a
violation went unreported behind a `success`. The same hole was open for all
eight codes a rule finding can carry, not only the four rule-only ones: with
`Each AR must cite at least one RULE.` and no rationale, `--only
missing-citation` also printed `success`. Reported as
[issue #421](https://github.com/agent-grounds/grund/issues/421), found while a
reproducer for #416 built on `--only` alone would have called a live bug fixed.

## 2. Decision

### 2.1 Selecting a rule-produced code also selects `invalid-rule`

When `--only` names any of the eight rule-produced codes
[§FS-rules.7.6](../../functional-spec/FS-rules.md#76-selection-json-ordering-and-exits)
lists, `invalid-rule` counts as selected too. The kept row is the one the
unnarrowed run already prints, byte for byte; nothing new is worded. A run whose
`--only` set names no rule-produced code is unchanged, and so is every run with
no `--only` at all.

### 2.2 Every `invalid-rule` row, with no family matching

The carried rows are all of them, not those of the rule that would have
produced the selected code. A rule that fails to parse has no family:
`Every AR ought to have a Terms chapter.` is refused with "rule has no accepted
modality", so there is nothing to match it against. Matching would also mean
tagging every diagnostic with a family it does not carry today, a field kept
only for this filter. An extra row is cheap: every `invalid-rule` is already an
error that fails the unnarrowed run, and fixing it is an edit at the location
the row gives.

### 2.3 `--ignore` and the authority axis keep their precedence

`--ignore invalid-rule` still wins, as it wins over both axes today
([§FS-check.1.4](../../functional-spec/FS-check.md#14-selecting-findings-with---only-and---ignore)):
it is an explicit opt-out, and a second exception to ignore's precedence would
make selection unpredictable. `--only-rule` still applies to the carried rows. A
declared rule's `invalid-rule` carries that rule's authority, so a run scoped to
the `--rule` sentence drops it, and the sentence's own `invalid-rule` — whose
authority is `--rule` ([§FS-rules.7.6](../../functional-spec/FS-rules.md#76-selection-json-ordering-and-exits))
— stays.

### 2.4 One place

Selection stays a presentation query over the complete report: the widening is
in the code axis of the one selector both CLI renderers already call, and the
engine, the report, the Rust API and the LSP — which never had selection — are
untouched. [§FS-check.2.1.3](../../functional-spec/FS-check.md#213-the-success-line)
does not change: `success` still means the selected report is empty, and the
report is no longer empty.

## 3. Rejected alternative: a run-level line in place of `success`

The issue's softer remedy kept selection as it was and printed a line such as
"1 rule skipped as invalid; run without --only to see it" in place of
`success`, as the `[workspace]` cautions do. It is rejected because it keeps the
two parts a machine reads — exit `0` and empty JSON — and those are where the
harm is: a CI step or an agent gating on the exit code would still read a pass.

## 4. Consequences

- **One verdict moves from `0` to `1`.** A run narrowed to a rule-produced code
  over a tree holding an invalid rule now prints that rule's error and exits
  `1`, where it printed `success` and exited `0`. That includes a narrowed run
  whose selected check is unrelated to the broken rule: a CI job gating on
  `--only missing-citation` while an unrelated rule is invalid goes red. That is
  intended — it was reporting a pass for rules it never ran.

  **No tree that passes `grund check` today fails after.** A narrowed run can
  only flip when an `invalid-rule` exists. That finding is always an error, and
  selection only ever removes findings, so the unnarrowed run over the same tree
  already exits `1`.

  **Which clause licenses the move.** [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids)
  does, and each of its conditions is met:

  1. *Prior prohibition.* The old verdict violated [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips),
     which already applied when that verdict shipped: an incomplete run "never passes as if it
     were" complete, and "an empty tree … must never look like a clean one". A
     narrowed run whose every selected rule was skipped is exactly that run. It
     also broke [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded):
     "a skip that no section names is the bug". Nothing in the narrowed output
     named the skip, and nothing told the reader to drop `--only`.
     [§DF-unverifiable-rule-scope.4](DF-unverifiable-rule-scope.md#4-consequences) used the same requirement to license moving a
     silently skipped rule-driven comparison from `0` to `1`.
  2. *Accepted proof.* This record. The [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path does not fit,
     because there is no old form carried beside a
     new one to warn about first: the dropped row is the warning, and a release
     that printed it to stderr while keeping exit `0` would keep the very verdict
     the prohibition forbids for a further release. The
     [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) mechanical migration has no command to point
     at: the remedy is to fix a rule's prose, which no tool can write for its
     author.
  3. *Named release.* The change brings its own changelog entry, which names the
     move from exit `0` to `1` and the `--ignore invalid-rule` opt-out.
  4. *Actionable findings.* The finding that replaces the old verdict is the
     existing `invalid-rule` row, located at the rule's heading and naming why the
     rule was refused ([§FS-rules.7.1](../../functional-spec/FS-rules.md#71-invalid-rule)),
     which is where the edit goes.
  5. *No new licence.* The route is not a licence for anything else. Nothing is tightened, removed or newly prohibited: the
     error already fails every unnarrowed run, and this change only stops a
     narrowed one from hiding it.

  **The weak point.** For `chapter-cardinality`, the skipped check counts
  chapters, not citations, so [§REQ-no-missed-citation](../../requirements/REQ-no-missed-citation.md#req-no-missed-citation-every-citation-the-run-reads-is-checked) is read here for its
  general clauses — an incomplete run never passes as complete, and a skip is
  named — rather than for citations in particular. And
  [§FS-rules.7.6](../../functional-spec/FS-rules.md#76-selection-json-ordering-and-exits)
  did declare `invalid-rule` selectable "like every other public code", so an
  objector can say the drop was specified. It was specified as a consequence of
  the exact-code filter, not as a decision that a skipped rule may read as a
  pass; no section named that skip, which is the bug [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded)
  describes. Were the route refused, the fallback is the
  [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path: for one release the carried
  row goes to stderr as a warning, `success` is withheld, the exit stays `0`,
  and a note names the release in which it becomes the error above.
- A run that would have needed `--only invalid-rule` added by hand no longer
  does; adding it is harmless.
- [§REQ-deterministic-output](../../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)
  is unaffected: the carried rows sit in the report's ordinary sorted order.

## release-note: Release note

- [§FS-check.1.4](../../functional-spec/FS-check.md#14-selecting-findings-with---only-and---ignore), [§FS-check.2.1.2](../../functional-spec/FS-check.md#212-selection-filters-the-complete-report), [§FS-rules.7.6](../../functional-spec/FS-rules.md#76-selection-json-ordering-and-exits), [§DF-narrowed-check-keeps-invalid-rule](DF-narrowed-check-keeps-invalid-rule.md#df-narrowed-check-keeps-invalid-rule-a-check-narrowed-to-a-rules-code-keeps-the-error-that-says-the-rule-could-not-run), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips): **a check narrowed to a code rules produce keeps the `invalid-rule` errors.** Selecting any of the eight rule-produced codes — `chapter-cardinality`, `citation-cardinality`, `uncited-unit`, `unreached-declaration`, `missing-citation`, `forbidden-citation`, `discouraged-citation`, `suggested-citation` — with `--only` now also selects `invalid-rule`, every row of it, so a rule that could not run is never read as a pass. **Who this breaks:** the verdict moves from exit `0` to exit `1` for a run narrowed to a rule-produced code over a tree holding an invalid rule: `grund check --only chapter-cardinality` over a rule written as its heading line alone printed `success` and now prints `docs/rules/RULE-chapters.md:1: error: RULE-chapters is not a valid rule: rule rationale is empty`, and its `--format json` form prints that row where it printed nothing. Every finding that replaces the old verdict is the row the unnarrowed `grund check` already prints, naming the rule's location and why it was refused, which is where the fix goes, so no tree that passes the unnarrowed check fails after. **The opt-out:** `--ignore invalid-rule` still wins and restores the old view, and `--only-rule` still drops a declared rule's row beside a `--rule` trial sentence. A selection that names no rule-produced code is unchanged. Closes [issue #421](https://github.com/agent-grounds/grund/issues/421). (PR #438)
