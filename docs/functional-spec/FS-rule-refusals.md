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
  refuse when it is pasted back.
- **accepted selector** — A selector offered after a refusal, which `list --selector` does not
  refuse when it is pasted back.

## requirements: What every refusal holds to

Two requirements on refusals as a whole, each carrying a mark of
[§FS-config.requirements](FS-config.md#requirements-what-the-config-contract-holds-to). Four more are left out because they are stated
elsewhere: *an offered sentence stays accepted as the language grows* is
[§FS-rules.requirements.addition](FS-rules.md#requirementsaddition-the-language-grows-by-addition--directional); *a sentence that needs a phase-1 absence is
refused rather than half-parsed* is [§FS-rules.requirements.phase-one](FS-rules.md#requirementsphase-one-what-phase-1-leaves-out-is-owed-to-its-own-change--deferred); *a
reason's wording moves only through a window that keeps the old text as a
prefix*, as stage 2's wildcard reason does ([§FS-rule-refusals.sentences.order](FS-rule-refusals.md#sentencesorder-a-sentence-is-refused-for-the-first-stage-it-fails)),
is [§FS-errors.requirements.prefix-windows](FS-errors.md#requirementsprefix-windows-a-wording-change-keeps-the-old-message-whole-until-its-window-closes--realized-directional-for-the-next-change); and *a rule and a selector refused
for one subject give one reason* is [§REQ-one-answer](../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree), save the case its
differences list ([§REQ-one-answer.differences.identity](../requirements/REQ-one-answer.md#differencesidentity-identity)).

### requirements.typed: An offer is built from what was typed and what is configured, and nothing else — realized

On every surface that offers one — `check --rule`, a rule declaration's
`invalid-rule` finding and `list --selector` — the offer is the input as typed
with only the failed production rewritten ([§FS-rule-refusals.sentences.rewrite.replaced](FS-rule-refusals.md#sentencesrewritereplaced-only-the-failed-part-is-replaced),
[§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector)). It draws on the typed text and the configured kinds
alone, never on the scan, so no chapter name is invented and no object kind
guessed ([§FS-rule-refusals.sentences.rewrite.recovered](FS-rule-refusals.md#sentencesrewriterecovered-only-what-can-be-recovered-is-supplied)), and it is parsed again
and offered only once it is accepted ([§FS-rule-refusals.sentences.rewrite.reparsed](FS-rule-refusals.md#sentencesrewritereparsed-a-form-is-parsed-before-it-is-offered)),
its label saying when the configuration has to change first
([§FS-rule-refusals.sentences.named-sections](FS-rule-refusals.md#sentencesnamed-sections-a-subject-that-needs-named-sections-is-answered-with-one-they-make-valid)). Where that is not enough, nothing
is offered: the reason stands alone, followed on `check --rule` and
`list --selector` by the configured kinds where the missing piece is a kind
([§FS-rule-refusals.sentences.rewrite.no-form](FS-rule-refusals.md#sentencesrewriteno-form-where-no-form-is-offered), [§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector)); and
an ambiguity names every candidate and chooses none ([§FS-rule-refusals.sentences](FS-rule-refusals.md#sentences-strict-refusals)).
So an offer never names a kind, a chapter or a declaration that neither the
author nor the repository supplied, and pasting it back is not refused. Serves
[§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work), whose agent acts on the offer as printed.

### requirements.next-refusal: A refusal added from here on joins every table that enumerates them — directional

The refusals are enumerated rather than open-ended: the stage order a sentence
is refused in ([§FS-rule-refusals.sentences.order](FS-rule-refusals.md#sentencesorder-a-sentence-is-refused-for-the-first-stage-it-fails)), the rewrite each makes and
when it offers none ([§FS-rule-refusals.sentences.rewrite.by-refusal](FS-rule-refusals.md#sentencesrewriteby-refusal-the-rewrite-each-refusal-makes)), and the
refusals a selector answers with the true failure in place of a grammar
mismatch ([§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector)) each list them. A refusal added from here
on joins each in the change that adds it, rather than leaning on *every* to
reach it: it takes a stage, so a sentence that also fails a later stage is still
refused for the first; it takes a row naming what its form replaces, or that it
offers none and whether `known kinds:` follows, so
[§FS-rule-refusals.requirements.typed](FS-rule-refusals.md#requirementstyped-an-offer-is-built-from-what-was-typed-and-what-is-configured-and-nothing-else--realized) reaches it; and where its reason is true of
a selector, `list --selector` gives the same reason. This binds refusals added
from here on; it is not a claim about those written today. Serves
[§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work).

## sentences: Strict refusals

No paraphrase or unlisted production is accepted. A refusal, printed in the
shape of [§FS-errors.2.5](FS-errors.md#25-refused-input), identifies the failed production and gives its
canonical accepted rewrite: the sentence as typed with that production
replaced, which is accepted when it is pasted back.
An ambiguity names every candidate and chooses none. A chapter subject
refused for its chapter path names the component that failed, whichever
spelling reached it ([§FS-rule-refusals.sentences.chapter-path](FS-rule-refusals.md#sentenceschapter-path-a-chapter-path-is-refused-for-the-component-that-failed)). [§FS-rule-refusals.sentences.rewrite](FS-rule-refusals.md#sentencesrewrite-an-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced) builds
every rewrite, and it names the refusals that offer none because the sentence
does not say what belongs in their place: a path subject, an object kind that
is not configured, a subject from which no configured kind can be recovered,
and a sentence with no modality, verb or accepted count. In a repository whose
kinds are `GOAL`, `REQ`, `FS`, `AR` and `RULE`, with named sections on, the
documentation and parser tests carry at least these exact rows. Where a row
offers no rewrite, `check --rule` prints the reason (including the configuration
guidance of [§FS-rule-refusals.sentences.rewrite.no-form.place](FS-rule-refusals.md#sentencesrewriteno-formplace-a-refused-place-subject-names-its-citation-configuration) where applicable)
and then the line
`known kinds: GOAL, REQ, FS, AR, RULE`:

| Refused sentence | Exact reason and accepted rewrite(s) |
|---|---|
| `Each FS may not cite any AR.` | `modality "may not" is not accepted; accepted form: Each FS must not cite any AR.` |
| `Each FS must cite no AR.` | `"cite no" is not accepted; accepted form: Each FS must not cite any AR.` |
| `Each FS must cite a GOAL.` | `quantifier "a" is ambiguous; accepted forms: "Each FS must cite at least one GOAL." or "Each FS must cite exactly one GOAL."` |
| `Each FS must cite at least 1 GOAL.` | `numeric "at least 1" is not canonical; accepted form: Each FS must cite at least one GOAL.` |
| `Each FS must cite exactly 1 GOAL.` | `numeric "exactly 1" is not canonical; accepted form: Each FS must cite exactly one GOAL.` |
| `FS-login.requirements must cite each AR at most 1 times.` | `numeric "at most 1 times" is not canonical; accepted form: FS-login.requirements must cite each AR at most once.` |
| `Each FS must cite at least one GOAL and must not cite any AR.` | `conjunctions are not accepted; accepted forms: "Each FS must cite at least one GOAL." and "Each FS must not cite any AR."` |
| `Each FS must cite at least one GOAL` | `rule must end with "."; accepted form: Each FS must cite at least one GOAL.` |
| `each FS must cite at least one GOAL.` | `fixed word "Each" is case-sensitive; accepted form: Each FS must cite at least one GOAL.` |
| `Each file in vendor/ must cite at least one FS.` | `path subjects are not accepted in phase 1`, and no rewrite |
| `Each */FS must cite at least one GOAL.` | `subject namespaces must be local in phase 1; accepted form: Each FS must cite at least one GOAL.` |
| `FS-login.* must cite at least one REQ.` | `section-component wildcards are not accepted in phase 1; accepted form: FS-login must cite at least one REQ.` |
| `Each chapter of each FS must cite at least one REQ.` | `chapter-quantified subjects are not accepted in phase 1; accepted form: Each FS must cite at least one REQ.` |
| `FS-login.2 must cite at least one REQ.` | `numbered chapter subjects can detach when headings move; accepted form: FS-login must cite at least one REQ.` |
| `FS-login.requirements must cite at least one REQ.` with named sections off | `named chapter subjects require [id] named_sections = true; accepted form after enabling it: FS-login.requirements must cite at least one REQ.` |
| `Each POLICY must cite at least one GOAL.` | `unknown kind "POLICY"`, and no rewrite |
| `Every AR must cite at least one FS.` | `quantifier "every" is not accepted; accepted form: Each AR must cite at least one FS.` |
| `The design chapter of every AR must cite at least one FS.` | `quantifier "every" is not accepted; accepted form: The design chapter of each AR must cite at least one FS.` |
| `AR-trace.design must cite every FS exactly once.` | `quantifier "every" is not accepted; accepted form: AR-trace.design must cite each FS exactly once.` |

`FS-missing must cite at least one GOAL.` is syntactically valid and therefore
is not a pre-scan refusal. After scanning it produces the exact resolution
message `literal subject FS-missing does not resolve` ([§FS-rules.4](FS-rules.md#4-validation-lifecycle)).

Five of these rows once printed a fixed rewrite that named a chapter, a
declaration or a kind the sentence never typed. The decision to replace those
rewrites in place, and every other fixed rewrite with one built from the
typed sentence, is
[§DF-rule-refusal-rewrites](../decisions/functional/DF-rule-refusal-rewrites.md#df-rule-refusal-rewrites-a-refused-rules-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced-in-place).

### sentences.every: Unsupported every is refused as a quantifier, with an each repair

`Every KIND`, `The NAME chapter of every KIND`, and `cite every KIND COUNT`
remain outside the closed grammar of [§FS-rules.2](FS-rules.md#2-subject-selectors)
and [§FS-rules.3.3](FS-rules.md#33-per-target-coverage). On both rule surfaces,
`check --rule` and a configured declaration's `invalid-rule` finding, their
reason is `quantifier "every" is not accepted`, rather than a count or literal
subject failure. The first two are subject refusals at stage 5; the last is a
predicate refusal at stage 6 ([§FS-rule-refusals.sentences.order](FS-rule-refusals.md#sentencesorder-a-sentence-is-refused-for-the-first-stage-it-fails)).

The repair replaces only that quantifier token: `Every` becomes `Each`, and
`every` becomes `each`. It keeps the typed subject, complete chapter path,
modality, valid count and object kind, including configured non-default kinds.
The whole sentence is offered only after the ordinary reparse accepts it
([§FS-rule-refusals.sentences.rewrite.reparsed](FS-rule-refusals.md#sentencesrewritereparsed-a-form-is-parsed-before-it-is-offered)); further refused productions
are repaired by that same contract, and missing pieces are never guessed.
In particular an unknown kind leaves no form and earns `known kinds:` on the
ad-hoc surface. Earlier stages retain their reasons, so a missing period,
conjunction or refused modality takes precedence over this subject refusal,
and a refused subject takes precedence over `cite every`.

On `list --selector`, the two subject spellings get the same quantifier reason
and the corresponding complete selector with `Each` or `each`, reparsed under
the same configuration before it is offered. This refusal adds no breadcrumb.
Where no configured kind can be recovered, no selector is offered and
`known kinds:` follows. Existing named-sections guidance still governs a repair
that needs named sections; this quantifier reason cannot label an offer as
accepted after enabling them.

Only these quantifier positions are covered. A genuine chapter name `every`,
a component such as `every.detail`, and an ID such as `SYS-every` remain intact,
including inside a repair of a different token. No synonym, case normalization,
new accepted sentence or selector, exit change, or evaluation change follows.

### sentences.whitespace: Presence-name whitespace refusal

A presence `NAME` containing whitespace anywhere, including internal spaces,
tabs, and Unicode whitespace, remains refused. The released reason remains a
verbatim contiguous prefix under [§FS-errors.3](FS-errors.md#3-message-text), and the explanation
`NAME forbids whitespace anywhere.` remains the end of the message. Between
them stands the accepted form [§FS-rule-refusals.sentences.rewrite](FS-rule-refusals.md#sentencesrewrite-an-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced) builds: the sentence as typed with
the `NAME` trimmed of its surrounding whitespace. An empty `NAME`, or one with
whitespace inside, leaves no token to keep, so no form is offered and only the
`; ` that ends every reason stands between them. The first line below refuses
`Each FS must have exactly one  requirements chapter.`, with two spaces, and
the second refuses `Each FS must have exactly one Goal and hypothesis chapter.`
and `Each FS must have exactly one  chapter.`:

```text
chapter name must be a non-empty NAME with no surrounding whitespace; accepted form: Each FS must have exactly one requirements chapter. NAME forbids whitespace anywhere.
chapter name must be a non-empty NAME with no surrounding whitespace; NAME forbids whitespace anywhere.
```

An empty `NAME` uses the same refusal. This adds guidance without admitting any
new grammar. An ad-hoc refusal still has the shape of
[§FS-errors.2.5](FS-errors.md#25-refused-input), and a missing piece here is a chapter name, never a
kind, so no `known kinds:` line follows it.

### sentences.named-sections: A subject that needs named sections is answered with one they make valid

Whether named sections are on is read under the project that configures the
subject's kind ([§FS-scan.projects.kinds](FS-scan.md#projectskinds-whose-kinds-a-run-reads)); for a rule subject that is the
project whose configuration reads the rule.

With `[id] named_sections = false`, a subject refused because named chapter
subjects require named sections is answered with a subject that turning them on
makes valid, so a reader who follows the suggestion is not refused again. It is
built from what was typed and from the configured kinds:

1. The subject is parsed again with named sections on. Where that parse accepts
   it, the suggestion is the subject as typed.
2. Otherwise the suggestion is what [§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector)'s four steps build for that
   second refusal, written as a rule subject. `KIND.NAME` is selector-only
   ([§FS-rules.2](FS-rules.md#2-subject-selectors)), so where a selector would get `KIND.NAME[.NAME…]` the rule gets
   `The NAME[.NAME…] chapter of each KIND`, and where a selector would get a
   bare `KIND` the rule gets `Each KIND`.
3. The label is `accepted form after enabling it:` only where the suggested
   subject is refused with the configuration as it is, and `accepted form:`
   where the repository accepts it as configured. Only a refusal whose reason
   is the named-sections one may carry the label. Where another part of the
   sentence is refused first and only the form built for it would need named
   sections, nothing is suggested, because the label would name nothing that
   reason asked to enable: with named sections off,
   `FS-login.requirements may not cite any AR.` is refused with
   `modality "may not" is not accepted` alone.
4. Where no configured kind can be recovered, or a configured ID grammar cannot
   be compiled with named sections on, nothing is suggested.
5. The reason stays the one [§FS-rule-refusals.sentences](FS-rule-refusals.md#sentences-strict-refusals) gives for the subject as configured,
   `named chapter subjects require [id] named_sections = true`. The modality
   and predicate are the ones the author typed, rewritten as
   [§FS-rule-refusals.sentences.rewrite](FS-rule-refusals.md#sentencesrewrite-an-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced) says where they are refused too, and where that point offers
   no form for them nothing is suggested. The label of step 3 is decided on the
   whole offered sentence, not on its subject alone.

So every sentence offered after `accepted form after enabling it:` is accepted
by `check --rule` once `named_sections = true`, and every sentence offered after
a plain `accepted form:` is accepted with the configuration as it is. Accepted
means not refused before the scan; whether a literal resolves stays the scan's
question ([§FS-rules.4](FS-rules.md#4-validation-lifecycle)).

Where nothing is suggested, the two rule surfaces differ, because only one of
them can print a second line:

- `check --rule` prints the reason alone and then, where what could not be
  supplied is a kind ([§FS-rule-refusals.sentences.rewrite](FS-rule-refusals.md#sentencesrewrite-an-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced)), a `known kinds:` line in the
  form an unknown `--kind` ends in, naming the whole set the subject accepts
  ([§FS-list.1.1](FS-list.md#11---kind)): the citable kinds of the project whose configuration reads the
  subject, once each, in configuration order. That is the root project at a
  workspace root and the member under a member path. In one project these are
  the two lines `list --selector` prints for the same subject ([§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector)).
  At a workspace root that selector lists every loaded project's kinds
  ([§FS-list.1.2](FS-list.md#12---project)), and the rule only the root's, because its subject accepts no
  other. The line has the shape of [§FS-errors.2.5](FS-errors.md#25-refused-input).
- A configured rule declaration's `invalid-rule` finding is the reason alone,
  `<RULE-ID> is not a valid rule: <reason>`, because a finding is one message
  on one line ([§FS-rules.7.1](FS-rules.md#71-invalid-rule)).

In a repository whose kinds are `FS`, `REQ` and `GOAL`, with named sections off
and `FS-login` holding a named `requirements` chapter with the sections
`requirements.1` and `requirements.2`, these are the exact `check --rule`
refusals. Each `error:` line opens with
`named chapter subjects require [id] named_sections = true`, and the table
gives what follows it.

| Refused sentence (`… must cite at least one REQ.`) | After the reason |
|---|---|
| `FS-login.requirements` | `; accepted form after enabling it: FS-login.requirements must cite at least one REQ.` |
| `The requirements chapter of each FS` | `; accepted form after enabling it: The requirements chapter of each FS must cite at least one REQ.` |
| `FS-*.requirements` | `; accepted form after enabling it: The requirements chapter of each FS must cite at least one REQ.` |
| `FS.requirements` | `; accepted form after enabling it: The requirements chapter of each FS must cite at least one REQ.` |
| `The requirements.1 chapter of each FS` | `; accepted form after enabling it: The requirements chapter of each FS must cite at least one REQ.` |
| `FS-login.Requirements` | `; accepted form: FS-login must cite at least one REQ.` |
| `The Requirements chapter of each FS` | `; accepted form: Each FS must cite at least one REQ.` |
| `The * chapter of each FS` | `; accepted form: Each FS must cite at least one REQ.` |
| `POLICY.requirements` | nothing on the `error:` line, then the line `known kinds: FS, REQ, GOAL` |

The first two rows are the subjects enabling named sections does make valid,
and they print what they printed before this point existed; the first is the
named-sections-off row of [§FS-rule-refusals.sentences](FS-rule-refusals.md#sentences-strict-refusals). A subject refused for its own production
before named sections are asked about, such as `FS-login.*`, `FS-login.2` or
`Each chapter of each FS`, keeps its [§FS-rule-refusals.sentences](FS-rule-refusals.md#sentences-strict-refusals) row. With named sections on,
nothing here applies. The decision to replace the as-typed suggestion in place
rather than append to it is
[§DF-rule-after-enabling-rewrites](../decisions/functional/DF-rule-after-enabling-rewrites.md#df-rule-after-enabling-rewrites-a-rule-subject-that-needs-named-sections-is-answered-with-one-they-make-valid-in-place).

### sentences.chapter-path: A chapter path is refused for the component that failed

A chapter subject refused for its chapter path opens with the reason that
names what is wrong with the path, whichever spelling reached it. So the
literal spelling `FS-login.requirements.1` and the spelling
`The requirements.1 chapter of each FS` are refused for the same thing. One
component decides the reason, by its shape:

| The component | The reason |
|---|---|
| all digits, such as `1` | `numbered chapter subjects can detach when headings move` |
| holding `*` | `section-component wildcards are not accepted in phase 1` |
| empty, left by a leading, doubled or trailing separator | `named chapter subject "<subject>" does not match the configured section grammar` |

An empty component is not a number. Which component decides depends on the
spelling:

- In `The PATH chapter of each KIND`, as in a selector's `KIND.PATH`, it is the
  first component of `PATH` that is not a named one ([§FS-rules.2](FS-rules.md#2-subject-selectors)). Any other
  shape, such as `Requirements`, gets the section-grammar reason, and so does a
  path of named components that the configured section grammar refuses.
- In the literal spelling `ID.PATH`, a `*` anywhere in the path is refused
  first. Otherwise it is the first component that is empty or all digits. A
  literal path with neither goes on through the rest of the parse, so with
  named sections on `FS-login.Requirements` is refused for its ID grammar.

The literal spelling decides this before it asks whether named sections are on,
as it decides a numbered or wildcard path ([§FS-rule-refusals.sentences.named-sections](FS-rule-refusals.md#sentencesnamed-sections-a-subject-that-needs-named-sections-is-answered-with-one-they-make-valid)). So with named
sections off `FS-login.requirements.` still gets the section-grammar reason,
while `The requirements.1 chapter of each FS`, which asks first, keeps its
named-sections-off refusal.

This point decides the reason alone. The accepted form after `; ` is the typed
sentence with its subject rebuilt as [§FS-rule-refusals.sentences.rewrite.recovered](FS-rule-refusals.md#sentencesrewriterecovered-only-what-can-be-recovered-is-supplied) says, the subject
`list --selector` gives the same path written as a rule subject.

In a repository whose kinds are `FS` and `REQ`, with named sections on and
`FS-login` holding a named `requirements` chapter, these are the exact refusals.
`check --rule` prints each after `error: `, and a configured rule declaration's
`invalid-rule` finding prints it after `is not a valid rule: ` ([§FS-rules.7.1](FS-rules.md#71-invalid-rule)).

| Refused sentence (`… must cite at least one REQ.`) | Exact reason and accepted form |
|---|---|
| `The requirements.1 chapter of each FS` | `numbered chapter subjects can detach when headings move; accepted form: The requirements chapter of each FS must cite at least one REQ.` |
| `The requirements.* chapter of each FS` | `section-component wildcards are not accepted in phase 1; accepted form: The requirements chapter of each FS must cite at least one REQ.` |
| `FS-login.requirements.` | `named chapter subject "FS-login.requirements." does not match the configured section grammar; accepted form: FS-login.requirements must cite at least one REQ.` |
| `FS-login..requirements` | `named chapter subject "FS-login..requirements" does not match the configured section grammar; accepted form: FS-login must cite at least one REQ.` |

In the same repository `The requirements chapter of each FS` and
`FS-login.requirements` are accepted. A selector is read by the same parse, so
`list --selector` names the same component ([§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector)). The decision to
correct these reasons in place is
[§DF-rule-refusal-reasons](../decisions/functional/DF-rule-refusal-reasons.md#df-rule-refusal-reasons-a-chapter-path-is-refused-for-the-component-that-failed-corrected-in-place).

### sentences.count-range: A count above the largest supported count is refused for its range

A count `N` ([§FS-rules.3](FS-rules.md#3-the-five-sentence-families)) is held in the platform's unsigned machine word, so
the largest supported count is that word's largest value,
`18446744073709551615` on a 64-bit platform, and that count is accepted. A
numeral whose value is larger is refused for its range rather than its
spelling: it may be canonical, and the reader is sent to the numeric domain,
not the grammar. The reason keeps the count refusal's text whole as its prefix,
so a consumer matching that text still matches ([§FS-errors.3](FS-errors.md#3-message-text)), and
appends the bound, printed from the platform's word rather than written down:

```text
count must be a canonical positive base-10 integer; the count is above the largest supported count, 18446744073709551615
```

Counts are not made arbitrary-precision: the accepted language is unchanged,
and only the refusal of a count it already refused names the cause.

#### sentences.count-range.shapes: Every count reads it

The refusal is the same for a floor, a ceiling and an exact count (`at least
N`, `at most N`, `exactly N`) and for a per-target count (`at least N times`,
`at most N times`, `exactly N times`), on both rule surfaces: `check --rule`
prints it in the shape of [§FS-errors.2.5](FS-errors.md#25-refused-input), and a configured rule declaration's
`invalid-rule` finding carries it after `is not a valid rule: `
([§FS-rules.7.1](FS-rules.md#71-invalid-rule)).

#### sentences.count-range.leading-zeros: Leading zeros do not hide it

The value is read after any leading zeros, so `018446744073709551616` is
refused for its range too, and a count with leading zeros whose value is in
range keeps the leading-zeros refusal and its form
([§FS-rule-refusals.sentences.rewrite.by-refusal](FS-rule-refusals.md#sentencesrewriteby-refusal-the-rewrite-each-refusal-makes)).

#### sentences.count-range.no-form: No form is offered

There is no count in range the sentence names, so the reason stands alone and
nothing follows it ([§FS-rule-refusals.sentences.rewrite.no-form](FS-rule-refusals.md#sentencesrewriteno-form-where-no-form-is-offered)).

### sentences.modal-chapter: A chapter named after a modal word is refused for that name

A chapter subject whose `NAME` is a modal word, as in
`The must chapter of each FS must cite at least one FS.`, is cut at that word:
[§FS-rules.3.6](FS-rules.md#36-where-a-sentences-subject-ends) ends the subject at the first modality it finds in its fixed order, and
the `NAME` is the first. The subject left is `The`, which stage 5 of
[§FS-rule-refusals.sentences.order](FS-rule-refusals.md#sentencesorder-a-sentence-is-refused-for-the-first-stage-it-fails) refuses for the ID grammar. That reason is true of the
word `The` and names nothing the author can fix, so it is followed by the one
that can:

```text
literal subject "The" does not match the configured ID grammar — chapter name "must" is read as the rule's modality, so rename the chapter
```

The test is on what was typed: the subject the split leaves is exactly `The`,
and the text after the modality opens `chapter of each `. The name quoted is the
modality the split took, as typed: `must`, `must not`, `should` or
`should not`. The split is not moved and no sentence it refuses today is
accepted: making such a chapter reachable by name would change what
[§FS-rules.3.6](FS-rules.md#36-where-a-sentences-subject-ends) means, and is not this point.

The released reason stays a verbatim contiguous prefix ([§FS-errors.3](FS-errors.md#3-message-text)) and the
rest is appended after ` — `, as a selector's `each FS` adds the rule's reason
([§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector)). The append moves no verdict, so no release is named.
Nothing in the sentence says what the chapter should be called, so no form is
offered ([§FS-rule-refusals.sentences.rewrite.no-form](FS-rule-refusals.md#sentencesrewriteno-form-where-no-form-is-offered)), and the `known kinds:` line that
followed the reason still follows it on `check --rule`. A configured rule
declaration's `invalid-rule` finding carries the same reason after
`is not a valid rule: ` ([§FS-rules.7.1](FS-rules.md#71-invalid-rule)). A selector that is such a sentence, or
that is such a subject with a two-word modal name, splits where the rule does
([§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector)), so `list --selector` gives the same reason, then its own
`known kinds:` line.

In the repository of [§FS-rule-refusals.sentences.rewrite.fs-only](FS-rule-refusals.md#sentencesrewritefs-only-exact-rows-where-fs-is-the-only-kind), whose one kind is `FS`,
these are the exact refusals. Each `error:` line opens with
`literal subject "The" does not match the configured ID grammar — `, the table
gives what follows it, and on `check --rule` and `list --selector` the line
`known kinds: FS` comes next:

| Refused input | After the released reason |
|---|---|
| `check --rule 'The must chapter of each FS must cite at least one FS.'` | `chapter name "must" is read as the rule's modality, so rename the chapter` |
| `check --rule 'The must chapter of each FS should cite at least one FS.'` | `chapter name "must" is read as the rule's modality, so rename the chapter` |
| `check --rule 'The should chapter of each FS should cite at least one FS.'` | `chapter name "should" is read as the rule's modality, so rename the chapter` |
| `check --rule 'The must not chapter of each FS should not cite any FS.'` | `chapter name "must not" is read as the rule's modality, so rename the chapter` |
| `check --rule 'The should not chapter of each FS should cite at least one FS.'` | `chapter name "should not" is read as the rule's modality, so rename the chapter` |
| `list --selector 'The should chapter of each FS should cite at least one FS.'` | `chapter name "should" is read as the rule's modality, so rename the chapter` |
| `list --selector 'The must not chapter of each FS'` | `chapter name "must not" is read as the rule's modality, so rename the chapter` |

In the same repository a split that reaches the name later is not refused,
because the modality it takes first is the rule's own:
`The should chapter of each FS must cite at least one FS.`,
`The must chapter of each FS should not cite any FS.` and
`The should chapter of each FS must not cite any FS.` are accepted, and so is
the selector `The must chapter of each FS`, which is a subject and is never
split.

### sentences.rewrite: An accepted form is the typed sentence with the failed part replaced

Every sentence a rule surface offers after `accepted form:`, `accepted forms:`
or `accepted form after enabling it:` is the sentence the author typed with the
production that failed rewritten, and pasting it back to `check --rule` is not
refused, in every repository and not only in one that happens to configure the
kinds an example names. Accepted means what it means in [§FS-rule-refusals.sentences.named-sections](FS-rule-refusals.md#sentencesnamed-sections-a-subject-that-needs-named-sections-is-answered-with-one-they-make-valid): not
refused before the scan, so whether a literal subject resolves stays the scan's
question ([§FS-rules.4](FS-rules.md#4-validation-lifecycle)). The sentence front end that owns the refusals builds the
form, so `check --rule` and a configured rule declaration's `invalid-rule`
finding offer the same one. The reason before the `;` is not part of the form
and does not change with it. A form keeps a namespaced object kind as typed, so
in a member with no workspace in scope it may be unverifiable here
([§FS-rules.4.1](FS-rules.md#41-a-rule-this-scope-cannot-verify)), which is not a refusal of the form.

#### sentences.rewrite.replaced: Only the failed part is replaced

The subject, modality, count and object kinds that did not fail are kept as
they were typed, and only the production that failed is spelled canonically. A
refusal never answers with a sentence about `FS`, `GOAL`, `REQ` or
`AR-overview.system-overview` that the author did not write.

#### sentences.rewrite.recovered: Only what can be recovered is supplied

A refused subject is rebuilt with [§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector)'s four steps and written as a rule
subject, the way [§FS-rule-refusals.sentences.named-sections](FS-rule-refusals.md#sentencesnamed-sections-a-subject-that-needs-named-sections-is-answered-with-one-they-make-valid) step 2 writes one: where a selector would get
`KIND.NAME[.NAME…]` the rule gets `The NAME[.NAME…] chapter of each KIND`, and
where a selector would get a bare `KIND` the rule gets `Each KIND`. An object
kind is never guessed. A chapter name is never invented: the front end reads
the configuration and not the scan, so a subject keeps the named components
typed before the refused one, and where none was typed it falls back to its
declaration or to `Each KIND`. So an empty component keeps what was typed
before it and nothing after it, as `list --selector` does: `FS-login..requirements`
is offered `FS-login` and `FS-login.requirements.` is offered
`FS-login.requirements`. A refused subject that holds a modality followed
by a verb is the first of two clauses the modality split read into the subject
([§FS-rules.3.6](FS-rules.md#36-where-a-sentences-subject-ends)), joined by anything but the ` and ` that [§FS-rule-refusals.sentences.conjunction](FS-rule-refusals.md#sentencesconjunction-any-clauses-joined-by-and-are-refused-as-a-conjunction) refuses first, so
it is not rebuilt: a form built from it would drop a clause. No form is offered, and `known kinds:` follows.

#### sentences.rewrite.reparsed: A form is parsed before it is offered

The form is parsed again with the same vocabulary. Where another part of it is
refused, that part is replaced the same way and the form is parsed again. Each
pass replaces one refused production, so the passes end within the number of
productions a sentence has, and a pass that would leave the sentence as it was
ends them with no form. A form is offered only once the parser accepts it.
Where a pass has nothing to put in, or more than one candidate, no form is
offered. A refusal that offers several forms offers them only where every one
is accepted.

#### sentences.rewrite.no-form: Where no form is offered

Where no form is offered, the refusal is its reason alone. That reason includes
the supplementary configuration guidance of [§FS-rule-refusals.sentences.rewrite.no-form.place](FS-rule-refusals.md#sentencesrewriteno-formplace-a-refused-place-subject-names-its-citation-configuration)
where applicable; the guidance is not an accepted sentence rewrite. On `check --rule`,
the `known kinds:` line [§FS-rule-refusals.sentences.named-sections](FS-rule-refusals.md#sentencesnamed-sections-a-subject-that-needs-named-sections-is-answered-with-one-they-make-valid) prints follows the reason where what
could not be supplied is a kind, a subject's or an object's, naming the same
set, in the shape of [§FS-errors.2.5](FS-errors.md#25-refused-input). Where what could not be supplied
is anything else, nothing follows the reason. A configured rule declaration's
`invalid-rule` finding ends at the reason, including that guidance where applicable
([§FS-rules.7.1](FS-rules.md#71-invalid-rule)).

##### sentences.rewrite.no-form.place: A refused place subject names its citation configuration

When `Each PLACE must cite at least one KIND.` names a configured, scanned,
non-citable row with a configured file or folder, its refusal additionally
identifies that place and tells the author to express the obligation in
`grund.toml`: `[rules.citations.PLACE]`
with `must = ["KIND"]` in version 2 ([§FS-config-v2.rules.citations](FS-config-v2.md#rulescitations-rulescitations)), or
`[citations.PLACE]` with the same entry in version 1 ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)).
The hint is supplied even when that citation entry has not been written yet.
It names the supported configuration route, never an `accepted form`, and
never recommends making the row citable. That entry alone judges only a file
there that already cites something ([§FS-check.3.11](FS-check.md#311-missing-required-citation)), so where the row grounds
nothing at any strength the hint also names the grounding setting the Citation
directions bullet names ([§FS-init.2.3.5.2.2](FS-init.md#23522-a-home-whose-row-grounds-nothing-says-so)), in the live version, as what
makes a file there that cites nothing a finding. Where the row grounds, the hint
names the citation entry alone.

`Each file in PATH must cite at least one KIND.` receives the same guidance
when `PATH` identifies exactly one such row's configured file or folder.
Matching is conservative: compare complete project-relative configured paths,
allowing a folder's terminal `/` to be present or absent. Do not infer a row
from an ancestor containing several places, a descendant, a substring, a glob,
an absolute path or a path containing `.` or `..` components. An unrelated or
ambiguous path receives no configuration hint; neither does an unscanned or
citable row. A row's name matches exactly, without guessing aliases.

This guidance is required only for the complete canonical predicate
`must cite at least one KIND`, with one local citable kind named exactly as
configured. Do not suggest this obligation for a presence, incoming-citation,
prohibition, other modality or count, disjunction, namespaced or unknown object, or an
otherwise malformed sentence. It cannot stand in for those predicates.

Both subject forms remain refused with exit `2` on `check --rule`. Their
existing reasons remain intact as prefixes: the path-subject reason stays
`path subjects are not accepted in phase 1`, and a named place retains the
shared non-citable-kind reason ([§FS-errors.2.5.1](FS-errors.md#251-a-kind-that-declares-no-ids)).
The `known kinds:` line still follows on that command. Refusal precedence
([§FS-rule-refusals.sentences.order](FS-rule-refusals.md#sentencesorder-a-sentence-is-refused-for-the-first-stage-it-fails)) is unchanged: guidance supplements a
place/path-subject refusal, never an earlier refusal. Rule declarations carry
the same guidance in their `invalid-rule` reason; unrelated object-kind and
selector errors retain their existing reasons without a place obligation.
One sentence-specific derivation supplies the guidance for both subject
spellings, rather than changing the shared kind reason on other surfaces.

#### sentences.rewrite.by-refusal: The rewrite each refusal makes

| Refusal | What the form replaces | No form when |
|---|---|---|
| modality `may not` | `must not` | — |
| `cite no` | `<modality> not cite any` | — |
| lowercase `each` | `Each` | — |
| unsupported `Every` or `every` ([§FS-rule-refusals.sentences.every](FS-rule-refusals.md#sentencesevery-unsupported-every-is-refused-as-a-quantifier-with-an-each-repair)) | only the quantifier token, with `Each` or `each` in the same position | the ordinary reparse cannot build an accepted form; `known kinds:` follows only where a kind could not be supplied |
| no terminal `.` | appends `.` | — |
| numeric `at least 1`, `exactly 1` | `at least one`, `exactly one` | — |
| numeric `at least 1 times`, `at most 1 times`, `exactly 1 times` | `at least once`, `at most once`, `exactly once` | — |
| a per-target count without `times` | the count typed after the bound, then `times` | that count is not a numeral |
| a count with leading zeros | the count without them | the count is zero, is not a numeral, or is above the largest supported count |
| a count above the largest supported count ([§FS-rule-refusals.sentences.count-range](FS-rule-refusals.md#sentencescount-range-a-count-above-the-largest-supported-count-is-refused-for-its-range)) | — | always |
| quantifier `a` | two forms, `at least one` and `exactly one` | either form is refused |
| clauses joined by `and` ([§FS-rule-refusals.sentences.conjunction](FS-rule-refusals.md#sentencesconjunction-any-clauses-joined-by-and-are-refused-as-a-conjunction)) | one form per clause | any form is refused |
| a prohibition without `cite any` | `cite [count] KINDS` becomes `cite any KINDS` | the predicate is anything else |
| a presence object not ending in `chapter` | its last word becomes `chapter` or `chapters`, as the count takes | no word precedes it |
| a presence noun of the wrong number | `chapter` or `chapters`, as the count takes | — |
| a presence `NAME` with whitespace ([§FS-rule-refusals.sentences.whitespace](FS-rule-refusals.md#sentenceswhitespace-presence-name-whitespace-refusal)) | the `NAME` trimmed of its surrounding whitespace | the `NAME` is empty or has whitespace inside |
| a chapter subject in a presence rule | the subject's declaration, or `Each KIND` | — |
| a refused subject: an unknown kind, `*/`, a section-component wildcard, a numbered chapter, the section grammar, the ID grammar, `Each chapter of each` | the subject, rebuilt as [§FS-rule-refusals.sentences.rewrite.recovered](FS-rule-refusals.md#sentencesrewriterecovered-only-what-can-be-recovered-is-supplied) says | no configured kind is recovered, or the subject holds a clause; `known kinds:` follows |
| the ID grammar for a subject `The` cut short at a chapter named after a modal word ([§FS-rule-refusals.sentences.modal-chapter](FS-rule-refusals.md#sentencesmodal-chapter-a-chapter-named-after-a-modal-word-is-refused-for-that-name)) | — | always; `known kinds:` follows |
| a path subject, `Each file in …` | — | always; `known kinds:` follows |
| an unknown or malformed object kind | — | always; `known kinds:` follows |
| no modality, an unknown verb, a count that is not accepted, a count with no object | — | always |

Every row keeps the subject and modality as typed, so the presence, per-target
and count refusals no longer answer with `Each FS must` or
`AR-overview.system-overview` whatever was typed. A refusal whose reason
appends guidance keeps it where no form is offered: `count is not accepted`
keeps its list of the canonical counts, and the [§FS-rule-refusals.sentences.whitespace](FS-rule-refusals.md#sentenceswhitespace-presence-name-whitespace-refusal) refusal keeps
`NAME forbids whitespace anywhere.`.

The per-target upper-one refusal is immediate, with no alias or warning
window ([§DF-per-target-once-ceiling](../decisions/functional/DF-per-target-once-ceiling.md#df-per-target-once-ceiling-spell-a-per-target-ceiling-of-one-as-at-most-once)). The missing-`times` and leading-zero
rows still name their own failed production, but their forms are reparsed
([§FS-rule-refusals.sentences.rewrite.reparsed](FS-rule-refusals.md#sentencesrewritereparsed-a-form-is-parsed-before-it-is-offered)): `at most 1`, `at most 1 time`
and `at most 01 times` each end up offering `at most once`, preserving the
typed subject, object kind and modality. This does not change chapter,
ordinary outbound or inbound upper-one counts ([§FS-rules.3](FS-rules.md#3-the-five-sentence-families)).

#### sentences.rewrite.fs-only: Exact rows where `FS` is the only kind

In the repository of [§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector)'s first table, whose one kind is `FS`, with
named sections on and `FS-login` holding named `requirements` and `should`
chapters, these are the exact `check --rule` refusals. Each is printed in the
shape of [§FS-errors.2.5](FS-errors.md#25-refused-input), and its `error:` line is the reason and what follows it. A
configured rule declaration's finding carries the same text after
`is not a valid rule: `, and never the line after it.

| Refused sentence | Exact reason and accepted form(s) | Line after it |
|---|---|---|
| `Each FS may not cite any FS.` | `modality "may not" is not accepted; accepted form: Each FS must not cite any FS.` | none |
| `Each FS should cite no FS.` | `"cite no" is not accepted; accepted form: Each FS should not cite any FS.` | none |
| `each FS must cite at least one FS.` | `fixed word "Each" is case-sensitive; accepted form: Each FS must cite at least one FS.` | none |
| `Each FS must cite at least 1 FS` | `rule must end with "."; accepted form: Each FS must cite at least one FS.` | none |
| `Each FS must cite at least 1 FS.` | `numeric "at least 1" is not canonical; accepted form: Each FS must cite at least one FS.` | none |
| `Each FS should be cited by exactly 1 FS.` | `numeric "exactly 1" is not canonical; accepted form: Each FS should be cited by exactly one FS.` | none |
| `FS-login.requirements must cite each FS at least 1 times.` | `numeric "at least 1 times" is not canonical; accepted form: FS-login.requirements must cite each FS at least once.` | none |
| `FS-login.requirements should cite each FS at most 1 times.` | `numeric "at most 1 times" is not canonical; accepted form: FS-login.requirements should cite each FS at most once.` | none |
| `FS-login must cite each FS exactly 2.` | `per-target counts must end in "times"; accepted form: FS-login must cite each FS exactly 2 times.` | none |
| `Each FS must cite exactly 02 FS.` | `count must be a canonical positive base-10 integer; accepted form: Each FS must cite exactly 2 FS.` | none |
| `Each FS must cite a FS.` | `quantifier "a" is ambiguous; accepted forms: "Each FS must cite at least one FS." or "Each FS must cite exactly one FS."` | none |
| `Each FS must cite at least one GOAL and must not cite any AR.` | `conjunctions are not accepted` | `known kinds: FS` |
| `Each FS must cite at least one FS and must not cite any FS.` | `conjunctions are not accepted; accepted forms: "Each FS must cite at least one FS." and "Each FS must not cite any FS."` | none |
| `Each FS must cite at least one FS or must not cite any FS.` | `unknown kind "FS must cite at least one FS or"` | `known kinds: FS` |
| `Each FS should not cite at least one FS.` | `a prohibition must use "cite any"; accepted form: Each FS should not cite any FS.` | none |
| `FS-login must have exactly one requirements section.` | `chapter presence must end in "chapter"; accepted form: FS-login must have exactly one requirements chapter.` | none |
| `FS-login should have at most 2 requirements chapter.` | `chapter count has the wrong singular/plural spelling; accepted form: FS-login should have at most 2 requirements chapters.` | none |
| `FS-login must have exactly one  requirements chapter.` | `chapter name must be a non-empty NAME with no surrounding whitespace; accepted form: FS-login must have exactly one requirements chapter. NAME forbids whitespace anywhere.` | none |
| `FS-login must have exactly one Goal and hypothesis chapter.` | `chapter name must be a non-empty NAME with no surrounding whitespace; NAME forbids whitespace anywhere.` | none |
| `FS-login.requirements must have exactly one should chapter.` | `chapter subjects cannot have chapters; accepted form: FS-login must have exactly one should chapter.` | none |
| `The requirements chapter of each FS should have at least one should chapter.` | `chapter subjects cannot have chapters; accepted form: Each FS should have at least one should chapter.` | none |
| `FS-*.requirements must cite at least one FS.` | `literal subject "FS-*.requirements" does not match the configured ID grammar; accepted form: The requirements chapter of each FS must cite at least one FS.` | none |
| `FS.* must cite at least one FS.` | `section-component wildcards are not accepted in phase 1; accepted form: Each FS must cite at least one FS.` | none |
| `FS-login.2 should cite at most 2 FS.` | `numbered chapter subjects can detach when headings move; accepted form: FS-login should cite at most 2 FS.` | none |
| `Each chapter of each FS must be cited by at least one FS.` | `chapter-quantified subjects are not accepted in phase 1; accepted form: Each FS must be cited by at least one FS.` | none |
| `Each */FS must cite at least one FS.` | `subject namespaces must be local in phase 1; accepted form: Each FS must cite at least one FS.` | none |
| `Each POLICY must cite at least one FS.` | `unknown kind "POLICY"` | `known kinds: FS` |
| `Each file in vendor/ must cite at least one FS.` | `path subjects are not accepted in phase 1` | `known kinds: FS` |
| `Each FS must cite at least one POLICY.` | `unknown kind "POLICY"` | `known kinds: FS` |
| `FS-*.requirements must cite at least one GOAL.` | `literal subject "FS-*.requirements" does not match the configured ID grammar` | `known kinds: FS` |
| `The must chapter of each FS must cite at least one FS.` | `literal subject "The" does not match the configured ID grammar — chapter name "must" is read as the rule's modality, so rename the chapter` | `known kinds: FS` |
| `Each FS cites at least one FS.` | `rule has no accepted modality` | none |
| `Each FS must reference at least one FS.` | `verb is not accepted` | none |
| `Each FS must cite some FS.` | `count is not accepted; the canonical counts are "at least one", "at least N", "at most N", "exactly one" and "exactly N" for a base-10 N` | none |
| `Each FS must cite at least two FS.` | `count must be a canonical positive base-10 integer` | none |
| `Each FS must cite at least 18446744073709551616 FS.` | `count must be a canonical positive base-10 integer; the count is above the largest supported count, 18446744073709551615` | none |

The fourth row is offered a form only after a second pass: appending the `.`
leaves `at least 1`, which the next pass rewrites. The conjunction and the last
`FS-*.requirements` row are offered none for the opposite reason: the first
pass rebuilds what failed, and the form it leaves names a kind this repository
does not configure, which no pass may guess.

### sentences.conjunction: Any clauses joined by `and` are refused as a conjunction

The table's conjunction row is one instance of a general refusal, not the only
sentence it covers: one sentence states one rule ([§FS-rules.3](FS-rules.md#3-the-five-sentence-families)), so a predicate
that joins clauses is refused as a conjunction, whatever its subject, counts
and kinds. A sentence is a conjunction when it contains ` and ` immediately
followed by a modality word, `must ` or `should `. Every such ` and ` ends one
clause and starts the next, which begins with its own modality. No accepted
sentence contains one: a chapter `NAME` is a single token and an object joins
its kinds with ` or `, so `The and chapter of each FS must cite at least one
FS.` stays a sentence.

The reason is `conjunctions are not accepted`. Its forms are one sentence per
clause: the first is the first clause, and each later one is the first clause's
subject, read by [§FS-rules.3.6](FS-rules.md#36-where-a-sentences-subject-ends), followed by that clause, each ending in `.`. Each form
is then built and read again as [§FS-rule-refusals.sentences.rewrite](FS-rule-refusals.md#sentencesrewrite-an-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced) says, and they are offered only
where every one is accepted, after `; accepted forms: `, quoted, two joined as
`"A." and "B."` and more as `"A.", "B.", and "C."`. Where any one is not, the
reason stands alone ([§FS-rule-refusals.sentences.rewrite.no-form](FS-rule-refusals.md#sentencesrewriteno-form-where-no-form-is-offered)).

Where the check stands among the other refusals is
[§FS-rule-refusals.sentences.order](FS-rule-refusals.md#sentencesorder-a-sentence-is-refused-for-the-first-stage-it-fails)'s third row: after the terminal `.` and the
fixed-word and phase-1 subject refusals, and before the modality split of
[§FS-rules.3.6](FS-rules.md#36-where-a-sentences-subject-ends). So the subject is never read from a
fragment, and no refusal names part of a sentence as a kind. Both rule surfaces
carry the same reason: `check --rule` prints `error: <reason>` in the shape
of [§FS-errors.2.5](FS-errors.md#25-refused-input), and a configured rule declaration's
finding is `<RULE-ID> is not a valid rule: <reason>` ([§FS-rules.7.1](FS-rules.md#71-invalid-rule)).

In a repository whose one kind is `FS`, these are the exact reasons. On
`check --rule` the line after it follows the reason, and a configured rule
declaration's finding carries the reason alone:

| Refused sentence | Exact reason | Line after it |
|---|---|---|
| `Each FS must cite at least one FS and must not cite any FS.` | `conjunctions are not accepted; accepted forms: "Each FS must cite at least one FS." and "Each FS must not cite any FS."` | none |
| `Each FS must cite exactly one FS and must not cite any FS.` | `conjunctions are not accepted; accepted forms: "Each FS must cite exactly one FS." and "Each FS must not cite any FS."` | none |
| `Each FS must cite at least one FS and must cite at most one FS.` | `conjunctions are not accepted` | none |
| `Each FS must not cite any FS and must cite at least one FS.` | `conjunctions are not accepted; accepted forms: "Each FS must not cite any FS." and "Each FS must cite at least one FS."` | none |
| `The requirements chapter of each FS must cite at least one FS and should not cite any FS.` | `conjunctions are not accepted; accepted forms: "The requirements chapter of each FS must cite at least one FS." and "The requirements chapter of each FS should not cite any FS."` | none |
| `Each FS must cite at least one FS and must not cite any FS and should cite at most 2 FS.` | `conjunctions are not accepted; accepted forms: "Each FS must cite at least one FS.", "Each FS must not cite any FS.", and "Each FS should cite at most 2 FS."` | none |
| `Each FS must cite at least one GOAL and must not cite any AR.` | `conjunctions are not accepted` | `known kinds: FS` |
| `Each */FS must cite at least one FS and must not cite any FS.` | `subject namespaces must be local in phase 1` | `known kinds: FS` |

The third row's second clause spells a ceiling of one as a word, which
[§FS-rules.3](FS-rules.md#3-the-five-sentence-families) refuses with no form to put in its place, so the pair is not offered;
the sentence is still answered as a conjunction, because the conjunction is
the first production it fails. The seventh row's forms name kinds this
repository does not configure. The last row is refused first for its phase-1
subject, which holds the first clause and so is not rebuilt
([§FS-rule-refusals.sentences.rewrite.recovered](FS-rule-refusals.md#sentencesrewriterecovered-only-what-can-be-recovered-is-supplied)).

### sentences.order: A sentence is refused for the first stage it fails

A sentence is read in one fixed order of stages, and it is refused for the
first stage it fails; no later stage is read. The order is the one the
documented rows already imply ([§DF-one-answer-rulings-grammar](../decisions/functional/DF-one-answer-rulings-grammar.md#df-one-answer-rulings-grammar-one-grammar-keeps-the-documented-refusal-bytes-and-order-and-one-kind-universe-keeps-each-commands-vocabulary), D20), held here
as one table so it does not depend on how one phrase of another point is read:

| Stage | What it refuses | Example that also fails a later stage | Reason it opens with |
|---|---|---|---|
| 1. terminal period | a sentence that does not end in `.` | `Each FS may not cite any FS` | `rule must end with "."` |
| 2. fixed word and phase-1 subject | `each` for `Each`, and the deliberate phase-1 absences of [§FS-rules.12](FS-rules.md#12-deliberate-phase-1-absences): a path subject (`Each file in`), `Each */KIND`, `Each chapter of each KIND`, and a section-component wildcard in the first clause's subject, in any spelling | `FS-x.* must cite at least one FS and must not cite any FS.` | `section-component wildcards are not accepted in phase 1` |
| 3. conjunction | clauses joined by `and` ([§FS-rule-refusals.sentences.conjunction](FS-rule-refusals.md#sentencesconjunction-any-clauses-joined-by-and-are-refused-as-a-conjunction)) | `Each FS may not cite any FS and must cite at least one FS.` | `conjunctions are not accepted` |
| 4. modality | no accepted modality, `may not` included | `Each POLICY may not cite any FS.` | `modality "may not" is not accepted` |
| 5. the rest of the subject | unsupported `Every KIND` or `The NAME chapter of every KIND`, an unknown or non-citable kind, named sections off, the section grammar, a numbered chapter, the ID grammar, a chapter named after a modal word ([§FS-rule-refusals.sentences.modal-chapter](FS-rule-refusals.md#sentencesmodal-chapter-a-chapter-named-after-a-modal-word-is-refused-for-that-name)) | `Each POLICY must cite no FS.` | `unknown kind "POLICY"` |
| 6. the predicate | the verb, quantifier (including `cite every`), count (including per-target numeric `at most 1 times`) and object kind, and a chapter subject that would have chapters | `Each FS must cite no FS.` | `"cite no" is not accepted` |

The examples are read in a repository whose one kind is `FS`, with named
sections on. Two sentences the table of [§FS-rule-refusals.sentences](FS-rule-refusals.md#sentences-strict-refusals) gives whole,
`Each FS must cite no AR.` and `Each FS must cite a GOAL.`, are refused for that
row before stage 1; both end in `.`, so no sentence can tell.

Per-target numeric upper-one is a predicate-stage refusal. Missing terminal
periods, fixed-word and phase-1 subject refusals, conjunctions, modalities and
subject validation still win before it. Repairing their offered forms may also
repair the count ([§FS-rule-refusals.sentences.rewrite.reparsed](FS-rule-refusals.md#sentencesrewritereparsed-a-form-is-parsed-before-it-is-offered)), without
changing which reason opens the refusal.

Stage 2 reads its rows from the sentence's start. A path subject is `Each
file in`, refused whatever follows; `Each docs/fs/` names a kind, refused in
stage 5. The wildcard is read from the subject of the first clause, the text
before the first `and` that joins clauses ([§FS-rule-refusals.sentences.conjunction](FS-rule-refusals.md#sentencesconjunction-any-clauses-joined-by-and-are-refused-as-a-conjunction)), so that row is read only where an
accepted modality splits that subject off: `FS-x.* may not cite any FS.` is
refused for its modality, as before. Stage 2 decides a wildcard with the
subject parser's own test, so a sentence with no conjunction is refused exactly
as before, and a subject that reaches a wildcard only through a component that
fails earlier in stage 5 — `The * chapter of each FS` with named sections off —
keeps that component's reason. A numbered chapter, named sections off and every
kind refusal stay in stage 5, after the conjunction, so a refusal never names a
fragment of a sentence as a kind.

Stage 2's wildcard row is the one place this order differs from what a
sentence was refused with before it was written down: a wildcard subject
joined by `and` was refused as a conjunction. That reason moved through the
window of [§FS-errors.3.8](FS-errors.md#38-the-one-grammar-wording-migration), which kept the conjunction refusal as a prefix until
`0.19.0`; since then the wildcard reason stands alone.

## selectors: A refused selector is answered with a selector

Every `list` mode that takes `--selector`, `--size` included, refuses a
selector it cannot accept. Its `error:` line is the refused-input shape of
[§FS-errors.2.5](FS-errors.md#25-refused-input): it names the
production that failed and ends in exactly one accepted selector, which a
reader can paste back into `--selector`.

The reason is the one [§FS-rule-refusals.sentences](FS-rule-refusals.md#sentences-strict-refusals) gives for the same subject wherever that
reason is true, so a rule and a selector never disagree about what failed.
Five refusals name the true failure in place of a grammar or vocabulary
mismatch. A numbered component reached through `KIND.NAME` or
`The NAME chapter of each KIND` gets the numbered reason the ID spelling gets.
A wildcard component reached the same way gets the wildcard reason.
`Each chapter of each KIND` gets the chapter-quantified reason, not an unknown
kind. `Each */KIND` gets the namespace reason the rule sentence gets, not an
unknown kind. An empty component, left by a doubled or trailing separator, is
not a number in any spelling, so it gets the section-grammar reason and no
breadcrumb ([§FS-rule-refusals.sentences.chapter-path](FS-rule-refusals.md#sentenceschapter-path-a-chapter-path-is-refused-for-the-component-that-failed)).
`Every KIND` and `The NAME chapter of every KIND` get the unsupported-quantifier
reason and token-only repair of [§FS-rule-refusals.sentences.every](FS-rule-refusals.md#sentencesevery-unsupported-every-is-refused-as-a-quantifier-with-an-each-repair), rather than a literal-ID mismatch.

A fixed word in the wrong case, as in `each FS`, keeps the reason the selector
has always printed and adds the rule's. The literal-subject reason stays a
verbatim contiguous prefix ([§FS-errors.3](FS-errors.md#3-message-text)), followed by ` — ` and the rule's
`fixed word "Each" is case-sensitive`, and the `known kinds:` line still
follows it. The append moves no verdict, so no release is named
([§DF-one-answer-rulings-grammar](../decisions/functional/DF-one-answer-rulings-grammar.md#df-one-answer-rulings-grammar-one-grammar-keeps-the-documented-refusal-bytes-and-order-and-one-kind-universe-keeps-each-commands-vocabulary), G06).

The suggestion is built from what was typed and from the configured kinds,
never from a fixed template:

1. Its kind is the configured kind the text names or starts with: `FS-*` and
   `FS-login` both give `FS`.
2. A declaration written as a valid ID is kept.
3. The longest run of named components before the first refused one is kept:
   `requirements.1` gives `requirements`, and `*` gives nothing.
4. It keeps the spelling that was typed. `KIND[.NAME…]` and `Each KIND` keep
   theirs. An ID stays an ID when step 2 kept its declaration, and otherwise
   becomes `KIND[.NAME…]`. `The NAME chapter of each KIND` keeps its spelling
   while a `NAME` survives step 3, and otherwise becomes `Each KIND`.

With named sections off, a selector refused for needing them is suggested what
the same selector would be suggested with them on. Where enabling them makes it
valid, that is the selector as typed. Where it would still be refused, it is
the selector the four steps build for that refusal, and where no configured
kind is recovered nothing is suggested. The label says whether pasting it back needs
the change: `accepted selector after enabling it:` only where the suggestion
itself needs named sections, and `accepted selector:` where the repository
accepts it as configured. The reason stays the one [§FS-rule-refusals.sentences](FS-rule-refusals.md#sentences-strict-refusals) gives for
the subject as configured, that named chapter subjects require named sections,
so these refusals add no breadcrumb.

Where no configured kind can be recovered, nothing is guessed. The reason
stands alone, and a second line lists the configured citable kinds exactly as
an unknown `--kind` does ([§FS-list.1.1](FS-list.md#11---kind)). A kind that is configured but not citable is not a
kind that cannot be recovered: its reason is [§FS-errors.2.5.1](FS-errors.md#251-a-kind-that-declares-no-ids)'s, alone since
the window of [§FS-errors.3.8](FS-errors.md#38-the-one-grammar-wording-migration) closed, and the line after it is the same
`known kinds:` line.

Which kinds a selector accepts, and whether `KIND.NAME` needs named sections,
are both read in the selected projects as [§FS-scan.projects.kinds](FS-scan.md#projectskinds-whose-kinds-a-run-reads) says, the
second under the project that configures `KIND`. At a workspace root whose own
config leaves named sections off and whose member `api` turns them on,
`list --selector API.terms` lists `api/API-a.terms` exactly as
`grund api/API-a.terms` shows it, while `list --project root --selector
API.terms`, whose scope configures no `API`, is refused as it was. A refusal that earns the breadcrumb
below prints it as a third line, after the kinds:

```text
error: <reason>
known kinds: <kinds>
hint: grund show --batch --toc expands each selected unit into its sections
```

A refusal never suggests a rule sentence. A selector that is itself one, a
subject followed by a modality, is refused for being one. Its subject is the
one the rule sentence has, ending where [§FS-rules.3.6](FS-rules.md#36-where-a-sentences-subject-ends) ends it, so a selector and
`check --rule` read the same subject from the same sentence. Its suggestion is
that subject, the text before the modality, and when the subject would itself
be refused the refusal is the subject's own.

The refusals whose reason names a numbered chapter, a section-component
wildcard or a chapter quantifier add one stderr breadcrumb
([§FS-errors.1.2](FS-errors.md#12-what-stderr-carries)), whether or not a kind was recovered. The units they reach for are
found by expanding the chapter the suggestion selects, or, where nothing is
suggested, the units of a kind the `known kinds:` line names:

```text
hint: grund show --batch --toc expands each selected unit into its sections
```

In a repository whose one kind is `FS`, with named sections on and `FS-login`
holding named `requirements` and `should` chapters, these are the exact
refusals. A line marked *hint* is the breadcrumb above.

| Refused selector | Exact reason and accepted selector | Lines after it |
|---|---|---|
| `FS-*.requirements` | `literal subject "FS-*.requirements" does not match the configured ID grammar; accepted selector: FS.requirements` | none |
| `FS.requirements.1` | `numbered chapter subjects can detach when headings move; accepted selector: FS.requirements` | hint |
| `FS-login.requirements.1` | `numbered chapter subjects can detach when headings move; accepted selector: FS-login.requirements` | hint |
| `FS-login.requirements.` | `named chapter subject "FS-login.requirements." does not match the configured section grammar; accepted selector: FS-login.requirements` | none |
| `The requirements.1 chapter of each FS` | `numbered chapter subjects can detach when headings move; accepted selector: The requirements chapter of each FS` | hint |
| `FS.*` | `section-component wildcards are not accepted in phase 1; accepted selector: FS` | hint |
| `FS-login.*` | `section-component wildcards are not accepted in phase 1; accepted selector: FS-login` | hint |
| `Each chapter of each FS` | `chapter-quantified subjects are not accepted in phase 1; accepted selector: Each FS` | hint |
| `*/FS` | `subject namespaces must be local in phase 1; accepted selector: FS` | none |
| `Each */FS` | `subject namespaces must be local in phase 1; accepted selector: Each FS` | none |
| `Every FS` | `quantifier "every" is not accepted; accepted selector: Each FS` | none |
| `The requirements chapter of every FS` | `quantifier "every" is not accepted; accepted selector: The requirements chapter of each FS` | none |
| `FS.Requirements` | `named chapter subject "FS.Requirements" does not match the configured section grammar; accepted selector: FS` | none |
| `Each POLICY` | `unknown kind "POLICY"` | `known kinds: FS` |
| `POLICY.requirements` | `literal subject "POLICY.requirements" does not match the configured ID grammar` | `known kinds: FS` |
| `requirements` | `literal subject "requirements" does not match the configured ID grammar` | `known kinds: FS` |
| `each FS` | `literal subject "each FS" does not match the configured ID grammar — fixed word "Each" is case-sensitive` | `known kinds: FS` |
| `Each chapter of each POLICY` | `chapter-quantified subjects are not accepted in phase 1` | `known kinds: FS`, then hint |
| `API.requirements.1` | `numbered chapter subjects can detach when headings move` | `known kinds: FS`, then hint |
| `FS-login must cite at least one GOAL.` | `a rule sentence is not a selector; accepted selector: FS-login` | none |
| `The should chapter of each FS must cite at least one FS.` | `a rule sentence is not a selector; accepted selector: The should chapter of each FS` | none |
| `The should chapter of each FS should cite at least one FS.` | `literal subject "The" does not match the configured ID grammar — chapter name "should" is read as the rule's modality, so rename the chapter` | `known kinds: FS` |

With named sections off in the same repository, these are the exact refusals.
None of them earns the breadcrumb.

| Refused selector | Exact reason and accepted selector | Lines after it |
|---|---|---|
| `FS.requirements` | `named chapter subjects require [id] named_sections = true; accepted selector after enabling it: FS.requirements` | none |
| `FS-*.requirements` | `named chapter subjects require [id] named_sections = true; accepted selector after enabling it: FS.requirements` | none |
| `FS.requirements.1` | `named chapter subjects require [id] named_sections = true; accepted selector after enabling it: FS.requirements` | none |
| `FS.*` | `named chapter subjects require [id] named_sections = true; accepted selector: FS` | none |
| `The * chapter of each FS` | `named chapter subjects require [id] named_sections = true; accepted selector: Each FS` | none |
| `POLICY.requirements` | `named chapter subjects require [id] named_sections = true` | `known kinds: FS` |

An exact literal that does not resolve or is ambiguous keeps its refusal,
which suggests nothing. The rule surfaces, `check --rule` and a configured rule
declaration, answer a refused subject with a rule subject the same four steps
build, under the modality and predicate the author typed
([§FS-rule-refusals.sentences.rewrite.recovered](FS-rule-refusals.md#sentencesrewriterecovered-only-what-can-be-recovered-is-supplied)), and with named sections off as [§FS-rule-refusals.sentences.named-sections](FS-rule-refusals.md#sentencesnamed-sections-a-subject-that-needs-named-sections-is-answered-with-one-they-make-valid) says. A
chapter subject refused for a component of its path opens with the reason a
selector gets for the same component, as [§FS-rule-refusals.sentences.chapter-path](FS-rule-refusals.md#sentenceschapter-path-a-chapter-path-is-refused-for-the-component-that-failed) says. So a
selector and a rule refused for the same subject suggest the same units, each
in its own spelling: `FS-*.requirements` is answered with `FS.requirements` as
a selector and with `The requirements chapter of each FS` in a rule. The
decision to replace these lines rather than append to them is
[§DF-selector-refusal-rewrites](../decisions/functional/DF-selector-refusal-rewrites.md#df-selector-refusal-rewrites-a-refused-selector-is-answered-with-a-selector-and-its-old-lines-are-replaced-not-appended-to), and for the rule surfaces
[§DF-rule-refusal-rewrites](../decisions/functional/DF-rule-refusal-rewrites.md#df-rule-refusal-rewrites-a-refused-rules-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced-in-place).
