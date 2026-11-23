# FS-fields: a v2 kind states the shape of its declarations, and check holds every declaration to it

A version-2 kind can say what a declaration of it looks like: which named chapters it carries, in what order, what a scalar chapter may say, and whether anything else may appear. `grund check` then holds every declaration of the kind to that shape, on the channel each constraint's strength names ([§FS-config-v2.rules.strengths](FS-config-v2.md#rulesstrengths-one-strength-vocabulary-two-severities)), and the schema view and the agent guidance show the same shape. Decided in [§DF-kind-fields](../decisions/functional/DF-kind-fields.md#df-kind-fields-a-kind-states-its-fields-by-section-name-and-a-present-optional-field-is-still-checked). Serves [§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable) and [§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work).

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
docs/fs/FS-login.md:9: error: FS-login.terms is section 2 of 2; kind FS places it first; move the section there
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

Besides the three below, three requirements hold here and are stated where they are shared: a field's coordinate never moves when its display name does ([§REQ-spec-section-names](../requirements/REQ-spec-section-names.md#req-spec-section-names-a-section-name-says-what-the-thing-is-and-never-moves), served by [§FS-fields.identity.name](FS-fields.md#identityname-a-field-is-the-direct-section-of-its-name)), version 1 keeps its meaning ([§REQ-backwards-compatibility.1](../requirements/REQ-backwards-compatibility.md#1-what-is-covered), [§FS-fields.v1](FS-fields.md#v1-version-1s-only-field-is-its-value-chapter)), and a strength moves only the channel a finding reaches ([§FS-config-v2.requirements.strengths](FS-config-v2.md#requirementsstrengths-each-constraint-takes-the-strengths-it-can-mean-from-one-vocabulary-and-refuses-the-rest--realized-directional-for-the-next-constraint)).

### requirements.one-finding: One defect in a declaration's shape is one finding

