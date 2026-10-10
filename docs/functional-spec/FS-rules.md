# FS-rules: grounded declarations state and enforce chapter rules

A repository may state chapter and citation constraints once as grounded rule
declarations, have `grund check` enforce them, and render the same sentences to
agents before they write. This closes the checked-guidance loop of
[§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work) while keeping the strict, deterministic, fixed-severity behavior of the
existing checker. The product decision is
[§DF-chapter-rules](../decisions/functional/DF-chapter-rules.md#df-chapter-rules-chapter-rules-are-grounded-controlled-english-declarations-over-producer-neutral-facts).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, citable, body, section, coordinate, catalog),
[§FS-terms.terms.2](FS-terms.md#terms2-citations) (marker, citation, shorthand, citation site), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, alias),
[§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, severity, suggestion, verdict), and [§FS-terms.terms.6](FS-terms.md#terms6-rules-and-directions) (direction,
level, rule, grounded).

- **subject** — The authored phrase that opens a rule sentence and names the units the rule is
  about.
- **selector** — What a subject or target spelling matches: a kind, a named chapter, or one
  exact ID.
- **chapter** — A rule's unit inside a declaration: one named heading in its body, direct or
  nested, as against the declaration's whole body. The rule grammar's subject-unit word.
- **display name** — The label an author wrote after a chapter's coordinate: `Terms` in
  `## terms: Terms`. What a presence rule names, whatever file the heading is in
  ([§FS-rules.5.1.1](FS-rules.md#511-a-chapters-display-name-is-the-label-its-author-wrote)).
- **family** — One accepted rule grammar. A sentence outside every family is refused rather than
  reinterpreted.
- **unverifiable here** — A rule sentence every component of which is well-formed, whose
  object alias the scope the command ran in holds no vocabulary for at all. Not an
  invalid rule ([§FS-rules.4.1](FS-rules.md#41-a-rule-this-scope-cannot-verify)).
- **facts** — The `RuleFacts` an evaluator reads: the versioned, immutable record of the units,
  chapters and counts one scan produced.

## 1. Rule declarations and opt-in

One or more citable Markdown kinds may set the optional Boolean
`rules = true` on their `[[kinds]]` rows
([§FS-config.3.4.12](FS-config.md#3412-rules--rule-declaration-kinds)). Every
declaration of an enabled kind is one rule: its ID identifies the authority,
its title is the executable sentence, and its body is a non-empty rationale
scanned, cited, checked, and formatted as ordinary Markdown. A repository that
wants rule rationales to cite goals says so with the existing
`[citations.<rule-kind>]` surface; no kind name is hard-coded.

The rule title is a grammar island. The ID before the title delimiter remains a
normal declaration ID, but title tokens are never citations under either
`[reference] strict` setting. Marker insertion, shorthand expansion, and
cross-reference wrapping never rewrite the sentence, reusing the declaration-
heading exclusion of [§FS-fmt.2.3](FS-fmt.md#23-what-is-never-rewritten). The
rationale body remains ordinary Markdown in every respect.

Without any `rules = true` row, scanning, findings, exit status, CLI text and
JSON, LSP diagnostics, formatter output, `config show`, and managed-block bytes
are byte-for-byte those of the same repository before this feature. The config
schema remains version 1.

## 2. Subject selectors

Phase-1 rule subjects select local declaration or named-chapter units only.
They use each configured kind's effective ID grammar
([§FS-config.3.2](FS-config.md#32-id--id-grammar)) and the scanner's accepted
body-local section records. A declaration unit is its owned body. A chapter
unit is the named heading and its subtree through the line before the next
heading of the same or shallower depth. Rejected outside-body and unmarked
headings are not units.

The four accepted subject spellings, and the one shorthand only a selector
accepts, are:

| Authored subject | Selector and match |
|---|---|
| `Each KIND` | every local declaration of that configured citable kind |
| `ID` | the one local declaration with that full ID |
| `The NAME chapter of each KIND` | the chapter at that whole path of every local declaration of the kind that has one |
| `ID.NAME[.NAME…]` | the one local named chapter with that exact coordinate |
| `KIND.NAME[.NAME…]` | selector only: means `The NAME chapter of each KIND`, and a rule sentence refuses it |

In chapter subjects, `NAME` selects the section handle, not its display name.
For `## goal: Goal and hypothesis`, `The goal chapter of each BENCH` selects
the `goal` coordinate even though its display name is `Goal and hypothesis`.
Presence rules compare the other field ([§FS-rules.3.1](FS-rules.md#31-chapter-presence)). A chapter subject's
`NAME` is the chapter's whole path from its declaration, never only that
path's last component ([§FS-rules.2.1](FS-rules.md#21-a-chapters-name-is-its-whole-path)).

A token exactly equal to a configured citable kind name is the quantified kind
selector; any longer token must parse as a full ID under that kind's effective
grammar. Named components require `[id] named_sections = true` and obey the
configured named-section grammar. An exact literal subject must resolve to one
unit; zero matches or ambiguity produces `invalid-rule` ([§FS-rules.7.1](FS-rules.md#71-invalid-rule)). A quantified
subject selects only the units that exist: a chapter subject selects the named
chapter of every local declaration of the kind that has one, so a declaration
without it contributes no unit rather than a failing one. Contributing no unit
is not being outside the rule. A quantified chapter subject selects only the
chapters that exist, and a declaration of the kind that has none is reported
rather than passed over, so a chapter-scoped citation rule reaches every
declaration of its kind. That finding is
[§FS-rules.checks.unreached-declaration](FS-rules.md#checksunreached-declaration-unreached-declaration).

The two spellings therefore part on the same absence, and they stay two
different kinds of finding. Deleting the `requirements` chapter makes
`FS-login.requirements must cite at least one REQ.` an `invalid-rule` at the
rule heading, because an exact literal that does not resolve is a defect in the
sentence and is reported where the sentence is written. It makes
`The requirements chapter of each FS must cite at least one REQ.` an
`unreached-declaration` at `FS-login`, because a quantified subject that selects
nothing for a declaration is a gap in coverage and is reported where the
declaration is. Neither finding displaces the other, and where both spellings
stand over the same absent chapter both fire. Kind-level existence is not
expressible in phase 1.

Subject-side aliases, including `*/`, component wildcards,
numbered chapter literals, `Each chapter of each KIND`, and file, directory, or
path subjects are refused ([§FS-rules.3.5](FS-rules.md#35-strict-refusals)). The future component wildcard, if admitted,
will consume exactly one accepted section component; phase 1 accepts no such
token.

Object kind targets use the existing target-entry grammar of
[§FS-config.3.9.3](FS-config.md#393-alias-matching) unchanged, including a
pinned `alias/KIND` and `*/KIND`. Alternatives joined with `or` are one target
set. Repeated or differently ordered kinds normalize to the same byte-sorted
set for meaning and deduplication.

### 2.1 A chapter's NAME is its whole path

A chapter subject's `NAME` is the whole section path from the declaration to
the chapter, one named component or several. It selects the chapter at exactly
that path, at whatever depth the chapter is declared, and it is never compared
with the last component of a path alone. Take `FS-login`, whose
`## requirements` chapter holds a `### requirements.terms` chapter:

- `FS.requirements.terms` and `The requirements.terms chapter of each FS`
  select `FS-login.requirements.terms`, the same chapter its exact coordinate
  selects.
- `FS.terms` and `The terms chapter of each FS` select a `## terms` chapter
  directly under `FS-login` where there is one. They never select the nested
  `requirements.terms`, which only shares its last component.
- `FS.requirements` selects the `requirements` chapter, whose unit includes
  `requirements.terms`. It does not select `requirements.terms` as a unit of
  its own.

The path is the same under any `section_separator`, because the separator
stands only between a declaration and its first component: with
`section_separator = ":"` the dotted selector is `FS:requirements.terms`.

So a `NAME` means one thing on every surface that reads a subject: the rule
families ([§FS-rules.3](FS-rules.md#3-the-five-sentence-families)), the declarations a rule leaves unreached
([§FS-rules.checks.unreached-declaration](FS-rules.md#checksunreached-declaration-unreached-declaration)), and every `list --selector` mode,
`--size` included ([§FS-rules.8](FS-rules.md#8-command-surfaces)). A declaration contributes a unit exactly when
it has a chapter at that path, and it is unreached exactly when it has none.

## 3. The five sentence families

The normative language is the following closed controlled-English grammar.
Fixed words and kind names are case-sensitive, every rule ends with exactly one
terminal `.`, and one sentence contains exactly one semantic verb. `N` is a
positive base-10 integer. `one` takes singular `chapter`; every numeric `N`
other than one takes `chapters`. A lower bound and an exact count spell one as
the word `one` and refuse the numeral, so `at least 1` and `exactly 1` are not
sentences; an upper bound spells it as the numeral, so `at most 1` is the only
spelling of a ceiling of one. The only accepted families are the following.

### 3.1 Chapter presence

```text
<subject> must|should have at least one <NAME> chapter.
<subject> must|should have at least N <NAME> chapters.
<subject> must|should have at most N <NAME> chapter|chapters.
<subject> must|should have exactly one <NAME> chapter.
<subject> must|should have exactly N <NAME> chapters.
```

The subject may be a kind or exact declaration, not a chapter. The rule counts
the subject declaration's accepted direct chapters whose display name is
`NAME`. A count outside the stated interval produces `chapter-cardinality`.
The comparison is case-insensitive and does not match section handles: a direct
`## goal: Goal and hypothesis` chapter contributes zero to a presence rule for
`goal`, while `## goal: Goal` contributes one. Presence `NAME` is a non-empty
single token with no whitespace anywhere; the multi-word display name `Goal
and hypothesis` cannot be used as a presence `NAME`. This restriction does not
restrict authored display titles or change chapter-subject selection.
This family is how a repository states the *count* of a chapter — `exactly
one`, `at most N` — which no citation rule checks. A chapter-scoped citation
rule reaches a declaration that has no such chapter on its own
([§FS-rules.2](FS-rules.md#2-subject-selectors)) and reports it as `unreached-declaration`; a presence rule beside
it reports the same absence as `chapter-cardinality`, naming its own rule and
suppressing nothing ([§FS-rules.checks.unreached-declaration](FS-rules.md#checksunreached-declaration-unreached-declaration)). The two families
differ in the code they raise and in the counts they can state, not in whether
the absence is seen.

### 3.2 Outbound citation count

```text
<subject> must|should cite at least one <KIND> [or <KIND> ...].
<subject> must|should cite at least N <KIND> [or <KIND> ...].
<subject> must|should cite at most N <KIND> [or <KIND> ...].
<subject> must|should cite exactly N <KIND> [or <KIND> ...].
```

Every resolved physical citation site inside the subject unit whose target is
in the object set counts once. An ordinary `must cite at least one` failure at
zero reuses `missing-citation`; every other count failure uses
`citation-cardinality`.

### 3.3 Per-target coverage

```text
<subject> must|should cite each <KIND> at least once.
<subject> must|should cite each <KIND> at least N times.
<subject> must|should cite each <KIND> at most N times.
<subject> must|should cite each <KIND> exactly once.
<subject> must|should cite each <KIND> exactly N times.
```

For every declaration matched by the object kind selector, the rule separately
counts physical citation sites from the subject unit to that declaration. It
emits one `citation-cardinality` row per target whose count is outside the
constraint. If the subject is itself in the target set it is included; there is
no implicit self-exception. A target selector matching nothing yields no rows.

### 3.4 Inbound citation count and prohibition

Inbound counts use the same canonical counts as ordinary outbound citations:

```text
<subject> must|should be cited by at least one <KIND> [or <KIND> ...].
<subject> must|should be cited by at least N <KIND> [or <KIND> ...].
<subject> must|should be cited by at most N <KIND> [or <KIND> ...].
<subject> must|should be cited by exactly N <KIND> [or <KIND> ...].
```

Each resolved physical citation from a declaration of a source kind to the
subject unit counts once. A violation produces `uncited-unit`. A citation to a
numbered section in the subject unit's own body is a citation to that unit
([§FS-rules.5.1](FS-rules.md#51-facts-and-identity)), so `GOAL-x must be cited by at least one BENCH.` counts
`GOAL-x.4`; a citation to a named chapter, `GOAL-x.outcome`, is a citation to
that chapter, and counts for `GOAL-x.outcome` rather than for `GOAL-x`.

Prohibition has one spelling and no count synonym:

```text
<subject> must not|should not cite any <KIND> [or <KIND> ...].
```

Every offending physical citation site produces the existing
`forbidden-citation` or `discouraged-citation` finding ([§FS-rules.7.5](FS-rules.md#75-prohibition-and-recommendation-reuse)).

### 3.5 Strict refusals

No paraphrase or unlisted production is accepted. A refusal identifies the
failed production and gives its canonical accepted rewrite: the sentence as
typed with that production replaced, which is accepted when it is pasted back.
An ambiguity names every candidate and chooses none. A chapter subject
refused for its chapter path names the component that failed, whichever
spelling reached it ([§FS-rules.3.5.3](FS-rules.md#353-a-chapter-path-is-refused-for-the-component-that-failed)). [§FS-rules.3.5.4](FS-rules.md#354-an-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced) builds
every rewrite, and it names the refusals that offer none because the sentence
does not say what belongs in their place: a path subject, an object kind that
is not configured, a subject from which no configured kind can be recovered,
and a sentence with no modality, verb or accepted count. In a repository whose
kinds are `GOAL`, `REQ`, `FS`, `AR` and `RULE`, with named sections on, the
documentation and parser tests carry at least these exact rows. Where a row
offers no rewrite, `check --rule` prints the reason alone and then the line
`known kinds: GOAL, REQ, FS, AR, RULE`:

| Refused sentence | Exact reason and accepted rewrite(s) |
|---|---|
| `Each FS may not cite any AR.` | `modality "may not" is not accepted; accepted form: Each FS must not cite any AR.` |
| `Each FS must cite no AR.` | `"cite no" is not accepted; accepted form: Each FS must not cite any AR.` |
| `Each FS must cite a GOAL.` | `quantifier "a" is ambiguous; accepted forms: "Each FS must cite at least one GOAL." or "Each FS must cite exactly one GOAL."` |
| `Each FS must cite at least 1 GOAL.` | `numeric "at least 1" is not canonical; accepted form: Each FS must cite at least one GOAL.` |
| `Each FS must cite exactly 1 GOAL.` | `numeric "exactly 1" is not canonical; accepted form: Each FS must cite exactly one GOAL.` |
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

`FS-missing must cite at least one GOAL.` is syntactically valid and therefore
is not a pre-scan refusal. After scanning it produces the exact resolution
message `literal subject FS-missing does not resolve` ([§FS-rules.4](FS-rules.md#4-validation-lifecycle)).

Five of these rows once printed a fixed rewrite that named a chapter, a
declaration or a kind the sentence never typed. The decision to replace those
rewrites in place, and every other fixed rewrite with one built from the
typed sentence, is
[§DF-rule-refusal-rewrites](../decisions/functional/DF-rule-refusal-rewrites.md#df-rule-refusal-rewrites-a-refused-rules-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced-in-place).

#### 3.5.1 Presence-name whitespace refusal

A presence `NAME` containing whitespace anywhere, including internal spaces,
tabs, and Unicode whitespace, remains refused. The released reason remains a
verbatim contiguous prefix under [§FS-errors.3](FS-errors.md#3-message-text), and the explanation
`NAME forbids whitespace anywhere.` remains the end of the message. Between
them stands the accepted form [§FS-rules.3.5.4](FS-rules.md#354-an-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced) builds: the sentence as typed with
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
new grammar. An ad-hoc refusal still writes nothing to stdout and exits 2
([§FS-rules.4](FS-rules.md#4-validation-lifecycle)), and a missing piece here is a chapter name, never a
kind, so no `known kinds:` line follows it.

#### 3.5.2 A subject that needs named sections is answered with one they make valid

With `[id] named_sections = false`, a subject refused because named chapter
subjects require named sections is answered with a subject that turning them on
makes valid, so a reader who follows the suggestion is not refused again. It is
built from what was typed and from the configured kinds:

1. The subject is parsed again with named sections on. Where that parse accepts
   it, the suggestion is the subject as typed.
2. Otherwise the suggestion is what [§FS-rules.8.1](FS-rules.md#81-a-refused-selector-is-answered-with-a-selector)'s four steps build for that
   second refusal, written as a rule subject. `KIND.NAME` is selector-only
   ([§FS-rules.2](FS-rules.md#2-subject-selectors)), so where a selector would get `KIND.NAME[.NAME…]` the rule gets
   `The NAME[.NAME…] chapter of each KIND`, and where a selector would get a
   bare `KIND` the rule gets `Each KIND`.
3. The label is `accepted form after enabling it:` only where the suggested
   subject is refused with the configuration as it is, and `accepted form:`
   where the repository accepts it as configured.
4. Where no configured kind can be recovered, or a configured ID grammar cannot
   be compiled with named sections on, nothing is suggested.
5. The reason stays the one [§FS-rules.3.5](FS-rules.md#35-strict-refusals) gives for the subject as configured,
   `named chapter subjects require [id] named_sections = true`. The modality
   and predicate are the ones the author typed, rewritten as
   [§FS-rules.3.5.4](FS-rules.md#354-an-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced) says where they are refused too, and where that point offers
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
  supplied is a kind ([§FS-rules.3.5.4](FS-rules.md#354-an-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced)), a `known kinds:` line in the
  form an unknown `--kind` ends in, naming the whole set the subject accepts
  ([§FS-list.1.1](FS-list.md#11---kind)): the citable kinds of the project whose configuration reads the
  subject, once each, in configuration order. That is the root project at a
  workspace root and the member under a member path. In one project these are
  the two lines `list --selector` prints for the same subject ([§FS-rules.8.1](FS-rules.md#81-a-refused-selector-is-answered-with-a-selector)).
  At a workspace root that selector lists every loaded project's kinds
  ([§FS-list.1.2](FS-list.md#12---project)), and the rule only the root's, because its subject accepts no
  other. It writes nothing to stdout and exits 2.
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
named-sections-off row of [§FS-rules.3.5](FS-rules.md#35-strict-refusals). A subject refused for its own production
before named sections are asked about, such as `FS-login.*`, `FS-login.2` or
`Each chapter of each FS`, keeps its [§FS-rules.3.5](FS-rules.md#35-strict-refusals) row. With named sections on,
nothing here applies. The decision to replace the as-typed suggestion in place
rather than append to it is
[§DF-rule-after-enabling-rewrites](../decisions/functional/DF-rule-after-enabling-rewrites.md#df-rule-after-enabling-rewrites-a-rule-subject-that-needs-named-sections-is-answered-with-one-they-make-valid-in-place).

#### 3.5.3 A chapter path is refused for the component that failed

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
as it decides a numbered or wildcard path ([§FS-rules.3.5.2](FS-rules.md#352-a-subject-that-needs-named-sections-is-answered-with-one-they-make-valid)). So with named
sections off `FS-login.requirements.` still gets the section-grammar reason,
while `The requirements.1 chapter of each FS`, which asks first, keeps its
named-sections-off refusal.

This point decides the reason alone. The accepted form after `; ` is the typed
sentence with its subject rebuilt as [§FS-rules.3.5.4.2](FS-rules.md#3542-only-what-can-be-recovered-is-supplied) says, the subject
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
`list --selector` names the same component ([§FS-rules.8.1](FS-rules.md#81-a-refused-selector-is-answered-with-a-selector)). The decision to
correct these reasons in place is
[§DF-rule-refusal-reasons](../decisions/functional/DF-rule-refusal-reasons.md#df-rule-refusal-reasons-a-chapter-path-is-refused-for-the-component-that-failed-corrected-in-place).

#### 3.5.4 An accepted form is the typed sentence with the failed part replaced

Every sentence a rule surface offers after `accepted form:`, `accepted forms:`
or `accepted form after enabling it:` is the sentence the author typed with the
production that failed rewritten, and pasting it back to `check --rule` is not
refused, in every repository and not only in one that happens to configure the
kinds an example names. Accepted means what it means in [§FS-rules.3.5.2](FS-rules.md#352-a-subject-that-needs-named-sections-is-answered-with-one-they-make-valid): not
refused before the scan, so whether a literal subject resolves stays the scan's
question ([§FS-rules.4](FS-rules.md#4-validation-lifecycle)). The sentence front end that owns the refusals builds the
form, so `check --rule` and a configured rule declaration's `invalid-rule`
finding offer the same one. The reason before the `;` is not part of the form
and does not change with it.

##### 3.5.4.1 Only the failed part is replaced

The subject, modality, count and object kinds that did not fail are kept as
they were typed, and only the production that failed is spelled canonically. A
refusal never answers with a sentence about `FS`, `GOAL`, `REQ` or
`AR-overview.system-overview` that the author did not write.

##### 3.5.4.2 Only what can be recovered is supplied

A refused subject is rebuilt with [§FS-rules.8.1](FS-rules.md#81-a-refused-selector-is-answered-with-a-selector)'s four steps and written as a rule
subject, the way [§FS-rules.3.5.2](FS-rules.md#352-a-subject-that-needs-named-sections-is-answered-with-one-they-make-valid) step 2 writes one: where a selector would get
`KIND.NAME[.NAME…]` the rule gets `The NAME[.NAME…] chapter of each KIND`, and
where a selector would get a bare `KIND` the rule gets `Each KIND`. An object
kind is never guessed. A chapter name is never invented: the front end reads
the configuration and not the scan, so a subject keeps the named components
typed before the refused one, and where none was typed it falls back to its
declaration or to `Each KIND`. So an empty component keeps what was typed
before it and nothing after it, as `list --selector` does: `FS-login..requirements`
is offered `FS-login` and `FS-login.requirements.` is offered
`FS-login.requirements`.

##### 3.5.4.3 A form is parsed before it is offered

The form is parsed again with the same vocabulary. Where another part of it is
refused, that part is replaced the same way and the form is parsed again. Each
pass replaces one refused production, so the passes end within the number of
productions a sentence has, and a pass that would leave the sentence as it was
ends them with no form. A form is offered only once the parser accepts it.
Where a pass has nothing to put in, or more than one candidate, no form is
offered. A two-form refusal offers its pair only where both forms are accepted.

##### 3.5.4.4 Where no form is offered

Where no form is offered, the refusal is its reason alone. On `check --rule`,
the `known kinds:` line [§FS-rules.3.5.2](FS-rules.md#352-a-subject-that-needs-named-sections-is-answered-with-one-they-make-valid) prints follows the reason where what
could not be supplied is a kind, a subject's or an object's, naming the same
set; it writes nothing to stdout and exits 2. Where what could not be supplied
is anything else, nothing follows the reason. A configured rule declaration's
`invalid-rule` finding ends at the reason either way ([§FS-rules.7.1](FS-rules.md#71-invalid-rule)).

##### 3.5.4.5 The rewrite each refusal makes

| Refusal | What the form replaces | No form when |
|---|---|---|
| modality `may not` | `must not` | — |
| `cite no` | `<modality> not cite any` | — |
| lowercase `each` | `Each` | — |
| no terminal `.` | appends `.` | — |
| numeric `at least 1`, `exactly 1` | `at least one`, `exactly one` | — |
| numeric `at least 1 times`, `exactly 1 times` | `at least once`, `exactly once` | — |
| a per-target count without `times` | the count typed after the bound, then `times` | that count is not a numeral |
| a count with leading zeros | the count without them | the count is zero or is not a numeral |
| quantifier `a` | two forms, `at least one` and `exactly one` | either form is refused |
| the documented conjunction | two forms, one per clause | either form is refused |
| a prohibition without `cite any` | `cite [count] KINDS` becomes `cite any KINDS` | the predicate is anything else |
| a presence object not ending in `chapter` | its last word becomes `chapter` or `chapters`, as the count takes | no word precedes it |
| a presence noun of the wrong number | `chapter` or `chapters`, as the count takes | — |
| a presence `NAME` with whitespace ([§FS-rules.3.5.1](FS-rules.md#351-presence-name-whitespace-refusal)) | the `NAME` trimmed of its surrounding whitespace | the `NAME` is empty or has whitespace inside |
| a chapter subject in a presence rule | the subject's declaration, or `Each KIND` | — |
| a refused subject: an unknown kind, `*/`, a section-component wildcard, a numbered chapter, the section grammar, the ID grammar, `Each chapter of each` | the subject, rebuilt as [§FS-rules.3.5.4.2](FS-rules.md#3542-only-what-can-be-recovered-is-supplied) says | no configured kind is recovered; `known kinds:` follows |
| a path subject, `Each file in …` | — | always; `known kinds:` follows |
| an unknown or malformed object kind | — | always; `known kinds:` follows |
| no modality, an unknown verb, a count that is not accepted, a count with no object | — | always |

Every row keeps the subject and modality as typed, so the presence, per-target
and count refusals no longer answer with `Each FS must` or
`AR-overview.system-overview` whatever was typed. A refusal whose reason
appends guidance keeps it where no form is offered: `count is not accepted`
keeps its list of the canonical counts, and the [§FS-rules.3.5.1](FS-rules.md#351-presence-name-whitespace-refusal) refusal keeps
`NAME forbids whitespace anywhere.`.

##### 3.5.4.6 Exact rows where `FS` is the only kind

In the repository of [§FS-rules.8.1](FS-rules.md#81-a-refused-selector-is-answered-with-a-selector)'s first table, whose one kind is `FS`, with
named sections on and `FS-login` holding named `requirements` and `should`
chapters, these are the exact `check --rule` refusals. Each exits 2 with
nothing on stdout, and its `error:` line is the reason and what follows it. A
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
| `FS-login must cite each FS exactly 2.` | `per-target counts must end in "times"; accepted form: FS-login must cite each FS exactly 2 times.` | none |
| `Each FS must cite exactly 02 FS.` | `count must be a canonical positive base-10 integer; accepted form: Each FS must cite exactly 2 FS.` | none |
| `Each FS must cite a FS.` | `quantifier "a" is ambiguous; accepted forms: "Each FS must cite at least one FS." or "Each FS must cite exactly one FS."` | none |
| `Each FS must cite at least one GOAL and must not cite any AR.` | `conjunctions are not accepted` | `known kinds: FS` |
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
| `Each FS cites at least one FS.` | `rule has no accepted modality` | none |
| `Each FS must reference at least one FS.` | `verb is not accepted` | none |
| `Each FS must cite some FS.` | `count is not accepted; the canonical counts are "at least one", "at least N", "at most N", "exactly one" and "exactly N" for a base-10 N` | none |
| `Each FS must cite at least two FS.` | `count must be a canonical positive base-10 integer` | none |

The fourth row is offered a form only after a second pass: appending the `.`
leaves `at least 1`, which the next pass rewrites. The conjunction and the last
`FS-*.requirements` row are offered none for the opposite reason: the first
pass rebuilds what failed, and the form it leaves names a kind this repository
does not configure, which no pass may guess.

### 3.6 Where a sentence's subject ends

A sentence's subject is the text before its modality. The modality is found in
a fixed order, not at the earliest position: the first ` must not `, else the
first ` should not `, else the first ` must `, else the first ` should `. So
`The should chapter of each FS must cite at least one FS.` has the subject
`The should chapter of each FS`, because ` must ` is tried before ` should `,
and `The must chapter of each FS should not cite any AR.` has the subject
`The must chapter of each FS`, because ` should not ` is tried before ` must `.

## 4. Validation lifecycle

`config validate` validates only the optional `rules` key and its relationship
to the kind row. Rule titles are Markdown catalog data and are parsed after the
shared scan has built the catalog, then resolved before evaluation or managed-
block rendering. A missing title, empty rationale, refused title, unresolved or
ambiguous literal, or disabled named component is a located `invalid-rule` at
the rule heading. One invalid rule does not prevent other scanned findings from
being reported, and no surface silently omits it.

`check --rule` is different only where a title declaration would have supplied
a location. Grammar or vocabulary refusal is decided after config selection
and before scanning, prints `error: <reason>; accepted form: <rewrite>` (or
`accepted forms:`, or `accepted form after enabling it:` where the rewrite
needs `[id] named_sections = true`), writes nothing to stdout, and exits 2. A
refusal that offers no form ([§FS-rules.3.5.4](FS-rules.md#354-an-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced)) prints `error: <reason>` alone,
and then `known kinds: <kinds>` where what it could not supply is a kind
([§FS-rules.3.5.4.4](FS-rules.md#3544-where-no-form-is-offered)). A syntactically valid
unresolved literal requires the catalog, so after scanning it yields an
`invalid-rule` attributed to `--rule` and the ordinary finding exit 1.

`init` parses and resolves configured rules before writing. Any `invalid-rule`
is printed in its located form, the existing managed block remains byte-for-
byte untouched, and the command exits nonzero. It never renders an invalid
sentence and never silently skips one. A sentence that is well-formed and only
unverifiable from where the command ran is not an invalid one: that is
[§FS-rules.4.1](FS-rules.md#41-a-rule-this-scope-cannot-verify)'s case, and only that case.

If any scan or fact producer is incomplete, every closed-world rule conclusion
about absence or count is suppressed. Already-known positive site findings may
remain, all ordinary scan findings are retained, and the run keeps exit 2:
no incomplete tree is presented as a complete rule verdict.

### 4.1 A rule this scope cannot verify

An object kind pinned at an alias (`workshop/OP`) or matched across every
alias (`*/OP`) resolves wherever the run holds the workspace that alias
belongs to, and that is the workspace the run's own effective config declares —
never one climbed to from above. `check` and `init` read one vocabulary there,
so no two commands in one directory disagree about one sentence.

A run whose effective config declares no `[workspace]` holds no alias at
all. A member-scoped run is the ordinary case: its effective config is the
member's own `grund.toml` ([§FS-workspace.2](FS-workspace.md#2-workspace-configuration), [§FS-workspace.5.1](FS-workspace.md#51-a-member-run)), so a pinned or
any-member object kind names something the scope cannot judge either way. Such
a sentence is **unverifiable here** rather than invalid. It is reported at the
rule's own heading, keeps the code `invalid-rule`, takes the wording
[§FS-errors.3.7](FS-errors.md#37-the-rule-site-unknown-alias-wording-migration) fixes, and the run still exits nonzero.

Unverifiability is a verdict about the object kind's alias and about nothing
else. Every other part of the sentence is judged against the facts the scope
already holds, exactly as it is when the object kind is local — the grammar, the
rationale, and a literal subject that has to resolve. A rule that fails for any
of those reasons is an invalid rule with [§FS-rules.4](FS-rules.md#4-validation-lifecycle)'s consequences in full,
whichever alias its object names, and the run reports the fact it can act on
rather than the one it cannot.

#### 4.1.1 The distinction is mechanical

What makes a sentence unverifiable is the absence of every workspace alias,
which the run knows without judging anything:

| written object kind | the run's workspace vocabulary | verdict |
|---|---|---|
| `workshop/OP` | holds no alias at all | unverifiable here |
| `workshop/OP` | holds `workshop`, which declares `OP` | resolves |
| `workshop/NOPE` | holds `workshop` | invalid rule |
| `typo/OP` | holds aliases, none named `typo` | invalid rule |
| `*/OP`, `OP` not local | holds no alias at all | unverifiable here |
| `*/OP`, `OP` not local | holds aliases, none declaring `OP` | invalid rule |
| `*/OP`, `OP` declared locally | either | resolves |

A config that declares `[workspace]` holds at least its own project's alias,
whatever its member list expands to — only a config that declares none holds
nothing at all. So an empty member list and one naming a member the run cannot
reach come to the same verdict about the same rule, and no run standing at a
workspace root is told that no workspace is in scope where it stands.

The exception is therefore never a relaxation of resolution. Where the scope
could judge the alias and the answer was no, the sentence stays invalid and
keeps every consequence the paragraph above gives it.

#### 4.1.2 What `init` writes anyway

`init` withholds the managed-block write for an invalid rule and for nothing
else. A run whose every unresolved rule is unverifiable here writes the block,
renders each unverifiable sentence as authored ([§FS-rules.9.1](FS-rules.md#91-one-tree-renders-one-block)), reports each one,
and exits nonzero: it is the write that is not withheld, not the failure that is
forgiven. One genuinely invalid rule beside an unverifiable one puts the whole
run back under [§FS-rules.4](FS-rules.md#4-validation-lifecycle) — nothing written, exit nonzero.

Withholding the write is what made [§REQ-agents-md.2](../requirements/REQ-agents-md.md#2-the-managed-block-stays-current) unsatisfiable from inside a
member holding such a rule. `check` there reports the managed block out of date
and names `grund init` as the remedy; a refusal to write leaves no exit from
that loop, and no edit inside the member opens one.

## 5. Relational meaning

The normative meaning is Datalog over one finite, complete snapshot. The first
implementation is a hand-written evaluator; no Datalog surface or runtime is
part of the release.

### 5.1 Facts and identity

`RuleFacts` has an immutable header containing `schema_version`, project
identity, producer identity, and completeness. Opaque keys are scoped by
project and producer. Side metadata maps each key to its stable authored label
and repository-relative anchor; metadata does not participate in logical
equality. The relations are:

```text
decl(node, kind)
chapter(node, section_path, display_name)
contains(parent_node, child_node)
cites(site, immediate_from_node, target_node)
site_in(site, unit_node)
```

Every declaration and accepted chapter has its node facts whether or not it
contains a citation, so citations never define the quantified universe. Every
physical citation has a distinct site key, preserving multiplicity. `cites`
uses the immediate enclosing declaration or chapter as `from`; `site_in`
relates the site to every rule unit that contains it.

A resolved citation whose section is not itself a rule unit — one with a
numbered component, such as `GOAL-x.4` or `GOAL-x.outcome.2` — is a `cites` fact
to its nearest enclosing unit: the nearest named ancestor chapter, or else the
declaration. `GOAL-x.4` targets `GOAL-x`, and `GOAL-x.outcome.2` targets the
`outcome` chapter. The source side resolves the same way, so a site inside a
numbered section `goal.3` is `from` the `goal` chapter, not from the
declaration. A citation `grund check` resolves therefore counts for every rule
family exactly where it satisfies an ordinary `[citations.KIND]` obligation
([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)).
Only a citation that does not resolve — an unknown or ambiguous declaration, or
a section its declaration does not have — retains its ordinary findings and
contributes no `cites` fact. Counting these sites newly fails some rules on
trees that pass without them; those findings warn for one release
([§FS-rules.7.8](FS-rules.md#78-a-newly-counted-section-citation-warns-until-0180)).

#### 5.1.1 A chapter's display name is the label its author wrote

A `chapter` fact's `display_name` is the label the author wrote after the
chapter's coordinate: `Terms` in `## terms: Terms`. A nested chapter's
coordinate is its whole path, so `### requirements.terms: Terms` is
`chapter(c, requirements.terms, Terms)`, and its label is `Terms` as well. It
is the same label whether the declaration's body lives in a Markdown file or in
a source doc-comment. A heading in a doc-comment sits behind the comment that carries
it — `///`, `//!`, a block comment's ` * `, a hash comment's `#`, or any other
configured prefix
([§FS-config.3.5.14](FS-config.md#3514-comment_prefixes-compose-with-extensions))
— and that envelope says where the body lives, not what the chapter is called,
so it is never part of the label. `/// ## terms: Terms` in a Rust file and
`## terms: Terms` in a Markdown file are therefore the same fact,
`chapter(c, terms, Terms)`, and a presence rule naming `Terms` counts either
one ([§FS-rules.3.1](FS-rules.md#31-chapter-presence)). The same label is the
title of that chapter's `list --selector` row
([§FS-rules.8](FS-rules.md#8-command-surfaces)).

The envelope has a closing half too. A block comment's closing `*/` on the
heading's own line, and the whitespace before it, closes the comment that
carries the heading, so it is no more part of the label than the ` * ` that
opens the line: ` * ## terms: Terms */` on the last line of a C or Java block
comment is `chapter(c, terms, Terms)`, the fact `/// ## terms: Terms` gives.
Only the C-family closer is envelope, because only a C-family block comment
carries a heading on a line that can close it. A Python docstring's heading is
read as Markdown, and a Markdown heading is never trimmed, so an author who
writes `*/` at the end of either keeps it in the label. The `title` a
declaration's `--toc --format json` section map gives that chapter is the same
label ([§FS-show.3.1.3](FS-show.md#313-json)).

### 5.2 Family clauses

For subject set `S`, target set `T`, physical site `P`, chapter name `N`, the
subject's kind `K`, and the interval predicate `within`, the five families
mean:

```text
chapter_count(s, N, n) :- s in S, n = count { c : chapter(c, _, N), contains(s, c) }.
outbound_count(s, T, n) :- s in S, n = count { p : cites(p, _, t), t in T, site_in(p, s) }.
per_target_count(s, t, n) :- s in S, t in T,
                             n = count { p : cites(p, _, t), site_in(p, s) }.
inbound_count(s, T, n) :- s in S,
                          n = count { p : cites(p, f, s), f in declarations(T) }.
forbidden_site(s, p) :- s in S, cites(p, _, t), t in T, site_in(p, s).
```

Positive families report where `not within(n, cardinality)`. Prohibitions
report every `forbidden_site`. Negation and aggregates are closed-world only
when the snapshot is complete ([§FS-rules.4](FS-rules.md#4-validation-lifecycle)).

A chapter subject `The N chapter of each K` carries a second premise, taken
over the declarations of the kind rather than over the chapters `S` selected:

```text
chapter_handle(c, N) :- chapter(c, N, _).
owns(d, c)           :- decl(d, _), contains(d, c), chapter(c, _, _).
owns(d, c)           :- owns(d, p), contains(p, c), chapter(c, _, _).
chapter_of(d, N)     :- chapter_handle(c, N), decl(d, K), owns(d, c).
unreached(d, N)      :- decl(d, K), not chapter_of(d, N).
```

`chapter_handle` is the subject selector of [§FS-rules.2](FS-rules.md#2-subject-selectors) read as a relation. A
subject's `N` is an accepted section path, one named component or several
([§FS-rules.2.1](FS-rules.md#21-a-chapters-name-is-its-whole-path)), so it joins the whole of the section path — `chapter`'s
second position — and neither that path's last component nor the display name
in its third, which is what the presence family's `chapter_count` counts
([§FS-rules.3.1](FS-rules.md#31-chapter-presence)). The two clauses therefore
join different positions of the same relation, and this one follows the
selector rather than `chapter_count`: `chapter_of(d, N)` holds exactly when `d`
owns one of the chapters `S` selected, so contributing no unit and being
unreached are one set rather than two, and no declaration can both hand a
relation a unit and be reported as out of the rule's reach.

`owns` follows `contains` from a declaration down through its chapters, so a
nested chapter belongs to the declaration whose body holds it, not only to the
chapter directly above it. `contains(d, c)` alone reaches only `d`'s direct
chapters, which is what presence counts ([§FS-rules.3.1](FS-rules.md#31-chapter-presence)), and is too narrow for
a path of several components.

The outbound-count, per-target-coverage and inbound-count families report every
`unreached(d, N)` beside every `not within(n, cardinality)`
([§FS-rules.checks.unreached-declaration](FS-rules.md#checksunreached-declaration-unreached-declaration)). Two families do not, for two different
reasons. Chapter presence cannot reach the premise at all: its subject is a
kind or an exact declaration and never a chapter
([§FS-rules.3.1](FS-rules.md#31-chapter-presence)), so `S` is already the
declarations of the kind and nothing is absent from it. Prohibition admits a
chapter subject and still stays silent, because `forbidden_site` ranges over
`cites` facts and a citation site inside the chapter body is deleted along with
that body: for `must not cite` the forbidden site genuinely no longer exists.
That, rather than vacuity over an empty selection, is why the same argument
does not excuse the other three families.

### 5.3 Evaluation cost on independent records

Rule evaluation must keep checks usable as fast feedback
([§GOAL-fast-feedback.2](../goals.md#2-how-we-get-there)). For a fixed set of rules
over independent declarations with bounded chapter depth, chapters and physical
citation sites per declaration, and a fixed shared target vocabulary, evaluation
must grow roughly with the records and their facts. Selecting a declaration's
chapters, counting its citations, resolving owning declarations, and determining
which declarations a chapter selector leaves unreached must not repeatedly join
whole-snapshot relations for every subject.

The regression corpus has seven active rules, 26 shared `COND` declarations,
four shared `BOUNDARY` declarations, and records with `site`, `boundary`,
`condition`, `establishes`, and `repro` chapters; every fourth record also has
`requires`. Each record has nested site and repro sections and two or three
physical citations into the shared vocabulary. On optimized builds, a doubling
from 400 to 800 clean records must not both take more than three times as long
and take more than two seconds at 800. Each check has a finite execution limit;
a timeout fails the regression. No-rules controls and removing one required
chapter establish that the measured cost is active rule evaluation. Fixture
generation and compilation are outside the measured interval.

This bound is for that fixed-shape workload, not for arbitrary subject/target
products or a growing number of findings. Evaluation still reads the complete
snapshot: pruning records or rules, changing verdicts, or narrowing evaluation
to an output selection does not satisfy it. All family clauses, physical-site
multiplicity, containment and ownership, completeness gates, semantic grouping,
required/recommended channels, diagnostic ordering, and anchors retain their
existing meaning ([§FS-rules.5.1](FS-rules.md#51-facts-and-identity),
[§FS-rules.5.2](FS-rules.md#52-family-clauses),
[§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits)). No public API,
configuration, serialized fact format, or selection workflow is added.

## 6. Semantic deduplication

Before evaluation, constraints deduplicate by `(subject selector, modality,
relation, normalized target set, cardinality)`. Normalization preserves no
authored spelling beyond what appears in a finding.

Rules-only duplicates produce one finding per failing unit or site. The tail
contains every contributing rule ID in bytewise order, for example
`(RULE-a, RULE-b)`; an ad-hoc `--rule` origin participates in the same way.
The collapsed group's contributing origins are also the finding's `authority`
([§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits)), in that
same bytewise order, so the tail and the field are one fact rendered twice and
cannot disagree. A jointly authored finding therefore names `--rule` in its
`authority` beside every declared rule that reached the same meaning, and its
tail is unchanged by that: there is no marker, no reordering, and no second
line.

Config-to-rule deduplication is intentionally narrower. It applies only where
an existing `[citations]` entry and a rule express the same local bare citing
kind, declaration-wide unit, modality/level, `cite` relation, normalized target
entry, and cardinality. If config participates, its existing
`missing-citation`, `suggested-citation`, `forbidden-citation`, or
`discouraged-citation` finding wins byte-for-byte: no rule ID is appended, its
`authority` stays empty, and JSON gains no `sites` list. `[citations]` remains
authored, rendered, and interpreted as before. A rule that loses to config this
way authored nothing, so a trial sentence duplicating a `[citations]` direction
is invisible to a report scoped by `authority`
([§FS-rules.8](FS-rules.md#8-command-surfaces)).

## 7. Findings and channels

All rule findings use the ordinary text and NDJSON schemas of
[§FS-errors](FS-errors.md#fs-errors-grund-emits-messages-in-fixed-shapes).
`must` and `must not` are errors. `should` and `should not` are suggestions:
they appear only with `--suggestions`, carry `"channel":"suggestion"` in JSON,
and never affect exit status. Structural recommendations retain their
structural code on that channel. The one rule finding that was a warning is an
error from 0.16.0 ([§FS-rules.7.7](FS-rules.md#77-one-required-level-finding-is-a-warning-until-0160)); the only rule findings that are
warnings now are those a newly counted section citation alone produces, until
0.18.0 ([§FS-rules.7.8](FS-rules.md#78-a-newly-counted-section-citation-warns-until-0180)).

### 7.1 Invalid rule

`invalid-rule` is located at the rule heading. A configured parse failure is:

```text
<RULE-ID> is not a valid rule: <reason>; accepted form: <rewrite>
```

The rewrite is the sentence as authored with the failed production replaced
([§FS-rules.3.5.4](FS-rules.md#354-an-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced)). An ambiguous production uses `accepted forms:`, and a rewrite
that needs `[id] named_sections = true` uses `accepted form after enabling it:`.
A parse failure that offers no form, every refusal [§FS-rules.3.5.4.5](FS-rules.md#3545-the-rewrite-each-refusal-makes) says
offers none, is:

```text
<RULE-ID> is not a valid rule: <reason>
```

A resolution failure is:

```text
<RULE-ID> is not a valid rule: literal subject <selector> does not resolve
```

For an ad-hoc rule, `<RULE-ID>` is `--rule`.

### 7.2 Chapter cardinality

`chapter-cardinality` is located at the subject declaration title and includes
zero and surplus chapters:

```text
<subject> has <actual> <name> chapters; <RULE-ID> requires <count>
```

#### 7.2.1 Display-name comparison context

Keep the complete message above as a verbatim contiguous prefix
([§FS-errors.3](FS-errors.md#3-message-text)) and append:

```text
; expected display name "<name>" (case-insensitive, not section handle); observed direct chapters: <observed>
```

`<observed>` lists every accepted direct chapter in source order, each quoted
as `"<coordinate>: <display name>"`, separated by `, `; when there are none it
is `none`. Named coordinates expose the handle alongside the authored display
name, so the reported example ends in `"goal: Goal and hypothesis"` rather
than implying that the accepted chapter was not scanned. Nested descendants
and rejected headings do not enter this direct-chapter list or the count.

Text output and JSON `message` carry the same complete explanation. The code,
subject-title location, authority, severity/channel, other finding fields and
exit verdict stay unchanged. The rule guide states this display-name/handle
distinction beside its presence and chapter-subject examples and explains the
single-token presence `NAME` restriction using `goal: Goal and hypothesis`.

### 7.3 Outbound citation cardinality

An ordinary hard `at least one` rule with zero matches reuses
`missing-citation`:

```text
<subject> must cite <target-set> (<RULE-ID>)
```

Other ordinary counts and every per-target `cite each` miss use
`citation-cardinality`, located at the subject declaration or chapter title:

```text
<subject> cites <target-set> <actual> times; <RULE-ID> requires <count>
```

For `cite each`, `<target-set>` is the one canonical target ID and one row
is emitted per off-count target.

A chapter subject whose declaration has no such chapter reports
`unreached-declaration` instead, located at the declaration rather than at a
chapter title there is none of
([§FS-rules.checks.unreached-declaration](FS-rules.md#checksunreached-declaration-unreached-declaration)). It never reuses the
`cites <target-set> 0 times` line, which would name a unit that does not exist.

### 7.4 Inbound citation cardinality

`uncited-unit` is located at the subject declaration or chapter title:

```text
<subject> is cited by <source-set> <actual> times; <RULE-ID> requires <count>
```

The code covers zero, surplus, and other off-count inbound cardinalities. A
chapter subject whose declaration has no such chapter reports
`unreached-declaration` at the declaration instead, on the same reading as
[§FS-rules.7.3](FS-rules.md#73-outbound-citation-cardinality)'s.

### 7.5 Prohibition and recommendation reuse

A hard prohibition reuses the existing site-anchored `forbidden-citation`
wording of [§FS-check.3.12](FS-check.md#312-forbidden-citation), with
`(<RULE-ID>)` in place of `(citation direction)`. A negative recommendation
likewise reuses `discouraged-citation`, and an ordinary positive citation
recommendation reuses `suggested-citation`. Existing config-only bytes never
change.

### 7.6 Selection, JSON, ordering, and exits

`--only` and `--ignore` accept `invalid-rule`, `chapter-cardinality`,
`citation-cardinality`, `uncited-unit`, and `unreached-declaration` like every
other public code; their
value grammar takes codes only and never a rule identity
([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)).

The **rule-produced codes** are the eight a rule finding can carry: the four
only rules produce — `chapter-cardinality`, `citation-cardinality`,
`uncited-unit`, and `unreached-declaration` — and the four rules share with
citation directions ([§FS-rules.7.3](FS-rules.md#73-outbound-citation-cardinality),
[§FS-rules.7.5](FS-rules.md#75-prohibition-and-recommendation-reuse)) —
`missing-citation`, `forbidden-citation`, `discouraged-citation`, and
`suggested-citation`. `invalid-rule` is selectable like every other public code,
and is also selected whenever a code from that set is: a run narrowed to what
rules find keeps every `invalid-rule` row, because a skipped rule is a check that
run did not evaluate
([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)).
`--ignore invalid-rule` still removes them.

Rule-derived JSON adds no `sites` list — a rule finding names one site, so the
multi-site field stays `null` on every one of them. It does carry the rule
authority as a field: every rule-derived message names its rule authority, and
the same origins are the record's `authority`, a bytewise-sorted list of rule
origins ([§FS-errors.5.1](FS-errors.md#51-on-stdout--the-commands-output),
[§FS-output-shapes.1](FS-output-shapes.md#1-finding-object)). A chapter subject
is rendered as its canonical qualified coordinate.

A rule-derived *diagnostic* carries the authority of the one rule that produced
it rather than a group's: an `invalid-rule`
([§FS-rules.7.1](FS-rules.md#71-invalid-rule)) names that rule, so a trial
sentence whose literal subject does not resolve is attributed to `--rule` and is
retained by a report scoped to it. A finding no rule authored — a citation
direction, a dangling or duplicate declaration, a run-level warning — carries an
empty `authority`.

The existing bytewise `(path, line, message)` ordering is the sole ordering
authority. A `cite each` message places its target ID immediately after the
fixed `<subject> cites ` prefix, before the actual count, so same-anchor rows
sort by target-ID bytes even when targets share a prefix. Findings affect exits
under the ordinary mapping: hard findings exit 1, suggestions never move the
exit, invocation/config failures exit 2, and incomplete scans stay 2.

### 7.7 One required-level finding is a warning until 0.16.0

The ramp this heading names is spent: it closed in grund 0.16.0, where
`unreached-declaration` became an error on the ordinary `must` channel and the
rules report stopped having a warnings channel. What the check does is
specified at the section its code names,
[§FS-rules.checks.unreached-declaration](FS-rules.md#checksunreached-declaration-unreached-declaration). This address, heading text included, is
kept only so citations written before the promotion still resolve.

### 7.8 A newly counted section citation warns until 0.18.0

A required-level finding that exists only because [§FS-rules.5.1](FS-rules.md#51-facts-and-identity) counts a
citation to a numbered section is carried on the warnings channel rather than
the errors channel, for one release window. It covers every finding that
counting more sites can newly produce: a prohibition's `forbidden-citation` at
such a site, and the `citation-cardinality` or `uncited-unit` of an `at most N`
or `exactly N` count — outbound, per-target, or inbound — that the same count
without those sites satisfies. A count that fails without them is still an
error, and a count those sites bring inside its bounds passes at once: the
verdicts that newly pass land in the release that ships the change, and only
the ones that newly fail are ramped.

The warning is the error's message unchanged, followed by `; a citation to a
numbered section now counts, and this warning becomes an error in grund
0.18.0`, the pending clause of [§FS-distribution.4.2.3](FS-distribution.md#423-the-vocabulary-is-closed)'s closed vocabulary, so
the release guard reads the promise out of the shipped text. It carries the
code it will carry as an error, so `--only` and `--ignore` select it now as they
will then, and it leaves the exit status where the rest of the run put it. The
catalog rows of these codes keep `error` and no ramp
([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)), because the code's
severity does not move: only the findings this subsection names warn. At the
recommended level such a finding is an ordinary suggestion and owes no ramp,
because a suggestion never moves the exit status at any release.

At `0.18.0` these findings become errors on the ordinary `must` channel, and
this subsection becomes a pointer as [§FS-rules.7.7](FS-rules.md#77-one-required-level-finding-is-a-warning-until-0160) did. The decision and its
compatibility route are [§DF-section-citation-counts-in-rules](../decisions/functional/DF-section-citation-counts-in-rules.md#df-section-citation-counts-in-rules-a-resolved-citation-to-a-numbered-section-counts-in-chapter-rules).

## 8. Command surfaces

`grund check --rule "<sentence>" [<path>]` adds exactly one ad-hoc rule to all
configured rules. It never disables configured rules and deduplicates against
an identical one. Its validation and exit behavior are [§FS-rules.4](FS-rules.md#4-validation-lifecycle)'s.

`grund check --only-rule` narrows that run's report to what the trial sentence
authored: a finding is retained when its `authority`
([§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits)) contains the
`--rule` origin. It is a boolean with no `=value` form, repeats harmlessly, and
is `check`-only ([§FS-cli.3](FS-cli.md#3-cross-subcommand-flags)). It requires
`--rule`: given alone it is an invocation error — `error: --only-rule requires
--rule`, empty stdout, exit `2`, decided where the other selector values are
decided, before config discovery or scanning
([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)) —
because a run that scoped to no sentence would print `success` and exit `0`,
which reads as a verdict rather than as the mistake it is. It composes with
`--only` and `--ignore` by intersection, and `--ignore` still wins
([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)).
Bare `--rule` is unchanged: a run that passes no selector prints the same bytes
and exits the same way it does today.

Three consequences of scoping follow from what `authority` is rather than from
the flag, and each is behavior a caller should be able to rely on.

A finding the trial sentence and a declared rule authored jointly — one finding,
one tail naming both origins
([§FS-rules.6](FS-rules.md#6-semantic-deduplication)) — **is** retained, with its
authority and its message bytes exactly as an unscoped run renders them. The
sentence did author it, and scoping is not the place to relitigate the merge.

A trial sentence duplicating a `[citations]` direction yields an **empty** scoped
report. The config finding wins byte-for-byte and the rule authored nothing
([§FS-rules.6](FS-rules.md#6-semantic-deduplication)), so there is nothing for
`authority` to name, and the run prints `success` and exits `0`
([§FS-check.2.1.3](FS-check.md#213-the-success-line)) while the tree's own
findings still stand. This is known behavior, recorded here so it cannot change
by accident; making such a sentence visible is a separate question this point
does not answer.

A `should`-level trial sentence produces findings only in the suggestions
channel, which the default run withholds
([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)), so `--only-rule`
alone prints `success` and `--suggestions` is needed to see it. Nothing is
special here — `--only` behaves the same — but it is the one combination whose
empty report reads like a sentence that found nothing.

`grund list --selector "<selector>" [<path>]` filters the shared catalog to
matched declaration and chapter units and composes by intersection with the
existing path, kind, project, unused, summary, size, top, and format selectors
where their output modes admit unit rows. It selects exactly the units the
same subject selects in a rule, in every mode, `--size` included, so
`FS.requirements.terms` lists that nested chapter and `FS.terms` only a `terms`
chapter directly under its declaration ([§FS-rules.2.1](FS-rules.md#21-a-chapters-name-is-its-whole-path)). Text prints the
canonical coordinate, two spaces, location, two spaces, and title. A chapter JSON row uses the list
object's existing fields in their existing order, adds `"section"` immediately
after `"id"`, and puts the declaration ID in `id` and exact component path in
`section`; a declaration row remains byte-for-byte the ordinary list row. In
both forms a chapter row's title is the chapter's display name
([§FS-rules.5.1.1](FS-rules.md#511-a-chapters-display-name-is-the-label-its-author-wrote)).
Invalid syntax, unknown vocabulary, disabled named sections, and ambiguous
exact literals are exit-2 invocation errors, and the first three are answered
with a selector to paste rather than a rule sentence ([§FS-rules.8.1](FS-rules.md#81-a-refused-selector-is-answered-with-a-selector)). A valid selector with no matches
prints nothing and exits 0. `--selector` is a flag; the positional remains the
scan path.

### 8.1 A refused selector is answered with a selector

Every `list` mode that takes `--selector`, `--size` included, refuses a
selector it cannot accept on stderr, writes nothing to stdout, and exits `2`.
Its `error:` line names the production that failed and ends in exactly one
accepted selector, which a reader can paste back into `--selector`:

```text
error: <reason>; accepted selector: <selector>
```

The reason is the one [§FS-rules.3.5](FS-rules.md#35-strict-refusals) gives for the same subject wherever that
reason is true, so a rule and a selector never disagree about what failed.
Four refusals name the true failure in place of a grammar or vocabulary
mismatch. A numbered component reached through `KIND.NAME` or
`The NAME chapter of each KIND` gets the numbered reason the ID spelling gets.
A wildcard component reached the same way gets the wildcard reason.
`Each chapter of each KIND` gets the chapter-quantified reason, not an unknown
kind. `Each */KIND` gets the namespace reason the rule sentence gets, not an
unknown kind. An empty component, left by a doubled or trailing separator, is
not a number in any spelling, so it gets the section-grammar reason and no
breadcrumb ([§FS-rules.3.5.3](FS-rules.md#353-a-chapter-path-is-refused-for-the-component-that-failed)).

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
accepts it as configured. The reason stays the one [§FS-rules.3.5](FS-rules.md#35-strict-refusals) gives for
the subject as configured, that named chapter subjects require named sections,
so these refusals add no breadcrumb.

Where no configured kind can be recovered, nothing is guessed. The reason
stands alone, and a second line lists the configured citable kinds exactly as
an unknown `--kind` does ([§FS-list.1.1](FS-list.md#11---kind)). A refusal that earns the breadcrumb
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
| `FS.Requirements` | `named chapter subject "FS.Requirements" does not match the configured section grammar; accepted selector: FS` | none |
| `Each POLICY` | `unknown kind "POLICY"` | `known kinds: FS` |
| `POLICY.requirements` | `literal subject "POLICY.requirements" does not match the configured ID grammar` | `known kinds: FS` |
| `requirements` | `literal subject "requirements" does not match the configured ID grammar` | `known kinds: FS` |
| `Each chapter of each POLICY` | `chapter-quantified subjects are not accepted in phase 1` | `known kinds: FS`, then hint |
| `API.requirements.1` | `numbered chapter subjects can detach when headings move` | `known kinds: FS`, then hint |
| `FS-login must cite at least one GOAL.` | `a rule sentence is not a selector; accepted selector: FS-login` | none |
| `The should chapter of each FS must cite at least one FS.` | `a rule sentence is not a selector; accepted selector: The should chapter of each FS` | none |

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
([§FS-rules.3.5.4.2](FS-rules.md#3542-only-what-can-be-recovered-is-supplied)), and with named sections off as [§FS-rules.3.5.2](FS-rules.md#352-a-subject-that-needs-named-sections-is-answered-with-one-they-make-valid) says. A
chapter subject refused for a component of its path opens with the reason a
selector gets for the same component, as [§FS-rules.3.5.3](FS-rules.md#353-a-chapter-path-is-refused-for-the-component-that-failed) says. So a
selector and a rule refused for the same subject suggest the same units, each
in its own spelling: `FS-*.requirements` is answered with `FS.requirements` as
a selector and with `The requirements chapter of each FS` in a rule. The
decision to replace these lines rather than append to them is
[§DF-selector-refusal-rewrites](../decisions/functional/DF-selector-refusal-rewrites.md#df-selector-refusal-rewrites-a-refused-selector-is-answered-with-a-selector-and-its-old-lines-are-replaced-not-appended-to), and for the rule surfaces
[§DF-rule-refusal-rewrites](../decisions/functional/DF-rule-refusal-rewrites.md#df-rule-refusal-rewrites-a-refused-rules-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced-in-place).

## 9. Managed guidance and editor parity

With at least one rule kind, `grund init` renders `### Chapter rules`
immediately after `### Citation directions`. It repeats the existing
`must`/`should` legend and emits one bullet per rendered rule in qualified
rule-ID order — every valid rule, and every rule unverifiable here
([§FS-rules.4.1](FS-rules.md#41-a-rule-this-scope-cannot-verify)) — each bullet the exact authored sentence followed by its live
rule citation. A rule-enabled block uses v11 on the v10 base. With no rule kind,
`init` retains the v10 block byte-for-byte. `grund check` re-renders and
byte-compares this config-derived section whether or not every rule resolved;
drift is `agents-init`, and the comparison is never skipped in silence
([§FS-check.3.5.4](FS-check.md#354-no-config-derived-section-is-exempt-from-the-comparison)).

Hard rule findings and `invalid-rule` travel through the same core report to
the LSP with the same message, code, title/citation range, and severity as CLI
JSON. The LSP adds no rule parser or evaluator. Suggestions remain CLI-only and
opt-in.

### 9.1 One tree renders one block

The rendered section is a function of the authored sentences and of how the
entrypoint renders their citations ([§FS-init.2.3.5.10](FS-init.md#23510-chapter-rules)), never of the scope a
run was given, so the bullet an unverifiable rule earns is the sentence exactly
as its heading spells it. A member-scoped `init` therefore writes the bytes a
run that held the whole workspace would write for that same file, and a `check`
from either scope compares against the same render ([§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)).
Omitting the bullet where the alias could not be judged would make the two
scopes disagree about one file: the member would write a block its own workspace
root then reports as drifted.

#### 9.1.1 A path-scoped check compares against the same render

A path is a third scope the same sentence covers. `grund check <path>` compares
the managed block's `### Chapter rules` against the render `grund check .`
compares against, because the render is built from the rules the run's
resolution scope read ([§FS-check.1.3.6.1](FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution)) — the project's ordinary scope with its
rule-kind homes, wherever the path points — and never from its report scope. A
path that holds no rule declaration therefore still renders every bullet, and
the `agents-init` finding the comparison raises reaches the narrowed report by
its code ([§FS-check.1.3.6.1](FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution)).

So the two runs agree about `AGENTS.md` in both directions: neither calls the
block `init` wrote stale, and both call a block missing a bullet stale, with the
same finding. Rendering from the report scope would turn such a path into a
bulletless section, and the narrowed run and the whole-tree run would then ask
for opposite bytes in one file, which no `AGENTS.md` can satisfy.

## 10. Documentation and executable examples

The release includes one guide at `docs/user-facing/rules.md`, one runnable
golden example at `examples/rules/`, links from the root README and
`examples/README.md`, and no second skill. The guide teaches opt-in and rationale
bodies, every subject and family, counts and modalities, every finding and both
channels, ordering, both deduplication directions, every command flag,
validation lifecycle, every explicit phase-1 absence, what a quantified subject
does not select, and the declaration a chapter-scoped citation rule reports
rather than passes over.

The guide has a marked `### Chapter rules` writing section. Both repository and
binary-embedded copies of `skills/grund-init/SKILL.md` contain a marked byte-
identical copy of that section and remain wholly byte-identical to one another.
The section includes every accepted family, every row of the [§FS-rules.3.5](FS-rules.md#35-strict-refusals) table
and the [§FS-rules.3.5.1](FS-rules.md#351-presence-name-whitespace-refusal) refusal, each with its exact rewrite or, where it offers
none, the `known kinds:` line `check --rule` prints after the reason
([§FS-rules.3.5.4.4](FS-rules.md#3544-where-no-form-is-offered)), the finding each example produces, and the exact
`unreached-declaration` error a chapter-scoped citation rule produces about a
declaration that has no such chapter, and the two actions
that answer it ([§FS-rules.2](FS-rules.md#2-subject-selectors),
[§FS-rules.checks.unreached-declaration](FS-rules.md#checksunreached-declaration-unreached-declaration)). Its leading sentence on
that reach is the specification's word for word, differing at most in where it
wraps, so neither can be reworded without the other. The section
also quotes the reason clause of the `invalid-rule` message an exact chapter
subject produces once its chapter is gone, and that quote is the binary's own
wording.

The runnable example contains at least one passing and one violated instance of
all five families. Its guide quotes every violated instance's exact finding and
its goldens cover `invalid-rule`, `chapter-cardinality`, rule-derived
`missing-citation`, ordinary and per-target `citation-cardinality`,
`uncited-unit`, `forbidden-citation` with its rule tail,
`suggested-citation`, `discouraged-citation`, and `unreached-declaration` for
both a single rule and a two-rule group; both channels and suggestion-
neutral exit behavior; two same-anchor off-count targets with shared-prefix
IDs; config-to-rule and rule-to-rule deduplication; and the refusal set.

Five independent pins prevent drift:

1. `examples/rules/expected.*` run through the shared e2e runner.
2. A marked-row extraction test submits every accepted/refused guide row to the
   released parser and asserts acceptance or exact refusal, rewrite,
   `known kinds:` line, code, and channel.
3. Asset-sync tests compare guide section to repository skill and whole
   repository skill to the embedded copy.
4. The managed block's existing re-render byte comparison checks generated
   guidance.
5. A quoted-message test runs the exact chapter subject over a declaration
   whose chapter is gone and asserts the section's quoted reason clause is
   part of the `invalid-rule` message the binary emits.

## 11. Functional architecture constraint

Only two representations cross the parsing/evaluation boundary. The sentence
front end receives a title, rule identity/anchor, and config vocabulary and
returns `ParsedRule`: origin, anchor, subject selector, modality, relation,
normalized target set, and cardinality. It may not know `RuleFacts`, evaluate,
deduplicate, or synthesize findings.

Fact producers return only complete, immutable, versioned `RuleFacts` as [§FS-rules.5.1](FS-rules.md#51-facts-and-identity)
defines. The Markdown producer is phase 1's only producer, but the scanner is
rule-blind and contributes structural records rather than evaluating a rule.
The logic engine evaluates only `ParsedRule` over `RuleFacts`, deduplicates
semantic constraints, and emits located findings through the shared report
boundary. It may not know sentence text, Markdown, scanner records, or file
layout beyond fact anchors. Architecture ground and dependency/replacement
tests specify the component placement separately.

## 12. Deliberate phase-1 absences

There is no `[settings]`, `config show --at`, path/folder/file subject,
exception phrase, `grund:allow` comment, definition, derived term, component
wildcard, wildcard subject alias, new command verb, suppression mechanism,
SCIP/LSIF ingestion, symbol vocabulary, on-disk fact format, or Datalog
runtime. Settings must reuse this selector parser and independently answer
[§DF-fmt-suppression.2.2](../decisions/functional/DF-fmt-suppression.md#22-an-in-text-region-not-a-rule-keyed-by-declaration-section). A future adjacent-site exception may rely on rule prohibitions retaining their exact citation-site anchors. A future program producer inherits opaque identities, versioned immutable complete snapshots, repository-relative anchors, committed offline input, and evaluator independence, but no exchange format is chosen here.

## checks: Checks

A check this specification raises is a section named by its diagnostic code
([§REQ-spec-section-names.code](../requirements/REQ-spec-section-names.md#code-a-check-is-named-by-its-diagnostic-code)). The four codes the rule families reuse or raise —
`invalid-rule`, `chapter-cardinality`, `citation-cardinality` and
`uncited-unit` — keep the positional addresses they shipped at
([§FS-rules.7](FS-rules.md#7-findings-and-channels)) until the migration's own
`FS-rules` slice moves them in beside the one below.

### checks.unreached-declaration: Unreached declaration

A declaration of a kind that a chapter-scoped citation rule over that kind
cannot reach. The rule's subject is `The <NAME> chapter of each <KIND>`, the
declaration is a local declaration of `<KIND>`, and it has no accepted
chapter at the section path `<NAME>` ([§FS-rules.2.1](FS-rules.md#21-a-chapters-name-is-its-whole-path)), so it contributes no unit
to the selection
([§FS-rules.2](FS-rules.md#2-subject-selectors)) and the rule's relation says
nothing about it.

One finding per semantic rule group per unreached declaration. The group's
contributing origins are the finding's `authority` and its message tail exactly
as every other rule finding's are
([§FS-rules.6](FS-rules.md#6-semantic-deduplication)), so two byte-identical
rules yield one finding naming both, and two rules that mean different things
yield one finding each. Nothing suppresses either.

It fires for the three positive citation families — outbound count
([§FS-rules.3.2](FS-rules.md#32-outbound-citation-count)), per-target coverage
([§FS-rules.3.3](FS-rules.md#33-per-target-coverage)) and inbound count
([§FS-rules.3.4](FS-rules.md#34-inbound-citation-count-and-prohibition)) — and
for no other family, for the two reasons
[§FS-rules.5.2](FS-rules.md#52-family-clauses) gives. Like every other
closed-world conclusion it is withheld from an incomplete snapshot
([§FS-rules.4](FS-rules.md#4-validation-lifecycle)).

It is located at the subject declaration's title line, because the chapter
title that would otherwise anchor it is the thing that is missing. At the
required level the message is:

```text
<declaration> has no <name> chapter, so <authority> cannot reach it; add the chapter, or narrow the rule to the declarations that have one; this became an error in grund 0.16.0
```

carried as an error on the ordinary `must` channel ([§FS-rules.7](FS-rules.md#7-findings-and-channels)). At the
recommended level the same row is a suggestion with the landed clause dropped,
so the message ends at `have one` and is visible only under `--suggestions`.

The two actions the message names are the only two that answer it: add the
chapter, or narrow the rule's subject to the declarations that have one. A
chapter-presence rule standing beside the citation rule is not a third — it
raises its own `chapter-cardinality`
([§FS-rules.7.2](FS-rules.md#72-chapter-cardinality)) and suppresses nothing,
so one absent chapter under a paired presence and citation rule prints both
lines, each naming its own rule.

The code is selectable on the same surfaces as every other
([§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits)), so
`--ignore unreached-declaration` is the opt-out for a repository that wants the
absence reported by nothing, and a `should` rule is the opt-out for one that
wants it reported without failing the run.
