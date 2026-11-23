# FS-languages: how a source file's lines are read

How a scanned file's lines are read, defined once for every command that reads them: which lines form a comment block, which bytes of a comment line are its wrapper rather than its text, how a Python docstring is read, what opens and closes a Markdown fence, which comment prefixes a file's extension brings, and which string literals are read as one literal across lines. Serves [§GOAL-polyglot-citation](../goals.md#goal-polyglot-citation-ids-cite-cleanly-from-anywhere-they-are-useful), which is why a declaration and a citation are read alike from a Markdown file, a line comment, a block comment and a docstring.

## terms: Terms

Leans on [§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (doc-comment) and [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope).

- **comment block** — The run of lines one comment construct spans: a run of line comments, one
  block comment, or one Python docstring. Its extent is [§FS-languages.comment-block](FS-languages.md#comment-block-the-comment-block).
- **comment wrapper** — The bytes of a comment line that say it is a comment rather than what it
  says, such as the prefix and the whitespace before it. Which bytes they are, and when a block
  comment's closer is among them, is [§FS-languages.comment-wrapper](FS-languages.md#comment-wrapper-the-comment-wrapper).
- **fence** — A Markdown fenced code block, from its opening delimiter line to its closing one,
  in a Markdown file or read through the comment wrapper in a comment block.

## requirements: What every reading of a file's lines holds to

What a reading of a file's lines holds to with the rest of `grund` is stated once, where it is shared: the same lines give the same body in the editor, on the command line and through the bindings ([§REQ-one-answer.text.parity](../requirements/REQ-one-answer.md#textparity-the-editor-answers-as-the-command-line-does), [§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)); every comment form the defaults claim is proved by a dangling citation planted in it ([§REQ-no-missed-citation.3](../requirements/REQ-no-missed-citation.md#3-proven-per-host-language)); and what `show` prints of a source declaration is its author's text less the comment wrapper ([§FS-show.requirements.authored-bytes](FS-show.md#requirementsauthored-bytes-a-slice-is-the-authors-bytes-selected-and-never-rewritten)).

### requirements.one-block: Every command reads a run of line comments as the same block

`show`'s slice, `list --size` and the enclosing declaration `refs` reports all read the comment block of [§FS-languages.comment-block](FS-languages.md#comment-block-the-comment-block), so a `//` note under a `///` declaration is a block of its own: outside that declaration's body, and its citations have no enclosing declaration. Decided in D17c of [§DF-one-answer-rulings-parse.decision](../decisions/functional/DF-one-answer-rulings-parse.md#decision-decision). Serves [§REQ-one-answer.sections.map](../requirements/REQ-one-answer.md#sectionsmap-every-surface-reads-one-section-map).

## comment-block: The comment block

A comment block is one of these forms:

- `//` / `///` / `//!` line comments: a run of adjacent lines whose first non-whitespace token is the same line-comment prefix. `#`, `;` and `--` line comments follow the same rule per prefix ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned) lists the prefix set).
- `/* … */` block comments, `/**`- and `/*!`-opened alike: from the opener line to the closer line, both included. The closer is the one that ends the declaration's own comment: a declaration written as a one-line `/* … */` closes on its own line, so its block is that line alone, and a `/* … */` comment on the next line is neither a section of the declaration nor part of its body.
- Python triple-quoted docstrings (`""" … """` / `''' … '''`): from the line of the opening triple-quote to the line of the matching close, both included.

Adjacency is broken by any line that is not part of the same block: a code line, a blank line, or a different comment style. An empty comment line — the prefix alone, such as `//` or `#`, with nothing after it — stays inside the run.

A declaration's block is the one that holds its declaration line, cut short by any other declaration: reading outward from the declaration line in either direction, if another declaration line of any ID is encountered, the block ends at the line before it.

## comment-wrapper: The comment wrapper

Once the block is found, its comment prefixes are removed line by line, so the output is plain prose:

- The whitespace in front of the comment prefix belongs to the comment's envelope, not to the body: it is dropped together with the prefix, then a single space following the prefix is dropped if present. The remainder of the line is kept verbatim, so indentation the author wrote after the prefix (a nested list item, an indented code block) survives.
- The prefix removed is the whole form the line was written in ([§FS-languages.prefixes.forms](FS-languages.md#prefixesforms-the-forms-a-prefix-stands-for)): behind `;` that is the line's whole run of semicolons, so `;; One route per path.` and `;;;; One route per path.` both show as `One route per path.`. A block of such lines is one block keyed on `;`, whatever run each line opens with ([§FS-languages.comment-block](FS-languages.md#comment-block-the-comment-block)).
- For block-style continuation lines, a leading ` * ` (with surrounding spaces) is removed if present. Lines that do not have it are kept as-is, and a `## 1. Sub` among them is a heading of the block: section `.1` for the scanner and every query, printed by `show` as written ([§DF-one-parse-answers-from-0-19.3](../decisions/functional/DF-one-parse-answers-from-0-19.md#3-the-sides-p02-and-p03-take)), including under a declaration that only a bare line declares ([§FS-declarations.line.block-comment](FS-declarations.md#lineblock-comment-a-line-inside-a-block-comment), [§DF-bare-block-comment-line-declares](../decisions/functional/DF-bare-block-comment-line-declares.md#df-bare-block-comment-line-declares-a-line-inside-a-block-comment-declares-with-or-without-its-gutter-behind-any-of-its-openers)).
- Trailing comment-close tokens (`*/`) on their own line are dropped entirely.
- Blank lines inside the block are preserved.

### comment-wrapper.headings: The wrapper of a heading line

A heading in a doc-comment sits behind the comment that carries it, whatever prefix that comment uses ([§FS-languages.prefixes](FS-languages.md#prefixes-comment_prefixes-compose-with-extensions)), and that envelope says where the body lives, not what the chapter is called, so it is never part of the label ([§FS-rules.5.1.1](FS-rules.md#511-a-chapters-display-name-is-the-label-its-author-wrote)).

The envelope has a closing half too. A block comment's closing `*/` on the heading's own line, and the whitespace before it, closes the comment that carries the heading, so it is no more part of the label than the ` * ` that opens the line: ` * ## terms: Terms */` closing a C or Java block comment is `chapter(c, terms, Terms)`, the fact `/// ## terms: Terms` gives, and ` * ## 1. Step */` is printed `## 1. Step` by `show`, with the `--toc` `title` `Step` ([§DF-one-answer-rulings-parse](../decisions/functional/DF-one-answer-rulings-parse.md#df-one-answer-rulings-parse-one-parse-gives-every-command-the-same-extents-comment-blocks-anchors-and-title-citations), D17d). A declaration line is a heading line in this sense: the one-line `/** FS-b: Beta */` is titled `Beta`, as `//! FS-b: Beta` is ([§FS-declarations.line.block-comment](FS-declarations.md#lineblock-comment-a-line-inside-a-block-comment)). Only the C-family closer is envelope, because only a C-family block comment carries a heading on a line that can close it. A Python docstring's heading is read as Markdown, and a Markdown heading is never trimmed, so an author who writes `*/` at the end of either keeps it in the label or title.

## docstring: Docstrings

For Python docstrings, no prefix is stripped: only the surrounding `"""` lines are skipped. Both `"""` and `'''` docstrings are read, and only where `docstring_python` enables Python triple-quoted-string scanning ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)).

## fence: Fences

A fence opens with at most three leading spaces followed by a run of at least three backticks or tildes; it closes only on a run of the **same character** at least as long as the opener, again with at most three leading spaces and only whitespace after the run. A backtick opener cannot carry a backtick in its info string. An unclosed fence runs to end of file.

### fence.comment-block: A fence inside a comment block hides its headings

Inside a comment block ([§FS-languages.comment-block](FS-languages.md#comment-block-the-comment-block)) a fence is read on each line's text with its comment wrapper off ([§FS-languages.comment-wrapper](FS-languages.md#comment-wrapper-the-comment-wrapper)), by the rule above: `/// ```text`, ` * ```text` and a docstring's own ```` ```text ```` line each open one, and the matching delimiter behind the same wrapper closes it. A fence never outlives the comment block it opens in: one open where the block ends closes there.

While such a fence is open, a line that would be a section or a plain heading of the block, numbered, named or plain at any level, is not a heading for any command ([§FS-declarations.heading](FS-declarations.md#heading-what-a-heading-is)), exactly as inside a Markdown fence ([§FS-show.2.5](FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example)): it ends no lead, section or body, `--toc` does not list it, `refs` does not name it as a citation's enclosing section, `fmt` does not link to it, it draws no `section-heading-level`, `duplicate-section` or `section-outside-declaration` finding ([§FS-declarations.checks.section-heading-level](FS-declarations.md#checkssection-heading-level-section-heading-level-mismatch), [§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path), [§FS-declarations.checks.section-outside-declaration](FS-declarations.md#checkssection-outside-declaration-section-outside-a-declaration)), and a citation of the coordinate it spells is a missing section ([§FS-check.3.2](FS-check.md#32-missing-section)).

The fence hides headings and nothing else: a citation inside it is read and checked ([§FS-citations.recognized.contexts](FS-citations.md#recognizedcontexts-contexts-read-as-neither-prose-nor-code) skips only a Markdown file's unselected fence), a `grund:fmt off` or `grund:fmt on` line inside it toggles ([§FS-fmt.2.5.2](FS-fmt.md#252-grundfmt-off--grundfmt-on--a-region-at-a-time)), `show` flattens a citation wrapper inside it ([§FS-show.2.5](FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example)), and a source declaration line inside it is read as it would be outside ([§FS-declarations.stubs.target.2](FS-declarations.md#stubstarget2-a-heading-inside-a-fence-of-the-target-declares-nothing)). Decided in [§DF-doc-comment-fenced-heading](../decisions/functional/DF-doc-comment-fenced-heading.md#df-doc-comment-fenced-heading-a-heading-inside-a-fence-in-a-comment-block-is-not-a-section).

### fence.language: A fence's tag, and the fences `fenced` selects

A Markdown fence's **tag** is the first word of its info string: the text after the opening run, with leading and trailing whitespace stripped, up to the first space or tab. It is compared literally and case-sensitively with each tag `[schema.sources] fenced` lists ([§FS-config-v2.schema.sources.fenced](FS-config-v2.md#schemasourcesfenced-fenced-names-the-fences-whose-citations-are-read)), and a fence whose tag the list holds is **selected**. With `fenced = ["mermaid"]`, ```` ```mermaid title="Architecture" ```` and `~~~mermaid` are selected, and ```` ```Mermaid ````, ```` ```{.mermaid} ```` and a fence with an empty info string are not. Only a Markdown file's fences are selected: a fence in a comment block has no tag ([§FS-languages.fence.comment-block](FS-languages.md#fencecomment-block-a-fence-inside-a-comment-block-hides-its-headings)).

A selected fence is a fence for everything but citations. Its citations are live, as a raw-text HTML block's are ([§FS-citations.recognized.contexts](FS-citations.md#recognizedcontexts-contexts-read-as-neither-prose-nor-code)), and the configured marker and the `<§>` escape read there as they do in prose. Nothing else in it is read: no line of it is a heading, a declaration, a section, a value, a link reference definition or a `grund:fmt` directive, and no `grund fmt` pass or editor expansion rewrites a byte of it ([§FS-fmt.2.3.3](FS-fmt.md#233-a-fenced-code-block-is-an-illustration), [§FS-fmt.2.5.2.2](FS-fmt.md#2522-what-is-not-a-directive), [§FS-lsp.1.4.3](FS-lsp.md#143-where-the-expansion-refuses)). Its opening and closing lines are inert, so a citation written in an info string is not read. Decided in [§DF-fenced-citations](../decisions/functional/DF-fenced-citations.md#df-fenced-citations-a-markdown-fence-is-read-for-citations-only-when-schemasources-fenced-names-its-tag).

## prefixes: `comment_prefixes` compose with `extensions`

The two lists compose: adding `sql` without `--`, or `--` without `sql` (or another extension using that prefix), does not enable SQL doc-comments. Every default comment prefix has a path through the default extension list: `//` pairs with the extensions of [§FS-inline-citation-style.1.1.2](FS-inline-citation-style.md#112-languages)'s C-family row; `#` pairs with Python, Ruby, R, and shell extensions; `;` pairs with Lisp, Scheme, and Clojure extensions; `--` pairs with SQL, Haskell, Lua, and Ada extensions; and `*` / `/*` are block-comment continuation and opener forms in the C-family extensions. Any line whose first non-whitespace run is a configured prefix is eligible to host a declaration heading or a citation, and so is a line led by one of its forms ([§FS-languages.prefixes.forms](FS-languages.md#prefixesforms-the-forms-a-prefix-stands-for)) and any line inside a block a `/*` opened, with or without its ` * ` ([§FS-declarations.line.block-comment](FS-declarations.md#lineblock-comment-a-line-inside-a-block-comment)). The two keys themselves are [§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned).

This composition is version 1's. A version-2 file has no global prefix list: each language brings its own hosts, a built-in one from the epoch and a defined one from its `comment_prefixes`, and a file takes the hosts of the language its name resolves to ([§FS-config-v2.schema.sources](FS-config-v2.md#schemasources-schemasources)).

### prefixes.forms: The forms a prefix stands for

Four default prefixes stand for more than their own bytes:

| configured | stands for |
|---|---|
| `//` | `///`, `//!`, `//` |
| `/*` | `/**`, `/*!`, `/*` |
| `--` | `---`, `-- \|`, `--\|`, `-- ^`, `--^`, `--` |
| `;` | any run of semicolons: `;`, `;;`, `;;;`, `;;;;`, … |

A form is followed by whitespace or by the end of the line, as the prefix itself is, and a line it leads is in declaration position exactly as the same line behind the bare prefix ([§FS-declarations.line](FS-declarations.md#line-what-a-line-in-declaration-position-declares)): `--- FS-b: Beta` (Lua's LDoc), `-- | FS-b: Beta` (Haskell's Haddock) and `;;; FS-b: Beta` (Lisp) are in declaration position as `-- FS-b: Beta` and `; FS-b: Beta` are. A `## 1. Sub` behind a `--` or `;` form is a section heading wherever the same line behind the bare prefix is one.

The forms hang off the configured prefix, not the file's language: `--- FS-b:` declares in a SQL file too, a project that removed `--` or `;` from `comment_prefixes` gets none of its forms, and a prefix outside this table, such as a custom `%`, stands only for itself. Which blocks of these forms are doc comments rather than notes is [§FS-inline-citation-style.1.1.2](FS-inline-citation-style.md#112-languages). Decided in [§DF-line-prefix-forms-declare](../decisions/functional/DF-line-prefix-forms-declare.md#df-line-prefix-forms-declare-lua-haskell-and-lisp-doc-comment-forms-declare-and-a-lisp-doc-block-is-not-a-note).

### prefixes.haskell-block: A Haskell block comment hosts no declaration

Haskell's block comment `{- … -}`, the Haddock `{-| … -}` included, is no prefix and no form of one, so no line inside it is in declaration position: `{-| FS-b: Beta` declares nothing. The line form `-- |` declares ([§FS-languages.prefixes.forms](FS-languages.md#prefixesforms-the-forms-a-prefix-stands-for)). A marked citation inside the block is read and checked as anywhere else. Decided in [§DF-line-prefix-forms-declare](../decisions/functional/DF-line-prefix-forms-declare.md#df-line-prefix-forms-declare-lua-haskell-and-lisp-doc-comment-forms-declare-and-a-lisp-doc-block-is-not-a-note).

## multi-line-string: A string literal that spans lines

Which string literals are read as one literal across line breaks, defined once for every reading that asks whether a column of a source line is inside a string ([§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule)). Serves [§REQ-no-wrong-citation.2](../requirements/REQ-no-wrong-citation.md#2-no-false-alarms) and [§REQ-no-data-loss.2](../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own). Decided in [§DF-multi-line-string-literals](../decisions/functional/DF-multi-line-string-literals.md#df-multi-line-string-literals-a-string-literal-that-spans-lines-is-one-literal-for-a-fixed-set-of-forms).

The per-line quote toggle of [§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule) never carries across a line break: it flips on `'`, `"` and `` ` `` and starts every line neutral ([§DF-multi-line-string-literals.2.1](../decisions/functional/DF-multi-line-string-literals.md#21-what-carries-is-a-form-never-the-toggle)). What carries is a **recognized multi-line form**: one of the fixed openers of [§FS-languages.multi-line-string.forms](FS-languages.md#multi-line-stringforms-the-recognized-forms), in a file that reads the form's row ([§FS-languages.multi-line-string.suffix](FS-languages.md#multi-line-stringsuffix-which-row-and-which-quote-reading-a-file-reads)), with that form's exact closer.

A form changes only the lines that begin inside it. Such a line is inside a string literal from its first column up to the end of the form's closer; when the closer is on that line, the rest of the line after it is read by the per-line toggle, starting neutral, and when it is not, the whole line is inside the literal. The opening line and every other line are judged exactly as they are without this rule.

Forms are found in file order: once a form opens, nothing inside it opens another, and the search for the next opener resumes after its closer. Being inside a form changes only the answer to "is this column inside a string literal". Which lines are in declaration position ([§FS-declarations.line](FS-declarations.md#line-what-a-line-in-declaration-position-declares)), where a `/* … */` block opens and closes, and which lines form a comment block ([§FS-languages.comment-block](FS-languages.md#comment-block-the-comment-block)) are read as they are without it.

### multi-line-string.forms: The recognized forms

| Files | Form | Opener | Closer |
|---|---|---|---|
| `rs` | raw string | `r`, `br` or `cr`, then *n* `#` (*n* ≥ 0), then `"` | `"` followed by *n* `#` |
| `rs` | ordinary string | `"` | the next `"` not preceded by an odd run of `\` |
| `java` `kt` `kts` `scala` `swift` `dart` | triple-quoted | `"""`; in `dart` also `'''` | the next run of the same three characters |
| `cs` | raw string | a run of three or more `"`, optionally preceded by `$` characters | the next run of exactly as many `"` |
| `cs` | verbatim string | `@"`, `$@"` or `@$"` | a `"` that is not immediately followed by another `"` — `""` is a quote inside the string |
| `cpp` `cc` `cxx` `h` `hpp` `hh` `hxx` | raw string | `R"`, optionally prefixed `u8`, `u`, `U` or `L`, then a delimiter of at most 16 characters other than space, `(`, `)` and `\`, then `(` | `)`, the same delimiter, `"` |
| `go` | raw string | `` ` `` | the next `` ` `` |
| `js` `jsx` `mjs` `cjs` `ts` `tsx` | template literal | `` ` `` | the next `` ` `` not preceded by an odd run of `\` and not inside an interpolation |
| `py` | triple-quoted string | `"""` or `'''`, optionally prefixed as [§FS-citations.recognized.string-literals.assigned-data](FS-citations.md#recognizedstring-literalsassigned-data-assigned-python-triple-quoted-data) lists, that is not the first non-whitespace text on its line | the next unescaped run of the same three characters |
| `sh` `bash` `zsh` | heredoc | `<<` or `<<-`, optional spaces or tabs, then a word — an identifier, bare or wrapped in `'…'` or `"…"` — where the `<<` is neither preceded nor followed by another `<` | a line that is exactly the word; after `<<-`, leading tabs are dropped first |
| `php` | heredoc or nowdoc | `<<<`, optional spaces or tabs, then an identifier, bare or wrapped in `'…'` or `"…"` | a line whose first non-whitespace text is the identifier, not followed by an identifier character |

Four clarifications hold the table to one reading:

1. **A prefix is a whole word.** The letters before a Rust, C++ or Python opener are its prefix only when no identifier character comes before them: `bar"` opens a Rust ordinary string, not a raw one.
2. **In Rust, `'` opens nothing.** A lifetime, a character literal and an apostrophe are never an opener, and a `"` written as the character literal `'"'` is not one either.
3. **A heredoc's body starts on the next line.** The rest of the opener's own line is code, so `cat <<EOF | sort` keeps `| sort` live; the same holds for PHP. Only the first heredoc opened on a line is carried.
4. **A template literal tracks its interpolations.** Inside a template, `${` opens an interpolation and `{` and `}` nest within it, counted lexically; a `` ` `` inside an interpolation opens a nested template rather than closing the outer one, and the nested template has its own interpolations. The template closes at a `` ` `` when no interpolation is open. If the nesting does not balance before a closer is found, the template opens nothing ([§FS-languages.multi-line-string.bound](FS-languages.md#multi-line-stringbound-what-keeps-a-misread-opener-from-swallowing-a-file)).

In a file whose docstrings are read ([§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule)) a docstring ([§FS-languages.docstring](FS-languages.md#docstring-docstrings)) and an assigned-data span ([§FS-citations.recognized.string-literals.assigned-data](FS-citations.md#recognizedstring-literalsassigned-data-assigned-python-triple-quoted-data)) keep their own rules and win wherever they apply; the Python row adds only the triple-quoted string that neither of them claims, such as `call("""`, and applies whatever `[scan] docstring_python` says.

### multi-line-string.suffix: Which row and which quote reading a file reads

A file reads at most one row of the table in [§FS-languages.multi-line-string.forms](FS-languages.md#multi-line-stringforms-the-recognized-forms), and one **quote reading**: how the opener search of [§FS-languages.multi-line-string.bound](FS-languages.md#multi-line-stringbound-what-keeps-a-misread-opener-from-swallowing-a-file) and the comment search of [§FS-citations.recognized.contexts](FS-citations.md#recognizedcontexts-contexts-read-as-neither-prose-nor-code) read a `'`. One suffix names both, and which suffix that is depends on how the file is read:

- **A file read through a built-in language** reads that language's own row and quote reading, the ones its epoch suffixes name ([§FS-config-v2.schema.sources.epoch](FS-config-v2.md#schemasourcesepoch-the-v2-language-epoch)). That holds over a mapped suffix too, because a mapping takes a built-in whole ([§FS-config-v2.schema.sources.extensions](FS-config-v2.md#schemasourcesextensions-schemasourcesextensions-maps-a-suffix-to-a-language)): `h = "cpp"` reads C++ raw strings in a `.h` file, and `h = "c"` reads none there, because C has no row.
- **Every other file read as source** reads the row and quote reading its own final extension names: a version-1 file, a file read through a `source` definition ([§FS-config-v2.schema.sources.definitions](FS-config-v2.md#schemasourcesdefinitions-schemasourcesdefinitionsname-defines-a-language)), and a name no admitted language claims. A definition has no key for a row or a quote reading, so a `source` definition over a suffix the table names reads that row and cannot opt out of it, as a version-1 file cannot: `java = "myjava"` reads Java text blocks.
- **A Markdown file, and a file read through a `document` or `data` definition,** reads no row and no quote reading.

Only Rust and Haskell give a quote reading of their own: `rs`, where a character or byte literal is one unit and a lifetime or label is no quote at all ([§FS-languages.multi-line-string.bound](FS-languages.md#multi-line-stringbound-what-keeps-a-misread-opener-from-swallowing-a-file)), and `hs` and `lhs`, where a `'` after an identifier character is a Haskell prime and opens no literal ([§FS-citations.recognized.contexts](FS-citations.md#recognizedcontexts-contexts-read-as-neither-prose-nor-code)). Every other file reads `'` with the per-line toggle of [§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule). Decided in [§DF-defined-language-string-forms](../decisions/functional/DF-defined-language-string-forms.md#df-defined-language-string-forms-a-file-read-through-a-source-definition-reads-the-multi-line-strings-and-quotes-its-own-suffix-names).

### multi-line-string.bound: What keeps a misread opener from swallowing a file

Three rules bound every form, so that a wrongly detected opener costs at most the reading the file has without this rule and never hides a citation ([§DF-multi-line-string-literals.2.2](../decisions/functional/DF-multi-line-string-literals.md#22-two-bounds-so-a-misread-opener-costs-nothing)):

1. **An opener counts only in code.** It does not count where the per-line toggle of [§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule) already has a quote open, after a comment prefix that opens a line comment there ([§FS-languages.prefixes](FS-languages.md#prefixes-comment_prefixes-compose-with-extensions)), inside a `/* … */` block from its `/*` to its `*/` (in `rs` `kt` `kts` `scala` `swift` `dart`, where blocks nest, the `*/` that balances it), inside a Python docstring, inside an assigned-data span, or, for a `<<`, inside a shell arithmetic `(( … ))`. The search finds a `/*` in code wherever it starts, mid-line included, and reads blocks only to find openers: which lines host declarations stays as [§FS-declarations.line.block-comment](FS-declarations.md#lineblock-comment-a-line-inside-a-block-comment) says. In two languages the search reads a quote-like byte as the language does:
   - **In `rs`,** a character or byte literal, a `'` followed by one character or one escape and a `'`, such as `'x'`, `'\n'`, `'\''`, `'"'` or `b'x'`, is skipped as one unit, whatever it holds. A `'` that does not begin one, such as a lifetime or label (`'a`, `'static`) or a stray apostrophe, is no quote at all.
   - **In `js` `jsx` `mjs` `cjs` `ts` `tsx`,** a `/` in expression position begins a regular-expression literal. Expression position means the start of the line, or after `(` `,` `=` `:` `[` `!` `&` `|` `?` `{` `}` `;` `+` `-` `*` `%` `<` `>` `~` `^`, or after one of the words `return` `typeof` `instanceof` `in` `of` `new` `delete` `void` `throw` `case` `do` `else` `yield` `await`. The literal runs to its closing `/` on the same line, where that `/` is not escaped and not inside a `[…]` class. A `/` with no closing `/` on its line begins no literal, and the search reads on past it.

   The JavaScript reading is a guess ([§DF-multi-line-string-literals.2.2](../decisions/functional/DF-multi-line-string-literals.md#22-two-bounds-so-a-misread-opener-costs-nothing)). A literal whose span holds nothing the search reads — no unescaped quote the per-line toggle reads and no comment opener — is skipped. One whose span holds either is read both ways, under clause 3, so the `` ` `` in `` s.replace(/`/g, "") `` opens no template.
2. **A form counts only once its closer is found later in the file.** An opener with no closer, or a template literal whose interpolations do not balance before its closer, opens nothing, and the file reads as it would without this rule. A `/*` whose `*/` is not found ends the search under clause 3, and a line-initial one hosts declarations only once its `*/` is found ([§FS-declarations.line.block-comment](FS-declarations.md#lineblock-comment-a-line-inside-a-block-comment)).
3. **A line is read to its end, or the search ends.** Three things leave the search unable to say what the rest of a line opens, and at each it ends: a guessed regular-expression literal whose two readings, skipping it and reading through it, open different things past the line; a line that would take more than 32 such readings; and a `/*` with no `*/` later in the file, after which all of the rest may be comment. Every later line of the file then reads as it would without this rule.

### multi-line-string.unrecognized: The forms that are not carried

These forms are not recognized, and on the lines after their first they keep the per-line reading ([§DF-multi-line-string-literals.2.3](../decisions/functional/DF-multi-line-string-literals.md#23-the-forms-left-out-are-declared-and-fail-toward-reporting)):

- an ordinary quoted string that spans lines in shell, Ruby, PHP, Lisp, SQL or Lua;
- a Ruby heredoc and `%q{…}` and its relatives, where `<<` is also the append operator;
- a Lua long string `[[…]]`;
- a backslash line continuation inside an ordinary string, in any language;
- a second heredoc opened on the same line as the first;
- every form in a file that reads no row of the table ([§FS-languages.multi-line-string.suffix](FS-languages.md#multi-line-stringsuffix-which-row-and-which-quote-reading-a-file-reads)).

Each of these fails in one direction only: an interior line read as code can raise a false `unknown reference` for a qualified citation, and `grund fmt` can offer to rewrite it, but neither can hide a citation, because no unrecognized form starts a span. This list is the declared blind spot [§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded) asks for.