A shape check reports each defect once, at the site to edit, and nothing downstream of it: an absent field only as absent ([§FS-fields.order.after](FS-fields.md#orderafter-after-means-directly-after)), a misplaced field without the fields after it ([§FS-fields.checks.misplaced-field](FS-fields.md#checksmisplaced-field-misplaced-field)), a field written twice or headed at the wrong depth only by its heading's own finding ([§FS-fields.identity.name](FS-fields.md#identityname-a-field-is-the-direct-section-of-its-name)), and an absence a field and a chapter-presence rule both see as the field's finding alone, while a defect only one of them sees is still reported ([§FS-fields.rules](FS-fields.md#rules-fields-beside-chapter-rules)). Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

### requirements.satisfiable: A shape the loader accepts can be met, and is held

The loader refuses, at the key that makes it so, a shape that contradicts itself or that nothing could hold: placements no declaration can meet together ([§FS-fields.order.refusals](FS-fields.md#orderrefusals-what-the-loader-refuses)), a `title` or `one_of` entry no heading can show ([§FS-fields.identity.match](FS-fields.md#identitymatch-titles-and-values-match-byte-for-byte)), a scalar with no values or with a `title` or `content` beside them ([§FS-fields.content.one-of](FS-fields.md#contentone-of-a-scalars-value-is-its-display-name)), and fields or `closed` on a kind whose declarations have no authored body to hold them ([§FS-fields.forms](FS-fields.md#forms-form-lowers-to-the-version-1-contracts)). Every shape it accepts is then held, so a present optional field out of place, mistitled or outside its `one_of` is an error ([§FS-fields.strengths](FS-fields.md#strengths-strengths-and-channels)). Serves [§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work), whose reader takes the shape as true of every declaration of the kind.

### requirements.record: The agent and a schema view read the shape the checks read

A kind's shape is one record ([§FS-fields.record](FS-fields.md#record-the-record)), and the checks, `grund schema` ([§FS-schema.4.1](FS-schema.md#41-fields-are-lines-inside-their-kinds-node)) and the managed block's `### Declaration shapes` ([§FS-init.2.3.5.12.2](FS-init.md#235122-declaration-shapes)) each read it rather than describing the shape again, the block in the words of the schema view's shape lines; `config show` prints it back only as configuration ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)). Serves [§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work), for which a shape the agent is never told is enforced only after the fact.

## declare: Declaring the shape

### declare.field: A field is a table under its kind

`[schema.kinds.<KIND>.fields.<name>]` declares one field of `<KIND>`. `<name>` is a section name in the fixed named-component grammar `[a-z][a-z0-9-]*`; any other name is refused at its header, ``` `[schema.kinds.FS.fields.Terms]`: a field name is a section name matching `[a-z][a-z0-9-]*` ```. A field table follows its row's header: one written above `[schema.kinds.<KIND>]` is refused at its line, ``` `[schema.kinds.FS.fields.terms]` names no row: write [schema.kinds.FS] above it ```. The table admits:

| Key | Value | Default |
|---|---|---|
| `title` | the display name the chapter must show | none: any display name |
| `presence` | a strength, `must`, `warn`, `should` or `may`: how strongly the field is held ([§FS-fields.strengths](FS-fields.md#strengths-strengths-and-channels)) | `may` |
| `position` | `first` or `last` | none |
| `after` | the name of another field of the same kind | none |
| `content` | `prose` or `values` | `prose` |
| `one_of` | a non-empty list of display names | none |

`front-matter` is the one field name with a meaning of its own, its declarations' front matter ([§FS-fields.front-matter](FS-fields.md#front-matter-front-matter-is-a-field)), and it admits the keys of [§FS-fields.front-matter.config](FS-fields.md#front-matterconfig-declaring-the-field-and-its-keys); a kind that does not declare it reads `## front-matter:` as any other named section.

Fields are listed, rendered and reported in the order their headers are written ([§FS-config-v2.reader.4](FS-config-v2.md#reader4-order-and-location-are-kept)), and header order is not placement: only `position` and `after` place a field. An unknown key is ``unknown key `<key>` in [schema.kinds.<KIND>.fields.<name>] (expected title, presence, position, after, content, or one_of)`` ([§FS-config-v2.reader.1.keys](FS-config-v2.md#reader1keys-an-unknown-key-names-the-keys-its-table-admits)). A value outside its set is refused at its line, ``unknown [schema.kinds.FS.fields.terms] presence `maybe` (expected must, warn, should, or may)``, and the same for `position` (`expected first or last`) and `content` (`expected prose or values`).

### declare.closed: `closed` is a strength on the row

`closed` on `[schema.kinds.<KIND>]` takes a strength and defaults to `may`, an open kind. Any other value is ``unknown [schema.kinds.<KIND>] closed `<value>` (expected must, warn, should, or may)``. The strength is the channel of [§FS-fields.checks.undeclared-chapter](FS-fields.md#checksundeclared-chapter-undeclared-chapter).

`closed` is also a key of the front-matter field, and there it governs that field's keys ([§FS-fields.front-matter.config](FS-fields.md#front-matterconfig-declaring-the-field-and-its-keys)). The kind's `closed` judges direct sections, and the front-matter field is none, so the kind's `closed` never reaches a key.

### declare.form: `form` is a row key

`form` on `[schema.kinds.<KIND>]` is `prose`, `value` or `rule`, and defaults to `prose`. Any other value is ``unknown [schema.kinds.<KIND>] form `<value>` (expected prose, value, or rule)``. What `value` and `rule` mean is [§FS-fields.forms](FS-fields.md#forms-form-lowers-to-the-version-1-contracts).

## identity: Identity

### identity.name: A field is the direct section of its name

A field is the declaration's **direct** section whose section name equals the field's name: `status` in `## status: Accepted`. It is never matched by its display name, and never by a same-named section nested deeper, so `<§>DF-session.status` resolves whatever the status says and the coordinate never moves ([§REQ-spec-section-names](../requirements/REQ-spec-section-names.md#req-spec-section-names-a-section-name-says-what-the-thing-is-and-never-moves), [§DF-kind-fields.2.1](../decisions/functional/DF-kind-fields.md#21-a-field-is-matched-by-its-section-name-and-its-title-and-values-byte-for-byte)). A value chapter has the same identity ([§FS-values.2.5](FS-values.md#25-chapter-declared-value-roots)). The heading is an ordinary named section, in Markdown or in a doc-comment declaration body; a heading with no section name, `## Status`, is an `unmarked-heading` finding and is never a field ([§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading)). A one-line stub pointing at a doc-comment declaration has no body and is held to no field. The one field that is not a direct section is `front-matter`, on a kind that declares it ([§FS-fields.front-matter](FS-fields.md#front-matter-front-matter-is-a-field)).

A field written twice is the `duplicate-section` error ([§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path)), and every field check reads the first occurrence. A field heading at the wrong depth keeps its heading-depth finding ([§FS-declarations.checks.section-heading-level](FS-declarations.md#checkssection-heading-level-section-heading-level-mismatch)) and is still the field, which reads the section path, as a value chapter does.

### identity.match: Titles and values match byte for byte

`title` and every `one_of` entry are compared with the display name byte for byte, with no case folding and no whitespace trimming beyond what the heading parser already removes. A configured entry no heading could show is refused at its line ([§FS-fields.requirements.satisfiable](FS-fields.md#requirementssatisfiable-a-shape-the-loader-accepts-can-be-met-and-is-held)):

- one that is empty or has leading or trailing whitespace: `field FS.terms has title " Terms", which is empty or has surrounding whitespace`, and `field DF.status has one_of entry " Accepted", which is empty or has surrounding whitespace`;
- one that holds a control character, a line break, a carriage return, a tab or any other, since a heading is one line and its display name holds none: `field FS.notes has title "A\tB", which holds a control character no heading can show`, and `field FS.notes has one_of entry "Acc\nepted", which holds a control character no heading can show`. The message names the value on one line, writing `\n`, `\r` and `\t` as those escapes and any other control character as `\u{<hex>}`. A value that is empty or padded gets the whitespace message, whatever else it holds.

A front-matter key's `one_of` entries are compared the same way with the key's value ([§FS-fields.front-matter.values](FS-fields.md#front-mattervalues-a-keys-value-is-its-decoded-text)), but an entry is not refused for a control character: a quoted value on the key's line, `status: "a\nb"`, decodes to a line break, so such an entry can be met.

## order: Order

### order.ends: `first` and `last`

`position = "first"` makes the field the declaration's first direct section, and `last` its last. Numbered and named direct sections both count; the lead does not, and neither does the front matter, which stands above the body ([§FS-fields.front-matter](FS-fields.md#front-matter-front-matter-is-a-field)).

### order.after: `after` means directly after

`after = "<p>"` makes the field the direct section immediately after field `<p>` ([§DF-kind-fields.2.2](../decisions/functional/DF-kind-fields.md#22-after-means-directly-after-and-an-absent-member-is-skipped)). A chain whose first field is placed `first` occupies the opening direct sections in chain order; a chain whose first field has no `position` fixes only the relative order of its members, each directly after its predecessor, wherever the chain begins. A field absent from a declaration, whether optional or required and reported missing, is skipped, and the next field of its chain takes its place: an AR whose kind places `placement` first and `terms` after it, and which has no `placement`, must open with `terms`, and gets one finding, `missing-field` for `placement`. Where the first field of a chain without a `position` is absent, the next field is unconstrained.

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

A `content = "prose"` field, the default, holds any body, which is never judged; its display name is held to `title` where one is set ([§FS-fields.checks.mistitled-field](FS-fields.md#checksmistitled-field-mistitled-field)).

### content.values: A values field is the value chapter

A field with `content = "values"` is the value chapter of [§FS-values.2.5](FS-values.md#25-chapter-declared-value-roots): the same identity, the same value roots, the same `invalid-value-declaration` findings at the same lines, and the same prerequisites ([§FS-config.3.4.13](FS-config.md#3413-value_chapter--the-chapter-whose-named-children-are-values)). It is v1's `value_chapter` and nothing else, plus whatever presence, placement and title the field adds ([§DF-kind-fields.2.4](../decisions/functional/DF-kind-fields.md#24-the-value-chapter-moves-into-the-field-list)). A kind has at most one such field, and a second is refused with v1's one-chapter refusal at its `content` key.

### content.one-of: A scalar's value is its display name

A scalar field's display name outside its `one_of` is [§FS-fields.checks.invalid-field-value](FS-fields.md#checksinvalid-field-value-invalid-field-value), and the body below its heading is ordinary prose. Each of these is refused at its line, and a conflicting pair at the later of its two keys:

- `one_of` beside `title`: `field DF.status sets one_of and title; a scalar's title is its value`;
- `one_of` beside `content`: `field DF.status sets one_of and content; a scalar's content is its title`;
- an empty `one_of`: `field DF.status has an empty one_of; a scalar needs at least one value`;
- an entry listed twice: `field DF.status lists "Accepted" twice in one_of`.

## strengths: Strengths and channels

A field's `presence` is the strength of each of its findings, `missing-field`, `misplaced-field`, `mistitled-field` and `invalid-field-value`, so `warn` or `should` softens a present field's placement, title and value as well as its absence, and the kind's `closed` is the strength of `undeclared-chapter`. On the front-matter field, a key's own `presence` is the strength of its `missing-field` and of its `invalid-field-value`, and the field's own `closed` is the strength of `undeclared-chapter` for its keys. A strength moves only the channel, never the code or the message ([§FS-config-v2.rules.strengths](FS-config-v2.md#rulesstrengths-one-strength-vocabulary-two-severities)); the channel each strength gives, and the severity of a code no strength reaches, are the code's catalog row ([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)).

`may` waives absence and nothing else: a present `may` field is held to its placement, title and values as a `must` field is, and a present `may` key to its `one_of` ([§DF-kind-fields.2.3](../decisions/functional/DF-kind-fields.md#23-may-waives-absence-only)).

## forms: `form` lowers to the version-1 contracts

`form = "value"` means what v1's `values = true` means ([§FS-config.3.4.9](FS-config.md#349-values--first-class-value-declarations)), and `form = "rule"` what v1's `rules = true` means ([§FS-config.3.4.12](FS-config.md#3412-rules--rule-declaration-kinds)): the same declarations, the same findings, and the same prerequisites, refused with the same messages located at the `form` key. A `form = "rule"` kind with a `content = "values"` field is v1's `rules = true` with `value_chapter`.

One prerequisite of `form = "rule"` is refused in version 2's own words, at the `form` key ([§DF-reading-at-fetch-and-unread-homes.2.7](../decisions/functional/DF-reading-at-fetch-and-unread-homes.md#27-version-2s-rule-home-refusal-speaks-version-2)): a `files` entry that is not read as Markdown, because a `data` language reads it or no admitted language does ([§FS-config-v2.schema.sources.rules](FS-config-v2.md#schemasourcesrules-every-home-of-a-rule-kind-is-judged-by-its-reading)). The refusal names the first such entry in the order the key writes them, and the language that reads it:

```text
error: grund.toml:12: [schema.kinds.RULE] sets `form = "rule"`, but files entry `rules/rules.md` is read as `json`, not as Markdown; move the rules into a file read as Markdown
error: grund.toml:9: [schema.kinds.RULE] sets `form = "rule"`, but no admitted language reads files entry `rules/rules.md`; move the rules into a file read as Markdown
```

A `files` entry a `source` language reads is refused at load ([§FS-config-v2.checks.config-rule-file-not-markdown](FS-config-v2.md#checksconfig-rule-file-not-markdown-a-rule-kinds-files-entry-is-read-as-source)), and every other prerequisite keeps version 1's words at the `form` key.

A kind whose declarations have no authored body to hold a field takes no fields and cannot be closed. Each is refused at the field's header or at `closed`:

- `form = "value"`: `CONST is form = "value": its children are numbered components, so it takes no fields` (`… so it cannot be closed` at `closed`);
- `citable = false`: `NOTES is not citable: it declares nothing, so it takes no fields` (`… so it cannot be closed`);
- `fetch`: `TICKET is fetched: its declarations are a snapshot, so it takes no fields` (`… so it cannot be closed`).

## rules: Fields beside chapter rules

A chapter-presence rule ([§FS-rules.3.1](FS-rules.md#31-chapter-presence)) is valid, rendered, citable and evaluated beside fields in both versions, whether its subject is a kind or one exact declaration. Where a field expresses a presence rule and both see the same absence in one declaration, only the field's finding is reported ([§FS-rules.6](FS-rules.md#6-semantic-deduplication), [§DF-kind-fields.2.5](../decisions/functional/DF-kind-fields.md#25-a-field-and-a-rule-are-paired-at-the-merge-not-deduplicated-before-evaluation)); a defect only one of them sees is still reported by that one, so adding a field never weakens a rule. A scalar field can also narrow a rule's subject: `Each SWR whose verification is Test` selects the declarations whose `verification` field holds `Test`, read by the same display name `invalid-field-value` checks ([§FS-rules.2.2](FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)).

## record: The record

A kind's shape is one record of slots. Each slot carries its name, title, presence, placement and content (prose, values, or the scalar's values), and the kind carries `closed`; the front-matter slot carries what [§FS-fields.front-matter.record](FS-fields.md#front-matterrecord-the-record-and-what-renders-it) says. Slots stay in header order, and placement is data on each slot rather than a reordering.

## front-matter: Front matter is a field

On a kind that declares `[schema.kinds.<KIND>.fields.front-matter]`, that field is not a chapter. It is the front matter of its declaration's file, the block [§FS-check.1.1.5.2](FS-check.md#1152-front-matter-holds-no-heading) delimits, and the block's top-level keys are the field's sections. Decided in [§DF-front-matter-field](../decisions/functional/DF-front-matter-field.md#df-front-matter-field-a-kinds-front-matter-field-holds-its-files-yaml-front-matter-with-the-blocks-top-level-keys-as-its-sections), and for a `+++` block in [§DF-toml-front-matter](../decisions/functional/DF-toml-front-matter.md#df-toml-front-matter-a-kinds-front-matter-field-reads-a-toml-block-as-it-reads-a-yaml-one). Serves [§GOAL-configurable.1](../goals.md#1-what-is-configurable) and [§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work).

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

Any other key, `title`, `position`, `after`, `content` and `one_of` among them, is refused at its line, ``unknown key `position` in [schema.kinds.ADR.fields.front-matter] (expected presence or closed)`` ([§FS-config-v2.reader.1.keys](FS-config-v2.md#reader1keys-an-unknown-key-names-the-keys-its-table-admits)), and a value outside its set as on any field, ``unknown [schema.kinds.ADR.fields.front-matter] closed `yes` (expected must, warn, should, or may)``. `closed` is an unknown key on every other field.

`[schema.kinds.<KIND>.fields.front-matter.keys.<key>]` declares one key, a top-level key of the block. `<key>` is a section name in the field-name grammar `[a-z][a-z0-9-]*`; any other name is refused at its header in the field's words, ``` `[schema.kinds.ADR.fields.front-matter.keys.Status]`: a key name is a section name matching `[a-z][a-z0-9-]*` ```. A key table follows its field's header, as a field table follows its row ([§FS-fields.declare.field](FS-fields.md#declarefield-a-field-is-a-table-under-its-kind)); written above `[schema.kinds.<KIND>.fields.front-matter]`, it is refused at its line, ``` `[schema.kinds.ADR.fields.front-matter.keys.status]` names no field: write [schema.kinds.ADR.fields.front-matter] above it ```. The table admits two keys:

| Key | Value | Default |
|---|---|---|
| `presence` | a strength: whether the block must have the key | `may` |
| `one_of` | a non-empty list of values | none: any value |

Any other key is ``unknown key `title` in [schema.kinds.ADR.fields.front-matter.keys.status] (expected presence or one_of)``. `one_of` keeps the refusals of [§FS-fields.content.one-of](FS-fields.md#contentone-of-a-scalars-value-is-its-display-name) and [§FS-fields.identity.match](FS-fields.md#identitymatch-titles-and-values-match-byte-for-byte), each naming the key where they name the field: `key ADR.front-matter.status has an empty one_of; a scalar needs at least one value`. Keys are listed, rendered and reported in the order their headers are written.

`[schema.kinds.<KIND>.fields.front-matter.keys]` holds only tables: written as a header of its own with a key in it, it is ``unknown key `status` in [schema.kinds.ADR.fields.front-matter.keys] (it holds only tables)``, as `[schema.kinds.<KIND>.fields]` is ([§FS-config-v2.reader.2](FS-config-v2.md#reader2-every-row-and-field-is-written-under-its-own-header)). A `keys` table under any other field is an unknown config section, ``unknown config section `schema.kinds.ADR.fields.context.keys.status` ``.

### front-matter.reading: Reading the block

The block belongs to its file's first declaration ([§FS-check.1.1.5.2.1](FS-check.md#11521-its-citations-belong-to-the-files-first-declaration)) and is read as the field only when that declaration's kind declares it; any other block reads as [§FS-check.1.1.5.2](FS-check.md#1152-front-matter-holds-no-heading) says. A declaration of a kind that declares the field has no front matter when it is not its Markdown file's first declaration, when it is a doc-comment declaration, or when its file's line 1 begins with a byte-order mark ([§FS-check.1.1.5.2](FS-check.md#1152-front-matter-holds-no-heading)); where the field's `presence` is not `may`, each gets a `missing-field` ([§FS-fields.checks.missing-field](FS-fields.md#checksmissing-field-missing-field)).

A `---` block's content is YAML and a `+++` block's is TOML 1.1 ([§DA-yaml-value-reader](../decisions/architectural/DA-yaml-value-reader.md#da-yaml-value-reader-one-yaml-event-parser-builds-one-span-tree-and-value-rules-are-a-pass-over-it), [§DA-toml-front-matter-reader](../decisions/architectural/DA-toml-front-matter-reader.md#da-toml-front-matter-reader-one-toml-event-parser-builds-the-span-tree-a-toml-front-matter-block-is-read-through)), from the line after the opening delimiter to the line before the closing one, and every line a finding names is the file's own. A CRLF file reads as its LF form does: the same keys and values, at the same lines and columns, and no value or slice ends in a carriage return.

The content is one YAML document whose top level is a mapping, or one TOML document. Content that is empty, or only comments and blank lines, is a present field with no keys. Each of these is `unreadable-front-matter` ([§FS-fields.checks.unreadable-front-matter](FS-fields.md#checksunreadable-front-matter-unreadable-front-matter)):

- YAML content the YAML parser rejects, a tab in indentation included, named by the parser's message and the line it gives;
- TOML content the TOML parser or its decoder rejects, named by the first such message in source order and the line of the first byte it did not expect. A string still open when the content ends is reported at the closing delimiter's line, and a key TOML refuses to define again is not a parse error but a key written twice ([§FS-fields.front-matter.keys](FS-fields.md#front-matterkeys-its-keys-are-its-sections));
- YAML whose top level is a sequence or a scalar;
- YAML that holds a second document: a line such as `--- # x` does not close the block ([§FS-check.1.1.5.2](FS-check.md#1152-front-matter-holds-no-heading)), but in YAML it starts a document;
- collections nested more than 128 deep, at the line of the collection that goes past: the one nesting bound shared with value sources ([§FS-values.5.3](FS-values.md#53-incomplete-input-and-deterministic-output)), so no block makes a reader recurse past it ([§REQ-never-crashes.1](../requirements/REQ-never-crashes.md#1-every-failure-is-a-diagnostic)). The top level is the first level; in TOML each array and each inline table adds one, and a table opened by a header or a dotted key adds none, so `notes = [[…]]` is refused exactly where `notes: [[…]]` is.

### front-matter.keys: Its keys are its sections

The field's keys are the top-level keys of the block, in source order; a key nested inside a value is part of that value. A key is named by its decoded scalar, so `status`, `'status'` and `"status"` are one key, and a key whose name is a section name, `[a-z][a-z0-9-]*`, is the section `<ID>.front-matter.<key>`, whether or not the field declares it. A key and a field of one name, `status`, are two sections, `<ID>.front-matter.status` and `<ID>.status`, and neither satisfies the other ([§FS-fields.identity.name](FS-fields.md#identityname-a-field-is-the-direct-section-of-its-name)). Any other key has no coordinate and cannot be declared: a scalar key outside the grammar, such as `Reviewer` or `<<`, named in a finding by its decoded text, or a key that is not a scalar, a collection written as a key (`? [a, b]`) or an alias (`*k`), named by the text it is written as on its line, `ADR-cache.front-matter.*k`. A merge key `<<` merges nothing, because no YAML resolution is applied ([§DA-yaml-value-reader.2.1](../decisions/architectural/DA-yaml-value-reader.md#21-one-event-parser)), so a `status` that only a merge supplies is absent.

In a TOML block, each key the document writes at its top level, by `k = …`, a dotted key or a header, contributes to the top-level key named by its decoded first component ([§DF-toml-front-matter.decision.2](../decisions/functional/DF-toml-front-matter.md#decision2-a-key-is-the-first-component-of-what-the-document-writes)): `links.next = …` and `[links.next]` both contribute to `links`, and `"links.next" = …` to the one key `links.next`, which is not a section name. A key's site, and its place in source order, is its first contribution, and the components after the first are part of its value.

A key that is a section, written twice and compared decoded, is `duplicate-section` at its first occurrence, naming the others ([§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path)); every check reads that occurrence, as for a field written twice, and a read of the coordinate is refused as ambiguous ([§FS-show.2.2.2](FS-show.md#222-ambiguous-section)). A key with no coordinate that is written twice is not a duplicate: it names no citation target. In TOML, a key is written twice by each later contribution that TOML refuses because of what the allowed contributions before it have made the key, such as `k = 1` then `k.x = 2`, and never by one TOML allows, such as `k.x = …` then `k.y = …`; the finding names the line of each refused contribution and of no allowed one. A conflict below the top level, such as `[k.x]` twice, is part of `k`'s value and is not judged, as a key repeated inside a nested YAML mapping is not.

### front-matter.values: A key's value is its decoded text

A key's value is its scalar's text, with quotes and escapes resolved and no YAML or TOML type ever applied ([§DF-front-matter-field.decision.3](../decisions/functional/DF-front-matter-field.md#decision3-a-value-is-its-decoded-text), [§DF-toml-front-matter.decision.3](../decisions/functional/DF-toml-front-matter.md#decision3-a-value-keeps-its-spelling)). In YAML, `1.10`, `no` and `2026-02-03` stay as written, `'accepted'`, `"accepted"` and plain `accepted` are all `accepted`, and a tag or anchor on the value is ignored, so `!!str accepted` is `accepted` too. In TOML, a string, basic or literal, on one line or several, is its decoded text, so `'accepted'` and `"""accepted"""` are `accepted`; a number, boolean, date or time is its raw spelling, so `1_000` and `0x1F` stay those exact characters; and a bare word, `status = accepted`, is no value: the block is `unreadable-front-matter` ([§FS-fields.checks.unreadable-front-matter](FS-fields.md#checksunreadable-front-matter-unreadable-front-matter)). A YAML key with nothing after its colon, `status:`, is present with the value `""`, as TOML's `status = ""` is. A TOML key opened by a header or reached by a dotted key has the table they make as its value.

`one_of` takes a one-line value: a plain or quoted scalar that starts on the key's line and ends on it, as a TOML `"""accepted"""` written on one line does. A YAML block scalar (`|`, `>`), whose kept trailing newline would compare as `accepted\n`, a scalar or string that spans lines, a sequence or array, a mapping or inline table, a YAML alias, and a TOML table are not; each is `invalid-field-value` at the key's site ([§FS-fields.checks.invalid-field-value](FS-fields.md#checksinvalid-field-value-invalid-field-value)). A key without `one_of` holds any value, nested or not, and nothing in it is judged.

### front-matter.coordinates: Its coordinates

The first declaration of a file whose kind declares the field gains `<ID>.front-matter`, at the opening delimiter's line, and `<ID>.front-matter.<key>`, at the line of each key that is a section. Both are entries of the declaration's one section map, marked as lying in the front matter ([§FS-declarations.section.map](FS-declarations.md#sectionmap-the-section-map-is-the-declarations-body-span)), so every surface that reads the map answers for them alike ([§REQ-one-answer.sections.map](../requirements/REQ-one-answer.md#sectionsmap-every-surface-reads-one-section-map)). Neither is a heading or inside the body, which starts at the declaration's heading ([§FS-declarations.body](FS-declarations.md#body-where-a-declarations-body-ends)), and neither is ever enrolled in a section lock ([§FS-section-lock.holds.entry](FS-section-lock.md#holdsentry-one-line-per-locked-address)). A declaration with no front matter has neither, and one with an unreadable block has `<ID>.front-matter` and no key coordinate.

A coordinate's **slice** is its text in the file:

- for `<ID>.front-matter`, from the first byte of the opening delimiter to the last byte of the closing one, its line ending excluded;
- for a YAML key, from the key's first byte to the last byte of its value, as [§FS-values.2.2.2](FS-values.md#222-reader-fidelity) cuts a member. Comments between the key and the end of its value are inside, and comments before the key or after its value are not. A key with no value ends at its colon;
- for a TOML key, from the first byte of its first contribution, which is the key's own first byte, its opening quote where it is quoted, or the `[` or `[[` of a header, to the last byte of the last contribution in the key's **first run**, the contributions to the key that follow one another with no other top-level key's contribution between them. A `k = …` or `k.x = …` contribution ends at the last byte of its value, and a header's at the last byte of the last value written under it before the next header, or at its own `]` or `]]` where it holds none. Comments and blank lines after that byte are not inside. A key contributed to again after another key's contribution (`[links.extra]` after `[[authors]]` reopens `links`) gains no second slice: those lines are in `<ID>.front-matter`, as a comment between keys is.

A key's **title** is its value where that is a one-line scalar on the key's line, and it has none otherwise. `<ID>.front-matter` has none.

### front-matter.surfaces: What each surface does with them

| Surface | What it does with `ADR-x.front-matter` and `ADR-x.front-matter.status` |
|---|---|
| `grund show` of one of them | prints its slice with nothing added, in the default read, `--brief`, `--toc` and `--full` alike, since a slice has no lead of its own; its JSON is [§FS-output-shapes.4.4](FS-output-shapes.md#44-front-matter-coordinates)'s |
| a read of the whole declaration, in any mode | neither prints the block nor lists its coordinates, since the block is above the body ([§DF-front-matter-field.decision.4](../decisions/functional/DF-front-matter-field.md#decision4-whole-declaration-reads-stay-the-same)); a whole-declaration `--toc` lists headings only |
| `show --batch --all` | generates one query for each, among the declaration's section paths in their byte order ([§FS-show.2.6](FS-show.md#26-batch-resolution)) |
| `list --glob` | lists them as catalog coordinates ([§FS-coordinate-globs.matching.catalog](FS-coordinate-globs.md#matchingcatalog-a-glob-selects-catalog-coordinates)). `ADR-*.front-matter.*` lists the keys, with a key's title in the title column |
| `list --size` | one row for each, where lead and full both measure the slice ([§FS-output-shapes.5.2](FS-output-shapes.md#52-list---size---formatjson)). The lead-size check measures the same rows, so an oversized slice is warned at its coordinate's line ([§FS-declarations.checks.oversized-lead](FS-declarations.md#checksoversized-lead-oversized-lead-opt-in)) |
| `grund complete ids --sections` | offers them as sections of their declaration ([§FS-completions.2.2](FS-completions.md#22-what-the-catalog-offers)) |
| `refs`, `cover --glob`, and the language server's resolution and navigation | resolve them and go to the coordinate's line. `fmt` links a Markdown citation of one to the file with no fragment, as it does for a source-file home |
| `cover --lines` | a line inside a key's slice is in `front-matter.<key>`; the delimiter lines, comments and blank lines between keys, and the lines of a key with no coordinate are in `front-matter` ([§FS-cover.6.2](FS-cover.md#62-ownership-rules)) |
| `enclosing_section` of a citation in the block | `front-matter.<key>`, or `front-matter` outside every key's slice ([§FS-check.1.1.5.2.1](FS-check.md#11521-its-citations-belong-to-the-files-first-declaration)) |
| rule subjects | `front-matter` and `front-matter.<key>` are chapter units selected by their whole path ([§FS-rules.2.1](FS-rules.md#21-a-chapters-name-is-its-whole-path)). A chapter-presence rule counts direct chapters by display name ([§FS-rules.3.1](FS-rules.md#31-chapter-presence)), so it never counts them |
| the snapshot's `Section` record | an entry of `sections` like any other: its `path`, `line` and `title` as above, `title` the empty string where it has none, `value` and `valueRoot` `null`, and `headingLevel` `0`, which is the front-matter mark, since the record has no field of its own for it ([§FS-distribution.3.2.2.3](FS-distribution.md#3223-scan-snapshot)) |

### front-matter.record: The record, and what renders it

The front-matter slot carries its presence, its own `closed`, and its keys in header order, each key with its presence and its `one_of` values ([§FS-fields.record](FS-fields.md#record-the-record)). Its line is never a heading ([§FS-fields.identity.name](FS-fields.md#identityname-a-field-is-the-direct-section-of-its-name)). Every view prints its keys, and then its own `closed` where it is not `may`, nested under its field line, in the words of a shape line: `grund schema` as [§FS-schema.4.1.3](FS-schema.md#413-front-matter-is-no-heading) gives them, Mermaid and DOT as [§FS-schema.9](FS-schema.md#9-the-mermaid-source) and [§FS-schema.10](FS-schema.md#10-the-dot-source) do, and the managed block's `### Declaration shapes` as [§FS-init.2.3.5.12.2](FS-init.md#235122-declaration-shapes) does. So the kind of [§FS-fields.front-matter](FS-fields.md#front-matter-front-matter-is-a-field) renders in `grund schema` as:

```text
  ADR   kind                     docs/adr/0004-cache-availability.md
          field front-matter     `---` YAML or `+++` TOML front matter at the top of the file (must)
            key status           `status: <one of proposed, accepted, rejected, deprecated>` (must)
            key date             `date: <value>` (must)
            key decision-makers  `decision-makers: <value>` (may)
            closed               no other key (must)
          field context          `## context: <title>` (must)
          field decision         `## decision: <title>` (must)
          closed                 no other chapter except numbered ones (must)
  code  complement               every scanned file no other row claims
```

## v1: Version 1's only field is its value chapter

The v1 reader produces one field, from `value_chapter = "<name>"`: a `may` field named `<name>` with `content = "values"` and nothing else ([§FS-config.5.2](FS-config.md#52-a-versions-meaning-moves-only-as-backwards-compatibility-allows)). `fields`, `closed`, `form` and `one_of` on a v1 `[[kinds]]` row are unknown keys.

## checks: Checks

Each section below is one finding `grund check` reports about a declaration's shape, named by its finding code ([§REQ-spec-section-names.code](../requirements/REQ-spec-section-names.md#code-a-check-is-named-by-its-diagnostic-code)) and reported as every finding is ([§FS-check.2.1](FS-check.md#21-report-format)), at the severity its catalog row gives ([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)) from the strength [§FS-fields.strengths](FS-fields.md#strengths-strengths-and-channels) names, where the row takes one.

### checks.missing-field: Missing field

A declaration with no direct section of a field's name, reported at the declaration's title line: `FS-logout has no terms field; kind FS declares it; add its terms section`. A declared front-matter key is missing when the block has no key of its name, reported at the opening delimiter: `ADR-cache.front-matter has no date key; kind ADR declares it; add the key to the front matter`. No key is judged when the block is absent or unreadable.

The front-matter field is missing when its declaration has no front matter ([§FS-fields.front-matter.reading](FS-fields.md#front-matterreading-reading-the-block)), reported at the title line:

```text
docs/adr/cache.md:1: error: ADR-cache has no front-matter field; kind ADR declares it; add front matter at the top of its file
```

Where the declaration has none because it is not its Markdown file's first declaration, because it is a doc-comment declaration, or because a byte-order mark stands before its `---` or `+++` on line 1, the remedy says so instead ([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)).

### checks.misplaced-field: Misplaced field

A present field that does not stand where its placement puts it ([§FS-fields.order](FS-fields.md#order-order)), reported at the field's heading: `FS-login.terms is section 2 of 2; kind FS places it first; move the section there`. The counts are of direct sections, never the front matter. Each field is judged only against its nearest present predecessor in its chain, or against the position its chain's head fixes when every predecessor is absent, and the fields after a misplaced field are judged against where it actually stands, so one field out of place is one finding: a declaration that writes `1.`, `placement`, `terms` under the AR shape of [§FS-fields.order.after](FS-fields.md#orderafter-after-means-directly-after) gets one, for `placement`. The placement reads `first`, `last`, or `directly after <name>`, naming the nearest present predecessor; it reads `first` when every predecessor of a chain whose head is `first` is absent.

### checks.mistitled-field: Mistitled field

A present field with a `title` whose display name differs from it, reported at the field's heading: `FS-reset.terms is titled "Glossary"; kind FS titles it "Terms"; retitle the heading to match`.

### checks.invalid-field-value: Invalid field value

A present scalar field whose display name is not in its `one_of`, reported at the field's heading, the values listed in their configured order: `DF-session.status is "Maybe"; kind DF admits Proposed, Accepted, Superseded; retitle the heading to one of them`. A front-matter key with `one_of` is reported at the key's line, its value in place of the display name: `ADR-0004-cache-availability.front-matter.status is "acepted"; kind ADR admits proposed, accepted, rejected, deprecated; change the value to one of them`. A value that is not a one-line scalar on the key's line ([§FS-fields.front-matter.values](FS-fields.md#front-mattervalues-a-keys-value-is-its-decoded-text)) reads `ADR-cache.front-matter.status is not a one-line value; kind ADR admits proposed, accepted; write one of them on the key's line`.

### checks.undeclared-chapter: Undeclared chapter

A direct named section of a declaration of a closed kind that no field of the kind declares, reported at the section's heading, the fields listed in header order: `DF-session.surprise is not a field of kind DF, which is closed; its fields: status; rename it to one of them, number it, or add it to the kind's fields`. A closed kind with no field reads `its fields: none`. Numbered sections are always admitted.

A front-matter field whose `closed` is not `may` admits only its declared keys. Any other key, one that cannot be declared included ([§FS-fields.front-matter.keys](FS-fields.md#front-matterkeys-its-keys-are-its-sections)), is reported at its line, the declared keys listed in header order, `its keys: none` where there are none: `ADR-0004-cache-availability.front-matter.reviewer is not a key of kind ADR's front matter, which is closed; its keys: status, date, decision-makers; rename it to one of them, remove it, or add it to the field's keys`.

### checks.unreadable-front-matter: Unreadable front matter

The front matter of a declaration whose kind declares the front-matter field, when grund cannot read its keys ([§FS-fields.front-matter.reading](FS-fields.md#front-matterreading-reading-the-block)), reported at the opening delimiter. The message names the language, the reason, and the line of the file the reason is at:

```text
docs/adr/second.md:1: error: ADR-second.front-matter does not parse as YAML: a second document starts at line 3; fix the YAML
docs/adr/toml.md:1: error: ADR-toml.front-matter does not parse as TOML: string values must be quoted at line 2; fix the TOML
```

The reason is the YAML parser's message, the TOML parser's or decoder's, or one of grund's own: `its top level is not a mapping` and `a second document starts`, which only YAML gives, and `collections are nested more than 128 deep`, which both share.

### checks.front-matter-heading: Front-matter heading

A heading in a declaration whose kind declares the front-matter field, where the heading's section path is `front-matter` or begins with `front-matter.`, reported at the heading: `ADR-cache.front-matter is a heading, but kind ADR reads front-matter from the file's front matter; rename the section`. It is the heading's only finding, never also `undeclared-chapter`, because the heading declares nothing, as a rejected section heading declares nothing ([§FS-cover.6.2](FS-cover.md#62-ownership-rules)).
