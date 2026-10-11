# FS-languages: how a source file's lines are read

How a scanned file's lines are read, defined once for every command that reads them: which lines form a comment block, which bytes of a comment line are its wrapper rather than its text, how a Python docstring is read, what opens and closes a Markdown fence, and which comment prefixes a file's extension brings. `show` slices a body by these rules ([§FS-show.2.3](FS-show.md#23-source-declarations-in-code-and-doc-comments)), `check` skips a fence by them ([§FS-citations.recognized.contexts](FS-citations.md#recognizedcontexts-contexts-read-as-neither-prose-nor-code)), and a chapter's label is read through them ([§FS-rules.5.1.1](FS-rules.md#511-a-chapters-display-name-is-the-label-its-author-wrote)); none of them restates what this page says. Serves [§GOAL-polyglot-citation](../goals.md#goal-polyglot-citation-ids-cite-cleanly-from-anywhere-they-are-useful), which is why a declaration and a citation are read alike from a Markdown file, a line comment, a block comment and a docstring.

## terms: Terms

Leans on [§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (doc-comment) and [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope).

- **comment block** — The run of lines one comment construct spans: a run of line comments, one
  block comment, or one Python docstring. Its extent is [§FS-languages.comment-block](FS-languages.md#comment-block-the-comment-block).
- **comment wrapper** — The bytes of a comment line that say it is a comment rather than what it
  says, such as the prefix and the whitespace before it. Which bytes they are, and when a block
  comment's closer is among them, is [§FS-languages.comment-wrapper](FS-languages.md#comment-wrapper-the-comment-wrapper).
- **fence** — A Markdown fenced code block, from its opening delimiter line to its closing one.

## comment-block: The comment block

Extraction is precisely defined so that the implementation has no freedom and the same input produces the same output across editor, CLI, and binding callers.

The block surrounding a declaration line runs from an open boundary ([§FS-languages.comment-block.open](FS-languages.md#comment-blockopen-find-the-open-boundary)) to a close boundary ([§FS-languages.comment-block.close](FS-languages.md#comment-blockclose-find-the-close-boundary)), and ends early at any other declaration line ([§FS-languages.comment-block.early-end](FS-languages.md#comment-blockearly-end-terminate-early-on-another-declaration)). The block forms the scanner normalizes a file into are [§FS-languages.comment-block.forms](FS-languages.md#comment-blockforms-comment-blocks).

Two readings of a run of line comments stand today, and this page states both without choosing: the body `show` slices continues over lines of the same comment prefix family ([§FS-languages.comment-block.open](FS-languages.md#comment-blockopen-find-the-open-boundary)), and the scanner's block normalization over lines of the same line-comment prefix ([§FS-languages.comment-block.forms](FS-languages.md#comment-blockforms-comment-blocks)). Which one every command reads is ruled in [§DF-one-answer-rulings-parse.decision](../decisions/functional/DF-one-answer-rulings-parse.md#decision-decision) and changes behaviour in a later step, not here.

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
- For block-style continuation lines, a leading ` * ` (with surrounding spaces) is removed if present. Lines that do not have it are kept as-is.
- Trailing comment-close tokens (`*/`) on their own line are dropped entirely.
- Blank lines inside the block are preserved.

A Python docstring has no wrapper to remove ([§FS-languages.docstring](FS-languages.md#docstring-docstrings)).

### comment-wrapper.headings: The wrapper of a heading line

A heading in a doc-comment sits behind the comment that carries it — `///`, `//!`, a block comment's ` * `, a hash comment's `#`, or any other configured prefix ([§FS-languages.prefixes](FS-languages.md#prefixes-comment_prefixes-compose-with-extensions)) — and that envelope says where the body lives, not what the chapter is called, so it is never part of the label ([§FS-rules.5.1.1](FS-rules.md#511-a-chapters-display-name-is-the-label-its-author-wrote)).

The envelope has a closing half too. A block comment's closing `*/` on the heading's own line, and the whitespace before it, closes the comment that carries the heading, so it is no more part of the label than the ` * ` that opens the line: ` * ## terms: Terms */` on the last line of a C or Java block comment is `chapter(c, terms, Terms)`, the fact `/// ## terms: Terms` gives. Only the C-family closer is envelope, because only a C-family block comment carries a heading on a line that can close it. A Python docstring's heading is read as Markdown, and a Markdown heading is never trimmed, so an author who writes `*/` at the end of either keeps it in the label.

## docstring: Docstrings

For Python docstrings, no prefix is stripped — docstring content is plain text already; only the surrounding `"""` lines are skipped. Both `"""` and `'''` docstrings are read, and only where `docstring_python` enables Python triple-quoted-string scanning ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)). Where a docstring's block begins and ends is [§FS-languages.comment-block.open](FS-languages.md#comment-blockopen-find-the-open-boundary) and [§FS-languages.comment-block.close](FS-languages.md#comment-blockclose-find-the-close-boundary).

## fence: Fences

A fence opens with at most three leading spaces followed by a run of at least three backticks or tildes; it closes only on a run of the **same character** at least as long as the opener, again with at most three leading spaces and only whitespace after the run. A backtick opener cannot carry a backtick in its info string. An unclosed fence runs to end of file.

## prefixes: `comment_prefixes` compose with `extensions`

The two lists compose: adding `sql` without `--`, or `--` without `sql` (or another extension using that prefix), does not enable SQL doc-comments. Every default comment prefix has a path through the default extension list: `;` pairs with Lisp, Scheme, and Clojure extensions; `--` pairs with SQL, Haskell, Lua, and Ada extensions; and `*` / `/*` are block-comment continuation and opener forms in the C-family extensions. Any line whose first non-whitespace run is a configured prefix is eligible to host a declaration heading or a citation. Each claimed form has a strict-mode executable case that plants a marked dangling citation in that form ([§REQ-no-missed-citation.3](../requirements/REQ-no-missed-citation.md#3-proven-per-host-language)). The two keys themselves are [§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned).
