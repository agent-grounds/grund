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
`Kind { id_format, form, index, origin, fields, closed }`, where `form` is
`Prose | Value | Rule`, `origin` is `Local | External { fetch }`, `fields` is the
kind's declared fields in header order and `closed` the strength of the
undeclared-chapter check, `may` when the kind is open ([§FS-fields.record](../functional-spec/FS-fields.md#record-the-record)).
`Field { name, title, presence, placement, content }` lives in `config/fields.rs`,
with `placement` one of `First | Last | After(name)` and `content` one of
`Prose | Values | OneOf(values)`. `form` carries no payload: v1's `value_chapter`
is a `may` field with `Values` content, on a `Value` kind and on a `Rule` kind
alike, because v1 accepts `rules = true` together with `value_chapter` and a
lossless lowering keeps both ([§FS-fields.v1](../functional-spec/FS-fields.md#v1-version-1-is-unchanged)).
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
optional members, and the run's kind table: one row per project the config
declares, holding that project's kinds, its named-sections setting and each
kind's scalar fields with their `one_of` values, built,
judged against the pinned `[citations]` targets and held by one call into
`config/kinds.rs`. Workspace expansion makes that call where it stamps the
project roots, and a narrowed `cover` run makes it over the workspace its
config declares, judging nothing where that workspace does not expand and
only its root's kinds are read; a standalone run holds a table of one project. Every kind-name
question a command asks is answered from that table, within the command's own
scope ([§FS-scan.projects.kinds](../functional-spec/FS-scan.md#projectskinds-whose-kinds-a-run-reads)); a whose clause's field and value are judged
against the scalar fields of the project that configures its kind
([§FS-rules.2.2](../functional-spec/FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)), as its named-sections setting is. A rule vocabulary judges its objects with a table of
the same type, built from the projects its own run loaded, so a narrowed run's
namespaced object stays unverifiable ([§FS-rules.4.1](../functional-spec/FS-rules.md#41-a-rule-this-scope-cannot-verify)). It is read from configs, never from scanned
files, so it is config-time work. Beside those it carries the editor's **buffers** and the
**selection**. The buffers are the open documents the invocation was handed,
keyed by physical path, which every reader of the run reads as the files they
name through `sources` ([§AR-system.2.13](README.md#213-sources), [§FS-scan.text.buffer](../functional-spec/FS-scan.md#textbuffer-a-files-text-is-what-a-save-would-write)); a command-line run has
none. The selection is which projects load, what the path bounds for resolution
and what it bounds for the report ([§FS-scan.path](../functional-spec/FS-scan.md#path-what-a-path-argument-means), [§FS-scan.projects](../functional-spec/FS-scan.md#projects-which-projects-a-run-loads)). Each
command keeps its own: `check`, `show`, `list` and `id` with a path resolve
against the whole project the path discovers and narrow only what they report,
a workspace `check <dir>` outside every member is the root run narrowed to the
path ([§FS-scan.projects.narrowed](../functional-spec/FS-scan.md#projectsnarrowed-a-path-below-the-config-root-loads-what-a-run-from-it-would-see)), `refs` and `cover` with a path resolve
within the path's tree, and the editor's root lookup skips the member rewrite.

One function builds every `Run` from what the invocation decided: `open()`, in
the workspace component, the lowest that knows both the config and the members.
It takes the invocation, the path, the command, its flags, `--path-base` and the
buffers, discovers the config, applies the member rewrite and the workspace
boundary, and builds the `Run` once. Nothing a command decides is set on a `Run`
afterwards, as nothing is on the façade (section 5): a per-run fact a command
decides is an argument to `open()`, never a setter on a built `Run`. What is
completed afterwards is only what the run discovers rather than decides:
workspace expansion, which runs once a workspace check has its root config,
builds each member's `Run` through the same constructor and then records the
`workspace` facts it found (the boundary roots, project roots, scope path and
absent optional members, and the kind table, which a narrowed `cover` run reads
from its declared workspace instead) and the run warnings it raised. No other `Run` field
is read from a file. A command-line override of
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
3. A meaning v2 cannot spell, such as
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
set is that inventory's set: the 62 keys of the format in force.

| Key | Record path |
|---|---|
| `grund_config_version` | `version` |
| `project_name` | `name` |
| `project_description` | `presentation.description` |
| `kinds` | `schema.rows` |
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
| `built-in kinds` | `schema.rows` | A config with no `[[kinds]]`, like a tree with no config at all, gets the canonical rows of [§FS-config.3.4.4](../functional-spec/FS-config.md#344-the-default-kinds), with their homes, titles and citability. A declared `[[kinds]]` table replaces them all. |
| `name-keyed index` | `schema.rows` | A declared citable folder row that does not set `index` takes the default for its name: `E2E` has no index, every other name indexes `README.md`. |
| `fetch resolves must` | `rules.resolution` | A row with `fetch` and no `resolve` resolves at `must`. |
| `kind grounding inherits` | `rules.grounding` | A row that sets neither grounding key takes the `[reference]` values. |
| `complement name` | `schema.rows` | With no non-citable homeless row, the complement is named `code` and no row is added. |
| `nesting` | `schema.nesting` | A place inside another row's place sends its files to the complement (`NestedPlaceFallsToComplement`, [§FS-config.3.9.2](../functional-spec/FS-config.md#392-the-homeless-kind)). |
| `strict` | `schema.citation` | `strict` is on: only marked tokens are citations. |
| `named sections` | `schema.ids` | `named_sections` is off. |
| `scan exclusions` | `schema.sources` | `exclude` is `target`, `node_modules`, `.git`, `dist`, `build` and `.venv`. |
| `version` | `version` | A file that omits `grund_config_version` is version `1`. |

The built-in kinds are not every kind a generated config declares. The
`software-development` template declares `REQ`, which has no row here
([§FS-init.2.4.10](../functional-spec/FS-init.md#2410-req-is-declared-in-docsrequirements)), so
`init` does not describe the config it is about to write from these defaults: it
reads the rendered template text through the same reader a file on disk goes
through (`pending_config`), and the managed block it renders lists exactly the
kinds that file declares.

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
5. `KEYS` is also the v1 reader's outer-key gate: a key outside it is refused
   with the reader's `unknown config key` text, after the specific refusals
   that run first and keep their words, so the table and what the reader
   admits cannot differ, and `grund config explain` enumerates version 1 from
   it (section 9, [§FS-config-explain.3](../functional-spec/FS-config-explain.md#3-a-keys-answer)).

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
`Slot { handle: Named | Numbered, title, placement, presence, content: Prose | Values | OneOf }`.
It is a method over the `form` and `fields` of [§AR-config.1.3](AR-config.md#13-rows-places-and-kinds-over-one-vector), not stored data, so
there is no second hand-maintained model of fields: a `Value` kind yields one
`Numbered` slot with `Values` content for its numbered components, and then
every kind yields one `Named` slot per field, in header order, carrying the
field's title, placement, strength and content ([§FS-fields.record](../functional-spec/FS-fields.md#record-the-record)). A v1
value chapter arrives as one of those fields. Conformance reads a kind's value
shape through `slots()` ([§AR-checker.1.1](../../crates/grund-core/src/checker/report.rs)), so the checks and a schema view read
one sequence.

### 6.3 The schema view is lowered from a `Project` on demand

`SchemaView`, in `config/` beside `compiled.rs`, is the record [§FS-schema](../functional-spec/FS-schema.md#fs-schema-grund-schema-shows-what-a-repository-is-held-to-read-from-its-configuration-alone) prints:
its nodes in row order with their class, places, slots and grounding badge, its
junctions, and its edges in the order [§FS-schema.7](../functional-spec/FS-schema.md#7-order) fixes. One function lowers it
from a `Project` — the rows, `Kind::slots()` ([§AR-config.6.2](AR-config.md#62-a-kinds-shape-is-a-sequence-of-slots-derived-from-its-form)), `rules.citations`,
`rules.grounding` and the kind titles — and reads no scan, no `Catalog` and no ID
graph. It is built only by the command that prints it, not inside `compile`, so
`check` pays nothing for it. The loading path is the one `config show` already
uses, so the view and the checks read one record. The engine returns the record and
`grund-cli` holds the three printers ([§DA-engine-renders-nothing](../decisions/architectural/DA-engine-renders-nothing.md#da-engine-renders-nothing-the-engine-renders-nothing-so-the-deprecated-compat-frontend-retires)); the managed
block's renderer is untouched, and can later read the same record without depending
on CLI code.

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
`[presentation.*]` and `[workspace]`. Each of them reads a string, a scalar, a
list's element or a quoted key, through `strings.rs`, which reads it as the
readers before it did and records the first that closes early in each value for
its warning ([§FS-config-v2.reader.6](../functional-spec/FS-config-v2.md#reader6-a-string-ends-at-its-first-unescaped-quote)). A key the table's handler does not admit
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
language set ([§FS-config-v2.defaults.2](../functional-spec/FS-config-v2.md#defaults2-the-language-set-is-fixed-before-it-is-executed)), and one kind's fields together —
two ends, two values fields, an `after` that names no field, the `last` one or
an already-followed one, and a cycle ([§FS-fields.order.refusals](../functional-spec/FS-fields.md#orderrefusals-what-the-loader-refuses)). Each is kept
with the line it is about, and the earliest is reported. A field table's own
keys are judged as they are met, in `v2/fields.rs`, and a field or `closed` on
a row whose declarations have no authored body is refused at its header
([§FS-fields.forms](../functional-spec/FS-fields.md#forms-form-lowers-to-the-version-1-contracts)). The prerequisites `form` carries are v1's own functions,
called once the late checks pass, so both readers refuse a rule or value kind
with one text.

### 7.3 The lowering

The v2 epoch is a literal `Project` in `defaults.rs`, written out rather than
inherited from v1 ([§FS-config-v2.defaults.1](../functional-spec/FS-config-v2.md#defaults1-what-v2-fixes)). Each key overwrites the field
section 3.1 names for its v1 counterpart, and `warn` and `warn-not` fill the
fields the warning channel reads, beside `must` and `should`
([§FS-config-v2.rules.strengths](../functional-spec/FS-config-v2.md#rulesstrengths-one-strength-vocabulary-two-severities)):

| v2 spelling | Record |
|---|---|
| `[schema.kinds.<NAME>]` | one `Row`, in authored order; `citable = false` leaves `kind` empty |
| the rows' scanned places and the complement's extent | `schema.sources.include`, the one channel `scope_roots.rs` reads; the config root unless `[schema.kinds.code]` lists places ([§FS-config-v2.schema.places.complement](../functional-spec/FS-config-v2.md#schemaplacescomplement-code-is-the-complement-by-name)) |
| ownership of a nested place | `schema.nesting` = `DeepestPlaceWins`: a listed file, then the deepest folder ([§FS-config-v2.schema.places.ownership](../functional-spec/FS-config-v2.md#schemaplacesownership-a-listed-file-then-the-deepest-folder)) |
| `[schema.notes.lines]` | `must` → `max_lines`; the weakest softer threshold → `suggested_lines`, warning when it is `warn`; the rest kept as thresholds |
| `[schema.notes.columns]` | `must` → `max_columns`; the rest kept as thresholds |
| `[schema.leads.words]` | `warn` → `leads`; the rest kept as thresholds |
| `[rules.citations.<KIND>]` | the seven lists of `KindCitationRules`, `warn` and `warn-not` included |
| `[rules.citations.grounding]`, `[rules.citations.<NAME>.grounding]` | the ladder, whole; its `must` rung is the effective `require` and `level` pair |
| `fetch`, `[rules.resolution]` | `Origin::External`, and the kind's resolution, `must` unless `warn` |
| `form` | `Kind::form`, `prose` unless written |
| `closed` | `Kind::closed`, `may` unless written |
| `[schema.kinds.<NAME>.fields.<field>]` | one `Field`, in header order; `presence` `may`, no placement and `Prose` content unless written |

A state version 2 does not spell keeps its v1 record value, so a v1 file means
what it meant ([§FS-config-v2.mapping](../functional-spec/FS-config-v2.md#mapping-what-the-records-keep-apart)).

### 7.4 Writing it back

`Project::v2_toml` in `render.rs` is what `grund config show` prints for a v2
project: every effective value under the table of its concern, in a fixed
order, which loads back as v2 to the same records ([§FS-config.4.2](../functional-spec/FS-config.md#42-grund-config-show-path)). It returns
nothing for a v1 project, whose output stays the bytes it always was; writing a
v1 file in v2 spelling is migration and not this reader's. Migration (section 8)
reaches the same renderer from the other side: it converts a v1 `Project` into
v2-shaped records and hands those to `v2_toml`, so `config show` and
`config migrate` cannot disagree on v2 spelling
([§FS-config-migrate.2.1](../functional-spec/FS-config-migrate.md#21-stdout-is-the-candidate)).

## 8. Migration translates records

`grund config migrate` ([§FS-config-migrate](../functional-spec/FS-config-migrate.md#fs-config-migrate-grund-config-migrate---to-2-shows-the-version-2-file-that-means-what-a-version-1-config-means-and-writes-it-only-when-asked-and-only-when-it-does)) lives in
a `migrate` module under `crates/grund-core/src/config/`. It renders nothing: it returns one
typed record holding the candidate text, the key map, the defaults written out,
the notices, the refusals, the listed comments and both gates' results, and
`grund-cli` renders that record in `cli_config_migrate.rs` beside `cli_config.rs`.

### 8.1 What it reads, and how

- **The v1 file, once.** Migration opts into the v1 reader's owned line records,
  including raw text, key locations and attached comments, during its existing
  single parse; migration does not parse the file a second time
  ([§FS-config-migrate.4.3](../functional-spec/FS-config-migrate.md#43-everything-else-is-listed)). A tolerant mode of that reader, used by migration
  alone, accepts a path-shaped `[scan] exclude` entry so that it can be
  diagnosed; every other caller still refuses it. The ordinary reader's default
  pass neither constructs nor retains migration-only records
  ([§FS-config-migrate.4.3](../functional-spec/FS-config-migrate.md#43-everything-else-is-listed)). That policy is defined once at the shared reader, so all
  ordinary callers skip capture. Capturing leaves lowering and the first-error
  order of section 4 unchanged; migration keeps every comment and the same
  records, report and guidance equivalence gates ([§FS-config-migrate.4.1](../functional-spec/FS-config-migrate.md#41-attached-comments-move-with-a-one-to-one-key),
  [§FS-config-migrate.4.2](../functional-spec/FS-config-migrate.md#42-the-header-block-stays-at-the-top), [§FS-config-migrate.4.3](../functional-spec/FS-config-migrate.md#43-everything-else-is-listed), [§FS-config-migrate.5.1](../functional-spec/FS-config-migrate.md#51-the-records-gate),
  [§FS-config-migrate.5.2](../functional-spec/FS-config-migrate.md#52-the-report-gate), [§FS-config-migrate.5.3](../functional-spec/FS-config-migrate.md#53-guidance)).
- **The candidate, as a file would be read.** The candidate text is loaded back
  through `pending_config`, the path `init` uses for config text not yet on
  disk, so the records gate compares what the v2 reader really makes of it
  ([§FS-config-migrate.5.1](../functional-spec/FS-config-migrate.md#51-the-records-gate)).

### 8.2 The lowering table is its checklist

Every row of section 3's table, key and default alike, is a migration case:
the code's `KEYS` and `DEFAULTS` are iterated, and a row with no case fails a
test, as section 3.3 holds the reader ([§FS-config-migrate.mapping](../functional-spec/FS-config-migrate.md#mapping-every-version-1-key-and-default)).

### 8.3 The write is a shared writer

The atomic replace `fetch` used is `writers/atomic_install.rs`, a writer both
call, unchanged in behaviour for `fetch`, which maps each failure into its own
refusal. It gains the concurrent-edit check migration needs: the bytes the run
read are passed in and read again just before the rename ([§FS-config-migrate.6.2](../functional-spec/FS-config-migrate.md#62-atomic),
[§FS-config-migrate.6.3](../functional-spec/FS-config-migrate.md#63-a-concurrent-edit)).

## 9. The answer reads the gates

`grund config explain` ([§FS-config-explain](../functional-spec/FS-config-explain.md#fs-config-explain-grund-config-explain-answers-from-the-installed-binary-and-in-the-version-the-projects-config-is-read-as-what-any-key-accepts-and-means-and-what-any-check-code-means-and-what-turns-it-on)) lives in an `explain`
module under `crates/grund-core/src/config/`. Like migration it renders nothing:
it returns one typed answer, and `grund-cli` renders it in
`cli_config_explain.rs`, reached through the api entry `config show` uses.

### 9.1 What it reads

- **The version, and nothing else of the file.** The selection `config show`
  already makes ([§FS-config.5](../functional-spec/FS-config.md#5-schema-versioning)), with no reader lowering the file, so
  a file a reader refuses is still answered in its version
  ([§FS-config-explain.2](../functional-spec/FS-config-explain.md#2-which-version-answers)).
- **The gates.** Version 2's closed key list per table and the list of
  version-1 tables its refusal names; version 1's `KEYS` (section 3.3) and the
  inline members of `lead_size_warning`, admitted from one list in
  `config/point_sizes.rs` that the answer reads too. The answer enumerates from
  these and never from the schema, so it names exactly what a reader admits
  ([§FS-config-explain.3](../functional-spec/FS-config-explain.md#3-a-keys-answer)).
- **Two assets** in an `explain/` directory beside grund-core's other assets, each rendered by a
  script and compared byte for byte by a test that renders it again:
  `check-codes.json`, the code catalog's rows ([§FS-errors.5.5.1](../functional-spec/FS-errors.md#551-how-a-row-is-read)), and
  `grund.schema.json`, the published schema with its specification metadata
  removed, which words each key ([§FS-config-explain.3.1](../functional-spec/FS-config-explain.md#31-values-in-words)).

The code handles sit in model, rendered from the same rows: one typed handle
per code and the sorted list of them, which the selector vocabulary,
`check --help` and the Python binding read, and from which alone a finding is
built ([§AR-system.2.2](README.md#22-model)). The line `check` prints for a selected check
that was off reads its switch spellings from the same rendered rows
([§FS-check.1.4.1](../functional-spec/FS-check.md#141-a-selected-check-the-run-leaves-off-says-so)).

A code's remedy is read the same way. The renderer reads each row's *Fix* and
*Note* cells by the grammar of [§FS-errors.5.5.1](../functional-spec/FS-errors.md#551-how-a-row-is-read), refusing a blank *Fix*, an
unbalanced slot, an unknown reference, an ID or a link, expands each
reference, and writes each version's side into `check-codes.json`. It also
renders `model/remedies.rs`: one typed filler per template or case, taking its
slots, such as `section_address_changed(new_address, cited_address)`, whose
body reads the row and fills the cell. An emission site builds its message's
remedy only by calling its filler, so no remedy is worded in a string literal
of the checker, a misspelt slot or case does not compile, and the message and
the `config explain` answer that prints the same cell unfilled cannot drift
apart ([§AR-system.principle](README.md#principle-one-fact-one-point-one-implementation)).

### 9.2 Why it is in config

Only config reads a version (section 2), and every answer is in one. The gates
are config's own, and reaching `KEYS` or the version-1 table list from above
config would break the v1 reader's isolation that section 2 keeps. So the
answer is assembled where those lists live, and the frontend only renders it,
as it renders `config show` and migration ([§AR-system.4](README.md#4-dependency-direction)).
