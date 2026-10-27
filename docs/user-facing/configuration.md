<!-- Rendered from schemas/grund.schema.json by scripts/render_config_reference.py: edit the schema, then run the script. -->

# The `grund.toml` reference

Every key `grund.toml` accepts, table by table: first version 1, which a file
without `grund_config_version` is read as and keeps loading as, then version 2,
which a file selects with `grund_config_version = 2` and which `grund init` writes,
holding only what differs from a default
([§FS-init.2.4.3](../functional-spec/FS-init.md#243-every-written-key-differs-from-its-default)). A key you leave out
takes the default shown. `grund config validate` checks a file against these keys and
`grund config show` prints the configuration in effect, defaults included, in the
file's own version; for version 2 its tables keyed by a kind follow the order the
file writes its `[schema.kinds]` rows
([§FS-config.4.2.4](../functional-spec/FS-config.md#424-the-order-show-prints-in)).
`grund config explain '[TABLE] key'` gives one key's entry from the grund you run,
in your file's version, with no page needed ([config explain](explain.md)).

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
| `project_name` | string | — | The project's name; in a workspace, also its alias in `§alias/<ID>` citations. [§FS-config.3](../functional-spec/FS-config.md#3-keys-version-1) |
| `project_description` | string | — | One-line description shown next to the project in generated workspace member lists. [§FS-config.3](../functional-spec/FS-config.md#3-keys-version-1) |

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

`inline_style`, the note budgets and `inline_note_layout` are rendered into the managed block's `rules` section, so after changing one `grund check` reports the block stale until `grund init` runs ([§FS-inline-citation-style.5.5](../functional-spec/FS-inline-citation-style.md#55-rendered-not-live)).

### `[id]`

The ID grammar: the shape of every declared ID and section path. [§FS-config.3.2](../functional-spec/FS-config.md#32-id--id-grammar)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `format` | string | `"{kind}-{number}-{slug}"` | The ID template: `{kind}` plus `{number}`, `{slug}` or both; everything else is literal. [§FS-config.3.2](../functional-spec/FS-config.md#32-id--id-grammar) |
| `section_separator` | string | `"."` | The separator between an ID and its section path. [§FS-config.3.2.2](../functional-spec/FS-config.md#322-section_separator-must-stay-distinguishable) |
| `number_pattern` | string | `"\\d+"` | Regex for `{number}`. [§FS-config.3.2.4](../functional-spec/FS-config.md#324-each-pattern-compiles-on-its-own) |
| `slug_pattern` | string | `"[a-z0-9][a-z0-9-]*"` | Regex for `{slug}`. [§FS-config.3.2.4](../functional-spec/FS-config.md#324-each-pattern-compiles-on-its-own) |
| `named_sections` | boolean | `false` | Enable named section components, such as `## goals:`, matching `[a-z][a-z0-9-]*`: a name at any depth, under a name or a number. [§FS-config.3.2.7](../functional-spec/FS-config.md#327-named_sections--the-gate-for-explicit-section-names) |
| `section_heading_levels` | `"strict"` \| `"warn"` \| `"loose"` | `"strict"` | How a heading whose depth does not match its section path is reported: an error, a warning, or not at all. [§FS-config.3.3.2](../functional-spec/FS-config.md#332-section_heading_levels--heading-depth-against-path-depth) |

Numeric IDs keep their written digits in reports. With `format = "{kind}-{number}"`, a heading `# ADR-0001: Use Postgres` is listed as `ADR-0001`, and a citation of it links to `#adr-0001-use-postgres`. `ADR-1` resolves to that same declaration; its spelling does not change the heading's anchor. This applies to the default grammar and per-kind overrides in both config versions ([§FS-config.3.2.8](../functional-spec/FS-config.md#328-numeric-spelling-survives-reading-and-reporting)). `grund id ADR "Use Redis"` still allocates with a minimum of three digits (`ADR-002` when the highest number is 1); pass `--width 4` to mint `ADR-0002`. Existing padding does not set allocation width ([§FS-id.1.2](../functional-spec/FS-id.md#12-number-width)).

### `[[kinds]]`

One row per kind: where its declarations live and how they are read. `kinds = []` declares no kinds; leaving the key out keeps the built-in ones. [§FS-config.3.4](../functional-spec/FS-config.md#34-kinds--recognized-kinds)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `kind` | string | — | The kind's name: the ID prefix of a citable kind, its `[citations.<kind>]` key and its `--kind` selector. [§FS-config.3.4](../functional-spec/FS-config.md#34-kinds--recognized-kinds) |
| `prefix` | refused | — | Removed in grund 0.13.0 and refused: rename it to `kind`. [§FS-config.3.4.6](../functional-spec/FS-config.md#346-prefix-the-former-spelling-of-kind-removed-in-0130) |
| `folder` | string | — | A folder home: each declaration is the H1 of its own file under this directory. [§FS-config.3.4](../functional-spec/FS-config.md#34-kinds--recognized-kinds) |
| `file` | string | — | A single-file home: every declaration is a heading in this document. It is read only if its extension is in `[scan] extensions`, or, on a `values = true` row, as its value source by the value pass when it ends in `.json`, `.yaml` or `.yml`; otherwise `grund check` warns at this line. [§FS-config.3.4](../functional-spec/FS-config.md#34-kinds--recognized-kinds) |
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
| `include` | array of strings | `["requirements.md", "docs", "e2e", "src"]` | Directories and files to scan, from the config root; every kind's home is scanned as well. A file named here is read only if its extension is in `extensions`; when it is not, `grund check` warns at this line. [§FS-config.3.5.7](../functional-spec/FS-config.md#357-include-is-scanned-from-the-config-root) |
| `exclude` | array of strings | `["target", "node_modules", ".git", "dist", "build", ".venv"]` | Directory names skipped at any depth; a stub whose target sits under one is broken, because a target the scan does not read declares nothing. [§FS-config.3.5.16](../functional-spec/FS-config.md#3516-an-exclude-entry-containing--is-refused) |
| `extensions` | array of strings | `["md", "rs", "go", "java", "kt", "ts", "tsx", "js", "py", "c", "cpp", "swift", "scala", "rb", "php", "cs", "lisp", "scm", "clj", "sql", "hs", "lhs", "lua", "ada", "adb", "ads", "h", "cc", "cxx", "hpp", "hh", "hxx", "m", "mm", "kts", "jsx", "mjs", "cjs", "dart", "r", "R", "sh", "bash", "zsh", "cl", "el", "ss", "rkt", "cljs", "cljc"]` | The file extensions the scan reads. The default is `md` plus every extension grund's built-in language table names, so a language grund learns is read without a config change. [§FS-config.3.5.13](../functional-spec/FS-config.md#3513-an-extension-makes-a-file-readable-not-declarable) |
| `comment_prefixes` | array of strings | `["//", "#", ";", "--", "*", "/*"]` | The comment markers a citation or declaration in source code may follow. [§FS-config.3.5.14](../functional-spec/FS-config.md#3514-comment_prefixes-compose-with-extensions) |
| `docstring_python` | boolean | `true` | Read Python docstrings as well as `#` comments. [§FS-config.3.5](../functional-spec/FS-config.md#35-scan--what-gets-scanned) |
| `respect_gitignore` | boolean | `true` | Skip what `.gitignore` and other ignore files exclude, stub targets included: a stub whose target they exclude is broken, because a target the scan does not read declares nothing. [§FS-config.3.5.15](../functional-spec/FS-config.md#3515-respect_gitignore--the-ignore-files) |

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
| `must` | array of strings | `[]` | Kinds each declaration (or file, for a place) must cite; a miss is an error. A file of a place or the complement that cites nothing is not checked by this key; `require_grounding` makes it a finding. [§FS-config.3.9.1](../functional-spec/FS-config.md#391-levels) |
| `should` | array of strings | `[]` | Kinds it should cite; a miss is a suggestion. A file of a place or the complement that cites nothing is not checked by this key; `require_grounding` makes it a finding. [§FS-config.3.9.1](../functional-spec/FS-config.md#391-levels) |
| `may` | array of strings | `[]` | Kinds it may cite, even under a stricter default. [§FS-config.3.9.1](../functional-spec/FS-config.md#391-levels) |
| `should-not` | array of strings | `[]` | Kinds it should not cite; a citation is a suggestion. [§FS-config.3.9.1](../functional-spec/FS-config.md#391-levels) |
| `must-not` | array of strings | `[]` | Kinds it must not cite; a citation is an error. [§FS-config.3.9.1](../functional-spec/FS-config.md#391-levels) |

## Version 2

Version 2: keys grouped by concern — `[schema]` for what exists, `[rules]` for how it relates, `[presentation]` for what grund writes. Every constraint takes one strength: `must` is an error, `warn` a warning, `should` a suggestion shown by `grund check --suggestions`, and `may` is not checked.

### Top level

| Key | Type | Default | What it sets |
|---|---|---|---|
| `grund_config_version` | `2` | — | The config format version: 2 selects the version-2 reader. [§FS-config.5](../functional-spec/FS-config.md#5-schema-versioning) |
| `project_name` | string | — | The project's name; in a workspace, also its alias in `§alias/<ID>` citations. [§FS-config.3](../functional-spec/FS-config.md#3-keys-version-1) |

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
| `languages` | array of strings | the epoch's default languages | The languages a version-2 project reads: built-in names from the v2 epoch, or names defined under `[schema.sources.definitions]`. Omitted, the epoch's default set; `[]` reads nothing. [§FS-config-v2.schema.sources.languages](../functional-spec/FS-config-v2.md#schemasourceslanguages-languages-admits-languages-and-only-it-does) |
| `declarations` | `"doc-comments"` \| `"any-comment"` | `"doc-comments"` | Which comment blocks of a source file may declare an ID: only those the language's doc profile calls documentation, or every comment as in version 1. Citations are read in every comment either way. [§FS-config-v2.schema.sources.declarations](../functional-spec/FS-config-v2.md#schemasourcesdeclarations-declarations-picks-which-comments-declare) |
| `exclude` | array of strings | `[]` | Gitignore-style patterns, resolved against the config root, for paths the scan never reads. A stub whose target a pattern matches is broken, and the pattern is named as the reason its target is not read. Unlike version 1 there are no built-in names. [§FS-config-v2.schema.sources.exclude](../functional-spec/FS-config-v2.md#schemasourcesexclude-exclude-is-a-gitignore-style-glob-list) |
| `respect_gitignore` | boolean | `true` | Skip what `.gitignore` and other ignore files exclude, stub targets included: a stub whose target they exclude is broken, because a target the scan does not read declares nothing. [§FS-config-v2.schema.sources](../functional-spec/FS-config-v2.md#schemasources-schemasources) |

### `[schema.sources.extensions]`

Maps a file suffix, written without its dot, to an admitted language; an entry overrides the epoch's own assignment of that suffix. [§FS-config-v2.schema.sources.extensions](../functional-spec/FS-config-v2.md#schemasourcesextensions-schemasourcesextensions-maps-a-suffix-to-a-language)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `<suffix>` | string | — | The admitted language files with this suffix are read as. [§FS-config-v2.schema.sources.extensions](../functional-spec/FS-config-v2.md#schemasourcesextensions-schemasourcesextensions-maps-a-suffix-to-a-language) |

### `[schema.sources.definitions.<name>]`

One defined language. [§FS-config-v2.schema.sources.definitions](../functional-spec/FS-config-v2.md#schemasourcesdefinitions-schemasourcesdefinitionsname-defines-a-language)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `class` | `"document"` \| `"data"` \| `"source"` | required | The reader: Markdown, the JSON value-file reader, or the source reader through the comment hosts. Required. [§FS-config-v2.schema.sources.definitions](../functional-spec/FS-config-v2.md#schemasourcesdefinitions-schemasourcesdefinitionsname-defines-a-language) |
| `comment_prefixes` | array of strings | `[]` | The literal prefixes a comment is read behind, longest first. [§FS-config-v2.schema.sources.definitions](../functional-spec/FS-config-v2.md#schemasourcesdefinitions-schemasourcesdefinitionsname-defines-a-language) |
| `doc_comments` | `"none"` \| `"c-family"` \| `"python"` \| `"go"` \| `"ruby"` \| `"shell"` \| `"sql"` \| `"ldoc"` \| `"haddock"` \| `"roxygen"` \| `"lisp"` | `"none"` | The frozen doc profile that says which comments are documentation; `none` lets every comment declare. [§FS-config-v2.schema.sources.definitions](../functional-spec/FS-config-v2.md#schemasourcesdefinitions-schemasourcesdefinitionsname-defines-a-language) |
| `docstrings` | `"off"` \| `"python"` | `"off"` | Whether triple-quoted Python docstrings are read as comments. [§FS-config-v2.schema.sources.definitions](../functional-spec/FS-config-v2.md#schemasourcesdefinitions-schemasourcesdefinitionsname-defines-a-language) |

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

Every `[schema.notes]` table is rendered into the managed block's `rules` section, so after a change to one `grund check` reports the block stale until `grund init` runs ([§FS-inline-citation-style.5.5](../functional-spec/FS-inline-citation-style.md#55-rendered-not-live)).

### `[schema.leads.words]`

How many words a declaration's lead may take, by strength. [§FS-config-v2.schema.measures](../functional-spec/FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `must` | integer | — | Words past which a declaration's lead is an error. [§FS-config-v2.schema.measures](../functional-spec/FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys) |
| `warn` | integer | — | Words past which a declaration's lead is a warning. [§FS-config-v2.schema.measures](../functional-spec/FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys) |
| `should` | integer | — | Words past which a declaration's lead is a suggestion. [§FS-config-v2.schema.measures](../functional-spec/FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys) |
| `may` | integer | — | A word guideline that is not checked. [§FS-config-v2.schema.measures](../functional-spec/FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys) |

### `[schema.kinds.<NAME>]`

One row: its places, its form, and its fields. A row may list several places; a file listed in `files` belongs to its row, otherwise the deepest folder holding it decides, and a file in no place belongs to `code`. The row named `code` is the complement: it declares no IDs, and listing places on it narrows the scan to every place written. [§FS-config-v2.schema.places](../functional-spec/FS-config-v2.md#schemaplaces-rows-and-their-places)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `files` | array of strings | `[]` | Files from the config root, in written order. A listed file belongs to this row even inside another row's folder. On a citable row with no folders, every declaration is a heading in one of these documents. A place is normalized (`./a/` is `a`), must stay inside the config root, and belongs to one row. [§FS-config-v2.schema.places](../functional-spec/FS-config-v2.md#schemaplaces-rows-and-their-places) |
| `folders` | array of strings | `[]` | Folders from the config root, in written order. Each declaration is the H1 of its own file under one of them; a file in nested folders belongs to the deepest. A place is normalized, must stay inside the config root and outside every workspace member, belongs to one row, and a folder may not link out of the project root. [§FS-config-v2.schema.places](../functional-spec/FS-config-v2.md#schemaplaces-rows-and-their-places) |
| `citable` | boolean | `true` | `false` makes this a place: it cites, but declares no IDs. A non-citable row must list a place unless it is `code`, which is non-citable by name; `citable = true` on `code` is refused. [§FS-config-v2.schema.places](../functional-spec/FS-config-v2.md#schemaplaces-rows-and-their-places) |
| `scan` | boolean | `true` | `false` lists the kind in the Project map without walking any of its places; a scanned place nested under it is still walked. Refused on `code`: narrow its places or use `[schema.sources] exclude`. [§FS-config-v2.schema.places](../functional-spec/FS-config-v2.md#schemaplaces-rows-and-their-places) |
| `index` | string or `false` | README.md (no E2E special case in v2) | The index file each folder keeps, which must list every declaration that folder owns; `false` opts out. [§FS-config-v2.schema.places](../functional-spec/FS-config-v2.md#schemaplaces-rows-and-their-places) |
| `id_format` | string | inherits `[schema] id_format` | This kind's ID template, overriding `[schema] id_format`. [§FS-config-v2.schema.places](../functional-spec/FS-config-v2.md#schemaplaces-rows-and-their-places) |
| `fetch` | string | — | The executable `grund fetch` runs to save a missing external snapshot. [§FS-config-v2.schema.places](../functional-spec/FS-config-v2.md#schemaplaces-rows-and-their-places) |
| `form` | `"prose"` \| `"value"` \| `"rule"` | `"prose"` | What a declaration is: ordinary `prose`, a first-class `value`, or an executable `rule`. [§FS-fields.declare.form](../functional-spec/FS-fields.md#declareform-form-is-a-row-key) |
| `closed` | `"must"` \| `"warn"` \| `"should"` \| `"may"` | `"may"` | How strongly a chapter that is neither a declared field nor a numbered section is reported. [§FS-fields.declare.closed](../functional-spec/FS-fields.md#declareclosed-closed-is-a-strength-on-the-row) |

### `[schema.kinds.<NAME>.fields.front-matter]`

The file's front matter as a field: the first declaration's YAML block, its top-level keys the field's sections. It has no heading, so it takes no title, position, after, content or one_of. [§FS-fields.front-matter.config](../functional-spec/FS-fields.md#front-matterconfig-declaring-the-field-and-its-keys)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `presence` | `"must"` \| `"warn"` \| `"should"` \| `"may"` | `"may"` | How strongly a declaration without front matter is reported. [§FS-fields.front-matter.config](../functional-spec/FS-fields.md#front-matterconfig-declaring-the-field-and-its-keys) |
| `closed` | `"must"` \| `"warn"` \| `"should"` \| `"may"` | `"may"` | How strongly a top-level key the field does not declare is reported. [§FS-fields.front-matter.config](../functional-spec/FS-fields.md#front-matterconfig-declaring-the-field-and-its-keys) |

### `[schema.kinds.<NAME>.fields.front-matter.keys.<key>]`

One top-level key of the front matter: its presence and the values it may take. [§FS-fields.front-matter.config](../functional-spec/FS-fields.md#front-matterconfig-declaring-the-field-and-its-keys)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `presence` | `"must"` \| `"warn"` \| `"should"` \| `"may"` | `"may"` | How strongly a block without the key is reported. [§FS-fields.front-matter.config](../functional-spec/FS-fields.md#front-matterconfig-declaring-the-field-and-its-keys) |
| `one_of` | array of strings | — | The values the key may take, each a one-line scalar. [§FS-fields.front-matter.config](../functional-spec/FS-fields.md#front-matterconfig-declaring-the-field-and-its-keys) |

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
| `must` | array of strings | `[]` | Kinds this one must cite; a miss is an error. A file of a place or the complement that cites nothing is not checked by this key; `[rules.citations.<ROW>.grounding]` makes it a finding. [§FS-config-v2.rules.citations](../functional-spec/FS-config-v2.md#rulescitations-rulescitations) |
| `warn` | array of strings | `[]` | Kinds this one must cite; a miss is a warning. A file of a place or the complement that cites nothing is not checked by this key; `[rules.citations.<ROW>.grounding]` makes it a finding. [§FS-config-v2.rules.citations](../functional-spec/FS-config-v2.md#rulescitations-rulescitations) |
| `should` | array of strings | `[]` | Kinds this one should cite; a miss is a suggestion. A file of a place or the complement that cites nothing is not checked by this key; `[rules.citations.<ROW>.grounding]` makes it a finding. [§FS-config-v2.rules.citations](../functional-spec/FS-config-v2.md#rulescitations-rulescitations) |
| `may` | array of strings | `[]` | Kinds this one may cite. [§FS-config-v2.rules.citations](../functional-spec/FS-config-v2.md#rulescitations-rulescitations) |
| `should-not` | array of strings | `[]` | Kinds this one should not cite; a citation is a suggestion. [§FS-config-v2.rules.citations](../functional-spec/FS-config-v2.md#rulescitations-rulescitations) |
| `warn-not` | array of strings | `[]` | Kinds this one must not cite; a citation is a warning. [§FS-config-v2.rules.citations](../functional-spec/FS-config-v2.md#rulescitations-rulescitations) |
| `must-not` | array of strings | `[]` | Kinds this one must not cite; a citation is an error. [§FS-config-v2.rules.citations](../functional-spec/FS-config-v2.md#rulescitations-rulescitations) |

For a negative default that forbids a kind's own marked index entry, see
[index-entry repair advice](citation-directions.md) for the `may` edit and how
explicit prohibitions and independent rule sentences differ
([§FS-check.3.12.2](../functional-spec/FS-check.md#3122-repair-advice-preserves-a-required-index-entry)).

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
| `rules` | `"home"` \| `"root"` | `"home"` | Where each row's agent guidance is written: `home`, the `AGENTS.md` of the deepest directory holding the row's places; `root`, the root entrypoint, as a version-1 config renders it. [§FS-config-v2.presentation](../functional-spec/FS-config-v2.md#presentation-the-presentation-concern) |

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

For another string value of `anchors` or `links`, the refusal names the supplied
value and the only assignment this binary executes. For example, `links = "off"`
on line 5 produces the following diagnostic. Quotes, backslashes and control
characters in the value are escaped so the message stays on one line.
[§FS-config-v2.rollout.fmt-values](../functional-spec/FS-config-v2.md#rolloutfmt-values-a-recognized-formatting-key-names-the-value-it-cannot-execute)

```text
error: grund.toml:5: `[presentation.fmt] links` is part of the v2 format but not supported by this grund yet; unsupported value "off"; executes only links = "index"
```

### `[workspace]`

The sub-projects a workspace root checks, each in its own namespace. [§FS-config.3.8](../functional-spec/FS-config.md#38-workspace--sub-project-members)

| Key | Type | Default | What it sets |
|---|---|---|---|
| `members` | array of strings | `[]` | Member directories, or globs of them, each checked as its own project. [§FS-config.3.8](../functional-spec/FS-config.md#38-workspace--sub-project-members) |
| `optional_members` | array of strings | `[]` | Members that may be absent, such as an uninitialized submodule. [§FS-config.3.8](../functional-spec/FS-config.md#38-workspace--sub-project-members) |
| `include_root` | boolean | `true` | Whether the root is checked as a project too. [§FS-config.3.8](../functional-spec/FS-config.md#38-workspace--sub-project-members) |
