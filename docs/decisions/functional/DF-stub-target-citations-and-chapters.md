# DF-stub-target-citations-and-chapters: a stub's target lends the stubbed declaration its citations and chapters, and what that newly raises warns until 0.19.0

**Status:** Accepted
**Date:** 2026-10-12

## 1. Context

A stub's target outside `[scan] include` already gives the stub its declarations and their
sections ([§FS-declarations.stubs.target.5](../../functional-spec/FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged)). Its citations and its chapters it did not give:
the target was read for its declarations, and everything else of it was thrown away. Take this
tree, with `src/` outside the include:

```text
grund.toml      [scan] include = ["docs"]
docs/fs/x.md    # FS-x: [x](../../src/x.rs)
docs/goals/g.md # GOAL-g: G
src/x.rs        /// FS-x: X                 line 1
                ///
                /// Lead, for §GOAL-g.       line 3
                ///
                /// ## terms: Terms          line 5
```

| Question | `src/` outside | `src/` scanned |
|---|---|---|
| `check --rule "Each FS must cite at least one GOAL."` | `docs/fs/x.md:1: error: FS-x must cite GOAL (--rule)` | passes |
| `check --rule "Each FS must not cite any GOAL."` | passes | `src/x.rs:3: error: FS must not cite GOAL (--rule) …` |
| `check --rule "Each FS must have exactly one Terms chapter."` | `docs/fs/x.md:1: error: FS-x has 0 Terms chapters; …` | passes |
| `grund refs GOAL-g` | nothing | `src/x.rs:3: <§>GOAL-g` |
| `check` | `GOAL-g` is unused | no warning |

