# DF-stub-target-section-declared-twice: a section a stub's target declares twice is reported where it is written, scanned or not

**Date:** 2026-10-12

## status: Accepted

## 1. Context

A stub's sections are its target's, whether or not `[scan] include` reaches the target
([§DF-stub-sections-from-unscanned-target](DF-stub-sections-from-unscanned-target.md#df-stub-sections-from-unscanned-target-a-stubs-sections-are-its-targets-whether-or-not-the-scan-reaches-it), [§FS-check.3.2.1](../../functional-spec/FS-check.md#321-a-stubs-sections-are-its-targets-scanned-or-not)). Since grund 0.17.0 a citation into
an outside target resolves through the target's section set. When that set claims one path
twice, `show` and `refs` refuse the coordinate as ambiguous, but `check` passes the citation by
taking the first heading. Take this tree, with `src/` outside the include:

```text
docs/fs/x.md  # FS-x: [x](../../src/x.rs)
docs/use.md   Uses §FS-x.1.
src/x.rs      /// FS-x: X            line 1
              /// ## 1. One          line 5
              /// ## 1. One again    line 9
```

```console
$ grund check
success
$ grund FS-x.1
ambiguous section: FS-x.1 (declared at src/x.rs:5, src/x.rs:9)
```

With `src/` scanned, `check` reports `src/x.rs:5: error: duplicate section FS-x.1 (also
declared at src/x.rs:9)` and exits 1, whether or not anything cites `FS-x.1`
([§FS-declarations.checks.duplicate-section](../../functional-spec/FS-declarations.md#checksduplicate-section-duplicate-section-path)). The duplicate-section rule walks the catalog's
declarations, and a target outside the include is not among them, so the outside tree never
meets the rule. The difference is row U01 of the stub differences that
[§REQ-one-answer.differences.stubs](../../requirements/REQ-one-answer.md#differencesstubs-stubs) listed, and D8 of [§DF-one-answer-rulings-stubs](DF-one-answer-rulings-stubs.md#df-one-answer-rulings-stubs-a-stub-is-found-by-its-home-and-shape-read-as-the-scan-reads-and-shown-where-its-target-is) names it as
the one stub ruling that may be corrected in one release.

## 2. Decision

The duplicate-section rule judges the sections of every declaration a stub pairs with, at the
target's headings, exactly as it judges them with the target scanned
([§FS-declarations.stubs.target.5](../../functional-spec/FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged)). The outside tree above reports
`src/x.rs:5: error: duplicate section FS-x.1 (also declared at src/x.rs:9)` and exits 1, the
scanned tree's line and message, whether or not anything cites the section. A scanned target is
one declaration however many stubs point at it, so its duplicate is reported once. The finding,
its code, its text and its `sites` are the rule's own; nothing about it is new.

D8 says a target read for a stub "reports no finding of its own". That sentence covers the
target file's own hygiene: its other declarations, its notes and formatting, its leads, and
whether its citations resolve ([§DF-stub-sections-from-unscanned-target.5](DF-stub-sections-from-unscanned-target.md#5-consequences)). It does not cover
the section set every command resolves a citation through, which is a fact about the stubbed
ID, so a verdict about that set lands where the set is written, as a duplicate declaration
reached through stubs is named at its target ([§FS-declarations.stubs.pairing.3](../../functional-spec/FS-declarations.md#stubspairing3-a-declaration-reached-through-stubs-is-named-at-its-target)).

### 2.1 Reported whether or not anything cites it

The uncited tree is moved too, and it needs its own argument, because there no citation is
being guessed today. The guess is latent. The section set is what `check` resolves every
citation of `FS-x.1` through, so while that set claims `1` twice, a `<§>FS-x.1` written tomorrow
resolves to the first heading and the run that reads it reports nothing new: the commit that
writes it passes, and the wrong destination is in the tree before any finding could name it.
That is why the scanned tree reports a duplicate section whether or not anything cites it
([§DF-duplicate-section-path.2.1](DF-duplicate-section-path.md#21-the-collision-is-an-error-in-fs-declarationschecksduplicates-shape)): the collision is a property of the declaration, and a
coordinate with two destinations is wrong before its first reader. Reporting only the cited
case would leave the outside tree green over exactly the construct the scanned tree refuses,
and would move the error from the declaration that has to change to whichever file cites it
first.

## 3. Rejected alternative: `ambiguous-section` at the citation

`check` would refuse the citation as `show` does, at the citing line. `check` has never emitted
that code, so it would be a newly minted finding, which D8 puts on the warning release. It
would also leave an uncited duplicate unreported, which the scanned tree reports, and it would
locate the error at a line that has nothing to fix: the citation is right as written.

## 4. Rejected alternative: a warning

[§DF-duplicate-section-path.4](DF-duplicate-section-path.md#4-rejected-alternative-a-warning) refused a warning for the same collision in a scanned file, to keep
one ambiguity at one severity. A target outside the include is the same declaration read on
the stub's behalf, so a warning here would give one collision two severities, chosen by the
scan's scope.

## 5. Consequences

A tree whose stub points outside the include at a target that claims a section path twice
goes from pass to fail, cited or not: `check` exits 1 with the scanned tree's
`duplicate-section` error, located in the target. Every other tree answers as before. A
path-scoped run keeps the finding by its sites ([§FS-check.1.3.6.2](../../functional-spec/FS-check.md#1362-a-finding-that-spans-several-sites-is-in-scope-at-any-of-them)), so `grund check docs`
reports it only where a site lies under `docs`, as the scanned tree's narrowed run does. The
editor shows it at the target's heading when that file is open, where it already shows a
duplicate declaration reached through stubs. Row U01 leaves [§REQ-one-answer.differences.stubs](../../requirements/REQ-one-answer.md#differencesstubs-stubs).

## 6. The correction route

[§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) licenses the pass-to-fail move, and each of its five conditions holds.

1. *Prior prohibition.* The old verdict violated [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution), which forbids "silently preferring the first of two identical section paths" as "the shape of a guess even when it is deterministic", and that requirement already applied when the verdict shipped: its sentence is in every tag from v0.10.1, and the verdict shipped in v0.17.0 with [§DF-stub-sections-from-unscanned-target](DF-stub-sections-from-unscanned-target.md#df-stub-sections-from-unscanned-target-a-stubs-sections-are-its-targets-whether-or-not-the-scan-reaches-it), which made `check` resolve a citation through an outside target's sections. The cited tree is that guess taken; the uncited tree is that guess written down as the rule its next citation will be resolved by, which section 2.1 argues on its own terms. The same release's [§FS-declarations.checks.duplicate-section](../../functional-spec/FS-declarations.md#checksduplicate-section-duplicate-section-path) already said the headings that count for a stub are the target's, "which is also the file `grund <ID>.<path>` reads".
2. *Accepted proof.* This record. The [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path does not fit, because there is no old form a user chose and could keep beside a new one: the stub and the citation are right as written, and the only old answer is the guess. The [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) mechanical migration has no command to point at, because no command renumbers a heading; the remedy is the author's choice of which heading keeps the path.
3. *Named release.* The release note below names the verdict change, cited and uncited, for the release that ships it.
4. *Actionable findings.* The finding that replaces the old verdict is the existing `duplicate-section` error, located at the first colliding heading and naming every other one in its message and its `sites`; its fix is to renumber one heading ([§DF-duplicate-section-path.2.6](DF-duplicate-section-path.md#26-nothing-that-worked-stops-working)).
5. *No new licence.* This is not a licence for anything else: no rule, finding code or text is added, the sections judged are the ones every command already resolves through, and the target's own hygiene stays unjudged as D8 rules.

## release-note: Release note

- [§DF-stub-target-section-declared-twice](DF-stub-target-section-declared-twice.md#df-stub-target-section-declared-twice-a-section-a-stubs-target-declares-twice-is-reported-where-it-is-written-scanned-or-not), [§FS-declarations.stubs.target.5](../../functional-spec/FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution): **a section a stub's target declares twice is reported where it is written, whether or not the scan reaches the target.** A stub that points outside `[scan] include` at a declaration claiming one section path twice now fails `check` with the `duplicate-section` error the same tree reports with the target scanned, such as `src/x.rs:5: error: duplicate section FS-x.1 (also declared at src/x.rs:9)`. **Who this breaks:** the verdict changes from pass to fail for such a tree whether or not anything cites the section. Where something cites it, `check` used to resolve the citation to the first heading while `show` refused it as ambiguous; where nothing does, the next citation of that section would have been resolved the same way, silently. Every finding that replaces the old verdict is the existing error, naming its location in the target and every colliding heading, and the action is to renumber one of them.
