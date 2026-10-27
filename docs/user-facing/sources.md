# Sources: which files grund reads

Exclusion also refuses a stub target before it can lend content
([§FS-config-v2.schema.sources.exclude.stubs](../functional-spec/FS-config-v2.md#schemasourcesexcludestubs-a-pattern-is-the-exclude-reason-a-stubs-target-is-not-read), [§FS-declarations.stubs.target.3](../functional-spec/FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing)).
This applies to v1 directory names and v2 patterns, plus enabled ignore rules.
`respect_gitignore = false` disables ignore only; an explicit scan root outruns
descent exclusions. Being outside v1 `include` alone remains allowed.
Formatter exclusions only prevent rewrites and do not refuse a target.

A version-2 `grund.toml` says in one table, `[schema.sources]`, which files grund
reads, which reader and comment syntax each one takes, and which comments may
declare an ID. The contract is [§FS-config-v2.schema.sources](../functional-spec/FS-config-v2.md#schemasources-schemasources); this guide covers
choosing languages, remapping a suffix, defining a language of your own, keeping
generated trees out, and moving a version-1 file over. The runnable
[`examples/sources`](../../examples/sources/) does all of it in one small tree.

## Version 1 and version 2

A file without `grund_config_version`, or with `1`, is read by the version-1
reader. Nothing here changes it: `[scan] extensions`, `comment_prefixes`,
`exclude` and `docstring_python` keep their meaning, and any comment in
declaration position still declares ([§FS-config.5.2](../functional-spec/FS-config.md#52-every-older-version-keeps-its-meaning)). A file that writes
`grund_config_version = 2` has no `[scan]` table. Its sources are the table
below, and the reader refuses a v1 key rather than guess what it meant.

The differences that change what a tree reads:

| | Version 1 | Version 2 |
|---|---|---|
| What is read | the `[scan] extensions` list | the admitted `languages`, each with its own suffixes |
| Comment syntax | one global `comment_prefixes` list | each language's own hosts |
| Which comments declare | any comment in declaration position | only doc comments, by default |
| Built-in exclusions | `target`, `node_modules`, `.git`, `dist`, `build`, `.venv` | none |
| `exclude` entries | directory or file names | gitignore-style globs from the config root |

## The smallest table

```toml
grund_config_version = 2
project_name = "example"
[schema.sources]
[schema.kinds.FS]
folders = ["docs"]
```

An empty `[schema.sources]` admits every default language of the epoch below,
declares only in doc comments, excludes nothing, and honours `.gitignore`. Run
`grund config show` to see it written out: the admitted set is computed from the
configuration alone, never from the files on disk
([§FS-config-v2.schema.sources.effective](../functional-spec/FS-config-v2.md#schemasourceseffective-the-admitted-set-is-read-from-the-configuration-alone)).

## Choosing languages

`languages` is the whole admitted set ([§FS-config-v2.schema.sources.languages](../functional-spec/FS-config-v2.md#schemasourceslanguages-languages-admits-languages-and-only-it-does)):

```toml
[schema.sources]
languages = ["markdown", "rust", "python"]
```

An explicit list replaces the defaults, so a language you leave out is not read,
and `languages = []` reads nothing at all. A name is refused at its line when it
is neither built in nor defined, or when it is written twice. An opt-in language
is read only when the list names it, which is why a newer grund never widens an
omitted `languages`.

### The epoch

These are v2's built-in languages, fixed on 2026-10-06
([§FS-config-v2.schema.sources.epoch](../functional-spec/FS-config-v2.md#schemasourcesepoch-the-v2-language-epoch)). Suffixes are matched case-sensitively,
without the dot.

| Admission | Class | Language: suffixes | Comment hosts | Doc profile |
|---|---|---|---|---|
| default | document | `markdown`: `md` | Markdown headings | none |
| default | data | `json`: `json` | the JSON value-file reader | none |
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

A **document** language is read as Markdown and a **data** language by the JSON
value reader only. A **source** language is read through its comment hosts.
Any other suffix, such as `h`, `hpp`, `jsx`, `mjs` or `kts`, is read only
through a mapping.

No built-in language claims `yaml` or `yml`, so a `.yaml` or `.yml` file at a
value kind's home is read by the YAML value reader, as its extension says
([§FS-config-v2.schema.sources.values](../functional-spec/FS-config-v2.md#schemasourcesvalues-a-value-home-reads-a-file-by-its-language)). Map either suffix to a language
and the language decides instead: a `data` language reads the file as JSON, and
any other makes it no value source. Outside a value home no YAML file is read
as a value.

## Remapping a suffix

`[schema.sources.extensions]` maps a suffix to an admitted language
([§FS-config-v2.schema.sources.extensions](../functional-spec/FS-config-v2.md#schemasourcesextensions-schemasourcesextensions-maps-a-suffix-to-a-language)):

```toml
[schema.sources]
languages = ["markdown", "c", "cpp", "javascript"]
[schema.sources.extensions]
h = "c"
hpp = "cpp"
mjs = "javascript"
```

The file takes the target language whole: its reader, its comment hosts and its
doc profile. A mapping also overrides the epoch, so `md = "notes"` takes `.md`
away from Markdown. A key is a bare final extension: no dot, no `/`, and no
glob character. The target must be a language `languages` admits; a mapping
admits nothing by itself.

## Defining a language

`[schema.sources.definitions.<name>]` adds a language that `languages` may admit
and a mapping may point a suffix at ([§FS-config-v2.schema.sources.definitions](../functional-spec/FS-config-v2.md#schemasourcesdefinitions-schemasourcesdefinitionsname-defines-a-language)):

```toml
[schema.sources]
languages = ["markdown", "rust", "notes", "tex"]
[schema.sources.extensions]
note = "notes"
tex = "tex"
[schema.sources.definitions.notes]
class = "source"
comment_prefixes = ["%%"]
[schema.sources.definitions.tex]
class = "source"
comment_prefixes = ["%"]
```

| Key | Values | Default |
|---|---|---|
| `class` | `document`, `data`, `source` | required |
| `comment_prefixes` | literal, nonempty strings, each once | `[]` |
| `doc_comments` | `none`, `c-family`, `python`, `go`, `ruby`, `shell`, `sql`, `ldoc`, `haddock`, `roxygen`, `lisp` | `none` |
| `docstrings` | `off`, `python` | `off` |

A name matches `[a-z][a-z0-9_]*` and cannot reuse a built-in name. The reader
refuses a definition that cannot work, at the line that breaks it:

- a `source` definition with no comment prefix and no `docstrings = "python"`;
- a `data` definition with comment prefixes, or a `document` or `data` one with
  a doc profile or docstrings;
- `doc_comments = "python"` without `docstrings = "python"`;
- a profile none of the prefixes can spell, such as `c-family` without `//` or
  `/*`, or `ruby` without `#`.

A prefix is literal. It matches byte for byte at the first non-blank column, and
the longest prefix wins.

## Which comments declare

`declarations = "doc-comments"`, the default, lets a source file declare an ID
only inside a comment its language calls documentation; `"any-comment"` keeps the
version-1 reading ([§FS-config-v2.schema.sources.declarations](../functional-spec/FS-config-v2.md#schemasourcesdeclarations-declarations-picks-which-comments-declare)). Citations are
untouched either way: a marked citation in any comment resolves and is checked.

```rust
/// FS-existing: Real documentation
fn example() {
    // FS-existing: Ordinary root-ID note
    // See §FS-existing.
}
```

Version 1 declares `FS-existing` twice here and reports a duplicate. Version 2
declares it once, at line 1, and the second comment is a note whose citation
resolves.

What each doc profile accepts ([§FS-inline-citation-style.1.1.4](../functional-spec/FS-inline-citation-style.md#114-declaration-applicability)):

- `c-family`: a `///` or `//!` run, and a `/**` or `/*!` block. Not a plain
  `//` or `/* … */`, a PHP `#`, a comment trailing code, `////` or `/**/`.
- `python`: a docstring. Not a `#` comment.
- `ldoc`: a `--` run opening with exactly `---`. `haddock`: a `--` run whose
  first content is `|` or `^`. `roxygen`: a `#` run opening `#'`.
- `go`, `ruby`, `shell`, `sql`: a block directly above a definition, such as
  Go's `func` or Ruby's `def`, with no blank line between, or the file's leading
  block.
- `lisp`: a `;` block directly above a form whose head starts with `def` or
  `cl-def`, or is `ns`. It exists for `grund config migrate --to 2`, which
  carries version 1's Lisp extensions with it.
- `none`: every comment behind the language's hosts, so Lisp, Scheme, Clojure,
  Ada and a definition without `doc_comments` can still declare.

These are recognition limits, not parsing. grund reads a comment's prefix and,
for the position profiles, one following line. A Go comment separated from its
`func` by a blank line, or a Ruby `private def`, does not declare. A
declaration-shaped line in an ineligible comment is plain prose. It draws no
finding of its own, and it is not reported as a near miss.

## Excluding paths

`exclude` takes gitignore-style patterns, resolved from the directory that holds
`grund.toml` ([§FS-config-v2.schema.sources.exclude](../functional-spec/FS-config-v2.md#schemasourcesexclude-exclude-is-a-gitignore-style-glob-list)):

```toml
[schema.sources]
exclude = ["generated", "/docs/archive", "src/**/vendor/**", "build/", "!build/keep.md"]
```

- `generated` matches a file or directory of that name at any depth.
- `/docs/archive` is anchored: the root's `docs/archive`, not `lib/docs/archive`.
- `build/` matches directories only.
- `*`, `?`, `[…]`, `**` and `\` escapes mean what they do in `.gitignore`, and a
  later `!` re-includes. A `!` cannot rescue a file below a directory that is
  already pruned, so `build/keep.md` above stays unread.

The default is `[]`. A v2 project that wants v1's `target` or `node_modules`
skipped writes them. A pattern the parser refuses, such as an unclosed `[`, is
refused at the `exclude` line.

How it composes with the other filters ([§FS-config-v2.schema.sources.exclude.roots](../functional-spec/FS-config-v2.md#schemasourcesexcluderoots-how-exclude-composes-with-the-other-filters)):

1. A scan root, whether a row's place or a path you pass on the command line, is
   read even when a pattern names it. Only what lies below it is filtered.
2. `respect_gitignore = false` turns the ignore files off and leaves `exclude` in
   force. A `!` in one never overrides the other.
3. A row with `scan = false` is still reached by an explicit path or `--full`,
   and `exclude` still applies under `--full`.
4. A root outside the config root is matched by its spelling relative to the
   root. Anchored patterns never reach outside it.

`[presentation.fmt] exclude` uses the same grammar with a different effect: it
keeps `grund fmt` from rewriting a file that the scan still reads.

## Moving a version-1 file

`grund config migrate --to 2` is planned, not shipped (agent-grounds/grund#461);
[§FS-config-v2.schema.sources.migration](../functional-spec/FS-config-v2.md#schemasourcesmigration-what-a-v1-source-configuration-becomes) fixes what it will write. To keep the
meaning when moving by hand:

1. Write `declarations = "any-comment"`. That is version 1's reading, and you
   can switch to `"doc-comments"` once the duplicates it would expose are fixed.
2. Write `languages` out. For each v1 extension that has no epoch row, add a
   mapping, and a definition when no built-in language has its comment syntax.
3. For `.py` with `docstring_python = false`, define a language with
   `docstrings = "off"`, so a triple-quoted string keeps its raw reading.
4. Turn each `[scan] exclude` name into the directory glob `<name>/`.

The migration refuses, and so should you, where a v2 table cannot say the same
thing:

- a v1 `exclude` entry containing `/`. Version 1 never matched it, and as a glob
  it would start excluding files.
- a `.json` scanned as text while a kind's `values` also reads it as values. One
  file has one class.
- an extension or prefix outside what a mapping or definition accepts, or a
  source extension that would read no comment.
