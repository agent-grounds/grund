<!-- Rendered from schemas/grund.schema.json by scripts/render_config_reference.py: edit the schema, then run the script. -->

# The `grund.toml` reference

Every key `grund.toml` accepts, table by table: first version 1, the default, then
version 2, which a file selects with `grund_config_version = 2`. A key you leave out
takes the default shown. `grund config validate` checks a file against these keys and
`grund config show` prints the configuration in effect
([§FS-config.4](../functional-spec/FS-config.md#4-validation-and-inspection)).

## Editor support

The same keys are published as a JSON Schema,
[`schemas/grund.schema.json`](../../schemas/grund.schema.json), so a TOML language
server can complete and check them as you type. With [Taplo](https://taplo.tamasfe.dev/),
which is also what the Even Better TOML extension for VS Code runs, name it on the first
line of `grund.toml`:

```toml
#:schema https://raw.githubusercontent.com/agent-grounds/grund/main/schemas/grund.schema.json
```

or once for every project, in your `.taplo.toml`:

```toml
[[rule]]
include = ["**/grund.toml"]

[rule.schema]
path = "https://raw.githubusercontent.com/agent-grounds/grund/main/schemas/grund.schema.json"
```

Replace `main` with the tag of the `grund` you run, such as `v0.17.0`, to match its
keys exactly. The schema knows each key's type, values and default; the rules that
relate keys to each other, and to the files on disk, are `grund config validate`'s.

## Version 1

Version 1, the default.

### Top level

| Key | Type | Default | What it sets |
|---|---|---|---|
| `grund_config_version` | `1` | `1` | The config format version. Omitted means 1. [§FS-config.5](../functional-spec/FS-config.md#5-schema-versioning) |
| `project_name` | string | — | The project's name; in a workspace, also its alias in `§alias/<ID>` citations. [§FS-config.3](../functional-spec/FS-config.md#3-keys) |
| `project_description` | string | — | One-line description shown next to the project in generated workspace member lists. [§FS-config.3](../functional-spec/FS-config.md#3-keys) |

### `[reference]`

How a citation is written, and the project-wide grounding and inline-note defaults. [§FS-config.3.1](../functional-spec/FS-config.md#31-reference--citation-form)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `marker` | string | `"§"` | The character(s) that mark a citation. [§FS-config.3.1](../functional-spec/FS-config.md#31-reference--citation-form) |
| `trigger` | string | `"$$"` | The typed sequence the editor and `grund fmt` turn into the marker. [§FS-config.3.1](../functional-spec/FS-config.md#31-reference--citation-form) |
| `strict` | boolean | `true` | `true` reads only marked citations; `false` reads bare IDs too. [§FS-config.3.1](../functional-spec/FS-config.md#31-reference--citation-form) |
| `shorthand` | `"canonical"` \| `"accepted"` | `"canonical"` | Whether number-only citations are written out in full (`canonical`) or kept as written (`accepted`). [§FS-config.3.1.1](../functional-spec/FS-config.md#311-shorthand--persisted-number-only-citations) |
| `require_grounding` | boolean | `false` | Whether every scanned source file must cite a declared ID; the default for every `[[kinds]]` row. [§FS-config.3.1.7](../functional-spec/FS-config.md#317-require_grounding-and-grounding_level--defaults-for-kinds) |
| `grounding_level` | integer, 1–6 | `1` | The unit that must carry a citation: `1` the whole file, `N` every `H<N>` section too; the default for every `[[kinds]]` row. [§FS-config.3.1.7](../functional-spec/FS-config.md#317-require_grounding-and-grounding_level--defaults-for-kinds) |
| `conversation` | `"link"` | — | The repository's opinion on how agents render citations in conversation: `link`, or absent for none. [§FS-config.3.1.3](../functional-spec/FS-config.md#313-conversation--one-name-two-scopes) |
| `lead_size_warning` | inline table `{ max, unit }` | — | Opt in to a warning when a declaration's lead outgrows `max` `unit`s (`lines`, `words` or `bytes`). [§FS-config.3.1.2](../functional-spec/FS-config.md#312-lead_size_warning--the-oversized-lead-opt-in) |
| `inline_style` | `"citation-with-note"` \| `"citation-only"` | `"citation-with-note"` | Whether an inline citation may carry a note beside it. [§FS-config.3.1.8](../functional-spec/FS-config.md#318-inline_style-and-the-note-budgets) |
| `inline_note_suggested_lines` | integer | `1` | Lines past which an inline note is a suggestion (a warning with `warn_on_suggested`). [§FS-config.3.1.8](../functional-spec/FS-config.md#318-inline_style-and-the-note-budgets) |
| `inline_note_max_lines` | integer | `3` | The most lines an inline note may take; more is an error. [§FS-config.3.1.8](../functional-spec/FS-config.md#318-inline_style-and-the-note-budgets) |
| `inline_note_max_columns` | integer | `100` | The most columns an inline note may take; more is an error. [§FS-config.3.1.8](../functional-spec/FS-config.md#318-inline_style-and-the-note-budgets) |
| `inline_note_layout` | `"any"` \| `"citation-first-colon"` | `"any"` | Where the citation sits inside an inline note. [§FS-config.3.1.9](../functional-spec/FS-config.md#319-inline_note_layout-and-inline_note_layout_check) |
| `inline_note_layout_check` | `"off"` \| `"warn"` \| `"error"` | `"off"` | How a note that breaks `inline_note_layout` is reported. [§FS-config.3.1.9](../functional-spec/FS-config.md#319-inline_note_layout-and-inline_note_layout_check) |
| `warn_on_suggested` | boolean | `false` | Report notes past `inline_note_suggested_lines` as warnings rather than suggestions. [§FS-config.3.1.8](../functional-spec/FS-config.md#318-inline_style-and-the-note-budgets) |

### `[id]`

The ID grammar: the shape of every declared ID and section path. [§FS-config.3.2](../functional-spec/FS-config.md#32-id--id-grammar)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `format` | string | `"{kind}-{number}-{slug}"` | The ID template: `{kind}` plus `{number}`, `{slug}` or both; everything else is literal. [§FS-config.3.2](../functional-spec/FS-config.md#32-id--id-grammar) |
| `section_separator` | string | `"."` | The separator between an ID and its section path. [§FS-config.3.2.2](../functional-spec/FS-config.md#322-section_separator-must-stay-distinguishable) |
| `number_pattern` | string | `"\\d+"` | Regex for `{number}`. [§FS-config.3.2.4](../functional-spec/FS-config.md#324-each-pattern-compiles-on-its-own) |
| `slug_pattern` | string | `"[a-z0-9][a-z0-9-]*"` | Regex for `{slug}`. [§FS-config.3.2.4](../functional-spec/FS-config.md#324-each-pattern-compiles-on-its-own) |
| `named_sections` | boolean | `false` | Enable named section components, such as `## goals:`, matching `[a-z][a-z0-9-]*`. [§FS-config.3.2.7](../functional-spec/FS-config.md#327-named_sections--the-gate-for-explicit-section-names) |
| `section_heading_levels` | `"strict"` \| `"warn"` \| `"loose"` | `"strict"` | How a heading whose depth does not match its section path is reported: an error, a warning, or not at all. [§FS-config.3.3.2](../functional-spec/FS-config.md#332-section_heading_levels--heading-depth-against-path-depth) |

### `[[kinds]]`

One row per kind: where its declarations live and how they are read. [§FS-config.3.4](../functional-spec/FS-config.md#34-kinds--recognized-kinds)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `kind` | string | — | The kind's name: the ID prefix of a citable kind, its `[citations.<kind>]` key and its `--kind` selector. [§FS-config.3.4](../functional-spec/FS-config.md#34-kinds--recognized-kinds) |
| `prefix` | refused | — | Removed in grund 0.13.0 and refused: rename it to `kind`. [§FS-config.3.4.6](../functional-spec/FS-config.md#346-prefix-the-former-spelling-of-kind-removed-in-0130) |
| `folder` | string | — | A folder home: each declaration is the H1 of its own file under this directory. [§FS-config.3.4](../functional-spec/FS-config.md#34-kinds--recognized-kinds) |
| `file` | string | — | A single-file home: every declaration is a heading in this document. [§FS-config.3.4](../functional-spec/FS-config.md#34-kinds--recognized-kinds) |
| `title` | string | — | The kind's title in the Project map, `grund list --summary` and editor hovers. [§FS-config.3.4.3](../functional-spec/FS-config.md#343-title) |
| `citable` | boolean | `true` | `false` makes this a place: it cites, but declares no IDs. [§FS-config.3.4.1](../functional-spec/FS-config.md#341-citable--kinds-that-declare-no-ids) |
| `index` | string or `false` | README.md (folder kinds; `false` for a kind named E2E) | The folder's index file, which must list every declaration; `false` opts out. [§FS-config.3.4.2](../functional-spec/FS-config.md#342-index--the-kinds-index-file) |
| `scan` | boolean | `true` | `false` lists the kind in the Project map without walking it. [§FS-config.3.4.7](../functional-spec/FS-config.md#347-scan--a-place-that-is-listed-not-scanned) |
| `require_grounding` | boolean | inherits `[reference] require_grounding` | This kind's grounding requirement; it wins over `[reference]` and over `--require-grounding`. [§FS-config.3.4.8](../functional-spec/FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level) |
| `grounding_level` | integer, 1–6 | inherits `[reference] grounding_level` | This kind's grounding unit, overriding `[reference] grounding_level`. [§FS-config.3.4.8](../functional-spec/FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level) |
| `values` | boolean | `false` | Make every declaration of this kind a first-class value. [§FS-config.3.4.9](../functional-spec/FS-config.md#349-values--first-class-value-declarations) |
| `value_chapter` | string | — | The chapter whose named children are values, in every declaration of this kind. [§FS-config.3.4.13](../functional-spec/FS-config.md#3413-value_chapter--the-chapter-whose-named-children-are-values) |
| `rules` | boolean | `false` | Make every declaration title of this kind an executable rule sentence. [§FS-config.3.4.12](../functional-spec/FS-config.md#3412-rules--rule-declaration-kinds) |
| `format` | string | inherits `[id] format` | This kind's ID template, overriding `[id] format`. [§FS-config.3.4.10.1](../functional-spec/FS-config.md#34101-format) |
| `resolve` | `"must"` \| `"should"` | must when fetch is set, else absent | How a missing external snapshot is reported: `must` as a dangling citation, `should` as a warning. [§FS-config.3.4.10.2](../functional-spec/FS-config.md#34102-fetch-and-resolve) |
| `fetch` | string | — | The executable `grund fetch` runs to save a missing external snapshot. [§FS-config.3.4.10.2](../functional-spec/FS-config.md#34102-fetch-and-resolve) |

### `[scan]`

What the scan walks and reads. [§FS-config.3.5](../functional-spec/FS-config.md#35-scan--what-gets-scanned)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `include` | array of strings | `["requirements.md", "docs", "e2e", "src"]` | Directories and files to scan, from the config root; every kind's home is scanned as well. [§FS-config.3.5.7](../functional-spec/FS-config.md#357-include-is-scanned-from-the-config-root) |
| `exclude` | array of strings | `["target", "node_modules", ".git", "dist", "build", ".venv"]` | Directory names skipped at any depth. [§FS-config.3.5.16](../functional-spec/FS-config.md#3516-an-exclude-entry-containing--is-refused) |
| `extensions` | array of strings | `["md", "rs", "go", "java", "kt", "ts", "tsx", "js", "py", "c", "cpp", "swift", "scala", "rb", "php", "cs", "lisp", "scm", "clj", "sql", "hs", "lhs", "lua", "ada", "adb", "ads"]` | The file extensions the scan reads. [§FS-config.3.5.13](../functional-spec/FS-config.md#3513-an-extension-makes-a-file-readable-not-declarable) |
| `comment_prefixes` | array of strings | `["//", "#", ";", "--", "*", "/*"]` | The comment markers a citation or declaration in source code may follow. [§FS-config.3.5.14](../functional-spec/FS-config.md#3514-comment_prefixes-compose-with-extensions) |
| `docstring_python` | boolean | `true` | Read Python docstrings as well as `#` comments. [§FS-config.3.5](../functional-spec/FS-config.md#35-scan--what-gets-scanned) |
| `respect_gitignore` | boolean | `true` | Skip what `.gitignore` and other ignore files exclude. [§FS-config.3.5.15](../functional-spec/FS-config.md#3515-respect_gitignore--the-ignore-files) |

### `[output]`

How reports are printed. [§FS-config.3.6](../functional-spec/FS-config.md#36-output--report-format)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `format` | `"text"` \| `"json"` | `"text"` | The default report format, as `--format`. [§FS-config.3.6](../functional-spec/FS-config.md#36-output--report-format) |
| `color` | `"auto"` \| `"always"` \| `"never"` | — | Reserved: accepted and kept, with no effect yet. [§FS-config.3.6](../functional-spec/FS-config.md#36-output--report-format) |
| `relative_paths` | boolean | `true` | Report paths relative to the config root (`true`) or to where the command ran, as `--path-base`. [§FS-config.3.6.1](../functional-spec/FS-config.md#361-relative_paths--false--the-cli-base) |

### `[fmt]`

What `grund fmt` leaves alone. [§FS-config.3.10](../functional-spec/FS-config.md#310-fmt--suppressing-the-rewrite)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `exclude` | array of strings | `[]` | Files `grund fmt` never rewrites: gitignore-style globs from the config root. [§FS-config.3.10.1](../functional-spec/FS-config.md#3101-entries-are-gitignore-style-globs) |

### `[fmt.cross_refs]`

Whether `grund fmt` wraps citations in Markdown links, and the anchor style it writes. [§FS-config.3.7](../functional-spec/FS-config.md#37-fmtcross_refs--cross-reference-emission)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `enabled` | boolean | `true` | `false` stops `grund fmt` from wrapping citations in Markdown links. [§FS-config.3.7](../functional-spec/FS-config.md#37-fmtcross_refs--cross-reference-emission) |
| `anchor_format` | `"github"` \| `"gitlab"` \| `"mkdocs"` \| `"pandoc"` \| `"none"` | `"github"` | The heading-anchor style of the links `grund fmt` writes. [§FS-config.3.7](../functional-spec/FS-config.md#37-fmtcross_refs--cross-reference-emission) |

### `[workspace]`

The sub-projects a workspace root checks, each in its own namespace. [§FS-config.3.8](../functional-spec/FS-config.md#38-workspace--sub-project-members)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `members` | array of strings | `[]` | Member directories, or a directory ending in `/*` for all its children, each checked as its own project. [§FS-config.3.8](../functional-spec/FS-config.md#38-workspace--sub-project-members) |
| `optional_members` | array of strings | `[]` | Members that may be absent, such as an uninitialized submodule. [§FS-config.3.8](../functional-spec/FS-config.md#38-workspace--sub-project-members) |
| `include_root` | boolean | `true` | Whether the root is checked as a project too. [§FS-config.3.8](../functional-spec/FS-config.md#38-workspace--sub-project-members) |

### `[citations]`

Citation direction rules: one `[citations.<KIND>]` table per citing kind. [§FS-config.3.9](../functional-spec/FS-config.md#39-citations--citation-direction-rules)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `default` | `"must"` \| `"should"` \| `"may"` \| `"should-not"` \| `"must-not"` | — | What any kind may cite when no rule names the target; absent means `may`. [§FS-config.3.9.4](../functional-spec/FS-config.md#394-defaults-and-precedence) |

### `[citations.<KIND>]`

What a citing kind (or place) must, should, may, should not and must not cite. [§FS-config.3.9](../functional-spec/FS-config.md#39-citations--citation-direction-rules)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `default` | `"must"` \| `"should"` \| `"may"` \| `"should-not"` \| `"must-not"` | — | What this kind may cite when no list names the target, overriding `[citations] default`. [§FS-config.3.9.4](../functional-spec/FS-config.md#394-defaults-and-precedence) |
| `must` | array of strings | `[]` | Kinds each declaration (or file, for a place) must cite; a miss is an error. [§FS-config.3.9.1](../functional-spec/FS-config.md#391-levels) |
| `should` | array of strings | `[]` | Kinds it should cite; a miss is a suggestion. [§FS-config.3.9.1](../functional-spec/FS-config.md#391-levels) |
| `may` | array of strings | `[]` | Kinds it may cite, even under a stricter default. [§FS-config.3.9.1](../functional-spec/FS-config.md#391-levels) |
| `should-not` | array of strings | `[]` | Kinds it should not cite; a citation is a suggestion. [§FS-config.3.9.1](../functional-spec/FS-config.md#391-levels) |
| `must-not` | array of strings | `[]` | Kinds it must not cite; a citation is an error. [§FS-config.3.9.1](../functional-spec/FS-config.md#391-levels) |

## Version 2

Version 2: keys grouped by concern — `[schema]` for what exists, `[rules]` for how it relates, `[presentation]` for what grund writes. Every constraint takes one strength: `must` is an error, `warn` a warning, `should` a suggestion shown by `grund check --suggestions`, and `may` is not checked.

### Top level

| Key | Type | Default | What it sets |
|---|---|---|---|
| `grund_config_version` | `2` | — | The config format version: 2 selects the version-2 reader. [§FS-config.5](../functional-spec/FS-config.md#5-schema-versioning) |
| `project_name` | string | — | The project's name; in a workspace, also its alias in `§alias/<ID>` citations. [§FS-config.3](../functional-spec/FS-config.md#3-keys) |

### `[schema]`

What exists: the citation marker, the ID grammar, the sources, the note and lead measures, and the kinds. [§FS-config-v2.schema.1](../functional-spec/FS-config-v2.md#schema1-schema)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `marker` | string | `"§"` | The character(s) that mark a citation, as v1 `[reference] marker`. [§FS-config-v2.schema.1](../functional-spec/FS-config-v2.md#schema1-schema) |
| `shorthand` | `"canonical"` \| `"accepted"` | `"canonical"` | Whether number-only citations are written out in full (`canonical`) or kept as written (`accepted`). [§FS-config-v2.schema.1](../functional-spec/FS-config-v2.md#schema1-schema) |
| `id_format` | string | `"{kind}-{slug}"` | The ID template, as v1 `[id] format`. [§FS-config-v2.schema.1](../functional-spec/FS-config-v2.md#schema1-schema) |
| `slug_pattern` | string | `"[a-z][a-z0-9-]*"` | Regex for `{slug}`; the default starts with a letter. [§FS-config-v2.schema.1](../functional-spec/FS-config-v2.md#schema1-schema) |
| `number_pattern` | string | `"\\d+"` | Regex for `{number}`. [§FS-config-v2.schema.1](../functional-spec/FS-config-v2.md#schema1-schema) |
| `section_separator` | string | `"."` | The separator between an ID and its section path. [§FS-config-v2.schema.1](../functional-spec/FS-config-v2.md#schema1-schema) |
| `heading_depth` | `"must"` \| `"warn"` \| `"should"` \| `"may"` | `"must"` | How strongly a heading whose depth does not match its section path is reported. [§FS-config-v2.schema.1](../functional-spec/FS-config-v2.md#schema1-schema) |

### `[schema.sources]`

The sources a version-2 project is read from. [§FS-config-v2.schema.sources](../functional-spec/FS-config-v2.md#schemasources-schemasources)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `languages` | array of `"markdown"` | — | The source languages a version-2 project is read in. Required, and `["markdown"]` is the only value today. [§FS-config-v2.schema.sources](../functional-spec/FS-config-v2.md#schemasources-schemasources) |
| `exclude` | refused | — | Reserved; refused today. [§FS-config-v2.schema.sources](../functional-spec/FS-config-v2.md#schemasources-schemasources) |
| `respect_gitignore` | boolean | `true` | Skip what `.gitignore` and other ignore files exclude. [§FS-config-v2.schema.sources](../functional-spec/FS-config-v2.md#schemasources-schemasources) |
| `extensions` | refused | — | Reserved; refused today. |
| `definitions` | refused | — | Reserved; refused today. |

### `[schema.notes.lines]`

How many lines an inline note may take, by strength. [§FS-config-v2.schema.measures](../functional-spec/FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `must` | integer | `3` | The most lines an inline note may take; more is an error. [§FS-config-v2.schema.measures](../functional-spec/FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys) |
| `warn` | integer | — | Lines past which an inline note is a warning. [§FS-config-v2.schema.measures](../functional-spec/FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys) |
| `should` | integer | — | Lines past which an inline note is a suggestion. [§FS-config-v2.schema.measures](../functional-spec/FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys) |
| `may` | integer | — | A line guideline that is not checked. [§FS-config-v2.schema.measures](../functional-spec/FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys) |

### `[schema.notes.columns]`

How many columns an inline note may take, by strength. [§FS-config-v2.schema.measures](../functional-spec/FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `must` | integer | `100` | The most columns an inline note may take; more is an error. [§FS-config-v2.schema.measures](../functional-spec/FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys) |
| `warn` | integer | — | Columns past which an inline note is a warning. [§FS-config-v2.schema.measures](../functional-spec/FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys) |
| `should` | integer | — | Columns past which an inline note is a suggestion. [§FS-config-v2.schema.measures](../functional-spec/FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys) |
| `may` | integer | — | A column guideline that is not checked. [§FS-config-v2.schema.measures](../functional-spec/FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys) |

### `[schema.notes.text]`

Whether an inline note may carry text beside its citation. [§FS-config-v2.schema.measures](../functional-spec/FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `must` | boolean | `true` | `true` lets an inline note carry text beside its citation; `false` allows the citation alone. [§FS-config-v2.schema.measures](../functional-spec/FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys) |

### `[schema.notes.layout]`

Where the citation sits inside a note; write one strength only. [§FS-config-v2.schema.measures](../functional-spec/FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `must` | `"any"` \| `"citation-first-colon"` | — | The required layout; a deviation is an error. [§FS-config-v2.schema.measures](../functional-spec/FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys) |
| `warn` | `"any"` \| `"citation-first-colon"` | — | The required layout; a deviation is a warning. [§FS-config-v2.schema.measures](../functional-spec/FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys) |
| `should` | `"any"` \| `"citation-first-colon"` | — | The required layout; a deviation is a suggestion. [§FS-config-v2.schema.measures](../functional-spec/FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys) |
| `may` | `"any"` \| `"citation-first-colon"` | — | The layout, unchecked. [§FS-config-v2.schema.measures](../functional-spec/FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys) |

### `[schema.leads.words]`

How many words a declaration's lead may take, by strength. [§FS-config-v2.schema.measures](../functional-spec/FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `must` | integer | — | Words past which a declaration's lead is an error. [§FS-config-v2.schema.measures](../functional-spec/FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys) |
| `warn` | integer | — | Words past which a declaration's lead is a warning. [§FS-config-v2.schema.measures](../functional-spec/FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys) |
| `should` | integer | — | Words past which a declaration's lead is a suggestion. [§FS-config-v2.schema.measures](../functional-spec/FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys) |
| `may` | integer | — | A word guideline that is not checked. [§FS-config-v2.schema.measures](../functional-spec/FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys) |

### `[schema.kinds.<NAME>]`

One kind: its homes, its form, and its fields. [§FS-config-v2.schema.places](../functional-spec/FS-config-v2.md#schemaplaces-rows-and-their-places)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `files` | array of strings | `[]` | Single-file homes: every declaration is a heading in one of these documents. [§FS-config-v2.schema.places](../functional-spec/FS-config-v2.md#schemaplaces-rows-and-their-places) |
| `folders` | array of strings | `[]` | Folder homes: each declaration is the H1 of its own file under one of these directories. [§FS-config-v2.schema.places](../functional-spec/FS-config-v2.md#schemaplaces-rows-and-their-places) |
| `citable` | boolean | `true` | `false` makes this a place: it cites, but declares no IDs. [§FS-config-v2.schema.places](../functional-spec/FS-config-v2.md#schemaplaces-rows-and-their-places) |
| `scan` | boolean | `true` | `false` lists the kind in the Project map without walking it. [§FS-config-v2.schema.places](../functional-spec/FS-config-v2.md#schemaplaces-rows-and-their-places) |
| `index` | string or `false` | README.md (no E2E special case in v2) | The folder's index file, which must list every declaration; `false` opts out. [§FS-config-v2.schema.places](../functional-spec/FS-config-v2.md#schemaplaces-rows-and-their-places) |
| `id_format` | string | inherits `[schema] id_format` | This kind's ID template, overriding `[schema] id_format`. [§FS-config-v2.schema.places](../functional-spec/FS-config-v2.md#schemaplaces-rows-and-their-places) |
| `fetch` | string | — | The executable `grund fetch` runs to save a missing external snapshot. [§FS-config-v2.schema.places](../functional-spec/FS-config-v2.md#schemaplaces-rows-and-their-places) |
| `form` | `"prose"` \| `"value"` \| `"rule"` | `"prose"` | What a declaration is: ordinary `prose`, a first-class `value`, or an executable `rule`. [§FS-fields.declare.form](../functional-spec/FS-fields.md#declareform-form-is-a-row-key) |
| `closed` | `"must"` \| `"warn"` \| `"should"` \| `"may"` | `"may"` | How strongly a chapter that is neither a declared field nor a numbered section is reported. [§FS-fields.declare.closed](../functional-spec/FS-fields.md#declareclosed-closed-is-a-strength-on-the-row) |

### `[schema.kinds.<NAME>.fields.<field>]`

One field: a chapter of the kind, its presence, position and content. [§FS-fields.declare.field](../functional-spec/FS-fields.md#declarefield-a-field-is-a-table-under-its-kind)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `title` | string | — | The heading title the chapter must carry, exactly. [§FS-fields.declare.field](../functional-spec/FS-fields.md#declarefield-a-field-is-a-table-under-its-kind) |
| `presence` | `"must"` \| `"warn"` \| `"should"` \| `"may"` | `"may"` | How strongly a missing chapter is reported. [§FS-fields.declare.field](../functional-spec/FS-fields.md#declarefield-a-field-is-a-table-under-its-kind) |
| `position` | `"first"` \| `"last"` | — | Pin the chapter first or last among the declaration's sections. [§FS-fields.declare.field](../functional-spec/FS-fields.md#declarefield-a-field-is-a-table-under-its-kind) |
| `after` | string | — | Name of another field of the same kind this one comes directly after. [§FS-fields.declare.field](../functional-spec/FS-fields.md#declarefield-a-field-is-a-table-under-its-kind) |
| `content` | `"prose"` \| `"values"` | `"prose"` | `values` makes this chapter the kind's value chapter. [§FS-fields.declare.field](../functional-spec/FS-fields.md#declarefield-a-field-is-a-table-under-its-kind) |
| `one_of` | array of strings | — | The values a scalar chapter's title may take. [§FS-fields.declare.field](../functional-spec/FS-fields.md#declarefield-a-field-is-a-table-under-its-kind) |

### `[rules.citations]`

Citation direction rules: one `[rules.citations.<KIND>]` table per citing kind. [§FS-config-v2.rules.citations](../functional-spec/FS-config-v2.md#rulescitations-rulescitations)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `default` | `"may"` \| `"should-not"` \| `"warn-not"` \| `"must-not"` | may (effective; stored absent) | What a citing kind may cite when its rule does not name the target: `may`, or a prohibition at some strength. [§FS-config-v2.rules.citations.default](../functional-spec/FS-config-v2.md#rulescitationsdefault-a-default-may-forbid-never-oblige) |

### `[rules.citations.grounding]`

The grounding ladder: the unit, by strength, that must carry a citation. [§FS-config-v2.rules.grounding](../functional-spec/FS-config-v2.md#rulesgrounding-grounding-is-a-ladder-replaced-whole)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `must` | `"file"` \| `"h2"` \| `"h3"` \| `"h4"` \| `"h5"` \| `"h6"` | — | The unit (`file`, or every `h2`–`h6` section) that must carry a citation; a miss is an error. [§FS-config-v2.rules.grounding](../functional-spec/FS-config-v2.md#rulesgrounding-grounding-is-a-ladder-replaced-whole) |
| `warn` | `"file"` \| `"h2"` \| `"h3"` \| `"h4"` \| `"h5"` \| `"h6"` | — | The unit that must carry a citation; a miss is a warning. [§FS-config-v2.rules.grounding](../functional-spec/FS-config-v2.md#rulesgrounding-grounding-is-a-ladder-replaced-whole) |
| `should` | `"file"` \| `"h2"` \| `"h3"` \| `"h4"` \| `"h5"` \| `"h6"` | — | The unit that should carry a citation; a miss is a suggestion. [§FS-config-v2.rules.grounding](../functional-spec/FS-config-v2.md#rulesgrounding-grounding-is-a-ladder-replaced-whole) |
| `may` | `"file"` \| `"h2"` \| `"h3"` \| `"h4"` \| `"h5"` \| `"h6"` | — | A grounding unit that is not checked. [§FS-config-v2.rules.grounding](../functional-spec/FS-config-v2.md#rulesgrounding-grounding-is-a-ladder-replaced-whole) |

### `[rules.citations.<KIND>]`

What a citing kind (or place) must, warn, should, may, should not, warn not and must not cite. [§FS-config-v2.rules.citations](../functional-spec/FS-config-v2.md#rulescitations-rulescitations)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `default` | `"may"` \| `"should-not"` \| `"warn-not"` \| `"must-not"` | — | What this kind may cite when no list names the target: `may`, or a prohibition at some strength. [§FS-config-v2.rules.citations.default](../functional-spec/FS-config-v2.md#rulescitationsdefault-a-default-may-forbid-never-oblige) |
| `must` | array of strings | `[]` | Kinds this one must cite; a miss is an error. [§FS-config-v2.rules.citations](../functional-spec/FS-config-v2.md#rulescitations-rulescitations) |
| `warn` | array of strings | `[]` | Kinds this one must cite; a miss is a warning. [§FS-config-v2.rules.citations](../functional-spec/FS-config-v2.md#rulescitations-rulescitations) |
| `should` | array of strings | `[]` | Kinds this one should cite; a miss is a suggestion. [§FS-config-v2.rules.citations](../functional-spec/FS-config-v2.md#rulescitations-rulescitations) |
| `may` | array of strings | `[]` | Kinds this one may cite. [§FS-config-v2.rules.citations](../functional-spec/FS-config-v2.md#rulescitations-rulescitations) |
| `should-not` | array of strings | `[]` | Kinds this one should not cite; a citation is a suggestion. [§FS-config-v2.rules.citations](../functional-spec/FS-config-v2.md#rulescitations-rulescitations) |
| `warn-not` | array of strings | `[]` | Kinds this one must not cite; a citation is a warning. [§FS-config-v2.rules.citations](../functional-spec/FS-config-v2.md#rulescitations-rulescitations) |
| `must-not` | array of strings | `[]` | Kinds this one must not cite; a citation is an error. [§FS-config-v2.rules.citations](../functional-spec/FS-config-v2.md#rulescitations-rulescitations) |

### `[rules.citations.<KIND>.grounding]`

The same keys as `[rules.citations.grounding]`.

### `[rules.resolution]`

How strongly a fetched kind's missing snapshot is reported: `<KIND> = "must"` or `"warn"`. [§FS-config-v2.rules.resolution](../functional-spec/FS-config-v2.md#rulesresolution-rulesresolution)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `<KIND>` | `"must"` \| `"warn"` | — | The strength of a missing snapshot for this fetched kind. |

### `[presentation]`

The bytes grund writes: the trigger, the conversation opinion, kind titles and `grund fmt`. [§FS-config-v2.presentation](../functional-spec/FS-config-v2.md#presentation-the-presentation-concern)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `description` | string | — | A one-line description shown beside the project in generated workspace member lists. [§FS-config-v2.presentation](../functional-spec/FS-config-v2.md#presentation-the-presentation-concern) |
| `trigger` | string | `"$$"` | The typed sequence the editor and `grund fmt` turn into the marker. [§FS-config-v2.presentation](../functional-spec/FS-config-v2.md#presentation-the-presentation-concern) |
| `conversation` | `"link"` | — | The repository's opinion on how agents render citations in conversation: `link`, or absent for none. [§FS-config-v2.presentation](../functional-spec/FS-config-v2.md#presentation-the-presentation-concern) |
| `rules` | refused | — | Reserved; refused today. [§FS-config-v2.presentation](../functional-spec/FS-config-v2.md#presentation-the-presentation-concern) |

### `[presentation.kinds.<NAME>]`

One kind's display title. [§FS-config-v2.presentation](../functional-spec/FS-config-v2.md#presentation-the-presentation-concern)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `title` | string | — | The kind's title in the Project map, `grund list --summary` and editor hovers. [§FS-config-v2.presentation](../functional-spec/FS-config-v2.md#presentation-the-presentation-concern) |

### `[presentation.fmt]`

What `grund fmt` writes and leaves alone. [§FS-config-v2.presentation](../functional-spec/FS-config-v2.md#presentation-the-presentation-concern)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `anchors` | `"github"` | `"github"` | The heading-anchor style `grund fmt` writes in citation links; `github` is the only one today. [§FS-config-v2.presentation](../functional-spec/FS-config-v2.md#presentation-the-presentation-concern) |
| `links` | `"index"` | `"index"` | Where `grund fmt` points citation links; `index` is the only target today. [§FS-config-v2.presentation](../functional-spec/FS-config-v2.md#presentation-the-presentation-concern) |
| `exclude` | array of strings | `[]` | Files `grund fmt` never rewrites: gitignore-style globs, as v1 `[fmt] exclude`. [§FS-config-v2.presentation](../functional-spec/FS-config-v2.md#presentation-the-presentation-concern) |

### `[workspace]`

The sub-projects a workspace root checks, each in its own namespace. [§FS-config.3.8](../functional-spec/FS-config.md#38-workspace--sub-project-members)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `members` | array of strings | `[]` | Member directories, or globs of them, each checked as its own project. [§FS-config.3.8](../functional-spec/FS-config.md#38-workspace--sub-project-members) |
| `optional_members` | array of strings | `[]` | Members that may be absent, such as an uninitialized submodule. [§FS-config.3.8](../functional-spec/FS-config.md#38-workspace--sub-project-members) |
| `include_root` | boolean | `true` | Whether the root is checked as a project too. [§FS-config.3.8](../functional-spec/FS-config.md#38-workspace--sub-project-members) |
