# DF-section-citation-counts-in-rules: a resolved citation to a numbered section counts in chapter rules

**Status:** Accepted
**Date:** 2026-10-05

## 1. Context

agent-grounds/grund#449 reported a chapter rule that failed over a chapter that visibly cited its object. With `[id] named_sections = true`, the `goal` chapter of `BENCH-001-arrival-model-validation` cited `GOAL-001-latency-first-then-utilization.4`; the citation resolved, and `[citations.BENCH] must = ["GOAL"]` accepted it, yet `The goal chapter of each BENCH must cite at least one GOAL.` reported `BENCH-001-arrival-model-validation.goal must cite GOAL`. Adding a citation of the parent declaration beside `.4` made the finding go away.

The ground already said the citation counts. [§FS-rules.3.2](../../functional-spec/FS-rules.md#32-outbound-citation-count) counts "every resolved physical citation site inside the subject unit whose target is in the object set", and [§FS-rules.5.1](../../functional-spec/FS-rules.md#51-facts-and-identity) withheld a `cites` fact only from unresolved or ambiguous citations. No spec, decision, architecture or guide said that only declaration targets count. The fact producer disagreed in one place: it builds no node for a section with a numbered component, because numbered headings are not rule units ([§FS-rules.2](../../functional-spec/FS-rules.md#2-subject-selectors)), and it then dropped every citation whose section had no node, calling it an unresolved coordinate. A resolved `.4` is not one.

## 2. Decision

### 2.1 Count the citation, rather than explain why it is ignored

A resolved citation whose section is not a rule unit is a `cites` fact to its nearest enclosing unit — the nearest named ancestor chapter, or else the declaration — and the source side resolves the same way ([§FS-rules.5.1](../../functional-spec/FS-rules.md#51-facts-and-identity)). A citation to a numbered section in a unit's own body is therefore a citation to that unit for inbound counts too ([§FS-rules.3.4](../../functional-spec/FS-rules.md#34-inbound-citation-count-and-prohibition)). Every family reads the fact through the relations it already reads, so no relation is added and no family's clause changes.

### 2.2 Why not the issue's own remedy

The issue asked for the opposite: keep the citation uncounted, and have the `missing-citation` message ([§FS-rules.7.3](../../functional-spec/FS-rules.md#73-outbound-citation-cardinality)) and the managed guide ([§FS-rules.9](../../functional-spec/FS-rules.md#9-managed-guidance-and-editor-parity)) say that only a direct declaration target counts. That was rejected for three reasons.

- It writes into the specification a hole that [§FS-rules.3.2](../../functional-spec/FS-rules.md#32-outbound-citation-count) and [§FS-rules.5.1](../../functional-spec/FS-rules.md#51-facts-and-identity) rule out, so the message would explain a defect rather than a semantics.
- It cannot reach the same gap's worse twin. Under `The goal chapter of each BENCH must not cite any GOAL.` the same `.4` citation passed, because the prohibition never saw it; no finding was emitted, so there was no message to reword.
- It would change the managed block, and so its version, for every repository that has rules.

The author of #449 approved this remedy over the one the issue proposed.

### 2.3 The new failures take the deprecation path

Counting more sites makes some verdicts pass and others fail. The ones that newly pass — `at least` counts, outbound, per-target and inbound — land at once, because nothing that passed before breaks. The ones that newly fail — a prohibition, and an `at most N` or `exactly N` count of any family — are verdict changes under [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered), so they take [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)'s route: one release in which such a finding is a warning naming `grund 0.18.0` in its own bytes, then an error ([§FS-rules.7.8](../../functional-spec/FS-rules.md#78-a-newly-counted-section-citation-warns-until-0180)). The warnings ship in `grund 0.17.0`, the minor a verdict change requires, which is the `0.15.0` → `0.16.0` shape of [§DF-chapter-rule-reaches-every-declaration.2.5](DF-chapter-rule-reaches-every-declaration.md#25-a-warning-for-one-release-window-not-a-soft-must).

The channel is the one that record used and then removed: a temporary two-field return from the engine's required-level evaluation, errors and ramp warnings beside them, and a side-metadata mark on each site counted through its nearest unit. The mark is metadata, not a relation, so the logical facts are what [§FS-rules.5.1](../../functional-spec/FS-rules.md#51-facts-and-identity) says they are; the mark and the second field are deleted when the ramp closes.

### 2.4 Why not the correction route

The correction route, section 5 of [§REQ-backwards-compatibility](../../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly), would let the new failures land at once, but only where the old verdict violated a prior hard requirement at a cited numbered section. The nearest candidate is [§REQ-no-missed-citation](../../requirements/REQ-no-missed-citation.md#req-no-missed-citation-every-citation-the-run-reads-is-checked), and it is about the scanner recognising and validating citations — which this one was. Stretching it to cover how rules count a recognised citation is the invented prohibition that route's fifth condition refuses, so the route is considered and not used: this record claims no correction.

## 3. Alternatives considered

**Count numbered sections as rule units.** It would give `GOAL-x.4` a node of its own and make every family count it directly, but a numbered heading's coordinate moves when headings are reordered, which is why [§FS-rules.2](../../functional-spec/FS-rules.md#2-subject-selectors) refuses it as a subject. Attaching the site to its nearest stable unit counts it without making the unstable coordinate addressable.

**Ship the new failures as errors.** [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) forbids it, and [§DF-section-citation-counts-in-rules.2.4](DF-section-citation-counts-in-rules.md#24-why-not-the-correction-route) above is why the correction route does not open.

## 4. Consequences

A tree whose `at least` chapter rule failed only because a section citation was ignored now passes, and the workaround — a parent citation beside the section citation, written only to satisfy a rule — is no longer needed. A tree whose prohibition or upper bound a section citation silently evaded keeps exiting `0`, but prints a warning per finding instead of `success`, and exits `1` from `0.18.0`. Repositories with `named_sections` off are the widest seam: every section there is numbered, so every section citation inside a rule's subject starts counting. A repository with no `rules = true` kind is byte-for-byte unchanged. The promotion is a follow-up issue opened when this change merges.

## release-note: Release note

- [§FS-rules.5.1](../../functional-spec/FS-rules.md#51-facts-and-identity), [§FS-rules.3.4](../../functional-spec/FS-rules.md#34-inbound-citation-count-and-prohibition), [§FS-rules.7.8](../../functional-spec/FS-rules.md#78-a-newly-counted-section-citation-warns-until-0180), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **a resolved citation to a numbered section now counts in chapter rules.** `GOAL-x.4` counts as a citation of `GOAL-x`, and `GOAL-x.outcome.2` as one of the `outcome` chapter, in every rule family, as `[citations.KIND]` already counted them. A chapter rule that failed only because such a citation was ignored now passes. A finding the newly counted citation alone produces — a `must not cite` prohibition, or an `at most N` or `exactly N` count it pushes out of bounds — is a warning in this release, carrying the code it will carry as an error and ending `; a citation to a numbered section now counts, and this warning becomes an error in grund 0.18.0`; the exit code does not move until then. **Who this breaks, from 0.18.0:** a repository whose prohibition or upper-bound rule a section citation silently evaded, most widely one with `named_sections` off. The escapes are to remove or move the citation, loosen the rule's count, or pass `--ignore` on the finding's code. (agent-grounds/grund#449)
