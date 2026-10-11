# DF-one-answer-rulings-fmt-editor: `fmt` writes only into a complete link wrapper, never into a link destination

**Status:** Accepted
**Date:** 2026-10-11

## context: Context

[Issue #576](https://github.com/agent-grounds/grund/issues/576) unifies how every command reads
a tree under [§AR-system.principle](../../architecture/README.md#principle-one-fact-one-point-one-implementation), the rule [§DA-one-fact-one-point-one-implementation](../architectural/DA-one-fact-one-point-one-implementation.md#da-one-fact-one-point-one-implementation-every-fact-grund-decides-has-one-defining-point-one-implementation-and-reads-of-it-everywhere-else) records,
and the promise [§REQ-one-answer](../../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree) makes to users. Its author ruled D1 to D21 before any copy is
unified, and a cross-family agora settled a compatibility route for each ruling (entries
V-A02 to V-A26 of its verdict). This record holds the ruling of the step family *`fmt` and
the editor on the record*, where `fmt` rewrites at the spans the scan recorded and the
editor uses the scanner's line classifier.

## decision: Decision

The ruling stands as follows, split as V-A20 splits it. Each part names its route: the
warning release of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path), or the one-release correction of
<§>REQ-backwards-compatibility.5 where an existing requirement already forbade the old
verdict, and then the record names that requirement.

- **D17g**, the destination (V-A20). `fmt --write` rewrites a citation sitting inside a
  hand-authored link's destination, turning `See [the target](` and a cited ID `)` into a
  nested link, though [§FS-fmt.2.3](../../functional-spec/FS-fmt.md#23-what-is-never-rewritten) lists link destinations among what is never rewritten.
  Leaving that line alone is a candidate for <§>REQ-backwards-compatibility.5 under
  [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own), which bounds `fmt --write` to no other byte.
- **D17g**, the wrapper (V-A20). A link wrapper is `[§ID](dest)` with its closing `)`.
  Requiring that complete wrapper changes which existing wrappers `fmt` recognises, a
  policy no requirement fixes, so it takes the warning release,
  [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path).

A correction route named here is a candidate, not a pre-approved record. The correcting
step's own accepted decision record must prove all five conditions of
<§>REQ-backwards-compatibility.5 and carry the release note [§FS-distribution.4.6.4](../../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions) publishes.
Changed finding text stays on [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) even inside a correcting
release, and [§REQ-one-answer](../../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree) is never the requirement a correction rests on (V-A25).

## alternatives: Alternatives refused

- **One route for all of D17g.** Writing into a destination breaks a promise `fmt` already
  made; narrowing what counts as a wrapper breaks none. Shipping both by correction would
  move the wrapper grammar with no warning, and shipping both by warning would keep a
  forbidden write for another release.

## consequences: Consequences

This record publishes no notice and moves no verdict: it carries no `release-note`, and the
step's own decision record does. That step cites this record for the ruling and its routes
rather than deciding them again.

## why: Why

[§GOAL-no-silent-breakage](../../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path) asks that anything user-visible either stays compatible or crosses
a named window. Writing each ruling's window down before the change that moves it is what
keeps that change from choosing its own.
