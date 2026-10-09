# DF-stub-sections-from-unscanned-target: a stub's sections are its target's whether or not the scan reaches it

**Status:** Accepted
**Date:** 2026-10-08

## 1. Context

A citation of a section of a stub's ID answered from the recorded declarations of that ID.
Where the stub's target lay outside `[scan] include`, the only record was the stub's own, and a
stub records no section ([§FS-declarations.checks.duplicate-section.2](../../functional-spec/FS-declarations.md#checksduplicate-section2-scoped-to-that-declarations-body)). So `check` reported
`missing section <ID>.<path>` for a heading the target does declare, while it accepted the stub,
`show` printed that very section ([§FS-show.2.3.7](../../functional-spec/FS-show.md#237-a-stubs-target-is-found-by-its-id)), and the same tree with the target scanned
passed. Reported as [issue #529](https://github.com/agent-grounds/grund/issues/529).

The same reading reached past the finding. A rule counted no `cites` fact for the citation,
because [§FS-rules.5.1](../../functional-spec/FS-rules.md#51-facts-and-identity) contributes none for a section the declaration does not have. A bare index
entry of it was refused as `missing-index-entry`, because [§FS-check.3.17.4](../../functional-spec/FS-check.md#3174-only-a-citation-fmt-would-wrap-reaches-this-rule) admits no section that
no declaration declares, although `grund fmt --write` wraps that entry on both trees. And a value
binding to such a section was never compared: [§FS-values.5.1](../../functional-spec/FS-values.md#51-resolve-before-comparison) compares only once the component
resolves, so a binding that matches and one that does not were each `section not found`.

## 2. Decision

Every reader in `check` asks the target for the stub's sections, scanned or not
([§FS-check.3.2.1](../../functional-spec/FS-check.md#321-a-stubs-sections-are-its-targets-scanned-or-not)): the `missing section` finding, the rule facts, the value comparison, the
index-entry gate and the declaration-local rewrite answer from one section set, the one `show` slices
([§FS-show.2.2.2.2](../../functional-spec/FS-show.md#2222-the-headings-check-counts)). It extends [§DF-stub-pairs-with-unscanned-target](DF-stub-pairs-with-unscanned-target.md#df-stub-pairs-with-unscanned-target-a-stub-pairs-with-its-target-whether-or-not-the-scan-reaches-it) from the count of homes to
what a home holds: the scan scope decides neither. A section the target does not declare, and
any section of a broken stub, is reported as before.

## 3. Rejected alternative: stop judging a section of an unscanned target

Passing every section citation of such a stub would clear the false finding without reading the
target. It would also pass `<§>FS-second.2` where the target declares only `## 1.`, a coordinate
that resolves to nothing, which [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution) says must fail. The issue asked for the
target's sections to be read, not for the check to stop.

## 4. Rejected alternative: clear the false finding and leave the rest as it was

A narrower fix would drop the `missing section` and leave rules, value bindings and the index gate
reading the stub's empty record, so no selected run that passes today fails. Then a plain `grund check` over a
tree with a rule that forbids the citation would exit 0 where the scanned control reports the
rule, because the false finding that said why would be gone and the edge still uncounted; and a
binding of the wrong value would pass where the scanned control reports `value mismatch`. It
would contradict [§FS-rules.5.1](../../functional-spec/FS-rules.md#51-facts-and-identity), under which a citation `check` resolves counts for every rule
family, and [§FS-check.2.1.2](../../functional-spec/FS-check.md#212-selection-filters-the-complete-report), under which a selection filters a report that is already complete.

## 5. Consequences

The verdict moves in three ways, and no message template changes.

- **Fail to pass.** A plain `grund check`, and the editor diagnostics that are its findings,
  no longer report a section the unscanned target declares: neither `missing section` nor, for a
  named coordinate, `section not found`. A tree whose only errors were these exits 0. That
  includes a binding into a value whose root in the unscanned target is invalid: it is quiet at
  its site, as on the scanned tree ([§FS-values.5.1](../../functional-spec/FS-values.md#51-resolve-before-comparison)), and the root's own `invalid value declaration` is
  not reported, because `check` judges no declaration outside the scan. So such a tree now
  exits 0, even with a wrong value bound.
- **One finding for another.** A bare index citation of such a section was `missing-index-entry`
  at the stub beside `missing section` at the citation. It is now the `unlinked-index-entry`
  the scanned tree reports, naming `grund fmt --write`, which already wraps it. A value binding to
  such a section was `section not found` whether its value matched or not; one that matches is now
  clean, and one that does not is the `value mismatch` the scanned tree reports, naming the
  target's `path:line` ([§FS-values.5.2](../../functional-spec/FS-values.md#52-fixed-value-errors)). A malformed binding into such a value, aimed at its
  declared chapter, with two spaces, or split across two lines, read as prose and drew only
  `section not found`. It is now the `invalid value binding` the scanned tree reports
  ([§FS-values.3.1.1](../../functional-spec/FS-values.md#311-invalid-attempts-and-non-attempts)), beside a `section not found` only where the scanned tree reports one too.
- **Pass to fail.** A run that leaves the false finding out of its report while it reads the
  citation. `--only unlinked-index-entry` does that over such an index; `--ignore
  missing-section` or `--only value-mismatch` over a binding of the wrong value; and `--ignore
  missing-section` or `--only invalid-value-binding` over a malformed binding into such a value.
  So does any run that reaches a rule counting the citation: `--ignore missing-section`, `--only`
  with a code a rule produces such as `forbidden-citation`, a trial `--rule "<sentence>"
  --only-rule`, or a path holding the unit an inbound count judges but not the citation ([§FS-check.1.3.6.1](../../functional-spec/FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution)). The citation
  now counts as it does on the scanned tree: a named chapter for that chapter, never for the
  declaration that holds it, and a numbered section for its nearest named chapter, or else for the
  declaration. So a prohibition or an `at most N` or `exactly N` count can newly report, and an
  `at least` count can newly pass, while no chapter subject or count of chapters reaches the
  home's chapters, as before. A named chapter reports as an error. A numbered section reports
  as the warning [§FS-rules.7.8](../../functional-spec/FS-rules.md#78-a-newly-counted-section-citation-warns-until-0180) gives a newly counted section citation until grund 0.18.0, and the
  run still exits 0 until then.

  ```console
  $ grund check --rule "Each FS must not cite any AR." --only-rule   # AR-second's stub targets ../../source.rs, outside the scan
  docs/fs/uses.md:3: error: FS must not cite AR (--rule) — re-point the citation or downgrade it to a plain Markdown link
  ```

  This exited 0 with `success` before, and exits 1 now. A plain `grund check`, with no selection,
  already failed on the same tree with the false finding, so no tree that passes it fails after.

One rewrite moves with them. `grund fmt --cross-refs` rewrote a binding aimed at such a value's
declared chapter into a Markdown link. It now leaves the binding alone, as on the scanned tree,
because the value authority `check` refuses it by is the authority `fmt` protects ([§FS-values.8](../../functional-spec/FS-values.md#8-formatting-stability)).

Five things do not move. A configured citation direction ([§FS-config.3.9](../../functional-spec/FS-config.md#39-citations--citation-direction-rules)) drew this edge before,
section or not. A section the target does not declare, and every section of a broken stub, is
reported as it was. A target that declares the ID more than once, and every target of an ID
with more than one home, lends no section, because `show` refuses that ID as ambiguous rather
than read it ([§FS-show.2.3.7](../../functional-spec/FS-show.md#237-a-stubs-target-is-found-by-its-id)); its citations read as they did. The declaration-local rewrite ([§FS-fmt.2.4.6](../../functional-spec/FS-fmt.md#246-an-absent-target-section-withholds-this-rewrite-and-only-this-one)) cannot meet this case: a
stub has no body, so a numeric local citation after one has no owner scanned or not, and the sites
inside an unscanned target are never read. Last, the target is read only to answer a citation into
it, so what `check` judges of the home itself stays as it was outside the scan
([§FS-check.3.2.1](../../functional-spec/FS-check.md#321-a-stubs-sections-are-its-targets-scanned-or-not)): its declaration, whose errors go unreported as with the invalid root above,
the citations it makes, and its chapters as a rule's subjects or in a rule's count of chapters
([§FS-rules.2.1](../../functional-spec/FS-rules.md#21-a-chapters-name-is-its-whole-path), [§FS-rules.3.1](../../functional-spec/FS-rules.md#31-chapter-presence)). A citation of one of those chapters counts for it, and nothing else
about it is judged.

## 6. The correction route

[§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) licenses the pass-to-fail move, and each of its five conditions holds.

1. *Prior prohibition.* The old verdict violated [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution), which already applied when that verdict shipped: it is present in both released tags, v0.16.0 and v0.16.1.
   That section says "a section coordinate resolves to the declaration's recorded heading or fails". In both tags a stub's headings are its inline home's: "the headings that count are the inline home's" ([§FS-declarations.checks.duplicate-section.2](../../functional-spec/FS-declarations.md#checksduplicate-section2-scoped-to-that-declarations-body)), and for a stub "that set is the inline home's" ([§FS-show.2.2.2.2](../../functional-spec/FS-show.md#2222-the-headings-check-counts)). Neither exempts a home outside the scan.
   The old run resolved the coordinate against the stub's own record, which holds no heading, and so failed a coordinate whose declaration records it. The plain run's `missing section` is the false alarm [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms) forbids, and each pass in section 5 is the same misreading with its alarm out of view: the rule fact, the value comparison, the value authority a malformed binding is refused by and the index entry were withheld because the section read as absent.
   This reaches only the misreading. A section the target does not declare, and any section of a broken stub, keeps every verdict on both builds.
   Nor does [§DF-section-citation-counts-in-rules.2.4](DF-section-citation-counts-in-rules.md#24-why-not-the-correction-route)'s refusal reach this: there a citation resolved to its section and only how rules counted it changed, while here the citation did not resolve to the section its declaration records, which is resolution, not counting.
2. *Accepted proof.* This record. The [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path does not fit, because there is no old form to keep beside a new one, only a wrong answer, and nothing a user could write selects it: the stub and the citation are right as written.
   The [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) mechanical migration does not fit either, because no command can re-point a citation or rewrite a rule. `grund fmt --write`, which the index finding names, repairs that finding, not the old verdict.
3. *Named release.* The release note below names the verdict move in both directions, and the runs it reaches by selection and by path.
4. *Actionable findings.* Every finding that replaces the old verdict is an existing one: `unlinked-index-entry` at the citation, naming `grund fmt --write` ([§FS-check.3.17](../../functional-spec/FS-check.md#317-index-entry-is-not-a-link)), `value-mismatch` at the binding, naming both values and the declaring `path:line` ([§FS-values.5.2](../../functional-spec/FS-values.md#52-fixed-value-errors)), `invalid-value-binding` at the binding, naming the form a binding takes ([§FS-values.3.1.1](../../functional-spec/FS-values.md#311-invalid-attempts-and-non-attempts)), `forbidden-citation` at the citation, naming the rule and the edit to make ([§FS-rules.7.5](../../functional-spec/FS-rules.md#75-prohibition-and-recommendation-reuse)), or a count at the unit it judges, naming the rule, the count found and the count required ([§FS-rules.7.3](../../functional-spec/FS-rules.md#73-outbound-citation-cardinality), [§FS-rules.7.4](../../functional-spec/FS-rules.md#74-inbound-citation-cardinality)).
5. *No new licence.* This is not a licence for anything else: no rule, finding or prohibition is new, the section now found is the one [§FS-declarations.checks.duplicate-section.2](../../functional-spec/FS-declarations.md#checksduplicate-section2-scoped-to-that-declarations-body) says counts, and a section the target does not declare is still reported.

## release-note: Release note

- [§FS-check.3.2.1](../../functional-spec/FS-check.md#321-a-stubs-sections-are-its-targets-scanned-or-not), [§DF-stub-sections-from-unscanned-target](DF-stub-sections-from-unscanned-target.md#df-stub-sections-from-unscanned-target-a-stubs-sections-are-its-targets-whether-or-not-the-scan-reaches-it), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution): **a stub's sections are its target's whether or not `[scan] include` reaches it.** A citation of a section the target declares is no longer `missing section` (or `section not found`) when the target lies outside the scan, as it never was with the target scanned; rules count it, a value binding to it is compared, a malformed binding into a value it declares is the `invalid value binding` the scanned tree reports, which `grund fmt --cross-refs` now leaves unlinked as it does there, and a bare index entry of it is the `unlinked-index-entry` that `grund fmt --write` repairs rather than `missing-index-entry`. A section the target does not declare is reported in the same words as before. **Who this breaks:** the verdict moves both ways for a run that leaves the old false finding out of its report: `--only unlinked-index-entry`, `--ignore missing-section` or `--only value-mismatch` over a binding of the wrong value, `--ignore missing-section` or `--only invalid-value-binding` over a malformed binding, or a rule reached by `--ignore missing-section`, by `--only` with a code a rule produces such as `forbidden-citation`, by a trial `--rule "<sentence>" --only-rule`, or by a path that holds what an inbound count judges but not the citation. Such a run now counts the citation, so a prohibition or an `at most` or `exactly` count can fail where the run exited 0, a numbered section warning until grund 0.18.0, and an `at least` count can pass. Every finding that replaces the old verdict is an existing error naming its location and the action to take, and a plain `grund check`, with no selection, already failed over the same tree. Closes [issue #529](https://github.com/agent-grounds/grund/issues/529).
