# FS-languages: how a source file's lines are read

How a scanned file's lines are read, defined once for every command that reads them: which lines form a comment block, which bytes of a comment line are its wrapper rather than its text, how a Python docstring is read, what opens and closes a Markdown fence, which comment prefixes a file's extension brings, and which string literals are read as one literal across lines. `show` slices a body by these rules ([§FS-show.2.3](FS-show.md#23-source-declarations-in-code-and-doc-comments)), `check` skips a fence by them ([§FS-citations.recognized.contexts](FS-citations.md#recognizedcontexts-contexts-read-as-neither-prose-nor-code)), every string-literal reading starts a line in the state they give it ([§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule)), and a chapter's label is read through them ([§FS-rules.5.1.1](FS-rules.md#511-a-chapters-display-name-is-the-label-its-author-wrote)); none of them restates what this page says. Serves [§GOAL-polyglot-citation](../goals.md#goal-polyglot-citation-ids-cite-cleanly-from-anywhere-they-are-useful), which is why a declaration and a citation are read alike from a Markdown file, a line comment, a block comment and a docstring.

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

One requirement on the reading as a whole, carrying a mark of [§FS-config.requirements](FS-config.md#requirements-what-the-config-contract-holds-to). Three more are left out because they are stated elsewhere: *the same lines give the same body in the editor, on the command line and through the bindings* is [§REQ-one-answer.text.parity](../requirements/REQ-one-answer.md#textparity-the-editor-answers-as-the-command-line-does) and [§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes); *every comment form the defaults claim is proved by a dangling citation planted in it* is [§REQ-no-missed-citation.3](../requirements/REQ-no-missed-citation.md#3-proven-per-host-language), which [§FS-languages.prefixes](FS-languages.md#prefixes-comment_prefixes-compose-with-extensions) serves; and *what `show` prints of a source declaration is its author's text less the comment wrapper* is [§FS-show.requirements.authored-bytes](FS-show.md#requirementsauthored-bytes-a-slice-is-the-authors-bytes-selected-and-never-rewritten--realized).

### requirements.one-block: Every command reads a run of line comments as the same block — deferred

The extraction is defined to leave no freedom ([§FS-languages.comment-block](FS-languages.md#comment-block-the-comment-block)), yet a run of line comments is read two ways today: the body `show` slices continues across one comment prefix family, so a `//` note under a `///` declaration is part of its body ([§FS-languages.comment-block.open](FS-languages.md#comment-blockopen-find-the-open-boundary)), while the block forms stop where the prefix changes ([§FS-languages.comment-block.forms](FS-languages.md#comment-blockforms-comment-blocks)), so `refs` gives that note's citations no enclosing declaration ([§REQ-one-answer.differences.sections](../requirements/REQ-one-answer.md#differencessections-sections)). That `//` and `///` are separate blocks is ruled as D17c of [§DF-one-answer-rulings-parse.decision](../decisions/functional/DF-one-answer-rulings-parse.md#decision-decision), and carrying the ruling out is owed to the later step it names, through the warning release of [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path). Serves [§REQ-one-answer.sections.map](../requirements/REQ-one-answer.md#sectionsmap-every-surface-reads-one-section-map).

## comment-block: The comment block

Extraction is precisely defined so that the implementation has no freedom and the same input produces the same output across editor, CLI, and binding callers.

The block surrounding a declaration line runs from an open boundary ([§FS-languages.comment-block.open](FS-languages.md#comment-blockopen-find-the-open-boundary)) to a close boundary ([§FS-languages.comment-block.close](FS-languages.md#comment-blockclose-find-the-close-boundary)), and ends early at any other declaration line ([§FS-languages.comment-block.early-end](FS-languages.md#comment-blockearly-end-terminate-early-on-another-declaration)). The block forms the scanner normalizes a file into are [§FS-languages.comment-block.forms](FS-languages.md#comment-blockforms-comment-blocks).

Two readings of a run of line comments stand today: the body `show` slices continues over lines of the same comment prefix family ([§FS-languages.comment-block.open](FS-languages.md#comment-blockopen-find-the-open-boundary)), and the scanner's block normalization over lines of the same line-comment prefix ([§FS-languages.comment-block.forms](FS-languages.md#comment-blockforms-comment-blocks)). From 0.19.0 every command reads the forms: a block is a run of one exact prefix, so a `//` run next to a `///` block is a block of its own, and the family rule of [§FS-languages.comment-block.open](FS-languages.md#comment-blockopen-find-the-open-boundary) and [§FS-languages.comment-block.close](FS-languages.md#comment-blockclose-find-the-close-boundary) is read as that exact prefix. Until 0.19.0 `show` and `list --size` keep the family reading, by [§DF-one-answer-rulings-parse.decision](../decisions/functional/DF-one-answer-rulings-parse.md#decision-decision) (D17c) and [§DF-one-parse-answers-from-0-19](../decisions/functional/DF-one-parse-answers-from-0-19.md#df-one-parse-answers-from-0-19-one-parses-other-answers-ship-in-0190-after-a-release-that-announces-them-and-warns-where-a-verdict-moves).

### comment-block.open: Find the open boundary

Scan **backwards** from the declaration line over consecutive lines that are part of the same comment construct:

- For line-style comments (`//`, `///`, `//!`, `#`, `;`, `--`): consecutive lines whose first non-whitespace character matches the same comment prefix family. A blank line ends the block. A line whose first non-whitespace character is not a comment prefix ends the block.
- For block-style comments (`/* … */`, `/** … */`): scan backward until the opener is found (`/*` or `/**`). The opener line itself is part of the block.
- For Python triple-quoted docstrings: scan backward until the opening `"""` (or `'''`). The opener line is part of the block.

### comment-block.close: Find the close boundary

Scan **forwards** from the declaration line by the symmetric rules:

- Line-style: until a blank line or a non-comment line.
- Block-style: until the closing `*/`. The closer line is part of the block. The closer is the one that ends the declaration's own comment, wherever it falls: a declaration written as a one-line `/* … */` closes on its own line, so its block is that line alone, and a `/* … */` comment on the next line is a separate comment outside it — not a section of the declaration, and not part of its body.
- Python docstring: until the matching `"""` or `'''`. The closer line is part of the block.

### comment-block.early-end: Terminate early on another declaration

Scanning in **either direction**, if another declaration line of any ID is encountered, the block ends at the line before it. This is what allows two adjacent source declarations to live in the same comment without bleeding into each other — backward termination keeps a later declaration's block from absorbing the previous declaration's tail; forward termination keeps the previous declaration's block from absorbing the next declaration's head.

### comment-block.forms: Comment blocks

The block forms say where a block begins and ends:

- `//` / `///` / `//!` line comments: a run of adjacent lines whose first non-whitespace token is the same line-comment prefix.
- `#`, `;`, `--` line comments: same rule per prefix (see [§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned) for the full prefix set).
- `/* … */` block comments, `/**`- and `/*!`-opened alike: from opener to closer.
- Python triple-quoted docstrings (`""" … """` / `''' … '''`): from the opening triple-quote to the matching close.

Adjacency is broken by any line that is not part of the same block: a code line, a blank line, or a different comment style. An empty comment line — the prefix alone, such as `//` or `#`, with nothing after it — stays inside the run: it carries the prefix, so only a line without one breaks it.

## comment-wrapper: The comment wrapper

After the block is selected, comment prefixes are removed line-by-line so the output is plain prose:

- The whitespace in front of the comment prefix belongs to the comment's envelope, not to the body: it is dropped together with the prefix, then a single space following the prefix is dropped if present. The remainder of the line is kept verbatim, so indentation the author wrote after the prefix (a nested list item, an indented code block) survives. This holds for an indented `///`, `//!`, `//`, `#`, `;` or `--` line, such as a doc-comment on a method inside an `impl`, exactly as for a ` * ` continuation of a class member's Javadoc: the shown body starts at the same column as the same text behind a top-level `///`.
- The prefix removed is the whole form the line was written in ([§FS-languages.prefixes.forms](FS-languages.md#prefixesforms-the-forms-a-prefix-stands-for)): behind `;` that is the line's whole run of semicolons, so `;; One route per path.` and `;;;; One route per path.` both show as `One route per path.`, never as `; One route per path.`. A block of such lines is still one block keyed on `;`, whatever run each line opens with, so where a `;` block starts and ends does not move ([§FS-languages.comment-block](FS-languages.md#comment-block-the-comment-block)).
- For block-style continuation lines, a leading ` * ` (with surrounding spaces) is removed if present. Lines that do not have it are kept as-is. Under a declaration only [§FS-declarations.line.block-comment](FS-declarations.md#lineblock-comment-a-line-inside-a-block-comment) finds, such a line is read this way at once, a `## 1. Sub` among them a heading of the block ([§DF-bare-block-comment-line-declares](../decisions/functional/DF-bare-block-comment-line-declares.md#df-bare-block-comment-line-declares-a-line-inside-a-block-comment-declares-with-or-without-its-gutter-behind-any-of-its-openers)). Otherwise, from 0.19.0 `show` prints `## 1. Sub` as written and the line is a heading of the block ([§FS-languages.comment-wrapper.headings](FS-languages.md#comment-wrapperheadings-the-wrapper-of-a-heading-line)). Until 0.19.0 `show` prints it as `# 1. Sub` and the scanner records no section there ([§DF-one-parse-answers-from-0-19](../decisions/functional/DF-one-parse-answers-from-0-19.md#df-one-parse-answers-from-0-19-one-parses-other-answers-ship-in-0190-after-a-release-that-announces-them-and-warns-where-a-verdict-moves), [§FS-declarations.checks.parse-moves](FS-declarations.md#checksparse-moves-a-verdict-that-moves-in-0190)).
- Trailing comment-close tokens (`*/`) on their own line are dropped entirely.
- Blank lines inside the block are preserved.

A Python docstring has no wrapper to remove ([§FS-languages.docstring](FS-languages.md#docstring-docstrings)).

### comment-wrapper.headings: The wrapper of a heading line

A heading in a doc-comment sits behind the comment that carries it — `///`, `//!`, a block comment's ` * `, a hash comment's `#`, or any other configured prefix ([§FS-languages.prefixes](FS-languages.md#prefixes-comment_prefixes-compose-with-extensions)) — and that envelope says where the body lives, not what the chapter is called, so it is never part of the label ([§FS-rules.5.1.1](FS-rules.md#511-a-chapters-display-name-is-the-label-its-author-wrote)).

The envelope has a closing half too. A block comment's closing `*/` on the heading's own line, and the whitespace before it, closes the comment that carries the heading, so it is no more part of the label than the ` * ` that opens the line: ` * ## terms: Terms */` on the last line of a C or Java block comment is `chapter(c, terms, Terms)`, the fact `/// ## terms: Terms` gives. Only the C-family closer is envelope, because only a C-family block comment carries a heading on a line that can close it. A Python docstring's heading is read as Markdown, and a Markdown heading is never trimmed, so an author who writes `*/` at the end of either keeps it in the label. `show` keeps it in either today, so nothing here moves in 0.19.0.

## docstring: Docstrings

For Python docstrings, no prefix is stripped — docstring content is plain text already; only the surrounding `"""` lines are skipped. Both `"""` and `'''` docstrings are read, and only where `docstring_python` enables Python triple-quoted-string scanning ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)). Where a docstring's block begins and ends is [§FS-languages.comment-block.open](FS-languages.md#comment-blockopen-find-the-open-boundary) and [§FS-languages.comment-block.close](FS-languages.md#comment-blockclose-find-the-close-boundary).

## fence: Fences

A fence opens with at most three leading spaces followed by a run of at least three backticks or tildes; it closes only on a run of the **same character** at least as long as the opener, again with at most three leading spaces and only whitespace after the run. A backtick opener cannot carry a backtick in its info string. An unclosed fence runs to end of file.

### fence.comment-block: A fence inside a comment block hides its headings

Inside a comment block ([§FS-languages.comment-block](FS-languages.md#comment-block-the-comment-block)) a fence is read on each line's text with its comment wrapper off ([§FS-languages.comment-wrapper](FS-languages.md#comment-wrapper-the-comment-wrapper)), by the rule above: `/// ```text`, ` * ```text` and a docstring's own ```` ```text ```` line each open one, and the matching delimiter behind the same wrapper closes it. A fence never outlives the comment block it opens in: one still open where the block ends closes there, so it hides nothing in a later comment block of the file.

While such a fence is open, a line that would be a section or a plain heading of the block, numbered, named or plain at any level, is not a heading ([§FS-declarations.heading](FS-declarations.md#heading-what-a-heading-is)), exactly as inside a Markdown fence ([§FS-show.2.5](FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example)). It is not a section of the declaration whose body holds it, so a citation of the coordinate it spells is a missing section ([§FS-check.3.2](FS-check.md#32-missing-section)). It does not end a lead, a section or a body, `--toc` does not list it, `refs` does not name it as a citation's enclosing section, and `fmt` does not link to it. It draws no heading-level, duplicate-section or section-outside-declaration finding ([§FS-declarations.checks.section-heading-level](FS-declarations.md#checkssection-heading-level-section-heading-level-mismatch), [§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path), [§FS-declarations.checks.section-outside-declaration](FS-declarations.md#checkssection-outside-declaration-section-outside-a-declaration)).

The fence hides headings and nothing else. A citation inside it is still read and checked ([§FS-citations.recognized.contexts](FS-citations.md#recognizedcontexts-contexts-read-as-neither-prose-nor-code) skips only a Markdown file's fence), a `grund:fmt off` or `grund:fmt on` line inside it still toggles ([§FS-fmt.2.5.2](FS-fmt.md#252-grundfmt-off--grundfmt-on--a-region-at-a-time)), `show` still flattens a citation wrapper inside it ([§FS-show.2.5](FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example)), and a source declaration line inside it is read as it would be outside ([§FS-declarations.stubs.target.2](FS-declarations.md#stubstarget2-a-heading-inside-a-fence-of-the-target-declares-nothing)). Why a heading there was once a section, and why that changed in one release, is [§DF-doc-comment-fenced-heading](../decisions/functional/DF-doc-comment-fenced-heading.md#df-doc-comment-fenced-heading-a-heading-inside-a-fence-in-a-comment-block-is-not-a-section).

## prefixes: `comment_prefixes` compose with `extensions`

The two lists compose: adding `sql` without `--`, or `--` without `sql` (or another extension using that prefix), does not enable SQL doc-comments. Every default comment prefix has a path through the default extension list: `//` pairs with the C-family extensions — C and C++ sources and headers, Objective-C, the JavaScript and TypeScript spellings, Kotlin and Kotlin script, Dart, and the rest of [§FS-inline-citation-style.1.1.2](FS-inline-citation-style.md#112-languages)'s C-family row; `#` pairs with Python, Ruby, R, and shell extensions; `;` pairs with Lisp, Scheme, and Clojure extensions; `--` pairs with SQL, Haskell, Lua, and Ada extensions; and `*` / `/*` are block-comment continuation and opener forms in the C-family extensions, where `/*` also stands for the `/**` and `/*!` openers. Any line whose first non-whitespace run is a configured prefix is eligible to host a declaration heading or a citation, and so is any line inside a block a `/*` opened, with or without its ` * ` ([§FS-declarations.line.block-comment](FS-declarations.md#lineblock-comment-a-line-inside-a-block-comment)). A configured prefix stands for the doc-comment forms of [§FS-languages.prefixes.forms](FS-languages.md#prefixesforms-the-forms-a-prefix-stands-for) as well as for itself, so a line led by one of those forms is eligible the same way. Each claimed form has a strict-mode executable case that plants a marked dangling citation in that form ([§REQ-no-missed-citation.3](../requirements/REQ-no-missed-citation.md#3-proven-per-host-language)). The two keys themselves are [§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned).

This composition is v1's. A v2 file has no global prefix list: each language brings its own hosts, a built-in one from the epoch and a defined one from its `comment_prefixes`, and a file takes the hosts of the language its name resolves to ([§FS-config-v2.schema.sources](FS-config-v2.md#schemasources-schemasources)).

### prefixes.forms: The forms a prefix stands for

Four default prefixes stand for more than their own bytes, because the languages they serve spell a doc comment by doubling or marking the prefix:

| configured | stands for |
|---|---|
| `//` | `///`, `//!`, `//` |
| `/*` | `/**`, `/*!`, `/*` |
| `--` | `---`, `-- \|`, `--\|`, `-- ^`, `--^`, `--` |
| `;` | any run of semicolons: `;`, `;;`, `;;;`, `;;;;`, … |

A form is followed by whitespace or by the end of the line, as the prefix itself is. So `--- FS-b: Beta` (Lua's LDoc), `-- | FS-b: Beta` and `-- ^ FS-b: Beta` (Haskell's Haddock) and `;; FS-b: Beta` or `;;; FS-b: Beta` (Lisp, Scheme, Clojure and Emacs Lisp) are comment-prefixed lines exactly as `-- FS-b: Beta` and `; FS-b: Beta` are, in declaration position by [§FS-declarations.line](FS-declarations.md#line-what-a-line-in-declaration-position-declares), and a `## 1. Sub` behind any of them is a section heading wherever the same line behind the bare prefix is one. `----` is a rule line, as `////` is: after `---` comes a fourth `-`, not whitespace. Anything else glued to the prefix, such as `;x` or `-- *`, is no form of it.

The forms hang off the configured prefix, not the file's language, the way `///` declares in any file where `//` is configured: `--- FS-b:` declares in a SQL file too. A project that removed `--` or `;` from `comment_prefixes` gets none of its forms, and a prefix outside this table, such as a custom `%`, stands only for itself. The wrapper a form leaves behind is [§FS-languages.comment-wrapper](FS-languages.md#comment-wrapper-the-comment-wrapper); which blocks of these forms are doc comments rather than notes is [§FS-inline-citation-style.1.1.2](FS-inline-citation-style.md#112-languages), not this table's question.

### prefixes.haskell-block: A Haskell block comment hosts no declaration

Haskell's block comment `{- … -}`, the Haddock `{-| … -}` included, is no prefix and no form of one, so no line inside it is in declaration position: `{-| FS-b: Beta` declares nothing, and a citation of `FS-b` that it was meant to answer is an `unknown reference`. The line form `-- |` documents everything the block form does and declares by [§FS-languages.prefixes.forms](FS-languages.md#prefixesforms-the-forms-a-prefix-stands-for). A marked citation inside the block is still read and checked, so a dangling or missing-section citation written inside `{-| … -}` is reported at its line as anywhere else; nothing written there is missed, only declined as a host. Decided in [§DF-line-prefix-forms-declare](../decisions/functional/DF-line-prefix-forms-declare.md#df-line-prefix-forms-declare-lua-haskell-and-lisp-doc-comment-forms-declare-and-a-lisp-doc-block-is-not-a-note).

## multi-line-string: A string literal that spans lines

Which string literals are read as one literal across line breaks, defined once for every reading that asks whether a column of a source line is inside a string: the per-line scan of [§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule), and through it the source-string skips of [§FS-citations.recognized.string-literals](FS-citations.md#recognizedstring-literals-string-literals-in-source-files) and [§FS-citations.recognized.contexts](FS-citations.md#recognizedcontexts-contexts-read-as-neither-prose-nor-code), every `grund fmt` pass, the LSP on-type transform ([§FS-lsp.1.4.3](FS-lsp.md#143-where-the-expansion-refuses)) and completion ([§FS-lsp.1.6.1](FS-lsp.md#161-eligibility-exclusions-and-triggering)). None of them restates the forms below. Serves [§REQ-no-wrong-citation.2](../requirements/REQ-no-wrong-citation.md#2-no-false-alarms), because text a program prints is data rather than a citation on its interior lines as much as on its first, and [§REQ-no-data-loss.2](../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own), because `grund fmt` does not own those bytes. Decided in [§DF-multi-line-string-literals](../decisions/functional/DF-multi-line-string-literals.md#df-multi-line-string-literals-a-string-literal-that-spans-lines-is-one-literal-for-a-fixed-set-of-forms).

What carries across a line break is never the per-line quote toggle of [§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule): that toggle still flips on `'`, `"` and `` ` `` and still starts every line neutral, because carrying it would let one apostrophe in `// don't`, a character literal or a Rust lifetime `'a` hide every later citation in the file ([§REQ-no-missed-citation.1](../requirements/REQ-no-missed-citation.md#1-no-silent-skips)). What carries is a **recognized multi-line form**: one of the fixed openers of [§FS-languages.multi-line-string.forms](FS-languages.md#multi-line-stringforms-the-recognized-forms), in a file whose extension the form names, with that form's exact closer.

A form is read in three parts:

- **The opening line** is judged exactly as it is without this rule. The opener only marks where the form begins.
- **A line that begins inside a form** is inside a string literal from its first column up to the end of the form's closer. When the closer is on that line, the rest of the line after it is read by the per-line toggle, starting neutral. When the closer is not on that line, the whole line is inside the literal.
- **Every other line** is judged exactly as it is without this rule.

Forms are found in file order: once a form opens, nothing inside it opens another, and the search for the next opener resumes after its closer — so a form that opens and closes on one line carries nothing, and a second opener on the line after it is still found. Being inside a form changes only the answer to "is this column inside a string literal". Which lines are in declaration position ([§FS-declarations.line](FS-declarations.md#line-what-a-line-in-declaration-position-declares)), where a `/* … */` block opens and closes, and which lines form a comment block ([§FS-languages.comment-block](FS-languages.md#comment-block-the-comment-block)) are read as they are without it.

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

In a `.py` file a docstring ([§FS-languages.docstring](FS-languages.md#docstring-docstrings)) and an assigned-data span ([§FS-citations.recognized.string-literals.assigned-data](FS-citations.md#recognizedstring-literalsassigned-data-assigned-python-triple-quoted-data)) keep their own rules and win wherever they apply; the Python row adds only the triple-quoted string that neither of them claims, such as `call("""`, and applies whatever `[scan] docstring_python` says.

### multi-line-string.bound: What keeps a misread opener from swallowing a file

Three rules bound every form, so that a wrongly detected opener costs at most today's reading and never hides a citation:

1. **An opener counts only in code.** It does not count where the per-line toggle of [§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule) already has a quote open, after a comment prefix that opens a line comment there ([§FS-languages.prefixes](FS-languages.md#prefixes-comment_prefixes-compose-with-extensions)), inside a `/* … */` block from its `/*` to its `*/` (in `rs` `kt` `kts` `scala` `swift` `dart`, where blocks nest, the `*/` that balances it), inside a Python docstring, or inside an assigned-data span. `# cat <<EOF` in a shell comment opens nothing, and neither does a `<<` inside a shell arithmetic `(( … ))`. The search finds a `/*` where it would find an opener, in code, whether it starts its line or follows code, and resumes after the `*/`, on the same line or a later one. It reads blocks only to find openers: which lines host declarations stays as [§FS-declarations.line.block-comment](FS-declarations.md#lineblock-comment-a-line-inside-a-block-comment) says. In two languages the search reads a quote-like byte the way the language does, so that it never opens a form on a byte that is not a quote:
   - **In `rs`,** a character or byte literal, such as `'x'`, `'\n'`, `'\''`, `'"'` or `b'x'`, is skipped as one unit. A `'` that does not begin one, such as a lifetime or label (`'a`, `'static`) or a stray apostrophe, is no quote at all. Clause 2 of [§FS-languages.multi-line-string.forms](FS-languages.md#multi-line-stringforms-the-recognized-forms) depends on this reading.
   - **In `js` `jsx` `mjs` `cjs` `ts` `tsx`,** a `/` in expression position begins a regular-expression literal. Expression position means the start of the line, or after `(` `,` `=` `:` `[` `!` `&` `|` `?` `{` `}` `;` `+` `-` `*` `%` `<` `>` `~` `^`, or after one of the words `return` `typeof` `instanceof` `in` `of` `new` `delete` `void` `throw` `case` `do` `else` `yield` `await`. The literal runs to its closing `/` on the same line, where that `/` is not escaped and not inside a `[…]` class. A `/` with no closing `/` on its line begins no literal, and the search reads on past it.

   The Rust reading is the language's own: a `'` followed by one character or one escape and a `'` is a character literal and nothing else, so it is skipped whatever it holds. The JavaScript reading is a guess: a `/` may be a division or the `</` of a JSX closing tag. A literal whose span holds nothing the search reads — no unescaped quote the per-line toggle reads and no comment opener — is skipped, because reading through it comes to the same thing. One whose span holds either is read both ways, under clause 3. So the `` ` `` in `` s.replace(/`/g, "") `` opens no template. Neither does the `` ` `` in `` <b>Tip:</b> see <a href="/docs">…</a> or press {"`"} ``, where the `/` of `</b>` reads as a regular expression up to the `/` of `"/docs"`.
2. **A form counts only once its closer is found later in the file.** An opener with no closer, or a template literal whose interpolations do not balance before its closer, opens nothing, and the file reads as it would without this rule. A `/*` whose `*/` is not found ends the search under clause 3, and a line-initial one hosts declarations only once its `*/` is found ([§FS-declarations.line.block-comment](FS-declarations.md#lineblock-comment-a-line-inside-a-block-comment)).
3. **A line is read to its end, or the search ends.** The search never stops partway along a line and goes on to the next as if the rest held nothing. The rest might hold a real opener, and its closer, where that closer also reads as an opener — a `` ` ``, a `"`, a `"""` — would then open a form on the later line where it really closes and hide the code up to the next one. Three things leave the search unable to say what the rest of a line opens, and at each it ends: a guessed regular-expression literal whose two readings, skipping it and reading through it, open different things past the line; a line that would take more than 32 such readings; and a `/*` with no `*/` later in the file, after which all of the rest may be comment. Every later line of the file then reads as it would without this rule. A line comment runs to the end of its line, and after a heredoc's opener only a second heredoc can open, which is not carried ([§FS-languages.multi-line-string.unrecognized](FS-languages.md#multi-line-stringunrecognized-the-forms-that-are-not-carried)) and whose closer opens nothing.

### multi-line-string.unrecognized: The forms that are not carried

These forms are not recognized, and on the lines after their first they keep the per-line reading:

- an ordinary quoted string that spans lines in shell, Ruby, PHP, Lisp, SQL or Lua, where `'` is a quote and an apostrophe alike;
- a Ruby heredoc and `%q{…}` and its relatives, where `<<` is also the append operator;
- a Lua long string `[[…]]`;
- a backslash line continuation inside an ordinary string, in any language;
- a second heredoc opened on the same line as the first;
- every form in a file whose extension the table of [§FS-languages.multi-line-string.forms](FS-languages.md#multi-line-stringforms-the-recognized-forms) does not name.

Each of these fails in one direction only. An interior line read as code can raise a false `unknown reference` for a qualified citation, and `grund fmt` can offer to rewrite it; neither can hide a citation, because no unrecognized form starts a span. This list is the declared blind spot [§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded) asks for.
