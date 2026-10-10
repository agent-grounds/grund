# FS-check: grund validates every citation in a repo

The `check` command scans a repo and reports every violation of the grund citation scheme. Validation is explicit as `grund check [<path>]`; the bare `grund <ID>` default belongs to [§FS-show.1](FS-show.md#1-inputs). Serves [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration) and [§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, home, citable, body, section, coordinate,
lead, index, catalog), [§FS-terms.terms.2](FS-terms.md#terms2-citations) (marker, citation, qualified citation, shorthand,
canonical form, citation site), [§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (source declaration, stub, doc-comment, note),
[§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope, config root, workspace, member, alias), [§FS-terms.terms.5](FS-terms.md#terms5-findings)
(finding, severity, suggestion, caution, verdict, anchor), [§FS-terms.terms.6](FS-terms.md#terms6-rules-and-directions) (direction, level,
rule, grounded), and [§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations) (value, component, binding, snapshot).

- **grounding unit** — What `require_grounding` measures: a whole scanned file, or one
  doc-comment block where the row's `grounding_level` says so. A unit is grounded when it holds
  at least one recognized citation.
- **obligation** — A `must` or `should` entry of `[citations.<kind>]`, asking whether each unit
  of the citing kind cites the target kind at least once. It is answered per unit.
- **prohibition** — A `must-not` or `should-not` entry, which fires once per offending citation
  site rather than once per unit.
- **blind spot** — A place the default scope never reads, bounded and declared rather than
  discovered, and what `--full` exists to look into.
- **full-tree scope** — The scope `--full` selects: every file the scan can reach, in place of
  the configured default scope.

## 1. Inputs

- Optional path argument; defaults to the current directory. May be a directory or a single file (`grund check crates/grund-core/src/scanner/file_pass.rs` scopes the scan to one file but still discovers the `grund.toml` by walking up — [§FS-config.1](FS-config.md#1-file-location-and-discovery)).
- The scanned tree may contain markdown (`.md`) and source files (Rust, Go, Java, TS, Python, etc.).
- Optional `grund.toml` configuring the run per [§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable) ([§FS-config](FS-config.md#fs-config-grund-reads-a-toml-config-file-found-by-walking-up)).
- Optional `[workspace]` config; when present and `check` is run at the workspace root, `check` validates alias-qualified cross-project citations per [§FS-workspace](FS-workspace.md#fs-workspace-grund-validates-cross-project-citations-in-a-workspace).
- `--watch` is reserved for the planned resident checker ([§FS-check.6](FS-check.md#6-watch-mode---watch)) and is not accepted by the current CLI.
- `--require-grounding` — turn the grounding check ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)) on for this run regardless of `[reference] require_grounding` in `grund.toml` ([§FS-config.3.1](FS-config.md#31-reference--citation-form)). It only ever *adds* the check; it cannot switch off a config that already sets it. The flag and the key are **one knob**: the flag sets the same global default, so a `[[kinds]]` row that says `require_grounding = false` is still exempt under it, and `grounding_level` has no flag at all ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)). A run-level flag that overrode the row would make the flag mean something the key cannot say.
- `--suggestions` — emit the suggestions channel ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)) for this run. The flag never adds an error or changes the exit code: it only surfaces the advisory records [§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in) lists, which the default run withholds.
- `--rule "<sentence>"` — add one ad-hoc chapter rule to the configured rules
  for this run. It never disables them; validation, deduplication, and exits are
  [§FS-rules.4](FS-rules.md#4-validation-lifecycle) and
  [§FS-rules.8](FS-rules.md#8-command-surfaces)'s.
- `--only <code>` — retain only findings whose exact code is in the selected set, and every `invalid-rule` when that set names a code a rule produces ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)).
- `--ignore <code>` — remove findings whose exact code is in the selected set ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)).
- `--only-rule` — retain only findings the `--rule` sentence authored, by their `authority` rather than by their code. It requires `--rule`; given alone it is an invocation error with exit `2` ([§FS-rules.8](FS-rules.md#8-command-surfaces), [§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)).
- `--full` — scan the whole config root past `[scan] include`, reporting unresolved citations as out-of-scope findings ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full), [§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only)) plus the scanner-invariant `section-outside-declaration` error ([§FS-declarations.checks.section-outside-declaration](FS-declarations.md#checkssection-outside-declaration-section-outside-a-declaration)). It only ever *adds* findings; the in-scope report is unchanged.
- `--format text|json` — output shape, per [§FS-errors.5](FS-errors.md#5-json-format). The global flags `--version` and `--help` are handled before any scan ([§FS-cli](FS-cli.md#fs-cli-grunds-command-line-surface-conventions)).

### 1.1 Recognized citations

Per [§DF-reference-marker](../decisions/functional/DF-reference-marker.md#df-reference-marker-use--as-the-reference-marker-with--as-the-typing-trigger), a citation is the marker followed by an ID, e.g. `§FS-check.3.1`. The default marker is `§`; configurable via `grund.toml`.

In default mode (`[reference] strict = true`), only marker-prefixed citations are recognized — bare tokens are treated as plain text and do not trigger dangling citation errors. Repositories that still rely on bare citations may set `[reference] strict = false` as a compatibility mode after checking the migration surface with `grund fmt --marker` ([§FS-fmt](FS-fmt.md#fs-fmt-grund-normalizes-citations-in-bulk)).

Citations may appear in markdown prose, in source-file line/block comments, and in language doc-comments (Javadoc, JSDoc, Rustdoc, Python docstrings, etc.) — see [AR-scanner.2.3](../architecture/AR-scanner.md#23-citation-detection) and [AR-scanner.4](../architecture/AR-scanner.md#4-inline-declarations-in-language-doc-comments) for the exact contexts. `E2E` citations (`§E2E-<name>`) resolve against case directories under the configured `E2E` kind home per [AR-scanner.6](../architecture/AR-scanner.md#6-e2e-case-declarations).

#### 1.1.1 Off-grammar citations

An off-grammar citation that a catalog declaration backs ([§FS-config.3.2](FS-config.md#32-id--id-grammar)) participates in section and dangling checks, inbound counts, grounding, citation directions, `refs`, formatting, and editor navigation exactly like a conforming citation. The declaration's mismatch is reported at its heading ([§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss)), not at every citation.

#### 1.1.2 Named-section candidates

When `[id] named_sections = true`, the scanner consumes a whole ID-and-dot-tail candidate before deciding what it means ([§FS-config.3.2](FS-config.md#32-id--id-grammar)). A marker-prefixed legal named or mixed coordinate is a citation, including when the named section is missing. An unmarked candidate with a letter-bearing tail is one prose token and is suppressed whole even under `strict = false`; it never falls back to a bare-ID citation. A reserved `number.name` candidate is likewise never truncated to its numeric prefix. Full IDs claim candidates before number-only shorthand ([§FS-check.1.2.3](FS-check.md#123-the-full-id-always-wins)), and shorthand form and section existence stay independent facts ([§FS-check.1.2.4](FS-check.md#124-a-resolved-shorthand-is-a-real-edge)), exactly as for numeric coordinates.

#### 1.1.3 String literals in source files

In source files, a **bare** ID-shaped token whose start column falls inside a string literal is not treated as a citation (the same deterministic quote-tracking rule `grund fmt` uses — [§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule), [AR-scanner.2.3](../architecture/AR-scanner.md#23-citation-detection)), so an ID-shaped substring inside runtime data does not raise a false dangling citation. A marker-prefixed **unqualified** citation is recognized everywhere, string or not, except inside the assigned Python data span bounded by [§FS-check.1.1.3.1](FS-check.md#1131-assigned-python-triple-quoted-data) — the marker remains the signal of intent in every other string context. Markdown files have no string literals and neither source-file carve-out applies there. Decided in [§DF-python-assigned-triple-quoted-data](../decisions/functional/DF-python-assigned-triple-quoted-data.md#df-python-assigned-triple-quoted-data-module-level-assigned-triple-quoted-strings-are-data-not-docstrings).

##### 1.1.3.1 Assigned Python triple-quoted data

In a `.py` file scanned with `[scan] docstring_python = true`, a simple unindented assignment whose right-hand side begins with a triple-quoted string opens an **assigned-data span**, not a docstring. Its left-hand side is one Python identifier, optionally followed by a same-line annotation and then `=`; whitespace may surround the annotation separator and `=`. The string may use `'''` or `"""` and may carry, case-insensitively, no prefix or one of `r`, `u`, `b`, `f`, `t`, `br`/`rb`, `fr`/`rf`, or `tr`/`rt`. The prefix and delimiter must be the first non-whitespace text after `=`. Parenthesized assignments, destructuring targets, computed targets, indented assignments, and a string reached only after another expression are outside this rule: this is a bounded lexical distinction, not Python scope or AST analysis ([§FS-non-goals.3](FS-non-goals.md#3-code-ast-parsing)).

The span begins at the prefix when present, otherwise at the opening delimiter, and ends immediately after the next matching delimiter that is not escaped by an odd-length run of backslashes. It may open and close on one line or cross lines; quote-like text using the other delimiter and an escaped matching delimiter do not close it. No citation form, declaration, section heading, value tag or binding, or note is recognized inside the span. Source columns remain columns in the raw file. Text before the span retains ordinary source treatment, text after a same-line close resumes ordinary source treatment at its raw column, and the next line after a multiline close begins in neutral state. A later delimiter at the first non-whitespace column therefore opens the same real module, function, or method docstring it would have opened had the assignment not existed.

With `docstring_python = false`, no assigned-data state is introduced and every line retains the existing raw-line source-string behavior. A leading triple-quote with no qualifying assignment remains the existing docstring form when the gate is enabled. Ordinary one-line strings, other Python triple-quoted contexts, other source languages, Markdown, strict-mode bare-token handling, and qualified-citation suppression retain their existing rules.

#### 1.1.4 Markdown link destinations

In a Markdown file, the parallel carve-out is the link destination: a **bare** ID-shaped token whose start column falls inside an inline link's `(…)` half — `[text](…)` — is likewise not treated as a citation, because `grund fmt` never rewrites a link destination ([§FS-fmt.2.3](FS-fmt.md#23-what-is-never-rewritten)) and a finding whose only named fix the tool refuses to perform is one a repository can never clear ([§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)). The exclusion is total, not just a withheld error: such a token is not a citation for `refs`, for unused-declaration counting ([§FS-check.4.1](FS-check.md#41-unused-declaration)), or for grounding ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)) either. A marker-prefixed citation inside a link destination is unaffected — the marker is the signal of intent there too, exactly as it is inside a source-file string literal.

#### 1.1.5 Contexts read as neither prose nor code

Two contexts are read as neither prose nor code, so nothing inside them is a citation. A **fenced code block** in Markdown is skipped entirely: this is what makes an example ID safe to write in documentation without the `<§>` escape, and it is why the illustrations throughout these specs resolve to nothing. A fence opens with at most three leading spaces followed by a run of at least three backticks or tildes; it closes only on a run of the **same character** at least as long as the opener, again with at most three leading spaces and only whitespace after the run. A backtick opener cannot carry a backtick in its info string. An unclosed fence runs to end of file. In **source files only**, the qualified form `§<alias>/<ID>` is additionally skipped inside an inline-code span or a string literal, because `alias/ID` is shaped like a path, module reference, or URL ([AR-scanner.2.3](../architecture/AR-scanner.md#23-citation-detection)); the unqualified form stays live there, and neither skip applies in Markdown. Every other skip is a property of the *scan* rather than of the text — a file the scan skips, for any of the reasons [§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded) lists, is never read at all ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)).

##### 1.1.5.1 A raw HTML block holds no heading

A third Markdown context is read as raw text for structure but as prose for citations: a CommonMark **raw-text HTML block** (CommonMark's HTML block of type 1). It opens on a line whose text, after at most three leading spaces, begins with `<pre`, `<script`, `<style` or `<textarea`, case-insensitively, followed by whitespace, `>` or the end of the line. It closes after the first line, the opening line included, that contains the matching end tag `</pre>`, `</script>`, `</style>` or `</textarea>`, again case-insensitively. An unclosed block runs to end of file. No line of the block, its opening and closing lines included, is a heading: it declares no ID, opens no section, ends no lead and bounds no unit, however much it looks like `# <ID>: …` or `### 3.2 …`. CommonMark renders these lines as preformatted text and never as headings, so a heading recognized there would be a declaration no reader of the rendered page can see.

Citations inside the block stay live, unlike a fence's: the diagrams of [§FS-fmt.2.5.4](FS-fmt.md#254-the-case-this-exists-for) are written in `<pre>` precisely because a citation there still resolves, still counts as a use, and is still reported when it dangles. The two contexts do not nest. Inside a fence an HTML opener is fence content and opens nothing; inside the block a fence opener is block content and opens nothing. Every surface that asks whether a Markdown line is a heading reads the block by this one rule, as it reads the fence by the rule above. Other CommonMark HTML blocks, including comments, keep the existing reading.

#### 1.1.6 Value bindings

An exact explicit value binding additionally records its authored component, but its marker-prefixed token remains one citation under every rule above. The binding grammar and its narrower recognized source-comment contexts are [§FS-values.3](FS-values.md#3-explicit-value-bindings); `[reference] strict = false` never removes the binding's marker requirement.

#### 1.1.7 Fetched snapshots

A snapshot written by [§FS-fetch](FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot) is not a special input. Its configured file or folder home is in the ordinary scan scope ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)); its declaration, body, sections, and marked body citations are recognized by the rules above. `check` never invokes its configured integration.

#### 1.1.8 Declaration-local numeric section candidates

A configured marker followed immediately by one or more decimal components separated by literal
dots is a declaration-local section candidate. Recognition is marker-gated under both strict and
non-strict scanning; an unmarked number remains prose. The path separator here is always `.`,
independent of the configured separator between a full ID and its section. Full-ID citations and
the number-only ID shorthand of [§FS-check.1.2](FS-check.md#12-the-number-only-shorthand) claim their tokens first. Escapes and every scanner
exclusion in [§FS-check.1.1.5](FS-check.md#115-contexts-read-as-neither-prose-nor-code) retain their precedence.

The candidate must end as a whole token. If digits are followed by a tail that would otherwise
make the numeric prefix partial, such as `<§>2.goals`, `<§>2abc`, or a path with an empty
component such as `<§>2..1`, the scanner retains the complete digit-starting token for the
unsupported-syntax verdict in [§FS-check.3.24](FS-check.md#324-declaration-local-section-citation); it never emits an edge to
section `2`. One dot after an otherwise complete token is sentence punctuation and is not part of
the candidate; a repeated terminal dot run is retained as unsupported syntax rather than reduced
to that punctuation case. Named and mixed declaration-local shorthand are not recognized.

After declaration bodies are assigned, the candidate is owned only by the existing enclosing-body
rule ([AR-scanner.2.4](../architecture/AR-scanner.md#24-citing-side-classification)): Markdown
same-or-higher headings, source comment or docstring ends, and the nearest preceding declaration
inside a multi-declaration comment are boundaries. Adjacent citations and other files do not
supply an owner. A uniquely owned candidate becomes an ordinary citation edge to the owner's ID
and numeric path while preserving its local token text. An ownerless or genuinely ambiguous site
records no target. [§DF-declaration-local-section-shorthand](../decisions/functional/DF-declaration-local-section-shorthand.md#df-declaration-local-section-shorthand-local-numeric-section-citations-are-recognized-but-never-canonical)
settles why recognition is loud but the persisted form is never canonical.

#### 1.1.9 An ID in an escape position

A **bare** ID-shaped token that begins immediately after the literal escape — the configured marker wrapped in `<` and `>` — is not treated as a citation, in Markdown and in source files alike, and under both strict modes. [§FS-check.2.3.1](FS-check.md#231-escaped-citation-resolves) already says that the escaped form is inert because the marker is not immediately followed by the ID, so *no pass* treats it as a citation; the bare-token pass is a pass, and it is the escape rather than the mode that suppresses it. The exclusion is **total**, not just a withheld error: such a token is not a citation for `refs`, for unused-declaration counting ([§FS-check.4.1](FS-check.md#41-unused-declaration)), or for grounding ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)) either — the same reach as [§FS-check.1.1.4](FS-check.md#114-markdown-link-destinations) and for the same reason, because `grund fmt` never rewrites an escape position ([§FS-fmt.2.3](FS-fmt.md#23-what-is-never-rewritten)) and a finding whose only named fix the tool refuses to perform is one a repository can never clear. Both escape forms are covered by this one rule, so the qualified `<§>alias/ID` is exempt here and not only by [§FS-workspace.1.3](FS-workspace.md#13-the-shape-outside-a-workspace)'s unmarked-`alias/ID` rule. The escape is spelled with the **configured** marker: where the marker is `@`, `<@>ID` is the escape and a `<§>`-wrapped token is an ordinary bare one. The `escaped-citation-resolves` suggestion of [§FS-check.2.3.1](FS-check.md#231-escaped-citation-resolves) is unaffected, in both modes, and remains the only report of such a site. Recorded in [§DF-escape-position-is-not-a-citation](../decisions/functional/DF-escape-position-is-not-a-citation.md#df-escape-position-is-not-a-citation-an-escape-position-is-not-a-citation-in-either-strict-mode).

#### 1.1.10 The configured marker establishes the citation start

The configured marker immediately followed by a full ID establishes a citation start even
when the marker ends in a word character, such as `_`. No word boundary is required between
that marker and the ID. With `[reference] marker = "_"` and `[id] format = "{kind}_{slug}"`,
`_FS_login` is exactly one marked citation of `FS_login`, under either strict mode. Its record
retains the authored token, source line, and column at the marker; `refs`, `cover`, inbound
counts, grounding, and dangling checking consume that same edge. A missing target therefore
receives the ordinary located dangling finding rather than disappearing.

Only the explicit marker establishes this start: under `strict = false`, an unmarked ID still
requires its ordinary token boundary and cannot be discovered inside a longer word. Default
markers, qualified citations, section suffixes, and full-ID precedence retain their existing
meaning. Escapes and every context exclusion above apply before recognizing a marked token;
the marker does not make a fenced example or assigned Python data into a citation.

The compatibility impact is recorded in
[§DF-word-character-citation-markers](../decisions/functional/DF-word-character-citation-markers.md#df-word-character-citation-markers-recognize-accepted-word-character-markers).

### 1.2 The number-only shorthand

When a kind's effective format carries **both** `{number}` and `{slug}` ([§FS-config.3.2](FS-config.md#32-id--id-grammar)) — the default `{kind}-{number}-{slug}` that `grund init` writes — the number alone already identifies a declaration within its kind, so `§FS-042` is an abbreviation of `§FS-042-user-login` rather than a different ID. `check` **recognizes** and resolves that shape independently of the project's persisted-form policy. Under the default `[reference] shorthand = "canonical"` it reports a unique shorthand to be rewritten; under `"accepted"` the same resolved edge may persist ([§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)). It is never silently ignored, which is what [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration) means by "false negatives are bugs".

Three rules bound the recognition ([§FS-check.1.2.2](FS-check.md#122-the-marker-is-required), [§FS-check.1.2.3](FS-check.md#123-the-full-id-always-wins), [§FS-check.1.2.4](FS-check.md#124-a-resolved-shorthand-is-a-real-edge)), decided in [§DF-number-only-citation-shorthand](../decisions/functional/DF-number-only-citation-shorthand.md#df-number-only-citation-shorthand-the-number-only-shorthand-is-authoring-sugar-and-a-persisted-one-is-a-check-error).

#### 1.2.1 The shorthand shape

The shorthand shape is the kind's effective format with the `{slug}` placeholder and one adjacent literal separator removed. A kind whose effective format has no `{number}` (`{kind}-{slug}`, the form `grund` itself uses) or no `{slug}` (`{kind}-{number}`) has no shorthand and is untouched by this clause ([§FS-id.4.1](FS-id.md#41-number-less-id-formats)).

#### 1.2.2 The marker is required

A bare `FS-042` is plain text even under `strict = false`, where a bare *full* ID would count ([§FS-check.1.1](FS-check.md#11-recognized-citations)). `KIND-NNN` occurs constantly in the wild as issue keys, part numbers, and standards references, and unlike a full ID it carries no slug to make an accidental match unlikely — so the marker is what supplies the intent ([§DF-number-only-citation-shorthand.2.4](../decisions/functional/DF-number-only-citation-shorthand.md#24-the-marker-is-required-a-bare-shorthand-is-text)).

#### 1.2.3 The full ID always wins

The token must also end where the shorthand does. The full-ID pass claims its tokens first; the shorthand pass only sees what is left, and it claims a token only when the character after the match cannot continue an ID — an alphanumeric, `_`, or a literal from `format` that itself has a component after it. Without that trailing boundary the shorthand is a *prefix* of every longer ID-shaped token, so a full ID whose slug the grammar rejects (`§FS-042-User-Login`, `§FS-042_user_login`) would be read as `§FS-042` with a tail hanging off it. Such a token is not a citation at all: it is reported by nothing here and rewritten by nothing in [§FS-fmt.2.4](FS-fmt.md#24-shorthand-to-canonical). The separator has to be *followed* by a component to count, or a citation ending a sentence would be lost in any repo whose `format` uses `.` as a literal. And `/` never counts: it can only precede a kind, so `§FS-042/x` is a citation of `FS-042` exactly as `§FS-042-user-login/x` is one of the full ID — the shorthand and the canonical form must never disagree about the same boundary.

#### 1.2.4 A resolved shorthand is a real edge

When a shorthand matches exactly one declaration, the citation participates in the graph like any other: [§FS-refs](FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id) lists it, [§FS-cover](FS-cover.md#fs-cover-grund-groups-citations-by-scanned-file) groups it, the declaration stops being reported as unused ([§FS-check.4.1](FS-check.md#41-unused-declaration)), it grounds its file under `require_grounding` ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)), and it counts for citation directions ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)). The `.<section>` suffix works as it does on any citation, which means the section check ([§FS-check.3.2](FS-check.md#32-missing-section)) applies to it independently and on its own terms: a shorthand carrying a section that does not exist earns *both* findings, because the canonical form [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation) names is the right ID and still the wrong section.

#### 1.2.5 A shorthand in a numeric run

A recognized shorthand is not always *being used* as a citation: one glued to a second number — `§SPEC-001→SPEC-003` — is a numeral in a run, and while it resolves and counts like any other edge, `grund fmt` will not rewrite it and [§FS-check.3.15](FS-check.md#315-shorthand-citation-in-a-numeric-run) reports it instead of [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation) ([§FS-fmt.2.4.1](FS-fmt.md#241-a-shorthand-in-a-numeric-run-is-not-rewritten)).

#### 1.2.6 The shorthand as a CLI ID argument

The same shape is accepted as a **CLI ID argument** — `grund FS-042`, `grund FS-042.1`, `grund refs FS-042` — where nothing is persisted and the caller gets the declaration ([§FS-show.1](FS-show.md#1-inputs), [§FS-refs.1](FS-refs.md#1-inputs)). That is also what makes a clicked `§FS-042` open in a terminal or editor, since those clients hand the token straight to `grund` ([§FS-integrations.3.1](FS-integrations.md#31-terminal-clients-wezterm-kitty-tmux-iterm2)).

### 1.3 The full-tree scope (`--full`)

`[scan] include` and every scanned kind home ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)) are the roots the scan starts from, so a citation in a file outside them is not merely unchecked — it is invisible: it does not resolve, does not dangle, and appears in no report, so dangling IDs accumulate there unnoticed. A clean `check` then means "clean *within* the scope" but reads as "clean" — a false negative in the one command the workflow trusts, invisible by construction, because the citations that most need checking are the ones somebody forgot to bring into scope. [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration) calls that class a bug, so it must be *seen* without first editing the config to guess where to look; [§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded) accepts the bounded blind spot only because this flag exists to look into it.

`grund check --full` is that way. It cancels `[scan] include` for the scan, and with it the `scan = false` prune of an unscanned kind home ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned)) — and nothing else. Decided in [§DF-check-full-scope](../decisions/functional/DF-check-full-scope.md#df-check-full-scope-check---full-walks-past-scan-include-and-reports-unresolved-references-plus-orphaned-section-headings-out-there).

#### 1.3.1 The scan covers the whole config root

`[scan] exclude`, `.gitignore` and every other ignore file, hidden names — directory and file alike ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)) — workspace member boundaries ([§FS-workspace.6](FS-workspace.md#6-nested-project-boundary)), the E2E case-directory boundary wherever a citable `E2E` kind is configured ([AR-scanner.6](../architecture/AR-scanner.md#6-e2e-case-declarations)), and `[scan] extensions` all still apply exactly as they do without the flag. A file type `grund` does not scan stays unscanned; widening `extensions` is a config decision, and one flag that widened both would make "what did this run read" unanswerable. A boundary is a *declared* member, not any nested `grund.toml`: a project directory the workspace never declared is ordinary tree to both scans, so `--full` reads it and judges its citations under *this* project's grammar and kinds. That is what the plain scan does with it too, but the flag is what makes it reachable by default — a vendored, generated, or example project belongs in `[scan] exclude`, or in `[workspace] members` if it is one of ours, before a run adds `--full`.

#### 1.3.2 The wider scan reads a superset, each file once

Every root the plain scan starts from — each `[scan] include` entry and every kind home it scans ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)) — is scanned under `--full` too, whether or not `exclude`, an ignore file, or the hidden-directory rule would otherwise prune it: those three rules prune *descendants*, never the directory a scan starts at, so a gitignored, excluded, or hidden root is read by the plain run and must be read here. Without that, `--full` could read *fewer* files than `grund check` and hide a finding instead of adding one. A hidden **file** is the exception, read by neither run even as a root ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)); additivity survives it because the file is missing from both scans, not from one. Overlapping roots — an `include` entry inside another, or inside the config root the flag adds — name one file once; a file read twice would be a declaration duplicated with itself ([§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration)). "Once" is per *file*, not per path: an `include` root that is a symlink to a directory inside the config root, or a case alias of one on a case-insensitive filesystem, reaches its files under a spelling the config-root scan never produces, so a byte-identical compare cannot see the reread. The scan therefore starts at those roots *before* the config root and keeps the first spelling of each file — the one `grund check` prints without the flag. Every in-scope line is the plain run's, character for character; outside citation errors add the `outside [scan] include:` scope prefix, while [§FS-declarations.checks.section-outside-declaration](FS-declarations.md#checkssection-outside-declaration-section-outside-a-declaration)'s scanner invariant keeps its ordinary code and message.

#### 1.3.3 Two scopes, two rule sets

Inside the default scope, the report is the ordinary one. Outside it, [§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only)'s citation-resolution errors and its declaration-local form verdict ([§FS-check.3.24](FS-check.md#324-declaration-local-section-citation)) are reported, alongside [§FS-declarations.checks.section-outside-declaration](FS-declarations.md#checkssection-outside-declaration-section-outside-a-declaration)'s single declaring-side exception. No style, grounding, placement, direction, duplicate or unused rule is applied there, and neither is a verdict about what the formatter rewrote — [§FS-check.3.14.2](FS-check.md#3142-what-is-judged-outside-the-configured-scope) states the test each of those is held to, and the one subtraction it makes from that test. A `--full` that failed on conventions in directories that never opted into them would be run once and never again.

#### 1.3.4 Purely additive

The findings inside the default scope are exactly the ones `grund check` reports on the same tree, so `--full` can only ever turn a green run red, never the reverse. It is the ordinary check plus unresolved citations and section-like headings that violate declaration-body ownership in the wider scan.

#### 1.3.5 The unused-declaration warning is unchanged out there

A declaration inside `include` cited *only* from outside it keeps its `declared but never cited` warning ([§FS-check.4.1](FS-check.md#41-unused-declaration)) under `--full`, and the citation that would have retired it resolves and is reported by nothing. That is what additivity costs, and it is the right side of the trade: counting the wider scan's citations toward [§FS-check.4.1](FS-check.md#41-unused-declaration) would *remove* an in-scope finding, the one direction this flag must never move, and it would make the warning mean something different depending on a flag. The remedy is the one the finding already names — widen `include` so the citing file is governed, and the edge counts everywhere.

#### 1.3.6 An explicit path still narrows

`--full` cancels `include`, never a path the caller typed: `grund check <path> --full` reports exactly `<path>` ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)), which `include` does not bound, and has no out-of-scope findings. A path that *resolves to the config root* is the root scope and does widen — `grund check .`, `grund check ./`, and `grund check <abs-root>` are the bare `grund check --full`, out-of-scope findings included. Any other path leaves the flag nothing to cancel. What the path narrows is the report and not the resolution ([§FS-check.1.3.6.1](FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution)), so the finding set shrinks and gains no tier.

##### 1.3.6.1 A path scope narrows the report, not the resolution

A run given an explicit path reads the declarations and the citations its project's ordinary run reads, and reports findings only for the files at or under the path — or, where a finding spans several sites, for a finding naming one of them ([§FS-check.1.3.6.2](FS-check.md#1362-a-finding-that-spans-several-sites-is-in-scope-at-any-of-them)). Two scopes, and telling them apart is the whole rule. The **resolution scope** is the set of files the run reads so that citations resolve and citation edges count: the enclosing project's ordinary scope — `[scan] include` plus every walked kind home ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned), [§FS-config.3.5.8](FS-config.md#358-every-configured-kind-home-is-scanned)) — **union the path itself**, because a path may lie outside `include` and must still be read. The **report scope** is exactly the path. Every rule runs over the resolution scope, and a diagnostic anchored at a file outside the report scope is dropped before the report is written, unless it also names a site inside it ([§FS-check.1.3.6.2](FS-check.md#1362-a-finding-that-spans-several-sites-is-in-scope-at-any-of-them)). Decided in [§DF-path-scope-resolves-project-wide](../decisions/functional/DF-path-scope-resolves-project-wide.md#df-path-scope-resolves-project-wide-a-path-scoped-check-resolves-against-the-whole-project-and-reports-only-the-path).

So `grund check <path>` and `grund check .` never disagree about a citation they both read: an ID declared anywhere in the project resolves under either, and [§FS-check.3.1](FS-check.md#31-dangling-citation) fires under neither. Three verdicts move, and each is the whole-project run's own answer arriving where it was not asked before:

- A citation whose declaration lives outside the path no longer dangles — and the `ungrounded source file` error derived from that verdict clears with it ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)).
- A declaration cited only from outside the path is cited ([§FS-check.4.1.4](FS-check.md#414-a-citation-anywhere-in-the-resolution-scope-counts)).
- A duplicate declaration whose twin lies outside the path is reported, so a path-scoped run that was silent exits 1 ([§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration)). That is the one direction this rule adds a finding.

Two classes of diagnostic are not about a scanned file and keep exactly the behaviour they had. A run-level finding carries no path — the config findings, the scope cautions, the workspace run warnings — and the report filter never sees one. And the agent-entrypoint check ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)) is a probe over the project root rather than a finding about a scanned file, and already runs when no source file is scanned at all; its finding is exempted by its code — the second class of path-anchored diagnostic the filter lets through, beside the multi-site findings of [§FS-check.1.3.6.2](FS-check.md#1362-a-finding-that-spans-several-sites-is-in-scope-at-any-of-them). It is exempt as *that* finding and not as a file: an ordinary citation, style or declaration finding about `AGENTS.md` is dropped like any other outside the path, because the entrypoint being in `[scan] include` is the ordinary configuration rather than a contrived one, and the report stays the path plus the sites [§FS-check.1.3.6.2](FS-check.md#1362-a-finding-that-spans-several-sites-is-in-scope-at-any-of-them) keeps with it. The scope cautions are computed against the **report scope**, so a path that recognized nothing still earns its empty-scan caution ([§FS-check.2.2](FS-check.md#22-empty-scan)) however much the wider walk read — and the [§FS-check.1.3.6.3](FS-check.md#1363-a-file-the-wider-walk-could-not-read-is-said-out-loud) caution about a file out there does not suppress it ([§FS-check.2.2.3.1](FS-check.md#2231-findings-that-do-not-suppress-it)). The nothing-recognized caution ([§FS-check.4.5](FS-check.md#45-nothing-recognized)) is a different matter: a narrowed run never earns it at all ([§FS-check.4.5.4](FS-check.md#454-asked-only-of-the-whole-project)).

##### 1.3.6.2 A finding that spans several sites is in scope at any of them

A finding that inherently spans several sites ([§FS-check.2.1](FS-check.md#21-report-format)) is one finding about all of them, so the report-scope test is its anchor **or** any site it names. That is the mechanism behind the duplicate above: the message is located at the lexicographically-first site, which may be the twin the caller did not type, and the path-scoped run must still report it.

The diagnostic is kept whole — same anchor, same line, same bytes, under either scope — and is never re-anchored at the in-scope site. Re-anchoring would make `grund check <path>` print a line `grund check .` never prints, which is the disagreement this point exists to end; and the message already names every other site in its text, so a second path in the report surprises nobody.

The rule is general rather than written for one code, and the other two multi-site findings are decided by it rather than left to be found:

- [§FS-values.5.2](FS-values.md#52-fixed-value-errors)'s value mismatch is anchored at the binding and names the declaration as its site, so `grund check <the declaring file>` reports a mismatch it was silent about before. That is the same unkept promise as the duplicate wearing a different code, not a second direction.
- [§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path)'s sites all lie in the anchor's own file ([§FS-declarations.checks.duplicate-section.1](FS-declarations.md#checksduplicate-section1-scoped-to-one-declaration)), so the rule leaves it exactly as it was.

##### 1.3.6.3 A file the wider walk could not read is said out loud

A file the resolution scope could not read is outside the report scope like any other fact about a file out there: it is none of this run's errors and does not reach its exit code, which stays `1` for a report of errors and is [§FS-check.2.4](FS-check.md#24-an-incomplete-run)'s `2` only for a file at or under the path. A run whose *report* is the path is complete about the path.

Silence about it is not allowed, though. The citation whose declaration lay in that file is reported as unresolved, and a false positive must stay legible as one ([§REQ-no-wrong-citation.2](../requirements/REQ-no-wrong-citation.md#2-no-false-alarms)). So each unread file earns one CLI-level caution on **stderr** ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)) carrying the read failure's own reason and saying where the file stands:

```
warning: <path>: <reason> — outside the report scope, so a citation declared there may be reported as unresolved
```

It covers every class of read failure the walk records — a permission error, an undecodable file, a link the scan cannot resolve ([§FS-config.3.5.5](FS-config.md#355-a-link-the-scan-cannot-resolve-is-reported-and-not-scanned-into)) — and like every warning it leaves the exit code alone and stands in place of the `success` line ([§FS-check.2.1.3](FS-check.md#213-the-success-line)).

It carries the `io` code and is unhideable with it: neither `--only` nor `--ignore` can turn it off ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore), [§FS-check.2.1.2](FS-check.md#212-selection-filters-the-complete-report)), and `--only io` selects it beside the [§FS-check.2.4](FS-check.md#24-an-incomplete-run) read failures that do reach an exit code. It is the one `io` finding that is unhideable without exiting `2`, and the consequence is stated rather than regretted: a tree holding one unreadable file in its resolution scope prints that line in place of `success` for every `grund check <path>` in it, with no selector able to restore the line. That is the honest answer — the run genuinely could not read part of what it resolved against, and the whole-tree run over the same tree exits `2`. Minting a code of its own to buy suppressibility would add a row to [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)'s closed selector vocabulary for an advisory line, which is a wider change than this point needs.

For the same reason the chapter rules ([§FS-rules.4.1](FS-rules.md#41-a-rule-this-scope-cannot-verify)) are asked whether the **resolution** scope was read in full, not whether the report scope was: an absence fact must not be asserted over a read the run knows failed, and the narrowed list would make a run that could not read half its tree claim it had.

In a workspace the resolution scope is the enclosing project's and never a sibling member's: `grund check <member>/src/lib.rs` resolves against that member's `include` and homes, reports that one file, and crosses no member boundary ([§FS-workspace.5](FS-workspace.md#5-command-scope)). A cross-project citation keeps today's answer, `unknown project alias` ([§FS-workspace.5.1](FS-workspace.md#51-a-member-run)), which names its own scope and so is not an instance of what this point corrects.

The cost is that a path-scoped run walks what the whole-project run walks. That is what the correct answer costs, and it is the number [§DF-path-scope-resolves-project-wide](../decisions/functional/DF-path-scope-resolves-project-wide.md#df-path-scope-resolves-project-wide-a-path-scoped-check-resolves-against-the-whole-project-and-reports-only-the-path) records.

#### 1.3.7 A path the flag cannot widen earns a caution, not a refusal

When `--full` is passed with an explicit path that is not the config root, the run emits one CLI-level `warning:` ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)) on **stderr** and reports the same findings, on the same streams, with the same exit code as the run without the flag:

```
warning: --full has no effect with an explicit PATH — sim already resolves against [scan] include, and the report is the path either way
```

The flag has nothing to cancel because of the two-scope rule of [§FS-check.1.3.6.1](FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution), and not because a path bypasses `include`: the path already resolves against `include`, so cancelling it subtracts nothing from what the run reads, and the report is the path with the flag or without it.

Silently accepting the flag is the failure this mode exists to end in miniature: the caller asked for the wider search and got the ordinary run, with no signal. Rejecting it would be worse — the run is a valid one, and a script that passes `--full` uniformly would start failing on the invocation where it happens to be redundant. It is a warning like any other: the exit code is untouched, it stands in place of the `success` line on an otherwise clean run ([§FS-check.2.1](FS-check.md#21-report-format)), and under `--format json` it is one finding object on stderr, so a clean run's **stdout** stays empty either way.

#### 1.3.8 Workspaces widen per project

Run at a workspace root, `--full` applies to the root project and to every member ([§FS-workspace.5](FS-workspace.md#5-command-scope)): each scans its own tree past its own `[scan] include` and judges its findings against its own default scope, because `include` is a per-project statement. It widens the projects a run already has and never invents one, so under `[workspace] include_root = false` ([§FS-workspace.2](FS-workspace.md#2-workspace-configuration)) a file at the workspace root outside every member is read by nothing, with or without the flag — there is no root project whose `include` there would be to cancel.

#### 1.3.9 An empty default scope is still reported

A `--full` run whose *default* scope read no files gets the [§FS-check.2.2](FS-check.md#22-empty-scan) caution as well as its out-of-scope findings: the findings say where the citations actually are, the caution says the config has not been told.

#### 1.3.10 A flag, never a key

`--full` is a flag, never a `grund.toml` key. A project that wants its whole tree governed widens `include` and gets the whole rule set; `--full` exists for the tree whose config has drifted from where the code moved, and a config key for it would be a second, weaker `include`.

### 1.4 Selecting findings with `--only` and `--ignore`

Both flags repeat: repeated values form a union, duplicate values are harmless, and each occurrence takes exactly one code — comma-separated values are not split into a selector language. An absent `--only` set retains every code. The two compose: a finding is retained when its code is in `--only` (or no `--only` was given) and is not in `--ignore`, so ignore wins when both sets name the same code.

A code a rule can produce carries `invalid-rule` with it. When `--only` names any of the rule-produced codes [§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits) lists, `invalid-rule` counts as selected too, and every `invalid-rule` finding passes the code axis — not only those of the rule that would have produced the selected code, because a rule that fails to parse has no family to match on. A rule that could not run is otherwise a check the narrowed run never evaluated, and its empty selection would print `success` and exit `0`, the same bytes as a real pass ([§DF-narrowed-check-keeps-invalid-rule](../decisions/functional/DF-narrowed-check-keeps-invalid-rule.md#df-narrowed-check-keeps-invalid-rule-a-check-narrowed-to-a-rules-code-keeps-the-error-that-says-the-rule-could-not-run)). Nothing else about the code axis moves: `--ignore invalid-rule` still wins, as an explicit opt-out, and the authority axis below still applies to the carried rows. A run whose `--only` set names no rule-produced code selects exactly what it names.

Codes are the documented lowercase kebab-case vocabulary in [§FS-errors.5](FS-errors.md#5-json-format), not categories or message fragments. Selecting an opt-in code such as `oversized-lead` is valid even when its config key is absent, but selection never activates the finding ([§FS-declarations.checks.oversized-lead](FS-declarations.md#checksoversized-lead-oversized-lead-opt-in)).

Selection has a second axis, and the two intersect. `--only-rule` ([§FS-rules.8](FS-rules.md#8-command-surfaces)) selects by a finding's `authority` — the rules that produced it ([§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits)) — rather than by its code, because an authority is not a code and no composition of the code vocabulary can name one: a trial sentence emits the same codes the declared rules emit, by construction. A finding is retained when its code passes the code axis *and*, if `--only-rule` was given, its `authority` contains the `--rule` origin. `--ignore` still wins over both, so `--only-rule --ignore <the trial sentence's own code>` legitimately empties a scoped report; adding a second exception to ignore's precedence would make selection unpredictable. The `--only` and `--ignore` value grammar is untouched by this axis: `--ignore RULE-count` remains the invalid-code refusal below, because those flags take codes and a rule identity is not one.

Selector values are validated before config discovery or scanning. Lowercase kebab-case means `[a-z0-9]+(?:-[a-z0-9]+)*`: no uppercase, underscore, comma, empty segment, or leading/trailing hyphen. A missing or empty value is rejected as `error: --only requires a finding code` or `error: --ignore requires a finding code`; a value outside that grammar is rejected as `error: invalid finding code "<value>" (expected lowercase kebab-case)`; and a well-formed value outside the public catalog is rejected as ``error: unknown check finding code "<value>"; run `grund check --help` for supported codes``. These are CLI failures: stdout is empty and the exit is `2` ([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)). Validation is independent of flag spelling: both `--only value` / `--ignore value` and `--only=value` / `--ignore=value` have the same behavior. `--only-rule` is decided in the same place and refused the same way — `error: --only-rule requires --rule` when no trial sentence was given, empty stdout, exit `2` — and being a boolean it takes no value, so it has no `=value` spelling to validate and repeats harmlessly.

## 2. Outputs

A report on **stdout** — `check` is a linter and its findings are its output ([§FS-errors.1](FS-errors.md#1-streams)) — plus an exit code:

- `0` — no retained errors. Retained warnings and suggestions are allowed (they do not affect the exit code).
- `1` — at least one retained error.
- `2` — scan or CLI failure (I/O, malformed file, invalid `grund.toml`, or invalid invocation).

The three value findings are ordinary fixed-severity errors and exit `1`; home JSON input that [§FS-values.5.3](FS-values.md#53-incomplete-input-and-deterministic-output) counts as incomplete leaves the scan incomplete and exits `2`. What `--only` and `--ignore` may select is [§FS-check.2.1.2](FS-check.md#212-selection-filters-the-complete-report), and what an incomplete run prints is [§FS-check.2.4](FS-check.md#24-an-incomplete-run). For verbose text and JSON report examples, including empty JSON scans and global finding ordering, see [§FS-output-shapes](FS-output-shapes.md#fs-output-shapes-machine-readable-output-shapes).

### 2.1 Report format

Findings are written to **stdout**, one per line, in the form:

```
<path>:<line>: error: <message>
<path>:<line>: warning: <message>
<path>:<line>: suggestion: <message>
```

`<path>` is relative to the base `--path-base` selects, else `[output] relative_paths` — the config root under the default ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from), [§FS-config.3.6](FS-config.md#36-output--report-format)). `<line>` is 1-indexed. The `<path>:<line>:` prefix is mandatory on every finding so editors and agents can jump unmodified — this is the contract from [§GOAL-friendliness-first.1](../goals.md#1-hard-requirements).

Every retained located text finding carries its lowercase channel after that jump-friendly prefix: `error:` for [§FS-check.3](FS-check.md#3-errors-detected), `warning:` for [§FS-check.4](FS-check.md#4-warnings), and `suggestion:` for an enabled [§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in) advisory. The channel prefix is report structure rather than part of the finding's message; `<message>` retains its ordinary bytes. Text reports follow the grouped order [§FS-errors.4](FS-errors.md#4-determinism) fixes. Every retained finding remains present and unabridged.

When a finding inherently spans multiple sites (e.g., duplicate declarations, [§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration)), the message is located at the lexicographically-first site (sort by `path`, then `line`) and the other sites are listed parenthetically inside the message.

Selection happens before the fixed per-format sort, render, and exit decision ([§FS-check.2.1.2](FS-check.md#212-selection-filters-the-complete-report)). An otherwise empty selected report makes the default text form write exactly `success` plus a trailing newline ([§FS-check.2.1.3](FS-check.md#213-the-success-line)); with `--format=json`, the retained findings are emitted as NDJSON on stdout instead ([§FS-check.2.1.4](FS-check.md#214-json)).

#### 2.1.1 CLI-level messages

Lines that are about the run rather than a finding at a site in the repo — unknown subcommand, malformed flag, invalid `grund.toml` schema (when the config itself parses but a value is wrong), a per-file read failure mid-scan ([§FS-check.2.4](FS-check.md#24-an-incomplete-run)), the empty-scan caution ([§FS-check.2.2](FS-check.md#22-empty-scan)), the citation-obligation caution ([§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing)), the nothing-recognized caution ([§FS-check.4.5](FS-check.md#45-nothing-recognized)) — are CLI-level messages: on **stderr**, never on stdout, with the shape, prefix and exit rules, and JSON treatment of [§FS-errors.2.2](FS-errors.md#22-cli-level-message). A `grund.toml` schema error is one of the CLI-level messages that still point at a line, beside the absorbed-scan config error of [§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan) and the workspace warning of [§FS-check.4.10](FS-check.md#410-include_root--false-leaves-the-blocks-own-files-unread), in the form and for the reason [§FS-config.4.3](FS-config.md#43-invalid-config-behavior) gives. [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) was in that list until its ramp ended: in `check` it is now a located error on stdout, and the CLI-level line it still prints on the five other scanning surfaces is the one described there.

#### 2.1.2 Selection filters the complete report

`check` always completes the ordinary scan and every checker pass before applying `--only`, `--ignore`, and `--only-rule` ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)): selection is a query over that complete report by exact code **and** by rule authority, not a way to skip declaration, citation, maintenance, warning, or suggestion work. Coded errors, warnings, and enabled suggestions are selected by the same rule on both axes, after `--suggestions`, `--full`, and `--require-grounding` have decided which findings exist, and before the fixed per-format sort, render, and exit decision. Every retained finding keeps its message, location, code, sites, authority, and channel, and selectors cannot alter its relative order within its text severity group or the global JSON order; flag order and duplication cannot affect output ([§FS-errors.4](FS-errors.md#4-determinism)). CLI and config failures happen outside that selectable report, and an `io` finding cannot be hidden by either axis: either kind of incomplete run ([§FS-check.2.4](FS-check.md#24-an-incomplete-run)) remains visible and exits `2` regardless of selectors. Narrowing to a code a rule produces likewise carries the errors of the rules that could not run: the `invalid-rule` rows count as selected beside it ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)), so a narrowed run never reports a pass for a rule it skipped. Unhideable does not by itself mean `2`: the unread-source caution ([§FS-check.1.3.6.3](FS-check.md#1363-a-file-the-wider-walk-could-not-read-is-said-out-loud)) wears the same code and is unselectable with it, and is a warning about a file outside the report scope that leaves the exit code where the report put it. The unselected default run is unchanged, and no third exit status exists.

#### 2.1.3 The `success` line

When there are zero retained errors and zero retained warnings, and no retained suggestion or unselectable run-level line exists, the default text form writes exactly `success` plus a trailing newline to stdout. The line is only emitted for an otherwise empty selected report; a run that has a retained warning or suggestion prints that line instead. It says that the selected report is empty, not that the repository has no findings. There is no summary footer — the exit code is still the machine-readable verdict, and the per-finding lines are the human-readable detail.

#### 2.1.4 JSON

With `--format=json`, the retained findings are emitted as NDJSON on stdout instead — same stream, machine shape per [§FS-errors.5](FS-errors.md#5-json-format). JSON remains byte-, shape-, and order-compatible: its objects keep the global order [§FS-errors.4](FS-errors.md#4-determinism) fixes rather than inheriting text's severity groups. JSON remains findings-only: stdout is empty when the selected report has no retained findings, so `grund check --format=json | jq …` sees only finding objects. CLI-level `error:` / `warning:` lines, when there are any, go to stderr ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)), so a clean JSON run is empty on *both* streams and a `2` always means something on stderr.

### 2.2 Empty scan

A scan that read **no scannable files** at all, and turned up no findings (no errors, no warnings — including the agent-entrypoint check of [§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block), which still runs and still reports even when nothing is scanned), is almost always a misconfigured scope rather than a clean repo. Rather than print `success` and exit `0` — which reads as "all clear" — `check` emits one CLI-level `warning:` line ([§FS-errors.2.2](FS-errors.md#22-cli-level-message)) to **stderr**: it is a caution about the run, not a finding about the repo, so it does not belong on stdout with the findings. Its message names the likely cause ([§FS-check.2.2.2](FS-check.md#222-the-message-names-the-likely-cause)); its exit code, JSON form, and what suppresses it are [§FS-check.2.2.3](FS-check.md#223-exit-code-json-and-what-suppresses-it). This is the friendliness-first counterpart to the explicit success line ([§GOAL-friendliness-first.1](../goals.md#1-hard-requirements)): the run that scanned nothing is one of the cases where `success` would be the wrong answer ([§FS-check.2.1.3](FS-check.md#213-the-success-line)).

#### 2.2.1 Citation-direction obligation applies to nothing

When `[citations.<kind>]` contains at least one `must` or `should` obligation, but the citing kind has no unit for the obligation to evaluate, `check` emits one CLI-level `warning:` on **stderr**. This is a run-level fact, not a finding at a repository site: the warning has no path, line, or sites, and it does not change the exit code or emit `success` in text mode. In `--format=json`, it is one standard warning finding on **stderr** with `path`, `line`, and `sites` all `null` ([§FS-errors.5](FS-errors.md#5-json-format)). Its stable finding code is `empty-citation-obligation`. The warning is independent of other findings: another warning or error does not suppress it.

The warning is emitted once per configured kind when all of the conditions [§FS-check.2.2.1.1](FS-check.md#2211-when-it-fires) lists hold; a workspace asks it per project, and an explicit path of the files it scans ([§FS-check.2.2.1.3](FS-check.md#2213-workspaces-and-explicit-paths)). Its messages distinguish the two kinds of missing unit ([§FS-check.2.2.1.2](FS-check.md#2212-the-messages)).

##### 2.2.1.1 When it fires

The warning fires for a kind when all of these conditions hold:

1. The table has a non-empty `must` or `should` list. A table containing only `must-not` or `should-not` entries has no obligation unit and does not warn. A kind with both levels is named by `must`; a `should`-only table is named by `should`.
2. The citing kind has a `folder` home, and that kind is scanned. File homes, the homeless kind, and `scan = false` kinds do not warn.
3. The run successfully scanned at least one file that belongs to that folder, excluding the folder's entry file. For a citable kind, the entry is its effective `index` (`README.md` when the key is omitted); for a non-citable kind, the entry is the literal `README.md`. `index = false` excludes no file. Files not successfully scanned, files outside the home, and files hidden or excluded by the scan do not count.
4. The ordinary obligation-unit derivation produced zero units: no declaration unit for a citable kind, or no citation-carrying scanned-file unit for a non-citable kind.

The folder membership and entry-file comparisons use the same normalized home matching as citation-source classification.

##### 2.2.1.2 The messages

The messages distinguish the two kinds of missing unit:

```text
warning: [citations.SKILL] must applies to nothing — skills/ declares no SKILL ID; did you mean `citable = false`?
warning: [citations.skill] must applies to nothing — no scanned file in skills/ carries a citation; set require_grounding = true on the skills/ row to make that an error
```

The non-citable message keeps its `require_grounding` half only where that row's **effective** `require_grounding` is off ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)) — it is advice, and where the row already grounds, the setting it asks for is made and this run is already reporting what it caught ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)). There the message stops at the fact:

```text
warning: [citations.skill] must applies to nothing — no scanned file in skills/ carries a citation
```

##### 2.2.1.3 Workspaces and explicit paths

Workspace checking asks the question separately for each project, against that project's config and scanned files ([§FS-workspace.5](FS-workspace.md#5-command-scope)). An explicit path still evaluates the files it scans, so a path such as `grund check skills` can earn this warning; it does not broaden the path to unrelated homes.

#### 2.2.2 The message names the likely cause

The message follows what the run was given: the repo root with `[scan] include` set ([§FS-check.2.2.2.1](FS-check.md#2221-the-repo-root)), an explicit path ([§FS-check.2.2.2.2](FS-check.md#2222-an-explicit-path)), a hidden file handed by name ([§FS-check.2.2.2.3](FS-check.md#2223-a-hidden-file)), or a `[workspace]` block that put no project in scope ([§FS-check.2.2.2.4](FS-check.md#2224-no-project-in-scope)).

##### 2.2.2.1 The repo root

When the scope is the repo root (no path argument, or `grund check .`) and `[scan] include` is set, the message names the `include` list and points at `grund.toml` / `grund init`, since the usual cause is a project whose sources live outside the default `docs/`, `e2e/`, `src/`.

##### 2.2.2.2 An explicit path

When an explicit path was given, the message names that path and the recognized extensions, since the usual cause is pointing `grund` at a tree with no `.md`/source files.

##### 2.2.2.3 A hidden file

When the explicit path is a **file whose own name begins with `.`** and whose extension is one `[scan] extensions` lists, the message names the hidden-name rule instead of [§FS-check.2.2.2.2](FS-check.md#2222-an-explicit-path)'s ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)). The extension list is the one rule that did *not* skip that file, so naming it would send the reader to edit config that was never the cause:

```
warning: nothing to scan — `docs/.notes.md` is a hidden file. grund reads no file whose own name begins with `.`, whatever `[scan] extensions` says. Rename it, or move what needs checking into a file that is not hidden.
```

A hidden file whose extension is *also* unlisted keeps [§FS-check.2.2.2.2](FS-check.md#2222-an-explicit-path)'s message: there the list is a true reason, and naming one of two causes would be its own misdirection. This case answers for a handed **file** only — a handed *directory* whose only listed-extension content is hidden keeps [§FS-check.2.2.2.2](FS-check.md#2222-an-explicit-path)'s message too, though the hidden-name rule is the sole reason it read nothing, because the scan does not record that it met candidates and rejected every one of them by name.

##### 2.2.2.4 No project in scope

When a `[workspace]` block put no project in scope at all — `include_root = false`, and every member it has is an optional one this checkout does not have ([§FS-workspace.2.2](FS-workspace.md#22-a-member-that-may-be-legitimately-absent)) — the message says exactly that. The messages of [§FS-check.2.2.2.1](FS-check.md#2221-the-repo-root) and [§FS-check.2.2.2.2](FS-check.md#2222-an-explicit-path) would both be false here, because the scan never looked under `[scan] include` and the tree `grund init --docs` scaffolds is not what is missing. This one names no remedy either, for the reason [§FS-check.4.9](FS-check.md#49-a-workspace-member-declared-optional-is-absent)'s finding names none: nothing is misconfigured, and the only thing that changes the answer is a fuller checkout.

#### 2.2.3 Exit code, JSON, and what suppresses it

This is a warning, not an error: the exit code stays `0` (a genuinely empty tree is not a failure), and `--format=json` emits the warning as one finding JSON object on stderr, the stream of the text `warning:` line rather than of the findings on stdout. A repo that *does* have a stale `AGENTS.md` block or any other finding **about the default scope** gets that finding (on stdout) and **no** empty-scan notice. Four findings are not about that scope and do not suppress it ([§FS-check.2.2.3.1](FS-check.md#2231-findings-that-do-not-suppress-it)).

##### 2.2.3.1 Findings that do not suppress it

- The redundant-config pair ([§FS-check.4.3](FS-check.md#43-redundant-config-pair)) is about which file the run read rather than what it scanned, so a repository mid-migration keeps the scope finding beside its config pair.
- The out-of-scope finding ([§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only)) is about the tree *outside* the scope, and is the case the caution is worth most ([§FS-check.1.3.9](FS-check.md#139-an-empty-default-scope-is-still-reported)).
- The absent-member finding ([§FS-check.4.9](FS-check.md#49-a-workspace-member-declared-optional-is-absent)) is about the members the run skipped rather than the scope it scanned: the block whose last project went missing is exactly the run that has nothing to read, and it must not lose the line saying so to the line saying why.
- The unread-source caution ([§FS-check.1.3.6.3](FS-check.md#1363-a-file-the-wider-walk-could-not-read-is-said-out-loud)) is about a file the *resolution* scope could not read, which lies outside the report scope the caution is diagnosing. Letting it suppress the caution would cost the run that needs the caution most: `grund check src/typo` in a tree holding one unreadable file anywhere would stop saying that nothing under the path matched and say something about that other file instead.

### 2.3 Suggestions channel *(opt-in)*

The `should` / `should-not` levels of `[citations]` ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)) produce **suggestions** — findings carried on the suggestions channel rather than at a third severity ([§FS-terms.terms.5](FS-terms.md#terms5-findings), [§FS-distribution.3.0.1](FS-distribution.md#301-report-and-finding), [§FS-rules.7](FS-rules.md#7-findings-and-channels)): they are advisory by RFC-2119 definition and grund has no per-site suppression mechanism, so surfacing them in the default run would replace the `success` line ([§FS-check.2.1.3](FS-check.md#213-the-success-line)) on a repo that has consciously accepted a deviation, and it would never recover. They are therefore withheld from the default run and live on a separate channel, decided in [§DF-citation-directions](../decisions/functional/DF-citation-directions.md#df-citation-directions-encode-citation-directions-as-checked-config-with-rfc-2119-levels).

`grund check --suggestions` ([§FS-check.1](FS-check.md#1-inputs)) emits them. A suggestion is a third report channel, **not** a third severity: [§FS-config.6](FS-config.md#6-what-is-not-configured-here) freezes the severity set at `{error, warning}`, so a suggestion carries `"channel": "suggestion"` rather than a `severity` ([§FS-errors.5](FS-errors.md#5-json-format)). The codes are `suggested-citation` (a `should` obligation a declaration does not meet), `discouraged-citation` (a `should-not` citation site), and `escaped-citation-resolves` ([§FS-check.2.3.1](FS-check.md#231-escaped-citation-resolves)).

`--only` and `--ignore` select suggestions only after `--suggestions` has enabled this channel; that selection, the text and JSON rendering, and the exit code suggestions never affect are [§FS-check.2.3.2](FS-check.md#232-selecting-and-printing-suggestions). `grund gap` is the standing home for these records once it ships ([§FS-check.2.3.3](FS-check.md#233-grund-gap-is-their-standing-home)).

Under v2, `should` and `should-not` are the strengths that reach this channel, wherever they are written: a citation rule, a measure, a ladder rung or `heading_depth` ([§FS-config-v2.rules.strengths](FS-config-v2.md#rulesstrengths-one-strength-vocabulary-two-severities)). `warn` and `warn-not` reach the warning channel instead, and `may` reaches none.


#### 2.3.1 Escaped citation resolves

A citation whose marker is bracketed — the schematic `<§>alias/ID` shape — is deliberately inert: the `§` is not immediately followed by the ID, so no pass treats it as a citation ([§FS-workspace.1](FS-workspace.md#1-citation-syntax)). That is how a citation's *shape* is written in prose without `grund check` resolving it. It also makes an escape of an ID that *does* exist ambiguous: usually a deliberate illustration, but also exactly what a live citation looks like once the marker is bracketed by accident, a slip that raises no dangling error ([§FS-check.3.1](FS-check.md#31-dangling-citation)) and navigates nowhere. So when an escaped citation's ID resolves to a real declaration, grund emits an `escaped-citation-resolves` suggestion at the escape site, naming the live `§`-form to switch to. It never replaces `success` or changes the exit code ([§FS-check.2.3.1.1](FS-check.md#2311-never-a-warning-or-error)), and it reads both escape forms ([§FS-check.2.3.1.2](FS-check.md#2312-both-escape-forms)).

##### 2.3.1.1 Never a warning or error

It is a suggestion, never a warning or error: illustrating a real ID is legitimate, so it must never replace `success` ([§FS-check.2.1.3](FS-check.md#213-the-success-line)) or change the exit code. It is the mirror of the [§FS-check.3.1](FS-check.md#31-dangling-citation) dangling check — that flags a live citation whose ID does not resolve; this flags an escaped one whose ID does.

##### 2.3.1.2 Both escape forms

Unqualified `<§>ID` and qualified `<§>alias/ID` escapes are both covered. The ID is parsed with the citing project's grammar, so a cross-project target under an unusual grammar may be skipped, which only ever withholds a suggestion.

#### 2.3.2 Selecting and printing suggestions

`--only` and `--ignore` select suggestions only after `--suggestions` has enabled this channel: a selector cannot surface a suggestion the run did not request. Selecting away every enabled suggestion restores the ordinary empty selected-report behavior from [§FS-check.2.1.3](FS-check.md#213-the-success-line).

- **Text** — `--suggestions` prints each suggestion in the located-finding shape `<path>:<line>: suggestion: <message>` ([§FS-check.2.1](FS-check.md#21-report-format)), after the error and warning groups in the same deterministic within-group order ([§FS-errors.4](FS-errors.md#4-determinism)). Without the flag, suggestions are not printed, and the `success` line still appears for a run with zero errors and zero warnings even if suggestions exist — a suggestion says the graph is thinner than recommended, not that it is malformed.
- **Exit code** — suggestions, retained or not, never affect it (`0`/`1`/`2` unchanged), exactly like the empty-scan caution.
- **JSON** — under `--suggestions`, suggestion objects are emitted on stdout alongside the findings with `"channel": "suggestion"`; a consumer filtering on `severity ∈ {error, warning}` is unaffected. Without the flag none are emitted.

#### 2.3.3 `grund gap` is their standing home

`grund gap` ([§RM-gap-report](../roadmap.md#rm-gap-report-orphan-and-uncovered-id-reports)) is the standing home for these records once it ships — a should-level miss is precisely "the graph is thinner than recommended," and gap is exit-code-neutral by design.

Chapter-rule `should` and `should not` results use this same opt-in channel
([§FS-rules.7](FS-rules.md#7-findings-and-channels)). They are withheld without
`--suggestions`, carry the suggestion channel in text and JSON, and never affect
the exit code. This adds no suppression mechanism and does not change why the
channel is opt-in.

### 2.4 An incomplete run

An invalid `grund.toml` aborts before any file is read ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)): exit `2`, a single `error:` line on stderr, nothing on stdout. A per-file failure *during* the scan — a file that cannot be read or decoded, an unreadable directory, or a link the scan cannot resolve ([§FS-config.3.5.5](FS-config.md#355-a-link-the-scan-cannot-resolve-is-reported-and-not-scanned-into)) — does not abort: the offending path is reported as `error: <path>: <reason>` on stderr, in the CLI-level shape of [§FS-errors.2.2](FS-errors.md#22-cli-level-message) because the path has no line to point at and "I could not read this" is about the run, not a finding about the graph; the scan continues over the remaining files; every finding collected from the readable files is still printed to stdout in the normal located `check` form; and the run exits `2` because the view of the tree was incomplete. A `2` therefore always means "do not trust this report as complete"; the printed findings are still real. Malformed input is answered with a finding naming the path and a truthful code, never an abort ([§REQ-never-crashes](../requirements/REQ-never-crashes.md#req-never-crashes-garbage-in-diagnostic-out)).

## 3. Errors detected

Each of the following is an error and contributes to a non-zero exit code.

### 3.1 Dangling citation

A recognized citation (per [§FS-check.1.1](FS-check.md#11-recognized-citations)) for which no declaration is found: `unknown reference <ID>`. A near ID ([§FS-check.3.1.1](FS-check.md#311-a-near-id)) or an inline-code context ([§FS-check.3.1.2](FS-check.md#312-an-illustration-in-inline-code)) adds a hint, and a fetch-enabled kind adds the fetch action ([§FS-check.3.1.3](FS-check.md#313-a-kind-that-fetches)).

Resolution is computed over the run's **resolution scope**, which for a run given an explicit path is the whole enclosing project rather than the path ([§FS-check.1.3.6.1](FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution)), so a path-scoped run never calls a project-resolvable ID unknown. The message and the code are the same under either scope.

A number-only shorthand citation ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)) is exempt from this rule and reported by [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation) instead — never both, because `unknown reference FS-042` would name a token that is not a full ID under the repo's own grammar. For a value binding, this ordinary resolution finding suppresses value comparison at the same site ([§FS-values.5.1](FS-values.md#51-resolve-before-comparison)).

#### 3.1.1 A near ID

If the target project contains a declared ID of the same kind that is close by deterministic edit distance, the finding appends one hint: `unknown reference FS-chek; did you mean FS-check?`. If no same-kind candidate is close enough, the message stays `unknown reference <ID>` so unrelated missing IDs do not produce noisy guesses.

#### 3.1.2 An illustration in inline code

When the dangling citation sits inside a Markdown inline-code span ([§FS-fmt.2.3.5](FS-fmt.md#235-an-inline-code-span-closes-on-a-run-of-its-own-length)) — where a `§`-citation is as often an illustration as a live one — the finding also offers the `<§>` escape ([§FS-check.2.3.1](FS-check.md#231-escaped-citation-resolves)): `unknown reference api/FS-zzz; write <§>api/FS-zzz if this is an illustration`. The two hints combine when a near-ID match and an inline-code context apply at once: `unknown reference api/FS-login; did you mean api/FS-logout? (or write <§>api/FS-login if this is an illustration)`. Outside inline code the escape hint is withheld, so an ordinary prose typo is nudged toward the near ID, not toward escaping.

#### 3.1.3 A kind that fetches

This is also the fixed finding for a fetch-enabled kind whose effective target-side resolution is `must` ([§FS-config.3.4.10](FS-config.md#3410-format-resolve-and-fetch--external-snapshot-kinds)). In that case the exact message is `unknown reference <qualified-ID>; no snapshot in <home> — run grund fetch <qualified-ID>`, its JSON code remains `dangling`, its severity is `error`, and it contributes exit 1. For a kind without `fetch`, the historical `unknown reference <qualified-ID>` bytes remain unchanged.

The near-ID and escaped-inline-code hints ([§FS-check.3.1.1](FS-check.md#311-a-near-id), [§FS-check.3.1.2](FS-check.md#312-an-illustration-in-inline-code)) take precedence over the fetch action. The message retains `unknown reference <qualified-ID>; no snapshot in <home>` and substitutes the existing conditional tail for the em-dash fetch tail: `; did you mean <candidate>?`, `; write <§><qualified-ID> if this is an illustration`, or their existing combined form. One citation site still produces one finding.

### 3.2 Missing section

A citation with a section suffix (`§FS-<user-login>.3.1` or, in an opted-in repository, `§FS-<user-login>.goals`) where the declaration exists but the requested section heading does not. A missing marker-prefixed named coordinate is never shortened to its declaration; it produces the ordinary `section not found` error and adds `write <§> before it to show the shape without citing it` to the message. A number-only shorthand carrying a missing named section produces both the existing shorthand finding and this finding: the persisted ID form and the requested target are independent facts ([AR-checker.2.12](../../crates/grund-core/src/checker/report.rs)).

An owned declaration-local numeric candidate ([§FS-check.1.1.8](FS-check.md#118-declaration-local-numeric-section-candidates)) follows the same rule. Its
[§FS-check.3.24](FS-check.md#324-declaration-local-section-citation) form
finding and this missing-section finding are independent, so a missing local path reports both.

For a value binding the explicit numeric component must resolve here before comparison; a missing component produces this finding alone, not a mismatch ([§FS-values.5.1](FS-values.md#51-resolve-before-comparison)).

#### 3.2.1 A stub's sections are its target's, scanned or not

A citation of a stub's ID names a section of the declaration the stub pairs with
([§FS-declarations.checks.duplicate.1](FS-declarations.md#checksduplicate1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned)), so the section exists exactly where that declaration
records a heading at the requested path, read from the stub's target whether or not
`[scan] include` reaches it. The stub declares no sections of its own
([§FS-declarations.checks.duplicate-section.2](FS-declarations.md#checksduplicate-section2-scoped-to-that-declarations-body)), so its own record never answers; the target's
declaration is the one `grund <ID>.<path>` slices ([§FS-show.2.3.7](FS-show.md#237-a-stubs-target-is-found-by-its-id)), and `check` and `show`
answer from that one section set ([§FS-show.2.2.2.2](FS-show.md#2222-the-headings-check-counts)). A section citation reads as it would were
the target scanned: stubs to one target are one home and so one section set
([§FS-declarations.checks.duplicate.2](FS-declarations.md#checksduplicate2-stubs-to-one-target-stand-for-its-declarations-once)), and a `--full` run, whose report inside the default scope
is the plain run's ([§FS-check.1.3.4](FS-check.md#134-purely-additive)), resolves it as the plain run does.

That answer is one fact, and every reader of it in `check` takes it: this finding, the `cites`
fact a rule counts a resolved citation by ([§FS-rules.5.1](FS-rules.md#51-facts-and-identity)), the value a binding is compared with once
its component resolves ([§FS-values.5.1](FS-values.md#51-resolve-before-comparison)), the value authority that makes a malformed binding an invalid
attempt rather than prose ([§FS-values.3.1.1](FS-values.md#311-invalid-attempts-and-non-attempts)) and that `fmt --cross-refs` protects from a rewrite
([§FS-values.8](FS-values.md#8-formatting-stability)), the gate that admits a bare index entry `fmt` can wrap ([§FS-check.3.17.4](FS-check.md#3174-only-a-citation-fmt-would-wrap-reaches-this-rule)), and the
declaration-local rewrite that declines an absent section ([§FS-fmt.2.4.6](FS-fmt.md#246-an-absent-target-section-withholds-this-rewrite-and-only-this-one)) with the command clause
it withholds there ([§FS-check.3.24.1](FS-check.md#3241-the-release-attribution-and-where-the-command-clause-is-withheld)).

Nothing else moves. A section the target does not declare is reported as before, in the same
words — `missing section <ID>.<path>` for a numeric path — and under the same `missing-section`
code. A broken stub
([§FS-declarations.checks.broken-stub](FS-declarations.md#checksbroken-stub-broken-inline-spec-stub)) pairs with nothing and gains no section: it keeps its
broken-stub finding, and a citation of one of its sections is `missing section` as before. A
target that declares the ID more than once pairs as `show` refuses it, ambiguously
([§FS-show.2.3.7](FS-show.md#237-a-stubs-target-is-found-by-its-id)), and lends no section, and so does every target of an ID with more than one home
([§FS-show.2.2.1](FS-show.md#221-ambiguous-id)). The target is read only to answer a citation into it, so what `check`
judges of the home itself stays as it was outside the scan: its declaration, the citations it
makes, and its chapters as a rule's subjects or in a rule's count of chapters
([§FS-rules.2.1](FS-rules.md#21-a-chapters-name-is-its-whole-path), [§FS-rules.3.1](FS-rules.md#31-chapter-presence)).
Decided in [§DF-stub-sections-from-unscanned-target](../decisions/functional/DF-stub-sections-from-unscanned-target.md#df-stub-sections-from-unscanned-target-a-stubs-sections-are-its-targets-whether-or-not-the-scan-reaches-it).

### 3.3 Duplicate declaration

Moved to [§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration). This address is kept so citations written before the move still resolve.

### 3.4 Broken inline-spec stub

Moved to [§FS-declarations.checks.broken-stub](FS-declarations.md#checksbroken-stub-broken-inline-spec-stub). This address is kept so citations written before the move still resolve.

### 3.5 Invalid agent entrypoint init block

If `<path>/AGENTS.md` exists, `check` verifies the versioned `grund init` block defined by [§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints); it also verifies the companion agent entrypoints [§FS-check.3.5.1](FS-check.md#351-which-entrypoints-are-verified) names. A missing managed block when one is required, an older block version, a newer unsupported block version, or a config-derived section that no longer byte-matches its re-render from the live config ([§FS-init.2.3.5](FS-init.md#235-citation-directions), [§FS-init.2.3.6](FS-init.md#236-clickable-citations)) is an error in scaffolded-entrypoint mode; a legacy block is an older version and broken delimiters are a distinct error ([§FS-check.3.5.2](FS-check.md#352-legacy-and-malformed-blocks)). This lets CI catch repos whose managed agent entry points were initialized and later drifted or need to be refreshed with `grund init`. Every variant is an error with code `agents-init`, with the reporting order, message text, and selector behavior of [§FS-check.3.5.3](FS-check.md#353-code-and-message).

#### 3.5.1 Which entrypoints are verified

`check` verifies known companion agent entrypoints whenever they exist and are not symlinks to `AGENTS.md` — `.rules` only where the owning `.zed/` directory or a managed block already in it says so, the evidence [§FS-init.2.1.1](FS-init.md#211-one-entrypoint-per-agent) attributes that too-generic name by; for example, existing standalone `AGENTS.override.md`, `CLAUDE.md`, `.claude/CLAUDE.md`, `GEMINI.md`, and `.github/copilot-instructions.md` files must carry the managed block as rendered for that file's own agent ([§FS-init.2.3.6](FS-init.md#236-clickable-citations)), while `CLAUDE.md -> AGENTS.md` is already covered by the canonical file. `check` does not require absent agent-directory-triggered companions from [§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place); once `grund init` creates one, it is validated because it exists.

If `AGENTS.md` does not exist, existing companion agent files without a managed block are treated as project-owned instructions and are not validated by `grund check`; this keeps config-only adoption from modifying or policing an existing agent setup. A companion that already contains a managed `grund init` block is still version-checked even without `AGENTS.md`, so repos initialized directly into `CLAUDE.md`, `GEMINI.md`, or another explicit entrypoint still get drift detection.

#### 3.5.2 Legacy and malformed blocks

A legacy H2-bounded block from v3 or earlier ([§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)) is still recognized and reported as an *older block version* — `` run `grund init` `` is its transition path to the delimited form, so existing repositories are told how to migrate rather than treated as malformed. Broken delimiters are a distinct error: a file whose delimiters [§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints) defines as **malformed** is reported as a malformed managed block, located at the offending delimiter line and naming the defect; `check` never rewrites the file, and `grund init` refuses to splice against broken delimiters for the same reason.

#### 3.5.3 Code and message

Every variant remains an error with code `agents-init`, and the default run still reports it after every content pass completes. Its text is the final template of [§FS-errors.3.6.1](FS-errors.md#361-the-final-templates), which classifies the work as repository maintenance and states that citation validity is unaffected; the three-release migration that led to it ended in `0.16.0` ([§FS-errors.3](FS-errors.md#3-message-text)). A selector may retain or remove this ordinary coded error, but never changes the checker pass that produced it.

#### 3.5.4 No config-derived section is exempt from the comparison

The re-render covers every config-derived section the live config produces, and no section is dropped from the comparison because part of the config could not be read. Where a configured chapter rule does not resolve, the `### Chapter rules` section is still rendered — from the valid rules together with the ones unverifiable in this scope ([§FS-rules.4.1](FS-rules.md#41-a-rule-this-scope-cannot-verify)) — and still byte-compared. Only a genuinely invalid rule leaves the section unrenderable, and that run already carries the rule's own located `invalid-rule` error at its heading, so the skip is named where a reader can act on it. A comparison the checker performs in one tree and silently omits in another is the undeclared blind spot [§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded) forbids.

A workspace run compares each project's block against the vocabulary of the workspace it loaded. That is what lets a run at the workspace root catch a member's managed block that is missing a bullet for a rule the root resolves, which is the disagreement [§FS-rules.9.1](FS-rules.md#91-one-tree-renders-one-block) exists to prevent.

### 3.6 Ungrounded unit *(opt-in)*

Off by default. Two config keys decide it, each written in `[reference]` as the default for every `[[kinds]]` row and settable on the row itself ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)): `require_grounding` says **whether** a place's files must be grounded ([§FS-check.3.6.1](FS-check.md#361-which-files-a-row-governs)), and `grounding_level` says **what the unit is** inside each of them ([§FS-check.3.6.2](FS-check.md#362-the-unit)). `grund check --require-grounding` ([§FS-check.1](FS-check.md#1-inputs)) sets the same global default, and an explicit `require_grounding = false` on a row wins over it.

A unit is **grounded** when it contains at least one recognized citation ([§FS-check.1.1](FS-check.md#11-recognized-citations)) whose ID resolves to a declaration — **or**, in a source file outside every non-citable home, when it declares an ID inline (a declaring file is grounded in the declaration it *is*, [AR-scanner.4](../architecture/AR-scanner.md#4-inline-declarations-in-language-doc-comments)). A unit that is neither is an error ([§FS-check.3.6.3](FS-check.md#363-findings)). A unit whose only citation is dangling ([§FS-check.3.1](FS-check.md#31-dangling-citation)) is *not* grounded — it gets both findings; fixing the citation clears both. The rule is a pure function of `(tree, config)`: it reads no git history and parses no code ([§FS-check.3.6.4](FS-check.md#364-a-pure-function-of-the-tree)). Decided in [§DF-require-grounding](../decisions/functional/DF-require-grounding.md#df-require-grounding-an-opt-in-check-that-every-source-file-cites-a-spec).

Under v2 the two keys are one ladder of strength-and-unit rungs, executed in order: each rung reports the units of its size that are ungrounded on its strength's channel, so a `warn` rung's ungrounded unit is a `warning:` with this message and code, and a narrower ladder replaces the wider one whole ([§FS-config-v2.rules.grounding](FS-config-v2.md#rulesgrounding-grounding-is-a-ladder-replaced-whole)). Inline declarations ground a unit as above, and the scanner still collects heading structure only for files a rung finer than `file` governs.


#### 3.6.1 Which files a row governs

Every scanned file resolves to exactly one `[[kinds]]` row, and that row's effective `require_grounding` decides whether the file is checked at all:

- **A non-citable kind's home** ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)) governs **every** scanned file in it, `.md` included — a `folder` home's files, or the one document of a `file` home ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)), which is a place a maintainer declared like any other and takes both keys on its row ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)).
- **A citable kind's folder home** governs the **source files** in it — a file the scan reads whose extension is not `.md` ([AR-scanner.1](../architecture/AR-scanner.md#1-tree-walk)).
- **The homeless kind** ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)) governs the source files no home claims, and a file claimed by two overlapping homes falls to it as well, the way its citing side already does ([AR-scanner.2.4](../architecture/AR-scanner.md#24-citing-side-classification)).

So Markdown is exempt except inside a non-citable home, and `require_grounding = true` on an unscanned home's row is a config error ([§FS-check.3.6.1.1](FS-check.md#3611-markdown-is-governed-only-in-a-non-citable-home)). A repository that sets only the global key keeps every level `1` and the file as the unit ([§FS-check.3.6.1.2](FS-check.md#3612-the-global-key-alone-is-the-rule-it-always-was)).

##### 3.6.1.1 Markdown is governed only in a non-citable home

Markdown is therefore exempt except inside a non-citable home, and that exception is a home rather than an extension. The Markdown exemption reasons about implementation versus document; a non-citable home is neither guess — it is a directory the maintainer declared matters, and it is usually *all* Markdown, a skill, a runbook, a prompt library. Inheriting the exemption there would switch the rule off exactly where it was turned on. An unscanned home ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned)) has no scanned files, so the rule never reaches it — which is why `require_grounding = true` on such a row is a config error.

##### 3.6.1.2 The global key alone is the rule it always was

A repository that sets only `[reference] require_grounding = true` and configures no non-citable kind sees this rule exactly as it did before these keys existed: every row inherits the global `true`, every level is `1`, and the unit is the file.

#### 3.6.2 The unit

`grounding_level` is an integer in Markdown heading levels ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)). Each level **contains the one below it**, so the file itself is always a unit and nothing passes vacuously for lacking structure. In a Markdown file, level `L` adds a unit for every heading subtree of depth `2` to `L` ([§FS-check.3.6.2.1](FS-check.md#3621-in-a-markdown-file)); in a source file, the units below the file are doc-comment blocks, ranked by indentation ([§FS-check.3.6.2.2](FS-check.md#3622-in-a-source-file)); and a source declaration grounds its doc-comment block and its file, except inside a non-citable home ([§FS-check.3.6.2.3](FS-check.md#3623-the-inline-declaration-escape)).

##### 3.6.2.1 In a Markdown file

Level `L` makes a unit of the whole file and of every heading subtree whose depth is between `2` and `L`. A subtree runs from its heading to the line before the next heading at the same or a shallower depth, so a parent is satisfied by any descendant and a leaf must cite directly; text before the first heading belongs to the file rather than to a section. At level `1` there are no section units and the file is the only one, which is the unit every config had before the key existed. A file with no heading between `2` and `L` is one unit — the file — for the same reason.

##### 3.6.2.2 In a source file

There are two ranks, and they are read by indentation rather than by syntax ([§FS-non-goals.3](FS-non-goals.md#3-code-ast-parsing)): at level `2` every **unindented** doc-comment block is a unit — a parse-free stand-in for a top-level item, which holds across Rust, Python, Java, Go, and Kotlin — and at any higher level every doc-comment block is. What counts as a doc-comment is the per-language rule of [§FS-inline-citation-style.1.1](FS-inline-citation-style.md#11-doc-comments-are-not-sites), already read once per file by the scanner. The file is a unit at every level, as in Markdown.

##### 3.6.2.3 The inline-declaration escape

The inline-declaration escape of [§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in) applies per unit: a doc-comment block that declares an ID is grounded by that declaration, and so is the file it sits in. It has no effect inside a non-citable home, where a declaration is a misplaced declaration to begin with ([§FS-declarations.checks.misplaced-declaration](FS-declarations.md#checksmisplaced-declaration-misplaced-declaration-configured-kind-home)) — there the only way to ground a unit is to cite one.

#### 3.6.3 Findings

A finding is located at its unit — line 1 for a file, the heading line for a section, the block's first line for a doc-comment — and names the unit and, when the unit sits in a non-citable home, the home:

```
src/foo.rs:1: ungrounded source file: no § citation to a declared ID
skills/triage/SKILL.md:1: ungrounded file in kind home skills/: no § citation to a declared ID
skills/review/SKILL.md:14: ungrounded section `## Steps` in kind home skills/: no § citation to a declared ID
src/walk.rs:41: ungrounded doc-comment: no § citation to a declared ID
```

The marker in the message is the configured one ([§FS-config.3.1](FS-config.md#31-reference--citation-form)). Section units arise only inside a non-citable home, since that is the only place Markdown is governed, so a section finding always names one. Every failing unit is reported ([§FS-check.3.6.3.1](FS-check.md#3631-every-failing-unit-is-reported)).

##### 3.6.3.1 Every failing unit is reported

A file that cites nothing at level `2` earns the file finding *and* one per section, which is what "each level contains the one below it" means on the reporting side — the file is genuinely ungrounded, and so is each of its sections.

#### 3.6.4 A pure function of the tree

The rule is a pure function of `(tree, config)` like every other `check` rule ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)): it reads no git history ([§FS-non-goals.6](FS-non-goals.md#6-decision-database-audit-log-history-tracking)) and parses no code ([§FS-non-goals.3](FS-non-goals.md#3-code-ast-parsing)) — "source file" is decided by extension, a unit by heading depth or comment indentation, and "grounded" by the citations the scanner already collected. It is the floor of the grounding discipline — the verification-at-rest layer of [§GOAL-agent-grounding.1](../goals.md#1-the-three-layers), on top of which `grund cover` exposes the citation graph ([§FS-cover](FS-cover.md#fs-cover-grund-groups-citations-by-scanned-file)) and [§RM-cochange-gate](../roadmap.md#rm-cochange-gate-an-opt-in-commit-msg--ci-recipe-for-spec-and-test-edits) tracks the diff-aware co-change gate.

### 3.7 Misplaced declaration (configured kind home)

Moved to [§FS-declarations.checks.misplaced-declaration](FS-declarations.md#checksmisplaced-declaration-misplaced-declaration-configured-kind-home). This address is kept so citations written before the move still resolve.

### 3.8 Cross-project citation failure

An alias-qualified citation whose alias path is unknown is reported at the citation site in every run, with or without a `[workspace]` ([§FS-workspace.1](FS-workspace.md#1-citation-syntax)); in a workspace run, so is one whose target declaration or target section is missing. The alias resolution rules live in [§FS-workspace.4](FS-workspace.md#4-resolution).

An unknown alias path names the projects it could have meant, so the fix is in the finding rather than in the config: the outermost workspace root searches every project in scope ([§FS-check.3.8.2](FS-check.md#382-candidates-at-the-outermost-root)); a run narrowed to a subtree searches only for a strict extension of its scope ([§FS-check.3.8.1](FS-check.md#381-a-strict-extension-of-the-narrowed-scope-is-safe-to-hint)) and otherwise names the subtree it covers ([§FS-check.3.8.3](FS-check.md#383-a-narrowed-run-offers-no-candidate)), in the wording [§FS-check.3.8.4](FS-check.md#384-the-scope-only-message-across-three-releases) stages. The citation remains unresolved and the run still fails.

#### 3.8.1 A strict extension of the narrowed scope is safe to hint

A narrowed run may search for a candidate only when its non-empty scope path is a strict, segment-wise prefix of the written alias path: the written path starts with every scope segment and has at least one segment after them. Thus `group/alph` is eligible in scope `group`, and `group/alpha/bet` in scope `group/alpha`. The search sees only the aliases the narrowed run loaded, so every candidate it can name is inside the subtree the run can judge.

An eligible path gets the outermost root's ordered bands, sorting, three-result limit, prose joining, and `; did you mean …?` rendering ([§FS-check.3.8.2](FS-check.md#382-candidates-at-the-outermost-root)); when the loaded aliases yield no candidate, the message is the bare `unknown project alias <path>` form. Outermost-root runs keep their unconditional candidate search.

Shorter and equal paths, paths whose segments do not begin with the scope, and merely lexical prefixes remain ineligible: in scope `group`, that excludes `alpha`, `group`, `outside/alpha`, and `grouped/alpha` alike. Each gets the scope-only message of [§FS-check.3.8.4](FS-check.md#384-the-scope-only-message-across-three-releases) with `<scope>` = `group` — in `0.13.2` through `0.15.0` the staged form with its exact compatibility suffix, from `0.16.0` the final template. Eligibility, candidate selection, resolution, and the failing verdict do not change in any of these releases.

#### 3.8.2 Candidates at the outermost root

At the outermost workspace root — the scope CI runs, and the only one that can see every project a path could name — candidates are taken from the projects in scope in one band only, best first: a project whose slash-separated path has every written segment as a **proper prefix** and at least one further segment, else one whose path **ends with** what was written (a dropped prefix — the mistake full alias paths invite, [§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)), else one whose **last segment** matches (a wrong prefix), else one a **typo** away under the near-match rule of [§FS-check.3.1.1](FS-check.md#311-a-near-id). Thus `group` matches `group/alpha`, and `group/alpha` matches `group/alpha/beta`; an exact `group`, `grouped/alpha`, and `other/group` do not match the proper-prefix band. The first non-empty band alone wins, without candidates from any lower band. Its candidates are deduplicated, sorted by alias bytes, and then limited to three — `grund list` is the catalogue, a finding is not — and a path with no candidate reports on its own, unchanged.

```text
docs/FS-root.md:3: unknown project alias sprayer; did you mean hardware/sprayer?
docs/FS-root.md:4: unknown project alias api; did you mean left/api or right/api?
docs/FS-root.md:5: unknown project alias group; did you mean group/alpha?
```

#### 3.8.3 A narrowed run offers no candidate

A run narrowed to a subtree ([§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)) holds only part of the tree, so a path naming a project outside it is unknown *here* while being exactly right at the workspace root. Except for the visibly in-scope case of [§FS-check.3.8.1](FS-check.md#381-a-strict-extension-of-the-narrowed-scope-is-safe-to-hint), such a run therefore offers **no candidate at all**. It cannot tell a dropped prefix from a path that correctly names a project outside its subtree, and every band reads it as the former: the dropped-prefix band included, because a *shorter* written path is itself a complete alias path whenever a top-level project carries that name, so re-pointing it at a deeper namesake rewrites a citation CI accepts into a different project's — green before and green after, so nothing catches it.

In every ineligible case it names the subtree it covers — as a *subtree*, since that scope's own alias path is one project among the several it holds — rather than reporting the path bare, which is neither "delete this" nor "re-prefix this" but "check this from the root."

#### 3.8.4 The scope-only message across three releases

A narrowed run's scope-only finding reads exactly:

```text
unknown project alias <path>; the <scope> project and its descendants are in scope here — check from the workspace root for a path outside that subtree
```

It says what "the subtree" holds — the named project and its descendants — before it says where to check from instead. From `0.13.2` through `0.15.x` it was a compatibility form that kept the earlier `only the <scope> subtree is in scope here` wording as a contiguous prefix and ended with a suffix naming `0.16.0` as the release its wording would change in; that form retired in `0.16.0` ([§FS-errors.3.3](FS-errors.md#33-the-narrowed-run-unknown-project-wording-migration)).

### 3.9 Section heading level mismatch

Moved to [§FS-declarations.checks.section-heading-level](FS-declarations.md#checkssection-heading-level-section-heading-level-mismatch). This address is kept so citations written before the move still resolve.

### 3.10 Inline citation style violation

A citation site in a code comment that violates the configured inline citation style — `inline_style = "citation-only"` with prose present, an inline note that exceeds `inline_note_max_lines`, or one that exceeds `inline_note_max_columns`. A site is an *inline* comment block, never a doc-comment block, so nothing in this rule reaches a citation written inside a `///`, a `/** … */`, a docstring, or a comment documenting the definition below it ([§FS-inline-citation-style.1.1](FS-inline-citation-style.md#11-doc-comments-are-not-sites)). The full mode and budget contract, and how multi-cap violations split into multiple findings, lives in [§FS-inline-citation-style.4.1](FS-inline-citation-style.md#41-errors--hard-caps); the schema for the controlling keys is in [§FS-config.3.1](FS-config.md#31-reference--citation-form). An opt-in layout check adds one further form ([§FS-check.3.10.1](FS-check.md#3101-layout-deviation)).

#### 3.10.1 Layout deviation

With `[reference] inline_note_layout` set to a layout and `inline_note_layout_check = "error"`, each line that [§FS-inline-citation-style.3.3.1](FS-inline-citation-style.md#331-per-line-not-per-site--and-only-where-a-note-opens) judges, in a citation site that carries a note ([§FS-inline-citation-style.3.3.2](FS-inline-citation-style.md#332-only-sites-that-carry-a-note)), and that does not match the configured form is an error located at that line ([§FS-inline-citation-style.3.3](FS-inline-citation-style.md#33-inline_note_layout--where-the-citations-sit), [§FS-inline-citation-style.4.4](FS-inline-citation-style.md#44-warnings-and-errors--opt-in-layout-deviations)). The same deviation is a warning under `inline_note_layout_check = "warn"` ([§FS-check.4.4](FS-check.md#44-inline-note-layout-deviation-opt-in)) and silent at the default `off`. It is the one member of this rule that anchors per line rather than at the site's first line, because a layout deviation is a property of the line an author has to edit.

### 3.11 Missing required citation

When `[citations]` ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)) sets a `must` obligation for a citing kind, every top-level declaration of that kind must carry at least one citation satisfying each `must` entry, anywhere in its body. A declaration that does not is an error located at the declaration line, naming the unmet target:

```
docs/architecture/AR-router.md:1: AR-router must cite FS or GOAL (citation direction)
```

The body extent and the citing-side classification come from the scanner ([AR-scanner.2.4](../architecture/AR-scanner.md#24-citing-side-classification)); the obligation pass is [AR-checker.2.9](../../crates/grund-core/src/checker/report.rs). The homeless kind ([§FS-check.3.11.1](FS-check.md#3111-the-homeless-kind)) and a non-citable kind ([§FS-check.3.11.2](FS-check.md#3112-a-non-citable-kind)) are asked per file instead, at the row's `grounding_level` ([§FS-check.3.11.3](FS-check.md#3113-the-unit-follows-grounding_level)); every failing unit is reported ([§FS-check.3.11.4](FS-check.md#3114-every-failing-unit-is-reported)), a file with no citation is no unit ([§FS-check.3.11.5](FS-check.md#3115-a-file-with-no-citation-is-no-unit)), and an `E2E` case has its own unit ([§FS-check.3.11.6](FS-check.md#3116-an-e2e-case)). The parallel `should` obligation is not an error; it is a suggestion ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)).

The ordinary rule sentence `<subject> must cite at least one <target-set>.`
reuses this code only when its actual count is zero. Rule-to-rule and the narrow
config-to-rule bridge deduplicate as
[§FS-rules.6](FS-rules.md#6-semantic-deduplication) specifies; when config
participates, this section's existing message stays byte-for-byte unchanged.

Under v2 the obligation is written as `must`, `warn` or `should` ([§FS-config-v2.rules.citations](FS-config-v2.md#rulescitations-rulescitations)). `must` is this error; `warn` is the same finding, code and message on the warning channel, exit `0`; `should` is the suggestion of [§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in). v1's `[citations]` keeps exactly the channels above.


#### 3.11.1 The homeless kind

A **homeless-kind** obligation ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)) — `code`, or whatever the project named it — is per file rather than per declaration: a source file that contains at least one citation but none satisfying the obligation is the error, located at line 1.

#### 3.11.2 A non-citable kind

**A non-citable kind's obligation is per file too** ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)), and its unit is every scanned file in the kind's home that carries at least one citation — **`.md` included**, unlike `code`. Obligations attach to declarations, and a kind that declares nothing would otherwise yield no units at all and let `must` pass vacuously; inheriting `code`'s Markdown exemption would do the same thing a second time, since such a home is usually all Markdown. The finding names the **home**, because the unit has no ID to print:

```
skills/review/SKILL.md:1: skills/ must cite FS (citation direction)
```

#### 3.11.3 The unit follows `grounding_level`

Both per-file units follow the row's `grounding_level` ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)): *whether* a place's files must cite and *what* they must cite are asked of the same thing ([§FS-check.3.6.2](FS-check.md#362-the-unit)). At level `2` the unit of a non-citable Markdown home is every `##` subtree that carries a citation, and of a source file every unindented doc-comment block that does; the file stays a unit at every level, satisfied by any citation under it. A row at level `1` — which is every configuration written before the key existed — sees no change, and a citable kind's unit stays its declaration at every level, a declaration already being a unit inside a file.

#### 3.11.4 Every failing unit is reported

As for grounding ([§FS-check.3.6.3.1](FS-check.md#3631-every-failing-unit-is-reported)), at level `2` or above a file whose citations satisfy no `must` entry earns the finding on the file unit *and* one on each section unit that satisfies none either: the file genuinely cites no such target, and neither does the section. The two lines differ only in the anchor, which is what tells the reader whether the miss is local to one section.

#### 3.11.5 A file with no citation is no unit

Units are built from citations, so a file carrying none produces no unit and `must` cannot fire on it — except that a scanned folder with real non-entry content earns the run-level warning of [§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing). The same zero-unit boundary [§FS-config.3.9.2](FS-config.md#392-the-homeless-kind) states for the homeless kind remains intentionally unwarned. In a non-citable home `require_grounding` closes the per-file grounding hole ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)): there the grounding rule follows the home rather than the file extension, so "cite something" and "cite an `FS`" are two keys that compose, while the warning of [§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing) points at the row's key when grounding is off.

#### 3.11.6 An `E2E` case

An `E2E`-kind obligation ([§FS-config.3.9.1](FS-config.md#391-levels)) is per case declaration, can be satisfied by the case's `spec.refs` manifest entries, and remains an error when the case has no scanned citations or matching manifest entry.

### 3.12 Forbidden citation

When `[citations]` ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)) sets a `must-not` prohibition for a citing kind, every citation site of that kind to a prohibited target is an error located at the citation site:

```
docs/functional-spec/FS-login.md:42: FS must not cite AR (citation direction) — re-point the citation or downgrade it to a plain Markdown link
```

The prohibition pass is [AR-checker.2.10](../../crates/grund-core/src/checker/report.rs); how it reads the citing and the cited kind is [§FS-check.3.12.1](FS-check.md#3121-how-the-two-kinds-are-read). The parallel `should-not` prohibition is not an error; it is a suggestion ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)). The sanctioned way to keep a discouraged downward pointer is a plain Markdown link, which is not a citation under `strict = true` and so is exempt from this rule.

A rule sentence `<subject> must not cite any <target-set>.` reuses this code and
the exact citation-site anchor. Its message replaces only the fixed `(citation
direction)` authority tail with `(<RULE-ID>)`; a config-derived duplicate keeps
this section's bytes unchanged ([§FS-rules.6](FS-rules.md#6-semantic-deduplication)).

Under v2 the prohibition is written as `must-not`, `warn-not` or `should-not` ([§FS-config-v2.rules.citations](FS-config-v2.md#rulescitations-rulescitations)). `must-not` is this error; `warn-not` is the same finding, code and message on the warning channel, exit `0`; `should-not` is the suggestion of [§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in).


#### 3.12.1 How the two kinds are read

The citing kind is the site's resolved `source_kind` ([AR-scanner.2.4](../architecture/AR-scanner.md#24-citing-side-classification)), named by kind for a citable kind, by **home** for a non-citable one the way [§FS-check.3.11.2](FS-check.md#3112-a-non-citable-kind) names it (`skills/ must not cite AR`), and by name for the homeless kind, which has no home to name it by ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)). The cited kind and alias come from the citation token, matched against the rule's alias grammar ([§FS-config.3.9.3](FS-config.md#393-alias-matching)).

### 3.13 Number-only shorthand citation

A recognized shorthand citation ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)) persisted in a scanned file is governed by the target project's `[reference] shorthand` policy ([§FS-config.3.1](FS-config.md#31-reference--citation-form)). Under the default `canonical` policy, a uniquely resolving site is reported with its replacement text and `grund fmt --write` applies that mechanical fix in bulk ([§FS-fmt.2.4](FS-fmt.md#24-shorthand-to-canonical)). Under `accepted`, the same marker-origin site is valid and produces no shorthand-form finding; its full canonical citation may coexist in the same file. The policy gate changes only the unique result: a shorthand matching no declaration or several still earns the unknown or ambiguous form of [§FS-check.3.13.3](FS-check.md#3133-one-finding-per-site-in-three-forms), and no policy permits grund to guess.

Under `canonical`, an error rather than a warning or a suggestion: a warning leaves the exit code alone, so a repo could accumulate shorthand citations forever while CI stayed green ([§DF-number-only-citation-shorthand.2.3](../decisions/functional/DF-number-only-citation-shorthand.md#23-it-is-an-error-not-a-warning-or-a-suggestion)). A citation of a kind whose effective format has no `{number}` or no `{slug}` never earns this finding, because that kind has no shorthand ([§FS-check.1.2.1](FS-check.md#121-the-shorthand-shape)). Where the text itself forbids the rewrite, the resolving form does not fire ([§FS-check.3.13.1](FS-check.md#3131-where-the-text-forbids-the-rewrite), [§FS-check.3.13.2](FS-check.md#3132-a-python-docstring-is-not-a-string-literal)).

#### 3.13.1 Where the text forbids the rewrite

A shorthand inside inline code ([§FS-fmt.2.3.5](FS-fmt.md#235-an-inline-code-span-closes-on-a-run-of-its-own-length)), a Markdown link destination, or a source string literal is exempt from the resolving form of this error, because [§FS-fmt.2.3](FS-fmt.md#23-what-is-never-rewritten) forbids the rewrite in those three contexts, where the text is legitimate as it stands, and an error whose only named fix the tool declines to perform there is one a repository can never clear. A suppressed scope and an external file-symlink target are not among them: `fmt` will not rewrite there either, but the error still fires and the author clears it by hand ([§FS-fmt.2.5](FS-fmt.md#25-suppressed-scopes), [§FS-fmt.2.3.2](FS-fmt.md#232-a-link-that-leaves-the-config-root-is-not-written-through)). The exempt citation is untouched in every other respect — it resolves, `refs` lists it, and it keeps its declaration from being reported unused ([§FS-check.1.2.4](FS-check.md#124-a-resolved-shorthand-is-a-real-edge)). The exemption is for the *mechanical* form only: a shorthand matching zero or several declarations is still reported in those contexts, because that is a dangling citation rather than a formatting nit.

#### 3.13.2 A Python docstring is not a string literal

A Python docstring is not a string literal for this exemption: its delimiters are doc-comment syntax, so the question is asked of its content ([§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule)), and a shorthand anywhere inside one — the opening line, a one-line docstring, an interior line, the closing line — is reported and rewritten exactly like one in a `#` comment. A shorthand inside a `"…"` or `'…'` literal on a **code** line is what stays exempt.

#### 3.13.3 One finding per site, in three forms

At most one *shorthand* finding per site, in one of three forms. Other rules judge the site on their own terms — a bad section ([§FS-check.3.2](FS-check.md#32-missing-section)) or a forbidden direction ([§FS-check.3.12](FS-check.md#312-forbidden-citation)) is a separate fact about the same citation and is reported separately:

```
docs/notes.md:5: shorthand citation §FS-042; write §FS-042-user-login
docs/notes.md:6: shorthand citation §FS-999 matches no declaration
docs/notes.md:7: shorthand citation §FS-042 is ambiguous: FS-042-user-login, FS-042-user-logout
```

The candidate list in the ambiguous form is sorted and complete — `grund` names every match and resolves none, because choosing one would be a guess and `check` reports facts about the tree ([§FS-check.5](FS-check.md#5-what-grund-does-not-check), [§GOAL-agent-grounding.3](../goals.md#3-what-this-rules-out), [§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution)). Duplicate *numbers* are not otherwise an error: [§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration) catches duplicate full IDs, and a repo may legitimately hold `FS-042-user-login` alongside `FS-042-user-logout` as long as nothing abbreviates them. The marker rendered in the message is the configured one ([§FS-config.3.1](FS-config.md#31-reference--citation-form)), and the qualified form names its alias (`<§>api/FS-042`, escaped here because this repo has no `api` member) so the replacement can be pasted as written.

### 3.14 Out-of-scope unresolvable citation *(`--full` only)*

Under `--full` ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)), a citation in a file outside the default scope that resolves to nothing — the ID is declared nowhere ([§FS-check.3.1](FS-check.md#31-dangling-citation)), the declaration exists but the cited section does not ([§FS-check.3.2](FS-check.md#32-missing-section)), the alias is unknown ([§FS-check.3.8](FS-check.md#38-cross-project-citation-failure)), or a number-only shorthand matches zero or several declarations ([§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)) — is an error ([§FS-check.3.14.1](FS-check.md#3141-an-error-not-a-warning)). So is a citation whose section coordinate no consumer can settle from outside the file it sits in: [§FS-check.3.24](FS-check.md#324-declaration-local-section-citation)'s whole-token form verdict on a declaration-local section citation — an owned numeric path, an ownerless or ambiguous site, or a digit-starting unsupported tail — settled from the token and the declaration structure of the file holding it, which may be the absence of a declaration. Five findings, not four, and what admits each of them out there is the test of [§FS-check.3.14.2](FS-check.md#3142-what-is-judged-outside-the-configured-scope) — a verdict reached from the citing token, the declaration structure of the file holding it, and the resolved graph — rather than resolution alone. The site is reported in the ordinary located-finding shape, with the scope named first and the rule's own message after it:

```
sim/world.py:12: outside [scan] include: unknown reference RES-061-world-arable-basin-screen
render/prompts.md:4: outside [scan] include: missing section DA-002-general-field-service-scope.1.4
```

#### 3.14.1 An error, not a warning

It moves the exit code to `1` like every other citation failure. A warning would leave `--full` exit-code-neutral, and a finding no CI run can fail on is one a repository accumulates behind forever — the argument [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation) already makes. Nothing turns red without being asked: the flag is opt-in.

#### 3.14.2 What is judged outside the configured scope

A finding is reported outside `[scan] include` when it is a verdict on a **citing token**, reached from that token, the declaration structure of the file holding it, and the resolved graph — and from nothing the project assigned the file: not its kind, not its home, not its citation obligations, not a convention a configured key switched on. The style, placement, grounding and direction rules each read the role the project gave the file; the duplicate and unused rules judge a declaration set a directory nobody configured is not part of. `[scan] include` is where a project says which files it gave those roles to, so a directory nobody configured has agreed to none of them ([§FS-check.1.3.3](FS-check.md#133-two-scopes-two-rule-sets)).

The test has one subtraction, and it is about the remedy rather than the subject: a verdict whose subject is what `grund fmt` did or did not rewrite is withheld out there, because `fmt` scopes by `[scan] include` too ([§FS-check.3.14.4](FS-check.md#3144-the-mechanical-shorthand-rewrite-is-withheld)), so out of scope such a verdict has no premise left — the rewrite it reports as not having happened was never due. [§FS-check.3.15](FS-check.md#315-shorthand-citation-in-a-numeric-run)'s numeric-run verdict is the one rule this removes ([§FS-check.3.15.5](FS-check.md#3155-withheld-out-of-scope)): it meets every clause of the test above and is still reported nowhere out there. The cut is along the verdict's **subject**, never the word *shorthand*. [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)'s zero-or-several form is a resolution failure rather than a formatting nit and keeps its finding out there ([§FS-check.3.14.4](FS-check.md#3144-the-mechanical-shorthand-rewrite-is-withheld), [§FS-check.3.14.5](FS-check.md#3145-a-compound-code-per-rule)), and [§FS-check.3.24](FS-check.md#324-declaration-local-section-citation)'s finding names a manual full replacement first, so out there it survives minus its `` ; run `grund fmt --write` `` clause ([§FS-check.3.24.1](FS-check.md#3241-the-release-attribution-and-where-the-command-clause-is-withheld)).

One declaring-side finding is admitted beside the citing-token ones — [§FS-declarations.checks.section-outside-declaration](FS-declarations.md#checkssection-outside-declaration-section-outside-a-declaration) — because invalid scanner structure precedes any project convention, so it retains the unprefixed `section-outside-declaration` code and its ordinary message outside scope too. The two admitted exceptions stand on the same ground rather than on a second and a third: both read only the file's own scanner structure. [§FS-check.3.24](FS-check.md#324-declaration-local-section-citation)'s form verdict is computed from the token's shape and the enclosing declaration body that owns it ([§FS-check.1.1.8](FS-check.md#118-declaration-local-numeric-section-candidates)), and holds under configured markers and both strict modes; nothing the project said about the file is consulted. A sixth rule asking to be reported out there is held to this test rather than argued about.

#### 3.14.3 Resolution sees the whole scan

An out-of-scope citation whose declaration is also out of scope resolves normally. This check reports citations that point at *nothing*, not citations that point outside the default scope.

#### 3.14.4 The mechanical shorthand rewrite is withheld

[§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)'s resolving form names `grund fmt --write` as its fix, and `fmt` scopes by `[scan] include`, so out there the error would name a fix the formatter declines to apply — the same reason [§FS-check.3.13.1](FS-check.md#3131-where-the-text-forbids-the-rewrite) withholds it at an unrewritable site. A shorthand that matches zero or several declarations is a resolution failure, not a formatting nit, and is reported.

#### 3.14.5 A compound code per rule

A finding here carries `out-of-scope-` followed by the code its in-scope equivalent carries: `out-of-scope-dangling`, `out-of-scope-missing-section`, `out-of-scope-local-section-citation`, `out-of-scope-unknown-project`, `out-of-scope-shorthand-citation`. A `--format=json` consumer ([§FS-errors.5](FS-errors.md#5-json-format)) then filters the scope by prefix and the rule by exact match, both on the `code` field the shape already carries; one code for all five would have left the rule readable only by parsing the message prose. The JSON shape gains no field.

#### 3.14.6 The scope leads the message

`outside [scan] include: ` comes first, before the rule's own text, because it is the fact that changes what to do: out there the usual fix is to widen the key, not to edit the citation, and a rule's own fix-it hint — `did you mean …?`, `or write <§>… if this is an illustration` — is likelier to be the wrong advice and would otherwise be read first. Naming the key is the whole remedy the message carries; it does not also spell out "widen `[scan] include`", because every out-of-scope finding would repeat the same sentence and [§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full) states the remedy once.

#### 3.14.7 A wider scan can fail wider

The flag also puts files the default scope never touched into the scan, so one that cannot be read or decoded out there is reported as the `error: <path>: <reason>` of [§FS-check.2.4](FS-check.md#24-an-incomplete-run) on stderr and the run exits `2` — "I could not read this" is a fact about the run, not one of these findings, and it holds for a file in either scope. A tree whose plain `check` exits `0` can therefore exit `2` under `--full`; that is the wider scan reporting what it found, not a regression.

### 3.15 Shorthand citation in a numeric run

A number-only shorthand ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)) that resolves to exactly one declaration but sits glued to a second number, so `grund fmt` will not rewrite it ([§FS-fmt.2.4.1](FS-fmt.md#241-a-shorthand-in-a-numeric-run-is-not-rewritten)). Decided in [§DF-shorthand-numeric-run](../decisions/functional/DF-shorthand-numeric-run.md#df-shorthand-numeric-run-a-marked-shorthand-glued-to-another-number-is-a-numeral-not-a-citation).

```
docs/changelog.md:3: shorthand §SPEC-001 sits in a numeric run and was not rewritten; write §SPEC-001-checkout, or <§>SPEC-001 if these are old numbers
```

This is [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)'s site with a different verdict, so it takes [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)'s place there rather than adding a second finding — at most one *shorthand* finding per site still holds, and rules judging a different fact about the same citation still report alongside it. Its code is `shorthand-numeric-run` ([§FS-errors.5](FS-errors.md#5-json-format)). It names both fixes ([§FS-check.3.15.1](FS-check.md#3151-both-exits-are-named)), is an error ([§FS-check.3.15.2](FS-check.md#3152-an-error-with-new-bytes)), is withheld in the three text contexts where [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)'s resolving form is ([§FS-check.3.15.3](FS-check.md#3153-the-rule-this-one-is-an-exception-to)), covers only the resolving form ([§FS-check.3.15.4](FS-check.md#3154-only-the-resolving-form)), and is withheld out of scope ([§FS-check.3.15.5](FS-check.md#3155-withheld-out-of-scope)).

#### 3.15.1 Both exits are named

`grund` cannot know which was meant and the author knows at a glance. If it was a citation, the canonical text is there to paste; if the numbers were a mapping, `<§>` is the escape for writing an ID without citing it ([§FS-check.2.3.1](FS-check.md#231-escaped-citation-resolves)), offered in the same shape [§FS-check.3.1.2](FS-check.md#312-an-illustration-in-inline-code) uses for a dangling citation that might be an illustration.

#### 3.15.2 An error, with new bytes

It is an error for the reason [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation) gives. No verdict moves on upgrade — these sites are already [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation) errors — but their bytes do: the [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation) message and its `shorthand-citation` code give way to this one, and an existing finding's text otherwise changes only through the deprecation path ([§REQ-backwards-compatibility.1](../requirements/REQ-backwards-compatibility.md#1-what-is-covered)). This change takes the pre-1.0 licence of [§REQ-backwards-compatibility.4](../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise) instead: carrying the old text beside the new for a release would cost more than the rename, because it would go on advising, at every such site, the edit that corrupts the line.

#### 3.15.3 The rule this one is an exception to

[§FS-check.3.13.1](FS-check.md#3131-where-the-text-forbids-the-rewrite) withholds the resolving form where the text forbids the rewrite, because an error whose only named fix the tool declines to perform there can never be cleared. That reasoning covers the three [§FS-fmt.2.3](FS-fmt.md#23-what-is-never-rewritten) text contexts, where the text is legitimate as it stands and no edit is wanted at all — and this finding is withheld there too, for that reason, while in a suppressed scope or an external file-symlink target it fires as [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)'s does. It does not cover a run, where the line does need an edit and a person can make it in one keystroke. What has to hold is that every finding names a fix, not that every fix is `fmt`'s.

#### 3.15.4 Only the resolving form

A shorthand in a run matching zero or several declarations keeps its [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation) message. That is a resolution failure, reported on its own terms, and a run is no reason to say less about it.

#### 3.15.5 Withheld out of scope

Under `--full` ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)) the site is outside `[scan] include`, and this finding is the one subtraction [§FS-check.3.14.2](FS-check.md#3142-what-is-judged-outside-the-configured-scope) makes from its own test: it meets every clause of that test, and is withheld all the same because its subject is the rewrite that did not happen, which [§FS-check.3.14.4](FS-check.md#3144-the-mechanical-shorthand-rewrite-is-withheld) withholds out there for the same reason — `fmt` scopes by `include` too, so the finding would name an edit no run in that scope is asking for.

### 3.16 Duplicate section path

Moved to [§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path). This address is kept so citations written before the move still resolve.

### 3.17 Index entry is not a link

A citation of a covered ID in the index file that is bare rather than a full Markdown link, and so not yet the entry [§FS-check.3.18.5](FS-check.md#3185-what-an-entry-is) requires, reported at **the citation's line in the index**:

```
docs/discussions/README.md:12: index entry §DISC-external-ticket-resolvers is not a link; unchecked in grund 0.11.0, an error in 0.12.0 — run `grund fmt --write`
```

The index is a *file* index: its job is to get a reader from the folder to the declaration, and a bare `§<ID>` in it is a promise the reader cannot follow. The entry owes the link `fmt` writes ([§FS-check.3.17.1](FS-check.md#3171-the-required-form-is-the-link-fmt-writes)) but is judged only by its shape ([§FS-check.3.17.2](FS-check.md#3172-the-shape-never-the-target)); the finding is an error on arrival ([§FS-check.3.17.3](FS-check.md#3173-an-error-on-arrival)) that only a citation `fmt` would wrap can earn ([§FS-check.3.17.4](FS-check.md#3174-only-a-citation-fmt-would-wrap-reaches-this-rule), [§FS-check.3.17.5](FS-check.md#3175-anything-else-is-not-an-entry)), once per ID ([§FS-check.3.17.6](FS-check.md#3176-one-finding-per-id)).

- **Code:** `unlinked-index-entry` ([§FS-errors.5](FS-errors.md#5-json-format)).

#### 3.17.1 The required form is the link `fmt` writes

The required form is exactly the link `grund fmt --cross-refs` writes ([§FS-fmt.6.2](FS-fmt.md#62-form)) — the relative path to the declaration's home, plus the heading anchor under the active `anchor_format`. That is the canonical target, not "has an anchor": a declaration whose home is a source file links to the bare file path with no anchor, and `anchor_format = "none"` drops anchors everywhere. `docs/architecture/README.md` already carries that case, and it is correct as written.

#### 3.17.2 The shape, never the target

`check` requires the **shape** — the citation wrapped as `[§<ID>…](<target>)` — and never the target. A wrap's URL is re-derived on every `grund fmt --cross-refs` pass ([§FS-fmt.6.3](FS-fmt.md#63-idempotency-and-re-derive)), so a heading rename that rots an anchor is a one-line `fmt` diff rather than a second finding here, and re-deriving it in `check` would put the anchor algorithm in a second command for no new coverage.

#### 3.17.3 An error on arrival

This half is an **error on arrival**, under [§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations): the message names the versions the verdict moved between, the fix is one documented command the tool ships, `grund fmt --write`, and the release notes it. That licence is only honest while the named command is one that would actually act here, which is what [§FS-check.3.17.4](FS-check.md#3174-only-a-citation-fmt-would-wrap-reaches-this-rule) is for.

For one release the two halves of the entry contract disagreed about severity — the greater offence warned while the lesser errored — and what decided that was having a fix command rather than the size of the offence ([§DF-index-compatibility-ramp](../decisions/functional/DF-index-compatibility-ramp.md#df-index-compatibility-ramp-a-findings-ramp-follows-its-fix-command-not-the-size-of-the-offence)). The inversion closed when [§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)'s ramp did ([§FS-check.3.18.9](FS-check.md#3189-an-error-because-the-deadline-the-warning-named-has-arrived)), and one verdict now covers both halves.

#### 3.17.4 Only a citation `fmt` would wrap reaches this rule

The bare form this reports is, exactly, an occurrence the next `grund fmt --write` turns into the link of [§FS-check.3.17.1](FS-check.md#3171-the-required-form-is-the-link-fmt-writes):

- in the index file, which is a Markdown file by construction — `index` must name one ([§FS-config.3.4.2](FS-config.md#342-index--the-kinds-index-file)) because `--cross-refs` runs on `.md` files only ([§FS-fmt.6.1](FS-fmt.md#61-scope));
- **marker-prefixed**, because without `--marker` the link pass leaves a bare token bare ([§FS-fmt.6.5](FS-fmt.md#65-interaction-with---marker)). `grund fmt --write --marker` *would* reach an unmarked token — but only by marking every bare citation in the tree, which is the repository-wide style choice a project on `[reference] strict = false` has already declined ([§FS-config.3.1](FS-config.md#31-reference--citation-form)). A finding may name a command that repairs it, not one that changes something else on the way; the same objection [§DF-index-always-linkified](../decisions/functional/DF-index-always-linkified.md#df-index-always-linkified-the-cross-reference-pass-always-runs-on-a-kinds-index-file) raises against `--cross-refs --write` as a fix;
- outside every zone `fmt` never writes in ([§FS-fmt.2.3](FS-fmt.md#23-what-is-never-rewritten), [§FS-fmt.6.4](FS-fmt.md#64-what-is-never-wrapped)): an inline-code span, ending where [§FS-fmt.2.3.5](FS-fmt.md#235-an-inline-code-span-closes-on-a-run-of-its-own-length) says, a Markdown link destination — a bare ID-shaped token inside `](…)` is a URL, not an entry — a fenced block, a declaration heading line, and an index file reached as an external file-symlink target, which `--write` reads and does not write through ([§FS-fmt.2.3.2](FS-fmt.md#232-a-link-that-leaves-the-config-root-is-not-written-through)). A bare citation in such an index is consequently not an entry and falls to [§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index), while a full link already present there still satisfies the entry obligation; an ordinary index, including a file symlink whose target stays inside the root, remains repairable and reaches this rule;
- and **naming a section that exists**, when it names one at all. The pass has to compute a link target ([§FS-fmt.6.2](FS-fmt.md#62-form)), and a citation whose section no declaration declares has none, so `fmt` passes over the line. Such a citation is already reported by [§FS-check.3.2](FS-check.md#32-missing-section), and adding a second finding whose named command answers `rewrote 0 lines` would be the trap [§FS-check.3.17.5](FS-check.md#3175-anything-else-is-not-an-entry) exists to avoid.

#### 3.17.5 Anything else is not an entry

Anything else in the index is **not an entry**: it neither satisfies [§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index) nor is reported here, and the ID falls to [§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index), whose fix is a human edit rather than a command. The alternative is an error whose named fix the tool declines to perform, which is an error a repository can never clear — the same trap [§FS-check.3.13.1](FS-check.md#3131-where-the-text-forbids-the-rewrite) stays out of, and by the same predicate ([§DF-index-entry-form.2.3](../decisions/functional/DF-index-entry-form.md#23-one-link-per-id-not-every-mention)).

The condition runs one way only: an entry that already *is* a link satisfies [§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index) whatever `fmt` would do with it. Off strict mode, where an unmarked token is a citation ([§FS-config.3.1](FS-config.md#31-reference--citation-form)), that means a hand-written `[FS-x](…)` around one is a correct entry and is left alone; under `strict = true` the same line carries no citation at all, so the ID has no entry and is [§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)'s.

#### 3.17.6 One finding per ID

Only an ID the index already cites in a form `fmt` would wrap ([§FS-check.3.17.4](FS-check.md#3174-only-a-citation-fmt-would-wrap-reaches-this-rule)) reaches this rule; an ID with neither that nor an entry is [§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)'s, and one cause never yields both findings. Where several citations of one ID sit in the index and none is a link, the finding is located at the first of them in file order.

### 3.18 Declaration missing from its kind's index

A kind configured with a `folder` and an index file — `README.md` unless `index` names another or opts out ([§FS-config.3.4.2](FS-config.md#342-index--the-kinds-index-file)) — promises that the index lists that folder's declarations; nothing verified it before. Every covered declaration the index does not name is one error, located at the **declaration's heading** and naming the index file:

```
docs/decisions/functional/DF-md-link-emission.md:1: DF-md-link-emission is not listed in docs/decisions/functional/README.md — became an error in grund 0.13.0
```

Its children say which kinds ([§FS-check.3.18.1](FS-check.md#3181-which-kinds-are-covered)) and declarations ([§FS-check.3.18.2](FS-check.md#3182-which-declarations-are-covered)) are covered, how an external source declaration enrolls ([§FS-check.3.18.3](FS-check.md#3183-an-external-source-declaration-enrolls-by-its-canonical-link), [§FS-check.3.18.4](FS-check.md#3184-what-does-not-enroll)), what an entry is ([§FS-check.3.18.5](FS-check.md#3185-what-an-entry-is), [§FS-check.3.18.6](FS-check.md#3186-and-nothing-more)), how a missing or unscanned index is judged ([§FS-check.3.18.7](FS-check.md#3187-a-missing-index-file), [§FS-check.3.18.8](FS-check.md#3188-a-run-that-cannot-see-the-index-does-not-judge-it)), and why this is an error ([§FS-check.3.18.9](FS-check.md#3189-an-error-because-the-deadline-the-warning-named-has-arrived)). Decided in [§DF-index-entry-form](../decisions/functional/DF-index-entry-form.md#df-index-entry-form-an-index-entry-is-one-full-link-per-id-and-nothing-else-about-the-page), [§DF-index-compatibility-ramp](../decisions/functional/DF-index-compatibility-ramp.md#df-index-compatibility-ramp-a-findings-ramp-follows-its-fix-command-not-the-size-of-the-offence), and [§DF-index-not-an-inbound-citation](../decisions/functional/DF-index-not-an-inbound-citation.md#df-index-not-an-inbound-citation-an-index-entry-is-navigation-not-use).

- **Code:** `missing-index-entry` ([§FS-errors.5](FS-errors.md#5-json-format)).

#### 3.18.1 Which kinds are covered

Folder kinds that declare IDs. A `citable = false` kind ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)) has no declarations, so it has no index and this rule never reaches it — which is why setting `index` on one is a config error rather than a silent no-op ([§FS-config.3.4.2](FS-config.md#342-index--the-kinds-index-file)).

#### 3.18.2 Which declarations are covered

Every ID of that kind with at least one declaration site anywhere under `folder` — the whole subtree, not its top level, because a kind's folder routinely holds a directory per topic or per year (`DISC`'s proposals all live in `docs/discussions/proposals/`). A stub-and-inline pair collapses the way [§FS-list.2](FS-list.md#2-behaviour) collapses it: the stub under `folder` is what puts the ID in the folder, and **one** entry for the ID satisfies the rule — pointing at wherever the body lives, which for an inline home is the source file. A declaration of some *other* kind sitting inside the folder is a misplaced declaration ([§FS-declarations.checks.misplaced-declaration](FS-declarations.md#checksmisplaced-declaration-misplaced-declaration-configured-kind-home)) and is not additionally demanded here.

#### 3.18.3 An external source declaration enrolls by its canonical link

An index may also **enroll one external source declaration directly**, with no stub under `folder`. The enrollment is deliberately a stricter form than an ordinary entry: an unqualified, marker-prefixed citation of the bare ID (no section) whose declaration is in a non-Markdown source file outside `folder`, wrapped as a Markdown link whose destination is exactly the one `grund fmt --cross-refs` derives from that index to the source home ([§FS-fmt.6.2](FS-fmt.md#62-form)). The same canonical link is both the act of membership and the satisfying entry, so `check` requires no third artifact. Where two kinds share one `folder` and one `index`, each enrolls its own: the link is matched to the kind its ID names, so configuration order never lets one kind hide another's external entry. `grund show` and `grund list` still see the source declaration as the only home; enrollment creates no declaration record and no synthetic stub ([§FS-show.2.3](FS-show.md#23-source-declarations-in-code-and-doc-comments), [§FS-list.2](FS-list.md#2-behaviour)).

#### 3.18.4 What does not enroll

Every condition distinguishes enrollment from surrounding prose. A foreign-kind or qualified citation, a citation of a section, a **number-only shorthand** ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)) — enrollment is the *persisted* whole ID, and a shorthand stays authoring sugar until `grund fmt --write` expands it ([§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)) — an unlinked mention, a link **nested inside another link's destination** or any other zone `fmt` never writes in ([§FS-check.3.17.4](FS-check.md#3174-only-a-citation-fmt-would-wrap-reaches-this-rule)), a link with a different destination, and a link to a Markdown declaration outside `folder` are ordinary citations. They neither enroll the ID nor become navigational for [§FS-check.4.1.2](FS-check.md#412-an-index-entry-does-not-count). A marker-prefixed bare-ID mention that `grund fmt --cross-refs` turns into the exact canonical link becomes an enrollment when that form is written — the stored link is the unambiguous signal. Removing it removes the external membership, so there is no missing-entry finding for an external declaration that the index no longer claims. Decided in [§DF-index-entry-form.2.7](../decisions/functional/DF-index-entry-form.md#27-a-canonical-bare-id-link-enrolls-an-external-inline-declaration).

#### 3.18.5 What an entry is

For an ID covered by a declaration under `folder`, one recognized citation ([§FS-check.1.1](FS-check.md#11-recognized-citations)) of the ID in the index file, written as a full Markdown link. The two conditions are the entry's contract and either one unmet is a finding: this rule is the first, and [§FS-check.3.17](FS-check.md#317-index-entry-is-not-a-link) is the second. For an external source declaration, the canonical link of [§FS-check.3.18.3](FS-check.md#3183-an-external-source-declaration-enrolls-by-its-canonical-link) establishes coverage and satisfies the entry simultaneously.

Between them sits a third case, and it lands here: a citation `grund fmt --write` would not wrap ([§FS-check.3.17.4](FS-check.md#3174-only-a-citation-fmt-would-wrap-reaches-this-rule)) is not an entry at all, so an index that mentions the ID only that way is reported *here*, where the fix is to write an entry, and never under [§FS-check.3.17](FS-check.md#317-index-entry-is-not-a-link), where the command the message names would decline to act ([§FS-check.3.17.5](FS-check.md#3175-anything-else-is-not-an-entry)).

So is a mention the scan never recorded as a citation at all — a markerless link or a bare file name under `strict = true`, where no citation exists to be discarded. The two reach this rule by different routes and are reported in the same words: where the index mentions the ID and holds no entry for it, the finding says the ID **appears** in the index but not as an entry, and names the form an entry takes. One sentence rather than two, because the remedy is the same either way and the look of [§FS-check.3.18.5.1](FS-check.md#31851-what-appears-means) sees both. The form it names follows `[reference] strict`, because a clause that named a stricter form than the run requires would be the very thing this wording exists to stop — a sentence the reader can see is not so about the page in front of them. Under `strict = true` that form is a marked Markdown link to the declaration; off strict mode, where a bare token is a citation too ([§FS-config.3.1](FS-config.md#31-reference--citation-form)), the marker is optional and what makes the link an entry is that its own text is the ID, which is the distinction [§FS-check.3.17.5](FS-check.md#3175-anything-else-is-not-an-entry) draws said where the reader is standing. What counts as a mention is that section.

##### 3.18.5.1 What "appears" means

The look is at text rather than at citations, and it is deliberately narrow. The index **mentions** the ID when some line of the index file carries either the ID as `grund` renders it — with or without the marker, with or without a section suffix, as an ID-shaped token on its own boundaries, so a longer ID containing this one is not a mention of it — or the file name of the declaration the finding is located at, as a whole name, so `overview.md` is not mentioned by `my-overview.md` and not by `overview.markdown`.

Two kinds of line are not looked at. A line inside a Markdown fenced code block, by the one fence reader every surface shares ([§FS-check.1.1.5](FS-check.md#115-contexts-read-as-neither-prose-nor-code)), because an index that illustrates the form in an example has listed nothing. And a declaration heading: a citation riding on one is not an entry either ([§FS-fmt.6.4](FS-fmt.md#64-what-is-never-wrapped)), and a line that declares an ID is not a page mentioning it.

Inline code is **not** excluded. An index that shows the ID in an inline span has mentioned it, which is what makes one sentence true both of a citation the scan recorded and discarded ([§FS-check.3.17.4](FS-check.md#3174-only-a-citation-fmt-would-wrap-reaches-this-rule)) and of a markerless mention it never recorded at all.

A mention is never itself a finding and never satisfies the rule ([§FS-check.3.18.6](FS-check.md#3186-and-nothing-more)): it chooses the words of a finding this rule has already raised. Where the index file could not be read there is no text to look at, so [§FS-check.3.18.7](FS-check.md#3187-a-missing-index-file)'s parenthesis stands alone and the two clauses never appear on one line.

#### 3.18.6 And nothing more

Layout is free: table or list, grouped or flat, in any order, with any prose around it. `docs/functional-spec/README.md` groups its entries under six curated headings, and a rule that dictated a table would break the best index in the tree. One link per ID is enough — every other occurrence of the ID in the index is untouched and is never a finding ([§DF-index-entry-form.2.3](../decisions/functional/DF-index-entry-form.md#23-one-link-per-id-not-every-mention)) — and the mention [§FS-check.3.18.5.1](FS-check.md#31851-what-appears-means) looks for is not an exception to that: it changes the words of a finding this rule already raises, and raises none.

#### 3.18.7 A missing index file

A missing index file is this same finding class, once per declaration in the folder: a folder whose index nobody wrote is the strongest form of the same fact, not a different one. It is also why the finding is located at the declaration and not at the index — an index file that does not exist has no line to point at, and every declaration has one. The message says which way it failed, in a parenthesis after the file name: `(the index file does not exist)`, `(the index file is a directory)` for a path that is one, and `(the index file could not be read)` for a file that is there and would not open. Three phrasings rather than one because "does not exist", said about a directory that plainly does, is a diagnosis the reader has to argue with before they can act on it.

#### 3.18.8 A run that cannot see the index does not judge it

Which IDs the index names comes from the scan, while the form of each entry comes from re-reading the file, and the two can disagree about whether the index was read at all: a narrowed `grund check <one-file>` ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)), or an index the `[scan]` set excludes, leaves the index unscanned while the declarations under the folder are still in view. Reporting every one of them as unlisted would be a finding about the scope, not about the tree, so this rule is skipped for an index file the run did not scan. An index file that is *not there* — missing, or a directory wearing the name — is a fact about the tree and is still reported.

#### 3.18.9 An error, because the deadline the warning named has arrived

No `grund` command writes a missing entry — rendering the index is not a pass `fmt` has — so this rule never had [§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)'s licence for a same-release verdict flip, and took [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)'s deprecation path instead. It arrived in `0.12.0` as a warning whose own text named the release it would become an error in, `0.13.0`, and that release ended the ramp. So the finding is an error like every other in this section — it contributes to the exit code ([§FS-check.3](FS-check.md#3-errors-detected)) and it stands in place of the `success` line ([§FS-check.2.1](FS-check.md#21-report-format)). The ramp bought what it was for: a repository that never listed its declarations was told, by the tool, in `0.12.0`, exactly which run would start failing.

The deadline clause is spent: a release still ahead is a date a reader can act on, and one that has arrived is not. What replaces it reports rather than promises, `— became an error in grund 0.13.0`, which is the past-tense half of the closed vocabulary [§FS-distribution.4.2](FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name) defines. A landed ramp names its release for the same reason a pending one names its deadline, and for one more: that clause is the only record of the flip a user reads without the changelog, and it is what makes the release path able to refuse a version this error would contradict.

### 3.19 Orphan name-bearing section path

Moved to [§FS-declarations.checks.orphan-section](FS-declarations.md#checksorphan-section-orphan-name-bearing-section-path). This address is kept so citations written before the move still resolve.

### 3.20 Invalid value declaration

In a kind opted into whole values, a readable Markdown or JSON declaration that violates [§FS-values.2](FS-values.md#2-value-declarations) is an error at the exact invalid heading, key, or element. An exact embedded tag in an invalid location or a marked root with an invalid shape is the same error, located at the tag for the root/authority failures and at the offending line for content/shape failures ([§FS-values.2.4](FS-values.md#24-embedded-section-value-roots)). Inside a declared value chapter the same error covers content the chapter may not hold and a root whose run is invalid, located at the offending line, or at the root heading for a root-level failure a chapter root has no tag to carry ([§FS-values.2.5](FS-values.md#25-chapter-declared-value-roots)). The code is `invalid-value-declaration`. Duplicate declarations retain [§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration) instead; a duplicate section may independently carry this finding, and home JSON input that [§FS-values.5.3](FS-values.md#53-incomplete-input-and-deterministic-output) counts as incomplete retains exit `2` ([§FS-check.2](FS-check.md#2-outputs)).

### 3.21 Invalid value binding

An attempted backtick-delimited binding that violates the exact grammar in [§FS-values.3.1](FS-values.md#31-the-only-binding-grammar), including a literal that closes its line onto a next-line citation ([§FS-values.3.1.1.1](FS-values.md#3111-a-literal-that-closes-its-line-onto-the-citation)), targets a declared value chapter heading itself, descends below one of a root's immediate components, or aims at a root one of whose components contains an ASCII space ([§FS-values.3.1.2](FS-values.md#312-a-binding-aimed-at-the-root)) is an error at the attempted form. Its code is `invalid-value-binding`. Unbackticked adjacency, bare citations (a next-line citation that a literal closes onto is not bare), a delimited form aimed at an ordinary unmarked section or at a named section outside every declared chapter, and the same shape for a whole declaration whose kind lacks `values = true` are not attempts and remain ordinary prose/citations.

### 3.22 Value mismatch

After ordinary citation and section resolution succeeds uniquely, a binding whose authored component differs from its whole declaration or marked root under [§FS-values.4](FS-values.md#4-exact-equality) — or, aimed at the root, whose literal does not split into that root's components ([§FS-values.5.2.2](FS-values.md#522-a-root-aimed-mismatch-names-the-first-unequal-component)) — is an error at the binding and names the component site: the first unequal component's, or component 1's when the counts differ. Its code is `value-mismatch`; the canonical text and NDJSON parity are fixed by [§FS-values.5](FS-values.md#5-resolution-findings-and-exit-status). An unknown alias, dangling ID, duplicate or invalid declaration, duplicate or missing section, invalid root, or noncanonical shorthand suppresses this comparison so one bad citation is never also reported as a mismatch. Ordinary declaration and section-structure findings run before comparison.

### 3.23 Section outside a declaration

Moved to [§FS-declarations.checks.section-outside-declaration](FS-declarations.md#checkssection-outside-declaration-section-outside-a-declaration). This address is kept so citations written before the move still resolve.

### 3.24 Declaration-local section citation

Every candidate from [§FS-check.1.1.8](FS-check.md#118-declaration-local-numeric-section-candidates) receives one whole-token form verdict with code
`local-section-citation`, and every one of them ends by naming the two releases its verdict moved
between ([§FS-check.3.24.1](FS-check.md#3241-the-release-attribution-and-where-the-command-clause-is-withheld)):

- An owned numeric path is an error whose remedy turns on whether the owner has the cited
  section. Where it has, the error is `local section citation <token>; write
  <marker><owner><separator><path> — unchecked in grund 0.13.1, an error in 0.14.0`, which gains a
  further `` ; run `grund fmt --write` `` exactly where the next formatter pass would write
  that site ([§FS-check.3.24.1](FS-check.md#3241-the-release-attribution-and-where-the-command-clause-is-withheld)). Where it has not, that citation would be reported missing in turn, so
  the error names the absence and gives the full-citation-or-escape guidance of the other two
  shapes instead, and never the command clause ([§FS-check.3.24.3](FS-check.md#3243-an-absent-target-section-is-answered-with-the-escape)). Either way the site remains a
  real edge for every graph consumer, and a missing target section independently receives
  [§FS-check.3.2](FS-check.md#32-missing-section).
- An ownerless or genuinely ambiguous site is an error that says no enclosing declaration can be
  chosen and instructs the author to write a full citation or escape the illustration, and then
  names the two releases. It has no guessed ID or navigation target, and never the command clause.
- A digit-starting mixed, named, or glued tail is an error naming the complete unsupported token
  and giving the same full-citation-or-escape guidance, and then the same two releases. No numeric
  prefix becomes an edge, and the command clause is not its either.

The finding covers the complete authored token for CLI and LSP ranges. An owned citation in a
location protected from automatic writing still names its manual full replacement where the owner
has the section, and the escape where it has not ([§FS-check.3.24.3](FS-check.md#3243-an-absent-target-section-is-answered-with-the-escape)); formatter
eligibility changes what can be rewritten, not whether persisted local form is canonical — it
changes only whether the message offers the command ([§FS-check.3.24.1](FS-check.md#3241-the-release-attribution-and-where-the-command-clause-is-withheld)). The
same rule applies under configured markers and both strict modes.

#### 3.24.1 The release attribution, and where the command clause is withheld

Every form of the finding ends `— unchecked in grund 0.13.1, an error in 0.14.0`: `0.14.0` is the
release the verdict moved in, and `0.13.1` the last release cut before it. That clause is the only
record of the flip a reader meets without opening the changelog, and it is what lets one line of
output separate *this tree predates the binary running over it* from *this citation is wrong* — the
same work [§FS-check.3.18.9](FS-check.md#3189-an-error-because-the-deadline-the-warning-named-has-arrived) states for the neighbouring rule, written in the past-tense half of
the closed release vocabulary [§FS-distribution.4.2](FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name) defines, so the release path can refuse a version
the clause would contradict. The flip itself keeps the licence it was taken under, [§REQ-backwards-compatibility.4](../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise);
what this clause adds is what [§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) asks a verdict-flipping finding to
say, and [§FS-check.3.17.3](FS-check.md#3173-an-error-on-arrival) already says.

`` ; run `grund fmt --write` `` follows the pair at an **owned** site whose text permits the
rewrite, inside the default scope — and nowhere else, because a finding may name a command
that repairs it and never one that would answer `rewrote 0 lines` ([§FS-check.3.17.5](FS-check.md#3175-anything-else-is-not-an-entry)). Four cases withhold
it, and a fifth deliberately does not:

- an ownerless, ambiguous, or digit-starting unsupported token, which [§FS-fmt.2.4](FS-fmt.md#24-shorthand-to-canonical) leaves
  byte-identical because there is no owner to expand it against;
- an owned site inside inline code, a Markdown link destination, or a source string literal — the
  three contexts [§FS-fmt.2.3](FS-fmt.md#23-what-is-never-rewritten) forbids the rewrite in. The error itself still fires here, unlike
  [§FS-check.3.13.1](FS-check.md#3131-where-the-text-forbids-the-rewrite)'s exemption, because the manual remedy the finding names is a fix the author
  can make; only the offer of the command goes;
- an out-of-scope finding of a `--full` run, where `fmt` does not reach at all, for the reason
  [§FS-check.3.14.4](FS-check.md#3144-the-mechanical-shorthand-rewrite-is-withheld) withholds the sibling's. The scope still leads the message and the attribution
  still trails it ([§FS-check.3.14.6](FS-check.md#3146-the-scope-leads-the-message));
- an owned site whose cited section does not resolve against the owner [§FS-fmt.2.4](FS-fmt.md#24-shorthand-to-canonical)
  picked for it, which that rewrite now leaves byte-identical
  ([§FS-fmt.2.4.6](FS-fmt.md#246-an-absent-target-section-withholds-this-rewrite-and-only-this-one)). The error itself
  still fires, in the wording [§FS-check.3.24.3](FS-check.md#3243-an-absent-target-section-is-answered-with-the-escape) gives it, which says on its own line that the
  owner has no such section — so a site the formatter refused is told from one it repaired by the
  finding itself. The `missing section` error beside it still fires as well
  ([§FS-check.3.2](FS-check.md#32-missing-section)), as the independent fact about the coordinate it is;
- and nothing else. A `[fmt] exclude` file ([§FS-fmt.2.5](FS-fmt.md#25-suppressed-scopes)), a `grund:fmt off` region, and an index
  or document reached as an external file-symlink target ([§FS-fmt.2.3.2](FS-fmt.md#232-a-link-that-leaves-the-config-root-is-not-written-through)) **keep** the clause,
  though `fmt` writes nothing there either. That is [§FS-check.3.13.1](FS-check.md#3131-where-the-text-forbids-the-rewrite)'s boundary, held to a second
  rule so the two do not disagree: the withholding is earned where nothing short of editing the
  citation itself would let `fmt` write the site, and not where the author's own act of
  configuration can lift the refusal. The text forbids its three contexts outright and an absent
  target section is further still from liftable — what is missing is not in the citation's
  neighbourhood at all but in the owner, so even moving the token out of its context leaves the
  rewrite wrong. A suppression the repository asked for the author can simply withdraw.

#### 3.24.2 An append, not a wording change

The clauses are **appended**: the text this rule shipped with survives as a verbatim contiguous
prefix, at the same offsets, on every one of the three shapes — on the owned shape, wherever the
owner has the cited section ([§FS-check.3.24.3](FS-check.md#3243-an-absent-target-section-is-answered-with-the-escape)). That is why the change needs no
deprecation path of its own under [§REQ-backwards-compatibility.1](../requirements/REQ-backwards-compatibility.md#1-what-is-covered) — the stable phrasing a tool
greps on is still there to grep, and a consumer matching a prefix, or matching
`code == "local-section-citation"`, reads exactly what it read before ([§FS-errors.3](FS-errors.md#3-message-text)). No verdict
moves with it: the same sites earn the finding, with the same code, severity, `sites`, count,
ordering, and exit code.

One consumer is affected, the one comparing a whole line for equality, and its migration is the
one [§FS-errors.3.6](FS-errors.md#36-the-agents-init-messages) already named for this project — the stable `code`. It is owed a window only
when a second wording change is coming, and none is: what ships is the final form, which is why
these clauses carry no `wording changes in <release>` deadline and schedule no removal release.

One exception was taken since, in place rather than through a window. Where the owner has no
section at the cited path, everything after `local section citation <token>; ` up to the release
attribution is replaced by [§FS-check.3.24.3](FS-check.md#3243-an-absent-target-section-is-answered-with-the-escape)'s text, because what it replaced was a citation the
same run reports missing: advice that fails when it is followed, which nothing that worked can
have depended on. The head through `local section citation <token>; ` and the attribution at the
end are kept there too, and so is every byte at a site whose section exists. Why that needed no
window either is [§DF-local-section-absent-target-escape](../decisions/functional/DF-local-section-absent-target-escape.md#df-local-section-absent-target-escape-an-owned-local-section-citation-whose-section-is-absent-is-answered-with-the-escape-in-place)'s.

#### 3.24.3 An absent target section is answered with the escape

Where the owner has no section at the cited path, the owned finding does not propose the full
citation of that coordinate, which would only trade this error for the `missing section` beside
it. It names the absence and offers the two remedies that clear the site, a full citation of
whichever document was meant or the escape for prose naming a file that declares no ID:

```text
local section citation <token>; <owner> has no section <path>, so write a full citation or <<marker>><tail> to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0
```

- **When.** Exactly where [§FS-check.3.2](FS-check.md#32-missing-section) reports `missing section` for the same coordinate on the
  same line, a stub's sections read as [§FS-check.3.2.1](FS-check.md#321-a-stubs-sections-are-its-targets-scanned-or-not) reads them; everywhere else the owned shape
  keeps its full-citation wording. The context does not matter — inline code, a link
  destination, a string literal, a `[fmt] exclude` file or a `grund:fmt off` region — and out
  past `[scan] include` under `--full` the scope still leads ([§FS-check.3.14.6](FS-check.md#3146-the-scope-leads-the-message)).
- **The words.** `<owner>` is the owner's ID as the first shape names it, `<path>` the section
  path `missing section` names after it, and `<tail>` the token with its marker taken off, as the
  other two shapes build it, so the escape is the token the author would type. A local path's
  components are always separated by `.` ([§FS-check.1.1.8](FS-check.md#118-declaration-local-numeric-section-candidates)), so `<path>` and `<tail>` are the same
  characters and `[id] section_separator` appears in neither; the escape carries the configured
  marker, `<@>5.1` under `@`.
- **The end.** The release attribution of [§FS-check.3.24.1](FS-check.md#3241-the-release-attribution-and-where-the-command-clause-is-withheld) comes last and nothing follows it: the
  command clause is never offered, because [§FS-fmt.2.4.6](FS-fmt.md#246-an-absent-target-section-withholds-this-rewrite-and-only-this-one) refuses the rewrite. Code, severity,
  location, count, ordering, exit code and the edge are those of an owned site whose section
  exists.

Line 3 of a use case whose `REFLECTION-jvm-reflection` has sections 1 and 2 only, where writing
`<§>5.1` as told clears both lines:

```text
usecases/reflection/jvm-reflection.md:3: error: local section citation §5.1; REFLECTION-jvm-reflection has no section 5.1, so write a full citation or <§>5.1 to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0
usecases/reflection/jvm-reflection.md:3: error: missing section REFLECTION-jvm-reflection.5.1
```

### 3.25 Invalid rule

A rule declaration whose title, rationale, vocabulary, named-section gate, or
exact literal subject is invalid produces `invalid-rule` at its heading. The
exact parse and resolution messages, and the pre-scan distinction for
`check --rule`, are [§FS-rules.4](FS-rules.md#4-validation-lifecycle) and
[§FS-rules.7.1](FS-rules.md#71-invalid-rule)'s.

### 3.26 Chapter cardinality

A `have` sentence whose named direct-chapter count is outside its constraint
produces `chapter-cardinality` at the subject declaration title, including
zero and surplus counts. Its fixed message is
[§FS-rules.7.2](FS-rules.md#72-chapter-cardinality)'s.

### 3.27 Citation cardinality

An outbound count not covered by [§FS-check.3.11](FS-check.md#311-missing-required-citation), and each off-count target of a
`cite each` sentence, produces `citation-cardinality` at the subject title.
Multiplicity, self-inclusion, per-target rows, message shape, and ordering are
[§FS-rules.3.2](FS-rules.md#32-outbound-citation-count),
[§FS-rules.3.3](FS-rules.md#33-per-target-coverage), and
[§FS-rules.7.3](FS-rules.md#73-outbound-citation-cardinality)'s.

### 3.28 Uncited unit

An inbound `be cited by` count outside its constraint produces `uncited-unit`
at the subject declaration or named-chapter title. Its count and message are
[§FS-rules.3.4](FS-rules.md#34-inbound-citation-count-and-prohibition) and
[§FS-rules.7.4](FS-rules.md#74-inbound-citation-cardinality)'s.

### 3.29 Unlisted `[workspace]` block

A directory that declares `[workspace]` and that **no enclosing `[workspace]` block lists among its `members`** is claimed by nobody: the enclosing project's scan absorbs its subtree when it reaches it, while a run started *at* it names every project from itself ([§FS-workspace.6.1.8](FS-workspace.md#618-a-block-no-enclosing-block-lists-is-outside-the-chain)). The two scopes then spell the same projects differently — `c/FS-c` inside the block, `root/FS-c` at the repository root — so a citation passes the inner check and fails the run CI does, which is [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration) failing in the one place the alias-path model exists to hold it. Every run whose tree scan meets such a block says so. In `check` it is an **error**: it reaches exit `1` and prints on **stdout** behind the `<path>:<line>: error:` prefix of [§FS-check.2.1](FS-check.md#21-report-format), located at the block's `[workspace]` line ([§FS-check.3.29.13](FS-check.md#32913-in-check-one-of-the-reports-errors)) — the deprecation ramp the warning announced, landing on the release it named ([§FS-check.3.29.14](FS-check.md#32914-an-error-because-the-deadline-the-warning-named-has-arrived)). On the five other scanning surfaces, which have no error channel for a fact about the run, it stays one CLI-level `warning:` on **stderr** ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages), [§FS-errors.2.2](FS-errors.md#22-cli-level-message)). Decided in [§DF-unlisted-workspace-block](../decisions/functional/DF-unlisted-workspace-block.md#df-unlisted-workspace-block-an-unlisted-workspace-block-is-reported-by-the-walk-that-meets-it).

What counts is [§FS-check.3.29.1](FS-check.md#3291-what-counts), and its edges are [§FS-check.3.29.2](FS-check.md#3292-an-unanswered-claim-is-not-reported-and-is-asked-quietly) to [§FS-check.3.29.5](FS-check.md#3295-include_root--false-changes-nothing). The message is [§FS-check.3.29.6](FS-check.md#3296-the-message) to [§FS-check.3.29.8](FS-check.md#3298-the-second-remedy-is-an-outcome-not-a-key); which commands report it, over what scan, is [§FS-check.3.29.9](FS-check.md#3299-every-command-that-scans-reports-it-not-check-alone) to [§FS-check.3.29.12](FS-check.md#32912-an-unlisted-block-outside-the-scan-is-unreported); its severity and rendering are [§FS-check.3.29.13](FS-check.md#32913-in-check-one-of-the-reports-errors) to [§FS-check.3.29.15](FS-check.md#32915-every-frontend-renders-it-located-at-the-blocks-workspace-line).

- **Code:** `unlisted-workspace-block` ([§FS-errors.5](FS-errors.md#5-json-format)).

#### 3.29.1 What counts

A directory the run's own scan reached, carrying a config under either discovery name ([§FS-config.1](FS-config.md#1-file-location-and-discovery)), whose config declares a `[workspace]` table, and whose canonical root no `[workspace]` block above it names among its `members` or `optional_members` entries. The claim question is the ancestor climb the claimed chain already runs ([§FS-workspace.6.1.7.2](FS-workspace.md#6172-the-quiet-climb-asks-the-same-ancestors)) — asked of a directory the scan found rather than of the run's own root — so it is read from `members` and `optional_members` entries alone and it climbs past the run's root exactly as that climb does. Both discovery names are probed at each scanned *directory*, which is what finds the `.agents/grund.toml` form: the scan never descends into a hidden directory ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)), so watching instead for scanned *files* named `grund.toml` would find half the blocks and call the other half claimed. A tree with no enclosing `[workspace]` block anywhere above it is the same case rather than a milder one — nothing claims the block, so nothing gives the projects under it a stable alias path, and the enclosing scan absorbs them just the same.

#### 3.29.2 An unanswered claim is not reported, and is asked quietly

An ancestor that *names* the candidate among its `members` and then cannot answer it — a member list that will not expand, a config that will not parse — leaves the claim unanswered ([§FS-workspace.6.1.8.1](FS-workspace.md#6181-two-shapes-stay-unreported)) and the block unreported, because no answer is not the answer that nothing claims it. That silence has a floor: the claim is read off the entry text before anything is expanded, so a block no ancestor *names* is never silenced by an ancestor's breakage, however broken that ancestor is. And the question is asked **quietly** — [§FS-workspace.6.1.7.5](FS-workspace.md#6175-an-unobtainable-members-value-leaves-the-claim-undecidable)'s undecidable-claim warning belongs to the climb that spells an alias path out of the chain, which this rule does not do, so a run that would otherwise never ask the chain anything gains no line from having asked.

#### 3.29.3 Three neighbouring shapes are not this finding

A nested directory carrying a plain `grund.toml` with no `[workspace]` table declares no projects to absorb: it is ordinary tree to the enclosing scan ([§FS-check.1.3.1](FS-check.md#131-the-scan-covers-the-whole-config-root)) and nothing reports it. A block that *is* listed is inside the claimed chain at every depth, whatever the nesting. And **a project root of this run is never a candidate** — the run's own root, and each member root the scan stops at ([§FS-workspace.6](FS-workspace.md#6-nested-project-boundary)) — because those are the scopes the run names everything else from, and a block absorbs nothing into a project it is itself the root of. Without that exemption the rule would fire on the run's own root the moment `--full` made it a scan root ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)), which is every workspace repository that sits under no enclosing one — that is to say, almost all of them.

#### 3.29.4 Only the outermost block of a chain

A `[workspace]` block below an unlisted one *is* claimed — by the unlisted block — so the claim test answers it on its own, and listing the outer block puts the whole chain back in the claimed chain. One finding for one edit. One block the scan reached twice — under its own path and under a directory symlink to it — is one finding for the same reason: the claim test resolves both spellings to one root, one edit clears both, and the spelling reported is the first the scan met. Two unlisted blocks neither of which lists the other are two findings, because they are two edits.

#### 3.29.5 `include_root = false` changes nothing

The key answers "is this block's root a project?"; the finding asks "does anything claim this block?". Same finding, same message. What that key costs the block's *own* files is [§FS-check.4.10](FS-check.md#410-include_root--false-leaves-the-blocks-own-files-unread), a separate finding on a separate condition: the two can fire on one block, because being claimed by nobody and being read by nobody are different holes with different repairs.

#### 3.29.6 The message

The message carries the block, what the absorption costs, the two config edits that clear it, and the release it became an error in. In `check` the location is the prefix and the text opens at the finding ([§FS-check.3.29.7](FS-check.md#3297-the-location-sits-in-the-prefix-where-the-finding-is-located-and-inside-the-text-where-it-is-not)):

```
b/grund.toml:3: error: this [workspace] is listed by no enclosing workspace — the projects under it are absorbed into `root` instead of named under their own alias path; add "b" to [workspace] members in grund.toml, or keep it out of that project's [scan] — an unlisted [workspace] became an error in grund 0.15.0
```

On the five other scanning surfaces of [§FS-check.3.29.9](FS-check.md#3299-every-command-that-scans-reports-it-not-check-alone) the same sentence keeps the location inside it, on stderr:

```
warning: b/grund.toml:3: this [workspace] is listed by no enclosing workspace — the projects under it are absorbed into `root` instead of named under their own alias path; add "b" to [workspace] members in grund.toml, or keep it out of that project's [scan] — an unlisted [workspace] became an error in grund 0.15.0
```

Two texts for two channels, and one word of the sentence is all the flip changed in either: `becomes` became `became` ([§FS-distribution.4.2](FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)).

#### 3.29.7 The location sits in the prefix where the finding is located, and inside the text where it is not

Where the finding is located — `check`'s report error ([§FS-check.3.29.13](FS-check.md#32913-in-check-one-of-the-reports-errors)) — the location is the bare `<path>:<line>:` prefix every other [§FS-check.3](FS-check.md#3-errors-detected) error wears, and it is not repeated inside the message text. That is [§FS-errors.2.2.1](FS-errors.md#221-a-location-inside-the-message-text)'s rule followed rather than an exception to it: a line opening with that prefix is the signal of a per-site finding on stdout, which is now what this is. Where the finding is *not* located — the CLI-level `warning:` the five other surfaces print — the location sits *inside* the message text, the shape [§FS-check.4.3.1](FS-check.md#431-a-cli-level-warning) and [§FS-config.4.3](FS-config.md#43-invalid-config-behavior) already use and the shape the undecidable-claim warning of [§FS-workspace.6.1.7.5](FS-workspace.md#6175-an-unobtainable-members-value-leaves-the-claim-undecidable) already prints for a neighbouring fact about a `[workspace]` block. `<line>` is the block's `[workspace]` line either way: the reader has two files to open and this is the one that is wrong. Every path in the line is rendered against the run's report base ([§FS-errors.3](FS-errors.md#3-message-text)), and `"b"` is the block's directory relative to the enclosing project's root, which is where both remedies are written. The absorbing project is named by the alias path this run spells it with, so the message can be matched against what [§FS-list](FS-list.md#fs-list-grund-lists-every-declared-id) printed.

#### 3.29.8 The second remedy is an outcome, not a key

The second remedy is stated as an outcome rather than as a key because which key carries it depends on the tree: `[scan] exclude` prunes descendants and never the directory a scan starts at ([§FS-check.1.3.2](FS-check.md#132-the-wider-scan-reads-a-superset-each-file-once)), so a block that is itself an `include` root leaves `include` as the edit, and a block below one takes `exclude`. Naming a key that clears the finding in one shape and not the other would be a remedy the reader has to argue with.

#### 3.29.9 Every command that scans reports it, not `check` alone

Every command whose run scans a project tree reports it: `check`, [§FS-list](FS-list.md#fs-list-grund-lists-every-declared-id), [§FS-refs](FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id), [§FS-cover](FS-cover.md#fs-cover-grund-groups-citations-by-scanned-file), [§FS-fmt](FS-fmt.md#fs-fmt-grund-normalizes-citations-in-bulk), and the ID read of [§FS-show](FS-show.md#fs-show-grund-reads-a-single-declaration-body-by-id) — as an error in `check`, the one of the six with an error channel and the one where the alias-path guarantee is gated, and as the `warning:` line of [§FS-check.3.29.6](FS-check.md#3296-the-message) on the other five, whose exit codes [§FS-cli.5](FS-cli.md#5-exit-code-mapping-is-fixed) freezes and whose business a fact about the run is not. That is a deliberate choice, because [§FS-check.4.3.2](FS-check.md#432-reported-by-check-config-validate-and-config-show) draws the opposite line for the redundant config pair, and three reasons make it. The absorbed spelling is what `list` prints, so a `list` that shows `root/FS-c` where the block shows `c/FS-c` and says nothing is the same silence the finding exists to break. `refs` and `fmt --cross-refs` resolve qualified citations against the same project map, so they are equally wrong under an absorbed block. And [§FS-check.4.3.2](FS-check.md#432-reported-by-check-config-validate-and-config-show)'s line does not reach this fact: a redundant config pair is about *which file the run read*, while this is about *how every command in the tree spells its projects*, which is not a question only `check` asks.

#### 3.29.10 Knowable only from the scan

This finding differs from the neighbouring `[workspace]` cautions in one respect: a fact knowable at workspace-boundary population is knowable before any scan, so every command that merely *loads* the workspace can carry it. This one is knowable only from the scan that meets the nested config, so it is a property of **what the run scanned** — carried by every command that scans, and silent wherever the scan does not reach the block. That is the earliest point at which the fact exists, not a second rule picked for convenience.

#### 3.29.11 The scope is the scan the run already makes

The roots `[scan] include` and the scanned `[[kinds]]` homes give it, and their subtrees, minus member boundaries and what `[scan] exclude`, the ignore files, and hidden directories prune below a root — never a root itself ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned), [§FS-workspace.6](FS-workspace.md#6-nested-project-boundary)). No second scan is made for config files: the entries are already being enumerated, and the added work is one config probe per scanned directory, which is what keeps [§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible) affordable here. It is also exactly the tree that gets absorbed — a block the scan never reaches absorbs nothing into the enclosing project.

#### 3.29.12 An unlisted block outside the scan is unreported

An unlisted block outside the run's scan is still unreported, a real limitation recorded rather than papered over. A narrowed `grund check <path>` ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)), a directory `[scan] exclude` prunes, a gitignored one, and a subtree behind a member boundary all leave a block unmet, and a run that cannot see something does not judge it — the same stance [§FS-check.3.18.8](FS-check.md#3188-a-run-that-cannot-see-the-index-does-not-judge-it) takes for an index the run did not scan. `grund check --full` widens the scan, so it reaches blocks the plain run does not and may report one more; that is the flag being additive ([§FS-check.1.3.4](FS-check.md#134-purely-additive)) about a caution rather than about a located finding, and it is the same edit [§FS-check.1.3.1](FS-check.md#131-the-scan-covers-the-whole-config-root) already recommends for a vendored or example project sitting inside the config root.

#### 3.29.13 In `check`, one of the report's errors

In `check` it is one of the report's errors. Like every error it reaches the exit code — a run whose scan meets an unlisted block exits `1` ([§FS-check.3](FS-check.md#3-errors-detected)) — and like every finding it stands in place of the `success` line ([§FS-check.2.1.3](FS-check.md#213-the-success-line)), which it already displaced as a warning, so nothing moves for a repository gating on that word. It is a **located** finding: it prints on **stdout** behind the `<path>:<line>: error:` prefix [§FS-check.2.1](FS-check.md#21-report-format) makes mandatory, and under `--format=json` it is one finding on stdout whose `path` is the block's own config and whose `line` is its `[workspace]` line, with `sites` `null` ([§FS-errors.5](FS-errors.md#5-json-format)). Those two fields were `null` for as long as this was a CLI-level warning; populating them is the schema move [§DF-unlisted-workspace-error-shape](../decisions/functional/DF-unlisted-workspace-error-shape.md#df-unlisted-workspace-error-shape-check-and-the-editor-take-the-located-shape-the-five-walking-surfaces-keep-the-cli-level-line) argues under [§REQ-backwards-compatibility.4](../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise). `--ignore unlisted-workspace-block` suppresses the finding and with it the exit code, because selection applies to an error exactly as it did to the warning ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)); the `code` is unchanged, because a code identifies a rule and does not travel with a severity.

#### 3.29.14 An error, because the deadline the warning named has arrived

No `grund` command writes either remedy — both are config edits and a judgement about which one the repository wants — so this rule never had [§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)'s licence for a same-release verdict flip, and took [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)'s deprecation path instead. It arrived in `0.13.0` as a warning whose own text named the release it would become an error in, and `0.15.0` is that release, reached. So the finding is an error like every other in this section — it contributes to the exit code ([§FS-check.3](FS-check.md#3-errors-detected)) and it stands in place of the `success` line ([§FS-check.2.1](FS-check.md#21-report-format)). The ramp bought what it was for: every `0.14.x` binary told a repository with an unlisted block, in the tool's own output, which release would start failing it. The same ramp, for the same reason, as [§FS-check.3.18.9](FS-check.md#3189-an-error-because-the-deadline-the-warning-named-has-arrived) — argued in [§DF-index-compatibility-ramp](../decisions/functional/DF-index-compatibility-ramp.md#df-index-compatibility-ramp-a-findings-ramp-follows-its-fix-command-not-the-size-of-the-offence).

The deadline clause is spent: a release still ahead is a date a reader can act on, and one that has arrived is not. What replaces it reports rather than promises, `— an unlisted [workspace] became an error in grund 0.15.0`, which is the past-tense half of the closed vocabulary [§FS-distribution.4.2](FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name) defines. It stays on both texts of [§FS-check.3.29.6](FS-check.md#3296-the-message) — the error's and the five surfaces' warning — because that clause is the only record of the flip a reader meets without the changelog, and it is what lets the release path refuse a version this finding would contradict. The ramp constant that held the named release ahead of the running version goes with the promise, and so does the unit test that read it: a test can hold only the pending half ([§FS-distribution.4.2.1](FS-distribution.md#421-a-test-can-hold-only-the-pending-half)), so what holds the landed half is an assertion on the clause's own bytes.

#### 3.29.15 Every frontend renders it, located at the block's `[workspace]` line

In `check` it is a located report error ([§FS-check.3.29.13](FS-check.md#32913-in-check-one-of-the-reports-errors)), and the location is the finding's own field: `path` is the block's config, `line` is its `[workspace]` line. **The anchor is the one thing the flip did not move** — it is the same `<path>:<line>` the warning named, and it is still carried as a field rather than left to be read back out of message text. What moved is which channel carries it.

On the five other surfaces of [§FS-check.3.29.9](FS-check.md#3299-every-command-that-scans-reports-it-not-check-alone) the finding travels in the run's warning channel and states its location *inside* the text ([§FS-check.3.29.7](FS-check.md#3297-the-location-sits-in-the-prefix-where-the-finding-is-located-and-inside-the-text-where-it-is-not)) and nowhere else: the CLI is the only frontend those five have, and it prints the sentence rather than placing it, so a location field there would be carried for nobody. The editor therefore takes this finding from `check`'s report, which is where the anchor is, and publishes it as an **error** on that `[workspace]` line ([§FS-lsp.1.1.3](FS-lsp.md#113-workspace-warnings-on-the-runs-warning-channel), [§FS-lsp.4](FS-lsp.md#4-determinism-and-parity-with-the-cli)) — one text and one location reaching every frontend from one place ([§FS-distribution.3.1](FS-distribution.md#31-rust-grund-core-crate)), as they already did, with the channel changed underneath. The three sibling `[workspace]` cautions stay on the run's channel and keep their anchors there, having no error channel to move into.

### 3.30 A workspace member swallows the block's own scan

A `[workspace]` block every one of whose scan roots lies inside one of its own members ([§FS-workspace.2.1](FS-workspace.md#21-a-member-that-swallows-the-blocks-own-scan)) reads nothing at all: its declarations are unreachable and its citations are never checked. Its configuration fails to load with the error [§FS-config.4.3](FS-config.md#43-invalid-config-behavior) defines for a bad `members` line — one `error:` line on stderr whose text is [§FS-check.3.30.1](FS-check.md#3301-the-message), nothing on stdout, exit `2` — from every command that loads the workspace ([§FS-check.3.30.2](FS-check.md#3302-every-command-that-loads-the-workspace-refuses)), with every block spelled from the run's own root ([§FS-check.3.30.3](FS-check.md#3303-one-run-spells-the-whole-tree-from-one-place)). No scan follows it ([§FS-check.3.30.4](FS-check.md#3304-no-empty-scan-caution-follows-it)), and it stays text under `--format json` ([§FS-check.3.30.5](FS-check.md#3305-it-stays-text-under---format-json)).

From grund 0.13.0 through 0.15.x the same fact was a `warning:` that left the exit code alone and named the release it would become an error in, the deprecation path of [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path), because no `grund` command repairs it. That release was 0.16.0, and the message now reports it in the past tense ([§FS-distribution.4.2](FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)). Decided in [§DF-absorbed-scan-warning](../decisions/functional/DF-absorbed-scan-warning.md#df-absorbed-scan-warning-a-scan-its-own-members-swallowed-is-a-warning-with-a-named-release-not-an-error); the flip's release record is [§DF-absorbed-scan-warning.release-note](../decisions/functional/DF-absorbed-scan-warning.md#release-note-release-note).

- **Code:** none. Like every config error it carries no JSON `code` and no selector ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)), so `--ignore` cannot reach it.

#### 3.30.1 The message

The line is located at the block's `members` line the way every config error is ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)), then names each covered root and the member entry it is inside, in config order. The member entry is named **as the config wrote it**; the covered root is named by its path **under the block root**, which is that spelling normalized rather than the spelling itself — an `include = ["./docs/"]` entry is named `docs`. Neither is the resolved path: that renders as nothing when it equals the render base and as an absolute path when it does not, and an author can edit neither ([§FS-errors.4](FS-errors.md#4-determinism)):

```
error: grund.toml:16: [workspace] members swallows this project's whole scan — every scan root is inside a member: `docs` in `docs` — so its declarations are unreachable and its citations are never checked. Point [scan] include at a directory that is not a member, or set include_root = false; this became an error in grund 0.16.0
```

#### 3.30.2 Every command that loads the workspace refuses

The question is asked where a run populates a block's member boundary, so `grund check`, `list`, `refs`, `show`, `cover`, `fmt`, and every other command that resolves that boundary refuse with it. It is asked of every block in a nested tree against that block's own `members` line, and of the run's root block before its nested members expand, so when a later expansion refusal would also apply, this error is the one reported and the run stops there — [§FS-config.4.3](FS-config.md#43-invalid-config-behavior) reports the first problem only.

`grund init` is the exception. It expands the workspace above its target only to teach the alias set, and when that expansion fails it leaves its Workspace members section out and the error to `check` ([§FS-init.2.3.4.15](FS-init.md#23415-workspace-members)), so it exits as it otherwise would and says nothing about the block.

#### 3.30.3 One run spells the whole tree from one place

Every line is rendered against the root this run was launched at, like every finding from a block above the run's root ([§FS-errors.3](FS-errors.md#3-message-text)), and that base is the same for every block the run reaches — the ones above it, the one it is rooted at, and the ones below it alike. Most commands never have to think about it: a run narrowed into a member is re-rooted onto that member before it scans, so the top of the tree it expands *is* where it was launched.

#### 3.30.4 No empty-scan caution follows it

Before 0.16.0 a `check` over an absorbed block printed [§FS-check.2.2](FS-check.md#22-empty-scan)'s empty-scan caution after the warning. The run now stops at the configuration, before any scan, so there is no scan for that caution to describe.

#### 3.30.5 It stays text under `--format json`

It is a config error, raised before any report exists, so like every launch-time error it keeps its `error:` text line on stderr under `--format json` ([§FS-errors.5.2.2](FS-errors.md#522-launch-time-messages-stay-text)), as an invalid configuration's error does.

## 4. Warnings

**`4.8` is vacant.** The unlisted-`[workspace]` rule held that number until the ramp of [§FS-check.3.29.14](FS-check.md#32914-an-error-because-the-deadline-the-warning-named-has-arrived) ended and the finding became an error; it is [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) now, under the heading its verdict belongs to. Its siblings kept the numbers they had rather than closing the gap: a section number is an address inside this repository and not a promise outside it, so a vacancy costs a reader this sentence while renumbering `4.9` to `4.14` would have moved 371 citations in 152 files to say the same thing.

### 4.1 Unused declaration

An ID that is declared but never cited. Reported as a warning, not an error — newly declared IDs may not yet have citations. Warnings never affect the exit code ([§FS-check.2](FS-check.md#2-outputs)). A resolving shorthand counts as a citation ([§FS-check.4.1.1](FS-check.md#411-a-resolving-shorthand-counts)), a kind's own index entry does not ([§FS-check.4.1.2](FS-check.md#412-an-index-entry-does-not-count)), and `E2E` declarations are exempt ([§FS-check.4.1.3](FS-check.md#413-e2e-declarations-are-exempt)). A citation counts wherever the run read it, which under an explicit path is the whole project ([§FS-check.4.1.4](FS-check.md#414-a-citation-anywhere-in-the-resolution-scope-counts)).

#### 4.1.1 A resolving shorthand counts

A number-only shorthand citation that resolves counts here like any other citation ([§FS-check.1.2.4](FS-check.md#124-a-resolved-shorthand-is-a-real-edge)): a declaration abbreviated as `§FS-042` everywhere is cited, and reporting it as unused would state the opposite of the truth.

#### 4.1.2 An index entry does not count

A citation that is a kind's own **index entry** ([§FS-check.3.18.5](FS-check.md#3185-what-an-entry-is)) does not count here. An index names every declaration in its folder by construction, so counting its entries would leave every ID in an indexed folder permanently cited and delete the signal this warning exists to give ([§DF-index-not-an-inbound-citation](../decisions/functional/DF-index-not-an-inbound-citation.md#df-index-not-an-inbound-citation-an-index-entry-is-navigation-not-use)). The exclusion is exactly the entry: a citation in an index file of an ID whose home lies *outside* that folder is an ordinary citation and counts like any other **unless that exact site is the canonical link that enrolls an external source declaration** ([§FS-check.3.18.3](FS-check.md#3183-an-external-source-declaration-enrolls-by-its-canonical-link)). Other citations of the enrolled ID on the same page still count. `grund refs` is unaffected and still lists every index entry — they are real citations, and a reader asking who points at an ID wants to be told that its index does.

#### 4.1.3 `E2E` declarations are exempt

`E2E` declarations ([AR-scanner.6](../architecture/AR-scanner.md#6-e2e-case-declarations)) are exempt: an end-to-end case is exercised by being run, not by being cited, so a `§E2E-<name>` that nothing cites is not a warning. Every other kind is subject to this rule, including Markdown and JSON value declarations; the citation inside a recognized value binding counts as a use ([§FS-values.3.2](FS-values.md#32-recognized-text-contexts)). `grund list --unused` ([§FS-list](FS-list.md#fs-list-grund-lists-every-declared-id)) uses the same default signal and suppresses uncited `E2E` cases unless `E2E` is explicitly selected with `--kind` (including a multi-kind filter such as `--kind FS,E2E`).

#### 4.1.4 A citation anywhere in the resolution scope counts

A citation counts wherever the run read it, so a declaration cited only from a file outside an explicit path is cited and earns no warning: a path-scoped run's `declared but never cited` warnings are exactly the whole-project run's ([§FS-check.1.3.6.1](FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution)).

This is not the `--full` case of [§FS-check.1.3.5](FS-check.md#135-the-unused-declaration-warning-is-unchanged-out-there), which leaves the warning standing, and the difference is a contract rather than a preference. `--full` owes additivity ([§FS-check.1.3.4](FS-check.md#134-purely-additive)) and so may never retire an in-scope finding. A path scope owes nothing of the kind: it is a subset run by construction, and it counts an edge **exactly where `grund check .` counts it** and never anywhere else, so it cannot disagree with the run additivity protects. [§FS-check.1.3.5](FS-check.md#135-the-unused-declaration-warning-is-unchanged-out-there)'s own remedy — widen `include` so the citing file is governed, and the edge counts everywhere — is what a path scope already reads.

### 4.2 Inline note soft-cap overrun *(opt-in)*

Off by default. When `[reference] warn_on_suggested = true` is set in the project's `grund.toml` ([§FS-config.3.1](FS-config.md#31-reference--citation-form)), a note whose line count exceeds `inline_note_suggested_lines` but stays within `inline_note_max_lines` is reported as a warning. The full contract — what counts as a soft-cap overrun and how it interacts with the hard-cap error in [§FS-check.3.10](FS-check.md#310-inline-citation-style-violation) — lives in [§FS-inline-citation-style.4.2](FS-inline-citation-style.md#42-warnings--opt-in-soft-cap). Off by default because the soft cap is primarily agent-facing guidance ([§FS-inline-citation-style.5](FS-inline-citation-style.md#5-agent-facing-rendering)); flipping the toggle escalates it to a `check`-time signal.

### 4.3 Redundant config pair

A directory that carries both a bare `grund.toml` and `.agents/grund.toml` ([§FS-config.1.1](FS-config.md#11-when-one-directory-carries-both)). The bare file is the config; the `.agents/` one is read by nothing, so a user who edits it changes nothing and is told so, in a CLI-level warning ([§FS-check.4.3.1](FS-check.md#431-a-cli-level-warning)) that `grund config validate` and `grund config show` also emit ([§FS-check.4.3.2](FS-check.md#432-reported-by-check-config-validate-and-config-show)):

```
warning: .agents/grund.toml is ignored — grund.toml takes precedence; delete one
```

#### 4.3.1 A CLI-level warning

It is a CLI-level `warning:` on **stderr** ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)), not a per-finding line: it is about which file the run read, not a finding at a site in the citation graph, and there is no offending line to point at — the whole file is ignored. Both paths are report paths and follow `[output] relative_paths` ([§FS-config.3.6](FS-config.md#36-output--report-format)) — relative to the config root by default — so the message names the two files a user has to choose between. Like every warning it leaves the exit code alone ([§FS-check.2](FS-check.md#2-outputs)), because the pair is the ordinary transient state of a migration between the two forms ([§DF-config-file-location.2.2](../decisions/functional/DF-config-file-location.md#22-the-bare-grundtoml-wins-a-tie-and-check-warns-about-the-pair)).

#### 4.3.2 Reported by `check`, `config validate` and `config show`

The same warning is emitted by `grund config validate` and `grund config show` ([§FS-config.4.1](FS-config.md#41-grund-config-validate-path), [§FS-config.4.2](FS-config.md#42-grund-config-show-path)) — those are the surfaces a user reaches for when the answer to "why is my config not taking effect" is that `grund` is reading the other file. No other command reports it: a redundant pair is a fact about the repository's configuration, and `show`, `list`, `refs`, `cover`, and `fmt` answer questions about its content.

#### 4.3.3 The deprecated-location sibling

[§FS-check.4.11](FS-check.md#411-config-read-from-the-deprecated-agents-location) is this finding's sibling and inherits every sentence of this section: it fires where the run *read* the `.agents/` file rather than ignored it, which is the case this one cannot be in.

### 4.4 Inline note layout deviation *(opt-in)*

Off by default. When `[reference] inline_note_layout` names a layout and `[reference] inline_note_layout_check = "warn"` ([§FS-config.3.1](FS-config.md#31-reference--citation-form)), every line of a note that [§FS-inline-citation-style.3.3.1](FS-inline-citation-style.md#331-per-line-not-per-site--and-only-where-a-note-opens) judges and that does not match the configured form is reported as a warning, located at that line. Setting the key to `"error"` reports the identical message as an error instead ([§FS-check.3.10](FS-check.md#310-inline-citation-style-violation)); `"off"`, or a `inline_note_layout` left at `any`, reports nothing. The form itself, the per-line rule, and the exemption for sites that carry no note live in [§FS-inline-citation-style.3.3](FS-inline-citation-style.md#33-inline_note_layout--where-the-citations-sit), and the channel table in [§FS-inline-citation-style.4.4](FS-inline-citation-style.md#44-warnings-and-errors--opt-in-layout-deviations).

Off by default because a layout is a house style rather than a correctness property, and the two severities exist so a repository can migrate on `warn` before it gates on `error` — the same ladder [§FS-check.4.2](FS-check.md#42-inline-note-soft-cap-overrun-opt-in) gives the soft cap.

### 4.5 Nothing recognized

A scan that read at least one file and recognized **nothing in it** — no declaration and no citation — is [§FS-check.2.2](FS-check.md#22-empty-scan)'s empty scan one step further in: the scope was right and the files were read, and the grammar matched none of their content. `check` then emits one CLI-level `warning:` line ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)) on **stderr**, whose text is [§FS-check.4.5.2](FS-check.md#452-the-message). It has two usual causes ([§FS-check.4.5.1](FS-check.md#451-the-usual-causes)), is asked per project ([§FS-check.4.5.3](FS-check.md#453-asked-per-project)) and only of a run over that project's root ([§FS-check.4.5.4](FS-check.md#454-asked-only-of-the-whole-project)), and is withheld beside any other finding about the scope ([§FS-check.4.5.5](FS-check.md#455-a-warning-withheld-beside-any-other-finding-about-the-scope)); the per-heading half is [§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss)'s ([§FS-check.4.5.6](FS-check.md#456-the-per-heading-half)). Decided in [§DF-nothing-recognized](../decisions/functional/DF-nothing-recognized.md#df-nothing-recognized-a-run-that-recognized-nothing-says-so-and-says-it-as-a-warning).

- **Code:** `nothing-recognized` ([§FS-errors.5](FS-errors.md#5-json-format)), with `path` and `line` null like every CLI-level finding.

#### 4.5.1 The usual causes

The usual causes are a tree nobody has declared in yet and a docs tree whose headings open with a prefix that is no configured kind ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)), either of which leaves every heading in the tree a non-declaration and the run's verdict `success` over a repository where nothing is grounded.

#### 4.5.2 The message

The line names how many files were read, the shape a declaration heading and a citation take under the configured format, and the configured kinds:

```
warning: nothing recognized — grund read 3 files and found no declaration and no citation in them. A declaration heading reads `# <KIND>-<NNN>-<slug>: <title>` and a citation `<marker><KIND>-<NNN>-<slug>`, under [id] format = "{kind}-{number}-{slug}" with <KIND> one of {AR, FS}. Either nothing is declared yet, or the headings are written to a different shape than that.
```

The shapes are rendered from the `[id] format` template, the same substitution [§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints) makes for the managed entrypoint block, and the citation shape carries the configured marker ([§FS-config.3.1](FS-config.md#31-reference--citation-form)). The closing sentence offers both readings of the fact, because the run cannot tell them apart without judging a line: a tree written to another format and a `grund init` scaffold nobody has declared in yet produce the identical report, and naming only the first would send a fresh adopter looking for a bug in a config that is fine. No example ID is built from `[id] number_pattern` and `[id] slug_pattern` and no corrected ID is proposed for any heading: `check` reports facts about the tree and the config ([§FS-check.3](FS-check.md#3-errors-detected) vs [§FS-check.4](FS-check.md#4-warnings)), and an ID assembled from those patterns would be a guess at what they accept.

#### 4.5.3 Asked per project

The question is asked **per project**, like [§FS-check.2.2](FS-check.md#22-empty-scan): in a workspace ([§FS-workspace.5](FS-workspace.md#5-command-scope)) each project is judged against its own config, since one project's grammar mismatch says nothing about another's. It asks *recognized*, not *declared* — a member that only cites declarations from another member declares nothing and is working as intended, so a citation anywhere in the project answers the question.

#### 4.5.4 Asked only of the whole project

It is asked only of a run whose scope **is** that project's root (no path argument, or a path that resolves to it). A narrowed `grund check <dir>` is a slice the caller chose, and a slice holding no declaration and no citation is an answer rather than a misconfiguration — the claim this caution makes is about a whole project, and a run that read part of one cannot make it.

#### 4.5.5 A warning, withheld beside any other finding about the scope

Like [§FS-check.2.2](FS-check.md#22-empty-scan) it is a warning, and like [§FS-check.2.2](FS-check.md#22-empty-scan) it is withheld from a run that has any other finding about the default scope: the exit code stays `0` (a tree with nothing in it yet is the ordinary first day of a repository), and a report that already says something about that scope is not the silent verdict this rule exists to break. It inherits every exception [§FS-check.2.2.3.1](FS-check.md#2231-findings-that-do-not-suppress-it) lists unchanged, and for the same reasons. A redundant-config pair ([§FS-check.4.3](FS-check.md#43-redundant-config-pair)) is a fact about which file was read — and a repository mid-migration between the two config names is exactly where a mismatched `[id] format` hides, in the file that is no longer read. The out-of-scope finding ([§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only)) is a fact about the tree beyond the scope, and a `--full` run that reports every citation out there while the default scope holds nothing is the strongest form of this diagnosis, not a reason to withhold half of it. What it buys is the `success` line — a warning stands in its place ([§FS-check.2.1](FS-check.md#21-report-format)), so the run that recognized nothing stops printing the same word as the run that checked everything.

#### 4.5.6 The per-heading half

The per-heading half — naming each heading that looks like a declaration and does not match — is [§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss), a different rule asking a different question: this one is arithmetic over what the scan recorded, that one is about what a single line came close to being. Where both could speak, [§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss) does and this one is withheld under the rule above, because "these two headings, at these lines" is the same fact said usefully.

### 4.6 Declaration near miss

Moved to [§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss). This address is kept so citations written before the move still resolve.

### 4.7 A workspace member swallows the block's own scan

Moved to [§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan) when the warning became an error in grund 0.16.0. This address is kept so citations written before the move still resolve.

#### 4.7.9 A later expansion refusal does not discard it

Retired in grund 0.16.0: the absorbed scan is now a config error, so when a later expansion refusal would also apply, the run stops at the absorbed scan and reports only it ([§FS-check.3.30.2](FS-check.md#3302-every-command-that-loads-the-workspace-refuses)). This address is kept so citations written before the change still resolve.

### 4.9 A workspace member declared optional is absent

A member listed in `[workspace] optional_members` whose path is not a directory in this checkout ([§FS-workspace.2.2](FS-workspace.md#22-a-member-that-may-be-legitimately-absent)). What it would have contributed was not read: no declaration in it reached a catalog, and no citation into it was resolved or reported ([§FS-workspace.4](FS-workspace.md#4-resolution)). The run is a report about less of the repository than it looks like, and saying so is this finding's whole job.

It is one located warning per absent entry ([§FS-check.4.9.1](FS-check.md#491-one-warning-per-absent-entry)), naming the entry and its member ([§FS-check.4.9.2](FS-check.md#492-the-entry-as-written-the-member-by-its-alias-path)) and no remedy ([§FS-check.4.9.3](FS-check.md#493-it-names-no-remedy-because-nothing-here-is-broken)). Unlike [§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan) and [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) it is printed on stdout ([§FS-check.4.9.4](FS-check.md#494-a-located-finding-on-stdout-not-a-cli-level-caution)), because the exit code cannot carry it ([§FS-check.4.9.5](FS-check.md#495-the-exit-code-forces-the-shape)); it names no release ([§FS-check.4.9.6](FS-check.md#496-it-names-no-release)), and its JSON form is [§FS-check.4.9.7](FS-check.md#497-its-json-form-is-on-stdout).

- **Code:** `optional-member-absent` ([§FS-errors.5](FS-errors.md#5-json-format)).

#### 4.9.1 One warning per absent entry

One warning per absent entry, in the report's ordinary sort ([§FS-errors.4](FS-errors.md#4-determinism)) — by entry name, since all of them share one path and line — located at the `optional_members` line of the block that holds it:

```
grund.toml:5: optional workspace member `vendored` is absent — citations into namespace `vendored` were not checked, so this run does not cover it
```

#### 4.9.2 The entry as written, the member by its alias path

The entry is named **as the config wrote it** and the member by the **whole alias path this run spells it with** — `vendored` and `sub/vendored` for one entry inside a nested block — which is the pair [§FS-check.3.30.1](FS-check.md#3301-the-message) and [§FS-check.3.29.7](FS-check.md#3297-the-location-sits-in-the-prefix-where-the-finding-is-located-and-inside-the-text-where-it-is-not) already use, for the same two reasons: the entry is what an author can edit ([§FS-workspace.2](FS-workspace.md#2-workspace-configuration)), and the alias path is what a citation has to write ([§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)).

#### 4.9.3 It names no remedy, because nothing here is broken

Most warnings in [§FS-check.4](FS-check.md#4-warnings) end in an edit, because they report a configuration that says something its author did not mean. This one reports a state the author declared in advance and a checkout that happens to be partial; the only thing that would "fix" it is a checkout with the member in it, which is not grund's to ask for and is often not available where the run happens. So the message stops at the fact.

#### 4.9.4 A located finding on stdout, not a CLI-level caution

It is a located finding on stdout, which departs deliberately from its two nearest neighbours — [§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan) and [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) both point at a line of their block's config, and [§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan) prints as a CLI-level `error:` on stderr ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)) — as [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) still does on the five scanning surfaces that have no report to put it in ([§FS-check.3.29.9](FS-check.md#3299-every-command-that-scans-reports-it-not-check-alone)), though in `check` it is an error now ([§FS-check.3.29.13](FS-check.md#32913-in-check-one-of-the-reports-errors)). Those two report a **misconfiguration**, and the reason for that CLI-level form is that every command in the tree is wrong under them: a block that reads nothing makes `grund list` silent, an unlisted block makes every command spell the same project two ways. So both are carried by every command that scans — [§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan) refusing where the workspace loads, [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) where the scan meets the block ([§FS-check.3.29.10](FS-check.md#32910-knowable-only-from-the-scan)). This finding is not that. Nothing is misconfigured — the repository said this may happen, and it happened — and no other command's output is wrong, because a member that is not there has nothing to list and nothing to point at. What is at stake is only the verdict `check` renders, and a statement about the coverage of a report belongs in the report.

#### 4.9.5 The exit code forces the shape

[§FS-check.2](FS-check.md#2-outputs) gives grund one way to say "do not trust this report as complete", and it is exit `2`. This is the one case where a run is deliberately incomplete and still exits `0` ([§FS-workspace.2.2](FS-workspace.md#22-a-member-that-may-be-legitimately-absent)), so the exit code carries nothing here and stdout has to. A CLI-level line would leave stdout saying only that the `success` line was withheld — a signal made of an absence, which under `2>/dev/null` or a folded CI log is indistinguishable from silence, and which is exactly the warning that scrolls past. The `<path>:<line>:` prefix is also honest here in a way it is not for [§FS-check.4.3.1](FS-check.md#431-a-cli-level-warning): there is one line, in a file the repository wrote, and it is the line a reader has to open to understand the run. `success` is withheld all the same, because it is withheld for every warning ([§FS-check.2.1.3](FS-check.md#213-the-success-line)) — but that is now a consequence of the finding rather than the whole of it.

#### 4.9.6 It names no release

[§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan) and [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) were warnings on the way to being errors ([§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)), and both have become one ([§FS-check.3.29.14](FS-check.md#32914-an-error-because-the-deadline-the-warning-named-has-arrived)). This one is permanent. An unverified member is not a state to be migrated off; it is the standing price of the opt-out, paid on every run of every checkout that takes it, and a repository that stops wanting to pay it deletes the entry.

#### 4.9.7 Its JSON form is on stdout

Under `--format=json` it is one warning finding on **stdout** with `path` and `line` set and `sites` null, like every other located finding ([§FS-errors.5](FS-errors.md#5-json-format)) — which is the other half of what the CLI-level shape would have cost, since a consumer filtering the report for coverage facts would have had to parse text on a second stream to find this one.

### 4.10 `include_root = false` leaves the block's own files unread

A `[workspace]` block that sets `include_root = false` is not a project, and its own files are read by nobody ([§FS-check.4.10.1](FS-check.md#4101-nobody-reads-the-blocks-own-files)): a declaration there reaches no catalog and a citation there is never checked, which is [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration) failing through one config key. `grund` emits one CLI-level `warning:` ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)) on **stderr** for such a block **whose own tree actually holds a file a scan would have read**. Decided in [§DF-unread-opted-out-block](../decisions/functional/DF-unread-opted-out-block.md#df-unread-opted-out-block-the-unread-files-of-an-opted-out-block-are-a-conditional-warning-that-never-ramps).

What counts is [§FS-check.4.10.2](FS-check.md#4102-what-counts), and [§FS-check.4.10.3](FS-check.md#4103-a-block-with-no-member-in-scope-is-not-this-finding) and [§FS-check.4.10.4](FS-check.md#4104-what-stays-silent-and-why-the-silence-is-the-point) are what does not. The message is [§FS-check.4.10.5](FS-check.md#4105-the-message), and it names one root ([§FS-check.4.10.6](FS-check.md#4106-one-root-not-every-root)). Every command that scans says it, once per block ([§FS-check.4.10.7](FS-check.md#4107-every-command-that-scans-says-it-and-each-block-says-it-once)), except over a failed workspace expansion ([§FS-check.4.10.8](FS-check.md#4108-a-failed-workspace-expansion-withholds-it)). It keeps its text under `--format json` ([§FS-check.4.10.9](FS-check.md#4109-a-launch-time-message-keeps-its-text-under---format-json)), stands in place of `success` without moving the exit code ([§FS-check.4.10.10](FS-check.md#41010-it-stands-in-place-of-the-success-line-and-never-moves-the-exit-code)), reaches every frontend ([§FS-check.4.10.11](FS-check.md#41011-one-of-the-runs-warnings-rendered-by-every-frontend)), and never becomes an error ([§FS-check.4.10.12](FS-check.md#41012-a-warning-permanently-with-no-release-it-becomes-an-error-in)).

#### 4.10.1 Nobody reads the block's own files

An intermediate opted-out block is still a segment in every alias path below it ([§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)), yet its own files are read by nobody: it has no scan of its own, an enclosing scan, where there is one, stops at the member boundary ([§FS-workspace.6](FS-workspace.md#6-nested-project-boundary)), and `--full` ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)) widens a project's scope and has no project to widen here. No run said so: over a grouping directory holding two dangling citations, `check`, `check --full` and `list` were all silent and exited `0` ([grund#71](https://github.com/agent-grounds/grund/issues/71)).

#### 4.10.2 What counts

**What counts is one question: would this block have read something, had it been a project?** That is the mirror of [§FS-workspace.2.1](FS-workspace.md#21-a-member-that-swallows-the-blocks-own-scan), which asks whether a block that *is* a project reads nothing — so one notion of "this block's own scope" answers both, and a `[[kinds]]` home or an unscanned home moves the two rules together. Take the block's **default scope** — the roots `[scan] include` and the scanned `[[kinds]]` homes give it ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)), the set [§FS-workspace.6](FS-workspace.md#6-nested-project-boundary)'s boundary prunes, asked of the default scope whatever `--full` says, because this is a property of the configuration and not of one scan. Drop a root that is not on disk: the scan skips it before it prunes, so it is read by nobody and costs nobody anything. Drop a root at or inside one of the block's own expanded member roots: those files *are* read, by the member — compared as canonical paths, the way the scan's own prune compares them. On what is left, probe for one file the scan would have read, under the block's own `[scan] extensions`, `exclude`, ignore files and hidden-name rules applied exactly as the scanner applies them ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)). The probe **stops at the first hit**: the cost is "is there one file here", not the size of the tree, and only a block that opted out ever pays it ([§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible)).

#### 4.10.3 A block with no member in scope is not this finding

`include_root = false` with no members — an absent `members` key, an empty list, or a non-empty list of globs that match no directories, beside no `optional_members` entry ([§FS-workspace.2.2](FS-workspace.md#22-a-member-that-may-be-legitimately-absent)) — is already a config error at that block's own line ([§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)). A configuration the run refuses is not one it also cautions about, and the caution's two remedies are not the repair that block needs.

#### 4.10.4 What stays silent, and why the silence is the point

Four shapes are deliberately not this finding, and each is a correct configuration that [§FS-check.4.10.2](FS-check.md#4102-what-counts) answers "no" for. A grouping directory that only groups — `grund.toml` and its members and nothing else — has no root on disk to read. A block whose own roots are all inside its members is [§FS-workspace.2.1](FS-workspace.md#21-a-member-that-swallows-the-blocks-own-scan)'s shape read from the other side. A tree whose only files are of unscanned types, or excluded, gitignored, or hidden, is a tree the block would not have read as a project either. And a root the config names that is not on disk rescues nothing. Firing on any of them would be a warning with no edit that clears it, which is permanent output tools learn to filter — the outcome [§DF-absorbed-scan-warning](../decisions/functional/DF-absorbed-scan-warning.md#df-absorbed-scan-warning-a-scan-its-own-members-swallowed-is-a-warning-with-a-named-release-not-an-error) already rejected once for a neighbouring finding.

#### 4.10.5 The message

The message carries the config line the reader should open, the tree that is unread, what that costs, and the two remedies:

```
warning: group/grund.toml:17: no project scans `docs`, so its citations are never checked. Set include_root = true, or point another project's [scan] include at it.
```

The breadcrumb is the block's own `include_root` line — the key that decided is the line to open — falling back to its `[workspace]` line if the key is absent, which the default `true` makes unreachable. It renders against the root this run was launched at, like every finding about a block above or below it ([§FS-errors.3](FS-errors.md#3-message-text)). The unread tree is named by its path **under the block root**, that spelling normalized rather than the spelling itself, exactly as [§FS-check.3.30.1](FS-check.md#3301-the-message) names a covered root, and never by the resolved path, for [§FS-check.3.30.1](FS-check.md#3301-the-message)'s reason.

#### 4.10.6 One root, not every root

[§FS-check.3.30.1](FS-check.md#3301-the-message) lists every covered root because its claim is universal — *every* one is inside a member — and the list is the evidence for it. This claim is existential: one unread root is the whole finding, one edit clears all of them, and probing the rest would buy nothing the answer depends on. The one named is the first in scope order, `[scan] include` in config order and then the `[[kinds]]` homes, which is fixed; the first *file* under it is whatever the filesystem handed back, which is not, and is therefore never named ([§FS-errors.4](FS-errors.md#4-determinism)).

#### 4.10.7 Every command that scans says it, and each block says it once

The question is asked where a run populates a block's member boundary — the block the run is rooted at, and each block below it — so `check`, [§FS-list](FS-list.md#fs-list-grund-lists-every-declared-id), [§FS-refs](FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id), [§FS-cover](FS-cover.md#fs-cover-grund-groups-citations-by-scanned-file), [§FS-fmt](FS-fmt.md#fs-fmt-grund-normalizes-citations-in-bulk) and every other command that resolves that boundary carry it, on exactly the surfaces that load the workspace ([§FS-check.3.30.2](FS-check.md#3302-every-command-that-loads-the-workspace-refuses)). The reproduction in the ticket used `check` and `list`, which is the whole argument: a silent-scope defect only `check` reports is half-reported. It is **once per block per run** — a workspace-wide run that expands the same block a second time still says it once — and a tree with two opted-out blocks earns one line each, the block the run is rooted at first and the blocks below it after, in the order the run reaches them. A run narrowed inside a member never populates the enclosing block's boundary, so it stays silent about a block it is not reading through.

#### 4.10.8 A failed workspace expansion withholds it

A run whose **workspace expansion fails** does not carry this finding for the blocks below its root, a silence declared and bounded ([§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded)) rather than a gap: the answer depends on where the run's other projects are, and a failed expansion is exactly the case where that list was never produced, so the alternative is not an earlier warning but a wrong one. The run is exiting `2` on a config error the reader has to repair before anything else it says is worth reading, and the finding returns on the next run. [§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan) is asked of the run's root block before that expansion, because what it asks is answered by the block's own members alone, so where it applies it is the error the run stops at ([§FS-check.3.30.2](FS-check.md#3302-every-command-that-loads-the-workspace-refuses)).

#### 4.10.9 A launch-time message keeps its text under `--format json`

It is a launch-time message, so it keeps its text under `--format json` ([§FS-errors.5.2.2](FS-errors.md#522-launch-time-messages-stay-text)) and carries no JSON `code` and no selector of its own ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)). That is the rendered shape of every launch-time message, [§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan)'s error included ([§FS-check.3.30.5](FS-check.md#3305-it-stays-text-under---format-json)), rather than [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block)'s, and the difference between them is *when the fact exists*. [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block)'s is knowable only from the scan that meets a nested config ([§FS-check.3.29.10](FS-check.md#32910-knowable-only-from-the-scan)), so it is one of `check`'s report errors and renders as a JSON finding with `path` and `line` set and `sites` null ([§FS-check.3.29.13](FS-check.md#32913-in-check-one-of-the-reports-errors)). This one is settled at boundary population, before a scan and before a report exists, and is carried by every command that resolves that boundary ([§FS-check.4.10.7](FS-check.md#4107-every-command-that-scans-says-it-and-each-block-says-it-once)), not by `check` alone — so the shape it renders in is fixed here rather than borrowed from `check`'s, and giving `check` a different one would make the text a consumer greps for depend on which command produced it.

#### 4.10.10 It stands in place of the `success` line, and never moves the exit code

[§FS-check.2.1.3](FS-check.md#213-the-success-line) is unchanged: a run with a warning prints the warning rather than `success`. This is the first `[workspace]` caution that has to say so out loud, because it is the first that can fire on an otherwise clean run — [§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan)'s block earned the empty-scan caution beside it while it was a warning ([§FS-check.2.2](FS-check.md#22-empty-scan)) and stops the run now ([§FS-check.3.30.4](FS-check.md#3304-no-empty-scan-caution-follows-it)), and [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block)'s is in the report already, as an error ([§FS-check.3.29.13](FS-check.md#32913-in-check-one-of-the-reports-errors)). The exit code stays where it was ([§FS-check.2](FS-check.md#2-outputs)): opting out is a legitimate choice, and [§FS-non-goals.9](FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization) fixes what a warning means.

#### 4.10.11 One of the run's warnings, rendered by every frontend

The fact is settled before any report exists, but what the engine hands back is a finding in the run's warning channel — carried on whatever the scanning command returns and rendered by whichever frontend asked ([§FS-distribution.3.1](FS-distribution.md#31-rust-grund-core-crate)) — rather than a line the engine wrote to a stream. It **is located at the block's `include_root` line**, falling back to its `[workspace]` line exactly as the breadcrumb of [§FS-check.4.10.5](FS-check.md#4105-the-message) does, and carries that anchor as the finding's own location so a frontend never parses the message for one. Every byte is the byte it is today, on stderr and under `--format json` alike, and the `success` line it stands in place of is unaffected. The editor is the surface that gains a reader: it publishes the warning on that `include_root` line ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)), which is where the one edit that clears it is written.

#### 4.10.12 A warning permanently, with no release it becomes an error in

This is one of the two `[workspace]` findings that do not ramp, beside [§FS-check.4.9](FS-check.md#49-a-workspace-member-declared-optional-is-absent) ([§FS-check.4.9.6](FS-check.md#496-it-names-no-release)), and the difference is not how wrong the repository is. A finding is eligible to become an error only when **every repository in that state is wrong** *and* **the configuration has a way to say "I meant it"** — the rule for all four, argued in [§DF-unread-opted-out-block.2.3](../decisions/functional/DF-unread-opted-out-block.md#23-what-makes-a-workspace-finding-ramp-and-what-makes-one-permanent). [§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan) and [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) pass both: a block that claims to be a project and reads nothing, and a block whose projects are spelled two ways, are wrong on every reading of them, and each has an edit that records the intent instead — `include_root = false` for one, listing the block for the other — so an error is a verdict their author can act on before it lands, and [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)'s deprecation path has got both there ([§FS-check.3.29.14](FS-check.md#32914-an-error-because-the-deadline-the-warning-named-has-arrived), [§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan)). [§FS-check.4.9](FS-check.md#49-a-workspace-member-declared-optional-is-absent) fails the first: an absent optional member is a state its repository declared in advance ([§FS-check.4.9.3](FS-check.md#493-it-names-no-remedy-because-nothing-here-is-broken)). This finding passes neither. A grouping directory holding a README nobody needs checked is doing exactly what its author meant, and neither remedy records that: making the block a project and pointing another project's `[scan] include` at it both change what the repository *is*. That the finding is a property of the configuration **and** the tree — the same `grund.toml` silent on Monday and reportable on Tuesday because somebody added `group/docs/notes.md` — is the symptom that makes it recognizable, not the reason: [§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index) depends on the tree in the same way and ramped anyway. There is therefore no version constant, no roadmap milestone, and no clause in the message naming a release.

### 4.11 Config read from the deprecated `.agents/` location

The config this run read is an `.agents/grund.toml` ([§FS-config.1.2](FS-config.md#12-the-agents-location-is-deprecated)). The file is still read and still governs the project — the location is deprecated, not withdrawn — and the run says so, naming the file it read and the bare `grund.toml` beside it that should hold it instead:

```
warning: .agents/grund.toml is a deprecated config location — move it to grund.toml
```

It is [§FS-check.4.3](FS-check.md#43-redundant-config-pair)'s finding in everything but its trigger ([§FS-check.4.11.1](FS-check.md#4111-fs-check43s-finding-in-everything-but-its-trigger)), on the same three surfaces and no more ([§FS-check.4.11.2](FS-check.md#4112-the-same-three-surfaces-as-fs-check43-and-no-more)), once per config ([§FS-check.4.11.3](FS-check.md#4113-once-per-config-not-once-per-scope)), and never beside [§FS-check.4.3](FS-check.md#43-redundant-config-pair) about one directory ([§FS-check.4.11.4](FS-check.md#4114-a-directory-carrying-both-names-earns-fs-check43-and-not-this)). Its JSON form is [§FS-check.4.11.5](FS-check.md#4115-under---formatjson-one-warning-finding-on-stderr). It stands in place of the `success` line ([§FS-check.4.11.6](FS-check.md#4116-it-stands-in-place-of-the-success-line)) and never becomes an error ([§FS-check.4.11.7](FS-check.md#4117-a-warning-permanently-with-no-release-it-becomes-an-error-in)).

#### 4.11.1 [§FS-check.4.3](FS-check.md#43-redundant-config-pair)'s finding in everything but its trigger

This is [§FS-check.4.3](FS-check.md#43-redundant-config-pair)'s finding in everything but its trigger, and for [§FS-check.4.3.1](FS-check.md#431-a-cli-level-warning)'s reason: it is a fact about *which file the run read*, not about that file's content and not about a site in the citation graph. So it is a CLI-level `warning:` on **stderr** ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)) and never a per-finding line — there is no offending line to point at, the whole file is the subject. Both paths are report paths and follow `[output] relative_paths` as [§FS-check.4.3.1](FS-check.md#431-a-cli-level-warning)'s do ([§FS-config.3.6](FS-config.md#36-output--report-format)), so the message names the move as the `git mv` a reader can type from the report's base. Like every warning it leaves the exit code alone ([§FS-check.2](FS-check.md#2-outputs)), because the location it names is a supported one and [§FS-non-goals.9](FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization) fixes what a warning means.

#### 4.11.2 The same three surfaces as [§FS-check.4.3](FS-check.md#43-redundant-config-pair), and no more

`grund check`, `grund config validate` and `grund config show` ([§FS-config.4.1](FS-config.md#41-grund-config-validate-path), [§FS-config.4.2](FS-config.md#42-grund-config-show-path)) carry it, byte-identically. `list`, `refs`, `cover`, `fmt` and a bare ID read stay silent, on [§FS-check.4.3.2](FS-check.md#432-reported-by-check-config-validate-and-config-show)'s argument unchanged. The silence is not an oversight to be widened later — a fact that never changes between runs, printed by every command, is permanent output tools learn to filter, and the surface a user reaches for to ask *where is my config* is `config show`.

#### 4.11.3 Once per config, not once per scope

A workspace names the root's config and every member's, each at the path that project's config was loaded under ([§FS-errors.4](FS-errors.md#4-determinism)) — `packages/beta/.agents/grund.toml`, moving to `packages/beta/grund.toml`. Root and member are separate configs and a workspace may mix the two forms ([§FS-workspace.2](FS-workspace.md#2-workspace-configuration)), so a member on the old path earns its own line while a member on the bare form earns none, and a root and a member both on the old path earn one line each.

#### 4.11.4 A directory carrying both names earns [§FS-check.4.3](FS-check.md#43-redundant-config-pair) and not this

The bare `grund.toml` won the tie ([§FS-config.1.1](FS-config.md#11-when-one-directory-carries-both)), so the config in force is on the home path already and nothing about it is deprecated; the `.agents/` file beside it is the one read by nothing, which is exactly what [§FS-check.4.3](FS-check.md#43-redundant-config-pair) reports. Emitting both would name one move twice and disagree about which of the two files is the problem.

#### 4.11.5 Under `--format=json`, one warning finding on stderr

Under `--format=json` it is one warning finding on stderr, carrying the `code` `deprecated-config-location`, with `path`, `line` and `sites` null ([§FS-errors.5](FS-errors.md#5-json-format)) — [§FS-check.4.3](FS-check.md#43-redundant-config-pair)'s shape, because it arrives the same way: one of the report's warnings, knowable from the config the run loaded rather than from the scan.

#### 4.11.6 It stands in place of the `success` line

It stands in place of the `success` line ([§FS-check.2.1.3](FS-check.md#213-the-success-line)), as every warning does, and that is the whole cost of the finding rather than a detail of it: a clean repository on the old path now prints this line where it printed `success`. [§REQ-backwards-compatibility.1](../requirements/REQ-backwards-compatibility.md#1-what-is-covered) governs that as a verdict change and permits it, since the exit code does not move. This repository pays it in its own e2e corpus, which is why the fixtures that had no reason to be on `.agents/` are on the bare form and the ones that remain are the ones whose subject is discovery itself.

#### 4.11.7 A warning permanently, with no release it becomes an error in

The promise lives in [§FS-config.1.2](FS-config.md#12-the-agents-location-is-deprecated), because it is a fact about the location rather than about this message: the fallback is never removed, so there is no version constant, no roadmap milestone, and no clause in the text naming a release.

### 4.12 Missing snapshot

A recognized citation has no declaration and targets a fetch-enabled kind whose effective `resolve` is `should` ([§FS-config.3.4.10](FS-config.md#3410-format-resolve-and-fetch--external-snapshot-kinds)). It is a distinct fixed warning, not a softened dangling error:

```text
<path>:<line>: no snapshot for <qualified-ID> in <home> — run grund fetch <qualified-ID>
```

The em-dash remedy tail is exactly `— run grund fetch <qualified-ID>`, with no inner backticks. The JSON code is `missing-snapshot`, severity is `warning`, and the message field is the text after `<path>:<line>: `. A warning-only run exits 0 and prints no `success` line. Every site gets one finding, remains in what `refs` and `cover` report, and may resolve after an explicit [§FS-fetch](FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot). The hints that take the fetch tail's place are [§FS-check.4.12.1](FS-check.md#4121-a-hint-takes-the-fetch-tails-place), and its workspace spelling is [§FS-check.4.12.2](FS-check.md#4122-in-a-workspace).

Under v2 the same warning is selected by `[rules.resolution] <KIND> = "warn"`, and `must` selects the error; v1's `resolve = "should"` keeps selecting it. An explicit v2 entry needs `fetch` on the kind's row ([§FS-config-v2.rules.resolution](FS-config-v2.md#rulesresolution-rulesresolution)).


#### 4.12.1 A hint takes the fetch tail's place

The near-ID and escaped-inline-code hints of [§FS-check.3.1.1](FS-check.md#311-a-near-id) and [§FS-check.3.1.2](FS-check.md#312-an-illustration-in-inline-code) take precedence over the fetch action. The message retains `no snapshot for <qualified-ID> in <home>` and substitutes the existing conditional `; did you mean …`, `; write <§>…`, or combined tail for the em-dash fetch tail. That precedence never produces both a dangling and a missing-snapshot finding for one site.

#### 4.12.2 In a workspace

In a workspace, the ID and remedy use the complete alias-qualified spelling while `<home>` is rendered from the run's report base ([§FS-workspace.8.1](FS-workspace.md#81-grund-aliasid)).

### 4.13 Oversized lead *(opt-in)*

Moved to [§FS-declarations.checks.oversized-lead](FS-declarations.md#checksoversized-lead-oversized-lead-opt-in). This address is kept so citations written before the move still resolve.

### 4.14 Unmarked Markdown heading

Moved to [§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading). This address is kept so citations written before the move still resolve.

## 5. What grund does not check

See [§FS-non-goals](FS-non-goals.md#fs-non-goals-what-grund-will-deliberately-not-do) — in particular [§FS-non-goals.1](FS-non-goals.md#1-markdown-link-validation) (markdown links / URLs), [§FS-non-goals.2](FS-non-goals.md#2-spelling-grammar-prose-quality) (spelling/grammar outside the explicit value form), and the convention that ID numbers are stable names, not ordinal positions. Value checking adds only the exact binding in [§FS-values.3.1](FS-values.md#31-the-only-binding-grammar): no surrounding-number inference, bare-literal lint, range/unit semantics, rendering, fingerprint/history check, or new reconciliation verb is performed ([§FS-values.9](FS-values.md#9-compatibility-and-explicit-exclusions)). The near misses that used to be listed here are [§FS-check.5.1](FS-check.md#51-the-near-misses-are-no-longer-here).

### 5.1 The near misses are no longer here

The declaration-side near miss is **no longer** in this section: a heading shaped like `# <KIND>-…: <title>` whose ID does not match the configured `[id] format` is reported per heading by [§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss), and a tree in which every heading misses that way says so twice over — once per line, and once as the run that recognized nothing ([§FS-check.4.5](FS-check.md#45-nothing-recognized)). Neither guesses the corrected ID. The citation-side near miss — a `§`-marked token in the shorthand shape — left this section earlier, when [§FS-check.1.2](FS-check.md#12-the-number-only-shorthand) and [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation) began recognizing and reporting it.

## 6. Watch mode (`--watch`)

`grund check --watch [<path>]` checks immediately, then stays resident and checks again when effective local inputs change. It is the terminal counterpart to [§FS-lsp](FS-lsp.md#fs-lsp-grund-ships-an-optional-lsp-server), serving [§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible). The approved lifecycle and output choices are recorded in [§DF-watch-terminal-loop](../decisions/functional/DF-watch-terminal-loop.md#df-watch-terminal-loop-a-terminal-watch-loop-preserves-ordinary-check-reports); delivery is tracked by [§RM-watch](../roadmap.md#rm-watch-implement-grund-check---watch).

How it notices a change is [§FS-check.6.1](FS-check.md#61-change-detection), what each run prints is [§FS-check.6.2](FS-check.md#62-each-run-is-a-plain-grund-check), how it ends is [§FS-check.6.3](FS-check.md#63-lifecycle), and which command takes the flag is [§FS-check.6.4](FS-check.md#64-scope).

### 6.1 Change detection

The watcher uses native `notify` filesystem notifications on Linux, macOS and Windows. Mounts and pseudo-filesystems that do not deliver usable notifications are unsupported. There is no polling fallback or configurable polling interval. Read/access events do not trigger checks.

#### 6.1.1 Subscribe before reading

Subscribe before the initial scan. Newly discovered inputs must be subscribed before they are read or probed for existence, including each candidate that can affect resolution. This ordering also holds in parallel workspace project scans. On refresh, add coverage, re-resolve under that coverage, then retire obsolete subscriptions. Events during discovery, subscription changes or scans must not disappear between a read and subscription; they arrange a subsequent check against the latest state.

#### 6.1.2 Bounded debounce and serialized work

While idle, run after 100 ms without a relevant event, or after 500 ms from the first pending relevant event, whichever comes first. A burst of writes or an atomic-save rename sequence within that window coalesces. Long saves can expose intermediate states; debounce is not a filesystem transaction.

During a scan/publication retain one pending rerun, rather than a queue entry per event. Scans and report publications are serialized, with no concurrent scans or interleaved reports. A pending rerun observes the latest inputs after the current run; continued writes cannot postpone idle work indefinitely.

#### 6.1.3 Effective input inventory

Observe every effective local input that can change the equivalent one-shot result, including paths outside the selected reporting subtree ([§FS-check.1.3.6.1](FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution)):

- Both `grund.toml` and `.agents/grund.toml` at upward discovery and ancestor-claim candidates, including absent candidates and config precedence changes.
- Workspace roots, nested members, member-glob parents and absent member candidates; resolution-wide source roots and kind homes.
- Catalog JSON, agent entrypoints, indexes, stub targets and other checker probes, and followed file-link targets outside source roots.
- Effective ancestor and nested `.gitignore`/`.ignore`, `.git/info/exclude`, global Git excludes, and the `.gitconfig`/XDG inputs through which those exclusions are discovered.

This is the shared discovery/checker's inventory, not a second set of resolution semantics. Re-resolve config, membership, ignores, catalogs and output defaults on each run. Narrowed checks retain resolution-wide coverage. Hidden discovery inputs such as `.agents/grund.toml` remain observed when hidden source directories are excluded.

Recursive subscriptions cover resolved source roots; ancestor/home coverage is shallow and path-filtered. Broad recursive roots may consume watches for ignored descendants. Missing or replaced inputs retain nearest-existing-parent anchors so creation, deletion, rename, atomic replacement, member additions/removals, and deleted/recreated roots refresh coverage rather than leaving dead subscriptions.

Lexical directory aliases retain their link-replacement anchors while sharing physical native coverage. Retiring an alias must preserve any coverage still required by another input, including recursive coverage; replacement or retargeting re-resolves that ownership under the anchors.

#### 6.1.4 Uncertain notifications and recovery

Lost events, overflow, unknown-path/uncertain events and internal queue saturation force full input rediscovery, rescan and subscription reconciliation. Replacement uses parent anchors to reattach. A failed setup/runtime subscription or failed watcher recovery is fatal under [§FS-check.6.3.2](FS-check.md#632-fatal-watcher-failures); the process must not silently remain resident with stale results. An ordinary input failure during that rescan remains recoverable under [§FS-check.6.3.1](FS-check.md#631-recoverable-runs) if usable discovery coverage can be retained.

### 6.2 Each run is a plain `grund check`

Each run preserves the ordinary check's stdout/stderr bytes, finding order and status on that tree state ([§FS-check.2](FS-check.md#2-outputs), [§FS-errors.4](FS-errors.md#4-determinism)), except the terminal controls in [§FS-check.6.2.2](FS-check.md#622-owned-terminal-screen). Reuse the ordinary checking, selection, sorting, rendering and status mapping. Every run preserves `--full`, grounding, suggestions, trial-rule selection, repeatable `--only`/`--ignore`, their precedence, and unhideable operational failures. Explicit `--format` overrides config defaults, including defaults changed while watching.

#### 6.2.1 Exact stream contract

Clean text prints the ordinary `success` marker ([§FS-check.2.1.3](FS-check.md#213-the-success-line)); JSON is ordinary finding NDJSON ([§FS-check.2.1.4](FS-check.md#214-json)). Clean JSON emits no record. Concatenated JSON therefore exposes neither clean runs nor every run boundary. No envelope, banner, timestamp or production run/completion record is added. Findings, warnings and operational errors keep their ordinary stream ownership. Redirected streams append the successive ordinary outputs.

#### 6.2.2 Owned terminal screen

Text owns an alternate screen only if stdout is an alternate-screen-capable terminal and stderr either shares that terminal or is redirected. Enter through stdout with `\x1b[?1049h`; immediately before each publication clear only that owned screen with `\x1b[H\x1b[2J`; restore through stdout with `\x1b[?1049l` on exit or before JSON publication. These spellings denote the corresponding escape bytes, not literal backslash text.

Shared-terminal stderr is cleared with the report; redirected stderr appends and receives no clearing bytes. Redirected stdout, distinct stdout/stderr terminals and dumb terminals append on both streams, with no enter/clear/restore bytes. JSON never enters or clears a screen. A text-to-JSON format change restores the owned screen before any JSON bytes; a later eligible text run can acquire it again. No stream changes owner. Exit restores pre-watch terminal content rather than retaining the last report. Fatal failure restores first, then prints its error.

### 6.3 Lifecycle

The process runs until interrupted or a fatal watcher failure. It is non-interactive: no TUI, key bindings or prompt ([§FS-non-goals.10](FS-non-goals.md#10-interactive-mode)). It performs no network I/O ([§FS-non-goals.11](FS-non-goals.md#11-network-access-during-a-check)), launches no configured processes and writes no filesystem inputs. It may observe local metadata and parent anchors needed by [§FS-check.6.1.3](FS-check.md#613-effective-input-inventory), beyond files whose contents the scan reads.

#### 6.3.1 Recoverable runs

Ordinary findings and config/read failures publish the ordinary status `0`, `1` or `2` and remain resident for repair. Initial invalid config retains discovery and parent coverage. Later failed runs retain the last usable inventory plus discovery anchors until correction. Config-dependent trial-rule refusals are recoverable runs; static invocation errors terminate before entering the loop ([§FS-check.6.4](FS-check.md#64-scope)).

#### 6.3.2 Fatal watcher failures

Setup/runtime subscription failures and failed watcher recovery restore any owned screen, print an actionable stderr `error:` diagnostic identifying the failed watching operation/input and exit `2`. This status overrides any previously completed check status. Release subscriptions and join workers on fatal exit as on interruption.

#### 6.3.3 Interrupt and completion

On Ctrl-C/SIGINT discard pending work. Let an active synchronous scan finish privately and discard its unpublished result. Publication already begun finishes and counts as completed only when both streams have been fully published and flushed. Shutdown may therefore wait for an active scan/publication.

Return the most recently completed run's status (`0`/`1`/`2`). If none has completed, restore any owned screen and print `error: interrupted before the first check completed` on stderr, returning `2`. Restore the screen, release subscriptions and join workers before exit; no watcher or worker remains resident. Release is bounded: a native backend that has not released its subscriptions within 5 seconds of being closed, on refresh or at exit, is a fatal watcher failure under [§FS-check.6.3.2](FS-check.md#632-fatal-watcher-failures), never an indefinite wait. An unpublished interrupted result never changes the last completed status.

### 6.4 Scope

`--watch` is a `check` flag spelled as `grund check --watch [<path>]` ([§FS-cli](FS-cli.md#fs-cli-grunds-command-line-surface-conventions)). Other subcommands reject it. Parse and validate static invocation choices once, preserving ordinary validation diagnostics and their precedence; reuse those choices for every run. Config-dependent validity and output defaults are re-evaluated with the tree, rather than frozen at startup. There is no daemon/service protocol, public binding watch API, incremental engine or LSP dependency.
