# DF-one-answer-rulings-stubs: a stub is found by its home and shape, read as the scan reads, and shown where its target is

**Date:** 2026-10-11

## status: Accepted

## context: Context

[Issue #576](https://github.com/agent-grounds/grund/issues/576) unifies how every command reads
a tree under [§AR-system.principle](../../architecture/README.md#principle-one-fact-one-point-one-implementation), the rule [§DA-one-fact-one-point-one-implementation](../architectural/DA-one-fact-one-point-one-implementation.md#da-one-fact-one-point-one-implementation-every-fact-grund-decides-has-one-defining-point-one-implementation-and-reads-of-it-everywhere-else) records,
and the promise [§REQ-one-answer](../../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree) makes to users. Its author ruled D1 to D21 before any copy is
unified, and a cross-family agora settled a compatibility route for each ruling (entries
V-A02 to V-A26 of its verdict). This record holds the rulings of the step family *stubs*:
what a stub is, what its target contributes, and where the stub's definitions live, beside
the verdict [§FS-declarations.stubs.verdict](../../functional-spec/FS-declarations.md#stubsverdict-one-verdict-per-stub-reached-once-per-scan) already owns.

## decision: Decision

The rulings stand as follows. Each one that moves a verdict names its route: the warning
release of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path), or the one-release correction of
<§>REQ-backwards-compatibility.5 where an existing requirement already forbade the old
verdict, and then the record names that requirement.

- **D2** (V-A04). A stub is a heading whose tail is one link, at any heading level, in a
  file of a kind's home as [§FS-terms](../../functional-spec/FS-terms.md#fs-terms-the-shared-vocabulary-of-the-functional-spec-and-the-architecture) defines it; the literal `docs/` test and the H1 test
  go. Route: the warning release, [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path), because no requirement
  forbids today's narrower shape and widening it can move duplicate verdicts.
- **D3** (V-A05). The ID inside a target is declared by the scanner's own pass, as if the
  target were scanned; [§FS-declarations.checks.broken-stub](../../functional-spec/FS-declarations.md#checksbroken-stub-broken-inline-spec-stub) already says so. A code path that
  still disagrees takes the warning release, [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path), since sharing a
  parser alone proves no forbidden verdict.
- **D3**, the conditional part (V-A05). A reader that is shown to resolve a citation to the
  wrong declarations, such as one counting a fenced heading in a target and so calling a
  healthy citation ambiguous, is a candidate for <§>REQ-backwards-compatibility.5 under
  [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution) or [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms), with
  [§DF-stub-pairs-with-unscanned-target.5](DF-stub-pairs-with-unscanned-target.md#5-the-correction-route) as the precedent.
- **D4** (V-A06). Plain `list` shows a healthy stub to an out-of-scan target at the target,
  with its title, as `list --size` already does and [§FS-list.3.4.6](../../functional-spec/FS-list.md#346-a-home-outside-the-scan) says. Placement is a
  presentation choice no requirement governs. Route: the warning release,
  [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path).
- **D8** (V-A10). A target read on behalf of a stub contributes every fact resolution uses
  and reports no finding of its own. `check` silently accepting a citation into a section
  path the target declares twice, which `show` refuses as [§DF-duplicate-section-path](DF-duplicate-section-path.md#df-duplicate-section-path-a-section-coordinate-names-one-heading-or-the-run-says-so) decided,
  is a candidate for <§>REQ-backwards-compatibility.5 under [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution), which
  names silently preferring the first of two identical section paths as a guess.
- **D8**, every other part (V-A10). Other reporting or fact-import changes, and any newly
  minted finding code, take the warning release, [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path), as
  [§DF-glob-citation.2](DF-glob-citation.md#2-verdict-correction-and-compatibility) split its code. Target-content suppression under
  [§FS-declarations.checks.oversized-lead.4](../../functional-spec/FS-declarations.md#checksoversized-lead4-which-sites-it-judges) stays.
- **D13** (V-C04). A stub whose target is itself a stub is broken. A chain through a
  recognised stub is already reported broken on `main`, and every command agrees, so the
  ruling moves no verdict and has no route.
- **D15** (V-A15, amended to `main`'s state). The stub definitions live in the `stubs`
  chapter of [§FS-declarations](../../functional-spec/FS-declarations.md#fs-declarations-a-declaration-is-addressable-once-from-one-allowed-place-and-holds-nothing-that-is-neither-a-coordinate-nor-a-finding): `stubs.verdict` already exists, the agreement bullets of
  `checks.broken-stub.4` moved to [§REQ-one-answer](../../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree) in step 0a, and the stubs step adds the
  shape, target and pairing sections and moves `broken-stub.1–.3` and `duplicate.1–.3`.
  Placement of text, not of a verdict: no route.
- **D16** (V-A16). An ID has declarations, not homes; [§FS-terms](../../functional-spec/FS-terms.md#fs-terms-the-shared-vocabulary-of-the-functional-spec-and-the-architecture) defines a home as where a
  kind's declarations live. The residual misuses are fixed when their text is touched, and
  each visible wording change takes the warning release, [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path).
  Shipped fields, such as the `home` key of `list --summary`, and accepted decision-record
  text, such as [§DF-stub-target-declared-twice](DF-stub-target-declared-twice.md#df-stub-target-declared-twice-a-stubs-target-that-declares-its-id-twice-is-two-homes-scanned-or-not)'s title, keep their words.

A correction route named here is a candidate, not a pre-approved record. The correcting
step's own accepted decision record must prove all five conditions of
<§>REQ-backwards-compatibility.5 and carry the release note [§FS-distribution.4.6.4](../../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions) publishes.
Changed finding text stays on [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) even inside a correcting
release, and [§REQ-one-answer](../../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree) is never the requirement a correction rests on (V-A25).

## alternatives: Alternatives refused

- **Ship D2 and D5 together with D8's correction.** A wider recognition and a stricter walk
  break no requirement; only D8's silent pick does, so only it may skip the warning.
- **Correct D3 on the shared parser alone.** One parser is a design; a correction needs a
  demonstrated wrong verdict.
- **Rename every `home` in shipped output.** Accepted text and shipped fields are frozen
  surfaces; renaming them would be a break, not a wording fix.

## consequences: Consequences

This record publishes no notice and moves no verdict: it carries no `release-note`, and each
ruling's own step writes the decision record that does. That step cites this record for the
ruling and its route rather than deciding the route again.

## why: Why

[§GOAL-no-silent-breakage](../../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path) asks that anything user-visible either stays compatible or crosses
a named window. Writing each ruling's window down before the change that moves it is what
keeps that change from choosing its own.
