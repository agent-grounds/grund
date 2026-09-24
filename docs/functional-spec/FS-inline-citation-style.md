# FS-inline-citation-style: configurable shape of inline code-comment citations

An inline citation in a code comment can carry a short rationale next to the `§<ID>` token — the project explains *why* this clause is grounded in that spec point. This spec defines a project-level house style for that rationale: whether it is allowed at all, how long it may run, and where the citation sits inside it. The same configuration drives `grund check` enforcement and the agent-facing copy in `AGENTS.md` / `CLAUDE.md` so the LLM that authors citations and the linter that validates them agree on the rules. Serves [§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable) and [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, body, section), [§FS-terms.terms.2](FS-terms.md#terms2-citations) (marker,
citation, shorthand, citation site), [§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (stub, doc-comment, note),
[§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, workspace, alias), [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, severity), and
[§FS-terms.terms.6](FS-terms.md#terms6-rules-and-directions) (direction).

- **comment block** — The contiguous run of comment lines a site is measured over. A blank line
  ends one; an empty comment line does not.
- **definition-starter** — The language keyword on the line below a block that makes the block a
  doc-comment rather than an inline comment.
- **leading comment** — A block with only blank lines or a shebang above it — how a language
  with no doc-comment syntax spells a module doc.

## 1. Scope

An **inline citation site** is an *inline comment* block — a maximal run of adjacent comment/docstring lines, by the scanner's existing line classes ([AR-scanner.4](../architecture/AR-scanner.md#4-inline-declarations-in-language-doc-comments)) — that contains at least one citation token recognized by [§FS-check.1.1](FS-check.md#11-recognized-citations). An inline citation site never spans more than one block: a code line, a blank line, or a different comment style ends a block, and an empty comment line does not ([§FS-inline-citation-style.1.2](FS-inline-citation-style.md#12-comment-blocks)). A **doc comment** block is not one, whatever it carries: [§FS-inline-citation-style.1.1](FS-inline-citation-style.md#11-doc-comments-are-not-sites) draws that line and every rule below stops at it. What this spec does not govern is listed in [§FS-inline-citation-style.1.3](FS-inline-citation-style.md#13-what-is-not-a-site), and what inside an inline citation site counts as a *note* is defined in [§FS-inline-citation-style.1.4](FS-inline-citation-style.md#14-notes).

### 1.1 Doc comments are not sites

A **doc comment** documents the definition that follows it, or the file it opens. An **inline comment** is every other comment — one among statements, a detached block, a note beside the clause it grounds. Only an inline comment is an inline citation site. Nothing in this spec reaches a citation written inside a doc comment: not `citation-only` ([§FS-inline-citation-style.3.1](FS-inline-citation-style.md#31-citation-only)), not the line and column budgets ([§FS-inline-citation-style.2.3](FS-inline-citation-style.md#23-counting-lines-and-columns), [§FS-inline-citation-style.4.1](FS-inline-citation-style.md#41-errors--hard-caps), [§FS-inline-citation-style.4.2](FS-inline-citation-style.md#42-warnings--opt-in-soft-cap)), not the layout ([§FS-inline-citation-style.3.3](FS-inline-citation-style.md#33-inline_note_layout--where-the-citations-sit), [§FS-inline-citation-style.4.4](FS-inline-citation-style.md#44-warnings-and-errors--opt-in-layout-deviations)). Everything else still does: the citation resolves and is dangling when it does not ([§FS-check.3.1](FS-check.md#31-dangling-citation)), counts toward the grounding floor ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)) and the citation directions ([§FS-check.3.11](FS-check.md#311-missing-required-citation), [§FS-check.3.12](FS-check.md#312-forbidden-citation)), is judged by the target project's shorthand policy when it is a persisted shorthand ([§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)), and is linkified by `grund fmt` ([§FS-fmt.6](FS-fmt.md#6-cross-reference-emission)).

This generalizes the exemption a doc comment that *declares* an ID already has ([§FS-inline-citation-style.3.3.6](FS-inline-citation-style.md#336-same-scope-as-the-rest-of-this-spec), [§FS-inline-citation-style.5](FS-inline-citation-style.md#5-agent-facing-rendering)): a class Javadoc or a module doc that names the spec point it implements, in the sentence that needs it, is the language's own documentation and renders into generated docs, not a note that ran long ([§DF-doc-comments-are-not-notes.2.1](../decisions/functional/DF-doc-comments-are-not-notes.md#21-the-site-is-an-inline-comment-a-doc-comment-is-not-a-site)).

#### 1.1.1 Recognizers

Which kind a block is, is read from the file's extension and one test on the block itself — never by parsing the host language ([§FS-inline-citation-style.6](FS-inline-citation-style.md#6-non-goals)). Two recognizers and a default:

1. **Marker languages** spell a doc comment with a marker of their own, so the marker *is* the language's own answer. The block is a doc comment when its line-comment marker — or, for a block comment, its opening line — is the doc form.
2. **Position languages** spell a doc comment exactly like every other comment (Go `//`, Ruby `#`, shell `#`, SQL `--`), so position answers instead. The block is a doc comment when the line immediately below it, with no blank line between, is a **definition-starter** for that language, or when the block is the file's **leading comment**: every line above it is blank, or is line 1 and a `#!` shebang. A leading comment is how these languages spell a module doc — a header at the top of a Go, Ruby, shell, or SQL file documents the file, the way `//!` and a module docstring do where a marker exists.
3. **Any other extension** has no doc-comment notion, so every comment block in it is inline. That is the behavior of every release before this rule, so nothing a conformant tree already passes changes on its account.

#### 1.1.2 Languages

Each extension belongs to one recognizer of [§FS-inline-citation-style.1.1.1](FS-inline-citation-style.md#111-recognizers), which tests for the doc form this table gives:

| extensions | recognizer | doc comment when |
|---|---|---|
| `rs` `c` `h` `cpp` `cc` `cxx` `hpp` `hh` `hxx` `m` `mm` `java` `cs` `kt` `kts` `scala` `swift` `js` `jsx` `mjs` `cjs` `ts` `tsx` `php` `dart` | marker (C family) | a line run whose marker is exactly `///` (`////` is a rule line, not a doc comment; JDK 23 Markdown documentation comments are `///` too) or `//!`; a block comment opening `/**` — not the empty `/**/` — or `/*!`. A plain `//`, `/* … */`, or PHP `#` block is inline wherever it sits, a `//` block directly above a `fn` included: by the language's own rules that is not documentation. |
| `py` | marker (docstring) | a `"""` or `'''` docstring block. A `#` block is inline, including one directly above a `def` — by PEP 257 only a docstring is documentation. |
| `lua` | marker (LDoc) | a `--` run whose first line starts with `---`, exactly three dashes; `----` is a rule line. Continuation lines are plain `--` lines and already belong to the same block. |
| `hs` `lhs` | marker (Haddock) | a `--` run whose first line's content after the `--` starts, after optional spaces, with `\|` or `^`. |
| `r` `R` | marker (roxygen2) | a `#` run whose first line starts with `#'`. |
| `go` | position | definition-starters `func`, `type`, `var`, `const`, `package`. |
| `rb` | position | definition-starters `class`, `module`, `def`. |
| `sh` `bash` `zsh` | position | `function <name>`, or `<name>()` / `<name> ()`, with `<name>` matching `[A-Za-z_][A-Za-z0-9_]*`. |
| `sql` | position | the definition-starter `create`, matched case-insensitively, so `CREATE OR REPLACE FUNCTION …` counts. |

A **definition-starter** matches when the next line, with leading whitespace removed, begins with the keyword followed by a non-identifier character or the end of the line: `func(` and `func main` match, `functional` does not, and an indented Ruby `def` inside a `class` still counts.

#### 1.1.3 Accepted corners

Four corners are known and accepted rather than repaired ([§DF-doc-comments-are-not-notes.2.6](../decisions/functional/DF-doc-comments-are-not-notes.md#26-the-corners-it-accepts)):

- A **dangling doc comment** — a `/** … */` or `///` inside a method body, which `javac`'s `-Xlint:dangling-doc-comments` and `rustc`'s `unused_doc_comments` already warn about — is a doc comment by its marker and is not measured. The language's own lint is the tool for a doc comment in the wrong place.
- **Position recognition is recognition, not parsing.** A Go `var` inside a function body and a Ruby `private def` are classified by the same one-line test: the first reads as a doc comment, the second does not. A miss in either direction only changes whether a block is *measured*; it never changes what a citation resolves to, or whether it resolves at all. The starter sets can widen later without a `grund_config_version` bump ([§FS-config.5](FS-config.md#5-schema-versioning)).
- A comment **trailing code** on the same line (`foo(); // §<ID>: note`) is what it already was: not a site ([§FS-inline-citation-style.3.3.6](FS-inline-citation-style.md#336-same-scope-as-the-rest-of-this-spec)).
- **Blank-line adjacency is adjacency.** A `#` block, a blank line, then a `def` is an inline comment — the blank line broke the block off the definition, the same way it breaks one block into two ([§FS-inline-citation-style.1.2](FS-inline-citation-style.md#12-comment-blocks)).

### 1.2 Comment blocks

The block forms say where a block begins and ends. They are the scanner's existing normalization, not a verdict on which blocks are sites:

- `//` / `///` / `//!` line comments: a run of adjacent lines whose first non-whitespace token is the same line-comment marker.
- `#`, `;`, `--` line comments: same rule per marker (see [§FS-config.3.5](FS-config.md#35-scan--what-gets-walked) for the full prefix set).
- `/* … */` block comments, `/**`- and `/*!`-opened alike: from opener to closer.
- Python triple-quoted docstrings (`""" … """` / `''' … '''`): from the opening triple-quote to the matching close.

Adjacency is broken by any line that is not part of the same block: a code line, a blank line, or a different comment style. An empty comment line — the marker alone, such as `//` or `#`, with nothing after it — stays inside the run: it carries the marker, so only a line without one breaks it.

### 1.3 What is not a site

This spec governs inline citation sites only. It does **not** govern:

- Citations inside Markdown spec bodies (prose in `docs/`, `tests/e2e/`, or any other `.md` file the scanner reads). Spec text governs itself.
- Declarations themselves — `# FS-foo: …` and `/// FS-foo: …` are declaration headings ([AR-scanner.2.1](../architecture/AR-scanner.md#21-declaration-detection)), and the scanner already excludes a declaration's own heading from the citations it records ([AR-scanner.2.3](../architecture/AR-scanner.md#23-citation-detection)). A doc-comment whose first line is a declaration heading and whose remaining lines are spec body is a declaration, not a citation site.
- Inline-spec stubs (`# <ID>: [<text>](<path>)`) — a `docs/` shape, not a code-comment shape.
- Bare ID-shaped tokens that the scanner already excludes from citations: tokens inside string literals in source files ([AR-scanner.2.3](../architecture/AR-scanner.md#23-citation-detection)), and any bare token at all under `[reference] strict = true` ([§FS-config.3.1](FS-config.md#31-reference--citation-form)). If the scanner doesn't see a citation, no site exists.
- Doc comments ([§FS-inline-citation-style.1.1](FS-inline-citation-style.md#11-doc-comments-are-not-sites)).

### 1.4 Notes

A *note* is any non-whitespace text inside an inline citation site that is not a comment-prefix character and not part of a `§<ID>[.<section>]` token (workspace-qualified `§<alias>/<ID>` tokens, [§FS-workspace.1](FS-workspace.md#1-citation-syntax), are citation tokens, not notes). What separates two citation tokens of one chain on one line is not a note either: whitespace, or a single comma with optional whitespace around it. So `// §FS-check.3.1  §FS-config.3.1` and `// §FS-check.3.1, §FS-config.3.1` are both pure citation comments — the second spells the chain the way [§FS-inline-citation-style.3.3](FS-inline-citation-style.md#33-inline_note_layout--where-the-citations-sit) requires a note's citation run to be spelled, and writing it must not turn a pointer into prose. Anything else between two tokens is a note: a second comma, a ` + `, a ` / `, an `and`. The exemption is bounded by the line because the separator is read between two tokens the same line holds: a chain wrapped across two comment lines — `// §FS-check.3.1,` and then `// §FS-config.3.1` on the next line — leaves the trailing comma with no following token to join, so that block carries a note, and under a configured layout ([§FS-inline-citation-style.3.3](FS-inline-citation-style.md#33-inline_note_layout--where-the-citations-sit)) its citation-bearing lines are judged like any other.

## 2. Configuration

The keys live in `[reference]`, and their values and defaults are stated there ([§FS-config.3.1](FS-config.md#31-reference--citation-form)): `inline_style`; the three budgets `inline_note_suggested_lines`, `inline_note_max_lines`, and `inline_note_max_columns`, the last a hard cap on the longest line of the inline citation site; the layout pair `inline_note_layout` ([§FS-inline-citation-style.3.3](FS-inline-citation-style.md#33-inline_note_layout--where-the-citations-sit)) and `inline_note_layout_check`; and `warn_on_suggested`. The budgets and the layout apply only when `inline_style = "citation-with-note"`.

### 2.1 Defaults

The zero-config defaults ([§GOAL-zero-config](../goals.md#goal-zero-config-works-on-any-conformant-tree)) are the ones [§FS-config.3.1](FS-config.md#31-reference--citation-form) gives. They preserve the convention this project already follows — a one-line note next to each `§<ID>` citation — and never reject inline citation sites that an existing conformant tree was already writing. `inline_note_layout = "any"` is that promise for the layout axis in particular: it imposes no shape at all, so a tree that never had a house style gains no findings and pays no classification work. `inline_note_layout_check = "off"` extends the second half of that promise to a tree that *has* adopted a house style but not the gate: with no channel for a verdict to reach, no line is classified either ([§FS-inline-citation-style.4.4](FS-inline-citation-style.md#44-warnings-and-errors--opt-in-layout-deviations)).

### 2.2 Load-time invariants

The load-time rules for these keys are stated with their schema in [§FS-config.3.1](FS-config.md#31-reference--citation-form): the invariant `inline_note_suggested_lines ≤ inline_note_max_lines` — a soft cap above the hard cap is meaningless — the two closed layout enums, and which keys are inert, still parsed and printed, under which settings. A violation fails on load with the standard config-error shape ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)), the same way an unknown `inline_style` does. The two layout keys are independent of each other and of the budgets, so `inline_note_layout_check` is legal at every layout.

### 2.3 Counting lines and columns

A site is measured three ways, each over the comment block [§FS-inline-citation-style.1.2](FS-inline-citation-style.md#12-comment-blocks) bounds: its physical lines ([§FS-inline-citation-style.2.3.1](FS-inline-citation-style.md#231-lines)), the characters on its longest line ([§FS-inline-citation-style.2.3.2](FS-inline-citation-style.md#232-columns)), and whether it carries a note ([§FS-inline-citation-style.2.3.3](FS-inline-citation-style.md#233-note-presence)).

#### 2.3.1 Lines

A site's line count is the physical extent of its comment block — `last_line - first_line + 1`. A single `// …` line counts as 1; a three-line `//` run or `/* … */` block counts as 3. Blank intra-block lines (a ` * ` filler inside `/* … */`, an empty `//` line) count toward the total: the rule measures the comment's physical size.

#### 2.3.2 Columns

A site's column width is the number of **characters** on its longest constituent line, counted from column 1: one column per Unicode scalar value, whatever UTF-8 spends encoding it. `é`, `—`, `×`, and the `§` marker itself are one column each and not the two or three bytes they occupy, so a note in accented or non-Latin prose gets the same budget as the same note in ASCII, and a project that configures `100` gets 100 columns in every language it writes ([§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible)).

The width is not the **byte** length: that is the start column the scanner records on each citation ([AR-scanner.3](../architecture/AR-scanner.md#3-output)), which addresses a position for an editor to jump to rather than bounding a length, and the two agree only on a line of pure ASCII. Nor is it the **display width**: a tab is one column and so is a double-width glyph, so the cap moves with neither a tabstop setting nor the font a reader renders the file in. Why the character is the unit, and why reading it as bytes was a defect rather than a policy, is decided in [§DF-note-columns-are-characters](../decisions/functional/DF-note-columns-are-characters.md#df-note-columns-are-characters-a-note-column-is-one-character-not-one-byte-and-not-one-display-cell).

#### 2.3.3 Note presence

After stripping a line's comment-prefix tokens (`//`, `*`, the opening `/**`, the docstring `"""`, etc.), every citation token, and each separator between two consecutive citation tokens on that line that [§FS-inline-citation-style.1.4](FS-inline-citation-style.md#14-notes) excludes from a note — whitespace with at most one comma — any non-whitespace character remaining on any line of the inline citation site is a note. The stripping is per line, so a comma that a wrapped chain leaves at the end of a line joins nothing and stays note text ([§FS-inline-citation-style.1.4](FS-inline-citation-style.md#14-notes)). This is the same line-normalization the scanner already does for declaration detection ([AR-scanner.4](../architecture/AR-scanner.md#4-inline-declarations-in-language-doc-comments)) — applied to the whole block instead of one line.

## 3. Styles

### 3.1 `citation-only`

An inline citation site may contain only its comment prefix(es) and one or more `§<ID>[.<section>]` tokens, separated by whitespace or by a single comma ([§FS-inline-citation-style.1.4](FS-inline-citation-style.md#14-notes)). A note anywhere in it ([§FS-inline-citation-style.2.3.3](FS-inline-citation-style.md#233-note-presence)) is an error. The intended use is repositories that prefer to keep all rationale in the spec, so code comments at inline citation sites become pure pointers. Under this style the `inline_note_*` keys have no effect ([§FS-inline-citation-style.2](FS-inline-citation-style.md#2-configuration)).

### 3.2 `citation-with-note`

A citation site may contain one or more citation tokens **plus** a note, bounded by `inline_note_max_lines` and `inline_note_max_columns` ([§FS-inline-citation-style.2.3](FS-inline-citation-style.md#23-counting-lines-and-columns)). The note may appear before, after, or between citation tokens — the budgets are this style's only constraint. A project that also wants one canonical arrangement of citation and note sets `inline_note_layout` ([§FS-inline-citation-style.3.3](FS-inline-citation-style.md#33-inline_note_layout--where-the-citations-sit)); at its default `any` the style is exactly as permissive as it reads here.

### 3.3 `inline_note_layout` — where the citations sit

`inline_style` says whether a note may exist and the budgets say how big it may be; neither says where the `§<ID>` tokens sit inside it. `inline_note_layout` is that third axis, orthogonal to the other two: it constrains arrangement only, never presence and never size. `"any"`, the default, imposes nothing — [§FS-inline-citation-style.3.2](FS-inline-citation-style.md#32-citation-with-note) as written ([§FS-inline-citation-style.3.3.10](FS-inline-citation-style.md#3310-why-any-is-the-default)). `"citation-first-colon"` requires the canonical form `<cite>[, <cite>]*: <note>`. Precisely: let `L` be a run of one or more recognized citation tokens joined by exactly `, ` (comma, one space), `W` one or more spaces, `T` any non-empty text, and `ε` the end of the content. A line **conforms** when its content ([§FS-inline-citation-style.3.3.8](FS-inline-citation-style.md#338-reading-a-lines-content)) matches

```
L ":" ( W T | ε )
```

Seven rules complete the definition, rule *n* at [§FS-inline-citation-style.3.3](FS-inline-citation-style.md#33-inline_note_layout--where-the-citations-sit).*n*; [§FS-inline-citation-style.3.3.9](FS-inline-citation-style.md#339-examples) shows lines that conform and lines that do not.

#### 3.3.1 Per line, not per site — and only where a note opens

A line of an inline citation site is judged when it carries at least one recognized citation token inside its content **and** either it is the first such line of that block, or its content opens with a citation token. Every other line is unconstrained. So a line with no citation says nothing about layout, and a `//` block may open with a summary line and carry its `// §<ID>: …` lines below it. And a citation-bearing line further down that opens with *prose* is the continuation of a note that already opened correctly, so a note that wraps within its line budget ([§FS-inline-citation-style.2.3.1](FS-inline-citation-style.md#231-lines)) may name a second spec point on its continuation line — the freedom [§FS-inline-citation-style.3.3.3](FS-inline-citation-style.md#333-one-edge-only) grants on one line, not taken back the moment the same note needs two.

The first citation-bearing line is judged unconditionally because it is the line that opens the note: that is what keeps `// note. §<ID>`, and a summary line followed by `// prose then §<ID>`, deviations. A continuation line that *opens* with a citation token is judged too, because it is indistinguishable from a note opening — `// §<ID> and continues` reads as a malformed opener whether or not the line above it ended mid-sentence.

#### 3.3.2 Only sites that carry a note

An inline citation site whose note presence is false ([§FS-inline-citation-style.2.3.3](FS-inline-citation-style.md#233-note-presence)) is exempt: a layout is a relation between a citation and a note, and a pure citation comment has none. Both spellings of a chain ([§FS-inline-citation-style.1.4](FS-inline-citation-style.md#14-notes)) qualify, the comma-joined one included — it is the very run this layout mandates in front of a colon, and a project that adopts the layout must not be told its noteless pointers are now malformed for lacking one.

The consequence is deliberate: a `// §<ID>` line followed by a prose-only line **in the same block** is one inline citation site *with* a note, so the citation line is judged and fails. A bulleted pointer inside such a block is the same fact wearing a list marker: skipping the marker ([§FS-inline-citation-style.3.3.8](FS-inline-citation-style.md#338-reading-a-lines-content)) lets `// - §<ID>` open with its run, and the run still has to reach the delimiter, so a bulleted line that names an ID and says nothing is a deviation wherever the block says something elsewhere.

#### 3.3.3 One edge only

The rule constrains what *opens* the line. Citations later on the line are free, so a note may name a second spec point in passing (`// §<ID>: note (see also §<other>)`) and still conform.

#### 3.3.4 Exact

Inside the content, whitespace and punctuation deviations are deviations. A space instead of `, ` between two citations, a comma with no space, a space before the colon, a missing colon, a citation written last inside the prose, and a dash used where the colon belongs all fail. A citation run followed by a colon and nothing else conforms — the colon may end the line. The `W` that separates the colon from the note is one or more space characters, never a tab. Exactness governs the separators inside the run and the delimiter that ends it; the indentation before the content is not part of the content ([§FS-inline-citation-style.3.3.8](FS-inline-citation-style.md#338-reading-a-lines-content)).

#### 3.3.5 Recognized tokens only

"Citation token" means exactly what the scanner already recognizes on that line ([§FS-check.1.1](FS-check.md#11-recognized-citations)). Under `strict = false` a bare `// FS-x: note` line is claimed by the *declaration* recognizer before it reaches this rule ([AR-scanner.2.1](../architecture/AR-scanner.md#21-declaration-detection)) — an inline declaration heading is not an inline citation site at all ([§FS-inline-citation-style.1.3](FS-inline-citation-style.md#13-what-is-not-a-site)) — which is precisely the ambiguity the canonical form removes: with the marker written, `// <§>FS-x: note` reads as a citation carrying a rationale and can never be mistaken for a declaration of the same ID.

#### 3.3.6 Same scope as the rest of this spec

Markdown bodies have no inline citation sites and are untouched ([§FS-inline-citation-style.1.3](FS-inline-citation-style.md#13-what-is-not-a-site)), and a comment trailing code on the same line (`foo(); // §<ID>: note`) is not an inline citation site today and does not become one here. A doc-comment block is not an inline citation site at all ([§FS-inline-citation-style.1.1](FS-inline-citation-style.md#11-doc-comments-are-not-sites)), so no line of one is judged here. A doc-comment whose first line is a declaration heading is a declaration and not an inline citation site ([§FS-inline-citation-style.1.3](FS-inline-citation-style.md#13-what-is-not-a-site)), so the `§<ID>` lines in its body are outside this rule exactly as they are outside the budgets — in a repository that declares IDs inline, that is often the densest citation block in a file, and the layout governs comments that *cite* rather than spec text that happens to live in code. [§FS-inline-citation-style.5](FS-inline-citation-style.md#5-agent-facing-rendering) says the same thing to the agent.

#### 3.3.7 The budgets still apply

Layout and size are judged independently; a line may deviate from the layout, exceed the column cap, or both, and each is its own finding.

#### 3.3.8 Reading a line's content

The form is read on the line's content **after** the comment prefix (`//`, `///`, `//!`, `#`, `;`, `--`, ` * `, `/**`, a docstring quote, …) and any block closer (`*/`, a closing docstring quote) have been stripped — the same prefix stripping [§FS-inline-citation-style.2.3.3](FS-inline-citation-style.md#233-note-presence) applies to decide note presence. Whatever indents the content past the prefix is stripped with it: a wrapped list continuation (`//   §<ID>: …`), an aligned ` *   ` filler, a tab after `#`. Indentation is comment formatting, not layout, so the citation run is read from the first byte of content that says anything.

A leading Markdown **list marker** is skipped the same way: `-`, `*`, or `+`, or an ordered `1.` / `1)`, followed by at least one space. A bulleted block of grounded points is a common shape in a plain comment too, so a bullet is item structure rather than note text, and `// - §<ID>: note` opens with its citation run. The skip is for this rule only: [§FS-inline-citation-style.2.3.3](FS-inline-citation-style.md#233-note-presence) still reads the marker as note text when it decides whether an inline citation site carries a note at all, so no bulleted pointer is silently reclassified and `inline_style = "citation-only"` judges one exactly as before. One marker is skipped, not a chain of them, and only where a space follows — `// -§<ID>: note` opens with a `-`.

#### 3.3.9 Examples

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

#### 3.3.10 Why `any` is the default

The default is `any` because a layout is a house style, not a correctness property: two projects may reasonably disagree, and a tree that adopts `grund` mid-life should not be told its comments are wrong on the day it upgrades ([§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path)). Choosing a value, and why the enforcement level is a second key, is decided in [§DF-inline-note-layout](../decisions/functional/DF-inline-note-layout.md#df-inline-note-layout-inline-note-layout-is-a-configured-house-style-checked-per-line-and-never-normalized).

## 4. Enforcement (`grund check`)

Findings are reported using the located-finding shape of [§FS-errors.2.1](FS-errors.md#21-located-finding), anchored at the **first line** of the offending inline citation site (so a multi-line block with a budget violation lands one diagnostic at its opener, not at every constituent line). The one exception is the layout rule of [§FS-inline-citation-style.4.4](FS-inline-citation-style.md#44-warnings-and-errors--opt-in-layout-deviations), which judges a single line and therefore anchors at it ([§FS-inline-citation-style.4.4.1](FS-inline-citation-style.md#441-one-finding-per-nonconforming-line)). The rule is a pure transformation of `Findings` ([AR-checker.4](../../crates/grund-core/src/checker/report.rs)) — the checker does **not** re-read files. The scanner annotates each recorded citation with its enclosing inline citation site's span, max-column width, note presence, and — when a layout and a check level ask for them ([§FS-inline-citation-style.4.4](FS-inline-citation-style.md#44-warnings-and-errors--opt-in-layout-deviations)) — that block's lines that fail the configured layout ([§FS-inline-citation-style.3.3](FS-inline-citation-style.md#33-inline_note_layout--where-the-citations-sit); [§FS-inline-citation-style.7.1](FS-inline-citation-style.md#71-scanner)), so the rule, the per-line anchor of [§FS-inline-citation-style.4.4](FS-inline-citation-style.md#44-warnings-and-errors--opt-in-layout-deviations) included, operates from `Findings` alone.

### 4.1 Errors — hard caps

Each of the following is an error and contributes to a non-zero exit code, per [§FS-check.2](FS-check.md#2-outputs):

| condition                                               | result                                                                                                             |
|---------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------|
| `inline_style = "citation-only"` and a note is present  | error: `inline citation must carry no prose`                                                                        |
| `lines > inline_note_max_lines`                         | error: `inline note is M lines, over the N-line maximum: lines A-B cite <citations>; a blank line splits a note, an empty comment line does not` |
| `max(columns) > inline_note_max_columns`                | error: `inline note is M columns, over the N-column maximum: line(s) A[-B] cite(s) <citations>`                     |

A single inline citation site that violates more than one cap produces one finding per violated cap (so the author sees every reason in a single pass). [§FS-inline-citation-style.4.1.1](FS-inline-citation-style.md#411-the-measured-size)–[§FS-inline-citation-style.4.1.3](FS-inline-citation-style.md#413-the-splitting-clause) define `M` and `N`, the site clause, and the splitting clause.

#### 4.1.1 The measured size

`M` is the measured size — physical lines ([§FS-inline-citation-style.2.3.1](FS-inline-citation-style.md#231-lines)), or characters ([§FS-inline-citation-style.2.3.2](FS-inline-citation-style.md#232-columns)) of the inline citation site's longest line — placed next to the cap `N` so the finding is actionable without re-measuring, in keeping with [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible). `M` pluralises by its own value (`1 line`, `2 lines`, `1 column`, `2 columns`); `N-line` and `N-column` are adjectival and never pluralise.

#### 4.1.2 The site clause

Both cap findings name the inline citation site: `A-B` is the block's `first_line`–`last_line`, or `line A` alone when the two coincide (only possible for the column cap); `<citations>` is every citation token of the block as written — marker, qualifier, section — in source order, deduplicated after the first occurrence, chain-spelled with `, ` the way [§FS-inline-citation-style.3.3](FS-inline-citation-style.md#33-inline_note_layout--where-the-citations-sit) already joins a citation run.

#### 4.1.3 The splitting clause

The line-count finding carries a further clause, `a blank line splits a note, an empty comment line does not`, restating [§FS-inline-citation-style.1.2](FS-inline-citation-style.md#12-comment-blocks)'s block rule at the point it fixes the finding: the citations name what has to move, and the clause names the boundary that moving them has to cross. The column cap carries no such clause — a wide line is fixed by wrapping, not by splitting.

### 4.2 Warnings — opt-in soft cap

`warn_on_suggested = false` (default): soft-cap overruns are **silent** at `check` time. The soft cap is purely guidance for the agent-facing surface ([§FS-inline-citation-style.5](FS-inline-citation-style.md#5-agent-facing-rendering)); humans get the same guidance through the same rendered copy.

`warn_on_suggested = true`: an inline citation site whose line count exceeds `inline_note_suggested_lines` but stays within `inline_note_max_lines` is reported as a **warning**: `inline note is M lines, over the N-line preferred limit: lines A-B cite <citations>; a blank line splits a note, an empty comment line does not`, with `M`, `N`, the site clause, and the splitting clause following the same rules as [§FS-inline-citation-style.4.1](FS-inline-citation-style.md#41-errors--hard-caps)'s line-count finding ([§FS-inline-citation-style.4.1.1](FS-inline-citation-style.md#411-the-measured-size)–[§FS-inline-citation-style.4.1.3](FS-inline-citation-style.md#413-the-splitting-clause)). Warnings never affect the exit code, per [§FS-check.4](FS-check.md#4-warnings).

The soft cap counts lines only: there is no `suggested_columns` knob, and column width is a single hard cap ([§FS-inline-citation-style.6.3](FS-inline-citation-style.md#63-no-second-column-measure)).

### 4.3 `grund fmt`

`grund fmt` does **not** auto-fix style violations under this spec — budgets and layout alike; an inline citation that violates `inline_style` rules is `check`'s problem, not `fmt`'s. Prose cannot be safely rewritten or truncated, and moving a citation across the prose that surrounds it is a prose edit, not a token rewrite: the formatter would have to decide where a sentence ends, whether a trailing `(§<ID>)` was parenthetical, and what punctuation the remainder now needs. The fix for a layout deviation is one token in the author's own editing loop, and migrating a tree is served by `inline_note_layout_check = "warn"` ([§FS-inline-citation-style.4.4](FS-inline-citation-style.md#44-warnings-and-errors--opt-in-layout-deviations)), which produces the worklist without touching a byte. The formatter's trigger-to-marker and bare-to-marker rewrites ([§FS-fmt.2.1](FS-fmt.md#21-trigger-to-marker), [§FS-fmt.2.2](FS-fmt.md#22-bare-to-marker-with---marker)) and its cross-reference emission ([§FS-fmt.6](FS-fmt.md#6-cross-reference-emission)) continue unchanged.

### 4.4 Warnings and errors — opt-in layout deviations

Off by default, twice over: `inline_note_layout = "any"` means there is no layout to deviate from, and `inline_note_layout_check = "off"` means a layout that *is* configured stays documentation. A project that sets only `inline_note_layout` has told its agents the house style through [§FS-inline-citation-style.5](FS-inline-citation-style.md#5-agent-facing-rendering) and asked `check` for nothing — the same standing the soft cap has under `warn_on_suggested = false`. At `off` no line is classified at all: the verdicts the scanner would record ([§FS-inline-citation-style.4](FS-inline-citation-style.md#4-enforcement-grund-check), [§FS-inline-citation-style.7](FS-inline-citation-style.md#7-architecture-impact)) have no consumer, so a project that documents a layout without gating it scans exactly as fast as one that has none.

With `inline_note_layout = "citation-first-colon"`:

| `inline_note_layout_check` | result                                                                     |
|----------------------------|----------------------------------------------------------------------------|
| `off` (default)            | silent; the layout is agent-facing guidance only ([§FS-inline-citation-style.5](FS-inline-citation-style.md#5-agent-facing-rendering))                       |
| `warn`                     | one **warning** per nonconforming line; the exit code is untouched ([§FS-inline-citation-style.4.2](FS-inline-citation-style.md#42-warnings--opt-in-soft-cap))   |
| `error`                    | one **error** per nonconforming line; the exit code becomes 1               |

Three properties are fixed at both levels ([§FS-inline-citation-style.4.4.1](FS-inline-citation-style.md#441-one-finding-per-nonconforming-line)–[§FS-inline-citation-style.4.4.3](FS-inline-citation-style.md#443-report-order)); [§FS-inline-citation-style.4.4.4](FS-inline-citation-style.md#444-why-two-levels) says why there are two.

#### 4.4.1 One finding per nonconforming line

Each finding is anchored at its nonconforming line, not at the inline citation site's opener. A layout deviation is a property of the line the author has to edit, and a five-line comment with two bad lines is two edits. This is the exception [§FS-inline-citation-style.4](FS-inline-citation-style.md#4-enforcement-grund-check) names: the budgets measure the inline citation site as a whole and keep their opener anchor.

#### 4.4.2 One message at both levels

The message is the same at `warn` and at `error`, so moving a project from `warn` to `error` changes the exit code and nothing a reader has to re-learn. It names the canonical shape with the configured marker, e.g. ``inline note must open with its citations and a colon (§<ID>: note)``.

#### 4.4.3 Report order

Report order is the existing deterministic order ([§FS-errors.4](FS-errors.md#4-determinism)) — the level chooses the channel, and in a text report the channel chooses the group, errors ahead of warnings; the JSON order ignores the channel.

#### 4.4.4 Why two levels

The two levels exist so a repository can adopt the style in the order adoption actually happens: turn on `warn`, migrate the tree with the report as the worklist, then turn on `error` to keep it migrated. That is the same ladder [§DF-require-grounding.2.4](../decisions/functional/DF-require-grounding.md#24-off-by-default) describes for the grounding floor. Choosing which channel a rule speaks through is a per-project configuration choice, not a redefinition of what a warning or an error *means* — those stay fixed by [§FS-check.2](FS-check.md#2-outputs).

## 5. Agent-facing rendering

The `init` machinery that writes versioned managed blocks into `AGENTS.md` / `CLAUDE.md` / sibling agent entrypoints ([§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)) reads the active values and emits the sentences describing the project's house style: the budgets line ([§FS-inline-citation-style.5.1](FS-inline-citation-style.md#51-the-budgets-line)), the block sentence ([§FS-inline-citation-style.5.2](FS-inline-citation-style.md#52-the-block-sentence)), the layout sentence ([§FS-inline-citation-style.5.3](FS-inline-citation-style.md#53-the-layout-sentence)) and the doc-comment sentence ([§FS-inline-citation-style.5.4](FS-inline-citation-style.md#54-the-doc-comment-sentence)), each under the conditions its section states. They are rendered, not live ([§FS-inline-citation-style.5.5](FS-inline-citation-style.md#55-rendered-not-live)); the last three move no managed-block version ([§FS-inline-citation-style.5.6](FS-inline-citation-style.md#56-no-managed-block-version)); and `grund config show` is the machine-readable form of the same keys ([§FS-inline-citation-style.5.7](FS-inline-citation-style.md#57-grund-config-show-is-the-machine-readable-form)).

### 5.1 The budgets line

The copy opens with one budgets line:

- `inline_style = "citation-only"` → `Inline citations carry no prose — put rationale in the spec.`
- `inline_style = "citation-with-note"`, `suggested_lines == max_lines` → e.g. `Inline notes: ≤ 1 line, ≤ 100 columns.`
- `inline_style = "citation-with-note"`, `suggested_lines < max_lines` → e.g. `Inline notes: ≤ 1 line preferred, hard cap 3 lines; ≤ 100 columns.`

The collapse rule is "if soft and hard are the same number, only mention the number" — the soft/hard distinction is a property of the *config*, not always a useful distinction in the agent prose.

### 5.2 The block sentence

Under `citation-with-note` only, one further sentence follows the budgets line and precedes the layout and doc-comment sentences ([§FS-inline-citation-style.5.3](FS-inline-citation-style.md#53-the-layout-sentence), [§FS-inline-citation-style.5.4](FS-inline-citation-style.md#54-the-doc-comment-sentence)): `A note is one comment block: a blank line splits it, an empty comment line does not.` — restating [§FS-inline-citation-style.1.2](FS-inline-citation-style.md#12-comment-blocks)'s block rule where the agent will need it to act on a cap finding. No sentence is added under `citation-only`: an inline citation site with no note has no block to split.

### 5.3 The layout sentence

Under `citation-with-note` only, when `inline_note_layout = "citation-first-colon"` is set, one further sentence follows the block sentence ([§FS-inline-citation-style.5.2](FS-inline-citation-style.md#52-the-block-sentence)), naming the canonical form with the configured marker and placeholder IDs — e.g. ``Lay each note out citation-first: `// §<ID>: <note>` (several citations: `// §<ID>, §<ID>: <note>`).`` No sentence is added under `citation-only`, where the layout is inert ([§FS-inline-citation-style.2](FS-inline-citation-style.md#2-configuration)). Under `inline_note_layout = "any"` nothing is appended and the rendered text is byte-identical to what a `grund` without this key produced, so no repository's managed block drifts on upgrade: its re-render ([§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)) is unchanged, and `grund init --check` reports no `would-update` for it ([§FS-init.1](FS-init.md#1-inputs)).

#### 5.3.1 Wider than the gate

The sentence names a house style for the comments an agent writes, and it is deliberately wider than the gate: `check` judges inline citation *sites*, so neither a doc comment ([§FS-inline-citation-style.1.1](FS-inline-citation-style.md#11-doc-comments-are-not-sites)) nor a doc-comment that declares an ID ([§FS-inline-citation-style.3.3.6](FS-inline-citation-style.md#336-same-scope-as-the-rest-of-this-spec)) is measured against the form at all. Both readings are the intended ones — an agent should lay out every note it writes the same way, and documentation, whether it is a declaration body or the Javadoc next to it, is text whose shape this spec does not govern. The practical consequence belongs to whoever migrates a tree: under `warn`, the worklist covers the citing inline comments and never the doc comments, so "the report is empty" means the inline citation sites are clean, not that every `§<ID>` line in the repository is citation-first.

#### 5.3.2 The same at every check level

`inline_note_layout_check` does **not** change the sentence. The house style is what the agent is asked to write; whether `check` reports a deviation as a warning, as an error, or not at all is a fact about the project's gate, not about the form. An agent told the form and then told it is only advisory would have been given a reason to ignore it.

### 5.4 The doc-comment sentence

One last sentence closes the rendered copy at **every** `inline_style`, after whatever the keys above produced, because where the gate stops is part of the house style an agent needs:

``Doc-comments (`///`, `//!`, `/** */`, a docstring, a Go, Ruby, shell or SQL comment right above a definition) are documentation, not notes: they are never measured, so cite in-sentence there.``

Without it the author and the linter disagree in the expensive direction: the agent reads a budget, sees a Javadoc that cites, and moves the citation to a detached `//` line above the block — out of the generated documentation and away from the sentence it supported — to satisfy a rule that never applied ([§FS-inline-citation-style.1.1](FS-inline-citation-style.md#11-doc-comments-are-not-sites)).

### 5.5 Rendered, not live

Like every other line [§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints) substitutes into the block, the sentences of [§FS-inline-citation-style.5.1](FS-inline-citation-style.md#51-the-budgets-line)–[§FS-inline-citation-style.5.4](FS-inline-citation-style.md#54-the-doc-comment-sentence) are *rendered*, not live: they are written into the managed block when `grund init` runs, and `check` version-checks that block ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)) without comparing them against the active config. A project that adopts `inline_note_layout`, changes its value, or drops back to `any` therefore re-runs `grund init` to refresh the block; until it does, the entrypoint keeps teaching the previous style and `check` says nothing about the mismatch. The citation-directions and clickable-citations sections do not share that standing: `check` re-renders them and byte-compares ([§FS-init.2.3.5](FS-init.md#235-citation-directions), [§FS-init.2.3.6](FS-init.md#236-clickable-citations)).

### 5.6 No managed-block version

Because they are rendered, not live ([§FS-inline-citation-style.5.5](FS-inline-citation-style.md#55-rendered-not-live)), the block, layout and doc-comment sentences move **no** managed-block version: a version bump would turn a silent staleness into an error for every repository, including the ones that never set the layout key. For the block and doc-comment sentences staleness is also cheap: a block that predates the block sentence teaches the same rule less precisely, and one that predates the doc-comment sentence teaches a narrower rule than the gate enforces, since that sentence only ever widens what an author may write. Either costs an over-careful comment, never a finding, and the block gains the sentence on the repository's next `grund init`. A correction that *narrows* the doc-comment sentence — as spelling out the position languages of [§FS-inline-citation-style.1.1.2](FS-inline-citation-style.md#112-languages) did — is the one staleness that can cost a finding rather than a comment, and it moves no version either, because the wider sentence was already wrong against [§FS-inline-citation-style.1.1](FS-inline-citation-style.md#11-doc-comments-are-not-sites) on the day it was rendered, so the correction only ever reduces the findings a repository sees.

### 5.7 `grund config show` is the machine-readable form

`grund config show` ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)) is the canonical machine-readable form: every key is printed at every value, no collapse ([§FS-inline-citation-style.5.1](FS-inline-citation-style.md#51-the-budgets-line)), so a human or downstream tool diffing config sees the raw shape.

## 6. Non-goals

Deliberately out of scope, in four groups ([§FS-inline-citation-style.6.1](FS-inline-citation-style.md#61-no-rewrite-in-grund-fmt)–[§FS-inline-citation-style.6.4](FS-inline-citation-style.md#64-no-per-project-reshaping-of-the-rule)).

### 6.1 No rewrite in `grund fmt`

No auto-rewrite of a note and no normalization of layout, in `--check` or in `--write` ([§FS-inline-citation-style.4.3](FS-inline-citation-style.md#43-grund-fmt)): prose changes need human judgment, and layout is check-only.

### 6.2 No scope beyond inline citation sites

- No scope growth. Spec text in Markdown bodies is not capped, and layout is judged on inline citation sites only — never in Markdown bodies, never on a comment trailing code, never on the code line below the comment ([§FS-inline-citation-style.3.3.6](FS-inline-citation-style.md#336-same-scope-as-the-rest-of-this-spec)).
- No host-language parsing to find definitions. Whether a block is a doc comment is one marker test or one next-line test ([§FS-inline-citation-style.1.1.1](FS-inline-citation-style.md#111-recognizers)). A miss in a position language is never fixed by acquiring a parser ([§FS-non-goals.3](FS-non-goals.md#3-code-ast-parsing)): a false negative, such as a Ruby `private def`, is fixed by widening a starter set, and a false positive, such as a Go `var` in a function body, is accepted ([§FS-inline-citation-style.1.1.3](FS-inline-citation-style.md#113-accepted-corners)).

### 6.3 No second column measure

- No `suggested_columns` knob. Column width is a single hard cap, a binary "too long" in symmetry with how editors and formatters already treat line length rather than a layered preference; in most repositories editor or formatter rules already govern it.
- No display-width awareness. Tabs count as one column; widening tabstops in an editor does not change whether a comment passes the cap.

### 6.4 No per-project reshaping of the rule

- No per-kind or per-file overrides. The style is repo-wide, matching [§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree) — two correctly-configured `grund` installs must agree on whether a tree is well-formed.
- No "warning for hard-cap miss." A hard-cap miss is always an error; if a project wants the soft tier to nag, it sets `warn_on_suggested = true`.
- No per-rule severity remap. `inline_note_layout_check` selects which channel *this* rule speaks through, from a fixed set; it does not let a project re-level any other rule, and it does not change what an error or a warning means ([§FS-non-goals.9](FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization), [§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)).
- No configuration of the language table. The recognizers and the extensions they claim are built in ([§FS-inline-citation-style.1.1.1](FS-inline-citation-style.md#111-recognizers), [§FS-inline-citation-style.1.1.2](FS-inline-citation-style.md#112-languages)): what an inline citation site *is* must not differ between two installs ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)). The table is widenable later without a `grund_config_version` bump ([§FS-config.5](FS-config.md#5-schema-versioning)).

## 7. Architecture impact

This rule is additive on top of the existing scanner + checker pipeline ([§FS-inline-citation-style.7.1](FS-inline-citation-style.md#71-scanner)–[§FS-inline-citation-style.7.4](FS-inline-citation-style.md#74-other-commands)), and it never grows past the comment block: a shape that lies outside what the scanner already records — e.g. "the next code line after the comment" — is **not** part of the inline citation site.

### 7.1 Scanner

In the scanner ([AR-scanner](../architecture/AR-scanner.md#ar-scanner-how-grund-discovers-declarations-and-citations)), each recorded `Citation` gains its enclosing inline citation site's information: `(first_line, last_line, max_columns, has_note)`, plus the ascending list of that block's lines that fail the configured layout ([§FS-inline-citation-style.3.3](FS-inline-citation-style.md#33-inline_note_layout--where-the-citations-sit)). The scanner already knows the comment-block extent on every line (it normalizes `/// …`, ` * …`, docstring interiors for declaration detection in [AR-scanner.4](../architecture/AR-scanner.md#4-inline-declarations-in-language-doc-comments)) — the addition is recording that extent on the citations the block contains, not new line-classification logic. Multiple citations in the same block carry the same span. A doc-comment block records no inline citation site at all, exactly as a declaring block records none.

### 7.2 What the scanner pays

The layout list is computed only when a layout is configured *and* `inline_note_layout_check` is not `off`, so both the default and a documented-only layout cost one comparison per inline citation site, allocate no per-block memo, and tokenize no line and classify no line on the field's account ([§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible)). The doc-or-inline classification of [§FS-inline-citation-style.1.1](FS-inline-citation-style.md#11-doc-comments-are-not-sites) sits in `comment_block.rs` alongside the block classifiers declaration detection already shares: the rule is chosen once per file from its extension, and the block is tested once — one comparison, made only for a block that carries a citation, which is where the scanner already has the block in hand.

### 7.3 Checker

The checker ([AR-checker](../../crates/grund-core/src/checker/report.rs)) gains one new rule under [AR-checker.2](../../crates/grund-core/src/checker/report.rs) — a pure pass over `findings.citations`, grouping by inline citation site, comparing line/column counts and note-presence against the `[reference] inline_*` settings, emitting located findings per [§FS-inline-citation-style.4.1](FS-inline-citation-style.md#41-errors--hard-caps) (and [§FS-inline-citation-style.4.2](FS-inline-citation-style.md#42-warnings--opt-in-soft-cap) when `warn_on_suggested = true`, [§FS-inline-citation-style.4.4](FS-inline-citation-style.md#44-warnings-and-errors--opt-in-layout-deviations) when `inline_note_layout_check` is not `off`). No file I/O: the per-line layout verdicts arrive on the inline citation site the scanner recorded, so the checker never re-reads a line to decide its shape.

### 7.4 Other commands

**`grund fmt`**, **`grund refs`**, **`grund cover`**, **`grund show`**: unaffected. The added fields are inert for every command except `check`.
