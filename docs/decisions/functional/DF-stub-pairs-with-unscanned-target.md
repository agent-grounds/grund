# DF-stub-pairs-with-unscanned-target: a stub pairs with its target whether or not the scan reaches it

**Status:** Accepted
**Date:** 2026-10-08

## 1. Context

A stub counted as a home of its own unless a scanned declaration of its ID sat at its
target, so a target outside `[scan] include` never paired. Two stubs of one ID pointing
at one such file were a `duplicate declaration` to `check`, an `ambiguous ID` to `show`,
and two tagged lines to `list`, while the same tree with the target scanned passed. A stub
beside a real second declaration was named at its own line, not where the target declares
the ID. The broken-stub check ([§FS-declarations.checks.broken-stub](../../functional-spec/FS-declarations.md#checksbroken-stub-broken-inline-spec-stub)) has always read that
target from disk to accept the stub; only the count of homes did not. Reported as
[issue #532](https://github.com/agent-grounds/grund/issues/532).

## 2. Decision

The count of homes takes the broken-stub check's reading of the target. A stub whose
target declares its ID pairs with it, scanned or not ([§FS-declarations.checks.duplicate.1](../../functional-spec/FS-declarations.md#checksduplicate1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned));
stubs to one target are one home ([§FS-declarations.checks.duplicate.2](../../functional-spec/FS-declarations.md#checksduplicate2-stubs-to-one-target-are-one-home)); and such a home is
named at the target's declaration, never at a stub's line
([§FS-declarations.checks.duplicate.3](../../functional-spec/FS-declarations.md#checksduplicate3-a-home-reached-through-stubs-is-named-at-its-target), [§FS-show.2.2.1](../../functional-spec/FS-show.md#221-ambiguous-id)). Each unscanned tree answers as its
scanned control does.

## 3. Rejected alternative: keep the edge undrawn under a path scope

A narrower fix would pair the stubs for the duplicate count but leave a path-scoped run's
edges as they were, so no run that passes today fails. It would contradict
[§FS-declarations.checks.duplicate.1](../../functional-spec/FS-declarations.md#checksduplicate1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned), under which the ID has one home whatever the scan
reaches, and the scanned control, which draws the edge. It would also make the path decide
resolution, which [§FS-check.1.3.6.1](../../functional-spec/FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution) rules out: `grund check <path>` and `grund check .`
never disagree about a citation they both read.

## 4. Consequences

The verdict moves in three ways, and the message templates do not change.

- **Fail to pass.** Stubs of one ID that point at one target outside `[scan] include` are
  one home: no `duplicate declaration`, no `ambiguous ID`, and `list` no longer tags them.
- **Sites renamed.** A duplicate or an ambiguity involving such a home names the target's
  declaration, as the scanned tree already did, so the located error may now sit in the
  unscanned target file ([§FS-check.2.1](../../functional-spec/FS-check.md#21-report-format)). A path-scoped run keeps a finding by its sites
  ([§FS-check.1.3.6.2](../../functional-spec/FS-check.md#1362-a-finding-that-spans-several-sites-is-in-scope-at-any-of-them)), so `check docs/b` over a stub in `docs/b` that collides with
  `docs/c/o.md` now passes: neither site lies in `docs/b`, which is also the scanned
  control's answer.
- **Pass to fail.** A path-scoped run whose citation of such an ID lies inside the path
  while the stubs lie outside it. The old run read the ID as ambiguous and drew no edge to
  it, and the duplicate that said why sat outside the path, so the report dropped it
  ([§FS-check.1.3.6.1](../../functional-spec/FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution)). The edge now counts, and a chapter rule fires inside the path:

  ```console
  $ grund check docs/fs     # docs/ar/a.md and docs/ar/b.md stub AR-x to ../../notes/x.md
  docs/fs/y.md:3: error: FS must not cite AR (RULE-no-ar) — re-point the citation or downgrade it to a plain Markdown link
  $ echo $?
  1                         # 0 before: `success`
  ```

  The run at the root already failed on the same tree, with the duplicate, so no tree
  that passes an unnarrowed `grund check` fails after. A configured citation direction
  does not move: it drew this edge before, ambiguous ID or not.

## 5. The correction route

[§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) licenses the pass-to-fail move, and each of its five conditions holds.

1. *Prior prohibition.* The old verdict violated [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded), which already applied when that verdict shipped, in two places.
   Its "Reported on only where you pointed" bullet says a path-scoped run reads the whole project, so that "nothing goes unresolved for lying outside the path, so no citation is missed and no edge is uncounted"; the old run passed with an edge inside the path uncounted, for stubs that lay outside it.
   And its closing line says "a skip that no section names is the bug": counting homes without reading an unscanned target was a skip no section named, while [§FS-declarations.checks.broken-stub](../../functional-spec/FS-declarations.md#checksbroken-stub-broken-inline-spec-stub) always read that target.
   [§DF-section-citation-counts-in-rules.2.4](DF-section-citation-counts-in-rules.md#24-why-not-the-correction-route) refused this requirement as a stretch for how rules count a citation the scanner recognised. Here there is no stretch: the path-scope bullet promises the edge in so many words.
2. *Accepted proof.* This record. The [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path does not fit, because there is no old form to carry beside a new one: the old pass is the verdict the requirement forbade, and a release-long warning would keep it.
   The [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) mechanical migration has no command to point at, because no command can re-point a citation or rewrite a rule.
3. *Named release.* The release note below names the verdict move, in both directions.
4. *Actionable findings.* Every finding that replaces the old verdict is the chapter rule's existing error, located at the citation inside the path and naming the rule and the edit to make ([§FS-rules.7.5](../../functional-spec/FS-rules.md#75-prohibition-and-recommendation-reuse)).
5. *No new licence.* This is not a licence for anything else: no rule, finding or prohibition is new, and the edge now counted is the one [§FS-declarations.checks.duplicate.1](../../functional-spec/FS-declarations.md#checksduplicate1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned) says exists.

## release-note: Release note

- [§FS-declarations.checks.duplicate.1](../../functional-spec/FS-declarations.md#checksduplicate1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned), [§FS-declarations.checks.duplicate.2](../../functional-spec/FS-declarations.md#checksduplicate2-stubs-to-one-target-are-one-home), [§FS-declarations.checks.duplicate.3](../../functional-spec/FS-declarations.md#checksduplicate3-a-home-reached-through-stubs-is-named-at-its-target), [§DF-stub-pairs-with-unscanned-target](DF-stub-pairs-with-unscanned-target.md#df-stub-pairs-with-unscanned-target-a-stub-pairs-with-its-target-whether-or-not-the-scan-reaches-it), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded): **a stub pairs with its target whether or not `[scan] include` reaches it.** Stubs of one ID that point at one file outside the scan are one home, so `check` no longer reports a `duplicate declaration`, `show` no longer refuses an `ambiguous ID`, and `list` no longer tags them; a duplicate or ambiguity that involves such a home names the target's `path:line`, never the stub's. **Who this breaks:** the verdict moves both ways for a path-scoped `grund check <path>`. Where a citation of such an ID lies inside the path while its stubs lie outside, a chapter rule now counts that edge, so `grund check docs/fs` can fail where it printed `success`; where a duplicate's sites have moved out of the path, the run can pass. Every finding that replaces the old verdict is the chapter rule's existing error, naming its location and the action to take, and the unnarrowed run over the same tree already failed. Closes [issue #532](https://github.com/agent-grounds/grund/issues/532). (PR #533)
