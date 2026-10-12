# DF-stubs-by-home-and-at-the-target: a rule declared through a stub applies now, and a stub is read by its home and shown at its target from 0.19.0

**Status:** Accepted
**Date:** 2026-10-12

## 1. Context

[§DF-one-answer-rulings-stubs](DF-one-answer-rulings-stubs.md#df-one-answer-rulings-stubs-a-stub-is-found-by-its-home-and-shape-read-as-the-scan-reads-and-shown-where-its-target-is) ruled what a stub is, what its target gives it and where it is
shown, and named each ruling's route. This record is the step that writes those definitions
down ([§FS-declarations.stubs](../../functional-spec/FS-declarations.md#stubs-what-a-stub-points-at)) and closes or schedules the stub differences
[§REQ-one-answer.differences.stubs](../../requirements/REQ-one-answer.md#differencesstubs-stubs) lists. One of them, U01, is a correction with its own record,
[§DF-stub-target-section-declared-twice](DF-stub-target-section-declared-twice.md#df-stub-target-section-declared-twice-a-section-a-stubs-target-declares-twice-is-reported-where-it-is-written-scanned-or-not). Everything else here takes the warning release of
[§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): no requirement forbids today's answer, and the author's answer
on [#576](https://github.com/agent-grounds/grund/issues/576) sends every ruling that only
makes commands agree through a warning release first.

## 2. Decision

### 2.1 A rule declared through a stub applies now, and its findings warn until 0.19.0

Row U05: `check` read a rule's sentence off the stub's own heading, whose title is a link, and
reported `<ID> is not a valid rule: rule declaration has no title`, while `grund <ID> --brief`
printed the sentence the target declares. The row named a target outside the include, but the
error fired with the target scanned too, because the rule loader read the stub in the RULE
home rather than the declaration it points at; with the target scanned the rule also applied,
from the declaration the walk read, so that tree carried the false error beside the rule's own
findings.

The loader now reads the sentence and the rationale from the declaration the ID pairs with
([§FS-rules.7.10](../../functional-spec/FS-rules.md#710-a-rule-declared-through-a-stub-warns-until-0190)). The false `invalid-rule` goes in the release that ships this, because
removing an error breaks no run, and a [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) window "is owed only
where something will break" ([§FS-config.1.2.2](../../functional-spec/FS-config.md#122-why-no-deadline-is-owed)). What can break is a run that deselects
`invalid-rule` today and would start seeing a rule it never ran: where the target is outside
the include, the rule's required-level findings warn until 0.19.0, in the ramp vocabulary
[§FS-rules.7.8](../../functional-spec/FS-rules.md#78-a-newly-counted-section-citation-warns-until-0180) and [§FS-rules.7.9](../../functional-spec/FS-rules.md#79-a-target-pinned-at-its-own-nested-alias-warns-until-0180) already use. Where the scan reads the target nothing new
applies, so nothing warns.

### 2.2 What a stub is changes in 0.19.0, and `check` warns now

D2: a stub is a heading whose title is one link, at any level, in a file of some kind's home
([§FS-declarations.stubs.shape](../../functional-spec/FS-declarations.md#stubsshape-what-a-stub-looks-like)). Today it is such a heading in a Markdown file with a directory
named `docs` anywhere on its path. In this release nothing is read differently; every heading
the two tests read differently gets `stub-shape` ([§FS-declarations.checks.stub-shape](../../functional-spec/FS-declarations.md#checksstub-shape-stub-shape-read-differently-from-grund-0190)), naming
the edit that keeps today's reading. The warning fires at every such heading, not only where a
verdict would move, because every one of them changes what its ID's declaration is, and
finding which of them move a verdict would take a second scan under the other test. The H1
test D2 also removes was never in the code: every command already reads a stub at any heading
level, so that half corrects the text alone.

The `docs` test reads the whole path, so a checkout below a directory named `docs` reads every
heading of the stub shape as a stub. That leak of the checkout's location into the answer
goes with the `docs` test in 0.19.0; until then the warning names it wherever it changes a
reading.

### 2.3 A stubbed declaration is shown at its target from 0.19.0

D4 and the fact-import part of D8 move these answers in 0.19.0, each to the answer the same
tree gives today with the target scanned ([§FS-declarations.stubs.pairing.4](../../functional-spec/FS-declarations.md#stubspairing4-every-surface-shows-a-paired-declaration-at-its-target)):

- **Placement.** Plain `list` shows a healthy stub's declaration at its target, with its title:
  `FS-x  src/x.rs:1  X` rather than `FS-x  docs/fs/x.md:1  → ../../src/x.rs`.
- **U06.** `check` places the `unused` warning at the target's line, not the stub's.
- **U03a to U03d.** `show --batch --all`, shell completion, the editor's completion and
  navigation, and `list --selector` read an outside target's sections, as `show` already does.

These only add or move answers, but D8 puts fact-import changes on the warning route, and
U03a moves `show --batch --all`'s exit code on U01's tree, from 0 to 1, where `check` already
fails. The rows stay listed until the release that moves them.

### 2.4 What moves no answer

D3 and D13 already hold on every command: a target declares the ID as the scanner's own pass
reads it ([§FS-declarations.stubs.target](../../functional-spec/FS-declarations.md#stubstarget-what-a-stubs-target-gives-it)), and a chain of stubs is broken
([§FS-declarations.stubs.target.4](../../functional-spec/FS-declarations.md#stubstarget4-a-stub-in-the-target-declares-nothing)). D15 is where the text sits, and D16 is which words it uses;
no command prints "home" for the place an ID is declared, so no visible wording changes, and
`list --summary`'s `home` key and the text of accepted records keep their words.

## 3. Rejected alternatives

- **U05's findings as errors at once.** A run that passes today with `--ignore invalid-rule`
  would go from pass to fail with no window.
- **Keep `invalid-rule` for one more release.** That keeps a false error in every plain run, to
  protect nobody.
- **U03a to U03d at once, because they only add answers.** The author's answer on #576 covers
  changes that only make commands agree, and #595's R06a to R06d took the same route.
- **Warn about D2 only where a verdict moves.** That needs a second scan under the other test,
  for a warning that every such heading earns anyway.

## release-note: Release note

- [§DF-stubs-by-home-and-at-the-target](DF-stubs-by-home-and-at-the-target.md#df-stubs-by-home-and-at-the-target-a-rule-declared-through-a-stub-applies-now-and-a-stub-is-read-by-its-home-and-shown-at-its-target-from-0190), [§FS-rules.7.10](../../functional-spec/FS-rules.md#710-a-rule-declared-through-a-stub-warns-until-0190), [§FS-declarations.checks.stub-shape](../../functional-spec/FS-declarations.md#checksstub-shape-stub-shape-read-differently-from-grund-0190), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **a rule declared through a stub now applies, and from 0.19.0 a stub is read by its home and shown at its target.** `check` no longer reports `rule declaration has no title` for a rule written as a stub, whether or not the scan reaches its target; where the target is outside `[scan] include` the rule now runs, and its findings are warnings until 0.19.0, so a run that passed with `--ignore invalid-rule` still passes. From 0.19.0 a heading whose title is one link is a stub only in a file of some kind's home, not wherever a directory named `docs` is on its path; `check` warns now, as `stub-shape`, at every heading that reading changes, and the exit code does not move. Also from 0.19.0, a stub to a target outside the scan is shown at the target by plain `list` and by the `unused` warning, and `show --batch --all`, shell completion, the editor and `list --selector` see the target's sections. **Who this affects:** repositories that write a rule as a stub, or a stub-shaped heading outside every kind's home or in a home with no `docs` on its path; each warning names the edit that keeps today's reading.
