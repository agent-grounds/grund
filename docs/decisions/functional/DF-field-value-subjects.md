# DF-field-value-subjects: A rule subject may select on a scalar field's value with one whose clause

**Date:** 2026-10-27

## status: Accepted

## 1. Context

A rule subject could select every declaration of a kind, one ID, or a named chapter
([§FS-rules.2](../../functional-spec/FS-rules.md#2-subject-selectors)). It could not select the declarations of a kind whose scalar field holds a
given value ([§FS-fields.content.one-of](../../functional-spec/FS-fields.md#contentone-of-a-scalars-value-is-its-display-name)), so a coverage rule that depends on what a
declaration says about itself could not be stated
([#1047](https://github.com/agent-grounds/grund/issues/1047), reproduced on `main` at
3c2b87875e). With `SWR.verification` admitting `Test`, `Analysis`, `Inspection` and
`Demonstration`, the rule a requirements project means,
`Each SWR whose verification is Test must be cited by at least one TC.`, was refused as
`unknown kind "SWR whose verification is Test"`, and `Each SWR must be cited by at least
one TC.` failed every requirement verified by inspection. The workarounds were to split the
kind by value, which writes the method twice and renames a requirement whenever its method
changes, or to leave the rule to review.

[§DF-chapter-rules.2](DF-chapter-rules.md#2-decision) deferred "broader selectors" rather than rejecting them, and
[§FS-rules.requirements.addition](../../functional-spec/FS-rules.md#requirementsaddition-the-language-grows-by-addition--directional) says what an addition must keep. This record is that
addition, for one selector.

## 2. Decision

### 2.1 One clause, on the subject

A quantified subject may end in one `whose FIELD is VALUE` clause: `Each KIND whose FIELD
is VALUE`, and `The NAME chapter of each KIND whose FIELD is VALUE`
([§FS-rules.2.2](../../functional-spec/FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)). Every family that takes the subject takes it with the clause. `FIELD`
is read as a whole section path, as a chapter's `NAME` is, and must name a scalar field of
the kind; `VALUE` must equal one of its `one_of` entries byte for byte. The declarations
whose field holds another value, or that lack the field, are outside the rule: they
contribute no unit, no family reports them, and none is unreached. The same parser serves
`check --rule`, declared rules and `list --selector`, so the three select the same units.

### 2.2 Not the exception phrase

[§FS-rules.12](../../functional-spec/FS-rules.md#12-deliberate-phase-1-absences) lists an exception phrase among the deliberate absences. An exception, as
#209 [§DF-field-value-subjects.3](DF-field-value-subjects.md#3-rejected-alternatives) defines it, carves a place or a site out of a subject and excuses the members it
covers: `except those in vendor/`, `grund:allow`. A whose clause excuses no member of the
subject it states. It selects positively, by a value grund already checks, and every
declaration it selects is held to the rule in full. A declaration it leaves out is not
excused from anything either: its field holds another valid value, or is absent where the
field allows absence, or is reported on its own as `missing-field` or
`invalid-field-value` ([§FS-rules.requirements.unseen](../../functional-spec/FS-rules.md#requirementsunseen-nothing-a-rule-governs-passes-unseen--realized-one-case-deferred)).

### 2.3 Targets stay out of scope

A whose clause narrows only a rule's subject. An object kind followed by one, as in
`Each AN must cite at least one SWR whose verification is Test.`, stays refused, and its
reason gains ` — a whose clause narrows only a rule's subject`
([§FS-rule-refusals.sentences.whose](../../functional-spec/FS-rule-refusals.md#sentenceswhose-a-whose-clause-is-refused-for-the-part-that-failed)). A filtered target set would change what an
object counts, and that is a change to every citation family's meaning, owed to its own
change. Admitting it later remains an addition, because every such sentence is refused
today.

### 2.4 Refused for the part that failed, through a window

A clause the grammar does not accept is refused for the clause, field or value that
failed, never as an unknown kind, in the order
[§FS-rule-refusals.sentences.whose](../../functional-spec/FS-rule-refusals.md#sentenceswhose-a-whose-clause-is-refused-for-the-part-that-failed) fixes. Those reasons replace bytes users
already see, so D10 of [§DF-one-answer-rulings-grammar](DF-one-answer-rulings-grammar.md#df-one-answer-rulings-grammar-one-grammar-keeps-the-documented-refusal-bytes-and-order-and-one-kind-universe-keeps-each-commands-vocabulary) sends them through the warning
release of [§FS-errors.3.10](../../functional-spec/FS-errors.md#310-the-whose-clause-wording-migration): `0.21.0` keeps the old reason as a prefix, and
`0.22.0` drops it. The same holds for any later refusal of a sentence whose clause is
accepted, such as its quantifier or an object kind, because that sentence too was an unknown
kind before. The exact-ID and object refusals only append, so they need no window.

## 3. Rejected alternatives

- **`FIELD` as a single handle.** It fits today, because a field is a direct section, but a
  front-matter key, which #1048 holds as `front-matter.<key>`, would then need a second
  spelling. Read as a path, the key becomes reachable by addition; `whose front-matter.<key>
  is …` is owed to #1042, because a key's value is its text rather than a display name and
  needs its own reader.
- **`is A or B`.** A filter decides per declaration, so two rules, one per value, select
  disjoint declarations and report exactly what one `or` rule would. `or` would buy brevity
  at the cost of a second meaning for ` or ` in a sentence whose object already uses it, and
  a tie-break for values that contain ` or `. The refusal offers the two rules instead.
- **Matching values against the config before the modality split.** That would move the
  split of [§FS-rules.3.6](../../functional-spec/FS-rules.md#36-where-a-sentences-subject-ends) for one clause, and a rule and a selector must split alike. A value
  the split cuts is refused, naming the value, as a chapter named `must` is.
- **A `whose` form of the `KIND.NAME` shorthand.** A selector already accepts both full
  spellings, and the grammar refuses a second spelling of one meaning
  ([§DF-chapter-rules.2](DF-chapter-rules.md#2-decision)).
- **Folding a filtered rule into a `[citations]` direction.** A direction is kind-wide and a
  filtered subject is not, so both report ([§FS-rules.6](../../functional-spec/FS-rules.md#6-semantic-deduplication)).
- **Replacing the refusal text in place.** The earlier refusal-rewrite records did so under
  the pre-release licence; D10 is the later ruling.

## 4. Consequences

- A repository that writes no whose clause sees no change. One that declares a whose rule,
  an `invalid-rule` error before, now has it evaluated
  ([§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered): no passing tree fails, and no accepted sentence
  changes meaning).
- The rule facts gain no relation and no producer changes: the filter is written over
  `contains` and `chapter` ([§FS-rules.5.2](../../functional-spec/FS-rules.md#52-family-clauses)), and the field's value is read by the one
  function `invalid-field-value` reads it with.
- "Applies to nothing" stays keyed on the kind ([§FS-check.2.2.1.4](../../functional-spec/FS-check.md#2214-a-rules-obligation)): a clause that
  selects nothing yet describes the project's state, not a broken rule.

## release-note: Release note

- [§DF-field-value-subjects](DF-field-value-subjects.md#df-field-value-subjects-a-rule-subject-may-select-on-a-scalar-fields-value-with-one-whose-clause), [§FS-rules.2.2](../../functional-spec/FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value), [§FS-errors.3.10](../../functional-spec/FS-errors.md#310-the-whose-clause-wording-migration): **A rule subject can select on a
  scalar field's value.** `Each SWR whose verification is Test must be cited by at least one
  TC.` holds only the requirements verified by test, and `list --selector` lists the same
  declarations. A declaration whose field holds another value is outside the rule. A whose
  clause that is still refused is answered for the field or value that failed, and a
  sentence whose clause is accepted but whose quantifier, object or predicate is refused is
  answered for that; through `0.21.0` either reason follows the old `unknown kind "…"` text
  as a prefix, which goes in `0.22.0`. **Who this breaks:** an exact-line consumer of the
  refusal of a sentence whose subject carries a whose clause; match the reason after ` — `. Closes
  [#1047](https://github.com/agent-grounds/grund/issues/1047).
