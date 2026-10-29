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
- **Place**: one entry of a row's `files` or `folders`, a path from the config root ([§FS-config-v2.schema.places](FS-config-v2.md#schemaplaces-rows-and-their-places)).
- **Complement**: the row named `code`, which owns every scanned file no other place claims ([§FS-config-v2.schema.places.complement](FS-config-v2.md#schemaplacescomplement-code-is-the-complement-by-name)).
- **Owner**: the one row a scanned file belongs to: a listed file's row, else the row of the deepest folder containing it, else the complement ([§FS-config-v2.schema.places.ownership](FS-config-v2.md#schemaplacesownership-a-listed-file-then-the-deepest-folder)).
- **Rollout refusal**: a located load error for a v2 clause the format defines but this binary does not yet execute ([§FS-config-v2.rollout](FS-config-v2.md#rollout-what-this-grund-executes-and-what-it-refuses)).

## requirements: What every version-2 config holds to

Three requirements on the format as a whole, each carrying a mark of [§FS-config.requirements](FS-config.md#requirements-what-the-config-contract-holds-to). Four more are left out because they are stated elsewhere: *a v1 file keeps every meaning it had, in the records as well as the reports* is [§FS-config.5.2](FS-config.md#52-every-older-version-keeps-its-meaning), under [§REQ-backwards-compatibility.1](../requirements/REQ-backwards-compatibility.md#1-what-is-covered); *a mistake is an error at its line*, of which the closed tables of [§FS-config-v2.reader.1](FS-config-v2.md#reader1-tables-and-keys-are-closed) are v2's instance, is [§FS-config.requirements.5](FS-config.md#requirements5-a-mistake-in-the-config-fails-loudly--realized-one-case-deferred); *a strength adds no third severity* is [§FS-config.6.1](FS-config.md#61-suggestions-are-not-a-third-severity); and *`config show` prints a v2 file in v2 spelling that loads back to the same values* is [§FS-config.requirements.6](FS-config.md#requirements6-the-effective-configuration-is-inspectable--realized-provenance-deferred).

### requirements.no-v1-meaning: A v2 file is read under v2's rules or refused, never under v1's — realized

Nothing of v1 is substituted into a file that writes `grund_config_version = 2`. v1's tables and discarded aliases are unknown sections there ([§FS-config-v2.reader.1](FS-config-v2.md#reader1-tables-and-keys-are-closed)); a default the file omits is the v2 epoch's, and one this grund cannot execute yet is refused at the version key rather than replaced, so the v1 extension list never stands in for the epoch language set ([§FS-config-v2.defaults.1](FS-config-v2.md#defaults1-what-v2-fixes), [§FS-config-v2.defaults.2](FS-config-v2.md#defaults2-the-language-set-is-fixed-before-it-is-executed)); a positive citation `default`, which v1 loads and which obliges nothing, is refused rather than given that meaning or another ([§FS-config-v2.rules.citations.default](FS-config-v2.md#rulescitationsdefault-a-default-may-forbid-never-oblige)); and a clause the format defines but this grund does not execute is refused at its line and never falls back to a v1 reading ([§FS-config-v2.rollout](FS-config-v2.md#rollout-what-this-grund-executes-and-what-it-refuses)). A v1 state v2 does not spell stays reachable through a v1 file alone ([§FS-config-v2.mapping](FS-config-v2.md#mapping-what-the-records-keep-apart)). Serves [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path): the version a file writes is the meaning it gets.

### requirements.strengths: Each constraint takes the strengths it can mean from one vocabulary, and refuses the rest — realized, directional for the next constraint

Every constraint draws its strength from one closed set, and the strength moves only the channel, never the finding's code or message ([§FS-config-v2.rules.strengths](FS-config-v2.md#rulesstrengths-one-strength-vocabulary-two-severities), [§FS-config-v2.schema.measures](FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys), [§FS-fields.strengths.table](FS-fields.md#strengthstable-which-strength-each-finding-takes)). Each admits the strengths it can mean and refuses the others at their line rather than mapping one onto its neighbour: a prohibition only where a citation can be forbidden, and positive strengths alone on a measure, a ladder and `heading_depth` ([§FS-config-v2.rules.strengths](FS-config-v2.md#rulesstrengths-one-strength-vocabulary-two-severities), [§FS-config-v2.schema.1](FS-config-v2.md#schema1-schema)); `must` alone on whether a note carries text ([§FS-config-v2.schema.measures](FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys)); `may` or a prohibition alone as a citation `default` ([§FS-config-v2.rules.citations.default](FS-config-v2.md#rulescitationsdefault-a-default-may-forbid-never-oblige)); and `must` or `warn` alone for resolution, since a missing snapshot is never a suggestion ([§FS-config-v2.rules.resolution](FS-config-v2.md#rulesresolution-rulesresolution)). A constraint added from here on takes its strengths from this set, says which it admits, and refuses the others at their line. Serves [§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable), whose repository adopts a constraint as a warning and promotes it to an error without changing format.

### requirements.ahead: What the format defines and this grund refuses is owed to its owners — deferred

The format is published whole and executed in part ([§FS-config-v2.rollout](FS-config-v2.md#rollout-what-this-grund-executes-and-what-it-refuses)). Owed, and refused at its authored line until it lands: `[presentation] rules` to #460; and every `anchors` and `links` value but the formatter's existing one to its own change ([§FS-config-v2.presentation](FS-config-v2.md#presentation-the-presentation-concern)). Migration from v1, which the records are kept apart for, is realized by [§FS-config-migrate](FS-config-migrate.md#fs-config-migrate-grund-config-migrate---to-2-shows-the-version-2-file-that-means-what-a-version-1-config-means-and-writes-it-only-when-asked-and-only-when-it-does). None of them loads with a v1 meaning meanwhile ([§FS-config-v2.requirements.no-v1-meaning](FS-config-v2.md#requirementsno-v1-meaning-a-v2-file-is-read-under-v2s-rules-or-refused-never-under-v1s--realized)). Serves [§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable).

## reader: The reader

### reader.1: Tables and keys are closed

The v2 reader admits exactly the tables and keys this file names, at the scope each is named at. An unknown table is ``unknown config section `<table>` ``. An unknown key is ``unknown key `<key>` in [<table>]``, or ``unknown key `<key>` at the top level`` in the envelope. A key written at a table that does not admit it is refused the same way. A key repeated inside one table is ``duplicate key `<key>` in [<table>] (first written at line <n>)``. Every refusal is located at the offending line, per [§FS-config.4.3](FS-config.md#43-invalid-config-behavior). An unknown key and a v1 table then say what to write instead, appended after that text, which stays whole ([§FS-errors.3](FS-errors.md#3-message-text)): an unknown key names the keys its table admits ([§FS-config-v2.reader.1.keys](FS-config-v2.md#reader1keys-an-unknown-key-names-the-keys-its-table-admits)), and a v1 table names the v2 tables its settings are written in ([§FS-config-v2.reader.1.v1-tables](FS-config-v2.md#reader1v1-tables-a-v1-table-names-the-v2-tables-its-settings-are-written-in)). What is appended admits nothing, and the v1 reader, whose unknown section is the same text, appends none ([§FS-config.5.2](FS-config.md#52-every-older-version-keeps-its-meaning)).

v1's spellings are not v2 keys. `[reference]`, `[id]`, `[[kinds]]`, `[scan]`, `[output]`, `[citations]` and `[fmt.cross_refs]` are unknown sections in v2, and so are `strict`, `named_sections`, `include`, and every other discarded alias ([§DISC-core-concerns.7.4](../discussions/proposals/2026-09-30-core-concerns.md#74-what-the-format-leaves-out)). v2 reads no `include`: the scan is the union of the rows' places and the complement's extent ([§FS-config-v2.schema.places](FS-config-v2.md#schemaplaces-rows-and-their-places)). Report preferences are run flags ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)), so `[output]` has no v2 spelling either.

#### reader.1.keys: An unknown key names the keys its table admits

An unknown key's refusal appends, after one space, the keys its table admits in the reader's `(expected …)` shape, in the order the point that specifies the table lists them: ``unknown key `max` in [schema.leads.words] (expected must, warn, should, or may)``. Two keys are joined by `or`, and three or more by commas with `, or` before the last, as the reader's other `(expected …)` lists are. The key is still refused at its line, and the list is the table's admitted keys and nothing else, so every key it names is one that table reads.

- **The envelope** names its two keys in [§FS-config.concerns](FS-config.md#concerns-every-key-belongs-to-exactly-one-concern)' order: ``unknown key `<key>` at the top level (expected grund_config_version or project_name)``.
- **A ladder**, `[rules.citations.grounding]` or `[rules.citations.<PLACE>.grounding]`, names the positive strengths in [§FS-config-v2.rules.strengths](FS-config-v2.md#rulesstrengths-one-strength-vocabulary-two-severities)' order. `[rules.citations]` names its one key, `default` ([§FS-config-v2.rules.citations.default](FS-config-v2.md#rulescitationsdefault-a-default-may-forbid-never-oblige)), and `[workspace]` names `members`, `optional_members` and `include_root`, in [§FS-config.3.8](FS-config.md#38-workspace--sub-project-members)'s order.
- **A table that holds only tables**, `[schema.notes]`, `[schema.leads]`, `[schema.kinds]`, `[schema.kinds.<NAME>.fields]`, `[schema.kinds.<NAME>.fields.front-matter.keys]`, `[schema.sources.definitions]`, `[rules]` or `[presentation.kinds]` written as a header of its own ([§FS-config-v2.reader.2](FS-config-v2.md#reader2-every-row-and-field-is-written-under-its-own-header)), appends `(it holds only tables)`.
- **A table whose keys are names** says what a key names: `[schema.sources.extensions]` appends `(expected a suffix)` ([§FS-config-v2.schema.sources.extensions](FS-config-v2.md#schemasourcesextensions-schemasourcesextensions-maps-a-suffix-to-a-language)), and `[rules.resolution]` appends `(expected a kind name)` ([§FS-config-v2.rules.resolution](FS-config-v2.md#rulesresolution-rulesresolution)). A key either table reads as a name keeps that table's own refusals.
- **A v1 key at a v2 table** is an unknown key like any other. It names the table's v2 keys, not the v1 key's own v2 spelling, which is [§FS-config-migrate.mapping](FS-config-migrate.md#mapping-every-version-1-key-and-default)'s: ``unknown key `folder` in [schema.kinds.FS] (expected files, folders, citable, scan, index, id_format, fetch, form, or closed)``.
- **A `front-matter` field and its keys** name their own: ``unknown key `after` in [schema.kinds.ADR.fields.front-matter] (expected presence or closed)`` and ``unknown key `placement` in [schema.kinds.ADR.fields.front-matter.keys.status] (expected presence or one_of)`` ([§FS-fields.front-matter.config](FS-fields.md#front-matterconfig-declaring-the-field-and-its-keys)).

#### reader.1.v1-tables: A v1 table names the v2 tables its settings are written in

A v1 table is refused at its header, before any key under it is read, and the refusal stays at that line ([§FS-config-v2.reader.4](FS-config-v2.md#reader4-order-and-location-are-kept)). So it names one set rather than one key's home: every v2 table that [§FS-config-migrate.mapping](FS-config-migrate.md#mapping-every-version-1-key-and-default) writes a key of that v1 table into. It appends `(a v1 table; in v2: <tables>)`, the tables joined by commas in this order:

| v1 header | What its refusal appends |
|---|---|
| `[reference]` | `(a v1 table; in v2: [schema], [schema.notes.*], [schema.leads.words], [rules.citations.grounding], [presentation])` |
| `[id]` | `(a v1 table; in v2: [schema])` |
| `[scan]` | `(a v1 table; in v2: [schema.sources], [schema.sources.extensions], [schema.sources.definitions.*], [schema.kinds.code])` |
| `[citations]` | `(a v1 table; in v2: [rules.citations])` |
| `[citations.<KIND>]` | `(a v1 table; in v2: [rules.citations.<KIND>])`, with the header's own `<KIND>` |
| `[fmt]`, `[fmt.cross_refs]` | `(a v1 table; in v2: [presentation.fmt])` |
| `[output]` | `(a v1 table; in v2: run flags, such as --format and --path-base)` |
| `[[kinds]]` | `(a v1 table; in v2: [schema.kinds.<NAME>])`, after the `(v2 writes no arrays of tables)` every array of tables carries |

`[schema.notes.*]` and `[schema.sources.definitions.*]` stand for the tables under them. `[output]` names no table because v2 has none for it: its keys are run flags ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)). `[[kinds]]` names the row each entry becomes ([§FS-config-v2.reader.2](FS-config-v2.md#reader2-every-row-and-field-is-written-under-its-own-header)) and leaves a row's `title`, `resolve` and grounding to [§FS-config-migrate.mapping](FS-config-migrate.md#mapping-every-version-1-key-and-default), which one concise line has no room for. A key v2 spells nowhere, such as `[reference] strict` or `[id] named_sections`, adds no table. Every other unknown table, such as `[schema.leads.lines]`, `[project]` or another array of tables, keeps the refusal of [§FS-config-v2.reader.1](FS-config-v2.md#reader1-tables-and-keys-are-closed) with nothing appended: it is not a v1 spelling, so there is no home to name.

### reader.2: Every row and field is written under its own header

A keyed row is written under its own explicit header, `[schema.kinds.<NAME>]`, and a field under `[schema.kinds.<NAME>.fields.<field>]`, so every key names its owner. An explicit header that is written twice is ``repeated table `[<table>]` (first written at line <n>)``, located at the second. A parent that TOML creates implicitly, as `[schema]` is created by writing `[schema.sources]`, is not an explicit header: writing `[schema]` after it is not a repetition.

### reader.3: One setting, one form

A setting has one form. Where the format spells a constraint as a table whose keys are strengths, a strength key is never also a table: `[rules.citations.FS.must]` beside `[rules.citations.FS] must = [...]`, or `[schema.notes.lines.must]`, is `` `must` is a strength key in [<table>], not a table ``. A value of the wrong TOML type is refused at its line with the type it expected.

### reader.4: Order and location are kept

The reader keeps the line every value was written at, so the one validation pass after reading anchors an error where the value was written ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)). It keeps the authored order of the explicit headers too: rows are listed, rendered and reported in the order the file writes them, as `[[kinds]]` rows are in v1.

### reader.5: An array may span lines

An array value may be written across lines; a string in it may not ([§FS-config-v2.reader.5.strings](FS-config-v2.md#reader5strings-a-string-closes-on-the-line-it-opens-on)). It opens on its key's line, and every line after it, with its `#` comment taken off, belongs to it until its `[` closes outside a string, so no line inside it is read as a header or a key. It is one value, read and located at its key's line ([§FS-config-v2.reader.4](FS-config-v2.md#reader4-order-and-location-are-kept)), and it may end in the trailing comma TOML allows before its `]`, which [§FS-config-v2.reader.5.commas](FS-config-v2.md#reader5commas-an-array-takes-one-trailing-comma-after-an-element) defines. Its elements are then read as a one-line list's are, each ending only at a comma outside a string ([§FS-config.3.11](FS-config.md#311-a-list-element-ends-at-a-comma-outside-a-string)). One still open at the end of the file is refused at its key's line as `unclosed array`, and where a line of it ended inside a string the refusal appends a parenthetical naming the first such line, ` (a string is still open at the end of line <n>)`, or the one [§FS-config-v2.reader.5.strings.multi-line](FS-config-v2.md#reader5stringsmulti-line-a-string-opened-with--is-one-version-2-does-not-read) gives a string that opened with `"""`.

#### reader.5.strings: A string closes on the line it opens on

A string in an array closes on the line it opens on, as a TOML basic string does, so a line break is never part of a value. A line of an array that ends inside a string is the array's *open line*, and the first such line is the one every message below names as `<n>`. A string is open at a line's end when the `"` that opened it has no closing `"` after it on that line; a `"` is a quote unless it is part of a `\"` escape, by the one rule [§FS-config.3.11](FS-config.md#311-a-list-element-ends-at-a-comma-outside-a-string) reads strings by, so `"a\"` at a line's end is still open. Each line still has its `#` comment taken off on its own ([§FS-config-v2.reader.5](FS-config-v2.md#reader5-an-array-may-span-lines)); once no string crosses a line, the string boundaries of the joined array are those of each line, so a `#` taken for a comment is never one inside a string.

The array is read and refused at its key's line ([§FS-config-v2.reader.4](FS-config-v2.md#reader4-order-and-location-are-kept)), as every array is, and the rule moves the verdict on files the 0.18.0 to 0.20.x readers loaded, so it arrives through a deprecation window ([§FS-config-v2.reader.5.strings.window](FS-config-v2.md#reader5stringswindow-through-grund-021x-an-array-a-later-line-closes-is-read-as-before-and-warns)) before the refusal ([§FS-config-v2.reader.5.strings.refusal](FS-config-v2.md#reader5stringsrefusal-from-grund-0220-the-open-line-ends-the-array)), the route [§DF-v2-string-closes-on-its-line](../decisions/functional/DF-v2-string-closes-on-its-line.md#df-v2-string-closes-on-its-line-a-string-in-a-version-2-array-closes-on-its-own-line-warned-for-in-0210-and-refused-from-0220) records. Version 1 reads no array across lines ([§FS-config.3](FS-config.md#3-keys-version-1)) and is untouched.

##### reader.5.strings.window: Through grund 0.21.x an array a later line closes is read as before and warns

Through grund 0.21.x an array that has an open line and that a later line closes is still read as the readers before it read it: the lines are joined with the break read as one space, so `exclude = [` / `"a` / `b",` / `]` is the list `["a b"]` and `"a,` / `b",` the list `["a, b"]`. The config loads, and the run earns one warning at the array's key's line, naming the open line and the release that refuses it:

```text
warning: grund.toml:7: unclosed array (a string is still open at the end of line 8), read for now with the line break as a space; close the string on line 8; this warning becomes an error in grund 0.22.0
```

Its code is `config-not-toml` ([§FS-config-v2.checks.config-not-toml](FS-config-v2.md#checksconfig-not-toml-a-version-2-config-holds-text-toml-refuses)), or `config-multiline-string` where the string opened with `"""` ([§FS-config-v2.reader.5.strings.multi-line](FS-config-v2.md#reader5stringsmulti-line-a-string-opened-with--is-one-version-2-does-not-read)). One array earns one warning, however many of its lines end inside a string. An array that no later line closes earns no warning: it is refused, now as before, as `unclosed array` with the parenthetical ([§FS-config-v2.reader.5](FS-config-v2.md#reader5-an-array-may-span-lines)).

##### reader.5.strings.refusal: From grund 0.22.0 the open line ends the array

From grund 0.22.0 no line after an open line is joined to the array: the open line is the array's last, the array is still open there, and it is refused at its key's line as `unclosed array` with the parenthetical naming the open line, exit `2` on every command that loads the config ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)). Where the readers before 0.22.0 would have closed the array on a later line, so that the file loaded through the window, the refusal ends with the release that made it one ([§FS-distribution.4.2](FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)):

```text
error: grund.toml:7: unclosed array (a string is still open at the end of line 8); this became an error in grund 0.22.0
```

Where they would not have closed it either, the array was refused before and the refusal claims no release: `exclude = ["a,b]` as the last line of the file is `error: grund.toml:7: unclosed array (a string is still open at the end of line 7)`, in 0.21.x and from 0.22.0 alike. The ancestor climb reads the array through the same join, so a `[workspace] members` value with an open line is not a list exactly where the reader refuses it, and another key's array with an open line is passed over as one still open at the end of the file is ([§FS-workspace.6.1.7.5](FS-workspace.md#6175-an-unobtainable-members-value-leaves-the-claim-undecidable)).

##### reader.5.strings.multi-line: A string opened with `"""` is one version 2 does not read

TOML admits a multi-line basic string, opened and closed by `"""`, and version 2 does not read one: a file that writes one is TOML, but not a `grund.toml` version 2 reads. So where the string open at the end of the open line opened with `"""`, every message above says that rather than that a string is unclosed. The parenthetical is ` (a multi-line string opens on line <n>, which version 2 does not read)`, the warning's code is `config-multiline-string` ([§FS-config-v2.checks.config-multiline-string](FS-config-v2.md#checksconfig-multiline-string-a-version-2-config-value-holds-a-multi-line-string)), and its remedy is to write the string on that line as one basic string:

```text
warning: grund.toml:7: unclosed array (a multi-line string opens on line 8, which version 2 does not read), read for now with the line break as a space; write the string on line 8 as one basic string; this warning becomes an error in grund 0.22.0
```

The window and the refusal are the ones above: through 0.21.x `exclude = [` / `"""a` / `b""",` / `]` loads as it always has, as the one element `""a b""`, its quotes kept, and from 0.22.0 it is refused. Reading multi-line strings is not part of version 2.

#### reader.5.commas: An array takes one trailing comma, after an element

An array, on one line or written across lines, takes one trailing comma before its `]`, and only after an element. A comma before the `]` comes off before the elements are read only where the text before it, whitespace skipped, is neither `[` nor `,`: that is the trailing comma TOML allows. Any other is a *comma with no element before it*, one whose text before it, whitespace skipped, ends in `[` or `,`. It stays where it is written, so the element reader meets an empty element there and refuses it as `expected string` at the key's line, as it refuses an empty element anywhere in a list ([§FS-config.3.11](FS-config.md#311-a-list-element-ends-at-a-comma-outside-a-string)). The join has already taken each line's `#` comment off ([§FS-config-v2.reader.5](FS-config-v2.md#reader5-an-array-may-span-lines)), so a comment between two commas is read as whitespace is. `["ab",]`, `["ab" , ]` and `[` / `"ab"` / `,` / `]` hold the one element `ab`, and `[]`, `[ ]` and `[` / `]` hold none; `["ab",,]`, `[,]`, `[,"a"]` and `["a",,"b"]` each hold a comma with no element before it. TOML refuses each of the last four, and so does every other tool that reads a `grund.toml` as TOML.

The 0.18.0 to 0.20.x readers took off the whole run of adjacent commas before the `]`, so where that run held a comma with no element before it, the array loaded: `["ab",,]`, `["ab" ,, ]`, `["ab",,,]` and an array whose last line is `"ab",,` as `["ab"]`, and `[,]`, `[ , ]`, `[,,]`, `[` / `,` / `]` and `[ # c` / `,` / `]` as `[]`. The rule moves that verdict, so it arrives through a deprecation window ([§FS-config-v2.reader.5.commas.window](FS-config-v2.md#reader5commaswindow-through-grund-021x-the-run-of-commas-before-the--is-taken-off-as-before-and-warns)) before the refusal ([§FS-config-v2.reader.5.commas.refusal](FS-config-v2.md#reader5commasrefusal-from-grund-0220-a-comma-with-no-element-before-it-is-expected-string)), the route [§DF-v2-array-one-trailing-comma](../decisions/functional/DF-v2-array-one-trailing-comma.md#df-v2-array-one-trailing-comma-a-version-2-array-takes-one-trailing-comma-after-an-element-warned-for-in-0210-and-refused-from-0220) records. Where whitespace stood between two commas, or a comma with no element before it stood before an element, they met an empty element and refused it, and that is unchanged: `[,"a"]`, `["a",,"b"]`, `["ab", ,]`, `[ , , ]` and `[` / `"ab",` / `,` / `]` are `expected string` at the key's line, in every release. Version 1 takes no trailing comma at all ([§FS-config.3.11](FS-config.md#311-a-list-element-ends-at-a-comma-outside-a-string)) and is untouched.

##### reader.5.commas.window: Through grund 0.21.x the run of commas before the `]` is taken off as before and warns

Through grund 0.21.x the whole run of adjacent commas before an array's `]` is still taken off, as the readers before it took it off, so `exclude = ["ab",,]` is the list `["ab"]` and `exclude = [,]` the empty list. Where that run held a comma with no element before it and the array then loads, the config loads, and the run earns one warning with code `config-not-toml` ([§FS-config-v2.checks.config-not-toml](FS-config-v2.md#checksconfig-not-toml-a-version-2-config-holds-text-toml-refuses)) at the key's line, naming the line of the first comma with no element before it and the release that refuses it:

```text
warning: grund.toml:9: expected string, read for now with every comma before the `]` taken off; delete each comma on line 9 that has no element before it; this warning becomes an error in grund 0.22.0
```

The line the remedy names is the one that comma stands on. That is the key's line except in an array written across lines, where the warning is still located at the key's line, as every value is ([§FS-config-v2.reader.4](FS-config-v2.md#reader4-order-and-location-are-kept)): `exclude = [` / `"ab",,` / `]` with its key on line 9 names line 10. An array earns one warning however many such commas it holds. An array whose run of commas left an empty element behind, so that the element reader refuses it, earns no warning: it is refused, now as before, as the bare `expected string`. The ancestor climb reads `[workspace] members` through the same join and records nothing: through 0.21.x it reads what the full load reads, so `members = ["a",,]` is the one member `a` ([§FS-workspace.6.1.7.5](FS-workspace.md#6175-an-unobtainable-members-value-leaves-the-claim-undecidable)).

An array that also has an open line ([§FS-config-v2.reader.5.strings.window](FS-config-v2.md#reader5stringswindow-through-grund-021x-an-array-a-later-line-closes-is-read-as-before-and-warns)), or a string in it that closes early ([§FS-config-v2.reader.6.window](FS-config-v2.md#reader6window-through-grund-021x-a-string-that-closes-early-is-read-as-before-and-warns)), earns each construct's warning, one per construct ([§FS-config-v2.checks.config-not-toml](FS-config-v2.md#checksconfig-not-toml-a-version-2-config-holds-text-toml-refuses)). Where a report keeps the config's order, as `config validate` and `config show` do, the open line's comes first, then this one, then the early close's, because the array's lines are joined, then its run of commas taken off, before its elements are read; `check` orders them as it orders every finding ([§FS-errors.4.1](FS-errors.md#41-ordering)), so there the early close's comes first, then this one, then the open line's. So `exclude = [` / `"a` / `b",,` / `]` prints the open line's warning before this one in `config validate` and `config show`, and this one first in `check`; `exclude = ["a" "b",,]` prints this one before the early close's in `config validate` and `config show`, and the early close's first in `check`.

##### reader.5.commas.refusal: From grund 0.22.0 a comma with no element before it is `expected string`

From grund 0.22.0 a comma with no element before it stays where it is written, and the array is refused at its key's line as `expected string`, exit `2` on every command that loads the config ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)). Where the array loaded through the window, the refusal ends with the release that made it one ([§FS-distribution.4.2](FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)):

```text
error: grund.toml:9: expected string; this became an error in grund 0.22.0
```

An array refused before keeps the bare message, in 0.21.x and from 0.22.0 alike: `exclude = ["ab", ,]` is `error: grund.toml:9: expected string`. From then on the ancestor climb reads such a `[workspace] members` value as not a list, as it reads any value the full load refuses ([§FS-workspace.6.1.7.5](FS-workspace.md#6175-an-unobtainable-members-value-leaves-the-claim-undecidable)).

### reader.6: A string ends at its first unescaped quote

A basic string the version-2 reader reads, a scalar value, an element of an array or a quoted key, opens at its `"` and closes at the first `"` after it that is not part of a `\"` escape, by the one rule [§FS-config.3.11](FS-config.md#311-a-list-element-ends-at-a-comma-outside-a-string) reads strings by. After its closing quote a scalar or a key holds nothing but whitespace, its `#` comment already taken off, and an element holds nothing but whitespace before the `,` that separates it from the next or the `]` that closes the array. A string *closes early* when anything else follows its closing quote: `fetch = "one" "two"`, `folders = ["docs" "specs"]` with its comma forgotten, `"a"b"` with a quote left unescaped, or the key `"md" "x"`. TOML refuses each of them, and so does every other tool that reads a `grund.toml` as TOML.

The 0.18.0 to 0.20.x readers checked a string only at the first and the last byte of its value or its element, so each of those loaded, as the one string `one" "two`, the one folder `docs" "specs`, the string `a"b` and the suffix `md" "x`. The rule moves that verdict, so it arrives through a deprecation window ([§FS-config-v2.reader.6.window](FS-config-v2.md#reader6window-through-grund-021x-a-string-that-closes-early-is-read-as-before-and-warns)) before the refusal ([§FS-config-v2.reader.6.refusal](FS-config-v2.md#reader6refusal-from-grund-0220-a-string-that-closes-early-is-expected-string)), the route [§DF-v2-string-closes-at-its-quote](../decisions/functional/DF-v2-string-closes-at-its-quote.md#df-v2-string-closes-at-its-quote-a-version-2-string-ends-at-its-first-unescaped-quote-warned-for-in-0210-and-refused-from-0220-and-version-1-keeps-its-reading) records. A string opened with `"""` is TOML's multi-line string, which version 2 does not read, and the messages say so ([§FS-config-v2.reader.6.multi-line](FS-config-v2.md#reader6multi-line-a--string-on-one-line-is-one-version-2-does-not-read)). Version 1 keeps the reading of the readers before it for good ([§FS-config.5.2](FS-config.md#52-every-older-version-keeps-its-meaning)) and meets the rule only when it is migrated ([§FS-config-migrate.3.1.8](FS-config-migrate.md#318-a-string-that-closes-early)).

#### reader.6.window: Through grund 0.21.x a string that closes early is read as before and warns

Through grund 0.21.x a string that closes early is read as the readers before it read it, from its opening quote to the last quote of its value or its element with every quote between kept, so `fetch = "one" "two"` is the string `one" "two` and `folders = ["docs" "specs"]` the one element `docs" "specs`. The config loads, and the run earns one warning with code `config-not-toml` ([§FS-config-v2.checks.config-not-toml](FS-config-v2.md#checksconfig-not-toml-a-version-2-config-holds-text-toml-refuses)) at the line of the key, naming the string as it is read, the line to edit and the release that refuses it:

```text
warning: grund.toml:5: expected string, read for now as the one string `docs" "specs`; end the string on line 5 at its closing quote: write `\"` for a quote inside it, or a `,` between two strings; this warning becomes an error in grund 0.22.0
```

The text in backticks is the string's text from its opening quote to that last quote, as written, so an escape in it is shown as it is written, and a line break the window of [§FS-config-v2.reader.5.strings.window](FS-config-v2.md#reader5stringswindow-through-grund-021x-an-array-a-later-line-closes-is-read-as-before-and-warns) joins is shown as the space it is read as. The line `on line <n>` names is the one the string closes early on, the line its closing quote stands on. That is the key's line except in an array written across lines, where `<n>` is the line of the element and the warning is still located at the key's line, as every value is ([§FS-config-v2.reader.4](FS-config-v2.md#reader4-order-and-location-are-kept)).

The remedy above is an element's, and both of its halves are a repair: a quote meant inside the string is written `\"`, and two strings meant are separated by a `,`. A scalar's or a key's is the first half alone, because a `,` between two strings is no repair where one string is read: `fetch = "one", "two"` closes early again.

```text
warning: grund.toml:5: expected string, read for now as the one string `one" "two`; end the string on line 5 at its closing quote: write `\"` for a quote inside it; this warning becomes an error in grund 0.22.0
```

A value earns one warning, naming the first string in it that closes early, however many do, as an array earns one warning however many of its lines end inside a string ([§FS-config-v2.reader.5.strings.window](FS-config-v2.md#reader5stringswindow-through-grund-021x-an-array-a-later-line-closes-is-read-as-before-and-warns)); a quoted key is a string of its own and earns its own. An array that also has an open line earns both warnings, one per construct ([§FS-config-v2.checks.config-not-toml](FS-config-v2.md#checksconfig-not-toml-a-version-2-config-holds-text-toml-refuses)). Where a report keeps the config's order, as `config validate` and `config show` do, the one for its open line comes first, because the array's lines are joined before its elements are read; `check` orders the two as it orders every finding ([§FS-errors.4.1](FS-errors.md#41-ordering)), so there the early close comes first.

A value the readers before it refused is refused as before and earns no warning. `exclude = ["target" "a,b"]`, whose second piece they cut at its comma, is still `expected string` at its key's line ([§FS-config.3.11](FS-config.md#311-a-list-element-ends-at-a-comma-outside-a-string)), and in `exclude = ["a"b", "c"]` the line ends inside a string, so [§FS-config-v2.reader.5.strings](FS-config-v2.md#reader5strings-a-string-closes-on-the-line-it-opens-on) reads it before any element is. A string that closes at its end, such as `"a\"b"`, `"a\\"` or `"a" # a comment`, earns nothing.

#### reader.6.refusal: From grund 0.22.0 a string that closes early is `expected string`

From grund 0.22.0 version 2 no longer reads a string the way the readers before it did: one that closes early is refused at its key's line as `expected string`, exit `2` on every command that loads the config ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)). Where the value loaded through the window, the refusal ends with the release that made it one ([§FS-distribution.4.2](FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)):

```text
error: grund.toml:5: expected string; this became an error in grund 0.22.0
```

A value refused before keeps the bare message, in 0.21.x and from 0.22.0 alike: `exclude = ["target" "a,b"]` is `error: grund.toml:5: expected string`. The readers that read one key of a config without loading it, the `project_name` of a config no run loads ([§FS-declarations.checks.misplaced-declaration.unlisted-project](FS-declarations.md#checksmisplaced-declarationunlisted-project-explain-the-enclosing-homes-beneath-an-unlisted-project)) and the ancestor climb's `[workspace] members` ([§FS-workspace.6.1.7.5](FS-workspace.md#6175-an-unobtainable-members-value-leaves-the-claim-undecidable)), record no finding: through 0.21.x they read what the full load reads, and from 0.22.0 they pass over a value the full load refuses, as they pass over any value they cannot read.

#### reader.6.multi-line: A `"""` string on one line is one version 2 does not read

A string that opens with `"""` and closes on its own line, such as `title = """one"""`, is a TOML multi-line basic string, which version 2 does not read ([§FS-config-v2.reader.5.strings.multi-line](FS-config-v2.md#reader5stringsmulti-line-a-string-opened-with--is-one-version-2-does-not-read)). Through grund 0.21.x it is read as before, from its first quote to its last with the quotes between kept, so that title is `""one""`, and it earns one warning with code `config-multiline-string` ([§FS-config-v2.checks.config-multiline-string](FS-config-v2.md#checksconfig-multiline-string-a-version-2-config-value-holds-a-multi-line-string)) at the key's line, with the remedy of a multi-line string written across lines:

```text
warning: grund.toml:8: expected string (a multi-line string, which version 2 does not read), read for now as `""one""`, its quotes kept; write the string on line 8 as one basic string; this warning becomes an error in grund 0.22.0
```

From grund 0.22.0 it is refused with the same parenthetical, `error: grund.toml:8: expected string (a multi-line string, which version 2 does not read); this became an error in grund 0.22.0`. Where the first string in a value that closes early is one opened with `"""`, this is the value's one warning, in place of `config-not-toml`. A `"""` string that a line break runs through is one construct, the one [§FS-config-v2.reader.5.strings.multi-line](FS-config-v2.md#reader5stringsmulti-line-a-string-opened-with--is-one-version-2-does-not-read) already warns for: `exclude = [` / `"""a` / `b""",` / `]` reaches the element reader as `"""a b"""` once its lines are joined, and it earns that warning alone.

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
| `[schema.sources] exclude` | `[]` | `[scan] exclude`, six directory names |
| `[schema.sources] declarations` | `"doc-comments"` | every comment in declaration position |
| The scan | the union of the scanned rows' places and the complement's extent, which is the config root unless `[schema.kinds.code]` lists places | `[scan] include` |
| `[rules.citations] default` | `may` | no default level |
| Languages | the epoch's default rows ([§FS-config-v2.schema.sources.epoch](FS-config-v2.md#schemasourcesepoch-the-v2-language-epoch)) | `[scan] extensions` and `comment_prefixes` |

The marker, `shorthand`, `section_separator` and `number_pattern` keep their v1 defaults. A v2 file that declares no `[schema.kinds]` row, or only rows with `citable = false`, has no kinds and still loads: it declares no IDs, and v1's built-in kind list and v1's refusal of a table with no citable kind belong to v1.

### defaults.2: The language set is fixed before it is executed

The v2 language set, with its suffixes, declaration hosts and explicit opt-in languages, is the one agent-grounds/grund#456's proposal fixed on 2026-10-06, and [§FS-config-v2.schema.sources.epoch](FS-config-v2.md#schemasourcesepoch-the-v2-language-epoch) is its table. A v2 file that omits `[schema.sources] languages` admits that table's default rows. The v1 extension list is never substituted for it, and a later binary never adds a row to the default set.

## schema: The schema concern

### schema.1: `[schema]`

`[schema]` admits `marker`, `shorthand`, `id_format`, `slug_pattern`, `number_pattern`, `section_separator` and `heading_depth`. All but `heading_depth` mean what their v1 counterparts in [§FS-config.3.1](FS-config.md#31-reference--citation-form) and [§FS-config.3.2](FS-config.md#32-id--id-grammar) mean, under the v2 defaults. `heading_depth` is a strength over the section-heading-level check of [§FS-declarations.checks.section-heading-level](FS-declarations.md#checkssection-heading-level-section-heading-level-mismatch): `must` is an error, `warn` a warning, `should` a suggestion, and `may` recognizes sections without checking their depth, which is v1's `loose`.

### schema.places: Rows and their places

A row is `[schema.kinds.<NAME>]`. It admits `files` and `folders`, lists of paths from the config root, which are its places; `citable`, default `true`; `scan`, default `true`; `index`; `id_format`; `fetch`; `form`; and `closed`, with its `fields` tables under it ([§FS-config-v2.schema.fields](FS-config-v2.md#schemafields-fields-and-forms)). These keys mean what the v1 row keys of [§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds) mean, written as lists and under v2 names: `id_format` is v1's row `format` and `fetch` is v1's `fetch`. A row may list any number of places, and both keys on one row: a v1 home is a one-element list. The scan is the union of every scanned row's places and the complement's extent ([§FS-config-v2.schema.places.complement](FS-config-v2.md#schemaplacescomplement-code-is-the-complement-by-name)); v2 keeps no second list of scan roots ([§FS-config-v2.reader.1](FS-config-v2.md#reader1-tables-and-keys-are-closed)). A citable row with no place is declared in doc-comments only and adds nothing to the scan. A row's places have one order, its `files` and then its `folders`, each in written order; every view that lists them, `grund config show` included, lists them in it, so what `show` prints loads back to the same rows.

```toml
grund_config_version = 2
[schema.kinds.FS]
folders = ["docs/spec/core", "docs/spec/plugins"]
[schema.kinds.test]
citable = false
folders = ["tests/e2e", "tests/integration"]
[schema.kinds.page]
citable = false
folders = ["docs"]
files = ["README.md", "AGENTS.md", "crates/grund/README.md"]
[schema.kinds.code]
folders = ["crates"]
```

This scans `docs`, the three listed files, both test folders and `crates`, and nothing else. `docs/spec/core/login.md` is `FS`'s, because the deepest place containing it is; `crates/grund/README.md` is `page`'s, because a listed file outranks every folder; every other file under `crates` is `code`'s ([§FS-config-v2.schema.places.ownership](FS-config-v2.md#schemaplacesownership-a-listed-file-then-the-deepest-folder)). The decisions behind the complement and ownership rules are [§DF-v2-place-ownership](../decisions/functional/DF-v2-place-ownership.md#df-v2-place-ownership-in-v2-the-row-named-code-is-the-complement-and-a-file-belongs-to-its-listed-row-or-else-its-deepest-folder).

#### schema.places.complement: `code` is the complement, by name

The row named `code` is the **complement**: the one row that holds source declarations of every kind, and the owner of every scanned file no other place claims. It is non-citable by definition: `citable = false` on it is redundant and accepted, and `citable = true` is refused at its line: `` `[schema.kinds.code]` is the complement and declares no IDs ``. Its places are its **extent**. A `code` row with no place, or no `code` row at all, has the config root as its extent, so a v2 file narrows the scan only by writing `[schema.kinds.code] folders = [...]` ([§FS-config-v2.defaults.1](FS-config-v2.md#defaults1-what-v2-fixes)). A file outside every place is then not read, except by `--full` ([§FS-config-v2.schema.places.boundaries](FS-config-v2.md#schemaplacesboundaries-what-a-place-may-name)). `[rules.citations.code]` governs the complement's files whether or not the row is written.

Every other row with no place is told apart by `citable`. A citable one is declared only in doc-comments ([§FS-config-v2.schema.places](FS-config-v2.md#schemaplaces-rows-and-their-places)). A non-citable one is a place with no place, refused at its header: `` `[schema.kinds.<NAME>]` has no place: give it `folders` or `files`, or name the complement `code` ``.

A scoped `AGENTS.md` that `grund init` generates ([§FS-init.2.3.5.11](FS-init.md#23511-scoped-guidance-a-rows-guidance-lands-above-its-places)) is not a unit of the place it sits in unless a row lists it in `files`, which claims it like any other file ([§FS-init.2.3.5.15](FS-init.md#23515-a-generated-scoped-entrypoint-is-not-a-unit-of-its-place)). A root `AGENTS.md` is scanned exactly when a place covers it: listed in some row's `files`, or inside the complement's extent, which it is when that extent is the config root ([§REQ-agents-md.4](../requirements/REQ-agents-md.md#4-scanned-like-everything-else)).

#### schema.places.ownership: A listed file, then the deepest folder

Every scanned file belongs to exactly one row. A file listed in a row's `files` belongs to that row. Otherwise it belongs to the row whose folder containing it is deepest, the complement's folders included; and a file in no place belongs to the complement. No tie can arise, because two equal places are refused ([§FS-config-v2.schema.places.boundaries](FS-config-v2.md#schemaplacesboundaries-what-a-place-may-name)). Two nested folders of one row are both that row's, and the deepest one owns the file. This replaces v1's rule, under which a file in two nested homes falls to the complement ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)), for v2 files only.

Every per-file rule asks this one question, so a nested place is never outside them:

- **Citation directions and grounding.** A file is held to its owner's `[rules.citations.<NAME>]` and ladder ([§FS-config-v2.rules.citations](FS-config-v2.md#rulescitations-rulescitations), [§FS-config-v2.rules.grounding](FS-config-v2.md#rulesgrounding-grounding-is-a-ladder-replaced-whole)). A finding that names a non-citable row by its place names the place that owns the file, such as `tests/integration/ must cite FS`.
- **Misplaced declarations.** A declaration of kind `K` belongs in a file owned by `K` or by the complement; a row whose places are all files keeps v1's single-file exactness across those files ([§FS-declarations.checks.misplaced-declaration](FS-declarations.md#checksmisplaced-declaration-misplaced-declaration-configured-kind-home)).
- **Indexes.** Each folder of a citable row has its own index, the row's `index` name resolved in that folder, which lists the declarations that folder owns. A declaration in a nested place of another row is listed in that row's index, and a `files` place is not indexed ([§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)).
- **Stubs.** A stub for a source declaration may sit in any folder of its kind, and one is enough ([§FS-declarations.stubs](FS-declarations.md#stubs-what-a-stub-points-at)).
- **What the views print.** `grund schema` lists every place of a row, and draws a `code` row that has places as the complement ([§FS-schema.4](FS-schema.md#4-nodes)). The Project map gives each kind one row linking every place ([§FS-init.2.3.4.4.1](FS-init.md#23441-rows-for-places)). `grund id` names every folder ([§FS-id.2.2](FS-id.md#22---format-json)). `grund config show` prints one `files` and one `folders` list per row, in that order, each in written order and normalized, and prints no `citable = false` on `code`; a place that does not exist is printed like any other, and an implicit complement prints no row.

#### schema.places.boundaries: What a place may name

Each rule here is a load error located at the entry it names unless it says otherwise ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)).

##### schema.places.boundaries.1: Normalized before compared

A place is compared after lexical normalization: `./docs`, `docs/` and `docs` are one place. Comparison is byte-exact; two spellings that differ only in case and name one directory on a case-insensitive filesystem resolve to one target and meet [§FS-config-v2.schema.places.boundaries.3](FS-config-v2.md#schemaplacesboundaries3-decided-on-the-path-as-written). A place that is absolute, or that climbs out of the config root with `..`, is refused, because its index, stubs and entrypoint would be written outside the project ([§REQ-no-data-loss.2](../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own)): ``place `<p>` in `[schema.kinds.<NAME>] <key>` is outside the config root``. A place that normalizes to the config root itself, such as `.` or `docs/..`, is refused too, because the complement already has the root as its extent ([§FS-config-v2.schema.places.complement](FS-config-v2.md#schemaplacescomplement-code-is-the-complement-by-name)): ``place `<p>` in `[schema.kinds.<NAME>] <key>` names the config root, which is the complement's extent``.

##### schema.places.boundaries.2: One place, one row

The same normalized place written twice, on two rows or on one, `files` against `folders` included, is refused at the later entry, naming the earlier: ``place `<p>` in `[schema.kinds.<NAME>] <key>` is already a place of `[schema.kinds.<OTHER>]` (line <n>); a place belongs to one row``. `<p>` is the later entry as written.

##### schema.places.boundaries.3: Decided on the path as written

Ownership reads the path as written, not its link target, as a v1 home does ([§FS-config.3.4.11](FS-config.md#3411-what-a-home-is-used-for)). Two places whose resolved targets coincide are the duplicate of [§FS-config-v2.schema.places.boundaries.2](FS-config-v2.md#schemaplacesboundaries2-one-place-one-row), because the walk keeps one spelling of a file ([§DF-symlink-scan.2.3](../decisions/functional/DF-symlink-scan.md#23-the-report-names-the-link-never-the-target)) and the other place would be silently empty. A folder place that is a directory link leaving the project root is refused rather than pruned ([§DF-symlink-scan.2.2](../decisions/functional/DF-symlink-scan.md#22-the-canonical-project-root-fences-directory-links-not-file-links-or-scan-paths)): ``place `<p>` in `[schema.kinds.<NAME>] folders` links out of the project root``. A `files` place that is a link is followed, as `CLAUDE.md -> AGENTS.md` is.

##### schema.places.boundaries.4: A missing place is quiet

A place that does not exist scans nothing and earns no finding, as a v1 home does ([§FS-config.3.5.8](FS-config.md#358-every-configured-kind-home-is-scanned)), so a fresh repository loads clean. A citation of a kind none of whose places exists says so at the citer ([§FS-check.3.1.4](FS-check.md#314-a-kind-none-of-whose-places-exists)).

##### schema.places.boundaries.5: Not inside a member

A place at or inside a workspace member's root, at any nesting depth, is refused, because the root scan never enters a member ([§FS-workspace.6](FS-workspace.md#6-nested-project-boundary)): ``place `<p>` in `[schema.kinds.<NAME>] <key>` is inside workspace member `<m>`, which the root never reads``.

##### schema.places.boundaries.6: `scan = false` covers the row

The key belongs to the row and covers every place of it; places that differ in it are two rows. Ownership answers nesting both ways: a scanned place under an unscanned one is walked and its files are its row's, and an unscanned place under a scanned one is pruned. `scan = false` on `code` is refused at its line: `` `[schema.kinds.code] scan = false` cannot unwalk the complement; narrow its places or use `[schema.sources] exclude` ``.

##### schema.places.boundaries.7: `--full` widens past the places

`grund check --full` reads the whole config root past every place and past `scan = false`, as it reads past v1's `include` ([§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only)); a file no place claims is the complement's and is reported in the outside-the-scope tier. A member's root stays out under `--full`. An explicit path argument reads what it names, an unscanned place included, and the files it reads have the owners a whole run gives them.

### schema.sources: `[schema.sources]`

`[schema.sources]` admits `languages`, `declarations`, `exclude` and `respect_gitignore`, plus the tables `[schema.sources.extensions]` and `[schema.sources.definitions.<name>]`. Together they say which files a v2 project reads, which reader and comment hosts each file takes, and which comments may declare an ID, and every one of those answers is read from the configuration and the file's own name, never from a kind row's `syntax` and never from the file's contents ([§GOAL-polyglot-citation](../goals.md#goal-polyglot-citation-ids-cite-cleanly-from-anywhere-they-are-useful)). A file's **language** is the one its final extension maps to: an entry of `[schema.sources.extensions]` first, then the epoch suffix of an admitted language ([§FS-config-v2.schema.sources.epoch](FS-config-v2.md#schemasourcesepoch-the-v2-language-epoch)). A file whose extension maps to no admitted language is not read, as a file of an unlisted extension is not read in v1 ([§FS-scan.scope.extensions](FS-scan.md#scopeextensions-an-extension-makes-a-file-readable-not-declarable)); a value home's value sources ([§FS-values.terms](FS-values.md#terms-terms)) are the one exception ([§FS-config-v2.schema.sources.values](FS-config-v2.md#schemasourcesvalues-a-value-home-reads-a-file-by-its-language)).

```toml
grund_config_version = 2
[schema.sources]
languages = ["markdown", "rust", "notes"]
declarations = "doc-comments"
exclude = ["generated", "/docs/archive", "src/**/vendor/**"]
[schema.sources.extensions]
rustdoc = "rust"
note = "notes"
[schema.sources.definitions.notes]
class = "source"
comment_prefixes = ["%%"]
```

This reads `.md`, `.rs` and `.rustdoc` files, the last as Rust, and `.note` files through the `%%` prefix; it lets only Rust doc comments, and any `%%` comment of a `.note` file, declare an ID; and it skips every `generated` file or directory at any depth, the root's `docs/archive`, and every `vendor` subtree below `src`.

#### schema.sources.languages: `languages` admits languages, and only it does

`languages` is a list of language names, built-in or defined under `[schema.sources.definitions]`. An omitted `languages` is the epoch's **default** set, in the epoch's order ([§FS-config-v2.schema.sources.epoch](FS-config-v2.md#schemasourcesepoch-the-v2-language-epoch)). An explicit list replaces that set whole, `[]` included: a list is one value ([§FS-config.principle.unit](FS-config.md#principleunit-one-leaf-key-overrides-and-one-value-is-whole)), and `[]` admits nothing, so no file is read. Neither a definition nor an extension mapping admits a language; only `languages` does.

A name that is neither built-in nor defined is refused at the `languages` line: ``unknown language `<name>` in [schema.sources] languages``. A name written twice is refused at that line: ``language `<name>` is listed twice in [schema.sources] languages``. The default set is fixed at the epoch, so installing a later binary never widens an omitted `languages`: a language added to grund after the epoch is an **opt-in** row, admitted only by naming it.

#### schema.sources.epoch: The v2 language epoch

These are the built-in languages of v2, fixed on 2026-10-06 by agent-grounds/grund#456's approved proposal. A suffix is matched case-sensitively and written without its leading dot. Every **default** row is admitted by an omitted `languages`, in this order; an **opt-in** row is admitted only by naming it. Any other suffix, such as `h`, `hpp`, `jsx`, `mjs` or `kts`, is read only through an explicit `[schema.sources.extensions]` entry.

| Admission | Class | Language: suffixes | Comment hosts | Doc profile |
|---|---|---|---|---|
| default | document | `markdown`: `md` | Markdown headings | none |
| default | data | `json`: `json` | the JSON value-file reader of [§FS-config.3.4.9](FS-config.md#349-values--first-class-value-declarations) | none |
| default | source | `rust`: `rs`; `c`: `c`; `cpp`: `cpp`; `java`: `java`; `kotlin`: `kt`; `scala`: `scala`; `swift`: `swift`; `javascript`: `js`; `typescript`: `ts`, `tsx`; `csharp`: `cs`; `php`: `php` | `//`, `/*`, `*`; PHP also `#` | `c-family` |
| default | source | `python`: `py` | `#`, and triple-quoted docstrings | `python` |
| default | source | `go`: `go` | `//` | `go` |
| default | source | `ruby`: `rb` | `#` | `ruby` |
| default | source | `sql`: `sql` | `--` | `sql` |
| default | source | `lua`: `lua` | `--` | `ldoc` |
| default | source | `haskell`: `hs`, `lhs` | `--` | `haddock` |
| default | source | `lisp`: `lisp`; `scheme`: `scm`; `clojure`: `clj` | `;` | none |
| default | source | `ada`: `ada`, `adb`, `ads` | `--` | none |
| opt-in | source | `shell`: `sh`, `bash`, `zsh` | `#` | `shell` |
| opt-in | source | `r`: `r`, `R` | `#`, and roxygen's `#'` | `roxygen` |
| opt-in | source | `objectivec`: `m`, `mm`; `dart`: `dart` | `//`, `/*`, `*` | `c-family` |

A language's comment hosts are the prefixes it reads a comment behind, each standing for the forms of [§FS-languages.prefixes.forms](FS-languages.md#prefixesforms-the-forms-a-prefix-stands-for), matched longest first. Its **doc profile** is the frozen recognizer [§FS-config-v2.schema.sources.declarations](FS-config-v2.md#schemasourcesdeclarations-declarations-picks-which-comments-declare) asks which of its comments are documentation; the profiles are those of [§FS-inline-citation-style.1.1.2](FS-inline-citation-style.md#112-languages) as they stood at the epoch, and [§FS-inline-citation-style.1.1.4](FS-inline-citation-style.md#114-declaration-applicability) fixes what each admits. A row is never changed once published: a language, a suffix or a profile that changes membership or a block boundary is a new opt-in row, a new profile name, or a new config version ([§FS-config.5.2](FS-config.md#52-every-older-version-keeps-its-meaning)).

#### schema.sources.extensions: `[schema.sources.extensions]` maps a suffix to a language

Each key is a suffix and each value a language name. An entry overrides the epoch's assignment of that suffix, so `h = "c"` reads `.h` files as C and `md = "notes"` takes Markdown's suffix away from it. A key is a nonempty literal final extension: no dot, no path separator, and no glob character among `*`, `?`, `[`, `]`, `{`, `}` or `!`. A malformed key is refused at its line: ``invalid suffix `<key>` in [schema.sources.extensions] (expected a final extension with no dot, separator or glob)``.

A value must name a language that `languages` admits. An unknown name is refused at the entry's line: ``unknown language `<name>` in [schema.sources.extensions]``; a known name that `languages` does not admit is refused there too: ``[schema.sources.extensions] <key> maps to `<name>`, which [schema.sources] languages does not admit``. The mapped file takes the target language whole: its reader, its comment hosts, its doc profile and, for `python`, its docstring reading, all together. A mapping selects none of them separately.

#### schema.sources.definitions: `[schema.sources.definitions.<name>]` defines a language

A definition adds a language to the names `languages` and `[schema.sources.extensions]` may use; it admits nothing and maps no suffix by itself, so a definition that no admitted mapping reaches loads and reads nothing. Its name matches `[a-z][a-z0-9_]*` and is not a built-in name: ``[schema.sources.definitions.<name>] cannot redefine the built-in language `<name>` ``, and a name outside the grammar is ``invalid language name `<name>` (expected [a-z][a-z0-9_]*)``, both at the header's line. It admits four keys, and an unknown key is refused at its line per [§FS-config-v2.reader.1](FS-config-v2.md#reader1-tables-and-keys-are-closed):

| Key | Values | Default |
|---|---|---|
| `class` | `document`, `data`, `source` | required |
| `comment_prefixes` | literal, nonempty, single-line strings, each once | `[]` |
| `doc_comments` | `none`, `c-family`, `python`, `go`, `ruby`, `shell`, `sql`, `ldoc`, `haddock`, `roxygen`, `lisp` | `none` |
| `docstrings` | `off`, `python` | `off` |

A definition without `class` is refused at its header: ``[schema.sources.definitions.<name>] needs `class` ``. Each value outside its column is refused at its line, naming the values it expected. The combinations below are refused at the line of the key that breaks them:

- A `document` definition reads its files with the Markdown reader, a `data` definition with the JSON value-file reader, never the YAML one of [§FS-values.2.2.3](FS-values.md#223-how-a-yaml-source-is-read). Neither takes a doc profile or docstrings other than `none` and `off`. A `document` definition may keep comment prefixes, for a fenced example's reading, as v1's Markdown does; a `data` definition takes none. Each is refused at the offending key's line: `` [schema.sources.definitions.<name>] class = "<class>" takes no <key> ``.
- A `source` definition needs a comment host: at least one prefix, or `docstrings = "python"`. One with neither is ``[schema.sources.definitions.<name>] reads no comment: give it comment_prefixes or docstrings = "python"``, at its header.
- `doc_comments = "python"` needs `docstrings = "python"`, and a marker profile whose marker no configured prefix can reach — `c-family` without `//` or `/*`, `ldoc`, `haddock` or `sql` without `--`, `roxygen`, `ruby` or `shell` without `#`, `go` without `//`, `lisp` without `;` — is refused at the `doc_comments` line: `` doc_comments = "<profile>" is unreachable: no comment prefix of [schema.sources.definitions.<name>] spells it ``.
- A prefix written twice is refused at the `comment_prefixes` line.

`lisp` is v1's Lisp position recognizer, frozen as it stands at this change ([§FS-inline-citation-style.1.1.2](FS-inline-citation-style.md#112-languages)): a comment block is a doc-comment when the next line opens a form whose head symbol starts with `def` or `cl-def`, or is `ns`. It is additive ([§FS-config.5.1](FS-config.md#51-new-keys-are-not-a-new-version)): an older binary refuses `lisp` at its line. It exists so that migration can carry the `lisp`, `scm`, `clj` and other Lisp extensions in v1's default extension list, which no other profile reads the same way ([§FS-config-migrate](FS-config-migrate.md#fs-config-migrate-grund-config-migrate---to-2-shows-the-version-2-file-that-means-what-a-version-1-config-means-and-writes-it-only-when-asked-and-only-when-it-does)).

A prefix is literal: matched byte for byte at the first non-blank column, the longest configured prefix first, and standing for nothing beyond itself. A definition's prefixes are its own; there is no global prefix list in v2.

#### schema.sources.values: A value home reads a file by its language

The value home still decides which files are asked: the `file` home itself, or each direct child of a `folder` home, as [§FS-values.2.2](FS-values.md#22-declarations-from-value-sources) lists them. Its language decides whether one is a value source. A file whose name maps to an admitted language is one exactly when that language's class is `data`, so a `.jsonc` under `jsonc = "json"` and a file a `data` definition reaches are read as JSON values, and a `.json` mapped to a `source` or `document` language is not. A file whose name maps to an admitted language is always read with the JSON reader. A file whose name maps to no admitted language is one when its final extension is `json`, `yaml` or `yml`, as in v1, and is read with the JSON reader for `json` and the YAML reader for the other two ([§FS-values.2.2.3](FS-values.md#223-how-a-yaml-source-is-read)): value sources are catalog input, not scan input ([§FS-values.2.2](FS-values.md#22-declarations-from-value-sources)), so a value home's `.json` or `.yaml` is read under `languages = ["markdown"]` too. A language never makes a file outside a value home a value source, and the text scan never reads a `data` language's files.

#### schema.sources.declarations: `declarations` picks which comments declare

`declarations` takes `"doc-comments"`, the v2 default, or `"any-comment"`. A line in declaration position ([§FS-declarations.line](FS-declarations.md#line-what-a-line-in-declaration-position-declares)) in a source file declares only when the comment block it sits in is **eligible**. Under `"any-comment"` every comment block is eligible, as in v1; under `"doc-comments"` the blocks below are. A Markdown heading and a value file are not comments and are not asked. The key changes nothing about citations: a marked citation in any comment is recognized, resolved and checked as before. Any other value is refused at its line: ``unknown [schema.sources] declarations `<value>` (expected doc-comments or any-comment)``. v1 has no such key, and its declarations stay those of every comment in declaration position ([§FS-config.5.2](FS-config.md#52-every-older-version-keeps-its-meaning)).

```rust
/// FS-existing: Real documentation
fn example() {
    // FS-existing: Ordinary root-ID note
    // See §FS-existing.
}
```

Here v1 declares `FS-existing` twice and reports the duplicate; v2 declares it once, at line 1, and the ordinary comment's citation still resolves to it.

##### schema.sources.declarations.eligible: Which blocks are eligible

The file's language is the one [§FS-config-v2.schema.sources](FS-config-v2.md#schemasources-schemasources) resolves from its name, never a kind row's setting, and that language's doc profile decides:

- a language whose profile is not `none` makes a block eligible exactly when the profile's recognizer classifies it as a doc comment, with the frozen applicability of [§FS-inline-citation-style.1.1.4](FS-inline-citation-style.md#114-declaration-applicability);
- a language whose profile is `none` — Lisp, Scheme, Clojure and Ada in the epoch, and a definition that writes no `doc_comments` — makes every comment block behind its configured hosts eligible, so a language with no notion of documentation can still declare;
- a `docstrings = "python"` language makes each docstring eligible, and under the `python` profile its `#` blocks are not.

##### schema.sources.declarations.gate: One gate, before every recognizer

Eligibility is decided once per block, from the language and the block alone, whether or not the block carries a citation and whether or not the inline-style budgets measured it. It is decided before the canonical declaration recognizer, the off-grammar and legacy fallback recognizers ([§FS-config.3.2.5](FS-config.md#325-off-grammar-declarations-stay-readable)) and the near-miss check ([§FS-declarations.checks.declaration-near-miss.1](FS-declarations.md#checksdeclaration-near-miss1-what-counts)), so a line in an ineligible block declares nothing, is no near miss, and cannot reach the catalog by a fallback. Every consumer of a declaration reads the same answer: body ranges and sections ([§FS-show.2.3.1](FS-show.md#231-what-counts-as-the-comment-block)), anchors, stub verification ([§FS-declarations.stubs](FS-declarations.md#stubs-what-a-stub-points-at)), the formatter's protection of declaration lines, and the editor. A section-suffixed token never declares its root ID in either mode ([§FS-declarations.line.section-suffix.2](FS-declarations.md#linesection-suffix2-with-the-colon-the-whole-token-is-the-near-miss)).

##### schema.sources.declarations.prose: An ineligible declaration-shaped line draws no new finding

A declaration-shaped line in an ineligible block is prose. It gets no finding of its own, and `declaration-near-miss` is not reused to report it. A marked citation written on it is still a citation and draws every finding a citation does.

#### schema.sources.exclude: `exclude` is a gitignore-style glob list

`exclude` is a list of gitignore-style patterns resolved against the config root ([§FS-config.1](FS-config.md#1-file-location-and-discovery)), the grammar `[presentation.fmt] exclude` already uses ([§FS-config.3.10.1](FS-config.md#3101-entries-are-gitignore-style-globs)). Its default is `[]`: v1's built-in names `target`, `node_modules`, `.git`, `dist`, `build` and `.venv` are not v2 defaults, and a v2 project that wants them writes them. A pattern is read as a `.gitignore` line is:

- a pattern with no `/` but a trailing one matches a file or directory of that name at any depth, so `generated` skips `generated/` and `a/b/generated` alike;
- a pattern with a leading or inner `/` is anchored at the config root, so `/docs/archive` skips the root's `docs/archive` and not `lib/docs/archive`;
- a trailing `/` matches directories only, so `build/` leaves a file named `build` read;
- `*`, `?`, bracket classes, `**` and backslash escapes mean what they mean in `.gitignore`, so `src/**/vendor/**` skips every file below any `vendor` directory under `src`;
- a `#` line is a comment, and `!` negates a pattern the list has already matched, the later line winning.

A negation never rescues a file below a directory the scan already pruned, as in git: `["build/", "!build/keep.md"]` leaves `build/keep.md` unread. A pattern the parser refuses is refused at the `exclude` line, per [§FS-config.4.3](FS-config.md#43-invalid-config-behavior).

##### schema.sources.exclude.roots: How `exclude` composes with the other filters

`exclude` is a rule about descent, so it composes as v1's component names do ([§FS-scan.scope.roots](FS-scan.md#scoperoots-a-scan-root-outruns-every-rule-about-descent)):

1. A **scan root** — a row's place, or a path handed to the command — is read even where a pattern names it; what lies below it is filtered. The hidden-file rule ([§FS-scan.scope.hidden](FS-scan.md#scopehidden-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent)) and language admission are rules about a file's own name and still win at a root.
2. Below a root, a path is read only if neither `exclude` nor an ignore file matches it. `respect_gitignore = false` turns the ignore files off and leaves `exclude` in force; the two are separate filters, and a `!` in one never overrides the other.
3. A row with `scan = false` is pruned from the default scope, and an explicit path argument or `--full` reaches it ([§FS-config.3.4.7.3](FS-config.md#3473-an-explicit-path-argument-still-reads-it), [§FS-config.3.4.7.5](FS-config.md#3475-under---full)); `--full` still applies `exclude`, the ignore files and the hidden rules. A narrowed `check` narrows only what it reports, never what resolves ([§FS-scan.path.check](FS-scan.md#pathcheck-check-reads-more-than-it-reports)).
4. A path reached through a symlink is matched by the spelling the walk reached it under ([§FS-scan.scope.links.names](FS-scan.md#scopelinksnames-the-directory-rules-apply-under-the-link-name)). Workspace ownership, the directory-link fences, E2E fixture boundaries, loop refusals and the one-read-per-physical-file rule are decided before `exclude` and are not loosened by it ([§FS-scan.identity.read-once](FS-scan.md#identityread-once-one-physical-file-is-read-once)).
5. A root outside the config root is matched by its lexical spelling relative to the config root: an anchored pattern never reaches outside the root, and an unanchored one still applies below it.

Scanner and formatter share the grammar and not the effect: `[presentation.fmt] exclude` keeps a file from being rewritten while the scan still reads it, and `[schema.sources] exclude` keeps the scan from reading it at all.

##### schema.sources.exclude.stubs: A pattern is the `exclude` reason a stub's target is not read

Under v2 a pattern is the `exclude` reason of [§FS-declarations.stubs.target.3](FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing),
in place of v1's directory names. It is asked as the walk asks it, of each directory
on a stub target's path and of the target itself, from each scan root the run walks
and from the config root, and at each step before the ignore files. A match refuses
the target before pairing: `broken-stub`, an error at the stub, with the reason,
pattern and remedy defined there. For example:

```text
docs/fs/x.md:1: error: stub link target ../gen/a.md is not scanned: [schema.sources] exclude matches it by gen; this became an error in grund 0.19.0 — move the target out of what gen matches, or take gen out of [schema.sources] exclude
```

#### schema.sources.effective: The admitted set is read from the configuration alone

The resolved source model — every admitted language with its class, suffixes, comment hosts and doc profile, the `declarations` mode, `exclude` and `respect_gitignore` — is computed from the configuration without reading a source file. `grund config show` prints it in v2 spelling ([§FS-config.requirements.6](FS-config.md#requirements6-the-effective-configuration-is-inspectable--realized-provenance-deferred)): `languages` always, written out in full, so an omitted list prints as the epoch's default set; `declarations`, `exclude` and `respect_gitignore` always; and each `[schema.sources.extensions]` entry and each definition with every key, the defaults included. Built-in rows are not printed, since the epoch fixes them. The output loads back to the same model.

##### schema.sources.effective.bindings: The embedding API publishes the model under `sources`

The embedding API's effective config, a schema-keyed mapping ([§FS-distribution.3.3.1](FS-distribution.md#331-typed-immutable-results)), carries the same model under a `sources` key. The key is present only for a v2 project, so a v1 snapshot keeps its bytes. Its `languages` lists the admitted languages in `languages` order, each a mapping of `name`, `class`, `suffixes` (a built-in's epoch suffixes, `[]` for a definition), `comment_prefixes`, `doc_comments`, `docstrings` (`true` where the language reads Python docstrings) and `builtin`. Its `extensions` maps each `[schema.sources.extensions]` suffix to its language, and `declarations` and `exclude` are those keys' values.

A v2 snapshot still publishes the v1-shaped `scan` mapping, whose keys are v1's. Its `comment_prefixes` is `[]` and its `docstring_python` is `false`, because v2 has no global prefix list and no global docstring switch: each language carries its own. Its `extensions` lists every suffix the text scan reads: each mapped suffix whose language is not a `data` one, in authored order, then each epoch suffix of an admitted built-in other than `json` that no mapping takes. Its `include` is `null`, since v2 reads no `include`, its `exclude` is `[]`, since v2's exclusions are `sources.exclude`, and its `respect_gitignore` is `[schema.sources] respect_gitignore`. `sources` is the authority: a caller that needs a v2 file's hosts reads them from that file's language.

#### schema.sources.migration: What a v1 source configuration becomes

These are the rules `grund config migrate --to 2` applies to v1's `[scan]` source keys ([§FS-config-migrate](FS-config-migrate.md#fs-config-migrate-grund-config-migrate---to-2-shows-the-version-2-file-that-means-what-a-version-1-config-means-and-writes-it-only-when-asked-and-only-when-it-does)). Every translation writes its defaults out, so the v2 file keeps v1's meaning after any later edit to the tree, and a report that matches on one tree is no proof of equivalence ([§FS-config-v2.mapping](FS-config-v2.md#mapping-what-the-records-keep-apart)).

1. `declarations = "any-comment"` is written, which is v1's declaration recognition.
2. `languages` is written explicitly. Each distinct v1 reading becomes one definition, named `v1_<profile>`: `v1_markdown` for `md`, `v1_<doc profile>` for a source extension, with `-` written `_` (`v1_c_family`, `v1_python`, `v1_lisp`), and `v1_plain` for a source extension with no doc profile. Its class is the one its v1 reader is — `document` for `md`, `source` otherwise — and it copies the **whole** v1 `comment_prefixes` list and the doc profile [§FS-inline-citation-style.1.1.2](FS-inline-citation-style.md#112-languages) gives those extensions. Each v1 extension is mapped to its definition by `[schema.sources.extensions]`. *Amended by agent-grounds/grund#461:* this rule first named each definition after its extension, which cannot always be done, because a definition may not take a built-in name (`c`, `go`, `java`, `lua`, `sql`, `lisp`, `ada` and others are built-in) and `R` breaks the name grammar. A Markdown definition keeps v1's prefixes, since v1 reads a Markdown fence through them.
3. `.py` takes `docstrings = "python"` and the `python` profile when `docstring_python = true`, and `docstrings = "off"` with no Python profile when it is `false`, which keeps v1's raw-line reading of a triple-quoted string.
4. Each v1 `[scan] exclude` name becomes the escaped, directory-only glob `<name>/`, which matches that directory at any depth, as v1 did.

Migration refuses, and leaves the config's bytes unchanged, rather than approximate:

- a path-shaped v1 `exclude` entry, one containing `/`, which never matched in v1: activating it as the glob it looks like would change what is scanned ([§DISC-core-concerns.8.3.1](../discussions/proposals/2026-09-30-core-concerns.md#831-amended-a-path-shaped-entry-is-refused-not-activated)); a project that wants the glob adopts it by hand;
- a v1 arrangement that scans `.json` as text while a kind's `values` also reads it as a value source, or that admits `.json` only through a value home: one class and one admission cannot hold both;
- a v1 extension or prefix outside the domains of [§FS-config-v2.schema.sources.extensions](FS-config-v2.md#schemasourcesextensions-schemasourcesextensions-maps-a-suffix-to-a-language) and [§FS-config-v2.schema.sources.definitions](FS-config-v2.md#schemasourcesdefinitions-schemasourcesdefinitionsname-defines-a-language), a source extension that would read no comment, and a reader v2 has no class for.

A `.json` extension scanned explicitly in v1 with no value authority becomes a `source` definition with its prefixes.

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

Column findings use [§FS-inline-citation-style.4.1.1.1](FS-inline-citation-style.md#4111-every-over-limit-line)'s over-limit line suffix on every reporting channel, filtered against the threshold selected for that finding.

### schema.fields: Fields and forms

`[schema.kinds.<NAME>.fields.<field>]`, with its keys `title`, `presence`, `position`, `after`, `content` and `one_of`, and a row's `form` and `closed`, are specified by [§FS-fields](FS-fields.md#fs-fields-a-v2-kind-states-the-shape-of-its-declarations-and-check-holds-every-declaration-to-it): what each key means, what the loader refuses, and the findings `grund check` reports. They are executed ([§FS-config-v2.rollout](FS-config-v2.md#rollout-what-this-grund-executes-and-what-it-refuses)).

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

#### rules.citations.uncited: An obligation never judges a file that cites nothing

For a place or the complement an obligation is asked per file, and only of a file that carries a citation ([§FS-check.3.11.5](FS-check.md#3115-a-file-with-no-citation-is-no-unit)): `[rules.citations.e2e] must = ["FS"]` fails a file in `tests/e2e/` that cites only `GOAL`, and passes one that cites nothing. Whether such a file cites at all is grounding's question, which `[rules.citations.<PLACE>.grounding]` asks ([§FS-config-v2.rules.grounding](FS-config-v2.md#rulesgrounding-grounding-is-a-ladder-replaced-whole)). Every text that says what `must`, `warn` or `should` does says so, because an author who reads a `must` as "every file cites FS" ships a rule that enforces less than it says. The keys' descriptions, which `grund config explain` prints ([§FS-config-explain.3](FS-config-explain.md#3-a-keys-answer)), are, for `must`:

```text
Kinds this one must cite; a miss is an error. A file of a place or the complement that cites nothing is not checked by this key; `[rules.citations.<ROW>.grounding]` makes it a finding.
```

and for `warn` and `should` the same second sentence after `Kinds this one must cite; a miss is a warning.` and `Kinds this one should cite; a miss is a suggestion.` The generated entrypoint says it per home ([§FS-init.2.3.5.2.2](FS-init.md#23522-a-home-whose-row-grounds-nothing-says-so)) and `grund schema`'s legend once ([§FS-schema.8.3](FS-schema.md#83-the-legend-says-an-obligation-skips-an-uncited-file)).

### rules.grounding: Grounding is a ladder, replaced whole

`[rules.citations.grounding]` is the project's ladder and `[rules.citations.<PLACE>.grounding]` a row's. A ladder's keys are the positive strengths and its values are units: `"file"`, or `"h2"` to `"h6"`, the heading levels of [§FS-check.3.6.2](FS-check.md#362-the-unit). Each rung asks that every unit of that size in a governed file be grounded, and reports an ungrounded one on its strength's channel ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)). A row's files are governed as [§FS-check.3.6.1](FS-check.md#361-which-files-a-row-governs) says. A stronger rung never uses a finer unit: `must = "h2"` beside `warn = "file"` is refused at the stronger key: ``grounding ladder: `must` (h2) is finer than `warn` (file); a stronger rung never uses a finer unit``.

A ladder is one setting. A row that writes a ladder replaces the project's ladder whole, and a rung the row leaves out is not inherited. This is [§FS-config.principle.unit](FS-config.md#principleunit-one-leaf-key-overrides-and-one-value-is-whole)'s whole-value rule, applied to a setting v1 spelled as two independent keys. `grund check --require-grounding` supplies the project ladder's `must` rung at `"file"` unless the project ladder already writes a `must` rung; the project ladder's other rungs remain, and a row that writes its own ladder still replaces the project's whole ([§FS-config.principle.cli](FS-config.md#principlecli-a-cli-input-enters-at-the-scope-its-flag-spells)).

### rules.resolution: `[rules.resolution]`

`[rules.resolution]` maps a kind to `must` or `warn`: how a citation of an external kind that has no snapshot is reported ([§FS-check.4.12](FS-check.md#412-missing-snapshot)). `warn` is the missing-snapshot warning, and `must` is the error. A kind with `fetch` defaults to `must`, as in v1. An explicit entry for a kind whose row has no `fetch` is refused at its line: `` `[rules.resolution] <KIND>` needs `fetch` on [schema.kinds.<KIND>] ``. `should` and `may` are refused, because a missing snapshot is never a suggestion. Resolution never fetches ([§REQ-runs-offline.1](../requirements/REQ-runs-offline.md#1-read-and-verification-paths-execute-nothing)).

## presentation: The presentation concern

`[presentation]` admits `description`, `trigger`, `conversation` and `rules`. `[presentation.kinds.<NAME>]` admits `title`, and `[presentation.fmt]` admits `anchors`, `links` and `exclude`. `rules` says where each row's guidance is written: `"home"`, the default, writes it into the `AGENTS.md` of the deepest directory holding the row's places ([§FS-init.2.3.5.11](FS-init.md#23511-scoped-guidance-a-rows-guidance-lands-above-its-places)), and `"root"` writes every row into the root entrypoint, exactly as a version-1 config renders ([§FS-init.2.3.5.11.5](FS-init.md#235115-rules--root-and-every-version-1-config-render-at-the-root)). Any other value is refused at its line: `` unknown [presentation] rules `<value>` (expected home or root) ``. `description`, `trigger`, `conversation` and a kind's `title` mean what `project_description`, `[reference] trigger`, `[reference] conversation` and a row's `title` mean in v1 ([§FS-config.3](FS-config.md#3-keys-version-1)). `[presentation.fmt] exclude` is v1's `[fmt] exclude` ([§FS-config.3.10](FS-config.md#310-fmt--suppressing-the-rewrite)), and `anchors = "github"` with `links = "index"` is the formatter's existing effect. Every other value of `anchors` and `links` is subject to [§FS-config-v2.rollout](FS-config-v2.md#rollout-what-this-grund-executes-and-what-it-refuses).

## rollout: What this grund executes, and what it refuses

The format is published whole, and this binary executes part of it. A clause the format defines but this binary does not execute is never accepted and ignored. It is refused at its authored line with `` `<clause>` is part of the v2 format but not supported by this grund yet ``, where `<clause>` is a table's header, `[schema.kinds.FS.fields.terms]`, or a table and key, `[presentation.fmt] anchors`. An omitted default that cannot be executed is refused at the version key; since #456 none is left. The refusal lifts when the feature's owner lands, and it never falls back to a v1 meaning.

| Clause | Status in this grund | Owner |
|---|---|---|
| Envelope, `[workspace]` | executed | #455 |
| `[schema]` keys, `heading_depth` | executed | #455 |
| `[schema.notes.*]`, `[schema.leads.words]` | executed | #455 |
| A row with one place; `citable`, `scan`, `index`, `id_format`, `fetch` | executed | #455 |
| `[rules.citations]`, ladders, `[rules.resolution]` | executed | #455 |
| `[presentation]` `description`, `trigger`, `conversation`, `[presentation.kinds.*] title`, `[presentation.fmt] exclude` | executed | #455 |
| `[presentation.fmt]` `anchors = "github"`, `links = "index"` | executed | #455 |
| `languages = ["markdown"]` | executed | #455 |
| Every other `languages` value, an omitted `languages`, `declarations`, `[schema.sources.extensions]`, `[schema.sources.definitions.*]`, `[schema.sources] exclude` | executed | #456 |
| A row with more than one place, a place inside another row's place, a `code` row with or without places | executed | #457 |
| `[schema.kinds.*.fields.*]`, `form`, `closed` | executed | #458 |
| Any other `anchors` or `links` value | refused | its own change; #459 closed without it |
| `[presentation] rules` | executed | #460 |

The published example of [§DISC-core-concerns.7.1](../discussions/proposals/2026-09-30-core-concerns.md#71-the-canonical-example) parses once each owner has landed, and every refusal above is pinned by a test that turns into an executed case on that day.

### rollout.fmt-values: A recognized formatting key names the value it cannot execute

For an unsupported string value of `[presentation.fmt] anchors` or `links`, the refusal keeps `` `[presentation.fmt] <key>` is part of the v2 format but not supported by this grund yet `` verbatim as a contiguous prefix and appends guidance identifying the supplied value as unsupported and the only assignment this binary executes: `anchors = "github"` or `links = "index"`, respectively. The supplied value is double-quoted, with quotes, backslashes and control characters escaped so the diagnostic stays on one line. The suffix's prose is not fixed; the prefix is ([§FS-errors.3](FS-errors.md#3-message-text), [§FS-errors.requirements.prefix-windows](FS-errors.md#requirementsprefix-windows-a-wording-change-keeps-the-old-message-whole-until-its-window-closes--realized-directional-for-the-next-change)).

The refusal remains at the authored key's line and exits 1. Invalid string syntax and non-string values retain their parse/type errors, rather than becoming unsupported-value refusals. This guidance does not change any other rollout refusal, accepted value, omitted default, effective `config show` setting, or formatting behavior.

## mapping: What the records keep apart

Both readers lower into the same records, so the records hold every state either one can mean. A v1 state that v2 does not spell keeps its meaning. Migration is [§FS-config-migrate](FS-config-migrate.md#fs-config-migrate-grund-config-migrate---to-2-shows-the-version-2-file-that-means-what-a-version-1-config-means-and-writes-it-only-when-asked-and-only-when-it-does), and it refuses each row below whose v2 spelling is `none`. This table is what a migration audits against, and tests on the records hold it, not a comparison of one tree's reports.

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
| A file in two nested homes belongs to the complement | none: the deepest place owns it | the nesting rule, `NestedPlaceFallsToComplement` against v2's `DeepestPlaceWins` |
| A complement renamed by a non-citable homeless row | none: the complement is `code`, titled by `[presentation.kinds.code] title` | the complement's row name |
| Every row's guidance renders in the root entrypoint | `[presentation] rules = "root"`, which a migrated file writes explicitly; a v2 file that omits it renders scoped guidance ([§FS-init.2.3.5.11](FS-init.md#23511-scoped-guidance-a-rows-guidance-lands-above-its-places)) | the layout of the generated guidance, so a migrated config writes the files its v1 config wrote |
| `[scan] include`, separate from the homes | none: the scan is the places and the complement's extent | the derived scan roots |
| A row's `value_chapter` | a field, which adds a "Declaration shapes" section to the generated guidance | the chapter's value roots, apart from the field model |
| `lead_size_warning` counted in lines or bytes | `[schema.leads.words]` only | the lead budget and its unit |
| `[fmt.cross_refs] enabled = false`, or an `anchor_format` other than `"github"` | none in this grund ([§FS-config-v2.rollout](FS-config-v2.md#rollout-what-this-grund-executes-and-what-it-refuses)) | the formatter's link policy |
| An `include` entry that does not exist | none: a place is a file or a folder | the authored scan root, of no known type |
| A project `grounding_level` that only `--require-grounding` switches on | a rung of a ladder, which is always on | the level, apart from whether it is enforced |

## checks: Checks

The check codes the version-2 reader's own findings carry, beside the `check` code catalog's row for each ([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)).

### checks.config-not-toml: A version-2 config holds text TOML refuses

A version-2 config that holds text TOML refuses, and that this grund still reads for one release as the readers before it did, earns one warning with code `config-not-toml` per construct, at the line of the key whose value holds it. Its constructs today are three. One is an array with a string still open at the end of a line that a later line closes ([§FS-config-v2.reader.5.strings.window](FS-config-v2.md#reader5stringswindow-through-grund-021x-an-array-a-later-line-closes-is-read-as-before-and-warns)):

```text
warning: grund.toml:7: unclosed array (a string is still open at the end of line 8), read for now with the line break as a space; close the string on line 8; this warning becomes an error in grund 0.22.0
```

Another is a string that closes before the end of its value or its element ([§FS-config-v2.reader.6.window](FS-config-v2.md#reader6window-through-grund-021x-a-string-that-closes-early-is-read-as-before-and-warns)), whose remedy drops its second half for a scalar or a key:

```text
warning: grund.toml:5: expected string, read for now as the one string `docs" "specs`; end the string on line 5 at its closing quote: write `\"` for a quote inside it, or a `,` between two strings; this warning becomes an error in grund 0.22.0
```

The third is an array with a comma before its `]` that has no element before it ([§FS-config-v2.reader.5.commas.window](FS-config-v2.md#reader5commaswindow-through-grund-021x-the-run-of-commas-before-the--is-taken-off-as-before-and-warns)), whose remedy names the line that comma stands on:

```text
warning: grund.toml:9: expected string, read for now with every comma before the `]` taken off; delete each comma on line 9 that has no element before it; this warning becomes an error in grund 0.22.0
```

The message is the refusal the construct becomes in the named release, then what it is read as until then, then the remedy, then the release ([§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)). It fires only for a config file read as version 2, and never for version 1: version 1 reads no value across lines ([§FS-config.3](FS-config.md#3-keys-version-1)), and it keeps its reading of a string that closes early for good ([§FS-config.5.2](FS-config.md#52-every-older-version-keeps-its-meaning)), which it meets only at `config migrate` ([§FS-config-migrate.3.1.8](FS-config-migrate.md#318-a-string-that-closes-early)), and its list reader already refuses every comma before a `]`, `["a",]` among them, as `expected string` ([§FS-config.3.11](FS-config.md#311-a-list-element-ends-at-a-comma-outside-a-string)).

It is a CLI-level `warning:` on stderr whose text carries the config line ([§FS-errors.2.2.1](FS-errors.md#221-a-location-inside-the-message-text), [§FS-errors.5.2.3](FS-errors.md#523-run-level-findings-in-checks-report)), on `check`, `config validate` and `config show` and no other command, as [§FS-check.4.11.2](FS-check.md#4112-the-same-three-surfaces-as-fs-check43-and-no-more) says of the other config findings, once per construct rather than once per scope. It leaves the exit code alone and stands in place of the `success` line ([§FS-check.4.11.6](FS-check.md#4116-it-stands-in-place-of-the-success-line)). `--ignore config-not-toml` drops it and `--only config-not-toml` selects it ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)). Under `--format=json` it is one warning finding carrying the code, its `path` the config file as a report path and its `line` the key's, in the shape [§FS-errors.5.2.3](FS-errors.md#523-run-level-findings-in-checks-report) gives a warning at a config line.

#### checks.config-not-toml.1: A warning in 0.21.x, the `unclosed array` or `expected string` refusal from 0.22.0

From grund 0.22.0 each construct is refused at load instead, exit `2`: the array as `error: grund.toml:7: unclosed array (a string is still open at the end of line 8); this became an error in grund 0.22.0` ([§FS-config-v2.reader.5.strings.refusal](FS-config-v2.md#reader5stringsrefusal-from-grund-0220-the-open-line-ends-the-array)), the string as `error: grund.toml:5: expected string; this became an error in grund 0.22.0` ([§FS-config-v2.reader.6.refusal](FS-config-v2.md#reader6refusal-from-grund-0220-a-string-that-closes-early-is-expected-string)), and the array with a comma no element precedes as `error: grund.toml:9: expected string; this became an error in grund 0.22.0` ([§FS-config-v2.reader.5.commas.refusal](FS-config-v2.md#reader5commasrefusal-from-grund-0220-a-comma-with-no-element-before-it-is-expected-string)). The code no longer fires for any of them. The code stays: a later construct of the same class, text TOML refuses that a version-2 reader loads, joins it under its own decision for the one release its window lasts.

### checks.config-multiline-string: A version-2 config value holds a multi-line string

A value in a version-2 config that holds a string opened with `"""` earns one warning with code `config-multiline-string` at the line of its key, per construct: an array whose `"""` string opens on one line and that a later line closes ([§FS-config-v2.reader.5.strings.multi-line](FS-config-v2.md#reader5stringsmulti-line-a-string-opened-with--is-one-version-2-does-not-read)), and a value, an element or a key whose `"""` string opens and closes on one line ([§FS-config-v2.reader.6.multi-line](FS-config-v2.md#reader6multi-line-a--string-on-one-line-is-one-version-2-does-not-read)). Such a file is TOML, which is why it is not `config-not-toml`: what is untrue of it is only that version 2 reads it, and through grund 0.21.x it is read as before, its quotes kept and a line break read as a space:

```text
warning: grund.toml:7: unclosed array (a multi-line string opens on line 8, which version 2 does not read), read for now with the line break as a space; write the string on line 8 as one basic string; this warning becomes an error in grund 0.22.0
warning: grund.toml:8: expected string (a multi-line string, which version 2 does not read), read for now as `""one""`, its quotes kept; write the string on line 8 as one basic string; this warning becomes an error in grund 0.22.0
```

It is [§FS-config-v2.checks.config-not-toml](FS-config-v2.md#checksconfig-not-toml-a-version-2-config-holds-text-toml-refuses)'s finding in everything but its trigger and its text: the same surfaces, channel, exit code, selectors and JSON shape ([§FS-errors.5.2.3](FS-errors.md#523-run-level-findings-in-checks-report)), and from grund 0.22.0 the same refusals, each with its multi-line parenthetical.
