# FS-config-v2: grund reads a version-2 config by concern, with one strength vocabulary and fixed defaults

A `grund.toml` that writes `grund_config_version = 2` is read by a reader of its own and spelled by concern: an envelope, then `[schema]` for what exists, `[rules]` for how nodes relate, and `[presentation]` for the bytes `grund` writes ([§FS-config.concerns](FS-config.md#concerns-every-key-belongs-to-exactly-one-concern)). Every constraint takes its strength from one closed set, so a repository can adopt a constraint as a warning, promote it to an error, or keep it advisory without changing format, and the defaults are fixed at the version rather than at whatever a binary ships ([§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable), [§GOAL-zero-config](../goals.md#goal-zero-config-works-on-any-conformant-tree)). Version 1 stays what it is: every v1 key, default and meaning in [§FS-config](FS-config.md#fs-config-grund-reads-a-toml-config-file-found-by-walking-up) is read by the v1 reader and keeps its meaning permanently ([§FS-config.5.2](FS-config.md#52-every-older-version-keeps-its-meaning)). Nothing in this file changes a v1 file's meaning.

This is the smallest working v2 file:

```toml
grund_config_version = 2
project_name = "example"
[schema.sources]
languages = ["markdown"]
[schema.kinds.FS]
files = ["requirements.md"]
[schema.kinds.GOAL]
files = ["docs/goals.md"]
[rules.citations.FS]
warn = ["GOAL"]
```

`grund check` reports a declaration in `requirements.md` that cites no `GOAL` as a standing warning, exit `0`, and the warning clears once the citation is written. Writing `must` makes it an error, `should` an opt-in suggestion, and `may` leaves it unchecked.

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, kind, home), [§FS-terms.terms.2](FS-terms.md#terms2-citations) (marker, citation), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan) and [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding). Defines:

- **Reader**: the code that turns one version's spelling into the project's records. Each version has its own, and a file is read by exactly one ([§FS-config.5](FS-config.md#5-schema-versioning)).
- **Strength**: one of `must`, `warn`, `should`, `may`, or a prohibition `must-not`, `warn-not`, `should-not`. It names the channel a finding reaches ([§FS-config-v2.rules.strengths](FS-config-v2.md#rulesstrengths-one-strength-vocabulary-two-severities)).
- **Measure**: a quantity a constraint compares against a threshold, such as a note's lines or a lead's words. A measure is a table name and its strengths are its keys ([§FS-config-v2.schema.measures](FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys)).
- **Ladder**: one or more grounding rungs (a strength paired with a unit) written as one setting ([§FS-config-v2.rules.grounding](FS-config-v2.md#rulesgrounding-grounding-is-a-ladder-replaced-whole)).
- **Epoch**: the defaults a version fixes. A later binary never changes them ([§FS-config-v2.defaults](FS-config-v2.md#defaults-the-v2-epoch)).
- **Rollout refusal**: a located load error for a v2 clause the format defines but this binary does not yet execute ([§FS-config-v2.rollout](FS-config-v2.md#rollout-what-this-grund-executes-and-what-it-refuses)).

## reader: The reader

### reader.1: Tables and keys are closed

The v2 reader admits exactly the tables and keys this file names, at the scope each is named at. An unknown table is ``unknown config section `<table>` ``. An unknown key is ``unknown key `<key>` in [<table>]``. A key written at a table that does not admit it is refused the same way. A key repeated inside one table is ``duplicate key `<key>` in [<table>] (first written at line <n>)``. Every refusal is located at the offending line, per [§FS-config.4.3](FS-config.md#43-invalid-config-behavior).

v1's spellings are not v2 keys. `[reference]`, `[id]`, `[[kinds]]`, `[scan]`, `[output]`, `[citations]` and `[fmt.cross_refs]` are unknown sections in v2, and so are `strict`, `named_sections`, `include`, and every other discarded alias ([§DISC-core-concerns.7.4](../discussions/proposals/2026-09-30-core-concerns.md#74-what-the-format-leaves-out)). v2 reads no `include`: the scan is the union of the rows' places ([§FS-config-v2.schema.places](FS-config-v2.md#schemaplaces-rows-and-their-places)). Report preferences are run flags ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)), so `[output]` has no v2 spelling either.

### reader.2: Every row and field is written under its own header

A keyed row is written under its own explicit header, `[schema.kinds.<NAME>]`, and a field under `[schema.kinds.<NAME>.fields.<field>]`, so every key names its owner. An explicit header that is written twice is ``repeated table `[<table>]` (first written at line <n>)``, located at the second. A parent that TOML creates implicitly, as `[schema]` is created by writing `[schema.sources]`, is not an explicit header: writing `[schema]` after it is not a repetition.

### reader.3: One setting, one form

A setting has one form. Where the format spells a constraint as a table whose keys are strengths, a strength key is never also a table: `[rules.citations.FS.must]` beside `[rules.citations.FS] must = [...]`, or `[schema.notes.lines.must]`, is `` `must` is a strength key in [<table>], not a table ``. A value of the wrong TOML type is refused at its line with the type it expected.

### reader.4: Order and location are kept

The reader keeps the line every value was written at, so the one validation pass after reading anchors an error where the value was written ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)). It keeps the authored order of the explicit headers too: rows are listed, rendered and reported in the order the file writes them, as `[[kinds]]` rows are in v1.

## defaults: The v2 epoch

### defaults.1: What v2 fixes

A key a v2 file omits takes the value below. These are the v2 epoch, and no later binary changes them; a different default is a different version ([§FS-config.5.2](FS-config.md#52-every-older-version-keeps-its-meaning)).

| Setting | v2 default | v1 default, unchanged |
|---|---|---|
| ID format, project and row | `{kind}-{slug}` | `{kind}-{number}-{slug}` |
| Slug pattern | `[a-z][a-z0-9-]*`, letter-led | `[a-z0-9][a-z0-9-]*` |
| Citation recognition | marked only; there is no opt-out | `strict = true`, `false` admits bare IDs |
| Named sections | always enabled; there is no switch | `named_sections = false` |
| `[schema] heading_depth` | `must` | `section_heading_levels = "strict"` |
| `[schema.sources] respect_gitignore` | `true` | `true` |
| The scan | the union of the rows' places, and the config root when no row has a place | `[scan] include` |
| `[rules.citations] default` | `may` | no default level |
| Languages | the epoch set of #456's approved proposal | `[scan] extensions` and `comment_prefixes` |

The marker, `shorthand`, `section_separator` and `number_pattern` keep their v1 defaults. A v2 file that declares no `[schema.kinds]` row has no kinds: v1's built-in kind list belongs to v1.

### defaults.2: The language set is fixed before it is executed

The v2 language set, with its suffixes, declaration hosts and explicit opt-in languages, is the one agent-grounds/grund#456's proposal fixed on 2026-10-06. Until a binary executes that exact set, a v2 file that omits `[schema.sources] languages` is refused at the version key: `v2 needs an explicit [schema.sources] languages: this grund does not support the v2 default language set yet`. The v1 extension list is never substituted for it. `languages = ["markdown"]` is executed now, by the Markdown scanner every version shares.

## schema: The schema concern

### schema.1: `[schema]`

`[schema]` admits `marker`, `shorthand`, `id_format`, `slug_pattern`, `number_pattern`, `section_separator` and `heading_depth`. All but `heading_depth` mean what their v1 counterparts in [§FS-config.3.1](FS-config.md#31-reference--citation-form) and [§FS-config.3.2](FS-config.md#32-id--id-grammar) mean, under the v2 defaults. `heading_depth` is a strength over the section-heading-level check of [§FS-declarations.checks.section-heading-level](FS-declarations.md#checkssection-heading-level-section-heading-level-mismatch): `must` is an error, `warn` a warning, `should` a suggestion, and `may` recognizes sections without checking their depth, which is v1's `loose`.

### schema.places: Rows and their places

A row is `[schema.kinds.<NAME>]`. It admits `files` and `folders`, lists of paths from the config root, which are its places; `citable`, default `true`; `scan`, default `true`; `index`; `id_format`; `fetch`; and `form`. A citable row with no place is declared in doc-comments only. These keys mean what the v1 row keys of [§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds) mean, written as lists and under v2 names: `id_format` is v1's row `format` and `fetch` is v1's `fetch`.

### schema.sources: `[schema.sources]`

`[schema.sources]` admits `languages`, `exclude` and `respect_gitignore`, plus the tables `[schema.sources.extensions]` and `[schema.sources.definitions.<name>]`. `exclude` holds gitignore-style globs.

### schema.measures: Measures are tables, strengths are keys

A measured constraint is written as a table named for its measure, whose keys are strengths and whose values are thresholds:

| Table | Measures | Strength keys | v1 counterpart |
|---|---|---|---|
| `[schema.notes.lines]` | an inline note's lines | `must`, `warn`, `should`, `may` | `inline_max_lines`, `inline_suggested_lines`, `warn_on_suggested` |
| `[schema.notes.columns]` | an inline note's columns | `must`, `warn`, `should`, `may` | `inline_max_columns` |
| `[schema.notes.text]` | whether a note carries text beside its citation | `must` | `inline_style` |
| `[schema.notes.layout]` | an inline note's layout | `must`, `warn`, `should`, `may` | `inline_note_layout`, `inline_note_layout_check` |
| `[schema.leads.words]` | a declaration lead's words | `must`, `warn`, `should`, `may` | `lead_size_warning` |

A stronger strength takes the looser threshold. `must = 1` beside `should = 3` is refused, at the stronger key: ``threshold ordering: `must` (1) is stricter than `should` (3); a stronger strength takes the looser threshold``. The note measures are those of [§FS-inline-citation-style.2](FS-inline-citation-style.md#2-configuration) and the lead measure is [§FS-declarations.checks.oversized-lead](FS-declarations.md#checksoversized-lead-oversized-lead-opt-in)'s. Only the channel moves with the strength.

### schema.fields: Fields and forms

`[schema.kinds.<NAME>.fields.<field>]` and a row's `form`, `closed` and `one_of` are part of the format, specified with agent-grounds/grund#458. They are subject to [§FS-config-v2.rollout](FS-config-v2.md#rollout-what-this-grund-executes-and-what-it-refuses).

## rules: The rules concern

### rules.strengths: One strength vocabulary, two severities

Every constraint takes one strength, and the strength names its channel:

| Strength | Prohibition | Channel |
|---|---|---|
| `must` | `must-not` | error, exit `1` |
| `warn` | `warn-not` | standing warning, exit `0` |
| `should` | `should-not` | suggestion, only under `--suggestions` |
| `may` | none | unchecked |

`warn` is not a synonym for `should`. The two reach different channels, and v1 already has both ([§FS-config.6.1](FS-config.md#61-suggestions-are-not-a-third-severity)). `warn` adds no third severity: the severity set stays `{error, warning}`, and suggestions keep their own channel ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)). A finding keeps its finding code and its message whatever its strength, so promoting a constraint from `warn` to `must` changes only the channel it is reported on. A prohibition exists only where the constraint is about a relation that can be forbidden, which is citations. A measure and a ladder take positive strengths only.

### rules.citations: `[rules.citations]`

`[rules.citations.<KIND>]` admits the list keys `must`, `warn`, `should`, `may`, `must-not`, `warn-not` and `should-not`, and the scalar `default`. Each list entry is a target or an any-of `A|B`, with v1's alias forms ([§FS-config.3.9.3](FS-config.md#393-alias-matching)). An obligation is asked of each declaration of the citing kind, and a prohibition fires at each citation site ([§FS-config.3.9.1.1](FS-config.md#3911-obligations-and-prohibitions)). Overlap between two lists is refused as in [§FS-config.3.9.5.1](FS-config.md#3951-overlap-not-textual-equality), and v2 locates the refusal at the later list's line. Precedence is v1's: an explicit target, then the kind's `default`, then the project's `[rules.citations] default` ([§FS-config.3.9.4](FS-config.md#394-defaults-and-precedence)).

#### rules.citations.default: A default may forbid, never oblige

`default`, at the project or the kind, takes `may`, `should-not`, `warn-not` or `must-not`. A positive default is refused at its line: `` `default = "must"` is not a v2 citation default (expected may, should-not, warn-not, or must-not) ``. In v1 a positive `default` loads and creates no obligation ([§FS-config-v2.mapping](FS-config-v2.md#mapping-what-the-records-keep-apart)). v2 does not spell that state, so it refuses the form rather than give it a second meaning.

### rules.grounding: Grounding is a ladder, replaced whole

`[rules.citations.grounding]` is the project's ladder and `[rules.citations.<PLACE>.grounding]` a row's. A ladder's keys are the positive strengths and its values are units: `"file"`, or `"h2"` to `"h6"`, the heading levels of [§FS-check.3.6.2](FS-check.md#362-the-unit). Each rung asks that every unit of that size in a governed file be grounded, and reports an ungrounded one on its strength's channel ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)). A row's files are governed as [§FS-check.3.6.1](FS-check.md#361-which-files-a-row-governs) says. A stronger rung never uses a finer unit: `must = "h2"` beside `warn = "file"` is refused at the stronger key: ``grounding ladder: `must` (h2) is finer than `warn` (file); a stronger rung never uses a finer unit``.

A ladder is one setting. A row that writes a ladder replaces the project's ladder whole, and a rung the row leaves out is not inherited. This is [§FS-config.principle.unit](FS-config.md#principleunit-one-leaf-key-overrides-and-one-value-is-whole)'s whole-value rule, applied to a setting v1 spelled as two independent keys. `grund check --require-grounding` supplies the project ladder `must = "file"`, and an explicit row ladder still wins whole ([§FS-config.principle.cli](FS-config.md#principlecli-a-cli-input-enters-at-the-scope-its-flag-spells)).

### rules.resolution: `[rules.resolution]`

`[rules.resolution]` maps a kind to `must` or `warn`: how a citation of an external kind that has no snapshot is reported ([§FS-check.4.12](FS-check.md#412-missing-snapshot)). `warn` is the missing-snapshot warning, and `must` is the error. A kind with `fetch` defaults to `must`, as in v1. An explicit entry for a kind whose row has no `fetch` is refused at its line: `` `[rules.resolution] <KIND>` needs `fetch` on [schema.kinds.<KIND>] ``. `should` and `may` are refused, because a missing snapshot is never a suggestion. Resolution never fetches ([§REQ-runs-offline.1](../requirements/REQ-runs-offline.md#1-read-and-verification-paths-execute-nothing)).

## presentation: The presentation concern

`[presentation]` admits `description`, `trigger`, `conversation` and `rules`. `[presentation.kinds.<NAME>]` admits `title`, and `[presentation.fmt]` admits `anchors`, `links` and `exclude`. `description`, `trigger`, `conversation` and a kind's `title` mean what `project_description`, `[reference] trigger`, `[reference] conversation` and a row's `title` mean in v1 ([§FS-config.3](FS-config.md#3-keys)). `[presentation.fmt] exclude` is v1's `[fmt] exclude` ([§FS-config.3.10](FS-config.md#310-fmt--suppressing-the-rewrite)), and `anchors = "github"` with `links = "index"` is the formatter's existing effect. Every other value of `anchors` and `links`, and `rules`, are subject to [§FS-config-v2.rollout](FS-config-v2.md#rollout-what-this-grund-executes-and-what-it-refuses).

## rollout: What this grund executes, and what it refuses

The format is published whole, and this binary executes part of it. A clause the format defines but this binary does not execute is never accepted and ignored. It is refused at its authored line with `` `<clause>` is part of the v2 format but not supported by this grund yet ``, where `<clause>` is a table's header, `[schema.kinds.FS.fields.terms]`, or a table and key, `[presentation.fmt] anchors`. An omitted default that cannot be executed is refused at the version key ([§FS-config-v2.defaults.2](FS-config-v2.md#defaults2-the-language-set-is-fixed-before-it-is-executed)). The refusal lifts when the feature's owner lands, and it never falls back to a v1 meaning.

| Clause | Status in this grund | Owner |
|---|---|---|
| Envelope, `[workspace]` | executed | #455 |
| `[schema]` keys, `heading_depth` | executed | #455 |
| `[schema.notes.*]`, `[schema.leads.words]` | executed | #455 |
| A row with at most one place, one entry in `files` or `folders`; `citable`, `scan`, `index`, `id_format`, `fetch` | executed | #455 |
| `[rules.citations]`, ladders, `[rules.resolution]` | executed | #455 |
| `[presentation]` `description`, `trigger`, `conversation`, `[presentation.kinds.*] title`, `[presentation.fmt] exclude` | executed | #455 |
| `[presentation.fmt]` `anchors = "github"`, `links = "index"` | executed | #455 |
| `languages = ["markdown"]` | executed | #455 |
| Any other `languages` value, an omitted `languages`, `[schema.sources.extensions]`, `[schema.sources.definitions.*]`, `[schema.sources] exclude` | refused | #456 |
| A row with more than one place, a place inside another row's place, a `code` row with places | refused | #457 |
| `[schema.kinds.*.fields.*]`, `form`, `closed`, `one_of` | refused | #458 |
| Any other `anchors` or `links` value | refused | #459 |
| `[presentation] rules` | refused | #460 |

The published example of [§DISC-core-concerns.7.1](../discussions/proposals/2026-09-30-core-concerns.md#71-the-canonical-example) parses once each owner has landed, and every refusal above is pinned by a test that turns into an executed case on that day.

## mapping: What the records keep apart

Both readers lower into the same records, so the records hold every state either one can mean. A v1 state that v2 does not spell keeps its meaning. Migration is agent-grounds/grund#461's. This table is what a migration audits against, and tests on the records hold it, not a comparison of one tree's reports.

| v1 state | v2 spelling | What the records keep |
|---|---|---|
| A positive `[citations] default` creates no obligation | `default` takes only `may`, `should-not`, `warn-not`, `must-not` | v1's positive default, still non-obligating |
| `strict = false` recognizes bare citations | none: citations are marked | the lexical policy, apart from the marker |
| `named_sections = false` | none: always enabled | the section-recognition policy |
| A digit-led slug pattern | letter-led default; an explicit `slug_pattern` | the compiled, authored pattern |
| A numbered ID format | `{kind}-{slug}` default; an explicit `id_format` | the compiled project and row format |
| `section_heading_levels = "loose"` | `heading_depth = "may"` | sections recognized, depth unchecked |
| Grounding keys inherited one by one | a narrower ladder replaces the wider one whole | the effective pair v1 resolves, per row |
| `resolve = "should"` warns | `[rules.resolution] <KIND> = "warn"` | the missing-snapshot warning |
| A note budget, silent or opted into a warning | `may` or `warn` on the measure | the guidance threshold, and the warning threshold |
