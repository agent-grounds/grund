# DA-one-fact-one-point-one-implementation: every fact grund decides has one defining point, one implementation, and reads of it everywhere else

**Status:** Accepted
**Date:** 2026-10-11

## context: Context

From 2026-10-07 to 2026-10-09 grund received 27 bug issues, most of them in three chains:
a stub whose target is outside the scan, `fmt` re-reading a heading instead of the scan's
record, and selector and rule-sentence refusals. Each fix changed one copy of a rule, and
its review found the next copy. Few were regressions; the copies were there before.

The cause is that grund decides each of five facts in many places: which files a run reads
(seven loaders and about eighteen scope predicates), the text of a file (about fifteen
readers, ten of them blind to the editor's unsaved text), a file's structure (the scanner
and about fourteen re-reads after it), which declarations an ID has through a stub, and
what a rule sentence or coordinate means. The specification has the same shape: most of
these facts have no definition chapter, a rule lives in the finding or the command that met
it first, and code cites whichever restatement was nearest. So `grund refs` cannot list the
copies, and a reviewer has to find them. The stub verdict of
[§FS-declarations.stubs.verdict](../../functional-spec/FS-declarations.md#stubsverdict-one-verdict-per-stub-reached-once-per-scan) showed the remedy for one family: one verdict, read by every
command, closed every disagreement its agreement test could find.

## decision: Decision

The rule is recorded as [§AR-system.principle](../../architecture/README.md#principle-one-fact-one-point-one-implementation), and the promise to users it secures as
[§REQ-one-answer](../../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree). Each family is unified in three passes, in order: its defining point is
settled in the specification and the restatements become pointers to it; every copy in the
code cites the point; then one owning function derives the fact and the copies are
deleted. A family is not unified ahead of a ruling on which copy is right, and a ruling
that moves a verdict ships under [§REQ-backwards-compatibility](../../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly) on its own decision record.

## alternatives: Alternatives refused

- **Keep fixing copies one at a time.** That is what produced the chains: each fix is
  correct and the next copy is still there.
- **State the rule as a requirement.** A requirement is a promise to users that outlives
  every rewrite of the implementation, and may not cite the architecture; the rule is about
  how the engine is built. The user-facing half is [§REQ-one-answer](../../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree).
- **A shared agreement point per family**, such as one beside the stub verdict. Every
  command reading the verdict is the rule applied, not a fact of its own, and a second point
  per family is a second place the agreement is written down.

## consequences: Consequences

The migration is a sequence of changes, each its own issue: the remaining new definition
points (citations and their zones, how a source file's lines are read, what a run scans,
rule refusals, and chapters of [§FS-declarations](../../functional-spec/FS-declarations.md#fs-declarations-a-declaration-is-addressable-once-from-one-allowed-place-and-holds-nothing-that-is-neither-a-coordinate-nor-a-finding) for stubs and structure); guards that make
a second copy fail CI — a citation ledger per defining point, a pointer guard, primitive
guards, a command-agreement comparison that proves [§REQ-one-answer](../../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree), grammar properties,
and a differential before each old path is deleted; then one run and one read, one parse,
stubs, `fmt` and the editor on the record, and one grammar. None of these guards exists
when this record is accepted.

As accepted, the families stand as follows; the guards make this list executable once they
land.

| Family | Defining point | Owner |
|---|---|---|
| Which files a run reads; what a path argument means | [§FS-check.1.3.6.1](../../functional-spec/FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution), [§FS-config.3.5](../../functional-spec/FS-config.md#35-scan--what-gets-scanned) (planned: a run spec) | the walk and seven loaders |
| The text of a file | [§FS-declarations.checks.broken-stub.1](../../functional-spec/FS-declarations.md#checksbroken-stub1-the-target-is-read-as-a-save-would-write-it) for stub targets only | about fifteen readers |
| Structure: body, sections, headings, anchors, comment blocks, fences, citation zones | [§FS-show.2.1](../../functional-spec/FS-show.md#21-whole-declaration-default), [§FS-show.2.1.2.1](../../functional-spec/FS-show.md#2121-the-map-is-the-declarations-body-span), [§FS-fmt.6.2](../../functional-spec/FS-fmt.md#62-form) | the scanner and about fourteen re-reads |
| Which declarations an ID has through a stub | [§FS-declarations.stubs.verdict](../../functional-spec/FS-declarations.md#stubsverdict-one-verdict-per-stub-reached-once-per-scan) | the stub resolution of the model |
| What a rule sentence or coordinate means | [§FS-rules.2](../../functional-spec/FS-rules.md#2-subject-selectors), [§FS-rules.3.5](../../functional-spec/FS-rules.md#35-strict-refusals) | the rule parser and about twenty-five lexers |

## why: Why

A citation an agent cannot trust grounds nothing ([§GOAL-no-dangling-refs](../../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration)), and the loop of
[§GOAL-agent-grounding](../../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work) holds only if every surface an agent reads gives the answer the gate
passed. Writing each fact once, and deriving it once, is what lets the specification find
the copies instead of a reviewer.
