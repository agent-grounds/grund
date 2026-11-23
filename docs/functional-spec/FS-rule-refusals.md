# FS-rule-refusals: a refused rule sentence or selector is answered with the input that is accepted

A rule sentence ([§FS-rules.3](FS-rules.md#3-the-five-sentence-families)) or a subject selector ([§FS-rules.2](FS-rules.md#2-subject-selectors)) that the grammar does not
accept is refused, never reinterpreted. This page says which inputs each surface
refuses, with what reason, and how the input offered in its place is built from
what was typed, so an agent that pastes the offer back is not refused again
([§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work)). The line a refusal is printed in is [§FS-errors.2.5](FS-errors.md#25-refused-input).

## terms: Terms

Leans on [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding) and
[§FS-rules.terms](FS-rules.md#terms-terms) (subject, selector, chapter, family, unverifiable here).

- **refusal** — The answer to an input outside the grammar: a reason naming the production that
  failed, and an accepted input where one can be built.
- **accepted form** — A rule sentence offered after a refusal, which `check --rule` does not
  refuse when it is pasted back and which judges the units the typed sentence named
  ([§FS-rule-refusals.sentences.rewrite.same-units](FS-rule-refusals.md#sentencesrewritesame-units-a-form-judges-the-units-the-typed-sentence-named-or-none-is-offered)).
- **accepted selector** — A selector offered after a refusal, which `list --selector` does not
  refuse when it is pasted back.

## requirements: What every refusal holds to

Refusals are also held to four requirements stated elsewhere: an offered sentence stays
accepted as the language grows ([§FS-rules.requirements.addition](FS-rules.md#requirementsaddition-the-language-grows-by-addition--directional)); a sentence that needs a
phase-1 absence is refused rather than half-parsed ([§FS-rules.requirements.phase-one](FS-rules.md#requirementsphase-one-what-phase-1-leaves-out-is-owed-to-its-own-change--deferred)); a
reason's wording moves only through a window that keeps the old text as a prefix
([§FS-errors.requirements.prefix-windows](FS-errors.md#requirementsprefix-windows-a-wording-change-keeps-the-old-message-whole-until-its-window-closes)); and a rule and a selector refused for one subject
give one reason ([§REQ-one-answer](../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree)), save the case [§REQ-one-answer.differences.identity](../requirements/REQ-one-answer.md#differencesidentity-identity) lists.

### requirements.typed: An offer is built from what was typed and what is configured, and nothing else

On every surface that offers one — `check --rule`, a rule declaration's `invalid-rule`
finding and `list --selector` — the offer is the input as typed with only the failed
production rewritten ([§FS-rule-refusals.sentences.rewrite](FS-rule-refusals.md#sentencesrewrite-an-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced), [§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector)). It
draws on the typed text and the configured kinds alone, never on the scan, so no chapter
name is invented and no object kind guessed ([§FS-rule-refusals.sentences.rewrite.recovered](FS-rule-refusals.md#sentencesrewriterecovered-only-what-can-be-recovered-is-supplied)),
and it is offered only once it is parsed again and accepted
([§FS-rule-refusals.sentences.rewrite.reparsed](FS-rule-refusals.md#sentencesrewritereparsed-a-form-is-parsed-before-it-is-offered)), its label saying when the configuration has
to change first ([§FS-rule-refusals.sentences.named-sections](FS-rule-refusals.md#sentencesnamed-sections-a-subject-that-needs-named-sections-is-answered-with-one-they-make-valid)). Where that is not enough,
nothing is offered: the reason stands alone, followed on `check --rule` and
`list --selector` by the configured kinds where the missing piece is a kind
([§FS-rule-refusals.sentences.rewrite.no-form](FS-rule-refusals.md#sentencesrewriteno-form-where-no-form-is-offered)); and an ambiguity names every candidate and
chooses none ([§FS-rule-refusals.sentences](FS-rule-refusals.md#sentences-strict-refusals)). So an offer never names a kind, a chapter or a
declaration that neither the author nor the repository supplied, a rule's offer judges the
units its sentence named ([§FS-rule-refusals.sentences.rewrite.same-units](FS-rule-refusals.md#sentencesrewritesame-units-a-form-judges-the-units-the-typed-sentence-named-or-none-is-offered)), and pasting it
back is not refused. Serves [§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work), whose agent acts on the offer as printed.

### requirements.next-refusal: A refusal added from here on joins every table that enumerates them

The refusals are enumerated rather than open-ended: the stage order a sentence is refused
in ([§FS-rule-refusals.sentences.order](FS-rule-refusals.md#sentencesorder-a-sentence-is-refused-for-the-first-stage-it-fails)), the rewrite each makes and when it offers none
([§FS-rule-refusals.sentences.rewrite.by-refusal](FS-rule-refusals.md#sentencesrewriteby-refusal-the-rewrite-each-refusal-makes)), and the refusals a selector answers with
the true failure in place of a grammar mismatch ([§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector)) each list
them. A refusal added from here on joins each in the change that adds it, rather than
leaning on *every* to reach it: it takes a stage, so a sentence that also fails a later
stage is still refused for the first; it takes a row naming what its form replaces, or that
it offers none and whether `known kinds:` follows, so [§FS-rule-refusals.requirements.typed](FS-rule-refusals.md#requirementstyped-an-offer-is-built-from-what-was-typed-and-what-is-configured-and-nothing-else)
reaches it; and where its reason is true of a selector, `list --selector` gives the same
reason. This binds refusals added from here on; it is not a claim about those already written.
Serves [§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work).

## sentences: Strict refusals

No paraphrase or unlisted production is accepted. A refusal, printed in the shape of
[§FS-errors.2.5](FS-errors.md#25-refused-input) on both rule surfaces, `check --rule` and a rule declaration's `invalid-rule`
finding ([§FS-rules.7.1](FS-rules.md#71-invalid-rule)), identifies the failed production and gives its canonical accepted rewrite:
the sentence as typed with that production replaced, which is accepted when it is pasted
back, or nothing where the sentence does not say what belongs in its place. An ambiguity
names every candidate and chooses none. A literal subject that names no declaration, such
as `FS-missing`, is not refused here: whether it resolves is the scan's question ([§FS-rules.4](FS-rules.md#4-validation-lifecycle)).
In a repository whose kinds are `GOAL`, `REQ`, `FS`, `AR` and `RULE`, with named sections
on:

| Refused sentence | Exact reason and accepted rewrite(s) |
|---|---|
| `Each FS may not cite any AR.` | `modality "may not" is not accepted; accepted form: Each FS must not cite any AR.` |
| `Each FS must cite a GOAL.` | `quantifier "a" is ambiguous; accepted forms: "Each FS must cite at least one GOAL." or "Each FS must cite exactly one GOAL."` |
| `Each file in vendor/ must cite at least one FS.` | `path subjects are not accepted in phase 1`, then the line `known kinds: GOAL, REQ, FS, AR, RULE` |
| `FS-login.2 must cite at least one REQ.` | `numbered chapter subjects can detach when headings move`, and nothing after it |

### sentences.every: Unsupported every is refused as a quantifier, with an each repair

`Every KIND`, `The NAME chapter of every KIND` and `cite every KIND COUNT` are outside the
grammar ([§FS-rules.2](FS-rules.md#2-subject-selectors), [§FS-rules.3.3](FS-rules.md#33-per-target-coverage)) and refused with `quantifier "every" is not accepted`,
on both rule surfaces and, for the two subject spellings, on `list --selector`. The first
two are subject refusals in stage 5 and the last a predicate refusal in stage 6
([§FS-rule-refusals.sentences.order](FS-rule-refusals.md#sentencesorder-a-sentence-is-refused-for-the-first-stage-it-fails)), so every earlier stage keeps its reason. The repair
replaces only the quantifier token, `Every` with `Each` and `every` with `each`, and keeps
the typed subject, its complete chapter path, the modality, a valid count and the object
kind, a configured non-default kind included; a selector is answered with the corresponding
complete selector, and no breadcrumb. Either is offered only once it is parsed again under
the same configuration and accepted ([§FS-rule-refusals.sentences.rewrite.reparsed](FS-rule-refusals.md#sentencesrewritereparsed-a-form-is-parsed-before-it-is-offered)), and never under an
`after enabling it:` label, which only the named-sections reason carries
([§FS-rule-refusals.sentences.named-sections](FS-rule-refusals.md#sentencesnamed-sections-a-subject-that-needs-named-sections-is-answered-with-one-they-make-valid)), so with named sections off
`The NAME chapter of every KIND` is offered nothing on either surface. Where no configured
kind can be recovered, nothing is offered and `known kinds:` follows. Only `Every` and
`every`, in these positions, are quantifiers: `EVERY FS` is a literal subject, and a chapter
named `every`, a component such as `every.detail` and an ID such as `SYS-every` stay intact,
even inside a repair of another token.

### sentences.whitespace: Presence-name whitespace refusal

A presence `NAME` containing whitespace anywhere, including internal spaces, tabs, and
Unicode whitespace, is refused, unless it holds `or` between two words, which
[§FS-rule-refusals.sentences.presence-or](FS-rule-refusals.md#sentencespresence-or-or-between-chapter-names-is-refused-for-the-or) answers first. Between the reason and the explanation
that ends the message stands the accepted form, the sentence with the `NAME` trimmed of its
surrounding whitespace, as on the first line below, for
`Each FS must have exactly one  requirements chapter.` with two spaces. An empty `NAME`,
or one with whitespace inside, leaves no token to keep, so no form is offered, as on the
second line, for `Each FS must have exactly one Goal and hypothesis chapter.` and
`Each FS must have exactly one  chapter.`:

```text
chapter name must be a non-empty NAME with no surrounding whitespace; accepted form: Each FS must have exactly one requirements chapter. NAME forbids whitespace anywhere.
chapter name must be a non-empty NAME with no surrounding whitespace; NAME forbids whitespace anywhere.
```

### sentences.presence-or: `or` between chapter names is refused for the `or`

A presence `NAME` ([§FS-rules.3.1](FS-rules.md#31-chapter-presence)) is the text between the count and the trailing `chapter`
or `chapters`. Where that text, split at whitespace, holds the word `or` with a word before
it and a word after it, the author wrote a choice between chapters, not one chapter title
with spaces in it, and the sentence is refused for the `or` in place of
[§FS-rule-refusals.sentences.whitespace](FS-rule-refusals.md#sentenceswhitespace-presence-name-whitespace-refusal)'s reason ([§DF-presence-or-refusal](../decisions/functional/DF-presence-or-refusal.md#df-presence-or-refusal-a-presence-name-holding-or-between-two-words-is-refused-for-the-or-corrected-in-place)):

```text
"or" between chapter names is not accepted; a presence rule counts one NAME.
```

The words on either side may be anything, `chapter` included, so
`exactly one dearer chapter or unsold chapter` is refused the same way, and whitespace
around the `NAME` does not hide the `or`. The test is on the word as typed: `or` in lower
case, standing alone. A `NAME` that is `or` alone is one token and is accepted; an `or`
with no word on one side, a capitalised `OR`, or an `or` inside a word such as `order`
leaves the `NAME` to [§FS-rule-refusals.sentences.whitespace](FS-rule-refusals.md#sentenceswhitespace-presence-name-whitespace-refusal). No form is offered: the rule
language has no chapter disjunction, and one presence rule per name requires every chapter
rather than any one of them ([§FS-rule-refusals.sentences.rewrite.same-units](FS-rule-refusals.md#sentencesrewritesame-units-a-form-judges-the-units-the-typed-sentence-named-or-none-is-offered)). It is read in
stage 6, before the whitespace refusal ([§FS-rule-refusals.sentences.order](FS-rule-refusals.md#sentencesorder-a-sentence-is-refused-for-the-first-stage-it-fails)).

### sentences.named-sections: A subject that needs named sections is answered with one they make valid

Whether named sections are on is read under the project that configures the subject's kind
([§FS-scan.projects.kinds](FS-scan.md#projectskinds-whose-kinds-a-run-reads)); for a rule subject that is the project whose configuration reads
the rule. With `[id] named_sections = false`, a subject refused because named chapter
subjects require named sections is answered with a subject that turning them on makes
valid, so a reader who follows the suggestion is not refused again
([§DF-rule-after-enabling-rewrites](../decisions/functional/DF-rule-after-enabling-rewrites.md#df-rule-after-enabling-rewrites-a-rule-subject-that-needs-named-sections-is-answered-with-one-they-make-valid-in-place)). It is built from what was typed and from the configured
kinds:

1. The subject is parsed again with named sections on. Where that parse accepts it, the
   suggestion is the subject as typed.
2. Otherwise the suggestion is what [§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector)'s four steps build for that
   second refusal, written as a rule subject as
   [§FS-rule-refusals.sentences.rewrite.recovered](FS-rule-refusals.md#sentencesrewriterecovered-only-what-can-be-recovered-is-supplied) says. Where those steps drop a component
   that was typed, such as `1`, `Requirements` or `*`, the rule gets nothing
   ([§FS-rule-refusals.sentences.rewrite.same-units](FS-rule-refusals.md#sentencesrewritesame-units-a-form-judges-the-units-the-typed-sentence-named-or-none-is-offered), [§DF-rule-form-same-units](../decisions/functional/DF-rule-form-same-units.md#df-rule-form-same-units-a-refused-rules-accepted-form-judges-the-units-the-typed-sentence-named-or-none-is-offered-in-place)).
3. The label is `accepted form after enabling it:` only where the suggested subject is
   refused with the configuration as it is, and `accepted form:` where the repository
   accepts it as configured. Only a refusal whose reason is the named-sections one may
   carry the label. Where another part of the sentence is refused first and only the form
   built for it would need named sections, nothing is suggested, because the label would
   name nothing that reason asked to enable: with named sections off,
   `FS-login.requirements may not cite any AR.` is refused with
   `modality "may not" is not accepted` alone.
4. Where no configured kind can be recovered, or a configured ID grammar cannot be compiled
   with named sections on, nothing is suggested.
5. The reason stays `named chapter subjects require [id] named_sections = true`, the one
   the subject gets as configured. The modality and
   predicate are the ones the author typed, rewritten as
   [§FS-rule-refusals.sentences.rewrite](FS-rule-refusals.md#sentencesrewrite-an-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced) says where they are refused too, and where that
   offers no form for them nothing is suggested. The label of step 3 is decided on the
   whole offered sentence, not on its subject alone.

So every sentence offered after `accepted form after enabling it:` is accepted by
`check --rule` once `named_sections = true`, and every sentence offered after a plain
`accepted form:` is accepted with the configuration as it is. A subject refused for its own
production before named sections are asked about, such as `FS-login.*`, `FS-login.2` or
`Each chapter of each FS`, keeps that production's reason. In a repository whose kinds are
`FS`, `REQ` and `GOAL`, with named sections off and `FS-login` holding a named
`requirements` chapter with the sections `requirements.1` and `requirements.2`, each
`error:` line of `check --rule` opens with
`named chapter subjects require [id] named_sections = true`, and the table gives what
follows it:

| Refused sentence (`… must cite at least one REQ.`) | After the reason |
|---|---|
| `FS-login.requirements` | `; accepted form after enabling it: FS-login.requirements must cite at least one REQ.` |
| `FS-*.requirements` | `; accepted form after enabling it: The requirements chapter of each FS must cite at least one REQ.` |
| `The requirements.1 chapter of each FS` | nothing, and no line after it |
| `POLICY.requirements` | nothing on the `error:` line, then the line `known kinds: FS, REQ, GOAL` |

### sentences.chapter-path: A chapter path is refused for the component that failed

A chapter subject refused for its chapter path opens with the reason that names what is
wrong with the path, whichever spelling reached it, so the literal `FS-login.requirements.1`
and `The requirements.1 chapter of each FS` are refused for the same thing, and
`list --selector` names the same component ([§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector)). One component
decides the reason, by its shape ([§DF-rule-refusal-reasons](../decisions/functional/DF-rule-refusal-reasons.md#df-rule-refusal-reasons-a-chapter-path-is-refused-for-the-component-that-failed-corrected-in-place)):

| The component | The reason |
|---|---|
| all digits, such as `1` | `numbered chapter subjects can detach when headings move` |
| holding `*` | `section-component wildcards are not accepted in phase 1` |
| empty, left by a leading, doubled or trailing separator | `named chapter subject "<subject>" does not match the configured section grammar` |

An empty component is not a number. Which component decides depends on the spelling:

- In `The PATH chapter of each KIND`, as in a selector's `KIND.PATH`, it is the first
  component of `PATH` that is not a named one ([§FS-rules.2](FS-rules.md#2-subject-selectors)). Any other shape, such as
  `Requirements`, gets the section-grammar reason, and so does a path of named components
  that the configured section grammar refuses.
- In the literal spelling `ID.PATH`, a `*` anywhere in the path is refused first. Otherwise
  it is the first component that is empty or all digits. A literal path with neither goes on
  through the rest of the parse, so with named sections on `FS-login.Requirements` is
  refused for its ID grammar.

The literal spelling decides this before it asks whether named sections are on, so with
named sections off `FS-login.requirements.` still gets the section-grammar reason, while
`The requirements.1 chapter of each FS`, which asks first, keeps its named-sections-off
refusal ([§FS-rule-refusals.sentences.named-sections](FS-rule-refusals.md#sentencesnamed-sections-a-subject-that-needs-named-sections-is-answered-with-one-they-make-valid)). Whether a form follows the reason is
[§FS-rule-refusals.sentences.rewrite.same-units](FS-rule-refusals.md#sentencesrewritesame-units-a-form-judges-the-units-the-typed-sentence-named-or-none-is-offered)'s: in a repository whose kinds are `FS` and
`REQ`, with named sections on and `FS-login` holding a named `requirements` chapter,

| Refused sentence (`… must cite at least one REQ.`) | Exact reason and accepted form |
|---|---|
| `FS-login.requirements.` | `named chapter subject "FS-login.requirements." does not match the configured section grammar; accepted form: FS-login.requirements must cite at least one REQ.` |
| `FS-login..requirements` | `named chapter subject "FS-login..requirements" does not match the configured section grammar` |

### sentences.count-range: A count above the largest supported count is refused for its range

A count `N` ([§FS-rules.3](FS-rules.md#3-the-five-sentence-families)) is held in the platform's unsigned machine word, so the largest
supported count is that word's largest value, `18446744073709551615` on a 64-bit platform,
and that count is accepted. A numeral whose value is larger, read after any leading zeros
as in `018446744073709551616`, is refused for its range rather than its spelling, in a
floor, a ceiling and an exact count (`at least N`, `at most N`, `exactly N`) and in a
per-target count (`at least N times`, `at most N times`, `exactly N times`) alike. The
reason keeps the count refusal's text whole as its prefix ([§FS-errors.3](FS-errors.md#3-message-text)) and appends the
bound, printed from the platform's word rather than written down:

```text
count must be a canonical positive base-10 integer; the count is above the largest supported count, 18446744073709551615
```

There is no count in range the sentence names, so no form is offered
([§FS-rule-refusals.sentences.rewrite.no-form](FS-rule-refusals.md#sentencesrewriteno-form-where-no-form-is-offered)). A count with leading zeros whose value is in
range gets the count reason without the bound, and the form
([§FS-rule-refusals.sentences.rewrite.by-refusal](FS-rule-refusals.md#sentencesrewriteby-refusal-the-rewrite-each-refusal-makes)).

### sentences.modal-chapter: A chapter named after a modal word is refused for that name

A chapter subject whose `NAME` is a modal word, as in
`The must chapter of each FS must cite at least one FS.`, is cut at that word:
[§FS-rules.3.6](FS-rules.md#36-where-a-sentences-subject-ends) ends the subject at the first modality it finds in its fixed order, and the
`NAME` is the first. The subject left is `The`, which stage 5 of
[§FS-rule-refusals.sentences.order](FS-rule-refusals.md#sentencesorder-a-sentence-is-refused-for-the-first-stage-it-fails) refuses for the ID grammar. That reason names nothing the
author can fix, so it is followed, after ` — `, by one that does:

```text
literal subject "The" does not match the configured ID grammar — chapter name "must" is read as the rule's modality, so rename the chapter
```

The test is on what was typed: the subject the split leaves is exactly `The`, and the text
after the modality opens `chapter of each `. The name quoted is the modality the split
took, as typed: `must`, `must not`, `should` or `should not`. The split is not moved, so no
such sentence is accepted, but a split that reaches the name later is not refused, because
the modality it takes first is the rule's own:
`The should chapter of each FS must cite at least one FS.` is accepted. No form is offered,
and the `known kinds:` line follows on `check --rule`
([§FS-rule-refusals.sentences.rewrite.no-form](FS-rule-refusals.md#sentencesrewriteno-form-where-no-form-is-offered)). A selector is split only where it is not
already an accepted subject, so the selector `The must chapter of each FS` is accepted, while
`The must not chapter of each FS`, whose two-word `NAME` no subject accepts, splits where the
rule does ([§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector)), and so does a selector that is such a sentence:
`list --selector` gives the same reason, then its own `known kinds:` line.

### sentences.whose: A whose clause is refused for the part that failed

A whose clause ([§FS-rules.2.2](FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)) the grammar does not accept is refused in stage 5 of
[§FS-rule-refusals.sentences.order](FS-rule-refusals.md#sentencesorder-a-sentence-is-refused-for-the-first-stage-it-fails), after the kind and the chapter path, and its reason
names the clause, the field or the value that failed, never part of the clause as a kind
([§DF-field-value-subjects.2.4](../decisions/functional/DF-field-value-subjects.md#24-refused-for-the-part-that-failed-through-a-window)). The checks run in this order, and the first that fails gives
the reason:

1. a version-1 config, which declares no scalar field, whatever the clause says and on every
   surface: `whose clauses require grund_config_version = 2`;
2. the clause's shape: no ` is `, then an empty `FIELD`, then an empty `VALUE`;
3. `FIELD` is not a field of the kind, then it is a field but not a scalar;
4. `VALUE` has surrounding whitespace, which no `one_of` entry can hold
   ([§FS-fields.identity.match](FS-fields.md#identitymatch-titles-and-values-match-byte-for-byte));
5. `VALUE` is not one of the field's `one_of` entries, which the reason lists in their
   configured order, as `invalid-field-value` does.

A kind that declares no scalar field says so in place of listing its scalar fields, in the
field reason and the scalar reason alike. Where a `one_of` entry starts at `VALUE`'s first
byte and runs past the modality the split of [§FS-rules.3.6](FS-rules.md#36-where-a-sentences-subject-ends) took, the reason for the cut
value goes on to name that entry and the modal word that ends the subject.

Only a value joined by ` or ` is offered a form: one sentence per value, built, read again
and offered as a conjunction's forms are ([§FS-rule-refusals.sentences.conjunction](FS-rule-refusals.md#sentencesconjunction-any-clauses-joined-by-and-are-refused-as-a-conjunction)), and only
where every one is accepted. Every other refusal here offers none, because an offer never
picks a field or a value for the author ([§FS-rule-refusals.requirements.typed](FS-rule-refusals.md#requirementstyped-an-offer-is-built-from-what-was-typed-and-what-is-configured-and-nothing-else)); since the
kind was recognised, none adds a `known kinds:` line. A clause where no clause belongs keeps
the reason that spelling has without one as its prefix ([§FS-errors.3](FS-errors.md#3-message-text)): after an exact ID
and after an object kind, the reason appends what a whose clause narrows, as the last two
rows below show, the object kind's with `known kinds:` after it on `check --rule`; and the
selector `KIND.NAME` followed by a clause keeps its section-grammar reason and accepted
selector, since the clause makes its last component malformed ([§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector)).

In a repository whose kinds are `SWR`, `TC` and `AN`, where `SWR` declares the scalar
`verification` with `one_of = ["Test", "Analysis", "Inspection", "Demonstration"]` and the
prose field `rationale`, `AN` declares the scalar `status` with
`one_of = ["Draft", "Must fix", "fix and retest", "fix must wait"]`, and `TC` declares no
field, these are the exact reasons, the same with `The evidence chapter of each SWR` for
`Each SWR`:

| Refused sentence | Exact reason |
|---|---|
| `Each SWR whose verfication is Test must be cited by at least one TC.` | `"verfication" is not a field of kind SWR; its scalar fields: verification` |
| `Each SWR whose rationale is Test must be cited by at least one TC.` | `field "rationale" of kind SWR is not a scalar: it sets no one_of; its scalar fields: verification` |
| `Each TC whose verification is Test must cite at least one SWR.` | `"verification" is not a field of kind TC; kind TC declares no scalar field` |
| `Each SWR whose verification is Tset must be cited by at least one TC.` | `"Tset" is not a value of SWR.verification; kind SWR admits Test, Analysis, Inspection, Demonstration` |
| `Each SWR whose verification is Test or Demonstration must be cited by at least one TC.` | `"Test or Demonstration" is not a value of SWR.verification; kind SWR admits Test, Analysis, Inspection, Demonstration; accepted forms: "Each SWR whose verification is Test must be cited by at least one TC." and "Each SWR whose verification is Demonstration must be cited by at least one TC."` |
| `Each AN whose status is fix must wait must cite at least one SWR.` | `"fix" is not a value of AN.status; kind AN admits Draft, Must fix, fix and retest, fix must wait — value "fix must wait" holds the modal word "must", which ends the subject, so a rule cannot name it` |
| `Each SWR whose verification must be cited by at least one TC.` | `whose clause "whose verification" has no " is "; write whose FIELD is VALUE` |
| `Each SWR whose  is Test must be cited by at least one TC.` (two spaces) | `whose clause "whose  is Test" names no field; write whose FIELD is VALUE` |
| `Each SWR whose verification is  must be cited by at least one TC.` (two spaces) | `whose clause "whose verification is " names no value; write whose FIELD is VALUE` |
| `Each SWR whose verification is Test  must be cited by at least one TC.` (two spaces) | `value "Test " of SWR.verification has surrounding whitespace, which no one_of value can hold` |
| `SWR-brake-latency whose verification is Test must be cited by at least one TC.` | `literal subject "SWR-brake-latency whose verification is Test" does not match the configured ID grammar — a whose clause narrows only Each KIND and The NAME chapter of each KIND` |
| `Each AN must cite at least one SWR whose verification is Test.` | `unknown kind "SWR whose verification is Test" — a whose clause narrows only a rule's subject` |

### sentences.rewrite: An accepted form is the typed sentence with the failed part replaced

Every sentence a rule surface offers after `accepted form:`, `accepted forms:` or
`accepted form after enabling it:` is the sentence the author typed with the production
that failed rewritten, and pasting it back to `check --rule` is not refused, in every
repository and not only in one that happens to configure the kinds an example names
([§DF-rule-refusal-rewrites](../decisions/functional/DF-rule-refusal-rewrites.md#df-rule-refusal-rewrites-a-refused-rules-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced-in-place)). The subject, modality, count and object kinds that did not
fail are kept as they were typed, and only the production that failed is spelled
canonically, so a refusal never answers with a sentence about a kind, a chapter or a
declaration the author did not write. Both rule surfaces offer the same form, and the
reason before the `;` does not change with it. A form keeps a namespaced object kind as
typed, so in a member with no workspace in scope it may be unverifiable here ([§FS-rules.4.1](FS-rules.md#41-a-rule-this-scope-cannot-verify)), which is not a refusal of the form.

#### sentences.rewrite.recovered: Only what can be recovered is supplied

A refused subject is rebuilt with [§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector)'s four steps and written as a
rule subject: where a selector would get `KIND.NAME[.NAME…]` the rule gets
`The NAME[.NAME…] chapter of each KIND`, and where a selector would get a bare `KIND` the
rule gets `Each KIND`. An object kind is never guessed. A refused subject that supplies
recoverable chapter or literal declaration scope keeps that scope, so
`Each FS requirements chapter`, `Each FS requirements` and `Each FS.requirements` can be
offered `The requirements chapter of each FS`, and `FS-store requirements chapter` can be
offered `FS-store.requirements`; these remain refused spellings, not new grammar. The
predicate, its modality, citation direction, count and object included, is kept. Where the
typed scope cannot be reconstructed unambiguously, no form is offered rather than a
whole-declaration obligation in place of a chapter one or `Each KIND` in place of a literal
([§FS-rule-refusals.sentences.rewrite.same-units](FS-rule-refusals.md#sentencesrewritesame-units-a-form-judges-the-units-the-typed-sentence-named-or-none-is-offered)), so `Each chapter of each FS`, which
supplies no chapter name, gets none.

A chapter name is never invented: the form reads the configuration and not the scan. A
refused subject that holds a modality followed by a verb is the first of two clauses the
modality split read into the subject ([§FS-rules.3.6](FS-rules.md#36-where-a-sentences-subject-ends)), joined by something other than
` and `, or by an ` and ` that [§FS-rule-refusals.sentences.conjunction](FS-rule-refusals.md#sentencesconjunction-any-clauses-joined-by-and-are-refused-as-a-conjunction) never read because stage 2
refused the subject first ([§FS-rule-refusals.sentences.order](FS-rule-refusals.md#sentencesorder-a-sentence-is-refused-for-the-first-stage-it-fails)), as in
`Each */FS must cite at least one FS and must not cite any FS.`. It is not rebuilt, since a
form built from it would drop a clause: no form is offered, and `known kinds:` follows.

#### sentences.rewrite.same-units: A form judges the units the typed sentence named, or none is offered

On both rule surfaces a form is offered only where its subject selects the units the typed
subject named and its predicate counts in each of them what the typed predicate counted;
being accepted when pasted back is not enough ([§DF-rule-form-same-units](../decisions/functional/DF-rule-form-same-units.md#df-rule-form-same-units-a-refused-rules-accepted-form-judges-the-units-the-typed-sentence-named-or-none-is-offered-in-place)). Where no accepted
sentence judges the same units, the refusal is its reason alone. Four refusals meet this:

1. A refused chapter path, in any spelling, rebuilt by [§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector)'s four
   steps. Where step 3 drops a component that was typed, the rebuilt subject is a parent
   chapter, the declaration or `Each KIND`, and no form is offered. That component may be a
   numbered one, a wildcard, a name the configured section grammar refuses, as it refuses
   every name with named sections off, or any component after an empty one. The one
   component that may be dropped is an empty one at the end of the path, which names
   nothing, so with named sections on `FS-login.requirements.` is still offered
   `FS-login.requirements`.
2. A chapter subject in a presence rule, refused with
   `chapter subjects cannot have chapters`. A presence rule counts the direct chapters of
   each declaration its subject names ([§FS-rules.3.1](FS-rules.md#31-chapter-presence)), and no accepted sentence counts the
   chapters under a chapter, whether the subject is a literal `ID.PATH` or
   `The NAME chapter of each KIND`, and with or without a whose clause.
3. A `distinct` before a later object kind where none follows the count,
   `cite at least 3 GOAL or distinct REQ`. `distinct` counts declarations over the whole
   `or` list ([§FS-rules.3.2.1](FS-rules.md#321-counting-distinct-cited-declarations)), so dropping it counts the `REQ` sites and moving it after
   the count counts different `GOAL` too. Where `distinct` also follows the count, the later
   one restates it and is dropped.
4. A `distinct` in an inbound count, `be cited by at least 2 distinct GOAL`. No accepted
   sentence counts different citers ([§DF-distinct-target-counts](../decisions/functional/DF-distinct-target-counts.md#df-distinct-target-counts-an-outbound-count-may-count-distinct-cited-declarations-spelled-distinct-after-the-count)), and dropping `distinct`
   counts citation sites instead.

Every other rebuilt subject keeps the units that were typed, and keeps its form;
`Each */KIND` answered with `Each KIND` drops only the namespace, which its reason,
`subject namespaces must be local in phase 1`, already names. `list --selector` judges
nothing, so it is not held to this ([§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector)). A numbered criterion and
the chapters under a chapter have no accepted rule spelling: whether a numbered criterion
is cited is what `grund refs` answers ([§FS-refs](FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id)), and the chapters under a chapter are what `grund list --glob` lists
([§FS-coordinate-globs](FS-coordinate-globs.md#fs-coordinate-globs-a-coordinate-glob-selects-catalog-coordinates-by-pattern)).

#### sentences.rewrite.reparsed: A form is parsed before it is offered

The form is parsed again with the same vocabulary. Where another part of it is refused,
that part is replaced the same way and the form is parsed again. Each pass replaces one
refused production, so the passes end within the number of productions a sentence has, and
a pass that would leave the sentence as it was ends them with no form. A form is offered
only once the parser accepts it. Where a pass has nothing to put in, or more than one
candidate, no form is offered. A refusal that offers several forms offers them only where
every one is accepted.

#### sentences.rewrite.no-form: Where no form is offered

Where no form is offered, the refusal is its reason alone. That reason may end in guidance
that is not an accepted sentence, such as the configuration hint below. Where what could
not be supplied is a kind, a subject's or an object's, `check --rule` follows the reason with
the `known kinds:` line of [§FS-errors.2.5](FS-errors.md#25-refused-input), in the form an unknown `--kind` ends in
([§FS-list.1.1](FS-list.md#11---kind)). It names the whole set the subject accepts: the citable kinds of the
project whose configuration reads the subject, once each, in configuration order, which is
the root project at a workspace root and the member under a member path. Where what could
not be supplied is anything else, nothing follows the reason. In one project these are the
two lines `list --selector` prints for the same subject ([§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector)); at a workspace root that selector lists
every loaded project's kinds ([§FS-list.1.2](FS-list.md#12---project)), and the rule only the root's, because its
subject accepts no other.

When `Each ROW must cite at least one KIND.` names a configured, scanned, non-citable row
with a place, its reason additionally names that row, as `; for configured row "ROW", `,
and tells the author to express the obligation in `grund.toml`: `[rules.citations.ROW]`
with `must = ["KIND"]` in version 2 ([§FS-config-v2.rules.citations](FS-config-v2.md#rulescitations-rulescitations)), or `[citations.ROW]`
in version 1 ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)), whether or not that entry is written yet. It names that
configuration route, never an accepted form, and never recommends making the row citable.
That entry alone judges only a file there that already cites something ([§FS-check.3.11](FS-check.md#311-missing-required-citation)),
so where the row grounds nothing at any strength the hint also names the grounding setting
the Citation directions bullet names ([§FS-init.2.3.5.2.2](FS-init.md#23522-a-home-whose-row-grounds-nothing-says-so)), in the live version, as what
makes a file there that cites nothing a finding; where the row grounds, it names the
citation entry alone. `Each file in PATH must cite at least one KIND.` gets the same
guidance when `PATH` is exactly one such row's place, compared as complete project-relative
configured paths, a folder's terminal `/` present or absent. An ancestor of several places,
a descendant, a substring, a glob, an absolute path, a path with `.` or `..` components, an
unscanned or citable row and an alias of a row's name get none. Only the complete canonical
predicate `must cite at least one KIND`, with one local citable kind named exactly as
configured, gets it; a presence, incoming-citation, prohibition, other modality or count,
disjunction, namespaced or unknown object, or otherwise malformed sentence does not. The
guidance follows the reason the refusal already has,
`path subjects are not accepted in phase 1` or the row's non-citable-kind reason
([§FS-errors.2.5.1](FS-errors.md#251-a-kind-that-declares-no-ids)), never an earlier stage's
refusal ([§FS-rule-refusals.sentences.order](FS-rule-refusals.md#sentencesorder-a-sentence-is-refused-for-the-first-stage-it-fails)); `known kinds:` follows on
`check --rule`, a rule declaration's `invalid-rule` reason carries the guidance too, and a
selector or object-kind refusal carries none.

#### sentences.rewrite.by-refusal: The rewrite each refusal makes

| Refusal | Its reason | What the form replaces | No form when |
|---|---|---|---|
| modality `may not` | `modality "may not" is not accepted` | `must not` | — |
| `cite no` | `"cite no" is not accepted` | `<modality> not cite any` | — |
| lowercase `each` | `fixed word "Each" is case-sensitive` | `Each` | — |
| unsupported `Every` or `every` ([§FS-rule-refusals.sentences.every](FS-rule-refusals.md#sentencesevery-unsupported-every-is-refused-as-a-quantifier-with-an-each-repair)) | `quantifier "every" is not accepted` | only the quantifier token, with `Each` or `each` in the same position | the reparse cannot build an accepted form; `known kinds:` follows only where a kind could not be supplied |
| no terminal `.` | `rule must end with "."` | appends `.` | — |
| numeric `at least 1`, `exactly 1` | `numeric "at least 1" is not canonical`, `numeric "exactly 1" is not canonical` | `at least one`, `exactly one` | — |
| numeric `at least 1 times`, `at most 1 times`, `exactly 1 times` ([§DF-per-target-once-ceiling](../decisions/functional/DF-per-target-once-ceiling.md#df-per-target-once-ceiling-spell-a-per-target-ceiling-of-one-as-at-most-once)) | `numeric "<bound> 1 times" is not canonical`, `<bound>` as typed | `at least once`, `at most once`, `exactly once` | — |
| a per-target count without `times` | `per-target counts must end in "times"` | the count typed after the bound, then `times` | that count is not a numeral |
| a count after a bound that is not a canonical positive base-10 integer, such as `01`, `0` or `two` | `count must be a canonical positive base-10 integer` | the count without its leading zeros | the count is zero, is not a numeral, or is above the largest supported count |
| a count above the largest supported count ([§FS-rule-refusals.sentences.count-range](FS-rule-refusals.md#sentencescount-range-a-count-above-the-largest-supported-count-is-refused-for-its-range)) | the count reason, then the bound | — | always |
| quantifier `a` | `quantifier "a" is ambiguous` | two forms, `at least one` and `exactly one` | either form is refused |
| clauses joined by `and` ([§FS-rule-refusals.sentences.conjunction](FS-rule-refusals.md#sentencesconjunction-any-clauses-joined-by-and-are-refused-as-a-conjunction)) | `conjunctions are not accepted` | one form per clause | any form is refused |
| a prohibition without `cite any` | `a prohibition must use "cite any"` | `cite [count] KINDS` becomes `cite any KINDS` | the predicate is anything else |
| a presence object not ending in `chapter` | `chapter presence must end in "chapter"` | its last word becomes `chapter` or `chapters`, as the count takes | no word precedes it |
| a presence noun of the wrong number | `chapter count has the wrong singular/plural spelling` | `chapter` or `chapters`, as the count takes | — |
| a presence `NAME` holding `or` between two words ([§FS-rule-refusals.sentences.presence-or](FS-rule-refusals.md#sentencespresence-or-or-between-chapter-names-is-refused-for-the-or)) | its own | — | always |
| a presence `NAME` with whitespace ([§FS-rule-refusals.sentences.whitespace](FS-rule-refusals.md#sentenceswhitespace-presence-name-whitespace-refusal)) | its own | the `NAME` trimmed of its surrounding whitespace | the `NAME` is empty or has whitespace inside |
| a chapter subject in a presence rule ([§FS-rule-refusals.sentences.rewrite.same-units](FS-rule-refusals.md#sentencesrewritesame-units-a-form-judges-the-units-the-typed-sentence-named-or-none-is-offered)) | `chapter subjects cannot have chapters` | — | always |
| `distinct` after `at least one` ([§FS-rules.3.2.1](FS-rules.md#321-counting-distinct-cited-declarations)) | `"distinct" is redundant after "at least one"` | drops `distinct` | — |
| `distinct` in per-target coverage or a prohibition | `"distinct" has no meaning per target`, `"distinct" has no meaning in a prohibition` | drops `distinct` | — |
| `distinct` before a later object kind | `"distinct" must follow the count` | drops each `distinct` after the first object kind | `distinct` does not also follow the count ([§FS-rule-refusals.sentences.rewrite.same-units](FS-rule-refusals.md#sentencesrewritesame-units-a-form-judges-the-units-the-typed-sentence-named-or-none-is-offered)); no `known kinds:` follows |
| `distinct` in an inbound count | `"distinct" counts only outbound citations` | — | always; no `known kinds:` follows |
| a refused subject: an unknown kind, `*/`, a section-component wildcard, a numbered chapter, the section grammar, the ID grammar, `Each chapter of each` | `unknown kind "<KIND>"`, `subject namespaces must be local in phase 1`, `section-component wildcards are not accepted in phase 1`, `numbered chapter subjects can detach when headings move`, the section-grammar reason ([§FS-rule-refusals.sentences.chapter-path](FS-rule-refusals.md#sentenceschapter-path-a-chapter-path-is-refused-for-the-component-that-failed)), `literal subject "<subject>" does not match the configured ID grammar`, `chapter-quantified subjects are not accepted in phase 1` | the subject, rebuilt as [§FS-rule-refusals.sentences.rewrite.recovered](FS-rule-refusals.md#sentencesrewriterecovered-only-what-can-be-recovered-is-supplied) says | scope cannot be reconstructed, rebuilding it drops a typed component ([§FS-rule-refusals.sentences.rewrite.same-units](FS-rule-refusals.md#sentencesrewritesame-units-a-form-judges-the-units-the-typed-sentence-named-or-none-is-offered)), no configured kind is recovered, or the subject holds a clause; `known kinds:` follows only for the latter two |
| the ID grammar for a subject `The` cut short at a chapter named after a modal word ([§FS-rule-refusals.sentences.modal-chapter](FS-rule-refusals.md#sentencesmodal-chapter-a-chapter-named-after-a-modal-word-is-refused-for-that-name)) | its own | — | always; `known kinds:` follows |
| a path subject, `Each file in …` | `path subjects are not accepted in phase 1` | — | always; `known kinds:` follows |
| a whose clause in a version-1 config, with no ` is `, an empty `FIELD` or `VALUE`, a `FIELD` that is no field or no scalar of the kind, a `VALUE` with surrounding whitespace, or a `VALUE` outside `one_of` ([§FS-rule-refusals.sentences.whose](FS-rule-refusals.md#sentenceswhose-a-whose-clause-is-refused-for-the-part-that-failed)) | its own | — | always; no `known kinds:` follows |
| a `VALUE` joined by ` or ` | its own | one form per value, as a conjunction's are | any form is refused, and always on a selector |
| a `VALUE` cut by the modality split | its own | — | always |
| an exact ID followed by a whose clause | its own | — | always; `known kinds:` follows on `list --selector` |
| an object kind followed by a whose clause | its own | — | always; `known kinds:` follows on `check --rule` |
| an unknown or malformed object kind | `unknown kind "<KIND>"` | — | always; `known kinds:` follows |
| no modality, an unknown verb, a count phrase outside the canonical counts, such as `some`, a count with no object | `rule has no accepted modality`, `verb is not accepted`, `count is not accepted; the canonical counts are "at least one", "at least N", "at most N", "exactly one" and "exactly N" for a base-10 N`, `count has no object` | — | always |

A form that keeps a `distinct` the repaired sentence makes redundant is refused when it is
parsed again, and the next pass drops it ([§FS-rule-refusals.sentences.rewrite.reparsed](FS-rule-refusals.md#sentencesrewritereparsed-a-form-is-parsed-before-it-is-offered)):
`at least 1 distinct GOAL` is offered `at least one GOAL`, and
`must not cite at least 2 distinct GOAL` is offered `must not cite any GOAL`, while
`exactly 1 distinct GOAL` keeps it. Where `distinct` follows the count, every other refusal
is the one the same sentence without `distinct` gets ([§DF-distinct-target-counts](../decisions/functional/DF-distinct-target-counts.md#df-distinct-target-counts-an-outbound-count-may-count-distinct-cited-declarations-spelled-distinct-after-the-count)). The
missing-`times` and canonical-integer rows name their own failed production, but their forms
are parsed again too: `at most 1`, `at most 1 time` and `at most 01 times` each end up
offering `at most once`, keeping the typed subject, object kind and modality. A form that
would name a kind the repository does not configure is refused when it is parsed again, and
no pass guesses one, so in a repository whose one kind is `FS`
`Each FS must cite at least one GOAL and must not cite any AR.` is answered with
`conjunctions are not accepted` and then the line `known kinds: FS`.

### sentences.conjunction: Any clauses joined by `and` are refused as a conjunction

One sentence states one rule ([§FS-rules.3](FS-rules.md#3-the-five-sentence-families)), so a predicate that joins clauses is refused as
a conjunction, whatever its subject, counts and kinds. A sentence is a conjunction when it
contains ` and ` immediately followed by a modality word, `must ` or `should `. Every such
` and ` ends one clause and starts the next, which begins with its own modality. A whose clause's `VALUE` may hold ` and ` ([§FS-rules.2.2](FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)) and is left whole; only
a `VALUE` whose last word is `and`, so that the sentence reads ` and must ` or
` and should ` there, is read as a conjunction, because this stage runs before the subject
is read.

The reason is `conjunctions are not accepted`. Its forms are one sentence per clause: the
first is the first clause, and each later one is the first clause's subject, read by
[§FS-rules.3.6](FS-rules.md#36-where-a-sentences-subject-ends), followed by that clause, each ending in `.`. Each form is then built and read
again as [§FS-rule-refusals.sentences.rewrite](FS-rule-refusals.md#sentencesrewrite-an-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced) says, and they are offered only where every one
is accepted, after `; accepted forms: `, quoted, two joined as `"A." and "B."` and more as
`"A.", "B.", and "C."`. Where any one is not, the reason stands alone. The check is stage 3
of [§FS-rule-refusals.sentences.order](FS-rule-refusals.md#sentencesorder-a-sentence-is-refused-for-the-first-stage-it-fails), before the modality split of [§FS-rules.3.6](FS-rules.md#36-where-a-sentences-subject-ends), so the
subject is never read from a fragment, and no refusal names part of a sentence as a kind.
In a repository whose one kind is `FS`:

| Refused sentence | Exact reason |
|---|---|
| `Each FS must cite at least one FS and must not cite any FS.` | `conjunctions are not accepted; accepted forms: "Each FS must cite at least one FS." and "Each FS must not cite any FS."` |
| `Each FS must cite at least one FS and must not cite any FS and should cite at most 2 FS.` | `conjunctions are not accepted; accepted forms: "Each FS must cite at least one FS.", "Each FS must not cite any FS.", and "Each FS should cite at most 2 FS."` |
| `Each FS must cite at least one FS and must cite at most one FS.` | `conjunctions are not accepted` |

The third row's second clause counts `at most one`, which is refused with no form, so neither
form is offered.

### sentences.order: A sentence is refused for the first stage it fails

A sentence is read in one fixed order of stages, and it is refused for the first stage it
fails; no later stage is read ([§DF-one-answer-rulings-grammar](../decisions/functional/DF-one-answer-rulings-grammar.md#df-one-answer-rulings-grammar-one-grammar-keeps-the-documented-refusal-bytes-and-order-and-one-kind-universe-keeps-each-commands-vocabulary), D20). The examples are read
in a repository whose one kind is `FS`, with named sections on:

| Stage | What it refuses | Example that also fails a later stage | Reason it opens with |
|---|---|---|---|
| 1. terminal period | a sentence that does not end in `.` | `Each FS may not cite any FS` | `rule must end with "."` |
| 2. fixed word and phase-1 subject | `each` for `Each`, and the deliberate phase-1 absences of [§FS-rules.12](FS-rules.md#12-deliberate-phase-1-absences): a path subject (`Each file in`), `Each */KIND`, `Each chapter of each KIND`, and a section-component wildcard in the first clause's subject, in any spelling | `FS-x.* must cite at least one FS and must not cite any FS.` | `section-component wildcards are not accepted in phase 1` |
| 3. conjunction | clauses joined by `and` ([§FS-rule-refusals.sentences.conjunction](FS-rule-refusals.md#sentencesconjunction-any-clauses-joined-by-and-are-refused-as-a-conjunction)) | `Each FS may not cite any FS and must cite at least one FS.` | `conjunctions are not accepted` |
| 4. modality | no accepted modality, `may not` included | `Each POLICY may not cite any FS.` | `modality "may not" is not accepted` |
| 5. the rest of the subject | unsupported `Every KIND` or `The NAME chapter of every KIND`, an unknown or non-citable kind, named sections off, the section grammar, a numbered chapter, the ID grammar, a chapter named after a modal word ([§FS-rule-refusals.sentences.modal-chapter](FS-rule-refusals.md#sentencesmodal-chapter-a-chapter-named-after-a-modal-word-is-refused-for-that-name)), then a whose clause, in the order of [§FS-rule-refusals.sentences.whose](FS-rule-refusals.md#sentenceswhose-a-whose-clause-is-refused-for-the-part-that-failed) | `Each POLICY must cite no FS.` | `unknown kind "POLICY"` |
| 6. the predicate | the verb, quantifier (including `cite every`), count (including per-target numeric `at most 1 times`) and object kind, an object kind followed by a whose clause included, a chapter subject that would have chapters, a presence `NAME`, its `or` between two words ([§FS-rule-refusals.sentences.presence-or](FS-rule-refusals.md#sentencespresence-or-or-between-chapter-names-is-refused-for-the-or)) before its whitespace, and last a `distinct` the predicate does not admit ([§FS-rules.3.2.1](FS-rules.md#321-counting-distinct-cited-declarations)) | `Each FS must cite no FS.` | `"cite no" is not accepted` |

Stage 2 reads its rows from the sentence's start. A path subject is `Each file in`, refused
whatever follows; `Each docs/fs/` names a kind, refused in stage 5. The wildcard is read
from the subject of the first clause, the text before the first `and` that joins clauses,
so that row is read only where an accepted modality splits that subject off:
`FS-x.* may not cite any FS.` is refused for its modality. A subject that reaches a wildcard
only through a component that fails earlier in stage 5 — `The * chapter of each FS` with
named sections off — keeps that component's reason.

## selectors: A refused selector is answered with a selector

Every `list` mode that takes `--selector`, `--size` included, refuses a selector it cannot
accept. Its `error:` line is the refused-input shape of [§FS-errors.2.5](FS-errors.md#25-refused-input): it names the
production that failed and, where recovery can supply one, ends in exactly one accepted
selector, which a reader can paste back into `--selector`.

The reason is the one [§FS-rule-refusals.sentences](FS-rule-refusals.md#sentences-strict-refusals) gives for the same subject wherever that
reason is true, so a rule and a selector never disagree about what failed. These refusals
name the true failure in place of a grammar or vocabulary mismatch. A numbered component
reached through `KIND.NAME` or `The NAME chapter of each KIND` gets the numbered reason the
ID spelling gets. A wildcard component reached the same way gets the wildcard reason.
`Each chapter of each KIND` gets the chapter-quantified reason, not an unknown kind.
`Each */KIND` gets the namespace reason the rule sentence gets, not an unknown kind. An
empty component, left by a doubled or trailing separator, is not a number in any spelling,
so it gets the section-grammar reason and no breadcrumb
([§FS-rule-refusals.sentences.chapter-path](FS-rule-refusals.md#sentenceschapter-path-a-chapter-path-is-refused-for-the-component-that-failed)). `Every KIND` and
`The NAME chapter of every KIND` get the unsupported-quantifier reason and token-only repair
of [§FS-rule-refusals.sentences.every](FS-rule-refusals.md#sentencesevery-unsupported-every-is-refused-as-a-quantifier-with-an-each-repair), rather than a literal-ID mismatch. A fixed word in
the wrong case, as in `each FS`, keeps the literal-subject reason as its prefix
([§FS-errors.3](FS-errors.md#3-message-text)), followed by ` — ` and the rule's `fixed word "Each" is case-sensitive`, and
the `known kinds:` line follows it ([§DF-one-answer-rulings-grammar](../decisions/functional/DF-one-answer-rulings-grammar.md#df-one-answer-rulings-grammar-one-grammar-keeps-the-documented-refusal-bytes-and-order-and-one-kind-universe-keeps-each-commands-vocabulary), G06).

A subject with a whose clause gets the reason [§FS-rule-refusals.sentences.whose](FS-rule-refusals.md#sentenceswhose-a-whose-clause-is-refused-for-the-part-that-failed) gives the
same subject in a rule, and no accepted selector: the clause's field and value are the
author's to choose, and a value joined by ` or ` has no one selector to offer. Since the
kind was recognised, no `known kinds:` line follows. A clause where no clause belongs is
answered as [§FS-rule-refusals.sentences.whose](FS-rule-refusals.md#sentenceswhose-a-whose-clause-is-refused-for-the-part-that-failed) says, an exact ID's with `known kinds:` after
it. In the repository of that point:

| Refused selector | Exact reason and accepted selector | Lines after it |
|---|---|---|
| `SWR.evidence whose verification is Test` | `named chapter subject "SWR.evidence whose verification is Test" does not match the configured section grammar; accepted selector: SWR` | none |

The suggestion is built from what was typed and from the configured kinds, never from a
fixed template. A chapter typed in another word order is kept and an unnamed
`Each chapter of each KIND` gets no selector, as
[§FS-rule-refusals.sentences.rewrite.recovered](FS-rule-refusals.md#sentencesrewriterecovered-only-what-can-be-recovered-is-supplied) says of a rule. A selector judges nothing: it
lists the nearest unit that holds what was typed, so it keeps the longest valid prefix of a
refused path where a rule is offered nothing
([§FS-rule-refusals.sentences.rewrite.same-units](FS-rule-refusals.md#sentencesrewritesame-units-a-form-judges-the-units-the-typed-sentence-named-or-none-is-offered)). The suggestion is built in four steps:

1. Its kind is the configured kind the text names or starts with: `FS-*` and `FS-login`
   both give `FS`.
2. A declaration written as a valid ID is kept.
3. The longest run of named components before the first refused one is kept:
   `requirements.1` gives `requirements`, and `*` gives nothing.
4. It keeps the spelling that was typed. `KIND[.NAME…]` and `Each KIND` keep theirs. An ID
   stays an ID when step 2 kept its declaration, and otherwise becomes `KIND[.NAME…]`.
   `The NAME chapter of each KIND` keeps its spelling while a `NAME` survives step 3, and
   otherwise becomes `Each KIND`.

With named sections off, a selector refused for needing them is suggested what the same
selector would be suggested with them on: the selector as typed where enabling them makes
it valid, else the selector the four steps build for the refusal it would still get, and
nothing where no configured kind is recovered. The label is
`accepted selector after enabling it:` only where the suggestion itself needs named
sections, and `accepted selector:` where the repository accepts it as configured. The
reason stays the one a rule gets for the subject as configured
([§FS-rule-refusals.sentences.named-sections](FS-rule-refusals.md#sentencesnamed-sections-a-subject-that-needs-named-sections-is-answered-with-one-they-make-valid)), so these refusals add no breadcrumb.

Where no configured kind can be recovered, nothing is guessed: the reason stands alone, and
a second line lists the configured citable kinds exactly as an unknown `--kind` does
([§FS-list.1.1](FS-list.md#11---kind)). A kind that is configured but not citable is not a kind that cannot be
recovered: its reason is [§FS-errors.2.5.1](FS-errors.md#251-a-kind-that-declares-no-ids)'s, and the line after it is the same
`known kinds:` line. Which kinds a selector accepts, and whether `KIND.NAME` needs named
sections, are both read in the selected projects as [§FS-scan.projects.kinds](FS-scan.md#projectskinds-whose-kinds-a-run-reads) says, the second
under the project that configures `KIND`.

A refusal never suggests a rule sentence. A selector that is itself one, a subject followed
by a modality, is refused for being one. Its subject is the one the rule sentence has,
ending where [§FS-rules.3.6](FS-rules.md#36-where-a-sentences-subject-ends) ends it, so a selector and `check --rule` read the same subject
from the same sentence. Its suggestion is that subject, and when the subject would itself
be refused the refusal is the subject's own.

The refusals whose reason names a numbered chapter, a section-component wildcard or a
chapter quantifier add one stderr breadcrumb ([§FS-errors.1](FS-errors.md#1-streams)), whether or not a kind was
recovered, as a third line after the kinds where there are kinds. The units they reach for
are found by expanding the chapter the suggestion selects, or, where nothing is suggested,
the units of a kind the `known kinds:` line names:

```text
hint: grund show --batch --toc expands each selected unit into its sections
```

In a repository whose one kind is `FS`, with named sections on, these are the exact
refusals. A line marked *hint* is the breadcrumb above.

| Refused selector | Exact reason and accepted selector | Lines after it |
|---|---|---|
| `FS-*.requirements` | `literal subject "FS-*.requirements" does not match the configured ID grammar; accepted selector: FS.requirements` | none |
| `FS.requirements.1` | `numbered chapter subjects can detach when headings move; accepted selector: FS.requirements` | hint |
| `Each chapter of each FS` | `chapter-quantified subjects are not accepted in phase 1` | `known kinds: FS`, then hint |
| `Each POLICY` | `unknown kind "POLICY"` | `known kinds: FS` |
| `API.requirements.1` | `numbered chapter subjects can detach when headings move` | `known kinds: FS`, then hint |
| `The should chapter of each FS must cite at least one FS.` | `a rule sentence is not a selector; accepted selector: The should chapter of each FS` | none |

With named sections off in the same repository, these are the exact refusals. None of them
earns the breadcrumb.

| Refused selector | Exact reason and accepted selector | Lines after it |
|---|---|---|
| `FS.requirements` | `named chapter subjects require [id] named_sections = true; accepted selector after enabling it: FS.requirements` | none |
| `FS.*` | `named chapter subjects require [id] named_sections = true; accepted selector: FS` | none |
| `The * chapter of each FS` | `named chapter subjects require [id] named_sections = true; accepted selector: Each FS` | none |
| `POLICY.requirements` | `named chapter subjects require [id] named_sections = true` | `known kinds: FS` |

An exact literal that does not resolve or is ambiguous keeps its refusal, which suggests
nothing. A selector and a rule refused for the same subject give the same reason, and where
both offer one they suggest the same units, each in its own spelling: `FS-*.requirements`
is answered with `FS.requirements` as a selector and with
`The requirements chapter of each FS` in a rule ([§DF-selector-refusal-rewrites](../decisions/functional/DF-selector-refusal-rewrites.md#df-selector-refusal-rewrites-a-refused-selector-is-answered-with-a-selector-and-its-old-lines-are-replaced-not-appended-to),
[§DF-rule-refusal-rewrites](../decisions/functional/DF-rule-refusal-rewrites.md#df-rule-refusal-rewrites-a-refused-rules-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced-in-place)).
