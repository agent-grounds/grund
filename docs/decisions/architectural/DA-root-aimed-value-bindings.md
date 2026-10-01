# DA-root-aimed-value-bindings: a value binding may aim at the whole root, its components joined by one ASCII space

**Status:** Accepted
**Date:** 2026-10-01
**Supersedes:** The binding-grammar clause of [§DA-explicit-value-bindings.2](DA-explicit-value-bindings.md#2-decision) — that `grund` compares only the binding grammar of [§FS-values.3.1](../../functional-spec/FS-values.md#31-the-only-binding-grammar) as it then stood, one component coordinate per binding. Its authority model, its JSON runtime surface and its current-tree-only check stand, and so does that record.

## 1. Context

A value is often a quantity: [§FS-values.4](../../functional-spec/FS-values.md#4-exact-equality) calls a unit a successive component by convention, so a reference price is declared as an amount and a unit. A sentence mentions that price once, but the only checked way to write it was twice, one binding per component, and the shipped example did exactly that. Everyone less diligent bound the amount and left the unit in prose, where it looks cited and can drift. That is the failure [§GOAL-agent-grounding](../../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work) exists to prevent, and [§GOAL-friendliness-first](../../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible) names the mechanism: staying cited has to be the path of least resistance.

The refusal was deliberate. [§FS-values.3.1](../../functional-spec/FS-values.md#31-the-only-binding-grammar) was the only binding grammar, [§FS-values.3.1.1](../../functional-spec/FS-values.md#311-invalid-attempts-and-non-attempts) named a value root itself as a refused target, and [§FS-non-goals.2](../../functional-spec/FS-non-goals.md#2-spelling-grammar-prose-quality) compared one authored component with one numbered component. The preamble of [§FS-non-goals](../../functional-spec/FS-non-goals.md#fs-non-goals-what-grund-will-deliberately-not-do) requires a decision record to move a non-goal, and this is that record.

## 2. Decision

A binding's component coordinate becomes optional, in the one grammar [§FS-values.3.1](../../functional-spec/FS-values.md#31-the-only-binding-grammar) already defines, rather than in a second form beside it. With the coordinate nothing changes. Without it the binding is root-aimed: its cited path resolves to a value root of any of the three authorities, and its literal is split at every ASCII space and compared component by component, in declared order, each under the equality [§FS-values.4](../../functional-spec/FS-values.md#4-exact-equality) already gives it ([§FS-values.3.1.2](../../functional-spec/FS-values.md#312-a-binding-aimed-at-the-root)). A path is read as root-aimed when it resolves to a root and as component-aimed when its parent does and its final numeric segment names an immediate component; no path reads both ways over valid authority ([§FS-values.5.1](../../functional-spec/FS-values.md#51-resolve-before-comparison)).

A value any of whose components contains an ASCII space may not be root-bound, whatever its arity, and the refusal is `invalid-value-binding` at the binding, naming the component. It is a property of the root-aimed spelling, never of the value, which stays valid and stays bindable at a component.

A root-aimed mismatch reports the first unequal component by its own coordinate and site, in the canonical text; when the part count differs from the component count it names the root and the joined declaration instead ([§FS-values.5.2.2](../../functional-spec/FS-values.md#522-a-root-aimed-mismatch-names-the-first-unequal-component)). The grammar message keeps its bytes for every form that earns it today.

## 3. Why

The join must be a function of what it cites, because [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution) forbids a lookup two values can satisfy without a written rule. Every component is already nonempty and edge-unspaced ([§FS-values.2.1](../../functional-spec/FS-values.md#21-markdown-declarations)), so forbidding an internal ASCII space as well makes the join one-to-one: joining with one space gives exactly `N - 1` separators, which split back into exactly the components. The rule names U+0020 alone, because nothing else can be confused with the separator.

One grammar keeps one predicate. The set of forms `check` refuses and the set `fmt --cross-refs` protects are decided by the same function so that they cannot drift apart ([§FS-values.8](../../functional-spec/FS-values.md#8-formatting-stability)), and a second form would need a second predicate in both places. The heading of [§FS-values.3.1](../../functional-spec/FS-values.md#31-the-only-binding-grammar) also stays true and stays put, so the citations into it keep resolving.

Reporting the first unequal component keeps [§FS-values.5.2](../../functional-spec/FS-values.md#52-fixed-value-errors), [§FS-values.5.2.1](../../functional-spec/FS-values.md#521-a-mixed-kind-mismatch-names-each-sides-kind), [§FS-output-shapes.1.1](../../functional-spec/FS-output-shapes.md#11-value-mismatch) and [§FS-lsp.1.1.2](../../functional-spec/FS-lsp.md#112-findings-the-core-report-shapes) unchanged to the byte: one coordinate, one declaration site, one component span. A blanket space refusal keeps the rule an author holds to one sentence that does not change as components are added.

## 4. Compatibility

No passing tree gains a finding, and this is mechanical rather than asserted. Every root-aimed form was `invalid-value-binding` before this decision, so a tree that exited `0` carries none, and the trees that change are trees that already exited `1`. They move to `0`, or stay at `1` with a different finding at the same site.

Two observable things move, and [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered) says how each is governed. The bytes move at one kind of site: a root-aimed binding over a space-bearing value swaps the grammar message for the space refusal, with the same `code`, severity and exit status. The consumer affected is one matching that whole line, and its migration is the stable `code`. The verdict moves only towards passing: `check` exits `0` where a root-aimed binding now agrees, and `fmt --cross-refs --check` stops reporting the bare whole-value form it used to rewrite ([§FS-values.9.2](../../functional-spec/FS-values.md#92-the-one-whole-value-formatting-behavior-that-moved)).

[§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) does not fit, because its finding names versions and its fix is one command, and here the finding disappears and there is nothing to run. Nor is this a correction of a verdict another requirement forbade, because the refusal was correct when it shipped. What applies is [§GOAL-no-silent-breakage.2](../../goals.md#2-the-deprecation-path)'s standing demand that no verdict move with neither a window nor a record: this is the record, and the release notes name the move.

## 5. Consequences

- An amount and its unit are one checked literal, and the unit is checked as a consequence of citing the quantity rather than as a second act of diligence.
- The scanner records an optional coordinate on a binding, and the checker splits the one relation that classified a root path as a refusal.
- `fmt --cross-refs` rewrites fewer lines, never more: the bare whole-value form it used to rewrite is now a binding or a protected refusal.
- No unit, range, ordering or conversion semantics, no configurable separator, no new configuration key or inline marker, and no inference from prose: [§FS-values.9](../../functional-spec/FS-values.md#9-compatibility-and-explicit-exclusions) keeps every exclusion.

## 6. Alternatives considered

| Option | Why not |
|---|---|
| A second binding form beside the grammar | It needs a second refusal predicate and a second protected set, which is how `check` and `fmt` drift apart, and it falsifies the heading of [§FS-values.3.1](../../functional-spec/FS-values.md#31-the-only-binding-grammar). |
| Report the whole literal against the joined declaration, with every component's site | It rewrites the canonical mismatch text, makes `sites` plural, needs a new go-to-definition target, and leaves [§FS-values.5.2.1](../../functional-spec/FS-values.md#521-a-mixed-kind-mismatch-names-each-sides-kind)'s two-sided clause with no meaning. |
| Narrow the space refusal to values of two or more components | It makes a binding's validity depend on how many components the cited value has, which the author reading their own sentence cannot see. |
| Report a part-count disagreement as `invalid-value-binding` | An author who writes one part against two components made the content mistake `value-mismatch` exists to catch, and a grammar error would send them to the grammar. |
| Declare the quantity as one string component | It gives up numeric equality, so `1200.0 USD` no longer agrees with `1200 USD`, and every JSON consumer has to split the string. |
