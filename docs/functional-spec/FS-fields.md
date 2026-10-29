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
- **Front-matter field**: the field named `front-matter` on a kind that declares it. It is its declaration's front matter ([§FS-check.1.1.5.2](FS-check.md#1152-front-matter-holds-no-heading)) and never a chapter ([§FS-fields.front-matter](FS-fields.md#front-matter-front-matter-is-a-field)).
- **Key**: a top-level key of a front-matter field's block, which is a section of the field when its name is a section name. A **declared key** is one the field lists under `keys`.
- **Contribution**: in a TOML block, one place the document writes a key at its top level: `k = …`, a dotted `k.x = …`, or a header `[k]`, `[k.x]` or `[[k]]` ([§FS-fields.front-matter.keys](FS-fields.md#front-matterkeys-its-keys-are-its-sections)).

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

`front-matter` is the one field name with a meaning of its own. On a kind that declares it, the field is its declarations' front matter ([§FS-fields.front-matter](FS-fields.md#front-matter-front-matter-is-a-field)) and not a chapter, and it admits its own keys and its `keys` tables ([§FS-fields.front-matter.config](FS-fields.md#front-matterconfig-declaring-the-field-and-its-keys)). A kind that does not declare it reads `## front-matter:` as any other named section.

Fields are listed, rendered and reported in the order their headers are written ([§FS-config-v2.reader.4](FS-config-v2.md#reader4-order-and-location-are-kept)). Header order is not placement: only `position` and `after` place a field. An unknown key is ``unknown key `<key>` in [schema.kinds.<KIND>.fields.<name>] (expected title, presence, position, after, content, or one_of)``, which names the keys above in their order ([§FS-config-v2.reader.1.keys](FS-config-v2.md#reader1keys-an-unknown-key-names-the-keys-its-table-admits)). A value outside its set is refused at its line in the reader's shape, ``unknown [schema.kinds.FS.fields.terms] presence `maybe` (expected must, warn, should, or may)``, and the same for `position` (`expected first or last`) and `content` (`expected prose or values`).

### declare.closed: `closed` is a strength on the row

`closed` on `[schema.kinds.<KIND>]` takes a strength and defaults to `may`, an open kind. Any other value is ``unknown [schema.kinds.<KIND>] closed `<value>` (expected must, warn, should, or may)``. A closed kind admits, as direct sections of its declarations, its declared fields and numbered sections, and nothing else. The strength is the channel of [§FS-fields.checks.undeclared-chapter](FS-fields.md#checksundeclared-chapter-undeclared-chapter).

`closed` is also a key of the front-matter field, and there it governs that field's keys, never the kind's chapters ([§FS-fields.front-matter.config](FS-fields.md#front-matterconfig-declaring-the-field-and-its-keys)). The kind's `closed` judges direct sections, and the front-matter field is none, so the kind's `closed` never reaches a key.

### declare.form: `form` is a row key

`form` on `[schema.kinds.<KIND>]` is `prose`, `value` or `rule`, and defaults to `prose`. Any other value is ``unknown [schema.kinds.<KIND>] form `<value>` (expected prose, value, or rule)``. What `value` and `rule` mean is [§FS-fields.forms](FS-fields.md#forms-form-lowers-to-the-version-1-contracts).

## identity: Identity

### identity.name: A field is the direct section of its name

A field is the declaration's **direct** section whose section name equals the field's name: `status` in `## status: Accepted`. It is never matched by its display name, and never by a same-named section nested deeper, so `<§>DF-session.status` resolves whatever the status says and the coordinate never moves ([§REQ-spec-section-names](../requirements/REQ-spec-section-names.md#req-spec-section-names-a-section-name-says-what-the-thing-is-and-never-moves)). This is the identity [§FS-values.2.5](FS-values.md#25-chapter-declared-value-roots) already gives a value chapter. The heading is today's named section, in Markdown or in a doc-comment declaration body, so no new heading syntax is parsed; a heading with no section name, `## Status`, is an `unmarked-heading` finding as before and is never a field ([§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading)). A one-line stub pointing at a doc-comment declaration has no body and is held to no field. The one field that is not a direct section is `front-matter`, on a kind that declares it: that field is the block at the top of its declaration's file, and its sections are the block's top-level keys ([§FS-fields.front-matter.keys](FS-fields.md#front-matterkeys-its-keys-are-its-sections)).

A field written twice is the existing `duplicate-section` error ([§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path)), and every field check reads the first occurrence. A field heading at the wrong depth keeps its heading-depth finding ([§FS-declarations.checks.section-heading-level](FS-declarations.md#checkssection-heading-level-section-heading-level-mismatch)); the field reads the section path, as a value chapter does.

### identity.match: Titles and values match byte for byte

`title` and every `one_of` entry are compared with the display name byte for byte, with no case folding and no whitespace trimming beyond what the heading parser already removes. A configured `title` or `one_of` entry that is empty or has leading or trailing whitespace is refused at its line: `field FS.terms has title " Terms", which is empty or has surrounding whitespace`, and `field DF.status has one_of entry " Accepted", which is empty or has surrounding whitespace`.

A Markdown heading is one line and its display name holds no control character, so a configured `title` or `one_of` entry that holds one, a line break, a carriage return, a tab or any other, is refused at its line too, since no heading could ever match it ([§FS-fields.requirements.satisfiable](FS-fields.md#requirementssatisfiable-a-shape-the-loader-accepts-can-be-met-and-is-held--realized)). The message names the value on one line, each control character written as its escape, `\n`, `\r` and `\t`, and any other as `\u{<hex>}`: `field FS.notes has title "A\nB\tC", which holds a control character no heading can show`, and `field DF.status has one_of entry "Acc\nepted", which holds a control character no heading can show`. A value that is empty or padded is refused by the whitespace message above, whatever else it holds.

A front-matter key's `one_of` entries are compared the same way, byte for byte, with the key's value, which is its decoded scalar text and never a type YAML or TOML would resolve it to ([§FS-fields.front-matter.values](FS-fields.md#front-mattervalues-a-keys-value-is-its-decoded-text)). A front-matter key's `one_of` entry is not refused for a control character: its value is a decoded scalar, and a quoted one written on the key's line, `status: "a\nb"`, decodes to a line break, so such an entry can be met.

## order: Order

### order.ends: `first` and `last`

`position = "first"` makes the field the declaration's first direct section, and `last` its last. Numbered and named direct sections both count; the lead does not, and neither does the front matter, which stands above the body ([§FS-fields.front-matter](FS-fields.md#front-matter-front-matter-is-a-field)).

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
| `after` naming `front-matter` on a kind that declares it | `after` | `ADR.context comes after front-matter, which is the file's front matter and has no place among the chapters` |

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

The front-matter field's findings reuse these codes at sites of their own, and [§FS-fields.front-matter.findings](FS-fields.md#front-matterfindings-what-it-reports) says where each is reported and which strength it takes: the field's `presence` for an absent block, a key's `presence` for an absent key or a value outside its `one_of`, and the field's own `closed` for an undeclared key.

An error exits `1`. A warning is a standing warning, and a run with only warnings exits `0`. A suggestion is reported only under `--suggestions` ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)). A finding keeps its code and message whatever its strength ([§FS-config-v2.rules.strengths](FS-config-v2.md#rulesstrengths-one-strength-vocabulary-two-severities)).

### strengths.may: A present optional field is still held to its shape

`may` waives absence and nothing else. A `may` field that is present is held to its placement, title and values, and a violation is an error. A `one_of` that nothing enforces would be a declaration that lies; an author who wants the shape soft writes `presence = "warn"`.

### strengths.surfaces: The codes are ordinary check codes

All five codes, and the front-matter field's two codes `unreadable-front-matter` and `front-matter-heading`, are in the check code catalog ([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)), are selectable with `--only` and `--ignore` ([§FS-check.1](FS-check.md#1-inputs)), sort with every other finding ([§FS-errors.4](FS-errors.md#4-determinism)), and reach the language server through the same report. In JSON they have the shape every finding has, with `sites` and `authority` both `null`, as a config-authored finding's are.

## forms: `form` lowers to the version-1 contracts

`form = "value"` means what v1's `values = true` means ([§FS-config.3.4.9](FS-config.md#349-values--first-class-value-declarations)), and `form = "rule"` what v1's `rules = true` means ([§FS-config.3.4.12](FS-config.md#3412-rules--rule-declaration-kinds)): the same declarations, the same findings, and the same prerequisites, refused with the same messages located at the `form` key. A `form = "rule"` kind with a `content = "values"` field is v1's `rules = true` with `value_chapter`. There is no second interpretation path: both readers lower into the one record, and the prerequisites are checked once, on it.

A kind whose declarations have no authored body to hold a field takes no fields and cannot be closed. Each is refused at the field's header or at `closed`:

- `form = "value"`: `CONST is form = "value": its children are numbered components, so it takes no fields` (`… so it cannot be closed` at `closed`);
- `citable = false`: `NOTES is not citable: it declares nothing, so it takes no fields` (`… so it cannot be closed`);
- `fetch`: `TICKET is fetched: its declarations are a snapshot, so it takes no fields` (`… so it cannot be closed`).

## rules: Fields beside chapter rules

A chapter-presence rule ([§FS-rules.3.1](FS-rules.md#31-chapter-presence)) stays valid, rendered, citable and evaluated in both versions, whether its subject is a kind or one exact declaration, and fields do not replace it. Where a field expresses a presence rule and both see the same absence in one declaration, the field's finding is the one reported ([§FS-rules.6](FS-rules.md#6-semantic-deduplication)). A defect only one of them sees is still reported by that one, so adding a field never weakens a rule. A scalar field can also narrow a rule's subject: `Each SWR whose verification is Test` selects the declarations whose `verification` field holds `Test`, read by the same display name `invalid-field-value` checks ([§FS-rules.2.2](FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)).

## record: The record

A kind's shape is one record, its slots, and every consumer reads that record: the checks above, the schema view ([§FS-schema.4.1](FS-schema.md#41-fields-are-lines-inside-their-kinds-node)), and the agent guidance's `### Declaration shapes` ([§FS-init.2.3.5.12.2](FS-init.md#235122-declaration-shapes)). Each slot carries its name, title, presence, placement and content (prose, values, or the scalar's values), and the kind carries `closed`. The front-matter slot carries its presence, its own `closed`, and its keys in header order, each with its presence and its values ([§FS-fields.front-matter.record](FS-fields.md#front-matterrecord-the-record-and-what-renders-it)). Slots stay in header order, and placement is data on each slot rather than a reordering. A config that declares fields renders the shapes, and with them the v45/v46 block ([§FS-init.2.3.7.1](FS-init.md#2371-the-current-version)).

## front-matter: Front matter is a field

On a kind that declares `[schema.kinds.<KIND>.fields.front-matter]`, that field is not a chapter. It is the front matter of its declaration's file, the block [§FS-check.1.1.5.2](FS-check.md#1152-front-matter-holds-no-heading) delimits and [§FS-check.1.1.5.2.1](FS-check.md#11521-its-citations-belong-to-the-files-first-declaration) gives to the file's first declaration, and the block's top-level keys are the field's sections. A MADR record keeps its `status` there, an OKF concept its `type`, and a requirement its verification method. A format like these can state those facts once, on the kind, and `grund check`, `grund schema` and the agent guidance then hold them, rather than only a copy of each in a body chapter ([§GOAL-configurable.1](../goals.md#1-what-is-configurable), [§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work)). A kind that does not declare the field parses no block, and every byte it reports stays as it is. Decided in [§DF-front-matter-field](../decisions/functional/DF-front-matter-field.md#df-front-matter-field-a-kinds-front-matter-field-holds-its-files-yaml-front-matter-with-the-blocks-top-level-keys-as-its-sections), and for a `+++` block in [§DF-toml-front-matter](../decisions/functional/DF-toml-front-matter.md#df-toml-front-matter-a-kinds-front-matter-field-reads-a-toml-block-as-it-reads-a-yaml-one).

```toml
[schema.kinds.ADR]
files = ["docs/adr/0004-cache-availability.md"]
id_format = "{kind}-{number}-{slug}"
closed = "must"
[schema.kinds.ADR.fields.front-matter]
presence = "must"
closed = "must"
[schema.kinds.ADR.fields.front-matter.keys.status]
presence = "must"
one_of = ["proposed", "accepted", "rejected", "deprecated"]
[schema.kinds.ADR.fields.front-matter.keys.date]
presence = "must"
[schema.kinds.ADR.fields.front-matter.keys.decision-makers]
[schema.kinds.ADR.fields.context]
presence = "must"
[schema.kinds.ADR.fields.decision]
presence = "must"
```

With that config, a MADR 4.0 record whose front matter misspells its status and carries a key MADR does not have,

```markdown
---
status: acepted
date: 2026-02-03
decision-makers: [Bo Lind]
reviewer: Ana Ruiz
---

# ADR-0004-cache-availability: Cache room availability in Redis
```

is reported as

```text
docs/adr/0004-cache-availability.md:2: error: ADR-0004-cache-availability.front-matter.status is "acepted"; kind ADR admits proposed, accepted, rejected, deprecated; change the value to one of them
docs/adr/0004-cache-availability.md:5: error: ADR-0004-cache-availability.front-matter.reviewer is not a key of kind ADR's front matter, which is closed; its keys: status, date, decision-makers; rename it to one of them, remove it, or add it to the field's keys
```

and the run exits `1`.

### front-matter.config: Declaring the field and its keys

`[schema.kinds.<KIND>.fields.front-matter]` admits two keys:

| Key | Value | Default |
|---|---|---|
| `presence` | a strength: whether the declaration must have front matter | `may` |
| `closed` | a strength: whether a key the field does not declare is reported | `may`, an open field |

Any other key is refused at its line with the reader's unknown-key message, ``unknown key `position` in [schema.kinds.ADR.fields.front-matter] (expected presence or closed)`` ([§FS-config-v2.reader.1.keys](FS-config-v2.md#reader1keys-an-unknown-key-names-the-keys-its-table-admits)). `title`, `position`, `after`, `content` and `one_of` are refused this way, because the block has no heading to title, no place among the chapters, and no display name. A value outside its set is refused as on any field, ``unknown [schema.kinds.ADR.fields.front-matter] closed `yes` (expected must, warn, should, or may)``. `closed` stays an unknown key on every other field.

`[schema.kinds.<KIND>.fields.front-matter.keys.<key>]` declares one key, a top-level key of the block. `<key>` is a section name in the field-name grammar `[a-z][a-z0-9-]*`. Any other name is refused at its header in the field's words: ``` `[schema.kinds.ADR.fields.front-matter.keys.Status]`: a key name is a section name matching `[a-z][a-z0-9-]*` ```. A key table follows its field's header, as a field table follows its row ([§FS-fields.declare.field](FS-fields.md#declarefield-a-field-is-a-table-under-its-kind)). Written above `[schema.kinds.<KIND>.fields.front-matter]`, it is refused at its line: ``` `[schema.kinds.ADR.fields.front-matter.keys.status]` names no field: write [schema.kinds.ADR.fields.front-matter] above it ```. The table admits two keys:

| Key | Value | Default |
|---|---|---|
| `presence` | a strength: whether the block must have the key | `may` |
| `one_of` | a non-empty list of values | none: any value |

Any other key is ``unknown key `title` in [schema.kinds.ADR.fields.front-matter.keys.status] (expected presence or one_of)``. `one_of` keeps the refusals of [§FS-fields.content.one-of](FS-fields.md#contentone-of-a-scalars-value-is-its-display-name) and [§FS-fields.identity.match](FS-fields.md#identitymatch-titles-and-values-match-byte-for-byte), each naming the key: `key ADR.front-matter.status has an empty one_of; a scalar needs at least one value`, `key ADR.front-matter.status lists "accepted" twice in one_of`, and `key ADR.front-matter.status has one_of entry " accepted", which is empty or has surrounding whitespace`. Keys are listed, rendered and reported in the order their headers are written.

`[schema.kinds.<KIND>.fields.front-matter.keys]` holds only tables. Written as a header of its own, with a key in it, it is ``unknown key `status` in [schema.kinds.ADR.fields.front-matter.keys] (it holds only tables)``, as `[schema.kinds.<KIND>.fields]` is ([§FS-config-v2.reader.2](FS-config-v2.md#reader2-every-row-and-field-is-written-under-its-own-header)). A `keys` table under any other field stays an unknown config section, ``unknown config section `schema.kinds.ADR.fields.context.keys.status` ``. Another field's `after = "front-matter"` is refused ([§FS-fields.order.refusals](FS-fields.md#orderrefusals-what-the-loader-refuses)). The field is refused on a value, non-citable or fetched kind in the words of [§FS-fields.forms](FS-fields.md#forms-form-lowers-to-the-version-1-contracts). `config show` prints the tables back as written, and version 1 has no fields to read them into ([§FS-fields.v1](FS-fields.md#v1-version-1-is-unchanged)).

### front-matter.reading: Reading the block

A block is parsed only when its owner's kind declares the field, and its owner is the file's first declaration ([§FS-check.1.1.5.2.1](FS-check.md#11521-its-citations-belong-to-the-files-first-declaration)). Every other block parses as nothing and reads exactly as [§FS-check.1.1.5.2](FS-check.md#1152-front-matter-holds-no-heading) says. That covers a block whose first declaration is of a kind that does not declare the field, and a block in a file that declares nothing.

The delimiter names the language. A `---` block's content is YAML, read by the one YAML span tree, `scanner/yaml.rs` ([§DA-yaml-value-reader](../decisions/architectural/DA-yaml-value-reader.md#da-yaml-value-reader-one-yaml-event-parser-builds-one-span-tree-and-value-rules-are-a-pass-over-it)). A `+++` block's content is TOML 1.1, read by the one TOML span tree, `scanner/toml.rs` ([§DA-toml-front-matter-reader](../decisions/architectural/DA-toml-front-matter-reader.md#da-toml-front-matter-reader-one-toml-event-parser-builds-the-span-tree-a-toml-front-matter-block-is-read-through)). The content runs from the line after the opening delimiter to the line before the closing one, the closing `---` or `...` of a YAML block and the closing `+++` of a TOML one, so the delimiter lines are neither YAML nor TOML. Every line and byte either tree reports is counted in the file, so a finding names the file's line. A line ends at LF or CRLF, so a CRLF file reads as its LF form does: the same keys and values, at the same lines and columns, and no value or slice ends in a carriage return.

A YAML block's content is one YAML document whose top level is a mapping. A TOML block's content is one TOML document, whose top level is always a table. Content that is empty, or only comments and blank lines, is a present field with no keys, in either language. Each of these is `unreadable-front-matter` at the opening delimiter ([§FS-fields.checks.unreadable-front-matter](FS-fields.md#checksunreadable-front-matter-unreadable-front-matter)):

- YAML content the YAML parser rejects, a tab in indentation included, named by the parser's message and its line;
- TOML content the TOML parser or its decoder rejects, named by the first such message in source order and the line of the first byte it did not expect. A bare word as a value, `status = accepted`, is `string values must be quoted`. A string still open when the content ends is reported at the closing delimiter's line. A key TOML refuses to define again is not a parse error here: it is a key written twice ([§FS-fields.front-matter.keys](FS-fields.md#front-matterkeys-its-keys-are-its-sections));
- YAML content whose top level is a sequence or a scalar, `its top level is not a mapping`;
- YAML content that holds a second document, `a second document starts`. A line such as `--- # x` is not exactly a delimiter, so it does not close the block ([§FS-check.1.1.5.2](FS-check.md#1152-front-matter-holds-no-heading)), but in YAML it starts a document;
- collections nested more than 128 deep, `collections are nested more than 128 deep`, at the line of the collection that goes past. This is the span trees' one nesting bound, shared with value sources ([§FS-values.5.3](FS-values.md#53-incomplete-input-and-deterministic-output)), so no block makes either reader recurse past it ([§REQ-never-crashes.1](../requirements/REQ-never-crashes.md#1-every-failure-is-a-diagnostic)). Levels are counted the way the YAML tree counts them, the top level being the first. In TOML, each array and each inline table is one level more than the value that holds it, so `notes = [[…]]` is refused exactly where `notes: [[…]]` is. A table opened by a header or a dotted key is written flat and adds no level.

An unreadable block is still a present field, so `missing-field` is not also reported for it. None of its keys is read, so none is judged, and it gives no key a coordinate.

A file whose line 1 begins with a byte-order mark has no front matter ([§FS-check.1.1.5.2](FS-check.md#1152-front-matter-holds-no-heading)), however the rest of the line reads. Where the mark is followed by exactly `---` or `+++`, the author can see a block that grund does not, so the declaration's `missing-field` names the mark ([§FS-fields.front-matter.findings](FS-fields.md#front-matterfindings-what-it-reports)).

### front-matter.keys: Its keys are its sections

The field's keys are the top-level keys of the block's mapping, or of its table in TOML, in source order. A key nested inside a value is no key of the field; it is part of that value. A key is named by its decoded scalar, so `status`, `'status'` and `"status"` are one key. A key whose name is a section name, `[a-z][a-z0-9-]*`, is a section of the field, `<ID>.front-matter.<key>`, whether or not the field declares it. Any other key is still a key but has no coordinate, and a closed field reports it with the shorter remedy `rename it to one of them or remove it`, because it cannot be declared. That covers:

- a scalar key outside the grammar, such as `Reviewer`, `review_date` or `<<`, named in a finding by its decoded text;
- a key that is not a scalar, a collection written as a key (`? [a, b]`) or an alias (`*k`), named in a finding by the text it is written as on its line, `ADR-cache.front-matter.*k`.

A merge key `<<` is a plain key outside the grammar, and nothing is merged. The keys it would merge are not keys of the field, because no YAML resolution is applied ([§DA-yaml-value-reader.2.1](../decisions/architectural/DA-yaml-value-reader.md#21-one-event-parser)), so a `status` that only a merge supplies is absent.

In a TOML block, a top-level key is the first component of every key the document writes at its top level. `status = …` writes `status`, the dotted key `links.next = …` writes `links`, and the headers `[links]`, `[links.next]` and `[[authors]]` write `links`, `links` and `authors`. Each of these is a contribution to its key. A key is named by its decoded first component, so `status`, `"status"` and `'status'` are one key, and `"links.next" = …` writes the one key `links.next`, which is not a section name. A key's site is its first contribution, and the key keeps that place in source order. The components after the first are part of the key's value, as a nested YAML key is. A TOML key is always a string, so it is never named by its text the way a YAML collection key is.

A key that is a section written twice, compared decoded, is `duplicate-section` at its first occurrence, naming the others ([§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path)): `duplicate section ADR-cache.front-matter.status (also declared at docs/adr/cache.md:4); remove all but one of the keys`. Every check reads the first occurrence, as for a field written twice ([§FS-fields.identity.name](FS-fields.md#identityname-a-field-is-the-direct-section-of-its-name)), and a read of the coordinate is refused as ambiguous ([§FS-show.2.2.2](FS-show.md#222-ambiguous-section)). A key with no coordinate that is written twice is not a duplicate: it names no citation target.

In a TOML block, a key is written twice by every later contribution that TOML refuses because of what the key already is. Each contribution is judged against what the contributions before it that TOML allowed have made the key, so a refused one changes nothing, and the finding names the line of each refused one and of no allowed one. What TOML refuses depends on what the key already is:

- a value, `k = …`, whatever the value: every later contribution, `k = …` again, `k.x = …`, `[k]`, `[k.x]` or `[[k]]`. So `k = 1` then `k.x = 2`, `k = {…}` then `[k.x]` or `k.y = …`, and `k = […]` then `[[k]]` are each the key written twice;
- a table its dotted keys define, `k.x = …`: `k = …`, `[k]` and `[[k]]`;
- a table its own header defines, `[k]`: `[k]` and `[[k]]`;
- a table only a header below it has opened, `[k.x]` or `[[k.x]]`: `[[k]]`;
- an array of tables, `[[k]]`: `[k]`.

Every other contribution is allowed, and the key stays one key written once: `k.y = …` or `[k.y]` to a table of dotted keys, `[k.y]` to a table its own header defines, `[k]` or `[k.y]` to a table only a header below it has opened, after which `[k]` makes it a table its own header defines, and `[[k]]` or `[k.x]` to an array of tables. TOML calls each refused contribution a parse error. Reading it as a key written twice keeps the findings those of the same key written twice between `---` lines, and the reading never admits, at the level grund judges, a page that TOML refuses. A conflict below the top level, such as `[k.x]` twice, is part of `k`'s value and is not judged, as a key repeated inside a nested YAML mapping is not.

### front-matter.values: A key's value is its decoded text

A key's value is its scalar as the parser decodes it, with quotes and escapes resolved and no YAML type ever applied. So `1.10`, `no` and `2026-02-03` stay the text they are written as, and `'accepted'`, `"accepted"` and `accepted` are all `accepted`. A tag or an anchor on the value is ignored, so `!!str accepted` and `&s accepted` are `accepted` too. The value is compared byte for byte with `one_of` ([§FS-fields.identity.match](FS-fields.md#identitymatch-titles-and-values-match-byte-for-byte)). A key with nothing after its colon, `status:`, is present and its value is `""`.

`one_of` takes a one-line value: a plain or quoted scalar that starts on the key's line and ends on it. A block scalar (`|`, `>`) is not one, because YAML keeps its trailing newline and the comparison would read `accepted\n`. Nor is a scalar folded onto later lines, a sequence, a mapping, or an alias. Each of these is `invalid-field-value`, `ADR-cache.front-matter.status is not a one-line value; kind ADR admits proposed, accepted; write one of them on the key's line`. A key without `one_of` holds any value, nested or not, and nothing in it is judged.

In a TOML block, the value of a key written `k = …` is what follows its `=`, and the value of a key opened by a header or reached by a dotted key is the table they make. A string, basic or literal, on one line or several, is its decoded text, with escapes resolved, so `"accepted"`, `'accepted'`, `"""accepted"""` and `"accepted"` are all `accepted`. Every other scalar is its raw spelling, as written in the file, and no TOML type is applied: `1_000`, `0x1F`, `1.10`, `true`, `2026-02-03` and `1979-05-27T07:32:00Z` stay those exact characters, so `1_000` is never `1000` and `0x1F` never `1F`. TOML has no empty value, so `status = ""` is the counterpart of YAML's `status:`: present, and its value `""`. `one_of` takes the same one-line value: a scalar that starts on the key's line and ends on it, which `"""accepted"""` written on one line is. A string that spans lines is not one, and nor is an array, an inline table, or the table a header or a dotted key makes. Each is the same `not a one-line value` finding, at the key's site.

### front-matter.constructs: What each YAML or TOML construct gets

How each construct of YAML reads in the front matter of a kind that declares the field, as [§FS-values.2.2.3](FS-values.md#223-how-a-yaml-source-is-read) says for a value source:

| Construct | In front matter |
|---|---|
| a comment | read by no key; a `§` citation in it is live ([§FS-check.1.1.5.2](FS-check.md#1152-front-matter-holds-no-heading)) |
| a quoted key | its decoded name, the same key as the plain one |
| a key outside the grammar, a merge key `<<` included | a key with no coordinate; a closed field reports it |
| a non-scalar key, an alias key | a key with no coordinate, named by its text; a closed field reports it |
| a key written twice | `duplicate-section` when it is a section, the first occurrence read |
| a nested key | part of its top-level key's value |
| a tag or an anchor on a value | ignored |
| an alias, a block scalar, a multi-line scalar, a sequence or a mapping as a value | held, and `not a one-line value` under `one_of` |
| an empty value | present, `""` |
| a second document | `unreadable-front-matter` |
| a top level that is not a mapping | `unreadable-front-matter` |
| collections nested more than 128 deep | `unreadable-front-matter` |
| a parse error, a tab in indentation | `unreadable-front-matter` |
| a byte-order mark before the opening delimiter | no front matter, and a `missing-field` that names the mark |
| a CRLF line ending | read as LF |

And how each construct of TOML reads in a `+++` block, the same answers wherever TOML has the construct:

| Construct | In front matter |
|---|---|
| a comment | read by no key; a `§` citation in it is live ([§FS-check.1.1.5.2](FS-check.md#1152-front-matter-holds-no-heading)) |
| a quoted key, basic or literal | its decoded name, the same key as the bare one |
| a key outside the grammar, a quoted key holding a dot included | a key with no coordinate; a closed field reports it |
| a dotted key, `k.x = …` | the key `k`, its site the first contribution |
| a header, `[k]` or `[k.x]` | the key `k`, its site the first contribution |
| an array of tables, `[[k]]`, written again | one key `k`, its site the first `[[k]]` |
| `k.x = …` then `k.y = …` or `[k.y]`; `[k.x]` then `[k.y]` or `[k]`; `[[k]]` then `[k.x]` | one key, written once |
| `k = …` twice, `[k]` twice, or `k = …` then `[k]` | `duplicate-section` when it is a section, the first contribution read |
| `k = 1` then `k.x = 2` | `duplicate-section` when it is a section, the first contribution read |
| `k.x = 1` then `[k]` | `duplicate-section` when it is a section, the first contribution read |
| `k = {…}` then `[k.x]` or `k.y = …` | `duplicate-section` when it is a section, the first contribution read |
| `k = […]` or `[k]` then `[[k]]`; `[[k]]` then `[k]` | `duplicate-section` when it is a section, the first contribution read |
| a conflict below the top level, `[k.x]` twice | part of `k`'s value, not judged |
| a basic or literal string, on one line or several | its decoded text, escapes resolved |
| a number, a boolean, a date or a time | its raw spelling, `1_000` and `0x1F` as written |
| `k = ""` | present, `""` |
| a string that spans lines, an array, an inline table, or the table a header or dotted key makes, as a value | held, and `not a one-line value` under `one_of` |
| arrays and inline tables nested past the one bound | `unreadable-front-matter` |
| a parse error, the decoder's included | `unreadable-front-matter`, at the first error's line |
| a line that is exactly `+++` inside a multi-line string | closes the block ([§FS-check.1.1.5.2](FS-check.md#1152-front-matter-holds-no-heading)): the string is cut, `unreadable-front-matter` at the closing delimiter's line, and the lines after it are Markdown |
| a byte-order mark before the opening delimiter | no front matter, and a `missing-field` that names the mark |
| a CRLF line ending | read as LF |

### front-matter.coordinates: Its coordinates

The first declaration of a file whose kind declares the field gains two kinds of coordinate:

- `<ID>.front-matter`, at the opening delimiter's line;
- `<ID>.front-matter.<key>`, at the line of each key that is a section.

Both are entries of the declaration's one section map, marked as lying in the front matter ([§FS-declarations.section.map](FS-declarations.md#sectionmap-the-section-map-is-the-declarations-body-span)), so every surface that reads the map answers for them alike ([§REQ-one-answer.sections.map](../requirements/REQ-one-answer.md#sectionsmap-every-surface-reads-one-section-map)). Neither is a heading, and neither is inside the body. The body still starts at the declaration's heading ([§FS-declarations.body](FS-declarations.md#body-where-a-declarations-body-ends)), so no lead, fingerprint or section lock moves. They are never enrolled in a section lock, because their paths are named and have no heading ([§FS-section-lock.holds.entry](FS-section-lock.md#holdsentry-one-line-per-locked-address)). A declaration with no front matter has neither. One with an unreadable block has `<ID>.front-matter` and no key coordinate.

A coordinate's **slice** is its text in the file:

- for `<ID>.front-matter`, from the first byte of the opening delimiter to the last byte of the closing one, its line ending excluded;
- for `<ID>.front-matter.<key>`, from the key's first byte to the last byte of its value, as [§FS-values.2.2.2](FS-values.md#222-reader-fidelity) cuts a member. Comments between the key and the end of its value are inside, and comments before the key or after its value are not. A key with no value ends at its colon, and no slice ends in a carriage return.

In a TOML block, a key's coordinate is at the line of its first contribution, and its slice starts at that contribution's first byte: the key's own first byte, its opening quote where it is quoted, and the `[` or `[[` of a header, never the key's name inside it. The slice ends at the last byte of the last contribution in the key's **first run**, the contributions to the key that follow one another with no other top-level key's contribution between them. A `k = …` or `k.x = …` contribution ends at the last byte of its value. A header's contribution ends at the last byte of the last value written under it before the next header, or at its own `]` or `]]` where it holds none. Comments and blank lines after that last byte are not inside, and no slice ends in a carriage return. A key contributed to again after another key's contribution, which TOML allows (`[links.extra]` after `[[authors]]` reopens `links`), gains no second slice: those later lines are in `<ID>.front-matter`, as a comment between keys is.

A key's **title** is its value where that is a one-line scalar on the key's line, and it has none otherwise. `<ID>.front-matter` has none.

### front-matter.surfaces: What each surface does with them

| Surface | What it does with `ADR-x.front-matter` and `ADR-x.front-matter.status` |
|---|---|
| `grund show` of one of them | prints its slice, as a value-source member's slice is printed, with nothing added ([§FS-output-shapes.4.3](FS-output-shapes.md#43-value-source-declarations)). The default read, `--brief`, `--toc` and `--full` all print it, since a slice has no lead of its own. In JSON, `body` is the slice, `path` and `line` are its start, and `anchor` is `null` ([§FS-show.3.1.3.1](FS-show.md#3131-the-heading-anchor)) |
| a read of the whole declaration, in any mode | unchanged: the block is above the body, so the read neither prints it nor lists its coordinates, and a whole-declaration `--toc` lists headings only |
| `show --batch --all` | generates one query for each, among the declaration's section paths in their byte order ([§FS-show.2.6.2](FS-show.md#262-exhaustive-generation)) |
| `list --glob` | lists them as catalog coordinates ([§FS-coordinate-globs.matching.catalog](FS-coordinate-globs.md#matchingcatalog-a-glob-selects-catalog-coordinates)). `ADR-*.front-matter.*` lists the keys, with a key's title in the title column |
| `list --size` | one row for each, where lead and full both measure the slice ([§FS-output-shapes.5.2](FS-output-shapes.md#52-list---size---formatjson)). The lead-size check measures the same rows, so an oversized slice is warned at its coordinate's line ([§FS-declarations.checks.oversized-lead](FS-declarations.md#checksoversized-lead-oversized-lead-opt-in)) |
| `grund complete ids --sections` | offers them as sections of their declaration ([§FS-completions.2.2](FS-completions.md#22-what-the-catalog-offers)) |
| `refs`, `cover --glob`, and the language server's resolution and navigation | resolve them and go to the coordinate's line. `fmt` links a Markdown citation of one to the file with no fragment, as it does for a source-file home |
| `cover --lines` | a line inside a key's slice is in `front-matter.<key>`; the delimiter lines, comments and blank lines between keys, and the lines of a key with no coordinate are in `front-matter` ([§FS-cover.6.2](FS-cover.md#62-ownership-rules)) |
| `enclosing_section` of a citation in the block | `front-matter.<key>`, or `front-matter` outside every key's slice ([§FS-check.1.1.5.2.1](FS-check.md#11521-its-citations-belong-to-the-files-first-declaration)) |
| rule subjects | `front-matter` and `front-matter.<key>` are chapter units selected by their whole path ([§FS-rules.2.1](FS-rules.md#21-a-chapters-name-is-its-whole-path)), so `The front-matter.sources chapter of each METRIC must cite at least one POL.` reads the citations in the `sources` key. A chapter-presence rule counts direct chapters by display name ([§FS-rules.3.1](FS-rules.md#31-chapter-presence)), so it never counts them |
| section lock | never enrolled |
| the snapshot's `Section` record | an entry of `sections` like any other: its `path`, `line` and `title` as above, `title` the empty string where it has none, `headingLevel` `0` because it has no heading, and `value` and `valueRoot` `null`. The record gains no field: `headingLevel` `0` is the front-matter mark, and every other entry keeps its bytes ([§FS-distribution.3.2.2.3](FS-distribution.md#3223-scan-snapshot)) |

### front-matter.findings: What it reports

| What is wrong | Code | Reported at | Strength from | Message |
|---|---|---|---|---|
| the declaration has no front matter | `missing-field` | the title line | the field's `presence` | `ADR-cache has no front-matter field; kind ADR declares it; add front matter at the top of its file` |
| …because it is not its Markdown file's first declaration | `missing-field` | the title line | the field's `presence` | `ADR-later has no front-matter field; kind ADR declares it; only a file's first declaration has front matter; move it to a file of its own` |
| …because it is a doc-comment declaration | `missing-field` | the title line | the field's `presence` | `ADR-code has no front-matter field; kind ADR declares it; only a Markdown file's first declaration has front matter; move it to a Markdown file of its own` |
| …because line 1 is a byte-order mark and a delimiter | `missing-field` | the title line | the field's `presence` | `ADR-cache has no front-matter field; kind ADR declares it; remove the byte-order mark before the --- on line 1`, the `+++` in its place where that is the delimiter the mark is followed by |
| a declared key is absent | `missing-field` | the opening delimiter | the key's `presence` | `ADR-cache.front-matter has no date key; kind ADR declares it; add the key to the front matter` |
| a value is not in `one_of` | `invalid-field-value` | the key's line | the key's `presence` | `ADR-cache.front-matter.status is "acepted"; kind ADR admits proposed, accepted; change the value to one of them` |
| a value under `one_of` is not one line | `invalid-field-value` | the key's line | the key's `presence` | `ADR-cache.front-matter.status is not a one-line value; kind ADR admits proposed, accepted; write one of them on the key's line` |
| a key the field does not declare, in a closed field | `undeclared-chapter` | the key's line | the field's `closed` | as [§FS-fields.checks.undeclared-chapter](FS-fields.md#checksundeclared-chapter-undeclared-chapter) |
| a key written twice, or in TOML contributed to again in a way TOML refuses | `duplicate-section` | its first occurrence | always an error | as [§FS-fields.front-matter.keys](FS-fields.md#front-matterkeys-its-keys-are-its-sections) |
| the block cannot be read | `unreadable-front-matter` | the opening delimiter | always an error | as [§FS-fields.checks.unreadable-front-matter](FS-fields.md#checksunreadable-front-matter-unreadable-front-matter) |
| a heading claims the field's coordinate | `front-matter-heading` | the heading | always an error | as [§FS-fields.checks.front-matter-heading](FS-fields.md#checksfront-matter-heading-front-matter-heading) |

Each strength reaches its channel as [§FS-fields.strengths.table](FS-fields.md#strengthstable-which-strength-each-finding-takes) says, and a `may` key whose value is present and outside its `one_of` is an error ([§FS-fields.strengths.may](FS-fields.md#strengthsmay-a-present-optional-field-is-still-held-to-its-shape)). One defect is one finding ([§FS-fields.requirements.one-finding](FS-fields.md#requirementsone-finding-one-defect-in-a-declarations-shape-is-one-finding--realized)). No key is judged when the block is absent or unreadable. A key written twice is judged once, at its first occurrence. A `## front-matter:` heading is `front-matter-heading` and nothing else. `misplaced-field` counts direct sections only, never the block ([§FS-fields.order.ends](FS-fields.md#orderends-first-and-last)).

### front-matter.record: The record, and what renders it

The front-matter slot carries its presence, its own `closed`, and its keys in header order, each key with its presence and its `one_of` values ([§FS-fields.record](FS-fields.md#record-the-record)). Every view prints its keys nested under its field line, in the view's own words for a field line:

- `grund schema` ([§FS-schema.4.1](FS-schema.md#41-fields-are-lines-inside-their-kinds-node)) prints the field line, `field front-matter`, with its presence and, where it is not `may`, `closed <strength>`, joined by ` · `. Each key follows as a line of its own, `key <name>`, two columns deeper than the field line, its facts in the facts column: its presence, then `one of …` where it has values;
- Mermaid and DOT ([§FS-schema.9](FS-schema.md#9-the-mermaid-source), [§FS-schema.10](FS-schema.md#10-the-dot-source)) print the label lines `field front-matter: must · closed must` and `key status: must · one of proposed, accepted`;
- the managed block's `### Declaration shapes` ([§FS-init.2.3.5.12.2](FS-init.md#235122-declaration-shapes)) prints `  - field front-matter must · closed must`, then `    - key status must · one of proposed, accepted`.

So the ticket's kind renders in `grund schema` as:

```text
  ADR   kind                     docs/adr/0004-cache-availability.md
          field front-matter     must · closed must
            key status           must · one of proposed, accepted, rejected, deprecated
            key date             must
            key decision-makers  may
          field context          must
          field decision         must
          closed must
  code  complement               every scanned file no other row claims
```

The managed block's version pair does not move for this ([§FS-init.2.3.7.1](FS-init.md#2371-the-current-version)). Every released grund refuses `closed` on a field and refuses a `keys` table, so every config that loads today renders the same bytes, `[fields.front-matter]` with only `presence` set included. The config JSON schema gives the field a definition of its own, which admits `presence`, `closed` and `keys`, and gives a key one that admits `presence` and `one_of`. `grund config explain` answers for both tables and both new codes ([§REQ-discoverable-offline](../requirements/REQ-discoverable-offline.md#req-discoverable-offline-the-installed-grund-answers-for-every-setting-and-every-check-code)).

### front-matter.placement: Which declaration has it

Only a Markdown file's first declaration has front matter. A later declaration of a kind that declares the field, in the same file, has none, and neither does a doc-comment declaration. Under `presence` other than `may`, each gets the `missing-field` that says why. The same holds when the file's first declaration is of a kind that does not declare the field: its block is not parsed, and a later declaration of an opted-in kind has no front matter. A stub is held to no field, as today ([§FS-fields.identity.name](FS-fields.md#identityname-a-field-is-the-direct-section-of-its-name)).

On a kind that declares the field, a body `## status:` chapter and a front-matter `status` key are two different things, with two coordinates, `ADR-x.status` and `ADR-x.front-matter.status`. How a field and a chapter-presence rule share one absence ([§FS-fields.rules](FS-fields.md#rules-fields-beside-chapter-rules)) is unchanged.

## v1: Version 1 is unchanged

The v1 reader produces exactly one field: `value_chapter = "<name>"` is a `may` field named `<name>` with `content = "values"` and nothing else, so every v1 finding and every byte of v1 output stays as it is ([§FS-config.5.2](FS-config.md#52-every-older-version-keeps-its-meaning)). `fields`, `closed`, `form` and `one_of` on a v1 `[[kinds]]` row stay unknown keys.

## checks: Checks

Each section below is one finding `grund check` reports about a declaration's shape, and its name is the finding code verbatim, as [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog) publishes it and [§REQ-spec-section-names.code](../requirements/REQ-spec-section-names.md#code-a-check-is-named-by-its-diagnostic-code) requires. Its channel is its strength's ([§FS-fields.strengths.table](FS-fields.md#strengthstable-which-strength-each-finding-takes)).

### checks.missing-field: Missing field

A declaration with no direct section of a field's name, reported at the declaration's title line: `FS-logout has no terms field; kind FS declares it; add its terms section`. The front-matter field is missing when its declaration has no front matter, and a declared key when the block has no key of its name. Each is reported with a remedy of its own ([§FS-fields.front-matter.findings](FS-fields.md#front-matterfindings-what-it-reports)): `ADR-cache has no front-matter field; kind ADR declares it; add front matter at the top of its file`, and, at the block's opening `---`, `ADR-cache.front-matter has no date key; kind ADR declares it; add the key to the front matter`.

### checks.misplaced-field: Misplaced field

A present field that does not stand where its placement puts it ([§FS-fields.order](FS-fields.md#order-order)), reported at the field's heading: `FS-login.terms is section 2 of 2; kind FS places it first; move the section there`. The counts are of direct sections. The placement reads `first`, `last`, or `directly after <name>`, naming the nearest present predecessor; it reads `first` when every predecessor of a chain whose head is `first` is absent.

### checks.mistitled-field: Mistitled field

A present field with a `title` whose display name differs from it, reported at the field's heading: `FS-reset.terms is titled "Glossary"; kind FS titles it "Terms"; retitle the heading to match`.

### checks.invalid-field-value: Invalid field value

A present scalar field whose display name is not in its `one_of`, reported at the field's heading, the values listed in their configured order: `DF-session.status is "Maybe"; kind DF admits Proposed, Accepted, Superseded; retitle the heading to one of them`. A front-matter key with `one_of` is reported at the key's line, its value in place of the display name: `ADR-0004-cache-availability.front-matter.status is "acepted"; kind ADR admits proposed, accepted, rejected, deprecated; change the value to one of them`. A value that is not a one-line scalar on the key's line reads `ADR-cache.front-matter.status is not a one-line value; kind ADR admits proposed, accepted; write one of them on the key's line` ([§FS-fields.front-matter.values](FS-fields.md#front-mattervalues-a-keys-value-is-its-decoded-text)).

### checks.undeclared-chapter: Undeclared chapter

A direct named section of a declaration of a closed kind that no field of the kind declares, reported at the section's heading, the fields listed in header order: `DF-session.surprise is not a field of kind DF, which is closed; its fields: status; rename it to one of them, number it, or add it to the kind's fields`. A closed kind with no field reads `its fields: none`. Numbered sections are always admitted.

A front-matter field whose `closed` is not `may` admits only its declared keys. Any other key is reported at its line, the declared keys listed in header order, `its keys: none` where there are none: `ADR-0004-cache-availability.front-matter.reviewer is not a key of kind ADR's front matter, which is closed; its keys: status, date, decision-makers; rename it to one of them, remove it, or add it to the field's keys`. A key that cannot be declared, because its name is not a section name or it is not a scalar, ends in `rename it to one of them or remove it` ([§FS-fields.front-matter.keys](FS-fields.md#front-matterkeys-its-keys-are-its-sections)). Neither the kind's `closed` nor this finding reports a `## front-matter:` heading in a kind that declares the field; that heading is [§FS-fields.checks.front-matter-heading](FS-fields.md#checksfront-matter-heading-front-matter-heading) alone.

### checks.unreadable-front-matter: Unreadable front matter

The front matter of a declaration whose kind declares the front-matter field, when grund cannot read its keys ([§FS-fields.front-matter.reading](FS-fields.md#front-matterreading-reading-the-block)). It is reported at the opening delimiter and is always an error. The message names the language, the reason, and the line of the file the reason is at:

```text
docs/adr/cache.md:1: error: ADR-cache.front-matter does not parse as YAML: <reason> at line 3; fix the YAML
docs/adr/toml.md:1: error: ADR-toml.front-matter does not parse as TOML: string values must be quoted at line 2; fix the TOML
```

The `<reason>` is the parser's message, the YAML parser's or the TOML parser's or decoder's, or one of grund's own: `its top level is not a mapping` and `a second document starts`, which only YAML gives, and `collections are nested more than 128 deep`, which both share. The field is present, so `missing-field` is not also reported, and no key of the block is judged.

### checks.front-matter-heading: Front-matter heading

A heading in a declaration whose kind declares the front-matter field, where the heading's section path is `front-matter` or begins with `front-matter.`. It is reported at the heading and is always an error: `ADR-cache.front-matter is a heading, but kind ADR reads front-matter from the file's front matter; rename the section`. The heading declares nothing, as a rejected section heading declares nothing ([§FS-cover.6.2](FS-cover.md#62-ownership-rules)), so it is not a section, not a rule unit and not also `undeclared-chapter`. In a kind that does not declare the field, `## front-matter:` is an ordinary named section.
