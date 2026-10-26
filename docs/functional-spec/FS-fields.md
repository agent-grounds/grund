# FS-fields: a v2 kind states the shape of its declarations, and check holds every declaration to it

A version-2 kind can say what a declaration of it looks like: which named chapters it carries, in what order, what a scalar chapter may say, and whether anything else may appear. `grund check` then holds every declaration of the kind to that shape, on the channel each constraint's strength names ([§FS-config-v2.rules.strengths](FS-config-v2.md#rulesstrengths-one-strength-vocabulary-two-severities)). Today that shape lives in prose and in review, and the only checked part is a chapter count ([§FS-rules.3.1](FS-rules.md#31-chapter-presence)), which passes an FS whose Terms chapter comes second. Stating it once, on the kind, makes it overridable like every other default ([§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable)) and gives the checks, the schema view and the agent guidance one record to read ([§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work)). The decision and what it rejected are [§DF-kind-fields](../decisions/functional/DF-kind-fields.md#df-kind-fields-a-kind-states-its-fields-by-section-name-and-a-present-optional-field-is-still-checked). Nothing here changes a version-1 file ([§FS-fields.v1](FS-fields.md#v1-version-1-is-unchanged)).

```toml
grund_config_version = 2
project_name = "example"
[schema.sources]
languages = ["markdown"]

[schema.kinds.FS]
folders = ["docs/fs"]
[schema.kinds.FS.fields.terms]
title = "Terms"
presence = "must"
position = "first"

[schema.kinds.DF]
folders = ["docs/df"]
closed = "must"
[schema.kinds.DF.fields.status]
one_of = ["Proposed", "Accepted", "Superseded"]
presence = "must"
position = "first"
```

With that config, an FS whose `## terms: Terms` chapter comes after `## 1. Behaviour`, and a DF that writes `## status: Maybe` and a `## surprise: …` chapter, are reported as:

```text
docs/df/DF-session.md:3: error: DF-session.status is "Maybe"; kind DF admits Proposed, Accepted, Superseded; retitle the heading to one of them
docs/df/DF-session.md:7: error: DF-session.surprise is not a field of kind DF, which is closed; its fields: status; rename it to one of them, number it, or add it to the kind's fields
docs/fs/FS-login.md:7: error: FS-login.terms is section 2 of 2; kind FS places it first; move the section there
```

