# DF-rule-form-same-units: a refused rule's accepted form judges the units the typed sentence named, or none is offered, in place

**Date:** 2026-10-29
**Supersedes:** Three choices of the rule surfaces' accepted forms, each where the form drops chapter scope the author typed. From the second bullet of [§DF-rule-refusal-rewrites.2](DF-rule-refusal-rewrites.md#2-decision), the choice to answer `FS-login.*` and `FS-login.2` with `FS-login` or `Each FS`. From its third bullet, the choice to answer `The requirements.1 chapter of each FS` and `The requirements.* chapter of each FS` with `The requirements chapter of each FS`, and `FS-login..requirements` with `FS-login`. From [§DF-rule-after-enabling-rewrites.2](DF-rule-after-enabling-rewrites.md#2-decision), the choice to rebuild a named-sections-off subject by the same four steps where those steps drop a typed component, as `FS-login.Requirements` answered with `FS-login`. The rest of both decisions stands, and so do both records.

Amends [§FS-rule-refusals.terms](../../functional-spec/FS-rule-refusals.md#terms-terms), [§FS-rule-refusals.sentences](../../functional-spec/FS-rule-refusals.md#sentences-strict-refusals), [§FS-rule-refusals.sentences.named-sections](../../functional-spec/FS-rule-refusals.md#sentencesnamed-sections-a-subject-that-needs-named-sections-is-answered-with-one-they-make-valid), [§FS-rule-refusals.sentences.chapter-path](../../functional-spec/FS-rule-refusals.md#sentenceschapter-path-a-chapter-path-is-refused-for-the-component-that-failed), [§FS-rule-refusals.sentences.rewrite.recovered](../../functional-spec/FS-rule-refusals.md#sentencesrewriterecovered-only-what-can-be-recovered-is-supplied), [§FS-rule-refusals.sentences.rewrite.by-refusal](../../functional-spec/FS-rule-refusals.md#sentencesrewriteby-refusal-the-rewrite-each-refusal-makes), [§FS-rule-refusals.sentences.rewrite.fs-only](../../functional-spec/FS-rule-refusals.md#sentencesrewritefs-only-exact-rows-where-fs-is-the-only-kind) and [§FS-rule-refusals.selectors](../../functional-spec/FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector), and adds [§FS-rule-refusals.sentences.rewrite.same-units](../../functional-spec/FS-rule-refusals.md#sentencesrewritesame-units-a-form-judges-the-units-the-typed-sentence-named-or-none-is-offered). Serves [§GOAL-agent-grounding](../../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work), whose agent acts on the offer as printed. Reported as [grund#1072](https://github.com/agent-grounds/grund/issues/1072).

## status: Accepted

## 1. Context

A refused rule sentence ends in `accepted form:` and a sentence to paste back.
[§DF-rule-refusal-rewrites](DF-rule-refusal-rewrites.md#df-rule-refusal-rewrites-a-refused-rules-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced-in-place) made that sentence the typed one with the failed
part replaced, and promised that it is accepted when pasted back. It did not
promise that the sentence asks the same question, and on two paths it does
not (agent-grounds/grund#1072).

- A numbered chapter subject, `REQ-coupons.2.2 must be cited by at least one
  DESIGN.`, was offered `REQ-coupons must be cited by at least one DESIGN.`. A
  citation of any chapter of `REQ-coupons` satisfies the offer, so a design
  that cites `2.1` turns it green while `grund refs REQ-coupons.2.2` prints
  nothing. A wildcard, a name the section grammar refuses, and a name after an
  empty component are dropped the same way, to a parent chapter or the
  declaration.
- A chapter subject given a chapter count, `The unresolved-questions chapter of
  each RFC must have at most 1 open chapter.`, was offered `Each RFC must have
  at most 1 open chapter.`. That counts the `Open` chapters directly under each
  RFC and none under `unresolved-questions`, so an RFC with two open questions
  passes. With `at least one` the verdict is reversed: the offer flags the RFC
  that has open questions and passes the one that has none.

Both offers were documented rows, and the refusal gave no sign that they judge
other units. [§FS-rule-refusals.sentences.rewrite.recovered](../../functional-spec/FS-rule-refusals.md#sentencesrewriterecovered-only-what-can-be-recovered-is-supplied) already said "never
replace a chapter obligation with a whole-declaration obligation or a literal
with `Each KIND` merely to produce an accepted sentence", and agent-grounds/grund#1001
was this defect on a third path, a chapter typed in another word order, fixed
in place by keeping the chapter or withholding the form. The same point then
kept a longest-valid-prefix fallback that does exactly what it forbids, and
the presence row rewrote a chapter subject to its declaration with no
condition at all.

These lines are message text on both rule surfaces, `check --rule` and a
configured rule declaration's `invalid-rule` finding, which tools may match
([§FS-errors.3](../../functional-spec/FS-errors.md#3-message-text), [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered)). So changing them is a decision
rather than a fix.

## 2. Decision

Both rule surfaces offer an accepted form only where its subject selects the
units the typed subject named and its predicate counts in each of them what the
typed predicate counted ([§FS-rule-refusals.sentences.rewrite.same-units](../../functional-spec/FS-rule-refusals.md#sentencesrewritesame-units-a-form-judges-the-units-the-typed-sentence-named-or-none-is-offered)).
Where none does, the refusal is its reason alone, and no `known kinds:` line
follows, because what is missing is chapter scope and not a kind. That is the
shape `Each chapter of each FS` has had since agent-grounds/grund#1001. Two
refusals meet it:

- a refused chapter path that the selector's four steps can rebuild only by
  dropping a typed component: a numbered one, a wildcard, a name the configured
  section grammar refuses, as it refuses every name with named sections off,
  or a name after an empty component. An empty component at the end, as in
  `FS-login.requirements.`, names nothing, and with named sections on that form
  stays;
- a chapter subject in a presence rule, always, because no accepted sentence
  counts the chapters under a chapter.

`list --selector` keeps its longest-valid-prefix offer, byte for byte. A
selector judges nothing: it lists the nearest unit that holds what was typed,
and its breadcrumb says how to reach inside it. The other offers keep their
forms, because they keep the units: `every` repaired to `each`,
`FS-*.requirements` answered with `The requirements chapter of each FS`, the
reordered spellings of agent-grounds/grund#1001, and `Each */FS` answered with
`Each FS`, whose reason already names the namespace it drops.

The new text replaces the old in place, in one release, under the pre-release
licence of [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise).

- Every byte that goes is a form that judged other units than the sentence it
  answered. Followed, it either passed over the unit its author asked about or
  flagged another one. Nothing that worked can have depended on it: the
  sentence was refused before and stays refused.
- A consumer of the `invalid-rule` finding keeps its code, its path, line,
  severity and authority, the exit code, selection by `--only invalid-rule`,
  and the text through the reason. Only what follows the reason goes.
- `check --rule` has no code. A consumer of it keys on the exit code and on the
  sentence it passed, and this change moves neither.
- No verdict moves. The accepted and refused sentences, the exit codes and the
  finding code stay as they are, so this is not a correction under
  <§>REQ-backwards-compatibility.5 and needs none of that route's proof.

## 3. Consequences

- A consumer matching a refused rule's exact line reads the reason alone for a
  numbered or wildcard chapter path, a chapter name the section grammar refuses,
  a name after an empty component, and a chapter subject in a presence rule.
  One matching through the reason keeps matching.
- A tool that pastes back every `accepted form` it is offered gets nothing to
  paste for those sentences. That is the point: there is no accepted spelling
  of what they ask. `grund refs` answers whether a numbered chapter is cited,
  and `grund list --glob` lists the chapters under a chapter.
- The Python `check(rule=…)` and Node `check(root, { rule })` failure message is
  the stderr of `check --rule`, so it changes with it.
- `list --selector`, its reasons, offers, `known kinds:` line and breadcrumb do
  not move.
- Exit codes, stdout, finding codes, locations and the set of accepted
  sentences do not move, and no repository that passed starts failing, because
  every line this touches is already a failure.
- The worked examples that quoted these offers to explain what they changed now
  quote the reason alone.

## 4. Alternatives considered

| Approach | Why rejected |
|---|---|
| Keep the form and say what it changes, as `accepted form, which counts a citation of any chapter of REQ-coupons: …` | It still offers a sentence that judges other units, under a label a reader pastes from. The label becomes a second grammar to parse, and an agent that pastes after `accepted form` lands on the wrong rule all the same. |
| Withhold the form only where the fallback reaches the declaration or `Each KIND`, the two paths the report reproduces | `The requirements.1 chapter of each FS` answered with `The requirements chapter of each FS` widens one level less and judges other units all the same. Fixing the deepest copy of the rule and leaving the shallower one standing is the defect agent-grounds/grund#1001 already left behind once. |
| Withhold the selector's offer too | A selector lists units and judges none, so its nearest-unit offer selects a superset that a reader then narrows, and the breadcrumb says how. Taking it away would leave `list --selector` with nothing to paste for a refusal whose prefix is useful. |
| Offer `grund refs` or `grund list --glob` as the form | Neither is a rule sentence, and the `accepted form` label promises one that `check --rule` accepts. A command in that position is a guess at what the author will do with the answer. |
| Migrate over three releases, as [§FS-errors.3.7](../../functional-spec/FS-errors.md#37-the-rule-site-unknown-alias-wording-migration) did | A window lets exact-line consumers move to a stable code. The finding's code already exists and does not move, and `check --rule` has none, so a window gives them nothing new. It would also print a form that judges other units for two more releases. |
| Change `check --rule` only and keep the finding's bytes | One function renders both surfaces. Keeping the finding would need a second renderer whose only job is to print a sentence that judges other units, on the surface where following it costs a commit. |

## release-note: Release note

- [§FS-rule-refusals.sentences.rewrite.same-units](../../functional-spec/FS-rule-refusals.md#sentencesrewritesame-units-a-form-judges-the-units-the-typed-sentence-named-or-none-is-offered): **a refused rule offers an
  `accepted form:` only where that form judges the units your sentence named.**
  `grund check --rule` and a configured rule declaration's `invalid-rule`
  finding no longer answer a numbered or wildcard chapter path with its parent
  chapter or its declaration, nor a chapter subject given a chapter count with
  its declaration. `REQ-coupons.2.2 must be cited by at least one DESIGN.` was
  offered `REQ-coupons must be cited by at least one DESIGN.`, which any
  citation of any chapter of `REQ-coupons` satisfies, and
  `The unresolved-questions chapter of each RFC must have at most 1 open
  chapter.` was offered `Each RFC must have at most 1 open chapter.`, which
  counts other chapters. Both now print the reason alone, as
  `Each chapter of each FS` does. So do `FS-login.*`, `FS-login.2`, `FS.*`,
  `The requirements.1 chapter of each FS`, `The requirements.* chapter of each
  FS`, `FS-login..requirements`, and, with named sections off,
  `FS-login.Requirements`, `The Requirements chapter of each FS` and
  `The * chapter of each FS`. With named sections on, `FS-login.requirements.`
  keeps its form, and `grund list --selector` does not change. **Who this breaks:** a script
  matching the exact stderr of `check --rule`, the exact `message` of an
  `invalid-rule` finding, or the exact failure message of the Python
  `check(rule=…)` or Node `check(root, { rule })`, for such a sentence, and a
  tool that pastes back the form it was offered. Exit codes, stdout, the
  finding code and location, the reasons and the set of accepted sentences do
  not move. Closes [issue #1072](https://github.com/agent-grounds/grund/issues/1072).
