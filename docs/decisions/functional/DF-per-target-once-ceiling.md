# DF-per-target-once-ceiling: spell a per-target ceiling of one as at most once

**Date:** 2026-10-19

## status: Accepted

## context: The old spelling was defined, but not taught

Issue #835 records `grund 0.18.1-dev` at
`d76f5248fce024d83dc3d66781b06c668bcdac6b` refusing `at most once` with
`per-target counts must end in "times"`, while accepting `at most 1 times`.
The lower-bound control accepted `at least once` and refused `at least 1 times`
with its canonical rewrite. The parser followed the then-current grammar:
[§FS-rules.3](../../functional-spec/FS-rules.md#3-the-five-sentence-families) required numeric upper-one, and [§FS-rules.3.3](../../functional-spec/FS-rules.md#33-per-target-coverage) listed only the
numeric per-target ceiling. This changes that grammar; it does not correct a
verdict forbidden by a prior hard requirement.

The author's approval on #835 (2026-10-19, issue comment 6259521618) records
the compatibility evidence: no guide, example or skill taught `at most 1 times`;
the only use found was the repository's own `mixed_path_errors.rs` fixture.
The user guide already advised plural `times` for numeric counts other than
one. This evidence supports a narrow rename, not an assertion that users could
never have written a sentence the grammar allowed.

## decision: One per-target spelling, with immediate refusal of numeric one

Per-target ceilings use [§FS-rules.3.3](../../functional-spec/FS-rules.md#33-per-target-coverage)'s `at most once`. Numeric upper-one
is immediately refused with [§FS-rule-refusals.sentences](../../functional-spec/FS-rule-refusals.md#sentences-strict-refusals)'s actionable rewrite;
typed offers and malformed-one recovery follow
[§FS-rule-refusals.sentences.rewrite.by-refusal](../../functional-spec/FS-rule-refusals.md#sentencesrewriteby-refusal-the-rewrite-each-refusal-makes). No alias or warning window is
introduced. Physical-site evaluation and finding text stay as specified there.

Only per-target counts change: `at most 1 chapter` and `at most 1 AR` read
naturally, while `at most 1 times` does not. Chapter, outbound and inbound
counts retain their spellings ([§FS-rules.3](../../functional-spec/FS-rules.md#3-the-five-sentence-families)); lower and exact per-target one
already use `once`.

## compatibility: Use the pre-1.0 allowance, not a verdict correction

The default window of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) would preserve the old
spelling for a release. Instead this decision uses the explicit pre-1.0
allowance in [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise), which expires at 1.0: carrying
an alias costs more than this rename. A warning window would carry two
spellings, warning metadata through configured and trial-rule paths,
release-gated removal and duplicate teaching, for one local text replacement
that no guide, example or skill previously taught. Permanent numeric
acceptance would leave the requested refusal unmet. The author approved
immediate refusal with this evidence and release notice.

The section 5 correction route of [§REQ-backwards-compatibility](../../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly) is not used: the old grammar
expressly permitted this spelling, and no separate prior prohibition is
claimed. Existing checks can newly fail and init can stop before writing until
the title is edited. Configured invalid rules locate the title and offer its
replacement ([§FS-rules.7.1](../../functional-spec/FS-rules.md#71-invalid-rule)); valid recommendations keep their opt-in,
exit-neutral channel ([§FS-rules.7.6](../../functional-spec/FS-rules.md#76-selection-json-ordering-and-exits)).

## release-note: Release note

- [§DF-per-target-once-ceiling](DF-per-target-once-ceiling.md#df-per-target-once-ceiling-spell-a-per-target-ceiling-of-one-as-at-most-once): **Per-target citation rules now spell a ceiling of
  one as `at most once`.** Replace `cite each AR at most 1 times.` with
  `cite each AR at most once.` (and likewise for other target kinds) in RULE
  titles and `check --rule` sentences. **Who this breaks:** a formerly passing
  check using numeric upper-one can now fail, and init refuses to write until
  the title is corrected; the refusal offers the accepted replacement. After
  editing rendered rules, rerun `grund init`. Other count families and the
  finding text `requires at most 1` are unchanged.
