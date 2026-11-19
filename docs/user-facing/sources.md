# Sources: which files grund reads

Configuration reference: [`[schema.sources]`](configuration.md#schemasources) covers the source readers, suffix mappings, exclusions and ignore behavior discussed here.

A version-2 `grund.toml` says in one table, `[schema.sources]`, which files grund
reads, which reader and comment syntax each one takes, and which comments may
declare an ID. The contract is [§FS-config-v2.schema.sources](../functional-spec/FS-config-v2.md#schemasources-schemasources); this guide covers
choosing languages, remapping a suffix, defining a language of your own, keeping
generated trees out, and reading a diagram fence's citations. The runnable
[`examples/sources`](../../examples/sources/) does all of it in one small tree.

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
txt = "markdown"
```

The file takes the target language whole: its reader, its comment hosts, its
doc profile and, for a built-in, its multi-line strings and quote reading. So
`txt = "rust"` reads Rust raw strings and lifetimes in a `.txt` file, and
`h = "c"` reads no C++ raw string in a `.h` file, because the built-in `c`
reads no multi-line form ([§FS-languages.multi-line-string.suffix](../functional-spec/FS-languages.md#multi-line-stringsuffix-which-row-and-which-quote-reading-a-file-reads)). A mapping also overrides the epoch, so `txt = "markdown"` reads a
`.txt` file as Markdown. `.md` is the one suffix that belongs to Markdown: `md`
maps only to a `document` language, such as `md = "v1_markdown"`. Mapping it
elsewhere is refused when the config loads, as it has been since grund 1.0.0
([§FS-config-v2.schema.sources.extensions](../functional-spec/FS-config-v2.md#schemasourcesextensions-schemasourcesextensions-maps-a-suffix-to-a-language)). A key is a bare final extension: no dot, no `/`, and no
glob character. The target must be a language `languages` admits; a mapping
admits nothing by itself.

## How a linked file is read

A file reached through a symlink is one file, and renaming the link changes
nothing about what it is read as ([§FS-scan.identity.reading](../functional-spec/FS-scan.md#identityreading-how-a-file-is-read-belongs-to-the-file-not-to-the-name-that-reached-it)):

1. A file inside the config root is read as its own name reads, when that name claims a reading.
2. A file whose own name claims nothing, or that lies outside the root, is read as the link's name reads.
3. Two links that read such a file differently are refused, and the file is read under neither.

Under `txt = "rust"`, then, `fs/spec.md -> real.txt` is Rust, because the
configuration reads `real.txt` as Rust and the `.md` on the link decides nothing:

```toml
[schema.sources]
languages = ["markdown", "rust"]
[schema.sources.extensions]
txt = "rust"
```

A name claims a reading when an admitted language reads it. So `docs/license.md -> ../LICENSE`
stays Markdown, because `LICENSE` claims nothing. Whether a name enters the scan
at all is still asked of that name: a hidden link, or one whose extension is
unlisted, stays unread whatever it points at.

A link whose reading moved earns a `reading-moved` warning that says what it was
read as and what it is read as now ([§FS-scan.checks.reading-moved](../functional-spec/FS-scan.md#checksreading-moved-a-link-is-read-as-its-targets-own-name-reads)). If the file should stay as it was,
rename the target or copy the file.

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

A definition has no key for multi-line strings or for how a `'` is read. A
`source` definition's files read the ones their own suffix names, with its built-in language rules: `java = "myjava"` still reads Java text blocks,
`rs = "myrs"` Rust raw strings and lifetimes, `hs = "myhs"` Haskell primes, and
a definition over `.txt` reads none. A definition over such a suffix cannot opt
out; the one way to read it without them is to map it to a built-in that has
none. A `document` or `data` definition's files read none
([§FS-languages.multi-line-string.suffix](../functional-spec/FS-languages.md#multi-line-stringsuffix-which-row-and-which-quote-reading-a-file-reads)).

## Which comments declare

`declarations = "doc-comments"`, the default, lets a source file declare an ID
only inside a comment its language calls documentation. Citations are
untouched either way: a marked citation in any comment resolves and is checked.

```rust
/// FS-existing: Real documentation
fn example() {
    // FS-existing: Ordinary root-ID note
    // See §FS-existing.
}
```

Only the documentation comment declares `FS-existing`; the second comment is
a note whose citation still resolves.

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
  `cl-def`, or is `ns`.
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

The default is `[]`; add generated directories such as `target/` or
`node_modules/` when the project needs them excluded. A pattern the parser refuses, such as an unclosed `[`, is
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
5. If an excluded file is a stub target, grund refuses the stub because it
   cannot read the target. This applies to exclusion patterns and enabled ignore files; `respect_gitignore = false`
   disables only ignore files. Being outside a configured place alone is allowed, and formatter exclusions
   affect rewrites only. An explicit scan
   root is read, but exclusions still filter what lies below it ([stub target
   rules](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-config-v2.md#schemasourcesexcludestubs-a-pattern-is-the-exclude-reason-a-stubs-target-is-not-read),
   [scan roots](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-config-v2.md#schemasourcesexcluderoots-how-exclude-composes-with-the-other-filters)).

`[presentation.fmt] exclude` uses the same grammar with a different effect: it
keeps `grund fmt` from rewriting a file that the scan still reads.

## Reading the citations in a diagram fence

A Markdown fence is an illustration by default: grund reads no citation in it,
which is what lets a document show citation syntax in a ```` ```text ```` block
without the `<§>` escape. A diagram is the exception that hurts. A Mermaid
flowchart whose labels cite the components they draw renders as a diagram only
inside a ```` ```mermaid ```` fence, and there its citations go unchecked: a
label naming a renamed declaration passes, and the declaration the diagram still
cites is reported as never cited.

`fenced` names the fences whose citations are read
([§FS-config-v2.schema.sources.fenced](../functional-spec/FS-config-v2.md#schemasourcesfenced-fenced-names-the-fences-whose-citations-are-read)):

```toml
[schema.sources]
fenced = ["mermaid"]
```

- **Matching.** A fence is selected when its tag, the first word of its info
  string, is in the list
  ([§FS-languages.fence.language](../functional-spec/FS-languages.md#fencelanguage-a-fences-tag-and-the-fences-fenced-selects)).
  ```` ```mermaid ````, ```` ```mermaid title="Flow" ```` and `~~~mermaid` are
  selected; ```` ```Mermaid ````, ```` ```{.mermaid} ```` and a fence with no
  info string are not. Tags compare case-sensitively, and a tag outside
  `[A-Za-z0-9][A-Za-z0-9_+.#-]*`, or one written twice, is refused at the
  `fenced` line.
- **What is read.** A selected fence's citations are read as a `<pre>` block's
  are: each one resolves, counts as a use in `refs`, `cover` and the unused
  warning, and is reported where it dangles. Nothing else in it is read. A
  heading-shaped line there declares nothing and opens no section, and a
  `grund:fmt` line toggles nothing.
- **Escapes.** To show an ID in a selected fence without citing it, escape it
  as `<§>AR-example`, as in a `<pre>` block or front matter. A fence whose tag
  the list does not name stays an illustration and needs no escape.
- **`fmt` leaves every fence alone.** No `grund fmt` pass and no editor
  expansion rewrites a byte inside a fence, because a Markdown link inside a
  Mermaid label breaks the diagram. Under `shorthand = "canonical"`, a shorthand
  citation in a selected fence is therefore reported and not expanded: write the
  full ID there by hand.
- **Scope.** The list is the project's own. It applies to every file the
  Markdown reader reads and to no fence inside a source file's comment block. A
  language definition named like a tag does not read its fences, and a tag
  admits no file. A workspace member selects its own fences.

The block `grund init` writes follows the list: with `fenced = ["mermaid"]`, its
rule on mentioning an ID ends "or use a fenced code block not tagged `mermaid`."
The runnable [`examples/fenced-citations`](../../examples/fenced-citations/)
reads one diagram.
