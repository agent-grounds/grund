# DF-one-answer-rulings-run: a path narrows the report, a stub reads what the walk reads, and a file is one file however it is spelled or held

**Status:** Accepted
**Date:** 2026-10-11

## context: Context

[Issue #576](https://github.com/agent-grounds/grund/issues/576) unifies how every command reads
a tree under [§AR-system.principle](../../architecture/README.md#principle-one-fact-one-point-one-implementation), the rule [§DA-one-fact-one-point-one-implementation](../architectural/DA-one-fact-one-point-one-implementation.md#da-one-fact-one-point-one-implementation-every-fact-grund-decides-has-one-defining-point-one-implementation-and-reads-of-it-everywhere-else) records,
and the promise [§REQ-one-answer](../../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree) makes to users. Before any copy is unified, its author ruled
on which copy is right: rulings D1 to D21, argued by a cross-family agora whose verdict
settled a compatibility route for each one (entries V-A02 to V-A26 of that verdict). This
record holds the rulings of the first step family, *one run, one read*: which files a run
reads, what a path argument means, and what one file is.

## decision: Decision

The rulings stand as follows. Each one that moves a verdict names its route: the warning
release of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path), or the one-release correction of
<§>REQ-backwards-compatibility.5 where an existing requirement already forbade the old
verdict, and then the record names that requirement.

- **D1** (V-A02). A path argument to `check`, `list`, `show` or `id` narrows what is
  reported; resolution always covers the owning project. [§FS-check.1.3.6.1](../../functional-spec/FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution) is the defining
  point, already decided by [§DF-path-scope-resolves-project-wide](DF-path-scope-resolves-project-wide.md#df-path-scope-resolves-project-wide-a-path-scoped-check-resolves-against-the-whole-project-and-reports-only-the-path), and [§FS-show.1.4](../../functional-spec/FS-show.md#14-path-and---path) and
  [§FS-list.1](../../functional-spec/FS-list.md#1-inputs) are the copies to correct. Route: the warning release,
  [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path). A query operand is not a citation, so no existing
  requirement forbade the old narrowing.
- **D14** (V-A03). In a workspace, `check <dir>` first discovers the projects a run from that
  directory would see, as `list <dir>` does today and as [§FS-workspace.8.8](../../functional-spec/FS-workspace.md#88-discovery-follows-grund-checks-walk-up) says, then
  resolves each of them completely; a member directory still means a member run. Read that
  way D14 and D1 compose. Route: the warning release, [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path), since
  [§FS-check.1.3.6.3](../../functional-spec/FS-check.md#1363-a-file-the-wider-walk-could-not-read-is-said-out-loud) classifies today's `unknown project alias` as a knowingly scoped finding.
- **D5** (V-A07). A stub's target must pass the walk's own filters, exclude and ignore
  files, with only the `include` gate removed; a stub to an excluded or ignored target turns
  broken under [§FS-declarations.checks.broken-stub](../../functional-spec/FS-declarations.md#checksbroken-stub-broken-inline-spec-stub). Route: the warning release,
  [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path), because this tightens a policy and the correction route
  refuses tightening.
- **D6** (V-A08). File identity is physical, for pairing, caching and buffers; a name is
  judged as written before pairing. Route: the warning release,
  [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path), because no false duplicate from lexical identity has been
  reproduced on `main`.
- **D6**, the conditional part (V-A08). A false duplicate caused by lexical-versus-physical
  identity, if one is ever reproduced, is a candidate for <§>REQ-backwards-compatibility.5
  under [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution) or [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms), with
  [§DF-stub-pairs-with-unscanned-target.5](DF-stub-pairs-with-unscanned-target.md#5-the-correction-route) as the precedent.
- **D7** (V-A09). A file that exists only as an unsaved editor buffer exists. Its reproduced
  false answers, a stub reported broken and a declaration reported duplicated until the same
  bytes are saved, are a candidate for <§>REQ-backwards-compatibility.5 under
  [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms) read through [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution), the pair
  [§DF-stub-pairs-with-unscanned-target.5](DF-stub-pairs-with-unscanned-target.md#5-the-correction-route) already calls false alarms. [§FS-lsp.1.1](../../functional-spec/FS-lsp.md#11-diagnostics) makes the
  editor's diagnostics `check`'s own findings.
- **D7**, every other part (V-A09). Any other change to which buffers count or how the
  overlay applies takes the warning release, [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path).
- **D12** (V-A14). When to start: now, on current `main` from a420f558af, because `Run` and
  `Catalog` are already there. A ruling about process; it moves no verdict and has no route.

A correction route named here is a candidate, not a pre-approved record. The correcting
step's own accepted decision record must prove all five conditions of
<§>REQ-backwards-compatibility.5 and carry the release note [§FS-distribution.4.6.4](../../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions) publishes.
Changed finding text stays on [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) even inside a correcting
release, and [§REQ-one-answer](../../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree) is never the requirement a correction rests on (V-A25).

## alternatives: Alternatives refused

- **Ship D1 and D14 in one release.** No existing requirement forbade the old narrowing, so
  there is nothing to correct, and a one-release change would move `show`'s and `check`'s
  answers with no warning.
- **Correct D6 at once.** A correction needs an old verdict that broke a requirement; an
  unreproduced verdict cannot be shown to break one.
- **Send D7 through a warning release.** The editor would keep reporting, for one more
  release, findings that saving the same bytes clears.

## consequences: Consequences

This record publishes no notice and moves no verdict: it carries no `release-note`, and each
ruling's own step writes the decision record that does. That step cites this record for the
ruling and its route rather than deciding the route again.

## why: Why

[§GOAL-no-silent-breakage](../../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path) asks that anything user-visible either stays compatible or crosses
a named window. Writing each ruling's window down before the change that moves it is what
keeps that change from choosing its own.