These are rows U04, U07 and U08 of [§REQ-one-answer.differences.stubs](../../requirements/REQ-one-answer.md#differencesstubs-stubs). D8 of
[§DF-one-answer-rulings-stubs](DF-one-answer-rulings-stubs.md#df-one-answer-rulings-stubs-a-stub-is-found-by-its-home-and-shape-read-as-the-scan-reads-and-shown-where-its-target-is) rules that a target read on a stub's behalf "contributes every
fact resolution uses and reports no finding of its own", and sends every part of it but the
section correction through the warning release of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path). Its author
read "exactly as … scanned" on #651 to govern every verdict about the stubbed ID and every
verdict the stubbed declaration's content feeds: its rule and chapter obligations, the inbound
counts of what it cites, and its section set. "Reports no finding of its own" keeps out
findings about the target file itself.

## 2. Decision

### 2.1 The stubbed declaration's citations and chapters are lent, and nothing else

Each citation the scanner's pass over the target assigns to a declaration the stub pairs with
is that declaration's citation, and each of its chapters is its chapter, exactly as with the
target scanned ([§FS-declarations.stubs.target.5](../../functional-spec/FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged)). A lent citation that resolves is an edge: it
counts toward a rule's obligation, prohibition and counts ([§FS-rules.5.1](../../functional-spec/FS-rules.md#51-facts-and-identity)), toward a
`[citations]` obligation and prohibition ([§FS-check.3.11](../../functional-spec/FS-check.md#311-missing-required-citation), [§FS-check.3.12](../../functional-spec/FS-check.md#312-forbidden-citation)), and `refs` lists
it, so the ID it cites is not `unused` ([§FS-refs.2](../../functional-spec/FS-refs.md#2-behaviour)). A lent chapter is a rule's subject and is
counted by a chapter rule, cited or not ([§FS-rules.3.1](../../functional-spec/FS-rules.md#31-chapter-presence)). After this change, moving `src/` in or
out of `[scan] include` moves none of the answers in the table above.

### 2.2 The line D8 draws

A lent citation that does not resolve counts for nothing, as in the scanned tree, where an
unknown or ambiguous target is never an edge, and it draws no finding, because whether the
target's citations resolve is the target's own hygiene. So do its notes and formatting, its
leads, its grounding and its other declarations: none of them is reported, and a citation the
stubbed declaration does not make is not lent. The split is between two kinds of reader. A
reader that asks what a declaration cites, or who cites an ID, reads the lent citations; a
reader that judges a citation site's own hygiene never sees them ([§AR-scanner.4.6](../../architecture/AR-scanner.md#46-a-stubs-home-is-recorded-once-after-the-walk)). That is D8's
two halves, each in the readers it governs.

### 2.3 Where a finding lands

A finding about a lent citation is reported at the citation's line in the target, where the
scanned tree reports it: `src/x.rs:3: … FS must not cite GOAL …`. That is the author's "report
it there, as U01 does". A finding about the declaration itself — a count it fails, a chapter
count, `unused` — stays at the stub's line, where it is reported today. Moving those to the
target is [§DF-stubs-by-home-and-at-the-target.2.3](DF-stubs-by-home-and-at-the-target.md#23-a-stubbed-declaration-is-shown-at-its-target-from-0190)'s U06, which this change widens to cover them
([§FS-declarations.stubs.pairing.4](../../functional-spec/FS-declarations.md#stubspairing4-every-surface-shows-a-paired-declaration-at-its-target)), so neither change waits on the other.

### 2.4 What it newly raises warns until 0.19.0

The verdicts that newly pass go in the release that ships this, because removing an error breaks
no run: the false `missing-citation`, the false `chapter-cardinality` and the false `unused`.
The findings that newly fail warn until 0.19.0 ([§FS-rules.7.11](../../functional-spec/FS-rules.md#711-a-citation-or-chapter-read-through-a-stub-warns-until-0190)), with the clause `; a citation
or chapter a stub's target holds now counts, and this warning becomes an error in grund 0.19.0`:
a prohibition a lent citation breaks, an `at most N` or `exactly N` count the lent citations or
chapters now exceed while the same count without them holds, and a `[citations]` `must` on a
target that cites nothing, which the stub's empty record never owed. A count that fails without
them stays an error. `refs`, `list`'s `refs` count and the editor's count gain rows; they move no
exit status, so they owe no window.

The window ends at 0.19.0, with [§FS-rules.7.10](../../functional-spec/FS-rules.md#710-a-rule-declared-through-a-stub-warns-until-0190)'s, in the change that ends that one. If 0.18.0 is
cut before this change merges, the landing release is 0.19.0, so this window, its clause and
[§FS-rules.7.11](../../functional-spec/FS-rules.md#711-a-citation-or-chapter-read-through-a-stub-warns-until-0190) move to 0.20.0, and the flip is its own step under #576.

## 3. Rejected alternatives

- **Append the lent citations to the walked citations, flagged by origin.** About a dozen hygiene
  checkers would each have to remember to skip them, and the first that forgot would report a
  finding about the target file that D8 rules out, with nothing to notice it.
- **Move every finding to the target now.** A count or chapter count is a finding about the
  declaration, and where a declaration reached through a stub is shown is #650's to move in
  0.19.0; moving it here would make the two changes wait on each other.
- **Raise the new findings as errors at once.** A tree with a stub to an outside target that
  passes today would fail on upgrade with no window, which [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) and D8
  forbid for a fact-import change.
- **Keep minting an outside target's chapters only where something cites into them.** Then a
  chapter rule would count the chapters a citation happened to reach, an answer neither tree
  gives.

## 4. Consequences

Only a tree with a stub whose target is outside `[scan] include` moves. Its false errors go; a
prohibition its target breaks, a bound its target now exceeds and a `[citations]` `must` its
target fails appear as warnings, and become errors in 0.19.0. `refs` gains the target's rows. A
target the scan reads moves nothing, because nothing is lent. Rows U04, U07 and U08 leave
[§REQ-one-answer.differences.stubs](../../requirements/REQ-one-answer.md#differencesstubs-stubs), and U06 widens to every finding about the declaration that
still lands at the stub.

## release-note: Release note

- [§DF-stub-target-citations-and-chapters](DF-stub-target-citations-and-chapters.md#df-stub-target-citations-and-chapters-a-stubs-target-lends-the-stubbed-declaration-its-citations-and-chapters-and-what-that-newly-raises-warns-until-0190), [§FS-declarations.stubs.target.5](../../functional-spec/FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged), [§FS-rules.7.11](../../functional-spec/FS-rules.md#711-a-citation-or-chapter-read-through-a-stub-warns-until-0190), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **a stub's target outside the scan now lends the stubbed declaration its citations and chapters, so rules, `[citations]`, `refs` and `unused` read it as if it were scanned.** A rule or `[citations]` obligation the target's citations meet no longer fails at the stub, a chapter rule counts the target's chapters, and `grund refs` lists the target's citations. **Who this affects:** repositories with a stub to a declaration outside `[scan] include`. A prohibition the target's citations break, a count or chapter count they now exceed, and a `[citations]` `must` on a target that cites nothing are reported as warnings ending `; a citation or chapter a stub's target holds now counts, and this warning becomes an error in grund 0.19.0`, and become errors in 0.19.0. Nothing is reported about the target file itself, and a citation there that resolves to nothing still counts for nothing.
