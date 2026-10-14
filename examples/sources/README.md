# Sources: which files a v2 project reads, and which comments declare

This runnable mini-repository demonstrates a version-2 `[schema.sources]` table ([§FS-config-v2.schema.sources](../../docs/functional-spec/FS-config-v2.md#schemasources-schemasources)). It is for a polyglot repository that wants to say exactly which files grund reads and how. Here that means three things: a suffix the epoch does not know, a comment syntax of the project's own, and a generated directory the scan must not read. Open `repo/grund.toml` first, then the three source files it names.

The configuration, in full:

```toml
grund_config_version = 2
project_name = "sources-example"

[schema.sources]
# Markdown, C, and a language of this project's own; nothing else is read.
languages = ["markdown", "c", "notes"]
# The v2 default: only a doc comment declares an ID.
declarations = "doc-comments"
# Gitignore-style, from this file's directory: every `generated` directory.
exclude = ["generated/"]

[schema.sources.extensions]
# `.h` is not an epoch suffix; read it as C.
h = "c"
note = "notes"

[schema.sources.definitions.notes]
class = "source"
comment_prefixes = ["%%"]
```

The `[schema.kinds.*]` rows that follow give `FS` the folder `docs/fs`, `AR` the folder `src` and `TERM` the folder `notes`.

Run it from the repository root:

```bash
grund list examples/sources/repo
echo $?    # 0
```

The list on stdout is the golden below; stderr is empty:

```text
AR-tokenizer     src/parser.h:1  The tokenizer splits a line into tokens
FS-parser        docs/fs/FS-parser.md:1  The parser reads one expression per line
TERM-expression  notes/glossary.note:1  What one line of input reads as
```

Each line is one setting at work:

- **A remapped suffix.** `src/parser.h` is read as C because `h = "c"` maps it ([§FS-config-v2.schema.sources.extensions](../../docs/functional-spec/FS-config-v2.md#schemasourcesextensions-schemasourcesextensions-maps-a-suffix-to-a-language)). Without the entry no admitted language claims `.h`, and the file is not read at all.
- **A custom language.** `notes/glossary.note` declares through `%%`, a prefix no built-in language has. The `notes` definition supplies it, and `note = "notes"` maps the suffix to it ([§FS-config-v2.schema.sources.definitions](../../docs/functional-spec/FS-config-v2.md#schemasourcesdefinitions-schemasourcesdefinitionsname-defines-a-language)). Its doc profile is `none`, so every `%%` comment may declare.
- **A doc comment declares, an ordinary comment does not.** `src/parser.h` holds two declaration-shaped lines. The first is `/// AR-tokenizer: …`, a doc comment. The second is a `// FS-parser: …` note inside a function body. Under `declarations = "doc-comments"` only the first declares ([§FS-config-v2.schema.sources.declarations](../../docs/functional-spec/FS-config-v2.md#schemasourcesdeclarations-declarations-picks-which-comments-declare)). The note's citation of `FS-parser` still resolves.
- **An excluded directory.** `src/generated/parser_gen.h` declares `AR-tokenizer` a second time. `generated/` keeps the scan out of every `generated` directory, so the copy is never read ([§FS-config-v2.schema.sources.exclude](../../docs/functional-spec/FS-config-v2.md#schemasourcesexclude-exclude-is-a-gitignore-style-glob-list)).

`grund check examples/sources/repo` prints `success`. Change `declarations` to `"any-comment"` and `exclude` to `[]`, the reading a v1 project has, and the same tree fails:

```text
docs/fs/FS-parser.md:1: error: duplicate declaration of FS-parser (also declared at src/parser.h:7)
src/generated/parser_gen.h:1: error: duplicate declaration of AR-tokenizer (also declared at src/parser.h:1)
src/parser.h:7: error: FS-parser declares kind FS inside AR home src
```

The trade-offs:

- `languages` is the whole admitted set. Naming it replaces the epoch's defaults, so a language you leave out stops being read. An omitted `languages` reads every default language of the epoch ([§FS-config-v2.schema.sources.languages](../../docs/functional-spec/FS-config-v2.md#schemasourceslanguages-languages-admits-languages-and-only-it-does)).
- v2 has no built-in `exclude` names. `target`, `node_modules` and the rest are read unless the list names them.
- A `none` profile cannot tell documentation from a note. Give a definition a `doc_comments` profile when its comments should be held to the doc-comment rule.

The [sources guide](../../docs/user-facing/sources.md) has the full epoch table, the limits of each comment host, and how a v1 configuration migrates.
