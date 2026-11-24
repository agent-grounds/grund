# FS-inline-citation-style: configurable shape of inline code-comment citations

An inline citation in a code comment may carry a short note beside its `§<ID>` token, saying why the clause it sits beside is grounded in that declaration. A project sets a house style for that note: whether it may exist, how long and how wide it may run, and where its citations sit inside it. The same keys decide what `grund check` reports and what the managed agent block tells the agent who writes the note, so the author and the linter agree on the rules. Serves [§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable) and [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, body, section), [§FS-terms.terms.2](FS-terms.md#terms2-citations)
(marker, citation, shorthand, citation site), [§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (source declaration, stub,
doc-comment, note), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, workspace, alias), [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding,
severity), and [§FS-terms.terms.6](FS-terms.md#terms6-rules-and-directions) (direction).

- **comment block** — The contiguous run of comment lines a site is measured over, as
  [§FS-languages.comment-block](FS-languages.md#comment-block-the-comment-block) delimits it.
- **definition-starter** — The language keyword on the line below a block that makes the block a
  doc-comment rather than an inline comment.
- **leading comment** — A block with only blank lines or a shebang above it — how a language
  with no doc-comment syntax spells a module doc.

## requirements: What the note budgets hold to

Two properties of the budgets and the layout are left out here because a requirement states them: two installs measure a note alike, with no per-file, per-kind or per-editor reshaping of the measure ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree), [§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)), and a default changes nothing a conformant tree sees ([§GOAL-zero-config](../goals.md#goal-zero-config-works-on-any-conformant-tree), [§REQ-backwards-compatibility.1](../requirements/REQ-backwards-compatibility.md#1-what-is-covered)).

### requirements.actionable: A budget finding is acted on as printed

A budget finding lands once per note, at its first line, and once per cap the note breaks, so one pass shows every reason. It prints the measured size beside the cap, the note's lines and every citation in it, and, for a line count, the boundary a split has to cross ([§FS-inline-citation-style.4.1](FS-inline-citation-style.md#41-errors--hard-caps)); the soft-cap warning carries the same clauses ([§FS-inline-citation-style.4.2](FS-inline-citation-style.md#42-warnings--opt-in-soft-cap)). A layout deviation lands at the line to edit and names the shape to write ([§FS-inline-citation-style.4.4](FS-inline-citation-style.md#44-warnings-and-errors--opt-in-layout-deviations)). Nothing has to be measured again or looked up to act on one, and a budget added from here on is reported the same way. Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

### requirements.shape-only: A note's size and layout never reach its citations

Whether a block is a note, how large it is and how it is laid out decide only what `check` measures and reports. Its citations resolve, count toward the grounding floor and the citation directions, and are linked by `fmt` as anywhere else ([§FS-inline-citation-style.1.1](FS-inline-citation-style.md#11-doc-comments-are-not-sites)), so a block recognized the wrong way changes what is measured and never what a citation means ([§FS-inline-citation-style.1.1.3](FS-inline-citation-style.md#113-accepted-corners)). No command rewrites a note to fit ([§FS-inline-citation-style.6.1](FS-inline-citation-style.md#61-no-rewrite-in-grund-fmt)), and every command but `check` — `fmt`, `refs`, `cover`, `show` — reads its citations whatever its measurements. Serves [§REQ-no-wrong-citation](../requirements/REQ-no-wrong-citation.md#req-no-wrong-citation-a-citation-never-resolves-to-a-guess) and [§REQ-no-data-loss](../requirements/REQ-no-data-loss.md#req-no-data-loss-grund-never-eats-user-content).

### requirements.agent-copy: The agent is told what the gate holds, as of the last render

The copy in the managed block is rendered from the keys `check` reads ([§FS-inline-citation-style.5](FS-inline-citation-style.md#5-agent-facing-rendering)): the caps, the block a cap finding says to split ([§FS-inline-citation-style.5.2](FS-inline-citation-style.md#52-the-block-sentence)), the layout where one is set ([§FS-inline-citation-style.5.3](FS-inline-citation-style.md#53-the-layout-sentence)), and where the gate stops ([§FS-inline-citation-style.5.4](FS-inline-citation-style.md#54-the-doc-comment-sentence)). It says what to write, never which channel a deviation reaches, so moving a layout check between `warn` and `error`, or setting `warn_on_suggested`, changes nothing the agent reads. It is a snapshot: once a key that decides what is judged is changed, it teaches the previous house style until `grund init` renders it again, which `grund init --check` reports as `would-update` and `check` does not ([§FS-inline-citation-style.5.5](FS-inline-citation-style.md#55-rendered-not-live), [§FS-init.1](FS-init.md#1-inputs)). A check added to the note gate from here on adds its sentence in the same change, and a key that only picks a channel adds none. Serves [§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work).

## 1. Scope

A **note** is an inline comment block — a maximal run of adjacent comment or docstring lines, as [§FS-languages.comment-block](FS-languages.md#comment-block-the-comment-block) bounds a comment block — that holds at least one citation token recognized by [§FS-citations.recognized](FS-citations.md#recognized-recognized-citations). A doc-comment block is never a note, whatever it carries.

### 1.1 Doc-comments are not sites

Only an inline comment is a note, and an inline comment is every comment that is not a doc-comment ([§FS-terms.terms.3](FS-terms.md#terms3-source-forms)): one among statements, a detached block, a note beside the clause it grounds ([§DF-doc-comments-are-not-notes.2.1](../decisions/functional/DF-doc-comments-are-not-notes.md#21-the-site-is-an-inline-comment-a-doc-comment-is-not-a-site)). No rule of this spec reaches a citation inside a doc-comment: not `citation-only` ([§FS-inline-citation-style.3.1](FS-inline-citation-style.md#31-citation-only)), not the line and column budgets ([§FS-inline-citation-style.2.3](FS-inline-citation-style.md#23-counting-lines-and-columns), [§FS-inline-citation-style.4.1](FS-inline-citation-style.md#41-errors--hard-caps), [§FS-inline-citation-style.4.2](FS-inline-citation-style.md#42-warnings--opt-in-soft-cap)), not the layout ([§FS-inline-citation-style.3.3](FS-inline-citation-style.md#33-inline_note_layout--where-the-citations-sit), [§FS-inline-citation-style.4.4](FS-inline-citation-style.md#44-warnings-and-errors--opt-in-layout-deviations)). Everything else still does: the citation resolves and is dangling when it does not ([§FS-check.3.1](FS-check.md#31-dangling-citation)), counts toward the grounding floor ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)) and the citation directions ([§FS-check.3.11](FS-check.md#311-missing-required-citation), [§FS-check.3.12](FS-check.md#312-forbidden-citation)), is judged by the target project's shorthand policy when it is a persisted shorthand ([§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)), and is linkified by `grund fmt` ([§FS-fmt.6](FS-fmt.md#6-cross-reference-emission)).

#### 1.1.2 Languages

Which kind a block is, is read from the file's extension and one test on the block itself, never by parsing the host language ([§FS-non-goals.3](FS-non-goals.md#3-code-ast-parsing), [§DF-doc-comments-are-not-notes.2.2](../decisions/functional/DF-doc-comments-are-not-notes.md#22-two-recognizers-chosen-by-file-type-never-by-parsing)). Each extension belongs to one recognizer:

- A **prefix** language spells a doc-comment with a comment prefix of its own: the block is a doc-comment when its line-comment prefix, or for a block comment its opening line, is the doc form the table gives.
- A **position** language spells a doc-comment like every other comment: the block is a doc-comment when the line immediately below it, with no blank line between, is a definition-starter for that language, or when it is the file's leading comment: every line above it is blank, or is line 1 holding a `#!` shebang ([§DF-doc-comments-are-not-notes.2.3](../decisions/functional/DF-doc-comments-are-not-notes.md#23-a-leading-comment-block-is-a-module-doc)).
- **Any other extension** has no doc-comment notion, so every comment block in it is inline.

| extensions | recognizer | doc-comment when |
|---|---|---|
| `rs` `c` `h` `cpp` `cc` `cxx` `hpp` `hh` `hxx` `m` `mm` `java` `cs` `kt` `kts` `scala` `swift` `js` `jsx` `mjs` `cjs` `ts` `tsx` `php` `dart` | prefix (C family) | a line run whose prefix is exactly `///` or `//!` (`////` is not); a block comment opening `/**` or `/*!`, but not the empty `/**/`. A plain `//`, `/* … */` or PHP `#` block is inline wherever it sits, directly above a `fn` included. |
| `py` | prefix (docstring) | a `"""` or `'''` docstring block. A `#` block is inline, directly above a `def` included. |
| `lua` | prefix (LDoc) | a `--` run whose first line starts with exactly `---` (`----` does not); its continuation lines are plain `--` lines of the same block. |
| `hs` `lhs` | prefix (Haddock) | a `--` run whose first line's content after the `--` starts, after optional spaces, with `\|` or `^`. |
| `r` `R` | prefix (roxygen2) | a `#` run whose first line starts with `#'`. |
| `go` | position | definition-starters `func`, `type`, `var`, `const`, `package`. |
| `rb` | position | definition-starters `class`, `module`, `def`. |
| `sh` `bash` `zsh` | position | `function <name>`, or `<name>()` / `<name> ()`, with `<name>` matching `[A-Za-z_][A-Za-z0-9_]*`. |
| `sql` | position | the definition-starter `create`, matched case-insensitively, so `CREATE OR REPLACE FUNCTION …` counts. |
| `lisp` `cl` `el` `scm` `ss` `rkt` `clj` `cljs` `cljc` | position | the next line, with leading whitespace removed, opens `(` and a head symbol that starts with `def` or `cl-def`, or is `ns`, so `(defun` and `(ns app.core)` match and `(let` does not. The number of semicolons decides nothing, so a `;;` block inside a function body stays a note. |

A definition-starter matches when the next line, with leading whitespace removed, begins with the keyword followed by a non-identifier character or the end of the line, so `functional` is no `func`. A Lisp head symbol is the run after the `(` up to whitespace, a parenthesis or the end of the line; `ns` must be the whole symbol, while `def` and `cl-def` need only begin it ([§FS-inline-citation-style.1.1.3](FS-inline-citation-style.md#113-accepted-corners)).

Every extension the table names is read by default ([§FS-scan.scope.extensions.default](FS-scan.md#scopeextensionsdefault-the-extensions-default-follows-the-language-table)).

#### 1.1.3 Accepted corners

Four corners are accepted rather than repaired ([§DF-doc-comments-are-not-notes.2.6](../decisions/functional/DF-doc-comments-are-not-notes.md#26-the-corners-it-accepts)):

- A **dangling doc-comment**, a `/** … */` or `///` inside a method body, is a doc-comment by its prefix and is not measured.
- **Position recognition is recognition, not parsing.** A Go `var` inside a function body reads as a doc-comment and a Ruby `private def` does not, and a `;;` block above a Lisp call whose head merely starts with `def`, such as `(default-value 'fill-column)`, reads as a doc-comment. A miss in either direction changes only whether a block is measured, never what a citation resolves to or whether it resolves at all.
- A comment **trailing code** on the same line (`foo(); // §<ID>: note`) is not a note.
- **Blank-line adjacency is adjacency.** A `#` block, a blank line, then a `def` is an inline comment ([§FS-inline-citation-style.1.1.2](FS-inline-citation-style.md#112-languages)).

#### 1.1.4 Declaration applicability

Under v2's `declarations = "doc-comments"`, the recognizers of [§FS-inline-citation-style.1.1.2](FS-inline-citation-style.md#112-languages) also decide which comments may declare an ID ([§FS-config-v2.schema.sources.declarations](FS-config-v2.md#schemasourcesdeclarations-declarations-picks-which-comments-declare)), through the doc profile a v2 language names ([§FS-config-v2.schema.sources.epoch](FS-config-v2.md#schemasourcesepoch-the-v2-language-epoch)), so measurement and declaration cannot disagree about what documentation is. A miss costs more in a declaration than in a measurement, so for declarations each profile is frozen as it stood at the v2 epoch, and the corners of [§FS-inline-citation-style.1.1.3](FS-inline-citation-style.md#113-accepted-corners) are its stated rules rather than accepted misses:

- `c-family`: a `///` or `//!` run and a `/**` or `/*!` block qualify, dangling inside a body or not; a plain `//` or `/* … */` block, a PHP `#` block and a comment trailing code do not; nor do the lookalikes `////` and `/**/`.
- `python`: a docstring qualifies, read lexically, with the assigned-data exclusion of [§FS-citations.recognized.string-literals.assigned-data](FS-citations.md#recognizedstring-literalsassigned-data-assigned-python-triple-quoted-data); a `#` block does not.
- `ldoc`: a `--` run whose first line opens with exactly `---`; `----` does not qualify. `haddock`: a `--` run whose first content is `|` or `^` after optional spaces. `roxygen`: a `#` run opening `#'`.
- `go`, `ruby`, `shell`, `sql`: a block whose next line, with no blank line between, is a definition-starter of [§FS-inline-citation-style.1.1.2](FS-inline-citation-style.md#112-languages) — Go's `func`, `type`, `var`, `const` and `package`, a local `var` included; Ruby's `class`, `module` and `def`, and not `private def` — or the file's leading block, after blank lines or a line-1 shebang. A detached block, and a block separated from its definition by a blank line, do not qualify.

The starter sets may widen for measurement without a profile changing ([§FS-inline-citation-style.6.4](FS-inline-citation-style.md#64-no-per-project-reshaping-of-the-rule)). Decided in [§DF-declarations-in-doc-comments](../decisions/functional/DF-declarations-in-doc-comments.md#df-declarations-in-doc-comments-a-v2-source-declares-in-documentation-not-in-a-note).

### 1.2 Comment blocks

The block forms say where a block begins and ends, and are [§FS-languages.comment-block](FS-languages.md#comment-block-the-comment-block). They are the scanner's existing normalization, not a verdict on which blocks are sites.

### 1.3 What is not a site

This spec governs notes only. It does **not** govern:

- Citations inside Markdown declaration bodies (prose in `docs/`, `tests/e2e/`, or any other file read as Markdown). Spec text governs itself.
- Declarations themselves. `# FS-foo: …` and `/// FS-foo: …` are declaration headings, and a declaration's own heading is not one of the citations a scan records; a doc-comment whose first line is a declaration heading and whose remaining lines are its body is a declaration, not a note.
- Inline-spec stubs (`# <ID>: [<text>](<path>)`), a `docs/` shape, not a code-comment shape.
- Bare ID-shaped tokens the scanner does not record as citations: one inside a string literal in a source file ([§FS-citations.recognized.string-literals](FS-citations.md#recognizedstring-literals-string-literals-in-source-files)), and any bare token at all under `[reference] strict = true` ([§FS-config.3.1](FS-config.md#31-reference--citation-form)).
- Doc-comments ([§FS-inline-citation-style.1.1](FS-inline-citation-style.md#11-doc-comments-are-not-sites)).

### 1.4 Notes

A note's *note text* is any non-whitespace character left on any of its lines once three things are stripped from that line: its comment-prefix tokens (`//`, `*`, the opening `/**`, the docstring `"""`, and so on, the stripping [§FS-languages.comment-wrapper](FS-languages.md#comment-wrapper-the-comment-wrapper) does for a declaration's body), every citation token [§FS-citations.recognized](FS-citations.md#recognized-recognized-citations) recognizes, workspace-qualified `§<alias>/<ID>` tokens included ([§FS-workspace.1](FS-workspace.md#1-citation-syntax)), and each separator between two consecutive citation tokens on that line. A separator is whitespace, or a single comma with optional whitespace around it, so `// §FS-check.3.1  §FS-config.3.1` and `// §FS-check.3.1, §FS-config.3.1` are both pure citation comments: the second spells the chain the way [§FS-inline-citation-style.3.3](FS-inline-citation-style.md#33-inline_note_layout--where-the-citations-sit) requires a citation run to be spelled. Anything else between two tokens is note text: a second comma, a ` + `, a ` / `, an `and`. A note *carries note text* when any is left. The stripping is per line, so a chain wrapped across two comment lines, `// §FS-check.3.1,` above `// §FS-config.3.1`, leaves a comma with no following token to join, and that note carries note text.

## 2. Configuration

The keys live in `[reference]`, and their values, defaults and load-time rules are [§FS-config.3.1](FS-config.md#31-reference--citation-form)'s: `inline_style`; the three budgets `inline_note_suggested_lines`, `inline_note_max_lines` and `inline_note_max_columns`, the last a hard cap on the longest line of the note; the layout pair `inline_note_layout` ([§FS-inline-citation-style.3.3](FS-inline-citation-style.md#33-inline_note_layout--where-the-citations-sit)) and `inline_note_layout_check` ([§FS-inline-citation-style.4.4](FS-inline-citation-style.md#44-warnings-and-errors--opt-in-layout-deviations)); and `warn_on_suggested` ([§FS-inline-citation-style.4.2](FS-inline-citation-style.md#42-warnings--opt-in-soft-cap)). The budgets and the layout apply only when `inline_style = "citation-with-note"`.

Under v2 the budgets are measure tables, `[schema.notes.lines]`, `[schema.notes.columns]`, `[schema.notes.text]` and `[schema.notes.layout]`, whose keys are strengths and whose values are thresholds, measured as [§FS-inline-citation-style.2.3](FS-inline-citation-style.md#23-counting-lines-and-columns) counts ([§FS-config-v2.schema.measures](FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys)).

### 2.3 Counting lines and columns

A note is measured over its comment block ([§FS-languages.comment-block](FS-languages.md#comment-block-the-comment-block)), by its physical lines and by the characters on its longest line.

#### 2.3.1 Lines

A note's line count is the physical extent of its block, `last_line - first_line + 1`: a single `// …` line counts as 1, and a three-line `//` run or `/* … */` block counts as 3. Blank intra-block lines (a ` * ` filler inside `/* … */`, an empty `//` line) count toward the total.

#### 2.3.2 Columns

A note's width is the number of **characters** on its longest line, counted from column 1, indentation and comment delimiters included: one column per Unicode scalar value, whatever UTF-8 spends encoding it, so `é`, `—`, `×` and the `§` marker are one column each. It is not the byte length, which is the start column the scanner records on each citation for an editor to jump to, and the two agree only on a line of pure ASCII. Nor is it the display width: a tab is one column and so is a double-width glyph, so the cap moves with neither a tabstop setting nor a font. Decided in [§DF-note-columns-are-characters](../decisions/functional/DF-note-columns-are-characters.md#df-note-columns-are-characters-a-note-column-is-one-character-not-one-byte-and-not-one-display-cell).

## 3. Styles

### 3.1 `citation-only`

A note may contain only its comment prefixes and one or more citation tokens joined by separators ([§FS-inline-citation-style.1.4](FS-inline-citation-style.md#14-notes)), so a citing comment is a pure pointer, for a project that keeps its rationale in the spec. A note that carries note text is an error ([§FS-inline-citation-style.4.1](FS-inline-citation-style.md#41-errors--hard-caps)). Under this style the `inline_note_*` keys have no effect ([§FS-config.3.1](FS-config.md#31-reference--citation-form)).

### 3.2 `citation-with-note`

A note may carry note text beside one or more citation tokens, before, after or between them, bounded by `inline_note_max_lines` and `inline_note_max_columns` ([§FS-inline-citation-style.2.3](FS-inline-citation-style.md#23-counting-lines-and-columns)). The budgets are this style's only constraint unless `inline_note_layout` sets an arrangement ([§FS-inline-citation-style.3.3](FS-inline-citation-style.md#33-inline_note_layout--where-the-citations-sit)).

### 3.3 `inline_note_layout` — where the citations sit

`inline_note_layout` constrains where the citation tokens sit in a note, never whether a note may exist and never how large it is; layout and size are judged independently, so one line may earn a layout finding, a column finding, or both. `any`, the default, imposes nothing ([§DF-inline-note-layout.2.6](../decisions/functional/DF-inline-note-layout.md#26-default-off-and-no-version-bumps)). `citation-first-colon` requires the form `<cite>[, <cite>]*: <note>`. Precisely: let `L` be a run of one or more recognized citation tokens joined by exactly `, ` (comma, one space), `W` one or more space characters, `T` any non-empty text, and `ε` the end of the content. A line **conforms** when its content matches

```
L ":" ( W T | ε )
```

**Which lines are judged.** Only a note that carries note text ([§FS-inline-citation-style.1.4](FS-inline-citation-style.md#14-notes)) is judged; a pure citation comment is exempt in either spelling of a chain. In a judged note, a line is judged when its content holds at least one citation token **and** either it is the first such line of the block or its content opens with a citation token. Every other line is free, and so is every citation after the run that opens a judged line. A `// §<ID>` line followed by a prose-only line in the same block is one note that carries note text, so the citation line is judged and fails ([§DF-inline-note-layout.2.2](../decisions/functional/DF-inline-note-layout.md#22-per-line-not-per-site)).

**A line's content.** The form is read on the line's content: what is left once the comment prefix (`//`, `///`, `//!`, `#`, `;`, `--`, ` * `, `/**`, a docstring quote, …), any block closer (`*/`, a closing docstring quote) and the whitespace that indents the content past the prefix have been stripped, the same prefix stripping [§FS-inline-citation-style.1.4](FS-inline-citation-style.md#14-notes) applies. One leading Markdown list marker, `-`, `*`, `+`, or an ordered `1.` / `1)`, followed by at least one space, is skipped the same way, so `// - §<ID>: note` opens with its citation run. The skip is this rule's alone: [§FS-inline-citation-style.1.4](FS-inline-citation-style.md#14-notes) still reads the marker as note text, so `inline_style = "citation-only"` judges a bulleted pointer as carrying note text.

**Exact.** Inside the content the form is exact, so a near miss such as a space where `, ` belongs, a space before the colon or a tab as `W` is a deviation ([§DF-inline-note-layout.2.3](../decisions/functional/DF-inline-note-layout.md#23-one-exact-canonical-form)).

Under `strict = false` a bare `// FS-x: note` line is a source declaration heading, not a note, so it is never judged ([§FS-inline-citation-style.1.3](FS-inline-citation-style.md#13-what-is-not-a-site)); with the marker written, `// §<ID>: note` is a citation carrying a note.

Conforming:

```rust
// §FS-check.3.1: dangling-ref enforcement entry point.
// §FS-check.3.1, §FS-config.3.1: the rule and the key that turns it on.
// §FS-check.3.1: the rule (see also §FS-config.3.1).
// Walks every recognized citation and resolves it.
// §FS-check.3.1: one error per unresolved ID.
//   §FS-check.3.1: indented past the prefix
// - §FS-check.3.1: a bulleted grounded point
// 1. §FS-check.3.1: an ordered one
/* §FS-check.3.1: a note that runs past one line and
   still names §FS-config.3.1 on the way */
```

Nonconforming:

```rust
// §FS-check.3.1 dangling-ref enforcement entry point
// dangling-ref enforcement entry point (§FS-check.3.1)
// §FS-check.3.1 §FS-config.3.1: the rule and its key
// §FS-check.3.1,§FS-config.3.1: the rule and its key
// §FS-check.3.1 — dangling-ref enforcement entry point
// Walks every recognized citation and resolves it.
// then §FS-check.3.1 decides                        <- first citation-bearing line
/* §FS-check.3.1: a note that runs past one line and
   §FS-config.3.1 opens the continuation */
```

## 4. Enforcement (`grund check`)

`check` reports a note that breaks the configured style as `inline-citation-style` ([§FS-check.3.10](FS-check.md#310-inline-citation-style-violation), [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)), located, rendered, selected and counted toward the exit code as every finding is ([§FS-check.2.1](FS-check.md#21-report-format)). Under v2 the strength of the threshold a note breaks picks the channel ([§FS-config-v2.rules.strengths](FS-config-v2.md#rulesstrengths-one-strength-vocabulary-two-severities)).

### 4.1 Errors — hard caps

Each of these is an error: a note that carries note text under `inline_style = "citation-only"`, a note longer than `inline_note_max_lines`, and a note whose longest line is wider than `inline_note_max_columns`. Each is located at the note's first line, and a note that breaks more than one cap gets one finding per cap:

```text
src/lib.rs:1: error: inline citation must carry no prose; move the prose into a comment block of its own
src/auth.rs:1: error: inline note is 5 lines, over the 3-line maximum: lines 1-5 cite §FS-001-auth.1, §FS-001-auth.2; a blank line splits a note, an empty comment line does not; shorten the note, or move its detail into a doc-comment
src/lib.rs:7: error: inline note is 47 columns, over the 40-column maximum: line 7 cites §FS-001-alpha; over-limit lines: line 7: 47 columns; shorten or rewrap its widest line
```

The line and column findings put the measured size, physical lines ([§FS-inline-citation-style.2.3.1](FS-inline-citation-style.md#231-lines)) or the characters of the note's longest line ([§FS-inline-citation-style.2.3.2](FS-inline-citation-style.md#232-columns)), beside the cap; the size pluralises by its own value (`1 line`, `2 lines`, `1 column`, `2 columns`), and the cap, `3-line` or `40-column`, never does. Each names the note by the span of its whole block, `lines <first>-<last>`, or `line <first>` alone when the two coincide (only possible for the column cap), and by every citation token of the block as written — marker, qualifier, section — in source order, deduplicated after the first occurrence, joined by `, ` as [§FS-inline-citation-style.3.3](FS-inline-citation-style.md#33-inline_note_layout--where-the-citations-sit) joins a citation run. The line-count finding then adds the block rule of [§FS-languages.comment-block](FS-languages.md#comment-block-the-comment-block), the boundary a split has to cross.

The column finding adds `; over-limit lines: <line widths>` after its site clause and before its fix. `<line widths>` lists every line of the block strictly wider than that finding's threshold, in ascending source order, separated by `, `, each as `line L: W columns`, where `L` is the 1-based physical line and `W` its width as [§FS-inline-citation-style.2.3.2](FS-inline-citation-style.md#232-columns) measures it, with `column` only when `W = 1`. A line exactly at the threshold is left out, and the label is `over-limit lines` even for one entry. A v2 column threshold of any strength carries the same suffix, measured against its own threshold ([§FS-config-v2.schema.measures](FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys)), and a hard column finding suppresses the softer column findings of the same note. The fix closes every message, as its catalog row words it ([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)).

### 4.2 Warnings — opt-in soft cap

At `warn_on_suggested = false`, the default, a soft-cap overrun is silent at `check` time: the soft cap is guidance the agent-facing copy states ([§FS-inline-citation-style.5.1](FS-inline-citation-style.md#51-the-budgets-line)).

At `warn_on_suggested = true`, a note whose line count exceeds `inline_note_suggested_lines` but stays within `inline_note_max_lines` is a warning. It is located, measured and worded as the line-count finding of [§FS-inline-citation-style.4.1](FS-inline-citation-style.md#41-errors--hard-caps) is, with `preferred limit` in place of `maximum`:

```text
src/lib.rs:1: warning: inline note is 2 lines, over the 1-line preferred limit: lines 1-2 cite §FS-001-alpha; a blank line splits a note, an empty comment line does not; shorten the note, or move its detail into a doc-comment
```

The soft cap counts lines only ([§FS-inline-citation-style.6.3](FS-inline-citation-style.md#63-no-second-column-measure)).

### 4.4 Warnings and errors — opt-in layout deviations

Layout checking is off by default, twice over: `inline_note_layout = "any"` leaves no layout to deviate from, and `inline_note_layout_check = "off"` keeps a configured layout as agent-facing guidance only ([§FS-inline-citation-style.5.3](FS-inline-citation-style.md#53-the-layout-sentence)), and no line is classified at all. With `inline_note_layout = "citation-first-colon"`, `inline_note_layout_check` picks the channel ([§DF-inline-note-layout.2.1](../decisions/functional/DF-inline-note-layout.md#21-two-keys-the-style-and-the-level-it-is-enforced-at), [§DF-require-grounding.2.4](../decisions/functional/DF-require-grounding.md#24-off-by-default)):

| `inline_note_layout_check` | result |
|---|---|
| `off` (default) | silent |
| `warn` | one **warning** per nonconforming line |
| `error` | one **error** per nonconforming line |

Each finding is located at its nonconforming line, not at the note's opener, because that is the line the author edits ([§DF-inline-note-layout.2.2](../decisions/functional/DF-inline-note-layout.md#22-per-line-not-per-site)). The message is the same at both severities, so moving a project from `warn` to `error` changes the exit code and nothing a reader has to re-learn; it names the canonical shape with the configured marker:

```text
src/lib.rs:4: error: inline note must open with its citations and a colon (§<ID>: note); rewrite the line in that form
```

Report order is the deterministic order of [§FS-errors.4](FS-errors.md#4-determinism): the severity chooses the channel, a text report puts errors ahead of warnings, and the JSON order ignores the channel.

## 5. Agent-facing rendering

`grund init` renders the house style into the managed block it writes into `AGENTS.md`, `CLAUDE.md` and the sibling agent entrypoints ([§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)), as sentences read from the active values, each under the conditions its section states.

### 5.1 The budgets line

The copy opens with one budgets line:

- `inline_style = "citation-only"` → `Inline citations carry no prose — put rationale in the spec.`
- `inline_style = "citation-with-note"`, `suggested_lines == max_lines` → e.g. `Inline notes: ≤ 1 line, ≤ 100 columns.`
- `inline_style = "citation-with-note"`, `suggested_lines < max_lines` → e.g. `Inline notes: ≤ 1 line preferred, hard cap 3 lines; ≤ 100 columns.`

When the soft and the hard cap are the same number, only the number is said.

### 5.2 The block sentence

Under `citation-with-note` only, one further sentence follows the budgets line and precedes the layout and doc-comment sentences ([§FS-inline-citation-style.5.3](FS-inline-citation-style.md#53-the-layout-sentence), [§FS-inline-citation-style.5.4](FS-inline-citation-style.md#54-the-doc-comment-sentence)): `A note is one comment block: a blank line splits it, an empty comment line does not.`, the block rule of [§FS-languages.comment-block](FS-languages.md#comment-block-the-comment-block) where the agent needs it to act on a cap finding.

### 5.3 The layout sentence

Under `citation-with-note` only, when `inline_note_layout = "citation-first-colon"` is set, one further sentence follows the block sentence ([§FS-inline-citation-style.5.2](FS-inline-citation-style.md#52-the-block-sentence)), naming the canonical form with the configured marker and placeholder IDs — e.g. ``Lay each note out citation-first: `// §<ID>: <note>` (several citations: `// §<ID>, §<ID>: <note>`).`` `inline_note_layout_check` does not change the sentence: it says what to write, never how the gate reports a deviation ([§DF-inline-note-layout.2.1](../decisions/functional/DF-inline-note-layout.md#21-two-keys-the-style-and-the-level-it-is-enforced-at)).

The sentence is wider than the gate: it asks the agent to lay out every citing comment it writes this way, while `check` judges notes only, never a doc-comment ([§FS-inline-citation-style.1.1](FS-inline-citation-style.md#11-doc-comments-are-not-sites)) or one that declares an ID ([§FS-inline-citation-style.1.3](FS-inline-citation-style.md#13-what-is-not-a-site)), so under `warn` an empty report means the notes are clean, not that every `§<ID>` line in the repository is citation-first.

### 5.4 The doc-comment sentence

One last sentence closes the rendered copy at **every** `inline_style`, after whatever the keys above produced, because where the gate stops is part of the house style an agent needs ([§DF-doc-comments-are-not-notes.2.5](../decisions/functional/DF-doc-comments-are-not-notes.md#25-one-rendered-sentence-and-no-version-bump)):

``Doc-comments (`///`, `//!`, `/** */`, a docstring, a Go, Ruby, shell, SQL or Lisp comment right above a definition) are documentation, not notes: they are never measured, so cite in-sentence there.``

### 5.5 Rendered, not live

The sentences of [§FS-inline-citation-style.5.1](FS-inline-citation-style.md#51-the-budgets-line)–[§FS-inline-citation-style.5.4](FS-inline-citation-style.md#54-the-doc-comment-sentence) are *rendered*, not live: `grund init` writes them into the managed block, and `check` compares them with the active config as it compares the rest of the block ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)). A project that adopts `inline_note_layout`, changes its value, drops back to `any`, or edits `[schema.notes]` therefore sees `check` report the block's `rules` section stale ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)) until it re-runs `grund init`.

### 5.6 No managed-block version

Retired: these sentences once moved no managed-block version, because `check` did not compare them. It now compares every line, so a change in grund to their wording moves the version like any other change to the block's render ([§FS-init.2.3.7](FS-init.md#237-the-block-version)). This address is kept so citations written before the change still resolve.

### 5.7 `grund config show` is the machine-readable form

`grund config show` ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)) prints each inline-style setting in effect, in the config file's own version, at its value with no collapse ([§FS-inline-citation-style.5.1](FS-inline-citation-style.md#51-the-budgets-line)). In a version-2 file a setting that is off, such as `[schema.notes.layout]` while no layout is required, prints nothing, and `grund config explain` answers for it ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)).

## 6. Non-goals

Deliberately out of scope.

### 6.1 No rewrite in `grund fmt`

`grund fmt` rewrites no note to fit its budget and normalizes no layout, in `--check` or in `--write`: a budget or layout violation is `check`'s alone, because prose cannot be safely shortened, and moving a citation across the prose around it is a prose edit that needs judgment no formatter has ([§DF-inline-note-layout.2.5](../decisions/functional/DF-inline-note-layout.md#25-check-only-grund-fmt-is-not-involved)). Its token rewrites ([§FS-fmt.2](FS-fmt.md#2-behavior)) and cross-reference emission ([§FS-fmt.6](FS-fmt.md#6-cross-reference-emission)) still run on a note, and an expansion can lengthen a note past its cap ([§FS-fmt.2.4](FS-fmt.md#24-shorthand-to-canonical)).

### 6.2 No scope beyond notes

- No scope growth. Spec text in Markdown bodies is not capped ([§FS-inline-citation-style.1.3](FS-inline-citation-style.md#13-what-is-not-a-site)), and layout is judged on notes only ([§FS-inline-citation-style.3.3](FS-inline-citation-style.md#33-inline_note_layout--where-the-citations-sit)) — never in Markdown bodies, never on a comment trailing code ([§FS-inline-citation-style.1.1.3](FS-inline-citation-style.md#113-accepted-corners)), never on the code line below the comment.
- No host-language parsing to find definitions ([§FS-non-goals.3](FS-non-goals.md#3-code-ast-parsing)): a block's kind is the one test of [§FS-inline-citation-style.1.1.2](FS-inline-citation-style.md#112-languages), and a miss is accepted ([§FS-inline-citation-style.1.1.3](FS-inline-citation-style.md#113-accepted-corners)) or fixed by widening a starter set ([§FS-inline-citation-style.6.4](FS-inline-citation-style.md#64-no-per-project-reshaping-of-the-rule)).

### 6.3 No second column measure

- No `suggested_columns` knob. Column width is a single hard cap, a binary "too long", as editors and formatters treat line length.
- No display-width awareness: a column is what [§FS-inline-citation-style.2.3.2](FS-inline-citation-style.md#232-columns) counts.

### 6.4 No per-project reshaping of the rule

- No per-kind or per-file overrides. The style is repo-wide, so two correctly configured `grund` installs agree on whether a tree is well-formed ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)).
- No warning for a hard-cap miss: a hard cap is always an error ([§FS-inline-citation-style.4.1](FS-inline-citation-style.md#41-errors--hard-caps)).
- No per-rule severity remap. `inline_note_layout_check` selects which channel *this* rule speaks through, from a fixed set; it does not let a project re-level any other rule, and it does not change what an error or a warning means ([§FS-non-goals.9](FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization), [§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)).
- No configuration of the language table. The recognizers and the extensions they claim are built in ([§FS-inline-citation-style.1.1.2](FS-inline-citation-style.md#112-languages)), so what a note *is* cannot differ between two installs ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree), [§DF-doc-comments-are-not-notes.2.4](../decisions/functional/DF-doc-comments-are-not-notes.md#24-no-knob)). The table and its starter sets may widen without a `grund_config_version` bump ([§FS-config.5](FS-config.md#5-schema-versioning)).