and the run exits `1`.

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, kind, body, section, field, coordinate, lead), [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding) and [§FS-config-v2.terms](FS-config-v2.md#terms-terms) (strength). Defines:

- **Direct section**: a section of a declaration whose path has one component, `FS-login.terms` or `FS-login.1`. The lead is not a section.
- **Display name**: what a section heading shows after its section path, `Accepted` in `## status: Accepted` ([§FS-rules.5.1.1](FS-rules.md#511-a-chapters-display-name-is-the-label-its-author-wrote)).
- **Placement**: where a field must stand among the direct sections: `first`, `last`, or directly after another field.
- **Chain**: fields linked by `after`, read from the field nothing names in `after` to the last one named.
- **Scalar field**: a field with `one_of`. Its display name is its value.
- **Closed kind**: a kind whose `closed` strength is not `may`. Its declarations admit only their declared fields and numbered sections as direct sections.
- **Expresses**: a field expresses a chapter-presence rule when the field's `title`, or its name where it has no title, equals the rule's chapter `NAME`, compared case-insensitively ([§FS-rules.6](FS-rules.md#6-semantic-deduplication)).

## requirements: What every declared shape holds to

Three requirements on a kind's shape as a whole, each carrying a mark of [§FS-config.requirements](FS-config.md#requirements-what-the-config-contract-holds-to). Three more are left out because they are stated elsewhere: *a field is found by its section name, so its coordinate never moves when its display name does* is [§REQ-spec-section-names](../requirements/REQ-spec-section-names.md#req-spec-section-names-a-section-name-says-what-the-thing-is-and-never-moves), which [§FS-fields.identity.name](FS-fields.md#identityname-a-field-is-the-direct-section-of-its-name) serves; *a v1 file reads and reports as it did* is [§REQ-backwards-compatibility.1](../requirements/REQ-backwards-compatibility.md#1-what-is-covered), which [§FS-fields.v1](FS-fields.md#v1-version-1-is-unchanged) keeps; and *a strength moves only the channel a finding reaches* is [§FS-config-v2.requirements.strengths](FS-config-v2.md#requirementsstrengths-each-constraint-takes-the-strengths-it-can-mean-from-one-vocabulary-and-refuses-the-rest--realized-directional-for-the-next-constraint).

### requirements.one-finding: One defect in a declaration's shape is one finding — realized

A shape check reports what is wrong once, at the site to edit, and nothing downstream of it. An absent field is skipped, so the next field of its chain takes its place and nothing is reported for the gap but the absence ([§FS-fields.order.absent](FS-fields.md#orderabsent-an-absent-member-is-skipped)); a misplaced field is judged where it stands, so the fields after it are not reported for following it ([§FS-fields.order.nearest](FS-fields.md#ordernearest-one-field-out-of-place-is-one-finding)); a field written twice is the existing `duplicate-section` error alone, every field check reading its first occurrence, and a field heading at the wrong depth keeps its depth finding and is still the field ([§FS-fields.identity.name](FS-fields.md#identityname-a-field-is-the-direct-section-of-its-name)); and where a field and a chapter-presence rule see one absence, the field's finding is the one reported, while a defect only one of them sees is still reported by that one ([§FS-fields.rules](FS-fields.md#rules-fields-beside-chapter-rules)). So no finding is the shadow of another. Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

### requirements.satisfiable: A shape the loader accepts can be met, and is held — realized

The loader refuses, at the key that makes it so, a shape that contradicts itself or that nothing could hold: two fields both `first` or both `last`, and a field after the last, after itself or in a cycle ([§FS-fields.order.refusals](FS-fields.md#orderrefusals-what-the-loader-refuses)); a `title` or `one_of` entry no heading can show ([§FS-fields.identity.match](FS-fields.md#identitymatch-titles-and-values-match-byte-for-byte)); a scalar with no values, or with a `title` or `content` beside its values ([§FS-fields.content.one-of](FS-fields.md#contentone-of-a-scalars-value-is-its-display-name)); and fields, or `closed`, on a kind whose declarations have no authored body to hold them ([§FS-fields.forms](FS-fields.md#forms-form-lowers-to-the-version-1-contracts)). Every shape it accepts is then held: `may` waives absence and nothing else, so a present optional field out of place, mistitled or outside its `one_of` is an error ([§FS-fields.strengths.may](FS-fields.md#strengthsmay-a-present-optional-field-is-still-held-to-its-shape)). A kind's configuration therefore never states a shape its declarations need not have. Serves [§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work), whose reader takes the shape as true of every declaration of the kind.

### requirements.record: The agent and a schema view read the shape the checks read — realized

A kind's shape is one record, and the checks, `grund schema` ([§FS-schema.4.1](FS-schema.md#41-fields-are-lines-inside-their-kinds-node)) and the managed block read it; `config show` prints it back only as configuration ([§FS-config.4.2](FS-config.md#42-grund-config-show-path), [§FS-fields.record](FS-fields.md#record-the-record)). The block renders each kind's shape in the words of the schema view's field lines, in the scoped block of the directory above the kind's places or in the root block under `rules = "root"` ([§FS-init.2.3.5.12.2](FS-init.md#235122-declaration-shapes)), while the view of every kind at once is `grund schema`. Each reads the record the checks read, its slots in header order with placement as data, rather than describing the shape again. Serves [§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work), for which a shape the agent is never told is enforced only after the fact.

## declare: Declaring the shape

### declare.field: A field is a table under its kind

`[schema.kinds.<KIND>.fields.<name>]` declares one field of `<KIND>`. `<name>` is a section name in the fixed named-component grammar `[a-z][a-z0-9-]*`; any other name is refused at its header, ``` `[schema.kinds.FS.fields.Terms]`: a field name is a section name matching `[a-z][a-z0-9-]*` ```. A field table follows its row's header: though TOML would accept a sub-table written first, a field header above `[schema.kinds.<KIND>]` is refused at its line, ``` `[schema.kinds.FS.fields.terms]` names no row: write [schema.kinds.FS] above it ```, so a kind's shape is read top-down from its row. The table admits:

| Key | Value | Default |
|---|---|---|
| `title` | the display name the chapter must show | none: any display name |
| `presence` | a strength: `must`, `warn`, `should` or `may` | `may` |
| `position` | `first` or `last` | none |
| `after` | the name of another field of the same kind | none |
| `content` | `prose` or `values` | `prose` |
| `one_of` | a non-empty list of display names | none |

Fields are listed, rendered and reported in the order their headers are written ([§FS-config-v2.reader.4](FS-config-v2.md#reader4-order-and-location-are-kept)). Header order is not placement: only `position` and `after` place a field. An unknown key is ``unknown key `<key>` in [schema.kinds.<KIND>.fields.<name>] (expected title, presence, position, after, content, or one_of)``, which names the keys above in their order ([§FS-config-v2.reader.1.keys](FS-config-v2.md#reader1keys-an-unknown-key-names-the-keys-its-table-admits)). A value outside its set is refused at its line in the reader's shape, ``unknown [schema.kinds.FS.fields.terms] presence `maybe` (expected must, warn, should, or may)``, and the same for `position` (`expected first or last`) and `content` (`expected prose or values`).

### declare.closed: `closed` is a strength on the row

`closed` on `[schema.kinds.<KIND>]` takes a strength and defaults to `may`, an open kind. Any other value is ``unknown [schema.kinds.<KIND>] closed `<value>` (expected must, warn, should, or may)``. A closed kind admits, as direct sections of its declarations, its declared fields and numbered sections, and nothing else. The strength is the channel of [§FS-fields.checks.undeclared-chapter](FS-fields.md#checksundeclared-chapter-undeclared-chapter).

### declare.form: `form` is a row key

`form` on `[schema.kinds.<KIND>]` is `prose`, `value` or `rule`, and defaults to `prose`. Any other value is ``unknown [schema.kinds.<KIND>] form `<value>` (expected prose, value, or rule)``. What `value` and `rule` mean is [§FS-fields.forms](FS-fields.md#forms-form-lowers-to-the-version-1-contracts).

## identity: Identity

### identity.name: A field is the direct section of its name

A field is the declaration's **direct** section whose section name equals the field's name: `status` in `## status: Accepted`. It is never matched by its display name, and never by a same-named section nested deeper, so `<§>DF-session.status` resolves whatever the status says and the coordinate never moves ([§REQ-spec-section-names](../requirements/REQ-spec-section-names.md#req-spec-section-names-a-section-name-says-what-the-thing-is-and-never-moves)). This is the identity [§FS-values.2.5](FS-values.md#25-chapter-declared-value-roots) already gives a value chapter. The heading is today's named section, in Markdown or in a doc-comment declaration body, so no new heading syntax is parsed; a heading with no section name, `## Status`, is an `unmarked-heading` finding as before and is never a field ([§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading)). A one-line stub pointing at a doc-comment declaration has no body and is held to no field.

A field written twice is the existing `duplicate-section` error ([§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path)), and every field check reads the first occurrence. A field heading at the wrong depth keeps its heading-depth finding ([§FS-declarations.checks.section-heading-level](FS-declarations.md#checkssection-heading-level-section-heading-level-mismatch)); the field reads the section path, as a value chapter does.

### identity.match: Titles and values match byte for byte

`title` and every `one_of` entry are compared with the display name byte for byte, with no case folding and no whitespace trimming beyond what the heading parser already removes. A configured `title` or `one_of` entry that is empty or has leading or trailing whitespace is refused at its line: `field FS.terms has title " Terms", which is empty or has surrounding whitespace`, and `field DF.status has one_of entry " Accepted", which is empty or has surrounding whitespace`.

## order: Order

### order.ends: `first` and `last`

`position = "first"` makes the field the declaration's first direct section, and `last` its last. Numbered and named direct sections both count; the lead does not.

### order.after: `after` means directly after

`after = "<p>"` makes the field the direct section immediately after field `<p>`. Fields linked by `after` form a chain. A chain whose first field is placed `first` occupies the opening direct sections in chain order. A chain whose first field has no `position` fixes only the relative order of its members: each stands directly after its predecessor, wherever the chain begins.

### order.absent: An absent member is skipped

A field that is absent from a declaration, whether optional or required and already reported missing, is skipped, and the next field of its chain takes its place. An AR whose kind places `placement` first and `terms` after it, and which has no `placement`, must open with `terms`, and gets one finding: `missing-field` for `placement`. Where the first field of a chain without a `position` is absent, the next field is unconstrained.

### order.nearest: One field out of place is one finding

Each field is judged only against its nearest present predecessor in its chain, or against the position its chain's head fixes when every predecessor is absent. A field that is out of place is one `misplaced-field` finding, and the fields after it in its chain are judged against where it actually stands, so the error does not cascade: a declaration that writes `1.`, `placement`, `terms` under the AR shape above gets one finding, for `placement`.

### order.refusals: What the loader refuses

Each of these is a located config error at the offending key, and the run exits `2` ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)):

| Written | Located at | Refused with |
|---|---|---|
| `position` and `after` on one field | the later of the two keys | `field AR.terms sets both position and after; a field takes one placement` |
| two fields `first`, or two `last` | the second field's `position` | `fields AR.placement and AR.scope are both first; only one field can be` |
| two fields `after` the same field | the second field's `after` | `fields AR.terms and AR.scope both come directly after AR.placement; chain one after the other` |
| `after` naming no field of the kind | `after` | `after = "plcement" names no field of AR; its fields: placement, terms, values` |
| `after` naming the field placed `last` | `after` | `AR.why is last; nothing can come after it` |
| `after` naming the field itself | `after` | `AR.a comes after itself` |
| a cycle of two fields | the later field's `after` | `AR.a and AR.b each come after the other; one of them needs position or no after` |
| a longer cycle, its fields in header order | the latest field's `after` | `AR.a, AR.b and AR.c come after one another in a cycle; one of them needs position or no after` |

The list of fields in the `after` refusal is in header order.

## content: Content

### content.prose: Prose is the default

A `content = "prose"` field, the default, holds any body. Its display name is checked against `title` when one is configured ([§FS-fields.checks.mistitled-field](FS-fields.md#checksmistitled-field-mistitled-field)).

### content.values: A values field is the value chapter

A field with `content = "values"` is the value chapter of [§FS-values.2.5](FS-values.md#25-chapter-declared-value-roots): the same identity, the same value roots, the same `invalid-value-declaration` findings at the same lines, and the same prerequisites ([§FS-config.3.4.13](FS-config.md#3413-value_chapter--the-chapter-whose-named-children-are-values)). It is v1's `value_chapter` and nothing else, plus whatever presence, placement and title the field adds. A kind has at most one such field, and a second is refused with v1's one-chapter refusal at its `content` key.

### content.one-of: A scalar's value is its display name

A field with `one_of` is a scalar: its display name is its value, and a value outside the list is [§FS-fields.checks.invalid-field-value](FS-fields.md#checksinvalid-field-value-invalid-field-value). The body below a scalar's heading is ordinary prose. Each of these is refused at its line, and a conflicting pair at the later of its two keys:

- `one_of` beside `title`: `field DF.status sets one_of and title; a scalar's title is its value`;
- `one_of` beside `content`: `field DF.status sets one_of and content; a scalar's content is its title`;
- an empty `one_of`: `field DF.status has an empty one_of; a scalar needs at least one value`;
- an entry listed twice: `field DF.status lists "Accepted" twice in one_of`.

## strengths: Strengths and channels

### strengths.table: Which strength each finding takes

The first four codes take the field's `presence`; `undeclared-chapter` takes the kind's `closed`.

| What is wrong | Code | Reported at | `must` | `warn` | `should` | `may` |
|---|---|---|---|---|---|---|
| a field is absent | `missing-field` | the declaration's title line | error | warning | suggestion | not reported |
| a field is out of place | `misplaced-field` | the field's heading | error | warning | suggestion | error |
| a field shows another title | `mistitled-field` | the field's heading | error | warning | suggestion | error |
| a scalar is not in `one_of` | `invalid-field-value` | the field's heading | error | warning | suggestion | error |
| a direct named section no field declares, in a closed kind | `undeclared-chapter` | the section's heading | error | warning | suggestion | open: not reported |
| a field is written twice | `duplicate-section` | as today | error at every strength | | | |
| a values chapter is filled wrongly | `invalid-value-declaration` | as today | error at every strength | | | |

An error exits `1`. A warning is a standing warning, and a run with only warnings exits `0`. A suggestion is reported only under `--suggestions` ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)). A finding keeps its code and message whatever its strength ([§FS-config-v2.rules.strengths](FS-config-v2.md#rulesstrengths-one-strength-vocabulary-two-severities)).

### strengths.may: A present optional field is still held to its shape

`may` waives absence and nothing else. A `may` field that is present is held to its placement, title and values, and a violation is an error. A `one_of` that nothing enforces would be a declaration that lies; an author who wants the shape soft writes `presence = "warn"`.

### strengths.surfaces: The codes are ordinary check codes

All five codes are in the check code catalog ([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)), are selectable with `--only` and `--ignore` ([§FS-check.1](FS-check.md#1-inputs)), sort with every other finding ([§FS-errors.4](FS-errors.md#4-determinism)), and reach the language server through the same report. In JSON they have the shape every finding has, with `sites` and `authority` both `null`, as a config-authored finding's are.

## forms: `form` lowers to the version-1 contracts

`form = "value"` means what v1's `values = true` means ([§FS-config.3.4.9](FS-config.md#349-values--first-class-value-declarations)), and `form = "rule"` what v1's `rules = true` means ([§FS-config.3.4.12](FS-config.md#3412-rules--rule-declaration-kinds)): the same declarations, the same findings, and the same prerequisites, refused with the same messages located at the `form` key. A `form = "rule"` kind with a `content = "values"` field is v1's `rules = true` with `value_chapter`. There is no second interpretation path: both readers lower into the one record, and the prerequisites are checked once, on it.

A kind whose declarations have no authored body to hold a field takes no fields and cannot be closed. Each is refused at the field's header or at `closed`:

- `form = "value"`: `CONST is form = "value": its children are numbered components, so it takes no fields` (`… so it cannot be closed` at `closed`);
- `citable = false`: `NOTES is not citable: it declares nothing, so it takes no fields` (`… so it cannot be closed`);
- `fetch`: `TICKET is fetched: its declarations are a snapshot, so it takes no fields` (`… so it cannot be closed`).

## rules: Fields beside chapter rules

A chapter-presence rule ([§FS-rules.3.1](FS-rules.md#31-chapter-presence)) stays valid, rendered, citable and evaluated in both versions, whether its subject is a kind or one exact declaration, and fields do not replace it. Where a field expresses a presence rule and both see the same absence in one declaration, the field's finding is the one reported ([§FS-rules.6](FS-rules.md#6-semantic-deduplication)). A defect only one of them sees is still reported by that one, so adding a field never weakens a rule.

## record: The record

A kind's shape is one record, its slots, and every consumer reads that record: the checks above, the schema view ([§FS-schema.4.1](FS-schema.md#41-fields-are-lines-inside-their-kinds-node)), and the agent guidance's `### Declaration shapes` ([§FS-init.2.3.5.12.2](FS-init.md#235122-declaration-shapes)). Each slot carries its name, title, presence, placement and content (prose, values, or the scalar's values), and the kind carries `closed`. Slots stay in header order, and placement is data on each slot rather than a reordering. A config that declares fields renders the shapes, and with them the v21/v22 block ([§FS-init.2.3.7.1](FS-init.md#2371-the-current-version)).

## v1: Version 1 is unchanged

The v1 reader produces exactly one field: `value_chapter = "<name>"` is a `may` field named `<name>` with `content = "values"` and nothing else, so every v1 finding and every byte of v1 output stays as it is ([§FS-config.5.2](FS-config.md#52-every-older-version-keeps-its-meaning)). `fields`, `closed`, `form` and `one_of` on a v1 `[[kinds]]` row stay unknown keys.

## checks: Checks

Each section below is one finding `grund check` reports about a declaration's shape, and its name is the finding code verbatim, as [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog) publishes it and [§REQ-spec-section-names.code](../requirements/REQ-spec-section-names.md#code-a-check-is-named-by-its-diagnostic-code) requires. Its channel is its strength's ([§FS-fields.strengths.table](FS-fields.md#strengthstable-which-strength-each-finding-takes)).

### checks.missing-field: Missing field

A declaration with no direct section of a field's name, reported at the declaration's title line: `FS-logout has no terms field; kind FS declares it; add its terms section`.

### checks.misplaced-field: Misplaced field

A present field that does not stand where its placement puts it ([§FS-fields.order](FS-fields.md#order-order)), reported at the field's heading: `FS-login.terms is section 2 of 2; kind FS places it first; move the section there`. The counts are of direct sections. The placement reads `first`, `last`, or `directly after <name>`, naming the nearest present predecessor; it reads `first` when every predecessor of a chain whose head is `first` is absent.

### checks.mistitled-field: Mistitled field

A present field with a `title` whose display name differs from it, reported at the field's heading: `FS-reset.terms is titled "Glossary"; kind FS titles it "Terms"; retitle the heading to match`.

### checks.invalid-field-value: Invalid field value

A present scalar field whose display name is not in its `one_of`, reported at the field's heading, the values listed in their configured order: `DF-session.status is "Maybe"; kind DF admits Proposed, Accepted, Superseded; retitle the heading to one of them`.

### checks.undeclared-chapter: Undeclared chapter

A direct named section of a declaration of a closed kind that no field of the kind declares, reported at the section's heading, the fields listed in header order: `DF-session.surprise is not a field of kind DF, which is closed; its fields: status; rename it to one of them, number it, or add it to the kind's fields`. A closed kind with no field reads `its fields: none`. Numbered sections are always admitted.
