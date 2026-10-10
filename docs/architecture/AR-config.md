# AR-config: one Project per project, read by one reader per version and lowered losslessly

A `grund.toml` is read once, by the reader of the version that spelled it, and
lowered into records that hold one concern each ([§FS-config.concerns](../functional-spec/FS-config.md#concerns-every-key-belongs-to-exactly-one-concern)): a
`Project` whose envelope says which project this is and whose `schema`, `rules`
and `presentation` say what exists, how it relates and what bytes get written;
a `Run` for the facts of one invocation; and a `Compiled` for what is derived
from the `Project` once. No component above config reads a version number, so a
second reader is a new file under `config/` and nothing else
([§FS-config.5.2](../functional-spec/FS-config.md#52-every-older-version-keeps-its-meaning)). This page is the decision [§DA-config-concern-records](../decisions/architectural/DA-config-concern-records.md#da-config-concern-records-the-configuration-becomes-three-concern-records-inside-an-envelope-and-the-checker-splits-in-two) made,
written down as the shape the code has; the sketch it amends is
[§DISC-core-concerns.5.1](../discussions/proposals/2026-09-30-core-concerns.md#51-the-core-records).

## placement: Where config sits

```text
grund.toml ─► discovery ─► [ v1 reader ] ─► Project ─► validate ─► compile ─► Compiled
                                              │                                  │
invocation ───────────────────────────────► Run                                  │
                                              └──► Config::from_records ◄────────┘
```

The third box of the pipeline ([§AR-system.2.3](README.md#23-config)). It takes the file discovery
chose and the invocation, and gives every component above it the records, and
the `Config` façade built from them for the components that have not moved off
it yet (section 5). It knows nothing of the tree it describes, and reads only
model and grammar below it ([§AR-system.4](README.md#4-dependency-direction)).

## terms: Terms

Leans on [§FS-terms.terms.1](../functional-spec/FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, catalog),
[§FS-terms.terms.4](../functional-spec/FS-terms.md#terms4-scanning-and-project-structure) (scan, scope, config root, workspace, member), and
[§FS-terms.terms.8](../functional-spec/FS-terms.md#terms8-the-architectures-own-words) (box).

- **record** — one of `Project`, `Run`, `Compiled`, or a struct they hold.
- **reader** — the code that parses one version's file and lowers it into a
  `Project`. There is one today, v1.
- **lowering** — the step from what a file spelled to the record field that
  holds its meaning, defaults and carve-outs included.
- **façade** — the flat `Config`, built from the records and written by nobody.

## 1. The records

The records live in `crates/grund-core/src/config/` and follow
[§DISC-core-concerns.5.1](../discussions/proposals/2026-09-30-core-concerns.md#51-the-core-records) with the places amendment of [§DISC-core-concerns.6.4](../discussions/proposals/2026-09-30-core-concerns.md#64-amended-after-the-verdict-v-a08-and-v-a14).

### 1.1 `Project` and its envelope

`Project { name, version, workspace, schema, rules, presentation }`. `name`,
`version` and `workspace` are the **envelope**: read before any concern, they
decide which file governs and which projects there are, and constrain no node
([§FS-config.concerns](../functional-spec/FS-config.md#concerns-every-key-belongs-to-exactly-one-concern)). `version` is `1` for every file the v1 reader reads.
`workspace` carries `declared`, `members`, `optional_members` and
`include_root`, each with the `ConfigLocation` it was written at.

### 1.2 `Schema`, `Rules`, `Presentation`

`Schema` holds `citation` (marker, strict, shorthand), `ids` (format,
separator, number and slug patterns, named sections, heading levels),
`sources` (include, exclude, extensions, comment prefixes, Python docstrings,
ignore files), `notes` (inline style, the three line and column budgets,
`warn_on_suggested`, layout and layout check), `leads`, `rows` and `nesting`.
`Rules` holds `citations`, `grounding` and `resolution`. `Presentation` holds
`fmt`, the per-kind `kinds` titles, `description`, `conversation`, `trigger`
and `output`.

### 1.3 Rows: places and kinds over one vector

`Schema::rows` is one `Row { name, places: Vec<Place>, kind: Option<Kind> }`
per `[[kinds]]` row, in file order. `Place { extent, scanned }` with
`Extent::Folder(path) | File(path) | Complement`.
`Kind { id_format, form, index, origin }`, where `form` is
`Prose | Value { chapter } | Rule { value_chapter }` and `origin` is `Local | External { fetch }`.
`Rule` keeps `value_chapter` because v1 accepts `rules = true` together with
`value_chapter`, and a lossless lowering keeps both.
`Schema::places()` yields `(name, &Place)` for every place of every row, and
`Schema::kinds()` yields `(name, &Kind)` for every row with a kind. The three
states [§DISC-core-concerns.10.1](../discussions/proposals/2026-09-30-core-concerns.md#101-the-five-decisions-the-2026-09-30-proposal-leaves-open) keeps apart are distinct by construction:

1. a **non-citable place** is `kind: None` with a folder or file place;
2. a **homeless citable kind** is `kind: Some(_)` with no places;
3. the **complement** is a place whose extent is `Complement`.

v1 lowers at most one place per row. Which v2 spelling marks the complement,
and whether the deepest place wins, are #457's choices and are not made here.

### 1.4 `index` is spelled on the kind and judged by rules

`[[kinds]] index` lowers into `Kind::index` ([§DISC-core-concerns.5.1](../discussions/proposals/2026-09-30-core-concerns.md#51-the-core-records)), while
[§DF-config-concerns](../decisions/functional/DF-config-concerns.md#df-config-concerns-a-keys-concern-is-derived-from-what-a-finding-from-it-can-be-about) classifies it under rules. It is the one key whose record
path and concern disagree; the lowering table marks it, and a second such key
is a test failure (section 3.3).

### 1.5 `Run` and `Compiled`

`Run { root, cli_base, config_file, redundant_config_file, scope, workspace,
warnings }` carries what one invocation decided and what workspace expansion
learned about this checkout. Its `scope` holds the full-tree and wide-resolution
flags, the citing-side classification switch and the `cover --lines` ranges; its
`workspace` holds the boundary roots, project roots, scope path and absent
optional members. No `Run` field is read from a file. A command-line override of
a file's value, such as `--require-grounding`, is a `Run` fact the façade applies
over the `Project` value. `Compiled { grammar, demand }` is `compile(&Project)`:
the ID grammar, and the `ScanDemand` that tells the scanner which rows' files
to record grounding structure for ([§AR-config.6.1](AR-config.md#61-scandemand-names-the-rows-that-record-structure)). It replaces `rebuild_grammar`.

## 2. The v1 reader and its isolation

Discovery, which decides which file governs ([§FS-config.1](../functional-spec/FS-config.md#1-file-location-and-discovery)), stays in
`config/` because it runs before a reader is chosen. Everything that parses
version 1 lives under `config/v1/`, relative to `crates/grund-core/src/`: the
top-level and table keys, the `[[kinds]]` row reader, the `[citations]` reader,
the grounding pair, the built-in kind table and the block readers. Its only
output is a `Project` and the `ConfigLocation`s of what it read.

Three rules hold the isolation:

1. Nothing outside `config/v1/` names a v1 type, and nothing above `config`
   reads `Project::version`.
2. Every default, override and historical carve-out of version 1 is applied in
   `config/v1/`, so it is attributable to v1. The records carry meanings, not
   spellings.
3. A meaning v2 cannot spell, such as the legacy FS home or
   `NestedPlaceFallsToComplement`, still has a record value, so the engine can
   represent everything a v1 file means.

## 3. The lowering table

Every v1 key and every implicit default lands in exactly one record field. The
table is data in `config/v1/mapping.rs` as two constants,
`KEYS: &[(&str, &str)]` and `DEFAULTS: &[(&str, &str)]`, each entry
`(name, record path)`. A path there equals the path below or extends it with
more `.`-separated fields. `tests/integration/test_config_lowering_map.py` holds
the code's table to this one.

### 3.1 One row per key

The key column is spelled the way [§DF-config-concerns.2.3](../decisions/functional/DF-config-concerns.md#23-the-inventory) spells it, and the
set is that inventory's set: the 61 keys of the format in force.

| Key | Record path |
|---|---|
| `grund_config_version` | `version` |
| `project_name` | `name` |
| `project_description` | `presentation.description` |
| `[reference] conversation` | `presentation.conversation` |
| `[reference] grounding_level` | `rules.grounding` |
| `[reference] inline_note_layout` | `schema.notes` |
| `[reference] inline_note_layout_check` | `schema.notes` |
| `[reference] inline_note_max_columns` | `schema.notes` |
| `[reference] inline_note_max_lines` | `schema.notes` |
| `[reference] inline_note_suggested_lines` | `schema.notes` |
| `[reference] inline_style` | `schema.notes` |
| `[reference] lead_size_warning` | `schema.leads` |
| `[reference] marker` | `schema.citation` |
| `[reference] require_grounding` | `rules.grounding` |
| `[reference] shorthand` | `schema.citation` |
| `[reference] strict` | `schema.citation` |
| `[reference] trigger` | `presentation.trigger` |
| `[reference] warn_on_suggested` | `schema.notes` |
| `[id] format` | `schema.ids` |
| `[id] named_sections` | `schema.ids` |
| `[id] number_pattern` | `schema.ids` |
| `[id] section_heading_levels` | `schema.ids` |
| `[id] section_separator` | `schema.ids` |
| `[id] slug_pattern` | `schema.ids` |
| `[[kinds]] citable` | `schema.rows` |
| `[[kinds]] fetch` | `schema.rows` |
| `[[kinds]] file` | `schema.rows` |
| `[[kinds]] folder` | `schema.rows` |
| `[[kinds]] format` | `schema.rows` |
| `[[kinds]] grounding_level` | `rules.grounding` |
| `[[kinds]] index` | `schema.rows` |
| `[[kinds]] kind` | `schema.rows` |
| `[[kinds]] require_grounding` | `rules.grounding` |
| `[[kinds]] resolve` | `rules.resolution` |
| `[[kinds]] rules` | `schema.rows` |
| `[[kinds]] scan` | `schema.rows` |
| `[[kinds]] title` | `presentation.kinds` |
| `[[kinds]] value_chapter` | `schema.rows` |
| `[[kinds]] values` | `schema.rows` |
| `[scan] comment_prefixes` | `schema.sources` |
| `[scan] docstring_python` | `schema.sources` |
| `[scan] exclude` | `schema.sources` |
| `[scan] extensions` | `schema.sources` |
| `[scan] include` | `schema.sources` |
| `[scan] respect_gitignore` | `schema.sources` |
| `[output] color` | `presentation.output` |
| `[output] format` | `presentation.output` |
| `[output] relative_paths` | `presentation.output` |
| `[fmt.cross_refs] anchor_format` | `presentation.fmt` |
| `[fmt.cross_refs] enabled` | `presentation.fmt` |
| `[workspace] include_root` | `workspace` |
| `[workspace] members` | `workspace` |
| `[workspace] optional_members` | `workspace` |
| `[citations] default` | `rules.citations` |
| `[citations.<KIND>] default` | `rules.citations` |
| `[citations.<KIND>] may` | `rules.citations` |
| `[citations.<KIND>] must` | `rules.citations` |
| `[citations.<KIND>] must-not` | `rules.citations` |
| `[citations.<KIND>] should` | `rules.citations` |
| `[citations.<KIND>] should-not` | `rules.citations` |
| `[fmt] exclude` | `presentation.fmt` |

`[output] color` is reserved and read by nothing ([§FS-config.3.6](../functional-spec/FS-config.md#36-output--report-format)), but a lossless
lowering keeps it, so `presentation.output` holds it.

### 3.2 One row per implicit default

A meaning a v1 file gets without writing a key. Each row is applied by the v1
reader and nowhere else.

| Default | Record path | What v1 means |
|---|---|---|
| `built-in kinds` | `schema.rows` | A config with no `[[kinds]]` gets the canonical rows of [§FS-config.3.4.4](../functional-spec/FS-config.md#344-the-default-kinds), with their homes, titles and citability. A declared `[[kinds]]` table replaces them all. |
| `legacy FS home` | `schema.rows` | A `grund.toml` that exists and declares no `[[kinds]]` gives `FS` the folder `docs/functional-spec` instead of the file `requirements.md`. A zero-config tree gets the file. |
| `name-keyed index` | `schema.rows` | A declared citable folder row that does not set `index` takes the default for its name: `E2E` has no index, every other name indexes `README.md`. |
| `fetch resolves must` | `rules.resolution` | A row with `fetch` and no `resolve` resolves at `must`. |
| `kind grounding inherits` | `rules.grounding` | A row that sets neither grounding key takes the `[reference]` values. |
| `complement name` | `schema.rows` | With no non-citable homeless row, the complement is named `code` and no row is added. |
| `nesting` | `schema.nesting` | A place inside another row's place sends its files to the complement (`NestedPlaceFallsToComplement`, [§FS-config.3.9.2](../functional-spec/FS-config.md#392-the-homeless-kind)). |
| `strict` | `schema.citation` | `strict` is on: only marked tokens are citations. |
| `named sections` | `schema.ids` | `named_sections` is off. |
| `scan exclusions` | `schema.sources` | `exclude` is `target`, `node_modules`, `.git`, `dist`, `build` and `.venv`. |
| `version` | `version` | A file that omits `grund_config_version` is version `1`. |

### 3.3 What the table is held to

1. The key set of section 3.1 equals the keys the reader's parse sites accept,
   and equals the [§DF-config-concerns](../decisions/functional/DF-config-concerns.md#df-config-concerns-a-keys-concern-is-derived-from-what-a-finding-from-it-can-be-about) inventory.
2. Each key's path starts with the record its concern names: `schema.`,
   `rules.` or `presentation.`, and for the envelope `name`, `version` or
   `workspace`. `[[kinds]] index` is the one exception (section 1.4).
3. The code's `KEYS` and `DEFAULTS` name exactly the rows above, each at the
   path given here or a field below it.
4. For every key row a unit test sets a non-default value, lowers, and asserts
   that the named field changed and no other field did. For every default row
   it lowers an empty file, and a file with no `[[kinds]]`, and asserts the
   value the row describes.

## 4. Validation runs once, after lowering

Meaning validation, as opposed to spelling, runs once on the `Project` in
`config/validate.rs`, whichever reader produced it: prefix-freedom of citable
names, uniqueness of names and of the complement, home and value prerequisites,
the grounding pair, and the marker that `strict` requires. Spelling errors, such
as an unknown key, a wrong type or a value outside a closed set, stay with the
reader that read the spelling. Each rule runs once, at the point the v1 reader
used to raise it: the `[reference]` meanings before the `[[kinds]]` table is
lowered and its entries refused, the member lists and `[citations]` after the
grammar compiles. That way a file with several errors reports the same first error.

An error keeps the text and the `path:line` anchor it has today
([§FS-config.4.3](../functional-spec/FS-config.md#43-invalid-config-behavior)), because each validated value carries the `ConfigLocation` it
was read from. `tests/integration/test_config_lowering_equivalence.py` holds
`grund config show` and `grund config validate`, output and exit code, for every
`grund.toml` under `tests/e2e/cases/` and `examples/` to the bytes captured
before the reader moved.

## 5. The `Config` façade, and the list that only shrinks

`Config` keeps every public field it has. One function builds it,
`Config::from_records(&Project, &Run, &Compiled)`, and nothing assigns to its
fields afterwards: a per-run change is made to the `Run`, and the façade is
rebuilt. A façade nobody writes cannot drift from the records it is built from.

`CONFIG_FACADE` in `tests/integration/test_dependency_direction.py` lists the
components that still name `Config` outside their tests. It is held the way
[§AR-system.4](README.md#4-dependency-direction) holds the upward reads: a component off the list that names
`Config` fails, a component on the list that no longer names it fails until it
is removed, so the list only shrinks. A sibling check fails on any write to a
`Config` field outside `config/`.

The list starts as `api`, `checker`, `config`, `queries`, `resolver`,
`scanner`, `workspace` and `writers`. `model` and `grammar` never named
`Config`, and `templates` and `rules` leave the list with this change:
`templates` renders from `&Project` and `&Compiled`.

`checker`, `resolver` and `scanner` leave the list when the checker splits
([§AR-checker.1](../../crates/grund-core/src/checker/report.rs)): each is handed the records it reads, and a `WorkspaceCheckTarget`
holds a member's `{ catalog, schema, compiled, run }` rather than a `Config`. The
resolver leaves with them because the checker and the scanner call into it and
could not leave while it stayed. `config`, `workspace`, `queries`, `writers` and
`api` remain.

## 6. What `compile` derives for the scanner and the shape views

### 6.1 `ScanDemand` names the rows that record structure

`ScanDemand { structure_rows }` is the set of rows whose effective
`grounding_level` is finer than the file ([§FS-config.3.4.8](../functional-spec/FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)), the complement's
row included. `compile` fills it through the same `grounding_level_for_kind` the
checker cuts units with ([§AR-checker.2.8](../../crates/grund-core/src/checker/report.rs)), so what is recorded and what is cut
stay one rule. `is_empty()` is the single-field answer a level-1 tree — every
configuration written before the keys existed — is excused by
([§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible)), and `records_structure(row)` is the per-file question the
scanner asks of the row it already looked up ([§AR-scanner.2.7.3](AR-scanner.md#273-demand-arrives-as-scandemand)). It extends the
global `grounding_units` flag rather than adding a second type beside it, so the
scanner reads no grounding key at all.

### 6.2 A kind's shape is a sequence of slots, derived from its form

`Kind::slots()` yields the kind's slots in declared order, each
`Slot { handle: Named | Numbered, presence, content: Prose | Values | OneOf }`.
It is a method over the `form` of [§AR-config.1.3](AR-config.md#13-rows-places-and-kinds-over-one-vector), not stored data, so there is no
second hand-maintained model of fields: a `Prose` or `Rule` kind yields no slot,
and a `Value` kind yields one — its value chapter as a `Named` handle with
`Values` content, or `Numbered` when it has no chapter. Conformance reads a
kind's value shape through `slots()` ([§AR-checker.1.1](../../crates/grund-core/src/checker/report.rs)), and a field model added to
the kind is yielded by the same method in the order it was declared, so the
checks and a schema view read one sequence.

## 7. The v2 reader

`config/v2/` reads a file that writes `grund_config_version = 2`
([§FS-config-v2](../functional-spec/FS-config-v2.md#fs-config-v2-grund-reads-a-version-2-config-by-concern-with-one-strength-vocabulary-and-fixed-defaults)) and lowers it into the same `Project`, so nothing above
`config/` learns there is a second spelling. Discovery reads the text once and
asks `v2::selects` whether it writes version 2 before any table; the v2 reader
and the v1 reader then share only what runs after lowering: `validate`,
`compile` and the façade (section 4). The v2 reader shares no spelling check
with v1, because its keys, tables and messages are its own ([§FS-config-v2.reader](../functional-spec/FS-config-v2.md#reader-the-reader)).

### 7.1 One walk, one concern per file

The reader is a single pass over the lines, in `walk.rs`. A header opens the
table `tables.rs` names, or is refused there: an unknown table, an array of
tables, a strength key written as a table ([§FS-config-v2.reader.3](../functional-spec/FS-config-v2.md#reader3-one-setting-one-form)), or a clause
this grund does not execute yet ([§FS-config-v2.rollout](../functional-spec/FS-config-v2.md#rollout-what-this-grund-executes-and-what-it-refuses)). A key is read the
moment it is met, by the file of its concern: `schema.rs` and `measures.rs`
for `[schema.*]`, `rules.rs` for `[rules.*]`, and `presentation.rs` for
`[presentation.*]` and `[workspace]`. A key the table's handler does not admit
is `unknown key`, and a repeated header or key is refused at its second line
([§FS-config-v2.reader.1](../functional-spec/FS-config-v2.md#reader1-tables-and-keys-are-closed), [§FS-config-v2.reader.2](../functional-spec/FS-config-v2.md#reader2-every-row-and-field-is-written-under-its-own-header)). Because every key is judged
where it is written, the first refusal reported is the first one in the file
([§FS-config-v2.reader.4](../functional-spec/FS-config-v2.md#reader4-order-and-location-are-kept)).

### 7.2 What waits for a table, and what waits for the file

A check that reads several keys of one table runs when the table closes: a
measure's thresholds ordered against each other and against the epoch's `must`
([§FS-config-v2.schema.measures](../functional-spec/FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys)), a ladder's rungs ([§FS-config-v2.rules.grounding](../functional-spec/FS-config-v2.md#rulesgrounding-grounding-is-a-ladder-replaced-whole)),
and two citation lists that can match one citation
([§FS-config-v2.rules.citations](../functional-spec/FS-config-v2.md#rulescitations-rulescitations)). A check that reads several tables runs in
`finish.rs` once the walk ends: a row's keys together, one row's place inside
another's, a ladder, resolution entry or title naming its row, and the omitted
language set ([§FS-config-v2.defaults.2](../functional-spec/FS-config-v2.md#defaults2-the-language-set-is-fixed-before-it-is-executed)). Each is kept with the line it is
about, and the earliest is reported.

### 7.3 The lowering

The v2 epoch is a literal `Project` in `defaults.rs`, written out rather than
inherited from v1 ([§FS-config-v2.defaults.1](../functional-spec/FS-config-v2.md#defaults1-what-v2-fixes)). Each key overwrites the field
section 3.1 names for its v1 counterpart, and `warn` and `warn-not` fill the
fields the warning channel reads, beside `must` and `should`
([§FS-config-v2.rules.strengths](../functional-spec/FS-config-v2.md#rulesstrengths-one-strength-vocabulary-two-severities)):

| v2 spelling | Record |
|---|---|
| `[schema.kinds.<NAME>]` | one `Row`, in authored order; `citable = false` leaves `kind` empty |
| the rows' scanned places | `schema.sources.include`, or the config root when no row has a place |
| `[schema.notes.lines]` | `must` → `max_lines`; the weakest softer threshold → `suggested_lines`, warning when it is `warn`; the rest kept as thresholds |
| `[schema.notes.columns]` | `must` → `max_columns`; the rest kept as thresholds |
| `[schema.leads.words]` | `warn` → `leads`; the rest kept as thresholds |
| `[rules.citations.<KIND>]` | the seven lists of `KindCitationRules`, `warn` and `warn-not` included |
| `[rules.citations.grounding]`, `[rules.citations.<NAME>.grounding]` | the ladder, whole; its `must` rung is the effective `require` and `level` pair |
| `fetch`, `[rules.resolution]` | `Origin::External`, and the kind's resolution, `must` unless `warn` |

A state version 2 does not spell keeps its v1 record value, so a v1 file means
what it meant ([§FS-config-v2.mapping](../functional-spec/FS-config-v2.md#mapping-what-the-records-keep-apart)).

### 7.4 Writing it back

`Project::v2_toml` in `render.rs` is what `grund config show` prints for a v2
project: every effective value under the table of its concern, in a fixed
order, which loads back as v2 to the same records ([§FS-config.4.2](../functional-spec/FS-config.md#42-grund-config-show-path)). It returns
nothing for a v1 project, whose output stays the bytes it always was; writing a
v1 file in v2 spelling is migration and not this reader's.
