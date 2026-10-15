# DF-section-lock-relative-fingerprint: a locked section's nested headings are fingerprinted relative to it, so a renumber is still caught

**Date:** 2026-10-13

## status: Accepted

## context: Context

The section lock records a fingerprint of the text a cited section held, and reports a move when that fingerprint turns up at another address of the declaration ([§FS-section-lock.checks.section-address-changed](../../functional-spec/FS-section-lock.md#checkssection-address-changed-a-cited-sections-text-sits-at-another-address)). The text is what `show` prints below the heading, nested headings included ([§FS-section-lock.holds.fingerprint](../../functional-spec/FS-section-lock.md#holdsfingerprint-the-fingerprint)). A complete-path nested heading spells its ancestors' path, so renumbering an ancestor rewrites it. The agora on [issue #662](https://github.com/agent-grounds/grund/issues/662) ran this on main: `FS-x.1.1`, a relative `### .1. Goals` under `## 1. Overview`, holds `#### 1.1.1. Latency` and is cited and locked. Chapter `1` becomes `2`, which forces `#### 2.1.1. Latency`, and a new chapter `1` with its own `### .1. Goals` is inserted. Nobody edited a word of prose, yet the recorded fingerprint is found nowhere, so the lock reads the move as an edit in place: `check` prints `success`, `fmt --check` lists `section lock: refresh FS-x.1.1`, and after `fmt --write` the citation of `FS-x.1.1` reads the new chapter's goals. The same tree with `#### .1. Latency` is caught.

Issue #662 lifts the reservation on a name under a number (`1.goals`) on the condition that the lock covers every case the lift opens ([§DF-number-name-sections.decision.lock](DF-number-name-sections.md#decisionlock-the-lock-is-what-answers-178)), and this gap is the one case the agora found where a renumber alone, with no edit and no lost baseline, gets past the lock. It exists today for relative sections, so it is decided on its own and lands first.

## decision: Decision

Before a section is hashed, each line the section map records as the heading of a section nested in it is written with its path relative to the section being fingerprinted, keeping the `#` run, the punctuation after the path and the title ([§FS-section-lock.holds.fingerprint](../../functional-spec/FS-section-lock.md#holdsfingerprint-the-fingerprint)). Both spellings of a nested heading then hash alike, and a renumber leaves the fingerprint of every section below it unchanged, so the move is reported.

### decision.one-place: Computed in one place

The rewrite is applied in the one place a fingerprint is computed, for the recorded entry and for every section it is compared with, so no comparison can mix the two forms ([§AR-system.principle](../../architecture/README.md#principle-one-fact-one-point-one-implementation)).

### decision.map: Matched by the section map

A line is rewritten only when the section map records it as a heading of a nested section, matched by position. A heading-shaped line in a fence or in prose, and a dotted line the grammar refuses, are hashed as written, so the rewrite never reads a line the scanner did not.

## rejected: Rejected alternatives

- **Forbid complete-path headings under a locked section.** It closes the gap by refusing the form [issue #662](https://github.com/agent-grounds/grund/issues/662) asks for, `#### 1.goals.latency:` under `### 1.goals:`.
- **Normalize citation text as well.** A section that cites its own subsections changes when a move updates those citations, and normalizing the citations would also hide a self-citation that genuinely changed its target. It stays an instance of the edit-and-move limit ([§FS-section-lock.limits.edit-and-move](../../functional-spec/FS-section-lock.md#limitsedit-and-move-a-section-moved-and-edited-in-one-change)).
- **A second key beside the fingerprint, such as the title.** [§FS-section-lock.limits.edit-and-move](../../functional-spec/FS-section-lock.md#limitsedit-and-move-a-section-moved-and-edited-in-one-change) already gives the reason: it reports a section retitled and edited in place whenever another section carries its old title.

## release-note: Release note

- [§FS-section-lock.holds.fingerprint](../../functional-spec/FS-section-lock.md#holdsfingerprint-the-fingerprint), [§FS-section-lock.limits.baseline](../../functional-spec/FS-section-lock.md#limitsbaseline-a-baseline-the-lock-never-saw): **a locked section's nested headings are fingerprinted relative to it, so a renumber that rewrites them is still reported as a move.** This concerns only locks written by a development build of grund since the section lock arrived; no released grund wrote one. Such a lock may list `section lock: refresh` rows under `grund fmt --check` for sections that hold nested headings. On a tree that passes `check` and `fmt --check` with the old binary, the upgrade changes no `check` verdict. A move the old binary reported and nobody resolved may no longer be reported, which is why the preflight comes first. So before upgrading, get `grund check` and `grund fmt --check` passing with the old binary, resolving any move it reports; then upgrade without renumbering anything, run `grund fmt --write` once, and review and commit the lock diff. That re-records the current texts; it does not recover a target lost earlier ([§FS-section-lock.limits.baseline](../../functional-spec/FS-section-lock.md#limitsbaseline-a-baseline-the-lock-never-saw)).
