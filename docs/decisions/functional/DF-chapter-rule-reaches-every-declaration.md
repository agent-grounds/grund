# DF-chapter-rule-reaches-every-declaration: a chapter-scoped citation rule reports the declaration that has no such chapter

**Status:** Accepted
**Date:** 2026-10-01

## 1. Context

A rule whose subject is `The <NAME> chapter of each <KIND>` selected only the declarations that had that chapter ([§FS-rules.2](../../functional-spec/FS-rules.md#2-subject-selectors)). A declaration missing it contributed no unit, so the rule was evaluated over a smaller set and said nothing about it: deleting the chapter was the one edit the rule could not see. The instruction `grund init` renders into `AGENTS.md` claims the whole kind, and the sentence `grund check` enforced did not.

This was specified rather than accidental, and deliberate: agent-grounds/grund#209 wrote presence and citation as two separate derived predicates on purpose. agent-grounds/grund#355 reported it as a `usability` defect and was closed by documenting the gap — the spec and all three copies of the `chapter-rules` writing block were taught that a quantified chapter subject selects only the chapters that exist, and that a chapter-scoped citation rule held for the whole kind only when a chapter-presence rule stood beside it. That was the right first move for a report about wording, and it left the pairing as something an author must remember rather than something the tool checks.

Two accepted records already argue the other way from inside the ground. [§DF-non-citable-kinds.2.5](DF-non-citable-kinds.md#25-obligations-get-a-per-file-unit-and-grounding-follows-the-home) refused a rule that "would have yielded zero units and passed vacuously", and [§DF-unwalked-kind-home.2.3](DF-unwalked-kind-home.md#23-no-citation-rules-on-it) generalised that refusal to any config carrying "an instruction the checker never enforces". A chapter-scoped citation rule over a declaration with no such chapter is the same emptiness one level down.

## 2. Decision

### 2.1 Contributing no unit is not being outside the rule

A quantified chapter subject still selects only the chapters that exist, and a declaration of the kind that has none is now reported rather than passed over ([§FS-rules.checks.unreached-declaration](../../functional-spec/FS-rules.md#checksunreached-declaration-unreached-declaration)). The reach of a sentence is what its author reads it as: `The requirements chapter of each FS must cite at least one REQ.` names every FS, so every FS either satisfies it or hears about it. #209's separation is reversed for the quantified chapter subject and for nothing else.

### 2.2 Three positive families, and the reason the other two are not a double standard

The finding fires for outbound count, per-target coverage and inbound count, and for neither of the remaining families ([§FS-rules.5.2](../../functional-spec/FS-rules.md#52-family-clauses)). Chapter presence cannot reach the premise at all: its subject is a kind or an exact declaration and never a chapter, so its selection is already the declarations of the kind and nothing is absent from it.

Prohibition admits a chapter subject and still stays silent, and the reason matters more than the outcome. It is **not** that a prohibition over an empty selection is vacuously true — that argument would excuse the other three equally. It is that `forbidden_site` ranges over `cites` facts, and a citation site inside the chapter body is deleted along with that body: there is no way to remove the heading and keep the <§>AR-one that was inside it, so for `must not cite` the forbidden site genuinely no longer exists. The rule is not blind to the edit; the edit removed exactly what the rule forbade.

### 2.3 A code of its own, located at the declaration

`unreached-declaration` rather than a reuse of `chapter-cardinality` ([§FS-rules.7.2](../../functional-spec/FS-rules.md#72-chapter-cardinality)), which is a count outside an interval under the presence family's authority; this is a citation rule reporting that its subject is absent. It cannot reuse the `cites <target-set> 0 times` line of [§FS-rules.7.3](../../functional-spec/FS-rules.md#73-outbound-citation-cardinality) either, which would name a unit that does not exist. It is located at the subject declaration's title line, because the chapter title that would otherwise anchor it is the thing that is missing.

### 2.4 One finding per semantic group, and both lines print

The finding is authored by a semantic rule group like every other rule finding, so [§FS-rules.6](../../functional-spec/FS-rules.md#6-semantic-deduplication) applies unchanged: two byte-identical rules yield one line naming both origins, and two rules that mean different things yield one line each. Where a chapter-presence rule stands beside the citation rule — which is what the documentation of #355 told authors to do — **both** lines print, each naming its own rule, and nothing is suppressed. Suppressing one rule's finding because another rule happened to fire would be a cross-rule interaction with no precedent in [§FS-rules](../../functional-spec/FS-rules.md#fs-rules-grounded-declarations-state-and-enforce-chapter-rules). The cost is visible in the published example at `examples/rules/`, where one missing chapter yields two lines.

The alternative — collapse every absence about one declaration into a single line naming every rule that quantified over that chapter — needs a grouping key `authority` does not have, whose contract is the origins of *one* semantic constraint, and it produces a message whose subject is a list rather than the one rule the reader has to fix.

### 2.5 A warning for one release window, not a soft `must`

[§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered) governs a new finding as a verdict change even where the exit code does not move, because a warning stands in place of the `success` marker. Route 2 is the only route open to it, and route 2 is a two-release ramp: this change ships the warning, naming `grund 0.16.0` in its own bytes, and the promotion is a follow-up issue blocked by this one plus [§RM-unreached-declaration-error](../../roadmap.md#rm-unreached-declaration-error-make-the-unreached-declaration-warning-an-error-in-0160). So a tree that exits `0` today still exits `0` when this ships; it only says so.

The channel that carries it is a two-field return from the engine's required-level evaluation — the errors it has always produced, and the absence warnings beside them — rather than a `severity` field on the diagnostic type. A severity field is a permanent third channel for every rule finding, it touches every construction site of a type the whole checker uses, and it would invite a general warning level for rules that nothing asked for and that [§FS-rules.7](../../functional-spec/FS-rules.md#7-findings-and-channels) would then have to define. The two-field form collapses back to one `Vec` when the ramp closes, so no clause survives promising authors a soft `must` ([§FS-rules.7.7](../../functional-spec/FS-rules.md#77-one-required-level-finding-is-a-warning-until-0160)).

At the recommended level the absence follows the rule's own level: a suggestion, immediately, with no ramp clause, because a suggestion never moves the exit status at any release.

### 2.6 The two spellings stay two kinds of finding

Deleting the chapter already made the exact spelling `FS-login.requirements must cite at least one REQ.` an `invalid-rule`. That stays untouched, and the inconsistency is resolved by saying why rather than by making them agree: an exact literal that does not resolve is a defect in the sentence, reported where the sentence is written; a quantified subject that selects nothing for a declaration is a gap in coverage, reported where the declaration is ([§FS-rules.2](../../functional-spec/FS-rules.md#2-subject-selectors)). Where both spellings stand over the same absent chapter, both fire.

## 3. Alternatives considered

**Compute the absence in the subject selector.** The selector is shared by all five families, and prohibition must stay silent, so folding it in there would be wrong for one of them and would hide the reason the other four differ.

**Suppress the citation rule's line where a presence rule covers the same `(kind, chapter)` pair.** Cheaper output for a repository that already did the right thing, at the price of a cross-rule interaction nothing else in the feature has: a finding would then depend on what some other sentence elsewhere in the tree happens to say.

**A warning that names no release and never expires.** [§FS-config.1.2.2](../../functional-spec/FS-config.md#122-why-no-deadline-is-owed) permits one where nothing will actually break. It is not this: the instruction and the check would still disagree about which declarations the rule covers, and the author would be told so forever rather than once.

**Ship the error directly.** [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) forbids it, and route 5 — the one route that could waive a window — forecloses itself by name for ordinary policy tightening.

## 4. Consequences

A repository with `rules = true` on a kind, at least one chapter-scoped citation rule at `must` level, and at least one declaration of that kind lacking the chapter now sees a warning per rule group per declaration. The exit code does not move; the bytes do, and a pipeline that greps for `success` rather than reading the exit code sees a line where it saw none. `--ignore unreached-declaration` is the escape for anyone who needs the window. A repository with no `rules = true` kind is byte-for-byte unchanged, which includes this one.

`grund list --selector` is unchanged, which keeps the exclusion #355 accepted; the guide's sentence about the declarations it omits is reworded, because they are no longer the ones a chapter rule says nothing about.
