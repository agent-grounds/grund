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
stubs to one target are one home ([§FS-declarations.checks.duplicate.2](../../functional-spec/FS-declarations.md#checksduplicate2-stubs-to-one-target-stand-for-its-declarations-once)); and such a home is
named at the target's declaration, never at a stub's line
([§FS-declarations.checks.duplicate.3](../../functional-spec/FS-declarations.md#checksduplicate3-a-declaration-reached-through-stubs-is-named-at-its-target), [§FS-show.2.2.1](../../functional-spec/FS-show.md#221-ambiguous-id)). Each unscanned tree answers as its
scanned control does.

## 3. Rejected alternative: keep the edge undrawn where the duplicate is not reported

A narrower fix would pair the stubs for the duplicate count but leave the edges of a run
that does not report the duplicate, by a path or by a selection, as they were, so no run
that passes today fails. It would contradict
[§FS-declarations.checks.duplicate.1](../../functional-spec/FS-declarations.md#checksduplicate1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned), under which the ID has one home whatever the scan
reaches, and the scanned control, which draws the edge. It would also make the path or the
selection decide resolution, which [§FS-check.1.3.6.1](../../functional-spec/FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution) and [§FS-check.2.1.2](../../functional-spec/FS-check.md#212-selection-filters-the-complete-report) rule out:
`grund check <path>` and `grund check .` never disagree about a citation they both read,
and `--only` and `--ignore` filter a report already complete.

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
- **Pass to fail.** A run that leaves the false duplicate out of its report while it
  reads a citation of such an ID. A path does that when the citation lies inside it and the
  stubs outside ([§FS-check.1.3.6.1](../../functional-spec/FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution)); so does a selection over the whole tree
  ([§FS-check.2.1.2](../../functional-spec/FS-check.md#212-selection-filters-the-complete-report)): `--only` with a code a rule produces, such as `forbidden-citation`
  or `citation-cardinality`; `--ignore duplicate`; or a trial
  `--rule "<sentence>" --only-rule`. The old run read the ID as ambiguous and counted no
  edge to it ([§FS-rules.5.1](../../functional-spec/FS-rules.md#51-facts-and-identity)). The edge now counts, so a rule sees it: a prohibition or an
  `at most N` or `exactly N` count can newly fail, and an `at least` count can newly pass.

  ```console
  $ grund check docs/fs     # docs/ar/a.md and docs/ar/b.md stub AR-x to ../../notes/x.md
  docs/fs/y.md:3: error: FS must not cite AR (RULE-no-ar) — re-point the citation or downgrade it to a plain Markdown link
  $ grund check --only forbidden-citation
  docs/fs/y.md:3: error: FS must not cite AR (RULE-no-ar) — re-point the citation or downgrade it to a plain Markdown link
  $ grund check docs/fs     # the rule reads "Each FS must cite at most 1 AR." and y.md also cites AR-z
  docs/fs/y.md:1: error: FS-y cites AR 2 times; RULE-no-ar requires at most 1
  ```

  Each of these exited 0 with `success` before, and exits 1 now. A plain `grund check`,
  with no path and no selection, already failed on the same tree with the duplicate, so no
  tree that passes it fails after. That holds of the trees this record moves: a stub whose
  target outside the scan declares its ID twice stayed one home here, and a plain
  `grund check` does newly fail on it once [§DF-stub-target-declared-twice](DF-stub-target-declared-twice.md#df-stub-target-declared-twice-a-stubs-target-that-declares-its-id-twice-is-two-homes-scanned-or-not) makes it two. A
  configured citation direction does not move: it drew this edge before, ambiguous ID or not.

## 5. The correction route

[§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) licenses the pass-to-fail move, and each of its five conditions holds.

1. *Prior prohibition.* The old verdict violated [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution), which already applied when that verdict shipped: it arrived with [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms) in e98a2cc9e9, and both are present in both released tags.
   That section binds resolution to [§FS-declarations.checks.duplicate](../../functional-spec/FS-declarations.md#checksduplicate-duplicate-declaration) by name, "duplicate declarations are reported rather than ranked", and a duplicate there is "any two declarations that are not stubs", in both tags too.
   The old run read an ID with one declaration and two healthy stubs as ambiguous, resolving its citations to a conflict the specification says does not exist, as [§DF-escape-position-is-not-a-citation.4](DF-escape-position-is-not-a-citation.md#4-what-this-costs-and-why-it-may-be-taken) corrected a run that counted as a citation text the specification says is not one.
   Every verdict that moves is that one reading: the plain run's `duplicate declaration` and `show`'s `ambiguous ID` are false alarms [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms) forbids, and each pass in section 4 is the same reading with its alarm out of view, the edge skipped because [§FS-rules.5.1](../../functional-spec/FS-rules.md#51-facts-and-identity) skips an ambiguous declaration and this one was misread as one.
   This reaches only the misreading. Two real declarations of one ID, cited from `docs/fs` under the same rule, leave `check docs/fs`, `--only forbidden-citation` and `--ignore duplicate` passing on both builds, because there the ambiguity is the right resolution and a run that does not show the duplicate is the reporting boundary [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded) allows.
   Nor does [§DF-section-citation-counts-in-rules.2.4](DF-section-citation-counts-in-rules.md#24-why-not-the-correction-route)'s refusal reach this: there a citation resolved to its declaration and only how rules counted it changed, while here the citation did not resolve to the one declaration its ID names, which is resolution, not counting.
2. *Accepted proof.* This record. The [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path does not fit, because there is no old form to keep beside a new one, only a wrong answer, and nothing a user could write selects it: the stubs are right as written.
   The ramp [§DF-section-citation-counts-in-rules.2.3](DF-section-citation-counts-in-rules.md#23-the-new-failures-take-the-deprecation-path) took through [§FS-rules.7.8](../../functional-spec/FS-rules.md#78-a-newly-counted-section-citation-warns-until-0180) was owed there because no hard requirement forbade a change in counting; a wrong resolution is forbidden, so the ramp is not the route here, as it was not for the wrong answer [§DF-path-scope-resolves-project-wide.2.6](DF-path-scope-resolves-project-wide.md#26-req-backwards-compatibility5-is-the-route-and-all-five-conditions-hold) corrected.
   The [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) mechanical migration has no command to point at, because no command can re-point a citation or rewrite a rule.
3. *Named release.* The release note below names the verdict move in both directions, and the runs it reaches by path and by selection.
4. *Actionable findings.* Every finding that replaces the old verdict is a rule's existing finding: `forbidden-citation` at the citation, naming the rule and the edit to make ([§FS-rules.7.5](../../functional-spec/FS-rules.md#75-prohibition-and-recommendation-reuse)), or a count at the declaration or chapter it judges, naming the rule, the count found and the count required ([§FS-rules.7.3](../../functional-spec/FS-rules.md#73-outbound-citation-cardinality), [§FS-rules.7.4](../../functional-spec/FS-rules.md#74-inbound-citation-cardinality)).
5. *No new licence.* This is not a licence for anything else: no rule, finding or prohibition is new, the edge now counted is the one [§FS-declarations.checks.duplicate.1](../../functional-spec/FS-declarations.md#checksduplicate1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned) says exists, and an ID with two real declarations still reads as ambiguous and counts no edge.

## release-note: Release note

- [§FS-declarations.checks.duplicate.1](../../functional-spec/FS-declarations.md#checksduplicate1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned), [§FS-declarations.checks.duplicate.2](../../functional-spec/FS-declarations.md#checksduplicate2-stubs-to-one-target-stand-for-its-declarations-once), [§FS-declarations.checks.duplicate.3](../../functional-spec/FS-declarations.md#checksduplicate3-a-declaration-reached-through-stubs-is-named-at-its-target), [§DF-stub-pairs-with-unscanned-target](DF-stub-pairs-with-unscanned-target.md#df-stub-pairs-with-unscanned-target-a-stub-pairs-with-its-target-whether-or-not-the-scan-reaches-it), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution): **a stub pairs with its target whether or not `[scan] include` reaches it.** Stubs of one ID that point at one file outside the scan are one home, so `check` no longer reports a `duplicate declaration`, `show` no longer refuses an `ambiguous ID`, and `list` no longer tags them; a duplicate or ambiguity that involves such a home names the target's `path:line`, never the stub's. **Who this breaks:** the verdict moves both ways for a run that leaves the old false duplicate out of its report, by a path such as `grund check docs/fs` or by a selection: `--only` with a code a rule produces, such as `forbidden-citation`, `--ignore duplicate`, or a trial `--rule "<sentence>" --only-rule`. Such a run now counts the edge to a citation of such an ID, so a prohibition or an `at most` or `exactly` count can fail where the run exited 0, and an `at least` count can pass; where a duplicate's sites have moved out of the path, the run can pass. Every finding that replaces the old verdict is a rule's existing error, naming its location and the action to take, and a plain `grund check`, with no path and no selection, already failed over the same tree. Closes [issue #532](https://github.com/agent-grounds/grund/issues/532). (PR #533)
