# FS-config-v2: grund reads a version-2 config by concern, with one strength vocabulary and fixed defaults

A `grund.toml` that writes `grund_config_version = 2` is read by a reader of its own and spelled by concern: an envelope, then `[schema]` for what exists, `[rules]` for how nodes relate, and `[presentation]` for the bytes `grund` writes ([§FS-config.concerns](FS-config.md#concerns-every-key-belongs-to-exactly-one-concern)). Every constraint takes its strength from one closed set, so a repository can adopt a constraint as a warning, promote it to an error, or keep it advisory without changing format, and the defaults are fixed at the version rather than at whatever a binary ships ([§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable), [§GOAL-zero-config](../goals.md#goal-zero-config-works-on-any-conformant-tree)). A version-1 file is read by the v1 reader and keeps every meaning [§FS-config](FS-config.md#fs-config-grund-reads-a-toml-config-file-found-by-walking-up) gives it ([§FS-config.5.2](FS-config.md#52-a-versions-meaning-moves-only-as-backwards-compatibility-allows)).

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

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, kind, home), [§FS-terms.terms.2](FS-terms.md#terms2-citations) (marker, citation), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, config root, place, complement place) and [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding). Defines:

- **Reader**: the code that turns one version's spelling into the project's records. Each version has its own, and a file is read by exactly one ([§FS-config.5](FS-config.md#5-schema-versioning)).
- **Strength**: one of `must`, `warn`, `should`, `may`, or a prohibition `must-not`, `warn-not`, `should-not`. It names the channel a finding reaches ([§FS-config-v2.rules.strengths](FS-config-v2.md#rulesstrengths-one-strength-vocabulary-two-severities)).
- **Measure**: a quantity a constraint compares against a threshold, such as a note's lines or a lead's words. A measure is a table name and its strengths are its keys ([§FS-config-v2.schema.measures](FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys)).
- **Ladder**: one or more grounding rungs (a strength paired with a unit) written as one setting ([§FS-config-v2.rules.grounding](FS-config-v2.md#rulesgrounding-grounding-is-a-ladder-replaced-whole)).
- **Epoch**: the defaults a version fixes. A later binary never changes them ([§FS-config-v2.defaults](FS-config-v2.md#defaults-the-v2-epoch)).
- **Owner**: the one row a scanned file belongs to: a listed file's row, else the row of the deepest folder containing it, else the complement ([§FS-config-v2.schema.places.ownership](FS-config-v2.md#schemaplacesownership-a-listed-file-then-the-deepest-folder)).
- **Rollout refusal**: a located load error for a v2 clause the format defines but this binary does not yet execute ([§FS-config-v2.rollout](FS-config-v2.md#rollout-what-this-grund-executes-and-what-it-refuses)).

## requirements: What every version-2 config holds to

Four requirements of version 2 are stated where they are shared, and not again here: a v1 file keeps every meaning it had, in the records as well as the reports ([§FS-config.5.2](FS-config.md#52-a-versions-meaning-moves-only-as-backwards-compatibility-allows), under [§REQ-backwards-compatibility.1](../requirements/REQ-backwards-compatibility.md#1-what-is-covered)); a mistake is an error at its line, of which the closed tables of [§FS-config-v2.reader.1](FS-config-v2.md#reader1-tables-and-keys-are-closed) are v2's instance ([§FS-config.requirements.5](FS-config.md#requirements5-a-mistake-in-the-config-fails-loudly)); a strength adds no third severity ([§FS-config.6](FS-config.md#6-what-is-not-configured-here)); and `config show` prints a v2 file in v2 spelling that loads back to the same values ([§FS-config.requirements.6](FS-config.md#requirements6-the-effective-configuration-is-inspectable)).

### requirements.no-v1-meaning: A v2 file is read under v2's rules or refused, never under v1's

Nothing of v1 is substituted into a file that writes `grund_config_version = 2`. v1's tables and discarded aliases are unknown sections there ([§FS-config-v2.reader.1](FS-config-v2.md#reader1-tables-and-keys-are-closed)); a default the file omits is the v2 epoch's, and one this grund cannot execute yet is refused at the version key rather than replaced, so the v1 extension list never stands in for the epoch language set ([§FS-config-v2.rollout](FS-config-v2.md#rollout-what-this-grund-executes-and-what-it-refuses)); a positive citation `default`, which v1 loads and which obliges nothing, is refused rather than given that meaning or another ([§FS-config-v2.rules.citations.default](FS-config-v2.md#rulescitationsdefault-a-default-may-forbid-never-oblige)); and a clause the format defines but this grund does not execute is refused at its line and never falls back to a v1 reading ([§FS-config-v2.rollout](FS-config-v2.md#rollout-what-this-grund-executes-and-what-it-refuses)). A v1 state v2 does not spell stays reachable through a v1 file alone ([§FS-config-v2.mapping](FS-config-v2.md#mapping-what-the-records-keep-apart)). Serves [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path): the version a file writes is the meaning it gets.

### requirements.strengths: Each constraint takes the strengths it can mean from one vocabulary, and refuses the rest

Every constraint draws its strength from one closed set, and the strength moves only the channel, never the finding's code or message ([§FS-config-v2.rules.strengths](FS-config-v2.md#rulesstrengths-one-strength-vocabulary-two-severities), [§FS-config-v2.schema.measures](FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys), [§FS-fields.strengths](FS-fields.md#strengths-strengths-and-channels)). Each admits the strengths it can mean and refuses the others at their line rather than mapping one onto its neighbour: a prohibition only where a citation can be forbidden, and positive strengths alone on a measure, a ladder and `heading_depth` ([§FS-config-v2.rules.strengths](FS-config-v2.md#rulesstrengths-one-strength-vocabulary-two-severities), [§FS-config-v2.schema.1](FS-config-v2.md#schema1-schema)); `must` alone on whether a note carries text ([§FS-config-v2.schema.measures](FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys)); `may` or a prohibition alone as a citation `default` ([§FS-config-v2.rules.citations.default](FS-config-v2.md#rulescitationsdefault-a-default-may-forbid-never-oblige)); and `must` or `warn` alone for resolution, since a missing snapshot is never a suggestion ([§FS-config-v2.rules.resolution](FS-config-v2.md#rulesresolution-rulesresolution)). Every constraint the format defines holds to this, and one added to the format must as well: it takes its strengths from this set, says which it admits, and refuses the others at their line. Serves [§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable), whose repository adopts a constraint as a warning and promotes it to an error without changing format.

### requirements.ahead: What the format defines and this grund refuses is owed to its owners

This requirement is deferred: what the format defines and this grund refuses is [§FS-config-v2.rollout](FS-config-v2.md#rollout-what-this-grund-executes-and-what-it-refuses). Serves [§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable).

## reader: The reader

Every refusal in this chapter fails the load at its line, reported and exiting as any invalid config does ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)).

### reader.1: Tables and keys are closed

The v2 reader admits exactly the tables and keys this file names, at the scope each is named at, and refuses anything else at its line. An unknown table is ``unknown config section `<table>` ``. An unknown key, or a key written at a table that does not admit it, is ``unknown key `<key>` in [<table>]``, or ``unknown key `<key>` at the top level`` in the envelope. A key repeated inside one table is ``duplicate key `<key>` in [<table>] (first written at line <n>)``. v1's spellings are not v2's: every v1 table but `[workspace]`, which both versions read ([§FS-config.concerns](FS-config.md#concerns-every-key-belongs-to-exactly-one-concern)), is unknown in a v2 file, and so is every discarded alias, such as `strict`, `named_sections` and `include` ([§DISC-core-concerns.7.4](../discussions/proposals/2026-09-30-core-concerns.md#74-what-the-format-leaves-out)). An unknown key and a v1 table then say what to write instead, appended after that text, which stays whole ([§FS-errors.3](FS-errors.md#3-message-text)).

#### reader.1.keys: An unknown key names the keys its table admits

An unknown key's refusal appends, after one space, the keys its table admits in the reader's `(expected …)` shape, in the order the point that specifies the table lists them: the envelope's `grund_config_version` and `project_name` in [§FS-config.concerns](FS-config.md#concerns-every-key-belongs-to-exactly-one-concern)' order, and `[workspace]`'s `members`, `optional_members` and `include_root` in [§FS-config.3.8](FS-config.md#38-workspace--sub-project-members)'s. Two keys are joined by `or`, and three or more by commas with `, or` before the last, as the reader's other `(expected …)` lists are. The list is the table's admitted keys and nothing else, so every key it names is one that table reads, and a v1 key gets the list of the v2 table it is written at, not its own v2 spelling, which is [§FS-config-migrate.mapping](FS-config-migrate.md#mapping-every-version-1-key-and-default)'s:

```text
error: grund.toml:4: unknown key `folder` in [schema.kinds.FS] (expected files, folders, citable, scan, index, id_format, fetch, form, or closed)
```

Two kinds of table append something else. A table that holds only tables, `[schema.notes]`, `[schema.leads]`, `[schema.kinds]`, `[schema.kinds.<NAME>.fields]`, `[schema.kinds.<NAME>.fields.front-matter.keys]`, `[schema.sources.definitions]`, `[rules]` or `[presentation.kinds]` written as a header of its own ([§FS-config-v2.reader.2](FS-config-v2.md#reader2-every-row-and-field-is-written-under-its-own-header)), appends `(it holds only tables)`. A table whose keys are names says what a key names: `[schema.sources.extensions]` appends `(expected a suffix)` ([§FS-config-v2.schema.sources.extensions](FS-config-v2.md#schemasourcesextensions-schemasourcesextensions-maps-a-suffix-to-a-language)), and `[rules.resolution]` appends `(expected a kind name)` ([§FS-config-v2.rules.resolution](FS-config-v2.md#rulesresolution-rulesresolution)). A key either table reads as a name keeps that table's own refusals.

#### reader.1.v1-tables: A v1 table names the v2 tables its settings are written in

A v1 table is refused at its header, before any key under it is read ([§FS-config-v2.reader.4](FS-config-v2.md#reader4-order-and-location-are-kept)), so its refusal names every v2 table that [§FS-config-migrate.mapping](FS-config-migrate.md#mapping-every-version-1-key-and-default) writes one of its keys into. It appends `(a v1 table; in v2: <tables>)`, the tables joined by commas in this order:

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

`[[kinds]]` names only the row each entry becomes, and leaves a row's `title`, `resolve` and grounding to [§FS-config-migrate.mapping](FS-config-migrate.md#mapping-every-version-1-key-and-default). Every other unknown table, such as `[schema.leads.lines]`, `[project]` or another array of tables, gets no such hint: it is not a v1 spelling, so there is no home to name.

### reader.2: Every row and field is written under its own header

A keyed row is written under its own explicit header, `[schema.kinds.<NAME>]`, and a field under `[schema.kinds.<NAME>.fields.<field>]`, so every key names its owner. An explicit header that is written twice is ``repeated table `[<table>]` (first written at line <n>)``, located at the second. A parent that TOML creates implicitly, as `[schema]` is created by writing `[schema.sources]`, is not an explicit header: writing `[schema]` after it is not a repetition.

### reader.3: One setting, one form

A setting has one form. Where the format spells a constraint as a table whose keys are strengths, a strength key is never also a table: `[rules.citations.FS.must]` beside `[rules.citations.FS] must = [...]`, or `[schema.notes.lines.must]`, is `` `must` is a strength key in [<table>], not a table ``. A value of the wrong TOML type is refused at its line with the type it expected.

### reader.4: Order and location are kept

The reader keeps the line every value was written at, so the one validation pass after reading anchors an error where the value was written ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)). It keeps the authored order of the explicit headers too: rows are listed, rendered and reported in the order the file writes them, as `[[kinds]]` rows are in v1.

#### reader.4.order: Which refusal a file with several is refused for

A version-2 file that holds more than one refusal is refused for the first of these steps that refuses:

1. The reading: the first refusal met top to bottom. A check across one table's keys, such as two overlapping lists, is made where the table ends.
2. The whole file: what a row's keys say together, a row against another row, and a table against the row it names, at the earliest line.
3. Each row's `form` prerequisites ([§FS-fields.forms](FS-fields.md#forms-form-lowers-to-the-version-1-contracts)), row by row: a rule home read by a `data` language or by none ([§FS-config-v2.schema.sources.rules](FS-config-v2.md#schemasourcesrules-every-home-of-a-rule-kind-is-judged-by-its-reading)) and a value row's first home among them.
4. The meanings both versions share: the `[reference]` settings, the kind table, the ID grammar, the workspace member lists, and last the citation tables, in the order [§FS-config.3.9.5](FS-config.md#395-validation) gives.
5. The pinned and `*/` citation targets, where the run judges them ([§FS-config.3.9.5](FS-config.md#395-validation)), and in a workspace anything else its expansion refuses.
6. The six that ask how the configuration reads a name, `config-md-not-markdown`, `config-index-not-markdown`, `config-rule-file-not-markdown`, `config-rule-folder-not-markdown`, `config-value-home-absent` and `config-value-home-invalid` ([§FS-config-v2.checks.config-md-not-markdown](FS-config-v2.md#checksconfig-md-not-markdown-md-is-mapped-away-from-markdown) and the sections after it): the one at the earliest line, and at one line the entry written first in its key ([§DF-markdown-homes-refused-in-1-0.2.6](../decisions/functional/DF-markdown-homes-refused-in-1-0.md#26-which-refusal-a-file-with-several-gets)).

### reader.5: An array may span lines

An array value may be written across lines; a string in it may not ([§FS-config-v2.reader.5.strings](FS-config-v2.md#reader5strings-a-string-closes-on-the-line-it-opens-on)). It opens on its key's line, and every line after it, with its `#` comment taken off, belongs to it until its `[` closes outside a string, so no line inside it is read as a header or a key. It is one value, read and located at its key's line ([§FS-config-v2.reader.4](FS-config-v2.md#reader4-order-and-location-are-kept)), and once the one trailing comma it may end in comes off ([§FS-config-v2.reader.5.commas](FS-config-v2.md#reader5commas-an-array-takes-one-trailing-comma-after-an-element)), its elements are read as a one-line list's are, each ending only at a comma outside a string ([§FS-config.3.11](FS-config.md#311-a-list-element-ends-at-a-comma-outside-a-string)). One still open at the end of the file is refused at its key's line as `unclosed array`. Version 1 reads no array across lines ([§FS-config.3](FS-config.md#3-keys-version-1)).

#### reader.5.strings: A string closes on the line it opens on

A string in an array closes on the line it opens on, as a TOML basic string does, so a line break is never part of a value. A line of an array that ends inside a string is an *open line*, and the first open line is the array's last: the array is still open there, so it is refused at its key's line as `unclosed array`, with a parenthetical naming the open line, the line the author has to edit ([§DF-v2-string-closes-on-its-line.2.1](../decisions/functional/DF-v2-string-closes-on-its-line.md#21-a-string-closes-on-the-line-it-opens-on)). A string is open at a line's end when the `"` that opened it has no closing `"` after it on that line, a string closing where [§FS-config-v2.reader.6](FS-config-v2.md#reader6-a-string-ends-at-its-first-unescaped-quote) closes it. Three quotes in a row that open a string open a multi-line string, read as TOML reads one: it closes only at the next run of three or more quotes, none part of a `\"` escape, the run taken whole. Version 2 does not read a multi-line string, so where the string open at the end of the open line opened with `"""`, the parenthetical names a multi-line string instead ([§DF-v2-string-closes-on-its-line.2.4](../decisions/functional/DF-v2-string-closes-on-its-line.md#24-two-codes-because-a--string-is-toml)). A line's `#` comment is found reading its quotes one at a time, as [§FS-config.3.11](FS-config.md#311-a-list-element-ends-at-a-comma-outside-a-string) closes a string, so on a line that holds a `"""` run the comment can change which refusal the file gets and the line it names, never whether it loads.

Where the readers before 0.22.0 would have read the array as a list of strings, the refusal ends with the release that made it one ([§FS-distribution.4.2](FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)):

```text
error: grund.toml:10: unclosed array (a string is still open at the end of line 11); this became an error in grund 0.22.0
error: grund.toml:10: unclosed array (a multi-line string opens on line 11, which version 2 does not read); this became an error in grund 0.22.0
```

That is a property of the array's text, not of the key that holds it, and their reading decides it, end to end: quotes read one at a time, each line break read as one space, the whole run of adjacent commas before the `]` taken off, and each element read from its first quote to its last with every quote between kept ([§DF-v2-open-line-closes-in-0-22](../decisions/functional/DF-v2-open-line-closes-in-0-22.md#df-v2-open-line-closes-in-0-22-the-version-2-open-line-window-closes-in-0220)). An array it does not read as a list of strings, such as `exclude = ["a,b]` as the last line of the file, or one with an element that is not a string, keeps the bare refusal. An array with an open line is refused for it before any of its elements is read, so a string in it that closes early ([§FS-config-v2.reader.6](FS-config-v2.md#reader6-a-string-ends-at-its-first-unescaped-quote)) or a comma in it with no element before it ([§FS-config-v2.reader.5.commas](FS-config-v2.md#reader5commas-an-array-takes-one-trailing-comma-after-an-element)) earns no refusal of its own.

#### reader.5.commas: An array takes one trailing comma, after an element

An array, on one line or written across lines, takes one trailing comma before its `]`, and only after an element ([§DF-v2-array-one-trailing-comma.2.1](../decisions/functional/DF-v2-array-one-trailing-comma.md#21-an-array-takes-one-trailing-comma-after-an-element)). A *comma with no element before it* is one whose text before it, whitespace and comments skipped, ends in `[` or `,`. A comma before the `]` that is not one comes off, and one that is stays where it is written, so the element reader meets an empty element there and refuses the array at its key's line as `expected string`, as it refuses an empty element anywhere in a list ([§FS-config.3.11](FS-config.md#311-a-list-element-ends-at-a-comma-outside-a-string)). So `["ab",]` and `[` / `"ab"` / `,` / `]` hold the one element `ab`, `[]` and `[` / `]` hold none, and `["ab",,]`, `[,]`, `[,"a"]` and `["a",,"b"]` are refused, as TOML refuses them. Version 1 takes no trailing comma at all ([§FS-config.3.11](FS-config.md#311-a-list-element-ends-at-a-comma-outside-a-string)).

The refusal ends `; this became an error in grund 0.22.0` ([§FS-distribution.4.2](FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)) where the readers before 0.22.0, which took off the whole run of adjacent commas before the `]`, would have read the array as a list of strings ([§FS-config-v2.reader.5.strings](FS-config-v2.md#reader5strings-a-string-closes-on-the-line-it-opens-on)): `["ab",,]` and `[,]` carry it, and so does `languages = ["nope",,]`, though those readers refused that key for its language. Where taking off the run still leaves an empty element, as in `["ab", ,]` or `[,"a"]`, or leaves an element that is not a string, as in `[1,,]`, the refusal is the bare `expected string`.

### reader.6: A string ends at its first unescaped quote

A basic string the version-2 reader reads, a scalar value, an element of an array or a quoted key, opens at its `"` and closes at the first `"` after it that is not part of a `\"` escape, by the one rule [§FS-config.3.11](FS-config.md#311-a-list-element-ends-at-a-comma-outside-a-string) reads strings by, in which a `\\` is an escape of its own, so `"a\\"` closes at its last quote. After its closing quote a scalar or a key holds nothing but whitespace, its `#` comment already taken off, and an element nothing but whitespace before the `,` or `]` that follows it. A string *closes early* when anything else follows its closing quote, as in `fetch = "one" "two"`, `folders = ["docs" "specs"]` with its comma forgotten, or `fetch = "a"b"` with a quote left unescaped, each of which TOML refuses too ([§DF-v2-string-closes-at-its-quote.2.1](../decisions/functional/DF-v2-string-closes-at-its-quote.md#21-a-version-2-string-ends-at-its-first-unescaped-quote)). Such a value is refused once, at its key's line, as `expected string`, for its first string that closes early.

Version 2 reads basic strings only, as version 1 does ([§FS-config.3](FS-config.md#3-keys-version-1)). A string that opens with `"""` and closes on its own line, such as `title = """one"""`, is a TOML multi-line basic string, which closes early ([§DF-v2-string-closes-at-its-quote.2.4](../decisions/functional/DF-v2-string-closes-at-its-quote.md#24-a--string-on-one-line-is-a-multi-line-string)); a value or an element that opens with `'''` is a multi-line literal string, and any other that opens with `'` a literal string. Version 2 reads none of these: each is refused at its key's line as `expected string`, with a parenthetical naming its form, so that a value that is visibly a string is not taken for one whose contents are at fault, and a literal string's refusal names the remedy too. Where the readers before 0.22.0, which read a string from the first quote of its value or its element to the last with every quote between kept, would have read the value as strings, whatever its key then made of them ([§FS-config-v2.reader.5.strings](FS-config-v2.md#reader5strings-a-string-closes-on-the-line-it-opens-on)), the refusal ends with the release that made it one ([§FS-distribution.4.2](FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)):

```text
error: grund.toml:8: expected string; this became an error in grund 0.22.0
error: grund.toml:10: expected string (a multi-line string, which version 2 does not read); this became an error in grund 0.22.0
error: grund.toml:2: expected string (a literal string, which version 2 does not read); write it as a basic string, "...", with \" for a quote inside it
error: grund.toml:2: expected string (a multi-line literal string, which version 2 does not read); write it as a basic string, "...", with \" for a quote inside it
```

A value those readers would not have read as strings keeps the bare `expected string`, such as `exclude = ["target" "a,b"]`, whose second piece they cut at its comma, and so does a value malformed in any other way, such as `fetch = true` where a string is expected. A `"""` string that a line break runs through is refused as an open line instead ([§FS-config-v2.reader.5.strings](FS-config-v2.md#reader5strings-a-string-closes-on-the-line-it-opens-on)). Version 1 keeps its own reading for good ([§FS-config.5.2](FS-config.md#52-a-versions-meaning-moves-only-as-backwards-compatibility-allows)) and meets this rule only when it is migrated ([§FS-config-migrate.3.1.8](FS-config-migrate.md#318-a-string-that-closes-early)). The readers that read one key without loading the config, an unloaded config's `project_name` ([§FS-declarations.checks.misplaced-declaration](FS-declarations.md#checksmisplaced-declaration-misplaced-declaration-configured-kind-home)) and the ancestor climb's `[workspace] members` ([§FS-workspace.6.1.7](FS-workspace.md#617-a-claiming-block-that-cannot-answer-fails-the-run)), pass over a value this rule refuses as over any they cannot read, and raise nothing of this rule's own ([§DF-v2-string-closes-at-its-quote.2.6](../decisions/functional/DF-v2-string-closes-at-its-quote.md#26-the-quick-readers-record-nothing)).

## defaults: The v2 epoch

A key a v2 file omits takes the value below. These are the v2 epoch, and no later binary changes them: a different default is a different version ([§FS-config.5.2](FS-config.md#52-a-versions-meaning-moves-only-as-backwards-compatibility-allows)).

| Setting | v2 default |
|---|---|
| ID format, project and row | `{kind}-{slug}` |
| Slug pattern | `[a-z][a-z0-9-]*`, letter-led |
| Citation recognition | marked only; there is no opt-out |
| Named sections | always enabled; there is no switch |
| `[schema] heading_depth` | `must` |
| `[schema.sources] respect_gitignore` | `true` |
| `[schema.sources] exclude` | `[]` |
| `[schema.sources] declarations` | `"doc-comments"` |
| The scan | the union of the scanned rows' places and the complement's extent, which is the config root unless `[schema.kinds.code]` lists places |
| `[rules.citations] default` | `may` |
| Languages | the epoch's default rows ([§FS-config-v2.schema.sources.epoch](FS-config-v2.md#schemasourcesepoch-the-v2-language-epoch)), never v1's extension list |

The marker, `shorthand`, `section_separator` and `number_pattern` keep their v1 defaults ([§FS-config.3.1](FS-config.md#31-reference--citation-form), [§FS-config.3.2](FS-config.md#32-id--id-grammar)). A v2 file whose `[schema.kinds]` holds no citable row has no kinds and still loads, declaring no IDs: v1's built-in kind list and its refusal of a table with no citable kind are v1's alone.

## schema: The schema concern

### schema.1: `[schema]`

`[schema]` admits `marker`, `shorthand`, `id_format`, `slug_pattern`, `number_pattern`, `section_separator` and `heading_depth`. All but `heading_depth` mean what their v1 counterparts mean ([§FS-config.3.1](FS-config.md#31-reference--citation-form), [§FS-config.3.2](FS-config.md#32-id--id-grammar)), under the v2 defaults. `heading_depth` takes a strength ([§FS-config-v2.rules.strengths](FS-config-v2.md#rulesstrengths-one-strength-vocabulary-two-severities)) over the section-heading-level check of [§FS-declarations.checks.section-heading-level](FS-declarations.md#checkssection-heading-level-section-heading-level-mismatch), and `may` recognizes sections without checking their depth.

### schema.places: Rows and their places

A row is `[schema.kinds.<NAME>]`. It admits `files` and `folders`, lists of paths from the config root that are its places; `citable`, default `true`; `scan`, default `true`; `index`; `id_format`; `fetch`; `form`; and `closed`, with its `fields` tables under it ([§FS-config-v2.schema.fields](FS-config-v2.md#schemafields-fields-and-forms)). They mean what the v1 row keys of [§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds) mean, written as lists, with v1's row `format` named `id_format`. A row may list any number of places under both keys, and a citable row with none is declared in doc-comments only. The scan is the union of every scanned row's places and the complement's extent ([§FS-config-v2.schema.places.complement](FS-config-v2.md#schemaplacescomplement-code-is-the-complement-by-name)); v2 keeps no second list of scan roots ([§FS-config-v2.reader.1](FS-config-v2.md#reader1-tables-and-keys-are-closed)). A row's places have one order, its `files` and then its `folders`, each as written, and every view that lists them lists them in it, so what `grund config show` prints loads back to the same rows.

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

This scans `docs`, the three listed files, both test folders and `crates`, and nothing else. `docs/spec/core/login.md` is `FS`'s, because the deepest place containing it is; `crates/grund/README.md` is `page`'s, because a listed file outranks every folder; and every other file under `crates` is `code`'s ([§FS-config-v2.schema.places.ownership](FS-config-v2.md#schemaplacesownership-a-listed-file-then-the-deepest-folder)). Decided in [§DF-v2-place-ownership](../decisions/functional/DF-v2-place-ownership.md#df-v2-place-ownership-in-v2-the-row-named-code-is-the-complement-and-a-file-belongs-to-its-listed-row-or-else-its-deepest-folder).

#### schema.places.complement: `code` is the complement, by name

The row named `code` is the complement: it owns every scanned file no other place claims, and it is the one row whose files may hold declarations of any kind but a single-file kind ([§FS-declarations.checks.misplaced-declaration](FS-declarations.md#checksmisplaced-declaration-misplaced-declaration-configured-kind-home)). It declares no IDs, so `citable = false` on it is accepted and `citable = true` refused at its line: `` `[schema.kinds.code]` is the complement and declares no IDs ``. Its places are its **extent**, and with no place, or no `code` row at all, its extent is the config root, so a v2 file narrows the scan only by writing `[schema.kinds.code] folders = [...]`; a file outside every place is then read only by `--full` ([§FS-config-v2.schema.places.boundaries](FS-config-v2.md#schemaplacesboundaries-what-a-place-may-name)). `[rules.citations.code]` governs the complement's files whether or not the row is written. A scoped `AGENTS.md` that `grund init` generates is not a unit of the place it sits in, and a row that lists it in `files` claims it like any other file ([§FS-init.2.3.5](FS-init.md#235-citation-directions)). A non-citable row other than `code` that has no place is refused at its header: `` `[schema.kinds.<NAME>]` has no place: give it `folders` or `files`, or name the complement `code` ``.

#### schema.places.ownership: A listed file, then the deepest folder

Every scanned file belongs to exactly one row: a file listed in a row's `files` to that row, any other to the row whose folder containing it is deepest, the complement's folders included, and a file in no place to the complement. No tie can arise, because two equal places are refused ([§FS-config-v2.schema.places.boundaries](FS-config-v2.md#schemaplacesboundaries-what-a-place-may-name)). Two nested folders of one row are both that row's, and the deepest one owns the file. Each path the walk meets has its own owner, so one physical file reached through two places answers to both, each at its own path ([§FS-scan.identity.findings.place](FS-scan.md#identityfindingsplace-a-place-finding-is-judged-at-each-spelling)).

Every per-file rule asks this one question. A file is held to its owner's citation directions and ladder ([§FS-config-v2.rules.citations](FS-config-v2.md#rulescitations-rulescitations), [§FS-config-v2.rules.grounding](FS-config-v2.md#rulesgrounding-grounding-is-a-ladder-replaced-whole)), and a finding that names a non-citable row names the place that owns the file, such as `tests/integration/ must cite FS`. A declaration is judged misplaced against its file's owner ([§FS-declarations.checks.misplaced-declaration](FS-declarations.md#checksmisplaced-declaration-misplaced-declaration-configured-kind-home)). Each folder of a citable row has its own index, the row's `index` name resolved in that folder, which lists the declarations that folder owns; a declaration in a nested place of another row is listed in that row's index, and a `files` place is not indexed ([§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)). A stub for a source declaration may sit in any folder of its kind, and one is enough ([§FS-declarations.stubs](FS-declarations.md#stubs-what-a-stub-points-at)).

`grund schema` and the Project map list every place of a row ([§FS-schema.4](FS-schema.md#4-nodes), [§FS-init.2.3.4](FS-init.md#234-managed-block-content-sections)), `grund id` every folder ([§FS-id.2.2](FS-id.md#22---format-json)), and `grund config show` prints one normalized `files` and one `folders` list per row, a place that does not exist included, with no `citable = false` on `code` and no row for an implicit complement.

#### schema.places.boundaries: What a place may name

A place is judged when the config loads, and each refusal here is a load error at the entry it names ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)).

- Places are compared after lexical normalization, byte for byte, so `./docs`, `docs/` and `docs` are one place.
- A place that is absolute or climbs out of the config root with `..` is refused ([§REQ-no-data-loss.2](../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own)): ``place `<p>` in `[schema.kinds.<NAME>] <key>` is outside the config root``, and so is one that normalizes to the config root itself, such as `.` or `docs/..`: ``place `<p>` in `[schema.kinds.<NAME>] <key>` names the config root, which is the complement's extent``.
- One place written twice, on two rows or on one, `files` against `folders` included, is refused at the later entry, `<p>` as that entry writes it: ``place `<p>` in `[schema.kinds.<NAME>] <key>` is already a place of `[schema.kinds.<OTHER>]` (line <n>); a place belongs to one row``. So are two places whose resolved targets coincide, such as two spellings that differ only in case on a case-insensitive filesystem, because the walk keeps one spelling of a file ([§DF-symlink-scan.2.3](../decisions/functional/DF-symlink-scan.md#23-the-report-names-the-link-never-the-target)) and the other place would be silently empty; two places are one only when both targets resolve.
- Ownership is decided on the place as written, not its link target ([§FS-config.3.4.11](FS-config.md#3411-what-a-home-is-used-for)). A `files` place that is a link is followed, as `CLAUDE.md -> AGENTS.md` is, and a folder place that is a directory link leaving the project root is refused ([§DF-symlink-scan.2.2](../decisions/functional/DF-symlink-scan.md#22-the-canonical-project-root-fences-directory-links-not-file-links-or-scan-paths)): ``place `<p>` in `[schema.kinds.<NAME>] folders` links out of the project root``. A folder place whose lookup fails for a reason other than absence is judged as far as the disk answers ([§FS-scan.scope.homes.lookup](FS-scan.md#scopehomeslookup-a-root-that-cannot-be-looked-up-is-a-scan-failure-not-an-absent-root)), whether or not its target exists: refused with that line if it has left the project root by then, and otherwise the scan failure of [§FS-scan.scope.homes.lookup](FS-scan.md#scopehomeslookup-a-root-that-cannot-be-looked-up-is-a-scan-failure-not-an-absent-root) ([§DF-outward-place-refused-when-denied](../decisions/functional/DF-outward-place-refused-when-denied.md#df-outward-place-refused-when-denied-a-version-2-folder-place-that-links-out-of-the-project-is-refused-whether-or-not-its-target-can-be-looked-up-and-exits-move-with-it)).
- A place that does not exist, a dangling link included wherever it points, scans nothing and earns no finding; a citation of a kind none of whose places exists says so at the citer ([§FS-check.3.1](FS-check.md#31-dangling-citation)).
- A place at or inside a workspace member's root is refused, because the root never enters a member ([§FS-workspace.6](FS-workspace.md#6-nested-project-boundary)): ``place `<p>` in `[schema.kinds.<NAME>] <key>` is inside workspace member `<m>`, which the root never reads``.
- `scan = false` covers every place of its row. A scanned place under an unscanned one is walked, and an unscanned place under a scanned one is pruned. On `code` it is refused at its line: `` `[schema.kinds.code] scan = false` cannot unwalk the complement; narrow its places or use `[schema.sources] exclude` ``.
- `grund check --full` reads the whole config root past every place and past `scan = false` ([§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only)); a file no place claims is the complement's and is reported in the outside-the-scope tier, and a member's root stays out. An explicit path argument reads what it names, an unscanned place included, and its files have the owners a whole run gives them.

### schema.sources: `[schema.sources]`

`[schema.sources]` admits `languages`, `declarations`, `exclude`, `respect_gitignore` and `fenced`, and the tables `[schema.sources.extensions]` and `[schema.sources.definitions.<name>]`. They say which files a v2 project reads, with which reader and comment hosts, and which comments may declare an ID; a file's reader, hosts and declaring comments are read from the configuration and the file's own name, never from a kind row's setting or the file's contents ([§GOAL-polyglot-citation](../goals.md#goal-polyglot-citation-ids-cite-cleanly-from-anywhere-they-are-useful)). A file's **language** is the one its final extension maps to: an entry of `[schema.sources.extensions]` first, then the epoch suffix of an admitted language ([§FS-config-v2.schema.sources.epoch](FS-config-v2.md#schemasourcesepoch-the-v2-language-epoch)). A file whose extension maps to no admitted language is not read ([§FS-scan.scope.extensions](FS-scan.md#scopeextensions-an-extension-makes-a-file-readable-not-declarable)), except as a value source ([§FS-config-v2.schema.sources.values](FS-config-v2.md#schemasourcesvalues-a-value-home-reads-a-file-by-its-language)).

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

`languages` lists language names, built-in or defined, and it alone admits a language: neither a definition nor an extension mapping does. Omitted, it is the epoch's **default** set, in the epoch's order ([§FS-config-v2.schema.sources.epoch](FS-config-v2.md#schemasourcesepoch-the-v2-language-epoch)); written, it replaces that set whole, and `[]` admits nothing, so no file is read ([§FS-config.principle.unit](FS-config.md#principleunit-one-leaf-key-overrides-and-one-value-is-whole)). A name neither built-in nor defined is refused at the `languages` line: ``unknown language `<name>` in [schema.sources] languages``, and so is a name written twice: ``language `<name>` is listed twice in [schema.sources] languages``.

#### schema.sources.epoch: The v2 language epoch

These are the built-in languages of v2. A suffix is matched case-sensitively and written without its dot. An omitted `languages` admits every **default** row, in this order, and an **opt-in** row is admitted only by naming it. Any other suffix, such as `h`, `jsx` or `kts`, is read only through `[schema.sources.extensions]`.

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

A language's comment hosts are the prefixes it reads a comment behind, each standing for the forms of [§FS-languages.prefixes.forms](FS-languages.md#prefixesforms-the-forms-a-prefix-stands-for), matched longest first. Its **doc profile** is the recognizer [§FS-config-v2.schema.sources.declarations](FS-config-v2.md#schemasourcesdeclarations-declarations-picks-which-comments-declare) asks which comments are documentation: a profile of [§FS-inline-citation-style.1.1.2](FS-inline-citation-style.md#112-languages) frozen at the epoch, admitting what [§FS-inline-citation-style.1.1.4](FS-inline-citation-style.md#114-declaration-applicability) fixes. A published row never changes: a language, a suffix or a profile that changes membership or a block boundary is a new opt-in row, a new profile name or a new config version ([§FS-config.5.2](FS-config.md#52-a-versions-meaning-moves-only-as-backwards-compatibility-allows)), so a later binary never widens an omitted `languages`.

#### schema.sources.extensions: `[schema.sources.extensions]` maps a suffix to a language

Each key is a suffix and each value a language name, and an entry overrides the epoch's assignment of that suffix: `h = "c"` reads `.h` files as C. A key is a nonempty literal final extension, with no dot, no path separator and no glob character among `*`, `?`, `[`, `]`, `{`, `}` or `!`, refused at its line otherwise: ``invalid suffix `<key>` in [schema.sources.extensions] (expected a final extension with no dot, separator or glob)``. A value names a language `languages` admits, refused at the entry's line otherwise: ``unknown language `<name>` in [schema.sources.extensions]`` for an unknown name, and ``[schema.sources.extensions] <key> maps to `<name>`, which [schema.sources] languages does not admit`` for a known one.

A mapped file takes the target language whole, and a mapping selects no part of it: its reader, comment hosts, doc profile, docstring reading and, for a built-in, its multi-line string row and quote reading ([§FS-languages.multi-line-string.suffix](FS-languages.md#multi-line-stringsuffix-which-row-and-which-quote-reading-a-file-reads)). `.md` is always Markdown: an `md` entry that names a language not of class `document` is refused ([§FS-config-v2.checks.config-md-not-markdown](FS-config-v2.md#checksconfig-md-not-markdown-md-is-mapped-away-from-markdown)), so `md` may name `markdown`, `v1_markdown`, which migration writes, or a `document` definition, and no other suffix is bound this way. A mapping speaks for a file whose own name carries the suffix, not for a name that reaches one: `txt = "rust"` reads `real.txt` as Rust, and a link `a.txt -> real.md` as Markdown ([§FS-scan.identity.reading](FS-scan.md#identityreading-how-a-file-is-read-belongs-to-the-file-not-to-the-name-that-reached-it)).

#### schema.sources.definitions: `[schema.sources.definitions.<name>]` defines a language

A definition adds a language name for `languages` and `[schema.sources.extensions]` to use, and admits and maps nothing by itself, so one that no admitted mapping reaches loads and reads nothing. Its name matches `[a-z][a-z0-9_]*` and is no built-in's, refused at the header otherwise: ``invalid language name `<name>` (expected [a-z][a-z0-9_]*)`` and ``[schema.sources.definitions.<name>] cannot redefine the built-in language `<name>` ``. It admits four keys, and an unknown key is refused at its line ([§FS-config-v2.reader.1](FS-config-v2.md#reader1-tables-and-keys-are-closed)):

| Key | Values | Default |
|---|---|---|
| `class` | `document`, `data`, `source` | required |
| `comment_prefixes` | literal, nonempty, single-line strings, each once | `[]` |
| `doc_comments` | `none`, `c-family`, `python`, `go`, `ruby`, `shell`, `sql`, `ldoc`, `haddock`, `roxygen`, `lisp` | `none` |
| `docstrings` | `off`, `python` | `off` |

A definition without `class` is refused at its header: ``[schema.sources.definitions.<name>] needs `class` ``. A value outside its column is refused at its line, naming the values it expected, and so is each of these combinations, at the line of the key that breaks it:

- A `document` definition is read by the Markdown reader and a `data` one by the JSON value-file reader, never the YAML one ([§FS-values.2.2.3](FS-values.md#223-how-a-yaml-source-is-read)). Neither takes a doc profile or docstrings other than `none` and `off`, and a `data` one takes no comment prefix, while a `document` one may keep prefixes for a fenced example's reading: `` [schema.sources.definitions.<name>] class = "<class>" takes no <key> ``.
- A `source` definition with no prefix and no `docstrings = "python"` is refused at its header: ``[schema.sources.definitions.<name>] reads no comment: give it comment_prefixes or docstrings = "python"``.
- `doc_comments = "python"` needs `docstrings = "python"`, and a marker profile no configured prefix reaches — `c-family` without `//` or `/*`, `ldoc`, `haddock` or `sql` without `--`, `roxygen`, `ruby` or `shell` without `#`, `go` without `//`, `lisp` without `;` — is refused at the `doc_comments` line: `` doc_comments = "<profile>" is unreachable: no comment prefix of [schema.sources.definitions.<name>] spells it ``.
- A prefix written twice is refused at the `comment_prefixes` line.

A prefix is literal: matched byte for byte at the first non-blank column, the longest first, and standing for nothing beyond itself; v2 has no global prefix list. `lisp` is v1's Lisp position recognizer, frozen ([§FS-inline-citation-style.1.1.2](FS-inline-citation-style.md#112-languages)). A definition carries no multi-line string row and no quote reading, and no key spells either: a `source` definition's files read those their own final extension names and cannot opt out, so `java = "myjava"` reads Java text blocks and a `txt` file reads none ([§FS-languages.multi-line-string.suffix](FS-languages.md#multi-line-stringsuffix-which-row-and-which-quote-reading-a-file-reads), [§DF-defined-language-string-forms](../decisions/functional/DF-defined-language-string-forms.md#df-defined-language-string-forms-a-file-read-through-a-source-definition-reads-the-multi-line-strings-and-quotes-its-own-suffix-names)), and a `document` or `data` definition's files read none.

#### schema.sources.values: A value home reads a file by its language

The value home decides which files are asked: the `file` home, or each direct child of a `folder` home ([§FS-values.2.2](FS-values.md#22-declarations-from-value-sources)). A file whose name maps to an admitted language is a value source exactly when that language's class is `data`, and is read with the JSON reader, so a `.jsonc` under `jsonc = "json"` is one and a `.json` mapped to a `source` or `document` language is not. A file whose name maps to no admitted language is one when its final extension is `json`, `yaml` or `yml`, read with the JSON reader for `json` and the YAML reader for the other two ([§FS-values.2.2.3](FS-values.md#223-how-a-yaml-source-is-read)), so a value home's `.json` or `.yaml` is read under `languages = ["markdown"]` too. A language never makes a file outside a value home a value source, and the text scan never reads a `data` language's files.

#### schema.sources.rules: Every home of a rule kind is judged by its reading

`form = "rule"` requires homes whose declarations are Markdown ([§FS-fields.forms](FS-fields.md#forms-form-lowers-to-the-version-1-contracts), [§FS-config.3.4.12](FS-config.md#3412-rules--rule-declaration-kinds)), and every entry of `files` and `folders` is judged by how the configuration reads it, never by its name, whatever the order of the entries ([§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule), [§DF-one-reading-answer](../decisions/functional/DF-one-reading-answer.md#df-one-reading-answer-whether-a-file-is-read-as-markdown-or-has-its-docstrings-read-is-the-configurations-one-answer-never-its-name-and-every-home-of-a-row-is-judged-by-it)):

- **A `files` entry read as Markdown** is a rule home whose rules are read and enforced, such as `rules/rules.txt` under `txt = "markdown"`.
- **A `files` entry read by a `data` language or by no admitted language** refuses the row at `form`, because nothing reads its declarations, even beside a Markdown entry ([§DF-v2-rule-home-by-language.2.1](../decisions/functional/DF-v2-rule-home-by-language.md#21-the-check-asks-the-readers-question)).
- **A `files` entry read by a `source` language** refuses the row at the `files` key ([§FS-config-v2.checks.config-rule-file-not-markdown](FS-config-v2.md#checksconfig-rule-file-not-markdown-a-rule-kinds-files-entry-is-read-as-source)).
- **A `folders` entry in which no suffix resolves to an admitted `document` language** refuses the row at the `folders` key ([§FS-config-v2.checks.config-rule-folder-not-markdown](FS-config-v2.md#checksconfig-rule-folder-not-markdown-a-rule-kinds-folder-can-hold-no-markdown)).
- **Any other `folders` entry** loads on its presence; a file the walk reads in it with a `source` language draws `rule-folder-file-not-markdown` ([§FS-config-v2.checks.rule-folder-file-not-markdown](FS-config-v2.md#checksrule-folder-file-not-markdown-a-file-in-a-rule-kinds-folder-is-read-as-source)).

Version 1 keeps its name test: a `file` home is Markdown when its final extension is `md` and a `folder` home is accepted on its presence, and neither the refusal of `config-rule-file-not-markdown` nor that of `config-rule-folder-not-markdown`, nor `rule-folder-file-not-markdown`, applies to it ([§FS-config.5.2](FS-config.md#52-a-versions-meaning-moves-only-as-backwards-compatibility-allows)).

#### schema.sources.declarations: `declarations` picks which comments declare

`declarations` takes `"doc-comments"`, the default, or `"any-comment"`, and any other value is refused at its line: ``unknown [schema.sources] declarations `<value>` (expected doc-comments or any-comment)``. A line in declaration position ([§FS-declarations.line](FS-declarations.md#line-what-a-line-in-declaration-position-declares)) in a source file declares only when its comment block is **eligible**. Under `"any-comment"` every block is; under `"doc-comments"` the file's language ([§FS-config-v2.schema.sources](FS-config-v2.md#schemasources-schemasources)) decides:

- a doc profile other than `none` makes a block eligible exactly when its recognizer classifies it as a doc comment, with the applicability of [§FS-inline-citation-style.1.1.4](FS-inline-citation-style.md#114-declaration-applicability);
- the profile `none` — Lisp, Scheme, Clojure and Ada in the epoch, and a definition with no `doc_comments` — makes every comment block behind the language's hosts eligible;
- `docstrings = "python"` makes each docstring eligible, and under the `python` profile `#` blocks are not.

A Markdown heading and a value file are not comments and are not asked, and a marked citation in any comment is recognized, resolved and checked in either mode. Decided in [§DF-declarations-in-doc-comments](../decisions/functional/DF-declarations-in-doc-comments.md#df-declarations-in-doc-comments-a-v2-source-declares-in-documentation-not-in-a-note).

```rust
/// FS-existing: Real documentation
fn example() {
    // FS-existing: Ordinary root-ID note
    // See §FS-existing.
}
```

Under `"any-comment"` this declares `FS-existing` twice and reports the duplicate; under `"doc-comments"` it declares it once, at line 1, and the ordinary comment's citation resolves to it.

Eligibility is decided once per block, from the language and the block alone, whether or not the block carries a citation or the inline-style budgets measured it, and before the canonical declaration recognizer, the off-grammar and legacy fallbacks ([§FS-config.3.2.5](FS-config.md#325-off-grammar-declarations-stay-readable)) and the near-miss check ([§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss)). So a declaration-shaped line in an ineligible block is prose: it declares nothing, reaches no catalog by a fallback, is no near miss and draws no finding of its own, while a marked citation on it draws every finding a citation does. Body ranges and sections ([§FS-show.2.3.1](FS-show.md#231-what-counts-as-the-comment-block)), anchors, stub verification ([§FS-declarations.stubs](FS-declarations.md#stubs-what-a-stub-points-at)), the formatter's protection of declaration lines and the editor all read the same answer. A section-suffixed token never declares its root ID in either mode ([§FS-declarations.line.section-suffix](FS-declarations.md#linesection-suffix-an-id-with-a-section-after-it-declares-nothing)).

#### schema.sources.exclude: `exclude` is a gitignore-style glob list

`exclude` is a list of patterns resolved against the config root and read as `.gitignore` lines are, the grammar of `[presentation.fmt] exclude` ([§FS-config.3.10](FS-config.md#310-fmt--suppressing-the-rewrite)), `#` comments and `!` negation included, the later line winning. Its default is `[]`: v1's built-in names `target`, `node_modules`, `.git`, `dist`, `build` and `.venv` are not v2 defaults. As in git, a negation never rescues a file below a directory already pruned: `["build/", "!build/keep.md"]` leaves `build/keep.md` unread. A pattern the parser refuses is refused at the `exclude` line.

`exclude` is a rule about descent, so it composes as v1's component names do ([§FS-scan.scope.roots](FS-scan.md#scoperoots-a-scan-root-outruns-every-rule-about-descent)):

1. A scan root, a row's place or a path handed to the command, is read even where a pattern names it; the hidden-file rule ([§FS-scan.scope.hidden](FS-scan.md#scopehidden-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent)) and language admission still apply to it.
2. Below a root, a path is read only if neither `exclude` nor an ignore file matches it. `respect_gitignore = false` turns the ignore files off and leaves `exclude` in force, and a `!` in one never overrides the other.
3. `--full` and an explicit path argument reach a row with `scan = false` ([§FS-config-v2.schema.places.boundaries](FS-config-v2.md#schemaplacesboundaries-what-a-place-may-name)), and `--full` still applies `exclude`, the ignore files and the hidden rules.
4. A path reached through a symlink is matched by the spelling the walk reached it under ([§FS-scan.scope.links.names](FS-scan.md#scopelinksnames-the-directory-rules-apply-under-the-link-name)). Workspace ownership, the directory-link fences, E2E fixture boundaries, loop refusals and the one-read-per-physical-file rule are decided first ([§FS-scan.identity.read-once](FS-scan.md#identityread-once-one-physical-file-is-read-once)), and `exclude` never admits what they keep out; it still keeps out what it matches, so a pattern that matches a looping directory link below a root keeps the loop out, with no loop reported ([§FS-check.2.4.3](FS-check.md#243-what-failed-and-where-decide-the-code-and-the-fix)).
5. A root outside the config root is matched by its lexical spelling relative to the config root, so an anchored pattern never reaches outside the root and an unanchored one still applies below it.

`[presentation.fmt] exclude` keeps a file from being rewritten while the scan still reads it; `[schema.sources] exclude` keeps the scan from reading it. A pattern is also the `exclude` reason a stub's target is not read ([§FS-declarations.stubs.target.3](FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing)), asked of each directory on the target's path and of the target itself, from each scan root the run walks and from the config root, before the ignore files.

#### schema.sources.fenced: `fenced` names the fences whose citations are read

`fenced` is a list of fence tags, `[]` by default. A Markdown fence whose tag the list holds is **selected** ([§FS-languages.fence.language](FS-languages.md#fencelanguage-a-fences-tag-and-the-fences-fenced-selects)): its citations are read as a raw-text HTML block's are ([§FS-citations.recognized.contexts](FS-citations.md#recognizedcontexts-contexts-read-as-neither-prose-nor-code)), and it stays a fence for everything else. Every other fence stays skipped ([§DF-fenced-citations](../decisions/functional/DF-fenced-citations.md#df-fenced-citations-a-markdown-fence-is-read-for-citations-only-when-schemasources-fenced-names-its-tag)).

```toml
grund_config_version = 2
[schema.sources]
fenced = ["mermaid"]
```

A tag matches `[A-Za-z0-9][A-Za-z0-9_+.#-]*`, so `c++` and `c#` are tags and `{.mermaid}` is not. A malformed tag is refused at the `fenced` line: ``invalid fence tag `<tag>` in [schema.sources] fenced (expected [A-Za-z0-9][A-Za-z0-9_+.#-]*)``, and so is a tag written twice, tags comparing case-sensitively as in a fence, so `mermaid` and `Mermaid` are two: ``fence tag `<tag>` is listed twice in [schema.sources] fenced``.

The list is one value of the project's own configuration ([§FS-config.principle.unit](FS-config.md#principleunit-one-leaf-key-overrides-and-one-value-is-whole)): no row and no command-line flag sets it, and a workspace member's fences are selected by its own configuration, never its root's. It applies to every file the Markdown reader reads, a `.md` file or one mapped to `markdown` or to a `document` definition, and to no other ([§FS-languages.fence.comment-block](FS-languages.md#fencecomment-block-a-fence-inside-a-comment-block-hides-its-headings)). It is independent of the languages: a tag need not name one, a language admitted or defined selects none of its fences, a tag admits no file, and a selected fence is read by the Markdown reader of the file it sits in, never by a definition that shares its tag.

#### schema.sources.effective: The admitted set is read from the configuration alone

The resolved source model, every admitted language with its class, suffixes, comment hosts and doc profile, and the `declarations`, `exclude`, `respect_gitignore` and `fenced` values, is computed from the configuration without reading a source file. `grund config show` prints it in v2 spelling ([§FS-config.requirements.6](FS-config.md#requirements6-the-effective-configuration-is-inspectable)): `languages` always, in full, so an omitted list prints as the epoch's default set; `declarations`, `exclude`, `respect_gitignore` and `fenced` always, in that order, an omitted `fenced` as `fenced = []` and a written one in written order; and each `[schema.sources.extensions]` entry and each definition with every key, defaults included, but no built-in row. The output loads back to the same model.

The embedding API's effective config ([§FS-distribution.3.3.1](FS-distribution.md#331-typed-immutable-results)) carries the model under a `sources` key, present only for a v2 project, and `sources` is the authority for a v2 file's hosts. A v2 snapshot also publishes the v1-shaped `scan` mapping:

| Key | Value |
|---|---|
| `sources.languages` | the admitted languages in `languages` order, each a mapping of `name`, `class`, `suffixes` (a built-in's epoch suffixes, `[]` for a definition), `comment_prefixes`, `doc_comments`, `docstrings` (`true` where the language reads Python docstrings) and `builtin` |
| `sources.extensions` | each `[schema.sources.extensions]` suffix to its language |
| `sources.declarations`, `sources.exclude` | those keys' values |
| `sources.fenced` | the tags in written order, `[]` where the key is omitted |
| `scan.comment_prefixes` | `[]` |
| `scan.docstring_python` | `false` |
| `scan.extensions` | every suffix the text scan reads: each mapped suffix whose language is not a `data` one, in authored order, then each epoch suffix of an admitted built-in other than `json` that no mapping takes |
| `scan.include` | `null` |
| `scan.exclude` | `[]` |
| `scan.respect_gitignore` | `[schema.sources] respect_gitignore` |

#### schema.sources.migration: What a v1 source configuration becomes

These are the rules `grund config migrate --to 2` applies to v1's `[scan]` source keys ([§FS-config-migrate](FS-config-migrate.md#fs-config-migrate-grund-config-migrate---to-2-shows-the-version-2-file-that-means-what-a-version-1-config-means-and-writes-it-only-when-asked-and-only-when-it-does)). Each writes its defaults out, so the v2 file keeps v1's meaning after any later edit to the tree, except on a block with `include_root = false` ([§FS-config-migrate.8.1](FS-config-migrate.md#81-a-block-with-include_root--false)).

1. `declarations = "any-comment"` is written.
2. `languages` is written explicitly, with one definition per distinct v1 reading: `v1_markdown` for `md`, `v1_<doc profile>` for a source extension with `-` written `_` (`v1_c_family`, `v1_python`, `v1_lisp`), and `v1_plain` for a source extension with no doc profile. Its class is `document` for `md` and `source` otherwise; it copies the **whole** v1 `comment_prefixes` list, a Markdown definition included, since v1 reads a Markdown fence through them, and the doc profile [§FS-inline-citation-style.1.1.2](FS-inline-citation-style.md#112-languages) gives those extensions; and `[schema.sources.extensions]` maps each v1 extension to it.
3. `.py` takes `docstrings = "python"` and the `python` profile when `docstring_python = true`, and `docstrings = "off"` with no Python profile when it is `false`, which keeps v1's raw-line reading of a triple-quoted string.
4. Each v1 `[scan] exclude` name becomes the escaped, directory-only glob `<name>/`, which matches that directory at any depth, as v1 did.

A `.json` extension scanned explicitly in v1 with no value authority becomes a `source` definition with its prefixes. Migration refuses, leaving the config's bytes unchanged, rather than approximate:

- a v1 `exclude` entry containing `/`, which never matched in v1 and as a glob would change what is scanned ([§DISC-core-concerns.8.3.1](../discussions/proposals/2026-09-30-core-concerns.md#831-amended-a-path-shaped-entry-is-refused-not-activated));
- `.json` scanned as text while a kind's `values` also reads it as a value source, or admitted only through a value home;
- a v1 extension or prefix outside the domains of [§FS-config-v2.schema.sources.extensions](FS-config-v2.md#schemasourcesextensions-schemasourcesextensions-maps-a-suffix-to-a-language) and [§FS-config-v2.schema.sources.definitions](FS-config-v2.md#schemasourcesdefinitions-schemasourcesdefinitionsname-defines-a-language), a source extension that would read no comment, and a reader v2 has no class for.

### schema.measures: Measures are tables, strengths are keys

A measured constraint is a table named for its measure, whose keys are strengths and whose values are thresholds:

| Table | Measures | Strength keys |
|---|---|---|
| `[schema.notes.lines]` | an inline note's lines | `must`, `warn`, `should`, `may` |
| `[schema.notes.columns]` | an inline note's columns | `must`, `warn`, `should`, `may` |
| `[schema.notes.text]` | whether a note carries text beside its citation | `must` |
| `[schema.notes.layout]` | an inline note's layout | `must`, `warn`, `should`, `may` |
| `[schema.leads.words]` | a declaration lead's words | `must`, `warn`, `should`, `may` |

The note measures are those of [§FS-inline-citation-style.2](FS-inline-citation-style.md#2-configuration) and the lead measure [§FS-declarations.checks.oversized-lead](FS-declarations.md#checksoversized-lead-oversized-lead-opt-in)'s; [§FS-config-migrate.mapping](FS-config-migrate.md#mapping-every-version-1-key-and-default) lists the v1 keys each table replaces. A stronger strength takes the looser threshold, and is refused at its key otherwise: ``threshold ordering: `must` (1) is stricter than `should` (3); a stronger strength takes the looser threshold``. Column findings use the over-limit line suffix of [§FS-inline-citation-style.4.1](FS-inline-citation-style.md#41-errors--hard-caps) on every reporting channel, filtered against the threshold selected for that finding.

### schema.fields: Fields and forms

`[schema.kinds.<NAME>.fields.<field>]`, with its keys `title`, `presence`, `position`, `after`, `content` and `one_of`, and a row's `form` and `closed`, are specified by [§FS-fields](FS-fields.md#fs-fields-a-v2-kind-states-the-shape-of-its-declarations-and-check-holds-every-declaration-to-it): what each key means, what the loader refuses, and the findings `grund check` reports.

## rules: The rules concern

### rules.strengths: One strength vocabulary, two severities

Every constraint takes one strength, and the strength names its channel:

| Strength | Prohibition | Channel |
|---|---|---|
| `must` | `must-not` | error, exit `1` |
| `warn` | `warn-not` | standing warning, exit `0` |
| `should` | `should-not` | suggestion, only under `--suggestions` |
| `may` | none | unchecked |

A finding keeps its code and its message whatever its strength, so promoting a constraint from `warn` to `must` changes only its channel. `warn` is not `should`: the two reach different channels, and the severities stay `{error, warning}`, with suggestions on their own channel ([§FS-config.6](FS-config.md#6-what-is-not-configured-here), [§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)). Only citations take a prohibition; a measure and a ladder take positive strengths only.

### rules.citations: `[rules.citations]`

`[rules.citations.<KIND>]` admits the list keys `must`, `warn`, `should`, `may`, `must-not`, `warn-not` and `should-not`, and the scalar `default`. Each list entry is a target or an any-of `A|B`, with v1's alias forms ([§FS-config.3.9.3](FS-config.md#393-alias-matching)). An obligation is asked of each declaration of the citing kind, and a prohibition fires at each citation site ([§FS-config.3.9.1](FS-config.md#391-levels)). Each refusal of the table names its line: an unknown or unwalked citing kind at the table's header, a target refused at the list that names it, bare or pinned, and an overlap between two lists ([§FS-config.3.9.5](FS-config.md#395-validation)) at the later list. Precedence is v1's: an explicit target, then the kind's `default`, then the project's `[rules.citations] default` ([§FS-config.3.9.4](FS-config.md#394-defaults-and-precedence)).

#### rules.citations.default: A default may forbid, never oblige

`default`, at the project or the kind, takes `may`, `should-not`, `warn-not` or `must-not`, and a positive default is refused at its line: `` `default = "must"` is not a v2 citation default (expected may, should-not, warn-not, or must-not) ``. A positive v1 `default` loads and obliges nothing ([§FS-config-v2.mapping](FS-config-v2.md#mapping-what-the-records-keep-apart)), and v2 refuses the form rather than give it a second meaning.

#### rules.citations.uncited: An obligation never judges a file that cites nothing

For a non-citable row or the complement an obligation is asked per file, and only of a file that carries a citation ([§FS-check.3.11](FS-check.md#311-missing-required-citation)): `[rules.citations.e2e] must = ["FS"]` fails a file in `tests/e2e/` that cites only `GOAL`, and passes one that cites nothing; whether a file cites at all is grounding's question, asked by the project's ladder or the row's ([§FS-config-v2.rules.grounding](FS-config-v2.md#rulesgrounding-grounding-is-a-ladder-replaced-whole)). Every text that says what `must`, `warn` or `should` does says so ([§DF-directions-render.6](../decisions/functional/DF-directions-render.md#6-a-home-whose-row-grounds-nothing-narrows-its-bullet)): the keys' descriptions `grund config explain` prints ([§FS-config-explain.3](FS-config-explain.md#3-a-keys-answer)), the generated entrypoint, per home ([§FS-init.2.3.5](FS-init.md#235-citation-directions)), and `grund schema`'s legend, once ([§FS-schema.8](FS-schema.md#8-the-table)). The description of `must` is:

```text
Kinds this one must cite; a miss is an error. A file of a place or the complement that cites nothing is not checked by this key; `[rules.citations.<ROW>.grounding]` makes it a finding.
```

and those of `warn` and `should` put the same second sentence after `Kinds this one must cite; a miss is a warning.` and `Kinds this one should cite; a miss is a suggestion.`

### rules.grounding: Grounding is a ladder, replaced whole

A scanned non-citable home closed to every citation ([§FS-config.3.9.4.1](FS-config.md#3941-a-row-closed-to-every-citation)) whose ladder asks for grounding still loads, and earns `closed-citation-grounding` ([§FS-check.checks.closed-citation-grounding](FS-check.md#checksclosed-citation-grounding-grounding-asks-for-citations-a-closed-row-permits-nowhere)). An empty `[rules.citations.<NAME>.grounding]` disables grounding for that row, replacing even a project ladder and overriding `--require-grounding`.

`[rules.citations.grounding]` is the project's ladder and `[rules.citations.<NAME>.grounding]` a row's. A ladder's keys are the positive strengths and its values are units: `"file"`, or `"h2"` to `"h6"`, the heading levels of [§FS-check.3.6.2](FS-check.md#362-the-unit). Each rung asks that every unit of that size in a governed file be grounded, and reports an ungrounded one on its strength's channel ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)); a row governs the files [§FS-check.3.6.1](FS-check.md#361-which-files-a-row-governs) says. A stronger rung never uses a finer unit, and is refused at its key otherwise: ``grounding ladder: `must` (h2) is finer than `warn` (file); a stronger rung never uses a finer unit``.

A ladder is one setting ([§FS-config.principle.unit](FS-config.md#principleunit-one-leaf-key-overrides-and-one-value-is-whole)): a row that writes one replaces the project's whole, and a rung the row leaves out is not inherited. `grund check --require-grounding` supplies the project ladder's `must` rung at `"file"` unless the project ladder already writes one; its other rungs remain, and a row's own ladder still replaces it whole ([§FS-config.principle.cli](FS-config.md#principlecli-a-cli-input-enters-at-the-scope-its-flag-spells)).

### rules.resolution: `[rules.resolution]`

`[rules.resolution]` maps a kind to `must` or `warn`: how a citation of an external kind that has no snapshot is reported ([§FS-check.4.12](FS-check.md#412-missing-snapshot)), `warn` as the missing-snapshot warning and `must` as the error. A kind with `fetch` defaults to `must`. An explicit entry for a kind whose row has no `fetch` is refused at its line: `` `[rules.resolution] <KIND>` needs `fetch` on [schema.kinds.<KIND>] ``. `should` and `may` are refused, because a missing snapshot is never a suggestion. Resolution never fetches ([§REQ-runs-offline.1](../requirements/REQ-runs-offline.md#1-read-and-verification-paths-execute-nothing)).

## presentation: The presentation concern

`[presentation]` admits `description`, `trigger`, `conversation` and `rules`; `[presentation.kinds.<NAME>]` admits `title`; and `[presentation.fmt]` admits `anchors`, `links` and `exclude`. `description`, `trigger`, `conversation` and a kind's `title` mean what `project_description`, `[reference] trigger`, `[reference] conversation` and a row's `title` mean in v1 ([§FS-config.3](FS-config.md#3-keys-version-1)), and `[presentation.fmt] exclude` is v1's `[fmt] exclude` ([§FS-config.3.10](FS-config.md#310-fmt--suppressing-the-rewrite)). `rules` says where each row's guidance is written: `"home"`, the default, into the `AGENTS.md` of the deepest directory holding the row's places ([§FS-init.2.3.5](FS-init.md#235-citation-directions)), and `"root"` every row into the root entrypoint, as a version-1 config renders ([§FS-init.2.3.5](FS-init.md#235-citation-directions)); any other value is refused at its line: `` unknown [presentation] rules `<value>` (expected home or root) ``. `links = "index"` runs the cross-reference pass of [§FS-fmt.6](FS-fmt.md#6-cross-reference-emission) over the files read as Markdown, and `anchors = "github"` slugs its anchors as [§FS-fmt.6.2.2](FS-fmt.md#622-the-anchor-slugifies-the-headings-rendered-text) says; any other value of either is refused ([§FS-config-v2.rollout.fmt-values](FS-config-v2.md#rolloutfmt-values-a-recognized-formatting-key-names-the-value-it-cannot-execute)).

## rollout: What this grund executes, and what it refuses

This grund executes every clause and every default the v2 format defines ([§FS-config-v2.defaults](FS-config-v2.md#defaults-the-v2-epoch)), except a value of `[presentation.fmt] anchors` other than `"github"` or of `links` other than `"index"` ([§FS-config-v2.rollout.fmt-values](FS-config-v2.md#rolloutfmt-values-a-recognized-formatting-key-names-the-value-it-cannot-execute)). A clause the format defines and this grund does not execute is never accepted and ignored, and never read with a v1 meaning: it is refused at its authored line with `` `<clause>` is part of the v2 format but not supported by this grund yet ``, where `<clause>` names the table and key, such as `[presentation.fmt] anchors`. An omitted default this grund does not execute is refused at the `grund_config_version` line, never replaced.

### rollout.fmt-values: A recognized formatting key names the value it cannot execute

For a string value of `[presentation.fmt] anchors` other than `"github"`, or of `links` other than `"index"`, the refusal keeps `` `[presentation.fmt] <key>` is part of the v2 format but not supported by this grund yet `` verbatim as a contiguous prefix and appends the supplied value, double-quoted, with quotes, backslashes and control characters escaped so the message stays on one line, and the only assignment this grund executes for the key:

```text
error: grund.toml:5: `[presentation.fmt] links` is part of the v2 format but not supported by this grund yet; unsupported value "all"; executes only links = "index"
```

The prefix is fixed and the suffix's prose is not ([§FS-errors.3](FS-errors.md#3-message-text), [§FS-errors.requirements.prefix-windows](FS-errors.md#requirementsprefix-windows-a-wording-change-keeps-the-old-message-whole-until-its-window-closes)). Invalid string syntax and a non-string value keep their parse and type errors.

## mapping: What the records keep apart

Both readers lower into the same records, so the records hold every state either one can mean, and a v1 state that v2 does not spell keeps its meaning. Migration ([§FS-config-migrate](FS-config-migrate.md#fs-config-migrate-grund-config-migrate---to-2-shows-the-version-2-file-that-means-what-a-version-1-config-means-and-writes-it-only-when-asked-and-only-when-it-does)) refuses each row below whose v2 spelling is `none`, except where a block with `include_root = false` gives the row no reader ([§FS-config-migrate.8.1](FS-config-migrate.md#81-a-block-with-include_root--false)). This table is what a migration audits against, and tests on the records hold it, not a comparison of one tree's reports.

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
| A file in two nested homes belongs to the complement | none: the deepest place owns it | the nesting rule |
| A complement renamed by a non-citable homeless row | none: the complement is `code`, titled by `[presentation.kinds.code] title` | the complement's row name |
| Every row's guidance renders in the root entrypoint | `[presentation] rules = "root"`, which a migrated file writes explicitly | the layout of the generated guidance, so a migrated config writes the files its v1 config wrote |
| `[scan] include`, separate from the homes | none: the scan is the places and the complement's extent | the derived scan roots |
| A row's `value_chapter` | a field, which adds a "Declaration shapes" section to the generated guidance; on a row that sets `fetch`, none ([§FS-config-migrate.3.1.7](FS-config-migrate.md#317-settings-this-grund-has-no-v2-spelling-for-yet)) | the chapter's value roots, apart from the field model |
| `lead_size_warning` counted in lines or bytes | `[schema.leads.words]` only | the lead budget and its unit |
| `[fmt.cross_refs] enabled = false`, or an `anchor_format` other than `"github"` | none in this grund ([§FS-config-v2.rollout](FS-config-v2.md#rollout-what-this-grund-executes-and-what-it-refuses)) | the formatter's link policy |
| An `include` entry that does not exist | none: a place is a file or a folder | the authored scan root, of no known type |
| A project `grounding_level` that only `--require-grounding` switches on | a rung of a ladder, which is always on | the level, apart from whether it is enforced |
| Multi-line strings and quotes are read by each file's extension | no key: the files of a `source` definition read the row and quote reading their own suffix names ([§FS-languages.multi-line-string.suffix](FS-languages.md#multi-line-stringsuffix-which-row-and-which-quote-reading-a-file-reads)) | the multi-line string and quote reading each suffix gets |

## checks: Checks

The check codes the version-2 reader's own findings carry, each with its row in the `check` code catalog ([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)). `rule-folder-file-not-markdown` is a finding of `check`, reported as every finding is ([§FS-check.2.1](FS-check.md#21-report-format)). Every other code here is spent: the construct its section describes is refused when the config loads, at the line that section gives, exits as every config refusal does ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)), and reaches the editor as a config that will not load ([§FS-lsp.2.2.4](FS-lsp.md#224-an-unusable-folder-is-skipped)). No selector removes a config failure ([§FS-check.2.1.2](FS-check.md#212-selection-filters-the-complete-report)), so on a tree that holds such a construct the refusal is what `--only` or `--ignore` with its code gets.

### checks.config-not-toml: A version-2 config holds text TOML refuses

`config-not-toml` names text TOML refuses: an array with a string still open at the end of a line, which a later line closes, is refused as `unclosed array` ([§FS-config-v2.reader.5.strings](FS-config-v2.md#reader5strings-a-string-closes-on-the-line-it-opens-on)), and a string that closes before the end of its value or its element, or an array with a comma before its `]` that has no element before it, as `expected string` ([§FS-config-v2.reader.6](FS-config-v2.md#reader6-a-string-ends-at-its-first-unescaped-quote), [§FS-config-v2.reader.5.commas](FS-config-v2.md#reader5commas-an-array-takes-one-trailing-comma-after-an-element)).

### checks.config-multiline-string: A version-2 config value holds a multi-line string

`config-multiline-string` names a string opened with `"""`, which is TOML but not what version 2 reads, and each is refused with a parenthetical that names the form: an array whose `"""` string a later line closes as `unclosed array` ([§FS-config-v2.reader.5.strings](FS-config-v2.md#reader5strings-a-string-closes-on-the-line-it-opens-on)), and a value, an element or a key whose `"""` string opens and closes on one line as `expected string` ([§FS-config-v2.reader.6](FS-config-v2.md#reader6-a-string-ends-at-its-first-unescaped-quote)).

### checks.config-fetch-names-nothing: A row's `fetch` names no executable

A `fetch` key on a version-2 row whose value names no executable ([§FS-fetch.2.1](FS-fetch.md#21-a-value-that-names-no-executable)) is refused at the key's line on every command that loads the config, `grund fetch` included:

```text
error: grund.toml:6: `[schema.kinds.AR] fetch` names no executable; name the program, or if this kind is not fetched remove the key and any `[rules.resolution] AR`, and its missing IDs are then errors; this became an error in grund 0.23.0
```

A row another refusal already covers is refused with that error instead, at the same line and without the `this became an error` clause: a `fetch` on a row with `citable = false` is told it needs a citable row, and one on a row without exactly one place is told it needs one place to hold its snapshots. Decided in [§DF-fetch-names-an-executable](../decisions/functional/DF-fetch-names-an-executable.md#df-fetch-names-an-executable-a-fetch-key-must-name-an-executable-warned-for-in-version-2-through-grund-022x-and-refused-from-0230-and-version-1-meets-the-rule-at-grund-fetch-and-config-migrate) and [§DF-fetch-names-an-executable-lands-in-0-23](../decisions/functional/DF-fetch-names-an-executable-lands-in-0-23.md#df-fetch-names-an-executable-lands-in-0-23-a-version-2-fetch-key-that-names-no-executable-is-refused-at-load-from-0230).

### checks.config-md-not-markdown: `md` is mapped away from Markdown

An `md` entry of `[schema.sources.extensions]` that names a language not of class `document` ([§FS-config-v2.schema.sources.extensions](FS-config-v2.md#schemasourcesextensions-schemasourcesextensions-maps-a-suffix-to-a-language)) is refused at the entry's line:

```text
error: grund.toml:8: [schema.sources.extensions] maps `md` to `rust`, which does not read Markdown; `.md` is always Markdown; drop the entry, or map `md` to a `document` language; this became an error in grund 1.0.0
```

Which of this refusal and the five after it a file holding several is refused for is [§FS-config-v2.reader.4.order](FS-config-v2.md#reader4order-which-refusal-a-file-with-several-is-refused-for). Decided in [§DF-markdown-homes-refused-in-1-0](../decisions/functional/DF-markdown-homes-refused-in-1-0.md#df-markdown-homes-refused-in-1-0-an-md-mapped-away-from-markdown-a-named-index-or-rule-home-not-read-as-markdown-and-a-later-value-home-that-is-absent-or-does-not-hold-are-refused-at-load-from-100).

### checks.config-index-not-markdown: A named index is not read as Markdown

A row's `index` that names a file the configuration does not read as Markdown ([§FS-config.3.4.2](FS-config.md#342-index--the-kinds-index-file), [§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule)) is refused at the `index` key's line, whether or not the file exists:

```text
error: grund.toml:9: `[schema.kinds.FS] index` must name a Markdown file; `index.md` is not read as Markdown; name a file a `document` language reads, or set `index = false`; this became an error in grund 1.0.0
```

A row that writes no `index` takes `README.md`, and where that file is not read as Markdown the row is judged as under `index = false`, with no refusal ([§DF-markdown-homes-warn-before-1-0.2.1](../decisions/functional/DF-markdown-homes-warn-before-1-0.md#21-six-load-warnings-each-naming-100)).

### checks.config-value-home-absent: A value row's later home does not exist

A `form = "value"` row whose first `files` or `folders` entry exists, and one of whose later entries does not, is refused at the line of the key that lists the absent entry, not at the `form` line, with the verdict an absent first entry already has ([§FS-config.3.4.9](FS-config.md#349-values--first-class-value-declarations)), in a line of its own that ends in the remedy only a later entry has:

```text
error: grund.toml:5: value home for kind `CONST` does not exist: values/absent.md; create the file, or drop the entry; this became an error in grund 1.0.0
```

A `folders` entry says `create the folder`. The first entry's refusal keeps its own words and its `hint:` line.

### checks.config-value-home-invalid: A value row's later home is the wrong shape, outside the root, or not looked up

A `form = "value"` row whose first `files` or `folders` entry holds, and one of whose later entries exists in the wrong shape (a `files` entry that is not a file, a `folders` entry that is not a folder), resolves outside the project root through a symlink, or cannot be looked up, is refused at the line of the key that lists it, as the first entry already is ([§FS-config.3.4.9](FS-config.md#349-values--first-class-value-declarations)):

```text
error: grund.toml:6: value home for kind `CONST` must be an existing file: values/sub; make it an existing file, or drop the entry; this became an error in grund 1.0.0
```

An entry outside the root says ``must normalize inside the project root: <entry>; name a path inside the project root, or drop the entry``, and one that cannot be looked up says `make the home and every directory above it accessible`. An absent entry is [§FS-config-v2.checks.config-value-home-absent](FS-config-v2.md#checksconfig-value-home-absent-a-value-rows-later-home-does-not-exist), and an entry the reader already refuses as a place (`..`, or a folder that links out of the root) keeps that refusal.

### checks.config-rule-file-not-markdown: A rule kind's `files` entry is read as source

A `form = "rule"` row's `files` entry that a `source` language reads ([§FS-config-v2.schema.sources.rules](FS-config-v2.md#schemasourcesrules-every-home-of-a-rule-kind-is-judged-by-its-reading)) is refused at the entry's line, wherever it sits in the list:

```text
error: grund.toml:11: [schema.kinds.RULE] files entry `rules/more.rs` is read as `rust`, which does not read Markdown, and every home of a rule kind must be; move the rules into a file a `document` language reads; this became an error in grund 1.0.0
```

A row that also holds an entry read by a `data` language or by no admitted language is refused for that one, at `form` ([§FS-config-v2.reader.4.order](FS-config-v2.md#reader4order-which-refusal-a-file-with-several-is-refused-for)).

### checks.config-rule-folder-not-markdown: A rule kind's folder can hold no Markdown

A `form = "rule"` row's `folders` entry, in a config under which no suffix resolves to an admitted `document` language ([§FS-config-v2.schema.sources.rules](FS-config-v2.md#schemasourcesrules-every-home-of-a-rule-kind-is-judged-by-its-reading)), is refused at the `folders` key's line. The configuration alone decides it, so an empty folder, one holding only `.gitkeep` and one that does not exist are refused alike:

```text
error: grund.toml:11: [schema.kinds.RULE] folders entry `rules` can hold no file read as Markdown, because no admitted language reads Markdown; admit one in [schema.sources] languages, such as `markdown`; this became an error in grund 1.0.0
```

It is not `unread-scan-root` ([§FS-scan.checks.unread-scan-root](FS-scan.md#checksunread-scan-root-a-configured-file-root-is-not-read)), which reports what a walk did not read.

### checks.rule-folder-file-not-markdown: A file in a rule kind's folder is read as source

A file the walk reads from a `form = "rule"` row's `folders` entry with a `source` language ([§FS-config-v2.schema.sources.rules](FS-config-v2.md#schemasourcesrules-every-home-of-a-rule-kind-is-judged-by-its-reading)) earns one `rule-folder-file-not-markdown` warning from `check`, at line 1 of that file:

```text
rules/r.rs:1: warning: read as `rust`, which does not read Markdown, in `rules`, a folder home of rule kind `RULE`, so a rule written as a Markdown heading here declares nothing; give it a name a `document` language reads, or move it out of `rules`
```

Only the walk sees what a folder holds, so this is no load refusal ([§DF-markdown-homes-warn-before-1-0.2.2](../decisions/functional/DF-markdown-homes-warn-before-1-0.md#22-a-file-in-a-rule-folder-read-as-source-stays-a-warning)). A folder `config-rule-folder-not-markdown` names is refused at load, so the walk never reaches one.
