# FS-rules: grounded declarations state and enforce chapter rules

A repository states chapter and citation constraints once, as rule declarations: `grund check` enforces them, and `grund init` renders the same sentences to agents before they write, which closes the checked-guidance loop of [§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work). The decision is [§DF-chapter-rules](../decisions/functional/DF-chapter-rules.md#df-chapter-rules-chapter-rules-are-grounded-controlled-english-declarations-over-producer-neutral-facts).

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
  nested, as against the declaration's whole body. The rule grammar's subject-unit word. A
  numbered or unmarked heading is no chapter; where a refusal says `numbered chapter`, it means
  a numbered section path written where a chapter is wanted.
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

The shared requirements bind rules without being restated here: a release moves a rule verdict only as [§REQ-backwards-compatibility.1](../requirements/REQ-backwards-compatibility.md#1-what-is-covered) and [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) allow, equal sentences over equal facts give equal findings ([§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)), and the editor reports a rule finding as `check` does ([§REQ-one-answer.text.parity](../requirements/REQ-one-answer.md#textparity-the-editor-answers-as-the-command-line-does)).

### requirements.unseen: Nothing a rule governs passes unseen

A rule that cannot run is reported at its own heading, never silently omitted: the run keeps its other findings and exits nonzero, `init` writes no block over an invalid rule ([§FS-rules.4](FS-rules.md#4-validation-lifecycle)), a sentence the run's scope cannot verify is reported all the same ([§FS-rules.4.1](FS-rules.md#41-a-rule-this-scope-cannot-verify)), and a run narrowed to the rule-produced codes keeps every `invalid-rule` row ([§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits)). A `must` or `should` rule whose subject names a kind with no local declaration says it applies to nothing, scoped or narrowed ([§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing)), while a whose clause that selects none of its kind's declarations says nothing, since that describes the project's state rather than a broken rule. A declaration a whose clause leaves outside the rule either holds what its field allows or is reported by the field check ([§FS-fields.strengths.table](FS-fields.md#strengthstable-which-strength-each-finding-takes), [§FS-rules.2.2](FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)), a finding `--only-rule` drops ([§FS-rules.8](FS-rules.md#8-command-surfaces)). A declaration a quantified chapter subject cannot reach is reported where it is declared ([§FS-rules.checks.unreached-declaration](FS-rules.md#checksunreached-declaration-unreached-declaration)), and an incomplete snapshot yields no closed-world absence or count and keeps exit 2 ([§FS-rules.4](FS-rules.md#4-validation-lifecycle)). A chapter-presence rule beside a chapter-scoped citation rule suppresses nothing, and since it counts display names where the citation rule selects handles ([§FS-rules.3.1](FS-rules.md#31-chapter-presence), [§FS-rules.2.1](FS-rules.md#21-a-chapters-name-is-its-whole-path)), the two report one absence together only where a chapter is titled by its handle. A unit only a rule fails beside a duplicate `[citations]` direction, because its only citation of the target kind does not resolve, is reported as the rule's ([§FS-rules.6](FS-rules.md#6-semantic-deduplication)). One case is deferred to its own change: a trial sentence that duplicates a `[citations]` direction authors nothing for a unit the direction reports ([§FS-rules.6](FS-rules.md#6-semantic-deduplication)), so `check --rule … --only-rule` prints `success` and exits `0` there ([§FS-check.2.1.3](FS-check.md#213-the-success-line)) while the tree's own findings stand. Serves [§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work), on the model [§REQ-no-missed-citation.1](../requirements/REQ-no-missed-citation.md#1-no-silent-skips) sets for a file the scan cannot read.

### requirements.addition: The language grows by addition

A family, selector or producer is added beside the ones that exist and leaves every accepted sentence meaning what it meant, so a refusal's rewrite stays accepted when it is pasted back ([§FS-rules.3](FS-rules.md#3-the-five-sentence-families), [§FS-rule-refusals.sentences](FS-rule-refusals.md#sentences-strict-refusals)). The change that adds a family answers the two questions [§FS-rules.5.2](FS-rules.md#52-family-clauses) answers for the five: whether it reports the declarations its subject leaves unreached ([§FS-rules.checks.unreached-declaration](FS-rules.md#checksunreached-declaration-unreached-declaration)), and whether its `NAME` reads a chapter's handle path ([§FS-rules.2.1](FS-rules.md#21-a-chapters-name-is-its-whole-path)) or the display name a presence rule counts ([§FS-rules.3.1](FS-rules.md#31-chapter-presence), [§FS-rules.5.1.1](FS-rules.md#511-a-chapters-display-name-is-the-label-its-author-wrote)). The five answer the second both ways, so there is no default to inherit. A code the family raises joins the rule-produced codes of [§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits) in the same change. This requirement is directional: it binds the families added from here on, not the five. Serves [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path).

### requirements.phase-one: What phase 1 leaves out is owed to its own change

Every absence [§FS-rules.12](FS-rules.md#12-deliberate-phase-1-absences) lists is deferred, each owed to its own change, and the decision behind the language defers them rather than reserving syntax for them ([§DF-chapter-rules.2](../decisions/functional/DF-chapter-rules.md#2-decision)). Until one lands, a sentence that needs it is refused rather than half-parsed ([§FS-rule-refusals.sentences](FS-rule-refusals.md#sentences-strict-refusals)), so its arrival changes no accepted sentence. Serves [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path).

## 1. Rule declarations and opt-in

A citable Markdown kind opts in with the optional Boolean `rules = true` on its `[[kinds]]` row ([§FS-config.3.4.12](FS-config.md#3412-rules--rule-declaration-kinds)), or with `form = "rule"` under a version-2 config ([§FS-fields.forms](FS-fields.md#forms-form-lowers-to-the-version-1-contracts)). Every declaration of such a kind is one rule: its ID is the rule's authority, its title is the executable sentence, and its body is a non-empty rationale that is ordinary Markdown in every respect. A repository with no rule kind declares no rule, so outside a `check --rule` run it gets no rule finding, its managed block has no `### Chapter rules` section ([§FS-init.2.3.5.10](FS-init.md#23510-chapter-rules)), and `config show` prints no `rules` key ([§FS-config.3.4.12](FS-config.md#3412-rules--rule-declaration-kinds)).

The title is a grammar island. The ID before the title delimiter is an ordinary declaration ID, but the title's tokens are never citations under either `[reference] strict` setting, and marker insertion, shorthand expansion and cross-reference wrapping never rewrite the sentence ([§FS-fmt.2.3](FS-fmt.md#23-what-is-never-rewritten)).

A rule's sentence and rationale are read from the declaration its ID pairs with ([§FS-declarations.stubs.pairing.1](FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned)), so a rule written as a stub in its kind's home is the rule its target declares, whether or not the scan reads the target, and its findings are those of the same rule declared inline. A target whose sentence is no rule is `invalid-rule` at the stub's line ([§FS-rules.7.1](FS-rules.md#71-invalid-rule), [§DF-stubs-by-home-and-at-the-target](../decisions/functional/DF-stubs-by-home-and-at-the-target.md#df-stubs-by-home-and-at-the-target-a-rule-declared-through-a-stub-applies-now-and-a-stub-is-read-by-its-home-and-shown-at-its-target-from-0190)).

## 2. Subject selectors

Rule subjects select local declaration or named-chapter units only, under each configured kind's effective ID grammar ([§FS-config.3.2](FS-config.md#32-id--id-grammar)) and the scanner's accepted body-local sections. A declaration unit is its owned body, and a chapter unit is the named heading and its subtree through the line before the next heading of the same or shallower depth; outside-body and unmarked headings are no units. The front-matter coordinates of a declaration whose kind declares a `front-matter` field are chapter units too, each its slice, selected by the whole path `front-matter` or `front-matter.<key>` ([§FS-fields.front-matter.surfaces](FS-fields.md#front-mattersurfaces-what-each-surface-does-with-them)).

| Authored subject | Selector and match |
|---|---|
| `Each KIND` | every local declaration of that configured citable kind |
| `ID` | the one local declaration with that full ID |
| `The NAME chapter of each KIND` | the chapter at that whole path of every local declaration of the kind that has one |
| `ID.NAME[.NAME…]` | the one local named chapter with that exact coordinate |
| `Each KIND whose FIELD is VALUE`, `The NAME chapter of each KIND whose FIELD is VALUE` | what the subject without the clause selects, narrowed to the declarations of the kind whose scalar field `FIELD` holds `VALUE` ([§FS-rules.2.2](FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)) |
| `KIND.NAME[.NAME…]` | selector only: means `The NAME chapter of each KIND`, and a rule sentence refuses it |

`KIND.NAME` selects by configured kind and `list --glob 'KIND-*.NAME'` by rendered ID text ([§FS-coordinate-globs](FS-coordinate-globs.md#fs-coordinate-globs-a-coordinate-glob-selects-catalog-coordinates-by-pattern)), so wherever the kind's IDs render as `KIND-…` the two list the same rows, except an off-grammar spelling such as `FS-old`, which only the glob reaches. Under another `[id] format` the glob spells the IDs as that format renders them, such as `KIND_*.NAME` for `{kind}_{slug}`, or `KIND*.NAME` where a placeholder follows `{kind}` directly.

In a chapter subject, `NAME` selects the section handle path ([§FS-rules.2.1](FS-rules.md#21-a-chapters-name-is-its-whole-path)), never the display name, which only presence rules compare ([§FS-rules.3.1](FS-rules.md#31-chapter-presence)): `The goal chapter of each BENCH` selects `## goal: Goal and hypothesis`.

A token exactly equal to a configured citable kind name is the quantified kind selector, which only after `Each ` or `of each ` may take a whose clause ([§FS-rules.2.2](FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)). Any longer token must parse as a full ID under that kind's effective grammar, and named components require `[id] named_sections = true` and obey the configured named-section grammar. An exact literal subject must resolve to one unit; zero matches or ambiguity is `invalid-rule` ([§FS-rules.7.1](FS-rules.md#71-invalid-rule)).

A quantified chapter subject selects only the chapters that exist, and a declaration of the kind that has none is reported rather than passed over, so a chapter-scoped citation rule reaches every declaration of its kind. That finding is [§FS-rules.checks.unreached-declaration](FS-rules.md#checksunreached-declaration-unreached-declaration). Such a declaration contributes no unit, which is not being outside the rule: only a whose clause puts a declaration outside it ([§FS-rules.2.2](FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value), [§DF-chapter-rule-reaches-every-declaration.2.1](../decisions/functional/DF-chapter-rule-reaches-every-declaration.md#21-contributing-no-unit-is-not-being-outside-the-rule)). An exact chapter literal over the same absent chapter is an `invalid-rule` at the rule heading instead, and where both spellings stand, both fire ([§DF-chapter-rule-reaches-every-declaration.2.6](../decisions/functional/DF-chapter-rule-reaches-every-declaration.md#26-the-two-spellings-stay-two-kinds-of-finding)).

Subject-side aliases, including `*/`, component wildcards, numbered chapter literals, `Each chapter of each KIND`, and file, directory or path subjects are refused ([§FS-rule-refusals.sentences](FS-rule-refusals.md#sentences-strict-refusals)).

Object kinds use the target-entry grammar of [§FS-config.3.9.3](FS-config.md#393-alias-matching), including a pinned `alias/KIND` and `*/KIND`. A target pinned at the rule's own member by its whole alias path, nested or not (`group/api/AR` in member `group/api`), matches what the bare kind matches, with the bare kind's findings ([§DF-nested-alias-rule-targets](../decisions/functional/DF-nested-alias-rule-targets.md#df-nested-alias-rule-targets-a-rule-target-pinned-at-its-own-nested-alias-matches-like-the-bare-kind)). Kinds joined with `or` are one target set, normalized to one byte-sorted set whatever their order or repetition.

### 2.1 A chapter's NAME is its whole path

A chapter subject's `NAME` is the whole section path from the declaration to the chapter, one named component or several, and selects the chapter at exactly that path, never by a path's last component alone. Take `FS-login`, whose `## requirements` chapter holds `### requirements.terms`:

- `FS.requirements.terms` and `The requirements.terms chapter of each FS` select `FS-login.requirements.terms`.
- `FS.terms` and `The terms chapter of each FS` select a `## terms` chapter directly under `FS-login` where there is one, never the nested `requirements.terms`.
- `FS.requirements` selects the `requirements` chapter, whose unit includes `requirements.terms` but not as a unit of its own.

The separator stands only between a declaration and its first component, so with `section_separator = ":"` the selector is `FS:requirements.terms`. `NAME` means this on every surface that reads a subject: the rule families ([§FS-rules.3](FS-rules.md#3-the-five-sentence-families)), the declarations a rule leaves unreached, and every `list --selector` mode, `--size` included ([§FS-rules.8](FS-rules.md#8-command-surfaces)). A whose clause's `FIELD` is a whole path the same way: it names the direct section at exactly that path.

### 2.2 A whose clause selects on a scalar field's value

A quantified subject may end in one whose clause: `Each KIND whose FIELD is VALUE` or `The NAME chapter of each KIND whose FIELD is VALUE`. Every family that takes the subject takes it with the clause. An exact ID, the selector-only `KIND.NAME` shorthand and an object kind take none ([§DF-field-value-subjects](../decisions/functional/DF-field-value-subjects.md#df-field-value-subjects-a-rule-subject-may-select-on-a-scalar-fields-value-with-one-whose-clause)).

The clause is read from the subject the modality split leaves ([§FS-rules.3.6](FS-rules.md#36-where-a-sentences-subject-ends)). It opens at the ` whose ` after the kind, and its first ` is `, or an ` is` that ends the subject, divides it into `FIELD` before and `VALUE` after.

- **`FIELD`** is a whole section path ([§FS-rules.2.1](FS-rules.md#21-a-chapters-name-is-its-whole-path)) naming a scalar field of the kind: a version-2 field with `one_of` ([§FS-fields.content.one-of](FS-fields.md#contentone-of-a-scalars-value-is-its-display-name)). A field is a direct section ([§FS-fields.identity.name](FS-fields.md#identityname-a-field-is-the-direct-section-of-its-name)), so only a one-component path names one. A prose field, a values field and anything in a version-1 config ([§FS-fields.v1](FS-fields.md#v1-version-1-is-unchanged)) are not scalars, and the clause reads no front-matter key.
- **`VALUE`** must equal one `one_of` entry byte for byte ([§FS-fields.identity.match](FS-fields.md#identitymatch-titles-and-values-match-byte-for-byte)), so `is test` does not match `Test`. It holds one value: ` or ` inside it is part of that value.

The clause selects the declarations of `KIND` whose first direct section at `FIELD` has `VALUE` as its display name, the answer [§FS-fields.checks.invalid-field-value](FS-fields.md#checksinvalid-field-value-invalid-field-value) reads, and for the chapter form their chapter at `NAME`. A declaration whose field holds another value, or that has no section at `FIELD`, is outside the rule: it contributes no unit, no family reports it, and it is never unreached. A selected declaration that lacks the chapter is unreached as it is without the clause. A clause the grammar does not accept is refused for the part that failed, never as an unknown kind ([§FS-rule-refusals.sentences.whose](FS-rule-refusals.md#sentenceswhose-a-whose-clause-is-refused-for-the-part-that-failed)).

## 3. The five sentence families

The language is a closed controlled-English grammar. Fixed words and kind names are case-sensitive, every rule ends with exactly one terminal `.`, and a sentence contains exactly one semantic verb. `N` is a positive base-10 integer no larger than the platform's unsigned machine word (`18446744073709551615` on a 64-bit platform); a larger `N` is refused for its range ([§FS-rule-refusals.sentences.count-range](FS-rule-refusals.md#sentencescount-range-a-count-above-the-largest-supported-count-is-refused-for-its-range)). A count of one, written `one` or `1`, takes singular `chapter`, and every other count takes `chapters`. A lower bound and an exact count spell one as the word `one` and refuse the numeral, so `at least 1` and `exactly 1` are not sentences; an upper bound spells it `1`, except per-target coverage, whose ceiling of one is `at most once` ([§FS-rules.3.3](FS-rules.md#33-per-target-coverage)). These five families are the only ones accepted.

### 3.1 Chapter presence

```text
<subject> must|should have at least one <NAME> chapter.
<subject> must|should have at least N <NAME> chapters.
<subject> must|should have at most N <NAME> chapter|chapters.
<subject> must|should have exactly one <NAME> chapter.
<subject> must|should have exactly N <NAME> chapters.
```

The subject is a kind, a kind narrowed by a whose clause ([§FS-rules.2.2](FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)), or an exact declaration, never a chapter. The rule counts the subject's accepted direct chapters whose display name is `NAME`, case-insensitively and never by handle: `## goal: Goal and hypothesis` contributes zero to a presence rule for `goal`, and `## goal: Goal` one. A count outside the interval is `chapter-cardinality` ([§FS-rules.7.2](FS-rules.md#72-chapter-cardinality)). `NAME` is a non-empty single token with no whitespace anywhere, which restricts neither authored display titles nor chapter subjects.

Presence is how a repository states a chapter's count, `exactly one` or `at most N`, which no citation rule checks. Beside version-2 fields a presence rule stays valid, and an absence both see is reported once ([§FS-fields.rules](FS-fields.md#rules-fields-beside-chapter-rules), [§FS-rules.6](FS-rules.md#6-semantic-deduplication)).

### 3.2 Outbound citation count

```text
<subject> must|should cite at least one <KIND> [or <KIND> ...].
<subject> must|should cite at least N <KIND> [or <KIND> ...].
<subject> must|should cite at most N <KIND> [or <KIND> ...].
<subject> must|should cite exactly N <KIND> [or <KIND> ...].
```

Every resolved physical citation site inside the subject unit whose target is in the object set counts once. An ordinary `must cite at least one` failure at zero reuses `missing-citation`, and every other count failure is `citation-cardinality` ([§FS-rules.7.3](FS-rules.md#73-outbound-citation-cardinality)). The `distinct` spelling counts the declarations those sites name rather than the sites ([§FS-rules.3.2.1](FS-rules.md#321-counting-distinct-cited-declarations)).

#### 3.2.1 Counting distinct cited declarations

```text
<subject> must|should cite at least N distinct <KIND> [or <KIND> ...].
<subject> must|should cite at most N distinct <KIND> [or <KIND> ...].
<subject> must|should cite exactly N distinct <KIND> [or <KIND> ...].
```

`distinct` is written once, right after the count, and covers the whole `or` list. The count is the number of declarations that own the targets of the sites [§FS-rules.3.2](FS-rules.md#32-outbound-citation-count) counts, through the same site filter: a declaration owns itself, and a chapter is owned by the declaration whose body holds it ([§FS-rules.5.2](FS-rules.md#52-family-clauses)). So `§SRC-x`, `§SRC-x.2` and `§SRC-x.outcome` count once between them, and a subject in its own target set counts itself once. `exactly one distinct SRC` and `at most 1 distinct SRC` are sentences; `at least one distinct SRC` means what `at least one SRC` means and is refused with that form ([§FS-rule-refusals.sentences](FS-rule-refusals.md#sentences-strict-refusals)). Per-target coverage, a prohibition and an inbound count refuse `distinct`, and so does a `distinct` written before a later object kind. A failure is `citation-cardinality` with a row of its own ([§FS-rules.7.3](FS-rules.md#73-outbound-citation-cardinality)), and the counting mode is part of the deduplication key ([§FS-rules.6](FS-rules.md#6-semantic-deduplication)).

Whether `distinct` is the keyword is decided from the kind table the rule's object kinds are judged in, never from the scan, so a member run and the workspace root read one sentence alike ([§DF-distinct-target-counts](../decisions/functional/DF-distinct-target-counts.md#df-distinct-target-counts-an-outbound-count-may-count-distinct-cited-declarations-spelled-distinct-after-the-count)). Where `distinct` is not a citable kind of the rule's own project, it is the keyword whenever an object follows it after the count, so `at least 3 distinct SRCX` is refused as `unknown kind "SRCX"`. Where `distinct` is such a kind, `at least 3 distinct.` counts it, and `distinct` followed by more is the keyword only where what follows is accepted as object kinds, citable or unverifiable.

### 3.3 Per-target coverage

```text
<subject> must|should cite each <KIND> at least once.
<subject> must|should cite each <KIND> at least N times.
<subject> must|should cite each <KIND> at most once.
<subject> must|should cite each <KIND> at most N times.
<subject> must|should cite each <KIND> exactly once.
<subject> must|should cite each <KIND> exactly N times.
```

All three bounds spell one as `once`; `N times` is accepted only above one. `at most once` means no minimum and a maximum of one physical site per target, and its finding keeps `requires at most 1` ([§DF-per-target-once-ceiling](../decisions/functional/DF-per-target-once-ceiling.md#df-per-target-once-ceiling-spell-a-per-target-ceiling-of-one-as-at-most-once)). The rule counts the physical sites from the subject unit to each declaration the object kinds match, the subject itself included, and emits one `citation-cardinality` row per target whose count is off.

### 3.4 Inbound citation count and prohibition

```text
<subject> must|should be cited by at least one <KIND> [or <KIND> ...].
<subject> must|should be cited by at least N <KIND> [or <KIND> ...].
<subject> must|should be cited by at most N <KIND> [or <KIND> ...].
<subject> must|should be cited by exactly N <KIND> [or <KIND> ...].
```

Each resolved physical citation from a declaration of a source kind to the subject unit counts once, and a count off its interval is `uncited-unit` ([§FS-rules.7.4](FS-rules.md#74-inbound-citation-cardinality)). A citation to a numbered section in the subject's own body is a citation to the subject ([§FS-rules.5.1](FS-rules.md#51-facts-and-identity)), so `GOAL-x must be cited by at least one BENCH.` counts `GOAL-x.4`; a citation to a named chapter, `GOAL-x.outcome`, counts for that chapter rather than for `GOAL-x`.

Prohibition has one spelling and no count synonym:

```text
<subject> must not|should not cite any <KIND> [or <KIND> ...].
```

Every offending physical citation site produces `forbidden-citation` or `discouraged-citation` ([§FS-rules.7.5](FS-rules.md#75-prohibition-and-recommendation-reuse)).

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

A sentence's subject is the text before its modality, found in a fixed order rather than at the earliest position: the first ` must not `, else the first ` should not `, else the first ` must `, else the first ` should `. Where the split reaches a chapter's modal-word name first, as in `The must chapter of each FS must cite at least one FS.`, the subject left is `The`, and the refusal names the chapter ([§FS-rule-refusals.sentences.modal-chapter](FS-rule-refusals.md#sentencesmodal-chapter-a-chapter-named-after-a-modal-word-is-refused-for-that-name)). The split cuts a whose clause's `VALUE` as it cuts a chapter's `NAME`: spaces, a capitalised `Must fix`, ` and ` and ` or ` survive it, a lowercase modal word inside a value ends the subject there, and the refusal names the value ([§FS-rule-refusals.sentences.whose](FS-rule-refusals.md#sentenceswhose-a-whose-clause-is-refused-for-the-part-that-failed)). No value is matched against the configuration before the split, so a rule and a selector split alike.

## 4. Validation lifecycle

`config validate` validates only the optional `rules` key and its relationship to the kind row. Rule titles are parsed after the shared scan has built the catalog, then resolved before evaluation or managed-block rendering. A missing title, empty rationale, refused title, unresolved or ambiguous literal, or disabled named component is a located `invalid-rule` at the rule heading; the run still reports every other finding, and no surface silently omits it.

`check --rule` refuses grammar or vocabulary after config selection and before scanning, in the refused-input shape of [§FS-errors.2.5](FS-errors.md#25-refused-input), with the accepted form, or none, that [§FS-rule-refusals.sentences.rewrite](FS-rule-refusals.md#sentencesrewrite-an-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced) builds. A syntactically valid literal that does not resolve needs the catalog, so after scanning it is an `invalid-rule` attributed to `--rule`, with the ordinary finding exit 1.

`init` parses and resolves configured rules before writing. An `invalid-rule` is printed in its located form, the managed block stays byte for byte untouched, and the command exits nonzero: `init` never renders an invalid sentence and never silently skips one. A sentence that is only unverifiable from where the command ran is not invalid ([§FS-rules.4.1](FS-rules.md#41-a-rule-this-scope-cannot-verify)).

If any scan or fact producer is incomplete, every closed-world conclusion about absence or count is suppressed. Already-known positive site findings may remain, every ordinary scan finding is kept, and the run exits 2: no incomplete tree is presented as a complete rule verdict.

### 4.1 A rule this scope cannot verify

An object kind pinned at an alias (`workshop/OP`) or matched across every alias (`*/OP`) resolves where the run holds the workspace that alias belongs to, which is the workspace the run's own effective config declares, never one climbed to from above; `check` and `init` therefore read one vocabulary in one directory ([§DF-unverifiable-rule-scope.2.1](../decisions/functional/DF-unverifiable-rule-scope.md#21-the-vocabulary-follows-the-runs-own-workspace-and-only-its-own)). A config that declares no `[workspace]` holds no alias at all, which is the ordinary case for a member-scoped run, whose effective config is the member's own `grund.toml` ([§FS-workspace.2](FS-workspace.md#2-workspace-configuration), [§FS-workspace.5.1](FS-workspace.md#51-a-member-run)). There a sentence with a pinned or any-member object kind is **unverifiable here** rather than invalid: it is reported at the rule's own heading, keeps the code `invalid-rule` and the wording [§FS-errors.3.7](FS-errors.md#37-the-rule-site-unknown-alias-reasons) fixes, and the run still exits nonzero ([§DF-unverifiable-rule-scope.2.2](../decisions/functional/DF-unverifiable-rule-scope.md#22-unverifiable-is-a-verdict-the-parser-reaches-not-a-judgement)).

Unverifiability is a verdict about the object kind's alias and nothing else: the grammar, the rationale and a literal subject are judged as for a local object kind, and a rule that fails on any of them is invalid with [§FS-rules.4](FS-rules.md#4-validation-lifecycle)'s consequences in full. The verdict is mechanical:

| written object kind | the run's workspace vocabulary | verdict |
|---|---|---|
| `workshop/OP` | holds no alias at all | unverifiable here |
| `workshop/OP` | holds `workshop`, which declares `OP` | resolves |
| `workshop/NOPE` | holds `workshop` | invalid rule |
| `typo/OP` | holds aliases, none named `typo` | invalid rule |
| `*/OP`, `OP` not local | holds no alias at all | unverifiable here |
| `*/OP`, `OP` not local | holds aliases, none declaring `OP` | invalid rule |
| `*/OP`, `OP` declared locally | either | resolves |

A config that declares `[workspace]` holds at least its own project's alias, whatever its member list expands to, so an empty member list and an unreachable member give the same verdict.

`init` withholds the managed-block write for an invalid rule and for nothing else. A run whose every unresolved rule is unverifiable here writes the block, renders each unverifiable sentence as authored ([§FS-rules.9.1](FS-rules.md#91-one-tree-renders-one-block)), reports each one, and exits nonzero; one invalid rule beside them puts the whole run back under [§FS-rules.4](FS-rules.md#4-validation-lifecycle), with nothing written ([§DF-unverifiable-rule-scope.2.3](../decisions/functional/DF-unverifiable-rule-scope.md#23-the-unverifiable-bullet-is-rendered-as-authored-not-omitted)).

## 5. Relational meaning

The normative meaning is Datalog over one finite, complete snapshot; grund ships no Datalog surface or runtime ([§FS-rules.12](FS-rules.md#12-deliberate-phase-1-absences)).

### 5.1 Facts and identity

`RuleFacts` is one immutable, versioned snapshot whose relations are:

```text
decl(node, kind)
chapter(node, section_path, display_name)
contains(parent_node, child_node)
cites(site, immediate_from_node, target_node)
site_in(site, unit_node)
```

Every declaration and accepted chapter has its node facts whether or not it contains a citation, so citations never define the quantified universe, and every physical citation has a distinct site key, so multiplicity is kept. `cites` uses the immediately enclosing declaration or chapter as `from`; `site_in` relates the site to every rule unit that contains it.

A resolved citation whose section is not itself a rule unit, one with a numbered component such as `GOAL-x.4` or `GOAL-x.outcome.2`, is a `cites` fact to its nearest enclosing unit, the nearest named ancestor chapter or else the declaration, and a citation of a name under a number, such as `GOAL-g.1.goals`, counts toward its nearest enclosing unit the same way ([§DF-mixed-path-citation-errors](../decisions/functional/DF-mixed-path-citation-errors.md#df-mixed-path-citation-errors-mixed-citation-findings-use-ordinary-errors-in-0190)). The source side resolves the same way, so a site inside a numbered section `goal.3` is `from` the `goal` chapter. A citation `grund check` resolves therefore counts for every family exactly where it satisfies an ordinary `[citations.KIND]` obligation ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)), and only one that does not resolve, an unknown or ambiguous declaration or a section its declaration does not have, keeps its ordinary findings and contributes no `cites` fact ([§DF-section-citation-counts-in-rules](../decisions/functional/DF-section-citation-counts-in-rules.md#df-section-citation-counts-in-rules-a-resolved-citation-to-a-numbered-section-counts-in-chapter-rules)).

The citations and chapters a stub's target outside `[scan] include` lends the stubbed declaration ([§FS-declarations.stubs.target.5](FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged)) are facts as if the target were scanned, so the rule and `[citations]` findings they produce are those of the same tree with the target scanned, at the line that tree reports them ([§FS-declarations.stubs.pairing.4](FS-declarations.md#stubspairing4-every-surface-shows-a-paired-declaration-at-its-target), [§DF-stub-target-citations-and-chapters](../decisions/functional/DF-stub-target-citations-and-chapters.md#df-stub-target-citations-and-chapters-a-stubs-target-lends-the-stubbed-declaration-its-citations-and-chapters-and-what-that-newly-raises-warns-until-0190)).

#### 5.1.1 A chapter's display name is the label its author wrote

A `chapter` fact's `display_name` is the label the author wrote after the chapter's coordinate: `Terms` in `## terms: Terms`, and in `### requirements.terms: Terms`, which is `chapter(c, requirements.terms, Terms)`. The comment wrapper a heading sits behind is never part of it ([§FS-languages.comment-wrapper.headings](FS-languages.md#comment-wrapperheadings-the-wrapper-of-a-heading-line)), so `/// ## terms: Terms` in a Rust file and `## terms: Terms` in a Markdown file are the same fact, `chapter(c, terms, Terms)`, and a presence rule naming `Terms` counts either one ([§FS-rules.3.1](FS-rules.md#31-chapter-presence)). The title a chapter's `list --selector` row and `--toc` entry carry is [§FS-list.3.1](FS-list.md#31---format-text-default)'s and need not match this label.

### 5.2 Family clauses

For subject set `S`, target set `T`, physical site `P`, chapter name `N`, the subject's kind `K`, and the interval predicate `within`, the five families mean:

```text
chapter_count(s, N, n) :- s in S, n = count { c : chapter(c, _, N), contains(s, c) }.
outbound_count(s, T, n) :- s in S, n = count { p : cites(p, _, t), t in T, site_in(p, s) }.
per_target_count(s, t, n) :- s in S, t in T,
                             n = count { p : cites(p, _, t), site_in(p, s) }.
inbound_count(s, T, n) :- s in S,
                          n = count { p : cites(p, f, s), f in declarations(T) }.
forbidden_site(s, p) :- s in S, cites(p, _, t), t in T, site_in(p, s).
```

The `distinct` count ([§FS-rules.3.2.1](FS-rules.md#321-counting-distinct-cited-declarations)) counts the declarations owning the counted targets, over `decl` and the `owns` relation below, and reports `unreached(d, N)` as the outbound count does:

```text
owner_of(t, d) :- decl(t, _), d = t.
owner_of(t, d) :- owns(d, t).
distinct_count(s, T, n) :- s in S,
                           n = count { d : cites(p, _, t), t in T, site_in(p, s), owner_of(t, d) }.
```

Positive families report where `not within(n, cardinality)`, and prohibitions report every `forbidden_site`. Negation and aggregates are closed-world only when the snapshot is complete ([§FS-rules.4](FS-rules.md#4-validation-lifecycle)).

A whose clause `whose F is V` ([§FS-rules.2.2](FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)) narrows the declarations every clause ranges over to the selected ones, with no new relation:

```text
selected(d) :- decl(d, K), contains(d, f), chapter(f, F, V).
```

`contains(d, f)` is direct containment, so `selected` holds exactly where [§FS-fields.checks.invalid-field-value](FS-fields.md#checksinvalid-field-value-invalid-field-value) reads the value `V`. Under a clause `S` holds only the selected declarations, or their chapters at `N`, and every `decl(d, K)` premise below reads `selected(d)`, so a declaration outside the rule is in no `S` and no `unreached`.

A chapter subject `The N chapter of each K` carries a second premise, over the declarations of the kind rather than the chapters `S` selected:

```text
chapter_handle(c, N) :- chapter(c, N, _).
owns(d, c)           :- decl(d, _), contains(d, c), chapter(c, _, _).
owns(d, c)           :- owns(d, p), contains(p, c), chapter(c, _, _).
chapter_of(d, N)     :- chapter_handle(c, N), decl(d, K), owns(d, c).
unreached(d, N)      :- decl(d, K), not chapter_of(d, N).
```

`chapter_handle` is the subject selector of [§FS-rules.2](FS-rules.md#2-subject-selectors) as a relation: it joins `chapter`'s whole section path ([§FS-rules.2.1](FS-rules.md#21-a-chapters-name-is-its-whole-path)), never its last component nor the display name `chapter_count` counts, so `chapter_of(d, N)` holds exactly when `d` owns a chapter `S` selected, and contributing no unit and being unreached are one set. `owns` follows `contains` down through chapters, so a nested chapter belongs to the declaration whose body holds it, while `contains(d, c)` alone reaches only the direct chapters presence counts.

The outbound-count, per-target-coverage and inbound-count families report every `unreached(d, N)` beside every `not within(n, cardinality)` ([§FS-rules.checks.unreached-declaration](FS-rules.md#checksunreached-declaration-unreached-declaration)). Presence never has a chapter subject, and prohibition reports none, because a forbidden site inside a missing chapter does not exist ([§DF-chapter-rule-reaches-every-declaration.2.2](../decisions/functional/DF-chapter-rule-reaches-every-declaration.md#22-three-positive-families-and-the-reason-the-other-two-are-not-a-double-standard)).

### 5.3 Evaluation cost on independent records

Rule evaluation must keep checks usable as fast feedback ([§GOAL-fast-feedback.2](../goals.md#2-how-we-get-there)). For a fixed set of rules over independent declarations with bounded chapter depth, chapters and physical citation sites per declaration, and a fixed shared target vocabulary, evaluation must grow roughly with the records and their facts: on an optimized build, doubling such a corpus from 400 to 800 clean records must not both take more than three times as long and take more than two seconds at 800. The bound is for that fixed-shape workload, not for arbitrary subject/target products or a growing number of findings, and it is met over the complete snapshot: pruning records or rules, changing verdicts, or narrowing evaluation to an output selection does not satisfy it.

## 6. Semantic deduplication

Before evaluation, constraints deduplicate by `(subject selector, modality, relation, normalized target set, cardinality)`. The target set carries its counting mode, by site, per target or distinct, so `at least 3 SRC` and `at least 3 distinct SRC` are two constraints with two findings. A whose clause is part of the subject selector: two rules whose subjects carry the same clause are one constraint, and a rule with a clause and one without are two.

Rules-only duplicates produce one finding per failing unit or site, whose tail names every contributing rule ID in bytewise order, for example `(RULE-a, RULE-b)`, an ad-hoc `--rule` origin included. The same origins in the same order are the finding's `authority` ([§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits)).

Config-to-rule deduplication is decided on the findings, unit by unit, and only where a `[citations]` entry and a rule express the same local bare citing kind, a declaration-wide unit over every declaration of that kind, the same modality and level, the `cite` relation, the same normalized target entry, and the same cardinality. A subject with a whose clause is never kind-wide, so like an exact-ID subject it reports its own finding beside the direction's, and a direction counts sites, so it never absorbs a `distinct` rule. Where, for one unit or site, config reports `missing-citation`, `suggested-citation`, `forbidden-citation` or `discouraged-citation` and the rule reports the same code there on the same channel, config's finding wins byte for byte and the rule's is dropped: no rule ID is appended, `authority` stays empty, and JSON gains no `sites` list. A rule that loses this way authored nothing for that unit ([§FS-rules.8](FS-rules.md#8-command-surfaces)).

A direction counts a citation by the kind its token names whether or not it resolves ([§FS-config.3.9.1.1](FS-config.md#3911-obligations-and-prohibitions)), and a rule counts only a resolved one ([§FS-rules.5.1](FS-rules.md#51-facts-and-identity)). So a declaration whose only `REQ` citation dangles or names a missing section satisfies `[citations.FS] must = ["REQ"]` and fails `Each FS must cite at least one REQ.`, and a dangling citation into a workspace member does the same to a pinned direction and rule. That rule's finding is reported with the rule's `authority` beside the token's `unknown reference` or `missing section` ([§FS-check.3.1](FS-check.md#31-dangling-citation), [§FS-check.3.2](FS-check.md#32-missing-section)), on every channel the rule's level reaches, and a trial sentence scoped by `--only-rule` reports it and fails the run ([§DF-direction-absorbs-findings-not-rules](../decisions/functional/DF-direction-absorbs-findings-not-rules.md#df-direction-absorbs-findings-not-rules-a-citations-direction-absorbs-a-duplicate-rules-finding-for-a-unit-it-reports-not-the-rule)).

Field-to-rule pairing happens where conformance and rule findings are merged, after the chapter-rule pass and before `--only` and `--ignore`, so the language server sees the same findings ([§DF-kind-fields.2.5](../decisions/functional/DF-kind-fields.md#25-a-field-and-a-rule-are-paired-at-the-merge-not-deduplicated-before-evaluation)). A version-2 field expresses a chapter-presence rule when its `title`, or its name where it has no title, equals the rule's chapter `NAME` case-insensitively ([§FS-fields.rules](FS-fields.md#rules-fields-beside-chapter-rules)). Where, for one declaration, that rule counts zero matching chapters and the field reports `missing-field` or `mistitled-field` on the same channel, the field's finding stands with its `authority` `null` and the rule's `chapter-cardinality` is dropped. A defect only one of them sees, or a finding on another channel, as of a `must` rule beside a `warn` field, is still reported.

## 7. Findings and channels

Rule findings use the ordinary text and NDJSON shapes of [§FS-errors](FS-errors.md#fs-errors-grund-emits-messages-in-fixed-shapes). `must` and `must not` findings are errors and `should` and `should not` findings are suggestions, reported as [§FS-check.2.1](FS-check.md#21-report-format) and [§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in) say. A structural recommendation keeps its structural code on that channel.

### 7.1 Invalid rule

`invalid-rule` is located at the rule heading. A configured rule's parse failure is:

```text
<RULE-ID> is not a valid rule: <reason>; accepted form: <rewrite>
```

The rewrite is the sentence as authored with the failed production replaced ([§FS-rule-refusals.sentences.rewrite](FS-rule-refusals.md#sentencesrewrite-an-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced)), after the label [§FS-errors.2.5](FS-errors.md#25-refused-input) gives it. A parse failure that offers no form, every refusal [§FS-rule-refusals.sentences.rewrite.by-refusal](FS-rule-refusals.md#sentencesrewriteby-refusal-the-rewrite-each-refusal-makes) says offers none, ends at `<reason>`. A resolution failure's reason is `literal subject <selector> does not resolve`, and for a `--rule` sentence whose literal subject does not resolve, `<RULE-ID>` is `--rule` ([§FS-rules.4](FS-rules.md#4-validation-lifecycle)).

### 7.2 Chapter cardinality

`chapter-cardinality` is located at the subject declaration's title and covers zero and surplus chapters:

```text
<subject> has <actual> <name> chapters; <RULE-ID> requires <count>; expected display name "<name>" (case-insensitive, not section handle); observed direct chapters: <observed>; <fix>
```

`<observed>` lists every accepted direct chapter in source order as `"<coordinate>: <display name>"`, separated by `, `, so the handle shows beside the authored display name, as in `"goal: Goal and hypothesis"`; with none it is `none`. Nested descendants, and numbered and rejected headings, which are not chapters, enter neither the list nor the count. `<fix>` is the row's `count` case ([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)). When the count is too low and a direct heading of the subject that is not a chapter reads `NAME`, case aside, after any number, the `named` case follows for the first such heading, so `## 1. Terms` at line 5 gets `; write line 5 as ## terms: Terms`: its `#` run, `NAME` in lower case, `: ` and its title. It is left out where that lower-case `NAME` is not a section name or is already the handle of a direct chapter. An unmarked heading it names shows the same form in its own finding ([§FS-declarations.checks.unmarked-heading.2.2](FS-declarations.md#checksunmarked-heading22-a-direct-field-title-or-name-is-also-shown-the-field-form)).

### 7.3 Outbound citation cardinality

An ordinary hard `at least one` rule with zero matches reuses `missing-citation`:

```text
<subject> must cite <target-set> (<RULE-ID>); add such a citation within it
```

Other ordinary counts and every per-target `cite each` miss are `citation-cardinality`, located at the subject declaration's or chapter's title:

```text
<subject> cites <target-set> <actual> times; <RULE-ID> requires <count>; add or remove those citations until the count fits
<subject> cites <actual> distinct <target-set>; <RULE-ID> requires <count>; add or remove those citations until the count fits
```

For `cite each`, `<target-set>` is the one canonical target ID, one row per off-count target. A `distinct` count ([§FS-rules.3.2.1](FS-rules.md#321-counting-distinct-cited-declarations)) is the second row at every count, zero included, its noun the declarations it counted, and a rule of each spelling on one subject gives one row of each.

A row whose count is above zero, over or under and on every channel, keeps its whole message and appends one clause naming every citation the count read, `; counted at <path>:<line> (<citation>), …`:

- **Entries.** One per citation the count read, a citation a stub's target lends listed where it is written. A `distinct` row lists every site the plain count would read, so three citations of one source are three entries under `cites 1 distinct SRC`.
- **Entry form.** `<path>:<line>` is where the citation is written, its path named as the report names every path ([§FS-errors.3](FS-errors.md#3-message-text)), at the name a whole run gives a file with two names ([§FS-scan.identity.findings.content](FS-scan.md#identityfindingscontent-a-content-finding-is-one-line-naming-every-name)). `<citation>` is the citation as written without its `§`, so the item's own numbered section shows as `INPUT-001-diesel.1`.
- **Order.** By path, then line, then column, with no cap; two citations on one line are two entries. JSON `sites` holds each distinct `{ path, line }` once in the same order ([§FS-errors.5.1](FS-errors.md#51-on-stdout--the-commands-output)).
- **The title.** The subject's title is the row's location and not a site, unless a counted citation is written on that line.
- **Zero.** A count of zero, `missing-citation`'s included, names nothing: its row ends where its template ends, with `"sites":null`.

A price record held to one priced item, citing the item at line 7 and the item's section 1 at line 12:

```text
docs/prices/PRICE-001-diesel-retail.md:1: error: PRICE-001-diesel-retail cites INPUT or LABOR 2 times; RULE-001-price-item requires exactly one; add or remove those citations until the count fits; counted at docs/prices/PRICE-001-diesel-retail.md:7 (INPUT-001-diesel), docs/prices/PRICE-001-diesel-retail.md:12 (INPUT-001-diesel.1)
```

The clause follows the remedy segment and is not part of the remedy ([§FS-errors.5.5.1](FS-errors.md#551-how-a-row-is-read)). On a file with two names, the identity clause ` (also at <path>:<line>)` stays directly after the remedy and `; counted at …` follows it. A row is in scope at any of its sites, so a run narrowed to a path holding its subject or any counted citation reports it whole, anchored at the subject ([§FS-check.1.3.6.2](FS-check.md#1362-a-finding-that-spans-several-sites-is-in-scope-at-any-of-them)).

A chapter subject whose declaration has no such chapter reports `unreached-declaration` at the declaration instead ([§FS-rules.checks.unreached-declaration](FS-rules.md#checksunreached-declaration-unreached-declaration)), never a `cites <target-set> 0 times` row naming a unit that does not exist.

### 7.4 Inbound citation cardinality

`uncited-unit` is located at the subject declaration's or chapter's title and covers zero, surplus, and every other off-count inbound cardinality:

```text
<subject> is cited by <source-set> <actual> times; <RULE-ID> requires <count>; add or remove the citations of it until the count fits
```

A row whose count is above zero appends [§FS-rules.7.3](FS-rules.md#73-outbound-citation-cardinality)'s counted clause and carries its `sites`. Its entries are the citations of the subject from the rule's source kinds, wherever they are written. A chapter subject whose declaration has no such chapter reports `unreached-declaration` at the declaration instead, as in [§FS-rules.7.3](FS-rules.md#73-outbound-citation-cardinality).

### 7.5 Prohibition and recommendation reuse

A hard prohibition reuses the site-anchored `forbidden-citation` of [§FS-check.3.12](FS-check.md#312-forbidden-citation), with `(<RULE-ID>)` for `(citation direction)` and index-entry repairs governed by [§FS-check.3.12.2](FS-check.md#3122-repair-advice-preserves-a-required-index-entry). Negative recommendations reuse `discouraged-citation`, and ordinary positive ones `suggested-citation`.

### 7.6 Selection, JSON, ordering, and exits

`--only` and `--ignore` take codes only and never a rule identity ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)). The **rule-produced codes** are the nine a rule finding can carry: the four only rules produce (`chapter-cardinality`, `citation-cardinality`, `uncited-unit`, `unreached-declaration`); the four rules share with citation directions (`missing-citation`, `forbidden-citation`, `discouraged-citation`, `suggested-citation`; [§FS-rules.7.3](FS-rules.md#73-outbound-citation-cardinality), [§FS-rules.7.5](FS-rules.md#75-prohibition-and-recommendation-reuse)); and `empty-citation-obligation`, the line a `must` or `should` rule shares with the `[citations]` table when its subject's kind has no local declaration ([§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing)). Selecting any of them also selects every `invalid-rule` row, because a skipped rule is a check the run did not evaluate ([§DF-narrowed-check-keeps-invalid-rule](../decisions/functional/DF-narrowed-check-keeps-invalid-rule.md#df-narrowed-check-keeps-invalid-rule-a-check-narrowed-to-a-rules-code-keeps-the-error-that-says-the-rule-could-not-run)), and every `empty-citation-obligation` row, because such a rule judged nothing; so `--only empty-citation-obligation` keeps the `invalid-rule` row of a rule that could not say so. `--ignore` still removes either.

A `citation-cardinality` or `uncited-unit` row above zero carries its counted citations in JSON `sites` ([§FS-rules.7.3](FS-rules.md#73-outbound-citation-cardinality)); every other rule finding's `sites` is `null`, but for the spellings of a file with two names that every content finding carries ([§FS-scan.identity.findings.content](FS-scan.md#identityfindingscontent-a-content-finding-is-one-line-naming-every-name)). Every rule-derived message names its rule authority, and JSON carries the same origins as the record's `authority`, a bytewise-sorted list ([§FS-errors.5.1](FS-errors.md#51-on-stdout--the-commands-output), [§FS-output-shapes.1](FS-output-shapes.md#1-finding-object)). A chapter subject is rendered as its canonical qualified coordinate.

A rule-derived diagnostic carries the one rule that produced it: an `invalid-rule` names its rule, so the `invalid-rule` of a trial sentence whose literal subject does not resolve is attributed to `--rule` and kept by `--only-rule`, as its applies-to-nothing line is ([§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing)). A finding no rule authored, such as a citation direction's, a dangling or duplicate declaration's, or any other run-level warning, carries an empty `authority`.

Rule findings take the report order of [§FS-errors.4.1](FS-errors.md#41-ordering) and the exit mapping of [§FS-cli.5](FS-cli.md#5-exit-code-mapping-is-fixed). A `cite each` message places its target ID directly after the fixed `<subject> cites ` prefix, before the actual count, so same-anchor rows sort by target-ID bytes even when targets share a prefix.

## 8. Command surfaces

`grund check --rule "<sentence>" [<path>]` adds exactly one ad-hoc rule to the configured ones. It never disables a configured rule and deduplicates against an identical one, and its validation and exits are [§FS-rules.4](FS-rules.md#4-validation-lifecycle)'s. Without `--only-rule` the report is the one the tree gives with the sentence declared, its rule named `--rule` wherever a finding names one, so it includes [§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing)'s stderr line when the subject names a kind with no local declaration, and `--only-rule` keeps that line, since `success` there would read as the sentence's pass.

`grund check --only-rule` narrows the report to what the trial sentence authored: a finding is retained when its `authority` ([§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits)) contains the `--rule` origin. It is a `check`-only boolean ([§FS-cli.3](FS-cli.md#3-cross-subcommand-flags)). It requires `--rule`: given alone it is the invocation error `error: --only-rule requires --rule`, with empty stdout and exit `2`, decided where the other selector values are, before config discovery or scanning ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)), because a run scoped to no sentence would print `success` and exit `0` ([§DF-rule-authority-is-a-field.2](../decisions/functional/DF-rule-authority-is-a-field.md#2-decision)). It composes with `--only` and `--ignore` by intersection, and `--ignore` still wins. A `should` trial sentence's findings are suggestions, so `--only-rule` alone prints `success` for it, and `--suggestions` shows them ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)). A trial sentence that duplicates a `[citations]` direction is the deferred case of [§FS-rules.requirements.unseen](FS-rules.md#requirementsunseen-nothing-a-rule-governs-passes-unseen).

`grund list --selector "<selector>" [<path>]` filters the shared catalog to the declaration and chapter units the same subject selects in a rule, in every mode, and composes by intersection with the path, kind, project, unused, summary, size, top and format selectors where their output modes admit unit rows. Text prints the canonical coordinate, two spaces, location, two spaces, and title. A JSON row is [§FS-output-shapes.5](FS-output-shapes.md#5-list---formatjson)'s, a chapter row carrying `section`, and in both forms a chapter row's title is [§FS-list.3.1](FS-list.md#31---format-text-default)'s. Invalid syntax, unknown vocabulary, disabled named sections and ambiguous exact literals are exit-2 invocation errors, and the first three are answered with a selector to paste rather than a rule sentence ([§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector)). A valid selector with no matches prints nothing and exits 0.

`grund list --glob <pattern>` is a second unit source beside `--selector` ([§FS-coordinate-globs](FS-coordinate-globs.md#fs-coordinate-globs-a-coordinate-glob-selects-catalog-coordinates-by-pattern)): it composes with the same selectors, in the same modes, by intersection, intersects with `--selector` beside it, and prints a section row in a chapter row's shape whether its path is numbered, named or mixed. No rule subject admits a glob.

### 8.1 A refused selector is answered with a selector

Moved to [§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector).

## 9. Managed guidance and editor parity

With at least one rule kind, `grund init` renders the `### Chapter rules` section of [§FS-init.2.3.5.10](FS-init.md#23510-chapter-rules), with a bullet for every valid rule and every rule unverifiable here ([§FS-rules.4.1](FS-rules.md#41-a-rule-this-scope-cannot-verify)), and `grund check` compares it like the rest of the block ([§FS-check.3.5.4](FS-check.md#354-no-part-of-the-block-is-exempt-from-the-comparison)).

The editor shows every hard rule finding and `invalid-rule` with the message, code, title or citation range and severity `check` gives it, and shows no suggestion ([§REQ-one-answer.text.parity](../requirements/REQ-one-answer.md#textparity-the-editor-answers-as-the-command-line-does), [§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)).

### 9.1 One tree renders one block

The rendered section is a function of the authored sentences and of how the entrypoint renders their citations ([§FS-init.2.3.5.10](FS-init.md#23510-chapter-rules)), never of the scope a run was given, so an unverifiable rule's bullet is its sentence as its heading spells it, a member-scoped `init` writes the bytes a whole-workspace run would write for that file, and `check` from either scope compares against that render ([§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes), [§DF-unverifiable-rule-scope.2.3](../decisions/functional/DF-unverifiable-rule-scope.md#23-the-unverifiable-bullet-is-rendered-as-authored-not-omitted)). A path is a third such scope: `grund check <path>` renders from the rules its resolution scope reads, the project's ordinary scope with its rule-kind homes, never from its report scope ([§FS-check.1.3.6.1](FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution)), so a path holding no rule declaration still renders every bullet and the comparison's `agents-init` finding reaches the narrowed report by its code. A render from the report scope would have the narrowed and the whole-tree run ask for opposite bytes in one file.

## 10. Documentation and executable examples

The rules are taught by one guide at `docs/user-facing/rules.md` and one runnable golden example at `examples/rules/`, linked from the root README and `examples/README.md`, with no second skill. The guide teaches opt-in and rationales, every subject, family, count, modality, finding, channel and flag, ordering, both deduplication directions, the validation lifecycle, every phase-1 absence, what a quantified subject does not select, and the declaration a chapter-scoped citation rule reports rather than passes over.

Its marked `### Chapter rules` writing section is copied byte for byte into the setup skill's `references/rules.md` ([§FS-init.5.5](FS-init.md#55-the-core-and-its-sections)), in the repository and the embedded skill alike ([§FS-init.5.3](FS-init.md#53-one-directory-two-surfaces)). The section holds every accepted family; every row of the [§FS-rule-refusals.sentences](FS-rule-refusals.md#sentences-strict-refusals) table and the [§FS-rule-refusals.sentences.whitespace](FS-rule-refusals.md#sentenceswhitespace-presence-name-whitespace-refusal) refusal, each with its exact rewrite or, where none is offered, the `known kinds:` line `check --rule` prints after the reason ([§FS-rule-refusals.sentences.rewrite.no-form](FS-rule-refusals.md#sentencesrewriteno-form-where-no-form-is-offered)); the finding each example produces; and the exact `unreached-declaration` error a chapter-scoped citation rule produces about a declaration without the chapter, with the two actions that answer it ([§FS-rules.checks.unreached-declaration](FS-rules.md#checksunreached-declaration-unreached-declaration)). Its leading sentence on that reach is the specification's word for word up to wrapping ([§FS-rules.2](FS-rules.md#2-subject-selectors)). It also quotes the reason clause of the `invalid-rule` message an exact chapter subject produces once its chapter is gone, in the binary's own wording.

The example holds at least one passing and one violated instance of all five families, and its guide quotes every violated instance's exact finding. Its goldens run through the shared e2e runner ([§FS-examples.5](FS-examples.md#5-e2e-reuse-without-duplication)) and cover `invalid-rule`, `chapter-cardinality`, rule-derived `missing-citation`, ordinary and per-target `citation-cardinality`, `uncited-unit`, `forbidden-citation` with its rule tail, `suggested-citation`, `discouraged-citation`, and `unreached-declaration` for both a single rule and a two-rule group; both channels and suggestion-neutral exits; two same-anchor off-count targets with shared-prefix IDs; config-to-rule and rule-to-rule deduplication; and the refusal set. The refusal inventory in `examples/rules/README.md` teaches the refusal `check --rule` actually prints under the example's kinds: its path-subject entry keeps `Each file in vendor/ must cite at least one FS.` with its reason and `known kinds:` line and no rewrite ([§FS-rule-refusals.sentences.rewrite.by-refusal](FS-rule-refusals.md#sentencesrewriteby-refusal-the-rewrite-each-refusal-makes)), and its introduction says some refusals offer no rewrite rather than promising one for every entry; a regression compares that entry with a CLI run ([§FS-examples.4](FS-examples.md#4-maintenance-contract)).

## 12. Deliberate phase-1 absences

There is no `[settings]`, `config show --at`, path, folder or file subject, exception phrase, `grund:allow` comment, definition, derived term, component wildcard, wildcard subject alias, new command verb, suppression mechanism, SCIP/LSIF ingestion, symbol vocabulary, on-disk fact format, Datalog runtime, or kind-level existence rule. A whose clause ([§FS-rules.2.2](FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)) is not the exception phrase: an exception carves a place or a site out of a subject and excuses the members it covers, while a whose clause selects positively, by a value the field check already holds, and excuses none of the declarations it selects ([§DF-field-value-subjects](../decisions/functional/DF-field-value-subjects.md#df-field-value-subjects-a-rule-subject-may-select-on-a-scalar-fields-value-with-one-whose-clause)).

## checks: Checks

A check this specification raises is a section named by its diagnostic code ([§REQ-spec-section-names.code](../requirements/REQ-spec-section-names.md#code-a-check-is-named-by-its-diagnostic-code)), and is reported as [§FS-check.2.1](FS-check.md#21-report-format) says. `invalid-rule`, `chapter-cardinality`, `citation-cardinality` and `uncited-unit` keep their positional sections under [§FS-rules.7](FS-rules.md#7-findings-and-channels).

### checks.unreached-declaration: Unreached declaration

A declaration is unreached when a rule whose subject is `The <NAME> chapter of each <KIND>`, with or without a whose clause, selects it as a local declaration of `<KIND>` ([§FS-rules.2.2](FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)) and it has no accepted chapter at the section path `<NAME>` ([§FS-rules.2.1](FS-rules.md#21-a-chapters-name-is-its-whole-path)): it contributes no unit, so the rule's relation says nothing about it. It fires for the outbound-count, per-target-coverage and inbound-count families and no other ([§FS-rules.5.2](FS-rules.md#52-family-clauses)), once per semantic rule group per declaration, with the group's origins as its `authority` and tail ([§FS-rules.6](FS-rules.md#6-semantic-deduplication)), and is withheld from an incomplete snapshot ([§FS-rules.4](FS-rules.md#4-validation-lifecycle)). It is located at the declaration's title line, because the chapter title that would anchor it is what is missing:

```text
<declaration> has no <name> chapter, so <authority> cannot reach it; add the chapter, or narrow the rule to the declarations that have one; this became an error in grund 0.16.0
```

Its severity and fix are its [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog) row; from a `should` rule the message ends at `have one`. The two actions it names are the only answers, and a whose clause can do the narrowing by a scalar field's value without changing the message. A chapter-presence rule beside the citation rule raises its own `chapter-cardinality` ([§FS-rules.7.2](FS-rules.md#72-chapter-cardinality)) and suppresses nothing, so one absent chapter under both prints both lines ([§DF-chapter-rule-reaches-every-declaration](../decisions/functional/DF-chapter-rule-reaches-every-declaration.md#df-chapter-rule-reaches-every-declaration-a-chapter-scoped-citation-rule-reports-the-declaration-that-has-no-such-chapter)).
