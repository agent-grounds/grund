# DF-one-answer-rulings-parse: one parse gives every command the same extents, comment blocks, anchors and title citations

**Date:** 2026-10-11

## status: Accepted

## context: Context

[Issue #576](https://github.com/agent-grounds/grund/issues/576) unifies how every command reads
a tree under [§AR-system.principle](../../architecture/README.md#principle-one-fact-one-point-one-implementation), the rule [§DA-one-fact-one-point-one-implementation](../architectural/DA-one-fact-one-point-one-implementation.md#da-one-fact-one-point-one-implementation-every-fact-grund-decides-has-one-defining-point-one-implementation-and-reads-of-it-everywhere-else) records,
and the promise [§REQ-one-answer](../../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree) makes to users. Its author ruled D1 to D21 before any copy is
unified, and a cross-family agora settled a compatibility route for each ruling (entries
V-A02 to V-A26 of its verdict). This record holds the rulings of the step family *one parse*:
what the scan records about a file's structure, so no command re-reads it.

## decision: Decision

The rulings stand as follows. Each one that moves a verdict names its route: the warning
release of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path), or the one-release correction of
<§>REQ-backwards-compatibility.5 where an existing requirement already forbade the old
verdict, and then the record names that requirement.

- **D9** (V-A11, amended). Every run builds complete records, and the cost is held by the
  instruction-count limits [§AR-ci.5.2](../../architecture/AR-ci.md#52-regression-limits) already states (5% on the 10k-file fixture, 12.5% on
  the other hot commands), wired to fail the build before the complete-record work merges;
  that step reports against [§GOAL-fast-feedback.1](../../goals.md#1-performance-targets). A budget, not a verdict: no route.
- **D17a–d** (V-A17). A deeper declaration does not end its parent's body (a); a section
  ends at the next heading of its level or above (b); `//` and `///` are separate comment
  blocks (c); a trailing `*/` is part of a C-family comment's wrapper, and a literal `*/` in
  Markdown or a docstring stays in its label (d), as [§FS-languages.comment-wrapper.headings](../../functional-spec/FS-languages.md#comment-wrapperheadings-the-wrapper-of-a-heading-line) says. Each changes what
  `show`, `refs` and `list --size` print. Route: the warning release,
  [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path), because no requirement forbids today's extents.
- **D17e** (V-A18, amended). Anchors come from the complete rendered text of the authored
  heading, under its renderer profile, with no generic trailing-hyphen trim ([§FS-fmt.6.2.3](../../functional-spec/FS-fmt.md#623-the-github-profile-reproduces-github-slugger)).
  The fragment `fmt` persists for a heading with closing hashes, `#1-steps-` for
  `## 1. Steps ##`, names no heading the file carries. Its fix is a candidate for
  <§>REQ-backwards-compatibility.5 under [§REQ-no-wrong-citation.3](../../requirements/REQ-no-wrong-citation.md#3-no-wrong-write), read through
  [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution), and `show`'s JSON `anchor` moves with it in the same release, as
  [§FS-show.3.1.3.1](../../functional-spec/FS-show.md#3131-the-heading-anchor) binds it to `fmt`'s.
- **D17e**, the title (V-A18). The TOC `title` bytes, `Steps ##` becoming `Steps`, are a
  separate output and take the warning release, [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path).
- **D17f** (V-A19). On a declaration's heading, only a citation of the ID it declares is
  dropped; a citation of another ID, as in `# FS-a: Builds on` a cited `FS-b`, is a
  citation. Its fix is a candidate for <§>REQ-backwards-compatibility.5 under
  [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded), because no section names that blind spot and [§AR-scanner.2.3](../../architecture/AR-scanner.md#23-citation-detection)
  drops only the declared ID. Any rewriting of the title text takes its own route.

D17g, the link wrapper, belongs to the `fmt` and editor step and is recorded in
[§DF-one-answer-rulings-fmt-editor](DF-one-answer-rulings-fmt-editor.md#df-one-answer-rulings-fmt-editor-fmt-writes-only-into-a-complete-link-wrapper-never-into-a-link-destination).

A correction route named here is a candidate, not a pre-approved record. The correcting
step's own accepted decision record must prove all five conditions of
<§>REQ-backwards-compatibility.5 and carry the release note [§FS-distribution.4.6.4](../../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions) publishes;
for D17e it must show the closing-hash case itself, since citing
[§DF-stub-heading-from-unscanned-target.6](DF-stub-heading-from-unscanned-target.md#6-the-correction-route) is not enough on its own. Changed finding text stays
on [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) even inside a correcting release, and [§REQ-one-answer](../../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree) is
never the requirement a correction rests on (V-A25).

## alternatives: Alternatives refused

- **Wait for every performance issue before complete records.** The instruction gate is the
  one limit stated and not enforced; switching it on is enough to catch the cost.
- **Correct D17a–d at once.** Today's extents break no requirement, so they are changed
  behaviour, not a defect, and users see the new bytes only after a warning.
- **Move D17e's title with its anchor.** The anchor is a wrong write; the title is text a
  reader may match on, and text changes take the warning release.

## consequences: Consequences

This record publishes no notice and moves no verdict: it carries no `release-note`, and each
ruling's own step writes the decision record that does. That step cites this record for the
ruling and its route rather than deciding the route again.

## why: Why

[§GOAL-no-silent-breakage](../../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path) asks that anything user-visible either stays compatible or crosses
a named window. Writing each ruling's window down before the change that moves it is what
keeps that change from choosing its own.
