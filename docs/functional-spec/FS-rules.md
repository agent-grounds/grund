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
- **whose clause** — `whose FIELD is VALUE` at the end of a quantified subject. It narrows the
  subject to the declarations of its kind whose scalar field `FIELD` holds `VALUE`
  ([§FS-rules.2.2](FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)).
- **outside the rule** — Said of a declaration of the subject's kind that the subject's whose
  clause does not select. It contributes no unit, no family reports it, and it is never
  unreached ([§FS-rules.2.2](FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)).
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

## requirements: What the rule language holds to

Three requirements on the rule language as a whole, each carrying a mark — *realized*, *directional* or *deferred* — as [§FS-config.requirements](FS-config.md#requirements-what-the-config-contract-holds-to) defines them. There is no fourth for what a requirement already states: that a repository opting into nothing sees the bytes it saw before ([§FS-rules.1](FS-rules.md#1-rule-declarations-and-opt-in)), and that a modality fixes its channel while a release moves a rule verdict only through a named window ([§FS-rules.7](FS-rules.md#7-findings-and-channels)), are [§REQ-backwards-compatibility.1](../requirements/REQ-backwards-compatibility.md#1-what-is-covered) and [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) applied to rules; that equal sentences over equal facts give equal findings ([§FS-rules.5](FS-rules.md#5-relational-meaning)) is [§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)'s; and that the editor reports a rule finding as `check` does ([§FS-rules.9](FS-rules.md#9-managed-guidance-and-editor-parity)) is [§REQ-one-answer.text.parity](../requirements/REQ-one-answer.md#textparity-the-editor-answers-as-the-command-line-does)'s.

### requirements.unseen: Nothing a rule governs passes unseen — realized, one case deferred

A rule that cannot run is reported at its own heading and never silently omitted: the run keeps its other findings and exits nonzero, and `init` writes no block over an invalid rule ([§FS-rules.4](FS-rules.md#4-validation-lifecycle)); a sentence the run's scope cannot verify is reported all the same ([§FS-rules.4.1](FS-rules.md#41-a-rule-this-scope-cannot-verify)); and a run narrowed to the rule-produced codes keeps every `invalid-rule` row ([§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits)). A `must` or `should` rule whose subject names a kind with no local declaration says it applies to nothing, scoped or narrowed, rather than passing ([§FS-check.2.2.1.4](FS-check.md#2214-a-rules-obligation)); a whose clause that selects none of the kind's declarations yet describes the project's state, not a broken rule, and says nothing. A declaration a whose clause leaves outside the rule does not pass unseen either: the clause selects by a scalar field the kind declares, so the declaration's field holds another value of its `one_of`, or is absent where the field is `may` and absence is allowed, or is reported on its own — an absent `must`, `warn` or `should` field as `missing-field` and a value outside `one_of` as `invalid-field-value`, at every strength ([§FS-fields.strengths.table](FS-fields.md#strengthstable-which-strength-each-finding-takes), [§FS-rules.2.2](FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)). A run narrowed by `--only-rule` drops those field findings with everything else the trial sentence did not author, so it shows nothing for such a declaration ([§FS-rules.8](FS-rules.md#8-command-surfaces)). A declaration a quantified chapter subject cannot reach is reported where it is declared rather than passed over ([§FS-rules.checks.unreached-declaration](FS-rules.md#checksunreached-declaration-unreached-declaration)), and an incomplete snapshot yields no closed-world absence or count and keeps exit 2 ([§FS-rules.4](FS-rules.md#4-validation-lifecycle)). A chapter-presence rule beside a chapter-scoped citation rule suppresses nothing, but it counts display names where the citation rule selects handles ([§FS-rules.3.1](FS-rules.md#31-chapter-presence), [§FS-rules.2.1](FS-rules.md#21-a-chapters-name-is-its-whole-path)), so the two report one absence together only where chapters are titled by their handles, as `## terms: Terms` is. A `[citations]` direction absorbs a duplicate rule's finding only where it reports the same unit itself, so a unit only the rule fails, because its only citation of the target kind does not resolve, is reported as the rule's ([§FS-rules.6](FS-rules.md#6-semantic-deduplication)). One case is **deferred**, owed to its own change: a trial sentence that duplicates a `[citations]` direction authors nothing for a unit the direction itself reports, so `check --rule … --only-rule` prints `success` there while the tree's own findings stand ([§FS-rules.8](FS-rules.md#8-command-surfaces)). Serves [§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work), on the model [§REQ-no-missed-citation.1](../requirements/REQ-no-missed-citation.md#1-no-silent-skips) sets for a file the scan cannot read.

### requirements.addition: The language grows by addition — directional

A family, selector or producer added from here on is added beside the ones that exist and leaves every accepted sentence meaning what it meant, so a refusal's rewrite stays accepted when it is pasted back ([§FS-rules.3](FS-rules.md#3-the-five-sentence-families), [§FS-rule-refusals.sentences](FS-rule-refusals.md#sentences-strict-refusals)). In the change that adds it, a new family answers the two questions [§FS-rules.5.2](FS-rules.md#52-family-clauses) answers for the five: whether it reports the declarations its subject leaves unreached ([§FS-rules.checks.unreached-declaration](FS-rules.md#checksunreached-declaration-unreached-declaration)), and which field of a chapter its `NAME` reads — the handle path a chapter subject selects ([§FS-rules.2.1](FS-rules.md#21-a-chapters-name-is-its-whole-path)) or the display name a presence rule counts ([§FS-rules.3.1](FS-rules.md#31-chapter-presence), [§FS-rules.5.1.1](FS-rules.md#511-a-chapters-display-name-is-the-label-its-author-wrote)). The five already answer the second both ways, as specified, so there is no default for the next one to inherit. A code the family raises joins the rule-produced codes of [§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits) in the same change, so a run narrowed to what rules find still keeps every `invalid-rule` row. This binds families added from here on; it is not a claim about the five. Serves [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path).

### requirements.phase-one: What phase 1 leaves out is owed to its own change — deferred

Every absence [§FS-rules.12](FS-rules.md#12-deliberate-phase-1-absences) lists — path, folder and file subjects, `[settings]`, an exception phrase, definitions and derived terms, component and subject wildcards, SCIP/LSIF ingestion and the rest — is **deferred**, and each is owed to its own change: no roadmap item names one, and the decision behind the language defers them rather than reserving syntax for them ([§DF-chapter-rules.2](../decisions/functional/DF-chapter-rules.md#2-decision)). Until one lands, a sentence that needs it is refused rather than half-parsed ([§FS-rule-refusals.sentences](FS-rule-refusals.md#sentences-strict-refusals)), so its arrival changes no accepted sentence; what the arrival must keep is already written, at [§FS-rules.12](FS-rules.md#12-deliberate-phase-1-absences) for settings, an adjacent-site exception and a program producer, and at [§FS-rules.2](FS-rules.md#2-subject-selectors) for a component wildcard. Serves [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path).

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
headings are not units. The front-matter coordinates of a declaration whose kind
declares a `front-matter` field are chapter units too, each its slice, selected by the
whole path `front-matter` or `front-matter.<key>` ([§FS-fields.front-matter.surfaces](FS-fields.md#front-mattersurfaces-what-each-surface-does-with-them)).

The four accepted subject spellings, the whose clause either quantified one may
end in, and the one shorthand only a selector accepts, are:

| Authored subject | Selector and match |
|---|---|
| `Each KIND` | every local declaration of that configured citable kind |
| `ID` | the one local declaration with that full ID |
| `The NAME chapter of each KIND` | the chapter at that whole path of every local declaration of the kind that has one |
| `ID.NAME[.NAME…]` | the one local named chapter with that exact coordinate |
| `Each KIND whose FIELD is VALUE`, `The NAME chapter of each KIND whose FIELD is VALUE` | what the subject without the clause selects, narrowed to the declarations of the kind whose scalar field `FIELD` holds `VALUE` ([§FS-rules.2.2](FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)) |
| `KIND.NAME[.NAME…]` | selector only: means `The NAME chapter of each KIND`, and a rule sentence refuses it |

`KIND.NAME` selects by configured kind and `list --glob 'KIND-*.NAME'` by rendered ID
text ([§FS-coordinate-globs](FS-coordinate-globs.md#fs-coordinate-globs-a-coordinate-glob-selects-catalog-coordinates-by-pattern)), so the two list the same rows wherever the kind's IDs render
as `KIND-…`, and part on an off-grammar spelling such as `FS-old`, which only the glob
reaches, and under a format whose `{kind}` is followed directly by a placeholder, where the
glob that lists them is `KIND*.NAME`.

In chapter subjects, `NAME` selects the section handle, not its display name.
For `## goal: Goal and hypothesis`, `The goal chapter of each BENCH` selects
the `goal` coordinate even though its display name is `Goal and hypothesis`.
Presence rules compare the other field ([§FS-rules.3.1](FS-rules.md#31-chapter-presence)). A chapter subject's
`NAME` is the chapter's whole path from its declaration, never only that
path's last component ([§FS-rules.2.1](FS-rules.md#21-a-chapters-name-is-its-whole-path)).

A token exactly equal to a configured citable kind name is the quantified kind
selector. After `Each ` or `of each `, and only there, such a token may be
followed by a whose clause ([§FS-rules.2.2](FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)); the selector shorthand takes
none, so `SWR whose verification is Test` is a longer token. Any longer token must parse as a full ID under that kind's effective
grammar. Named components require `[id] named_sections = true` and obey the
configured named-section grammar. An exact literal subject must resolve to one
unit; zero matches or ambiguity produces `invalid-rule` ([§FS-rules.7.1](FS-rules.md#71-invalid-rule)). A quantified
subject selects only the units that exist: a chapter subject selects the named
chapter of every local declaration of the kind that has one, so a declaration
without it contributes no unit rather than a failing one. Contributing no unit
is not being outside the rule: a declaration is outside the rule only where the
subject's whose clause does not select it
([§FS-rules.2.2](FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)), and a selected declaration that lacks the chapter still
contributes no unit and is still reported. A quantified chapter subject selects only the
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
path subjects are refused ([§FS-rule-refusals.sentences](FS-rule-refusals.md#sentences-strict-refusals)). The future component wildcard, if admitted,
will consume exactly one accepted section component; phase 1 accepts no such
token.

Object kind targets use the existing target-entry grammar of
[§FS-config.3.9.3](FS-config.md#393-alias-matching) unchanged, including a
pinned `alias/KIND` and `*/KIND`. A target pinned at the rule's own member by
that member's whole alias path, nested or not (`group/api/AR` in the member
whose alias path is `group/api`), matches exactly what the bare kind matches,
because the entry splits at its last `/`. Alternatives joined with `or` are one target
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

A whose clause's `FIELD` is a whole path in the same way
([§FS-rules.2.2](FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)): it names the direct section at exactly that path, and
never a nested section that only shares its last component.

### 2.2 A whose clause selects on a scalar field's value

A quantified subject may end in one whose clause: `Each KIND whose FIELD is
VALUE`, and `The NAME chapter of each KIND whose FIELD is VALUE`. The clause
belongs to the subject, so every family that takes the subject without it takes
it with it: all five for `Each KIND`, and the four citation families for the
chapter form, because presence takes no chapter subject
([§FS-rules.3.1](FS-rules.md#31-chapter-presence)). An exact ID takes no clause, because it already names one
declaration, and neither does the selector-only `KIND.NAME` shorthand, because a
selector already accepts both full spellings. An object kind takes none either:
a whose clause narrows only a rule's subject, and a target set stays the kinds
of [§FS-rules.2](FS-rules.md#2-subject-selectors)
([§DF-field-value-subjects](../decisions/functional/DF-field-value-subjects.md#df-field-value-subjects-a-rule-subject-may-select-on-a-scalar-fields-value-with-one-whose-clause)).

The clause is read from the subject the modality split leaves
([§FS-rules.3.6](FS-rules.md#36-where-a-sentences-subject-ends)). It opens at the ` whose ` after the kind, and its first
` is `, or an ` is` that ends the subject, divides it: `FIELD` is the text
before, `VALUE` the text after, to the end of the subject.

- **`FIELD`** is read as a chapter's `NAME` is, as a whole section path
  ([§FS-rules.2.1](FS-rules.md#21-a-chapters-name-is-its-whole-path)), and names a scalar field of the kind: a version-2 field with
  `one_of` ([§FS-fields.content.one-of](FS-fields.md#contentone-of-a-scalars-value-is-its-display-name)). A field is a direct section
  ([§FS-fields.identity.name](FS-fields.md#identityname-a-field-is-the-direct-section-of-its-name)), so today only a one-component path names one. A
  prose field, a values field, and anything in a version-1 config
  ([§FS-fields.v1](FS-fields.md#v1-version-1-is-unchanged)) are not scalars.
- **`VALUE`** must equal one `one_of` entry byte for byte
  ([§FS-fields.identity.match](FS-fields.md#identitymatch-titles-and-values-match-byte-for-byte)), so `is test` does not match `Test`. It holds one
  value: ` or ` inside it is part of that value, never a second one.

`FIELD` reads a section's handle path and `VALUE` its display name, which is the
answer [§FS-rules.requirements.addition](FS-rules.md#requirementsaddition-the-language-grows-by-addition--directional) asks of an addition. A front-matter key
is not a field this clause reads; `whose front-matter.<key> is …` is owed to
#1042, whose values are the key's text rather than a display name.

What the clause selects is the declarations of `KIND` whose first direct
section at `FIELD` has `VALUE` as its display name — the same answer
[§FS-fields.checks.invalid-field-value](FS-fields.md#checksinvalid-field-value-invalid-field-value) reads — and, for the chapter form,
their chapter at `NAME`. A declaration of the kind whose field holds another
value, or that has no section at `FIELD`, is **outside the rule**: it
contributes no unit, no family reports it, and it is never
[§FS-rules.checks.unreached-declaration](FS-rules.md#checksunreached-declaration-unreached-declaration). A selected declaration that lacks the
chapter is unreached exactly as it is without the clause. Nothing the clause
leaves out passes unseen
([§FS-rules.requirements.unseen](FS-rules.md#requirementsunseen-nothing-a-rule-governs-passes-unseen--realized-one-case-deferred)).

A clause the grammar does not accept is refused for the part that failed, never
as an unknown kind; the reasons are
[§FS-rule-refusals.sentences.whose](FS-rule-refusals.md#sentenceswhose-a-whose-clause-is-refused-for-the-part-that-failed). The decision is
[§DF-field-value-subjects](../decisions/functional/DF-field-value-subjects.md#df-field-value-subjects-a-rule-subject-may-select-on-a-scalar-fields-value-with-one-whose-clause).

## 3. The five sentence families

The normative language is the following closed controlled-English grammar.
Fixed words and kind names are case-sensitive, every rule ends with exactly one
terminal `.`, and one sentence contains exactly one semantic verb. `N` is a
positive base-10 integer no larger than the largest supported count, the
largest value of the platform's unsigned machine word (`18446744073709551615`
on a 64-bit platform); a larger `N` is refused for its range
([§FS-rule-refusals.sentences.count-range](FS-rule-refusals.md#sentencescount-range-a-count-above-the-largest-supported-count-is-refused-for-its-range)). `one` takes singular `chapter`; every numeric `N`
other than one takes `chapters`. A lower bound and an exact count spell one as
the word `one` and refuse the numeral, so `at least 1` and `exactly 1` are not
sentences; an upper bound spells it as the numeral, so `at most 1` is the only
spelling of a ceiling of one except for per-target coverage, whose count noun
is `times` and whose ceiling of one is `at most once`
([§FS-rules.3.3](FS-rules.md#33-per-target-coverage)). The only accepted families are the following.

### 3.1 Chapter presence

```text
<subject> must|should have at least one <NAME> chapter.
<subject> must|should have at least N <NAME> chapters.
<subject> must|should have at most N <NAME> chapter|chapters.
<subject> must|should have exactly one <NAME> chapter.
<subject> must|should have exactly N <NAME> chapters.
```

The subject may be a kind, a kind narrowed by a whose clause
([§FS-rules.2.2](FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)), or an exact declaration, not a chapter. The rule counts
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
A whole-kind presence rule stays valid under a version-2 config, beside the
fields that config may declare; where a field expresses the rule, one shared
absence is reported once ([§FS-rules.6](FS-rules.md#6-semantic-deduplication)).

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
<subject> must|should cite each <KIND> at most once.
<subject> must|should cite each <KIND> at most N times.
<subject> must|should cite each <KIND> exactly once.
<subject> must|should cite each <KIND> exactly N times.
```

In this family, all three bounds spell one as `once`; numeric `N times` is
accepted only above one. `at most once` normalizes to no minimum and maximum
one: zero or one physical citation to a target passes, and two to that same
target produce `citation-cardinality`. Counting remains by physical site,
not by distinct target. The finding keeps `requires at most 1`.
The compatibility decision is [§DF-per-target-once-ceiling](../decisions/functional/DF-per-target-once-ceiling.md#df-per-target-once-ceiling-spell-a-per-target-ceiling-of-one-as-at-most-once).

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

Moved to [§FS-rule-refusals.sentences](FS-rule-refusals.md#sentences-strict-refusals).

#### 3.5.1 Presence-name whitespace refusal

Moved to [§FS-rule-refusals.sentences.whitespace](FS-rule-refusals.md#sentenceswhitespace-presence-name-whitespace-refusal).

#### 3.5.2 A subject that needs named sections is answered with one they make valid

Moved to [§FS-rule-refusals.sentences.named-sections](FS-rule-refusals.md#sentencesnamed-sections-a-subject-that-needs-named-sections-is-answered-with-one-they-make-valid).

#### 3.5.3 A chapter path is refused for the component that failed

Moved to [§FS-rule-refusals.sentences.chapter-path](FS-rule-refusals.md#sentenceschapter-path-a-chapter-path-is-refused-for-the-component-that-failed).

#### 3.5.4 An accepted form is the typed sentence with the failed part replaced

Moved to [§FS-rule-refusals.sentences.rewrite](FS-rule-refusals.md#sentencesrewrite-an-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced).

##### 3.5.4.1 Only the failed part is replaced

Moved to [§FS-rule-refusals.sentences.rewrite.replaced](FS-rule-refusals.md#sentencesrewritereplaced-only-the-failed-part-is-replaced).

##### 3.5.4.2 Only what can be recovered is supplied

Moved to [§FS-rule-refusals.sentences.rewrite.recovered](FS-rule-refusals.md#sentencesrewriterecovered-only-what-can-be-recovered-is-supplied).

##### 3.5.4.3 A form is parsed before it is offered

Moved to [§FS-rule-refusals.sentences.rewrite.reparsed](FS-rule-refusals.md#sentencesrewritereparsed-a-form-is-parsed-before-it-is-offered).

##### 3.5.4.4 Where no form is offered

Moved to [§FS-rule-refusals.sentences.rewrite.no-form](FS-rule-refusals.md#sentencesrewriteno-form-where-no-form-is-offered).

##### 3.5.4.5 The rewrite each refusal makes

Moved to [§FS-rule-refusals.sentences.rewrite.by-refusal](FS-rule-refusals.md#sentencesrewriteby-refusal-the-rewrite-each-refusal-makes).

##### 3.5.4.6 Exact rows where `FS` is the only kind

Moved to [§FS-rule-refusals.sentences.rewrite.fs-only](FS-rule-refusals.md#sentencesrewritefs-only-exact-rows-where-fs-is-the-only-kind).

#### 3.5.5 Any clauses joined by `and` are refused as a conjunction

Moved to [§FS-rule-refusals.sentences.conjunction](FS-rule-refusals.md#sentencesconjunction-any-clauses-joined-by-and-are-refused-as-a-conjunction).

### 3.6 Where a sentence's subject ends

A sentence's subject is the text before its modality. The modality is found in
a fixed order, not at the earliest position: the first ` must not `, else the
first ` should not `, else the first ` must `, else the first ` should `. So
`The should chapter of each FS must cite at least one FS.` has the subject
`The should chapter of each FS`, because ` must ` is tried before ` should `,
and `The must chapter of each FS should not cite any AR.` has the subject
`The must chapter of each FS`, because ` should not ` is tried before ` must `.
Where the split reaches a chapter's modal-word name first, as in
`The must chapter of each FS must cite at least one FS.`, the subject left is
`The`, and the refusal names the chapter ([§FS-rule-refusals.sentences.modal-chapter](FS-rule-refusals.md#sentencesmodal-chapter-a-chapter-named-after-a-modal-word-is-refused-for-that-name)).
The split cuts a whose clause's `VALUE` as it cuts a chapter's `NAME`: spaces, a
capitalised `Must fix`, ` and ` and ` or ` survive it, a lowercase modal word
inside a value ends the subject there, and the refusal names the value
([§FS-rule-refusals.sentences.whose](FS-rule-refusals.md#sentenceswhose-a-whose-clause-is-refused-for-the-part-that-failed)). No value is matched against the
configuration before the split, so a rule and a selector split alike.

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
and before scanning, and is printed in the refused-input shape of
[§FS-errors.2.5](FS-errors.md#25-refused-input), with the accepted form, or none, that
[§FS-rule-refusals.sentences.rewrite](FS-rule-refusals.md#sentencesrewrite-an-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced) builds. A syntactically valid
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
trees that pass without them; those findings warned in 0.16.1 and 0.17.0 and
are errors from 0.18.0 ([§FS-rules.7.8](FS-rules.md#78-a-newly-counted-section-citation-warns-until-0180)).

#### 5.1.1 A chapter's display name is the label its author wrote

A `chapter` fact's `display_name` is the label the author wrote after the
chapter's coordinate: `Terms` in `## terms: Terms`. A nested chapter's
coordinate is its whole path, so `### requirements.terms: Terms` is
`chapter(c, requirements.terms, Terms)`, and its label is `Terms` as well. It
is the same label whether the declaration's body lives in a Markdown file or in
a source doc-comment, because the comment wrapper a heading sits behind is never
part of it
([§FS-languages.comment-wrapper.headings](FS-languages.md#comment-wrapperheadings-the-wrapper-of-a-heading-line)).
`/// ## terms: Terms` in a Rust file and
`## terms: Terms` in a Markdown file are therefore the same fact,
`chapter(c, terms, Terms)`, and a presence rule naming `Terms` counts either
one ([§FS-rules.3.1](FS-rules.md#31-chapter-presence)). The same label is the
title of that chapter's `list --selector` row
([§FS-rules.8](FS-rules.md#8-command-surfaces)). The `title` a
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

A subject with a whose clause ([§FS-rules.2.2](FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)), `whose F is V`, narrows the
declarations every clause ranges over to the selected ones, written over the
existing facts with no new relation:

```text
selected(d) :- decl(d, K), contains(d, f), chapter(f, F, V).
```

`contains(d, f)` is direct containment and `chapter(f, F, V)` joins the field's
section path and its display name, so `selected` holds exactly where
[§FS-fields.checks.invalid-field-value](FS-fields.md#checksinvalid-field-value-invalid-field-value) reads the value `V`. Under a whose clause
`S` holds only the selected declarations, or their chapters at `N`, and every
`decl(d, K)` premise below reads `selected(d)`, so a declaration outside the
rule is in no family's `S` and in no `unreached`. Without a clause every
declaration of `K` is selected, as before.

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

Config-to-rule deduplication is intentionally narrower, and like field-to-rule
pairing below it is decided on the findings, unit by unit, rather than by
discarding the rule before it runs. It applies only where an existing
`[citations]` entry and a rule express the same local bare citing kind,
declaration-wide unit over every declaration of that kind, modality/level, `cite` relation, normalized target
entry, and cardinality. Where, for one unit or site, config reports its existing
`missing-citation`, `suggested-citation`, `forbidden-citation`, or
`discouraged-citation` finding and the rule reports the same code there on the
same channel, config's finding wins byte-for-byte and the rule's is dropped: no
rule ID is appended, its `authority` stays empty, and JSON gains no `sites`
list. `[citations]` remains authored, rendered, and interpreted as before. A
rule that loses to config this way authored nothing for that unit, so a trial
sentence duplicating a `[citations]` direction that reports the same unit is
invisible to a report scoped by `authority`
([§FS-rules.8](FS-rules.md#8-command-surfaces)). A subject with a whose clause
([§FS-rules.2.2](FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)) is never kind-wide, so it never folds into a direction: beside
`[citations.SWR] must = ["TC"]`, `Each SWR whose verification is Test must
cite at least one TC.` reports its own finding for a selected declaration
beside the direction's, as an exact-ID subject does. Two rules whose subjects
carry the same clause are equal and deduplicate into one finding naming both;
a rule with a clause and one without are two constraints.

The two need not agree. A direction
([§FS-config.3.9.1.1](FS-config.md#3911-obligations-and-prohibitions)) counts a
citation by the kind its token names whether or not the token resolves, and the
token keeps its own `unknown reference` or `missing section` finding
([§FS-check.3.1](FS-check.md#31-dangling-citation),
[§FS-check.3.2](FS-check.md#32-missing-section)); a rule counts only a citation
that resolves
([§FS-rules.5.1](FS-rules.md#51-facts-and-identity)). So a declaration whose only
`REQ` citation is the dangling `<§>REQ-missing` or a missing section such as
`<§>REQ-b.9` satisfies `[citations.FS] must = ["REQ"]` and fails `Each FS must
cite at least one REQ.`, and a dangling `<§>mem/AR-x` into a workspace member
does the same to a `mem/AR` direction and rule.
A defect only the rule sees is still reported, as the rule's own finding with
the rule's `authority`, beside the token's `unknown reference` or `missing
section`, on every channel the rule's level reaches. Adding a `[citations]`
entry therefore never hides a finding the rule alone would report, and a trial
sentence scoped by `--only-rule` reports it and fails the run. The decision is
[§DF-direction-absorbs-findings-not-rules](../decisions/functional/DF-direction-absorbs-findings-not-rules.md#df-direction-absorbs-findings-not-rules-a-citations-direction-absorbs-a-duplicate-rules-finding-for-a-unit-it-reports-not-the-rule).

Field-to-rule pairing is narrower still, and it happens after evaluation
rather than before it. A version-2 field expresses a chapter-presence rule when
its `title`, or its name where it has no title, equals the rule's chapter `NAME`
case-insensitively ([§FS-fields.rules](FS-fields.md#rules-fields-beside-chapter-rules)). Where, for one declaration, that rule
counts zero matching chapters and the field reports `missing-field` or
`mistitled-field` on the same channel, the field's finding stands and the
rule's `chapter-cardinality` for that declaration is dropped: no rule ID is
appended and the field's `authority` stays `null`, as config wins above. A
defect only one of them sees is still reported: two chapters titled Terms under
different names fail the rule and not the field, `## glossary: Terms` fails the
field and not the rule, and a `must` rule beside a `warn` field reports both, so
adding a field never weakens a rule. The pairing runs where conformance and
rule findings are merged, after the chapter-rule pass and before `--only` and
`--ignore`, so the language server sees the same findings.

## 7. Findings and channels

All rule findings use the ordinary text and NDJSON schemas of
[§FS-errors](FS-errors.md#fs-errors-grund-emits-messages-in-fixed-shapes).
`must` and `must not` are errors. `should` and `should not` are suggestions:
they appear only with `--suggestions`, carry `"channel":"suggestion"` in JSON,
and never affect exit status. Structural recommendations retain their
structural code on that channel. The one rule finding that was a warning is an
error from 0.16.0 ([§FS-rules.7.7](FS-rules.md#77-one-required-level-finding-is-a-warning-until-0160)), and those a newly counted section citation
alone produces are errors from 0.18.0 ([§FS-rules.7.8](FS-rules.md#78-a-newly-counted-section-citation-warns-until-0180)); the findings of a rule declared through a stub whose target the scan does not read, and those only
the citations and chapters such a target lends produce, and those only a citation of a
name under a number raises, are errors from 0.19.0 ([§FS-rules.7.10](FS-rules.md#710-a-rule-declared-through-a-stub-warns-until-0190), [§FS-rules.7.11](FS-rules.md#711-a-citation-or-chapter-read-through-a-stub-warns-until-0190),
[§FS-rules.7.12](FS-rules.md#712-a-finding-only-a-citation-of-a-name-under-a-number-raises-warns-until-0190)), and so are those a target pinned at the rule's own nested alias
path alone produces ([§FS-rules.7.9](FS-rules.md#79-a-target-pinned-at-its-own-nested-alias-warns-until-0190)), so no `must` or `must not` rule finding is a
warning.

### 7.1 Invalid rule

`invalid-rule` is located at the rule heading. A configured parse failure is:

```text
<RULE-ID> is not a valid rule: <reason>; accepted form: <rewrite>
```

The rewrite is the sentence as authored with the failed production replaced
([§FS-rule-refusals.sentences.rewrite](FS-rule-refusals.md#sentencesrewrite-an-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced)), after the label [§FS-errors.2.5](FS-errors.md#25-refused-input) gives it.
A parse failure that offers no form, every refusal [§FS-rule-refusals.sentences.rewrite.by-refusal](FS-rule-refusals.md#sentencesrewriteby-refusal-the-rewrite-each-refusal-makes) says
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
; expected display name "<name>" (case-insensitive, not section handle); observed direct chapters: <observed>; add, remove or retitle chapters until the count fits
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
<subject> must cite <target-set> (<RULE-ID>); add such a citation within it
```

Other ordinary counts and every per-target `cite each` miss use
`citation-cardinality`, located at the subject declaration or chapter title:

```text
<subject> cites <target-set> <actual> times; <RULE-ID> requires <count>; add or remove those citations until the count fits
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
<subject> is cited by <source-set> <actual> times; <RULE-ID> requires <count>; add or remove the citations of it until the count fits
```

The code covers zero, surplus, and other off-count inbound cardinalities. A
chapter subject whose declaration has no such chapter reports
`unreached-declaration` at the declaration instead, on the same reading as
[§FS-rules.7.3](FS-rules.md#73-outbound-citation-cardinality)'s.

### 7.5 Prohibition and recommendation reuse

A hard prohibition reuses [§FS-check.3.12](FS-check.md#312-forbidden-citation)'s
site-anchored `forbidden-citation`, with `(<RULE-ID>)` for `(citation direction)`
and index-entry repairs governed by [§FS-check.3.12.2](FS-check.md#3122-repair-advice-preserves-a-required-index-entry).
Negative recommendations reuse `discouraged-citation`; ordinary positive ones,
`suggested-citation`. Ordinary config-only bytes stay unchanged.

### 7.6 Selection, JSON, ordering, and exits

`--only` and `--ignore` accept `invalid-rule`, `chapter-cardinality`,
`citation-cardinality`, `uncited-unit`, and `unreached-declaration` like every
other public code; their
value grammar takes codes only and never a rule identity
([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)).

The **rule-produced codes** are the nine a rule finding can carry: the four
only rules produce — `chapter-cardinality`, `citation-cardinality`,
`uncited-unit`, and `unreached-declaration` — the four rules share with
citation directions ([§FS-rules.7.3](FS-rules.md#73-outbound-citation-cardinality),
[§FS-rules.7.5](FS-rules.md#75-prohibition-and-recommendation-reuse)) —
`missing-citation`, `forbidden-citation`, `discouraged-citation`, and
`suggested-citation` — and `empty-citation-obligation`, the line a `must` or
`should` rule shares with the `[citations]` table when its subject's kind has no
local declaration
([§FS-check.2.2.1.4](FS-check.md#2214-a-rules-obligation)), so `--only
empty-citation-obligation` keeps the `invalid-rule` row of a rule that could not
say so. `invalid-rule` is selectable like every other public code,
and is also selected whenever a code from that set is: a run narrowed to what
rules find keeps every `invalid-rule` row, because a skipped rule is a check that
run did not evaluate
([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)).
`--ignore invalid-rule` still removes them. Every `empty-citation-obligation`
row is carried the same way, because a rule whose subject's kind has no local
declaration is a check that run judged nothing with ([§FS-check.2.2.1.4](FS-check.md#2214-a-rules-obligation)), and
`--ignore empty-citation-obligation` removes those.

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
retained by a report scoped to it. So does the line saying a `must` or `should`
rule applies to nothing ([§FS-check.2.2.1.4](FS-check.md#2214-a-rules-obligation)): it names its one rule, and a trial
sentence's line is retained by `--only-rule`. A finding no rule authored — a
citation direction, a dangling or duplicate declaration, any other run-level
warning — carries an empty `authority`.

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

The ramp this heading names is spent: it opened in grund 0.16.1 and closed in
grund 0.18.0, where a required-level finding that exists only because
[§FS-rules.5.1](FS-rules.md#51-facts-and-identity) counts a citation to a numbered section became an error on the
ordinary `must` channel. Such a finding is now the same code's ordinary error,
its message byte for byte the one a finding of that code without such a site
prints, with no clause after it: the code was an error throughout and only these
findings warned, so no finding is left to tell apart and the landed change has no
line of its own for the release guard to read, as with
[§FS-distribution.4.2.4](FS-distribution.md#424-the-scalar-clause-is-the-refs-warnings)'s scalar clause. What counts is specified at
[§FS-rules.5.1](FS-rules.md#51-facts-and-identity), and the decision and its completed route are
[§DF-section-citation-counts-in-rules](../decisions/functional/DF-section-citation-counts-in-rules.md#df-section-citation-counts-in-rules-a-resolved-citation-to-a-numbered-section-counts-in-chapter-rules). This address, heading text included, is
kept only so citations written during the window still resolve.

### 7.9 A target pinned at its own nested alias warns until 0.19.0

The window this heading names is spent: it opened in grund 0.18.0 and closed in grund 0.19.0,
where every required-level finding that exists only because a target pinned at the rule's own
member by a nested alias path now matches that member's declarations ([§FS-rules.2](FS-rules.md#2-subject-selectors)) became an
error on the ordinary `must` channel. Such a finding is now the same code's ordinary error, its
message the one the warning carried with `; a target pinned at the rule's own nested alias path
now matches, and this warning becomes an error in grund 0.19.0` gone and no clause in its place,
and it moves the exit status as any error does ([§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits)):

```text
group/api/docs/api/API-a.md:3: error: API must not cite group/api/AR (RULE-y) — re-point the citation or downgrade it to a plain Markdown link
```

What a pinned target matches is specified at [§FS-config.3.9.3](FS-config.md#393-alias-matching), and the decision and its completed
route are [§DF-nested-alias-rule-targets](../decisions/functional/DF-nested-alias-rule-targets.md#df-nested-alias-rule-targets-a-rule-target-pinned-at-its-own-nested-alias-matches-like-the-bare-kind) and [§DF-nested-alias-targets-land-in-0-19](../decisions/functional/DF-nested-alias-targets-land-in-0-19.md#df-nested-alias-targets-land-in-0-19-a-finding-a-target-pinned-at-its-own-nested-alias-alone-produces-is-an-error-from-0190). This
address, heading text included, is kept only so citations written during the window still
resolve.

### 7.10 A rule declared through a stub warns until 0.19.0

The window this heading names is spent: it opened in grund 0.18.0 and closed in grund 0.19.0,
where every required-level finding of a rule declared through a stub whose target is outside
`[scan] include` became an error on the ordinary `must` channel. Such a finding is now the same
code's ordinary error, its message byte for byte the one the same rule declared inline prints,
with no clause after it, and it moves the exit status as any error does ([§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits)):

```text
docs/fs/a.md:3: error: FS must not cite GOAL (RULE-r) — re-point the citation or downgrade it to a plain Markdown link
```

A rule's sentence and rationale are read from the declaration its ID pairs with
([§FS-declarations.stubs.pairing.1](FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned)), so a rule written as a stub in its kind's home is the rule
its target declares, the sentence `grund <ID> --brief` prints, whether or not the scan reads
the target; a target whose sentence is no rule is `invalid-rule` at the stub's line
([§FS-rules.7.1](FS-rules.md#71-invalid-rule)). The decision and its completed route are [§DF-stubs-by-home-and-at-the-target](../decisions/functional/DF-stubs-by-home-and-at-the-target.md#df-stubs-by-home-and-at-the-target-a-rule-declared-through-a-stub-applies-now-and-a-stub-is-read-by-its-home-and-shown-at-its-target-from-0190).
This address, heading text included, is kept only so citations written during the window still
resolve.

### 7.11 A citation or chapter read through a stub warns until 0.19.0

The window this heading names is spent: it opened in grund 0.18.0 and closed in grund 0.19.0,
where every required-level finding that only the citations and chapters a stub's target
outside `[scan] include` lends produce became an error on the ordinary `must` channel. Such a
finding is now the same code's ordinary error, its message byte for byte the one the same tree
with the target scanned prints, with no clause after it, at the line that tree reports it: a
prohibition's `forbidden-citation` at the lent citation's line in the target, and a count or
chapter count at the declaration's line there ([§FS-declarations.stubs.pairing.4](FS-declarations.md#stubspairing4-every-surface-shows-a-paired-declaration-at-its-target)):

```text
src/x.rs:3: error: FS must not cite GOAL (--rule) — re-point the citation or downgrade it to a plain Markdown link
src/x.rs:1: error: FS-x has 2 Terms chapters; --rule requires at most 1; expected display name "Terms" (case-insensitive, not section handle); observed direct chapters: "terms: Terms", "more: Terms"; add, remove or retitle chapters until the count fits
```

The same holds for the `[citations]` findings the lent citations produce ([§FS-check.3.11](FS-check.md#311-missing-required-citation),
[§FS-check.3.12](FS-check.md#312-forbidden-citation)). What a target lends is specified at [§FS-declarations.stubs.target.5](FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged), and the
decision and its completed route are [§DF-stub-target-citations-and-chapters](../decisions/functional/DF-stub-target-citations-and-chapters.md#df-stub-target-citations-and-chapters-a-stubs-target-lends-the-stubbed-declaration-its-citations-and-chapters-and-what-that-newly-raises-warns-until-0190).
This address, heading text included, is kept only so citations written during
the window still resolve.

### 7.12 A finding only a citation of a name under a number raises warns until 0.19.0

The mixed-citation window shipped in 0.18.0 and closes in 0.19.0. A resolved
mixed citation counts as an ordinary site under [§FS-rules.5.1](FS-rules.md#51-facts-and-identity). Required
prohibitions and upper-bound findings, including outbound, per-target and
inbound `at most` and `exactly` counts, use the ordinary errors, messages and
exit behavior ([§FS-rules.7.5](FS-rules.md#75-prohibition-and-recommendation-reuse), [§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits)), with no mixed-path clause. This
also applies to configured citation directions ([§FS-check.3.11](FS-check.md#311-missing-required-citation),
[§FS-check.3.12](FS-check.md#312-forbidden-citation)).

Only mixed-path relief ends. The nested-alias window ([§FS-rules.7.9](FS-rules.md#79-a-target-pinned-at-its-own-nested-alias-warns-until-0190)), the
stubbed-rule and lent-target windows ([§FS-rules.7.10](FS-rules.md#710-a-rule-declared-through-a-stub-warns-until-0190), [§FS-rules.7.11](FS-rules.md#711-a-citation-or-chapter-read-through-a-stub-warns-until-0190)) close in the same
release, so no window is left for a mixed site to fall into. Recommended findings remain opt-in
suggestions ([§FS-rules.7.5](FS-rules.md#75-prohibition-and-recommendation-reuse)), and explicit `warn` and `warn-not` strengths retain
their warning behavior ([§FS-config-v2.rules.strengths](FS-config-v2.md#rulesstrengths-one-strength-vocabulary-two-severities)).

The completed route is [§DF-mixed-path-citation-errors.decision](../decisions/functional/DF-mixed-path-citation-errors.md#decision-decision). This address
and heading remain for citations written during the window.

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
Bare `--rule` keeps its stdout and its exit: a run that passes no selector
prints the same findings and exits the same way, and gains only the stderr line
of [§FS-check.2.2.1.4](FS-check.md#2214-a-rules-obligation) when the sentence's subject names a kind with no local
declaration. A sentence that applies to nothing is said to, scoped or not,
because `success` there would read as the sentence's pass. A whose clause that
selects none of the kind's declarations is not that case and adds no line
([§FS-rules.2.2](FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)).

Three consequences of scoping follow from what `authority` is rather than from
the flag, and each is behavior a caller should be able to rely on.

A finding the trial sentence and a declared rule authored jointly — one finding,
one tail naming both origins
([§FS-rules.6](FS-rules.md#6-semantic-deduplication)) — **is** retained, with its
authority and its message bytes exactly as an unscoped run renders them. The
sentence did author it, and scoping is not the place to relitigate the merge.

A trial sentence duplicating a `[citations]` direction yields an **empty** scoped
report wherever the direction reports the same units. The config finding wins
byte-for-byte and the rule authored nothing for those units
([§FS-rules.6](FS-rules.md#6-semantic-deduplication)), so there is nothing for
`authority` to name, and the run prints `success` and exits `0`
([§FS-check.2.1.3](FS-check.md#213-the-success-line)) while the tree's own
findings still stand. This is known behavior, recorded here so it cannot change
by accident; making such a sentence visible is a separate question this point
does not answer.
A declaration a whose clause leaves outside the rule shows nothing in a scoped
report, whatever its field says: one whose value is outside the field's
`one_of` raises `invalid-field-value`, which no rule authored, so only the
unscoped run reports it
([§FS-rules.requirements.unseen](FS-rules.md#requirementsunseen-nothing-a-rule-governs-passes-unseen--realized-one-case-deferred)).
A unit the direction passes and the sentence fails, because its only citation
of the target kind does not resolve, is not that case: the sentence's finding
stands with `--rule` as its authority, and the scoped run reports it and exits
`1` ([§FS-rules.6](FS-rules.md#6-semantic-deduplication)).

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
chapter directly under its declaration ([§FS-rules.2.1](FS-rules.md#21-a-chapters-name-is-its-whole-path)), and `Each SWR whose
verification is Test` lists only the declarations its whose clause selects
([§FS-rules.2.2](FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)). Text prints the
canonical coordinate, two spaces, location, two spaces, and title. A chapter JSON row uses the list
object's existing fields in their existing order, adds `"section"` immediately
after `"id"`, and puts the declaration ID in `id` and exact component path in
`section`; a declaration row remains byte-for-byte the ordinary list row. In
both forms a chapter row's title is the chapter's display name
([§FS-rules.5.1.1](FS-rules.md#511-a-chapters-display-name-is-the-label-its-author-wrote)).
Invalid syntax, unknown vocabulary, disabled named sections, and ambiguous
exact literals are exit-2 invocation errors, and the first three are answered
with a selector to paste rather than a rule sentence ([§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector)). A valid selector with no matches
prints nothing and exits 0. `--selector` is a flag; the positional remains the
scan path.

`grund list --glob <pattern>` is a second unit source beside `--selector`
([§FS-coordinate-globs](FS-coordinate-globs.md#fs-coordinate-globs-a-coordinate-glob-selects-catalog-coordinates-by-pattern)): it composes with the same selectors, in the same modes, by
intersection, and prints the same rows, a section row in a chapter row's shape whether its
path is numbered, named or mixed. With `--selector` beside it, the two intersect. No rule
subject admits a glob, and no rule admission changes.

### 8.1 A refused selector is answered with a selector

Moved to [§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector).

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
([§FS-check.3.5.4](FS-check.md#354-no-part-of-the-block-is-exempt-from-the-comparison)).

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
binary-embedded copies of the setup skill carry a marked byte-identical copy of
that section in `references/rules.md`, the skill's `rules` section
([§FS-init.5.5](FS-init.md#55-the-core-and-its-sections)), and the two skill directories remain wholly byte-identical
to one another, file for file ([§FS-init.5.3](FS-init.md#53-one-directory-two-surfaces)).
The section includes every accepted family, every row of the [§FS-rule-refusals.sentences](FS-rule-refusals.md#sentences-strict-refusals) table
and the [§FS-rule-refusals.sentences.whitespace](FS-rule-refusals.md#sentenceswhitespace-presence-name-whitespace-refusal) refusal, each with its exact rewrite or, where it offers
none, the `known kinds:` line `check --rule` prints after the reason
([§FS-rule-refusals.sentences.rewrite.no-form](FS-rule-refusals.md#sentencesrewriteno-form-where-no-form-is-offered)), the finding each example produces, and the exact
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
3. Asset-sync tests compare guide section to the repository skill's
   `references/rules.md` and the whole repository skill directory to the
   embedded copy.
4. The managed block's existing re-render byte comparison checks generated
   guidance.
5. A quoted-message test runs the exact chapter subject over a declaration
   whose chapter is gone and asserts the section's quoted reason clause is
   part of the `invalid-rule` message the binary emits.

### 10.1. Example refusal inventory

The refusal inventory in `examples/rules/README.md` teaches the refusal actually
printed by `check --rule`, under the example's configured kind vocabulary. Its
path-subject entry retains `Each file in vendor/ must cite at least one FS.` and
shows the reason and `known kinds:` line, explaining that no accepted-form
rewrite is offered, as
[§FS-rule-refusals.sentences.rewrite.no-form](FS-rule-refusals.md#sentencesrewriteno-form-where-no-form-is-offered)
and [§FS-rule-refusals.sentences.rewrite.by-refusal](FS-rule-refusals.md#sentencesrewriteby-refusal-the-rewrite-each-refusal-makes)
define. The inventory's introduction acknowledges refusals without a rewrite
rather than promising an accepted rewrite for every entry. A regression reads
this maintained entry and compares it to a CLI invocation; a copied prose
fixture or a runtime-only assertion does not hold the example to this contract
([§FS-examples.4](FS-examples.md#4-maintenance-contract)).

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
runtime. A whose clause ([§FS-rules.2.2](FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)) is not the exception phrase: an
exception carves a place or a site out of a subject and excuses the members it
covers, while a whose clause selects positively, by a value the field check
already holds, and excuses none of the declarations it selects
([§DF-field-value-subjects](../decisions/functional/DF-field-value-subjects.md#df-field-value-subjects-a-rule-subject-may-select-on-a-scalar-fields-value-with-one-whose-clause)). Settings must reuse this selector parser and independently answer
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
cannot reach. The rule's subject is `The <NAME> chapter of each <KIND>`, with
or without a whose clause, the declaration is a local declaration of `<KIND>`
the subject selects ([§FS-rules.2.2](FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)), and it has no accepted
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
chapter, or narrow the rule's subject to the declarations that have one, which
a whose clause can do by a scalar field's value. A declaration the clause
leaves outside the rule is never reported here, and the message bytes do not
change with a clause. A
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
