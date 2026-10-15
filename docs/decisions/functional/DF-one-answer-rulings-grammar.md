# DF-one-answer-rulings-grammar: one grammar keeps the documented refusal bytes and order, and one kind universe keeps each command's vocabulary

**Date:** 2026-10-11

## status: Accepted

## context: Context

[Issue #576](https://github.com/agent-grounds/grund/issues/576) unifies how every command reads
a tree under [§AR-system.principle](../../architecture/README.md#principle-one-fact-one-point-one-implementation), the rule [§DA-one-fact-one-point-one-implementation](../architectural/DA-one-fact-one-point-one-implementation.md#da-one-fact-one-point-one-implementation-every-fact-grund-decides-has-one-defining-point-one-implementation-and-reads-of-it-everywhere-else) records,
and the promise [§REQ-one-answer](../../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree) makes to users. Its author ruled D1 to D21 before any copy is
unified, and a cross-family agora settled a compatibility route for each ruling (entries
V-A02 to V-A26 of its verdict). This record holds the rulings of the step family *one
grammar*: one coordinate lexer, one kind universe per run, and one rule grammar whose
refusals are data. It also holds the two token corrections V-A26 assigns to that step, G01
and G02, and the ruling on which new specifications the work creates.

## decision: Decision

The rulings stand as follows. Each one that moves a verdict names its route: the warning
release of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path), or the one-release correction of
<§>REQ-backwards-compatibility.5 where an existing requirement already forbade the old
verdict, and then the record names that requirement.

- **D10** (V-A12). Refusal bytes: the documented refusal rows stay golden, and generated
  text is used elsewhere. Any change to the wording of a message users already see takes the
  warning release, [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path), never a correction; an append that keeps
  the old text as a verbatim prefix moves nothing, as [§FS-errors.3](../../functional-spec/FS-errors.md#3-message-text) exempts it.
- **D11** (V-A13, amended as overtaken). The collision D11 solved is gone: #562 and #571
  merged as [§FS-rules.3.5.4](../../functional-spec/FS-rules.md#354-an-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced) and [§FS-rules.3.5.5](../../functional-spec/FS-rules.md#355-any-clauses-joined-by-and-are-refused-as-a-conjunction), and the grammar step moves them with
  pointers as [§REQ-spec-section-names.permanence](../../requirements/REQ-spec-section-names.md#permanence-a-name-is-permanent) requires. No verdict moves: no route.
- **D18** (V-A21, amended). One kind derivation and one refusal renderer, aware of the
  project and the command. Each command keeps its documented project vocabulary, as
  [§FS-rules.3.5.2](../../functional-spec/FS-rules.md#352-a-subject-that-needs-named-sections-is-answered-with-one-they-make-valid) and [§FS-list.1.1](../../functional-spec/FS-list.md#11---kind) fix them; golden quoting stays until its own migration;
  `alias/KIND` targets use the alias's kinds. Every part takes the warning release,
  [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path), since a config target is not a citation.
- **D18**, the conditional part (V-A21). Reading named sections under the owning project's
  gate is a candidate for <§>REQ-backwards-compatibility.5 under [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution) if
  the grammar step reproduces a citation it resolved to the wrong place.
- **D20** (V-A23). Refusal precedence is one ordered table that keeps `main`'s documented
  order: terminal period, then the fixed-word and phase-1 subject refusals, then conjunction,
  then modality ([§FS-rules.3.5.5](../../functional-spec/FS-rules.md#355-any-clauses-joined-by-and-are-refused-as-a-conjunction)). Any change to which refusal comes first takes the warning
  release, [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path).
- **G01** (V-A26). A citation token that stops too early, recording an edge to a shorter ID
  that is a prefix of the written one, is a candidate for <§>REQ-backwards-compatibility.5
  under [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution), which never substitutes a near miss.
- **G02** (V-A26). A citation of a loaded alias's ID that the lexer silently drops is a
  candidate for <§>REQ-backwards-compatibility.5 under [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded). G01 and
  G02 are wrong in every command alike, so they are correctness defects, not differences
  [§REQ-one-answer](../../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree) lists.
- **G06** (#600). Fix: `list --selector "each FS"` is wrong and the rule is right.
  [§FS-rule-refusals.selectors](../../functional-spec/FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector) already gives a refused selector the reason
  [§FS-rule-refusals.sentences](../../functional-spec/FS-rule-refusals.md#sentences-strict-refusals) gives the same subject, so the selector owes the rule's
  `fixed word "Each" is case-sensitive`. The route is D10's: the selector appends the rule's
  reason and keeps its literal-subject text as a verbatim prefix, which moves nothing. The
  row of [§REQ-one-answer.differences.identity](../../requirements/REQ-one-answer.md#differencesidentity-identity) closes with that append.
- **G09** (#600). Fix: `[citations]` is right and the three other surfaces are wrong.
  `code` is the homeless kind ([§FS-config.3.9.2](../../functional-spec/FS-config.md#392-the-homeless-kind)), and
  [§REQ-one-answer.identity.kinds](../../requirements/REQ-one-answer.md#identitykinds-every-command-reads-one-kind-verdict) refuses a non-citable kind for that reason everywhere,
  so a rule's subject and object, `list --kind` and `list --selector` owe `code` the
  non-citable reason, not `unknown`. The route is the row of
  [§DF-one-kind-verdict.decision.routes](DF-one-kind-verdict.md#decisionroutes-which-route-each-move-takes) for a non-citable kind on a selector or in a rule: the
  warning release, [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path), with the legacy reason kept and the new
  one appended now and the new reason alone in `0.19.0`. Both rulings are recorded here,
  beside G01 and G02, because the record whose step listed G09 has already published its
  notice.
- **D19** (V-A22). The new specifications are REQ-one-answer, FS-citations, FS-languages,
  FS-scan (the run only) and FS-rule-refusals, with no FS-kinds; stubs and structure become
  chapters of [§FS-declarations](../../functional-spec/FS-declarations.md#fs-declarations-a-declaration-is-addressable-once-from-one-allowed-place-and-holds-nothing-that-is-neither-a-coordinate-nor-a-finding). Each new FS carries exactly one `## terms: Terms` chapter as
  [§RULE-terms](../../rules/RULE-terms.md#rule-terms-each-fs-must-have-exactly-one-terms-chapter) requires, every new declaration uses named chapters as
  [§REQ-spec-section-names.shape](../../requirements/REQ-spec-section-names.md#shape-where-a-number-is-allowed) requires, and the general refusal shape goes to the free slot
  FS-errors.2.5. A ruling about structure; it moves no verdict and has no route.

A correction route named here is a candidate, not a pre-approved record. The correcting
step's own accepted decision record must prove all five conditions of
<§>REQ-backwards-compatibility.5 and carry the release note [§FS-distribution.4.6.4](../../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions) publishes.
Changed finding text stays on [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) even inside a correcting
release, and [§REQ-one-answer](../../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree) is never the requirement a correction rests on (V-A25).

## alternatives: Alternatives refused

- **Regenerate the documented refusals from the new table.** Users grep those bytes; a
  generated wording, or a new first refusal, would change them with no warning.
- **One accepted kind set for every command.** At a workspace root `list --kind` accepts
  every loaded project's kinds while a root rule's subject accepts only the root's; flattening
  them would break one of the two.
- **List G01 and G02 as one-answer differences.** Every command agrees on them; they break
  existing requirements, which is what a correction is for.

## consequences: Consequences

This record publishes no notice and moves no verdict: it carries no `release-note`, and each
ruling's own step writes the decision record that does. That step cites this record for the
ruling and its route rather than deciding the route again.

## why: Why

[§GOAL-no-silent-breakage](../../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path) asks that anything user-visible either stays compatible or crosses
a named window. Writing each ruling's window down before the change that moves it is what
keeps that change from choosing its own.
