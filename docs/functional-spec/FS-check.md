# FS-check: grund validates every reference in a repo

The `check` command walks a repo and reports every violation of the grund reference scheme. Validation is explicit as `grund check [<path>]`; the bare `grund <ID>` default belongs to [§FS-show.1](FS-show.md#1-inputs). Serves [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration) and [§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible).

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
- **full-tree scope** — The scope `--full` selects: every file the walk can reach, in place of
  the configured default scope.

## 1. Inputs

- Optional path argument; defaults to the current directory. May be a directory or a single file (`grund check crates/grund-core/src/scanner/file_pass.rs` scopes the scan to one file but still discovers the `grund.toml` by walking up — [§FS-config.1](FS-config.md#1-file-location-and-discovery)).
- The walked tree may contain markdown (`.md`) and source files (Rust, Go, Java, TS, Python, etc.).
- Optional `grund.toml` configuring the run per [§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable) ([§FS-config](FS-config.md#fs-config-grund-reads-a-toml-config-file-found-by-walking-up)).
- Optional `[workspace]` config; when present and `check` is run at the workspace root, `check` validates alias-qualified cross-project citations per [§FS-workspace](FS-workspace.md#fs-workspace-grund-validates-cross-project-citations-in-a-workspace).
- `--watch` is reserved for the planned resident checker ([§FS-check.6](FS-check.md#6-watch-mode---watch)) and is not accepted by the current CLI.
- `--require-grounding` — turn the grounding check ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)) on for this run regardless of `[reference] require_grounding` in `grund.toml` ([§FS-config.3.1](FS-config.md#31-reference--citation-form)). It only ever *adds* the check; it cannot switch off a config that already sets it. The flag and the key are **one knob**: the flag sets the same global default, so a `[[kinds]]` row that says `require_grounding = false` is still exempt under it, and `grounding_level` has no flag at all ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)). A run-level flag that overrode the row would make the flag mean something the key cannot say.
- `--suggestions` — emit the suggestions channel ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)) for this run. The flag never adds an error or changes the exit code: it only surfaces the advisory records [§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in) lists, which the default run withholds.
- `--rule "<sentence>"` — add one ad-hoc chapter rule to the configured rules
  for this run. It never disables them; validation, deduplication, and exits are
  [§FS-rules.4](FS-rules.md#4-validation-lifecycle) and
  [§FS-rules.8](FS-rules.md#8-command-surfaces)'s.
- `--only <code>` — retain only diagnostics whose exact finding code is in the selected set ([§FS-check.1.4](FS-check.md#14-selecting-diagnostics-with---only-and---ignore)).
- `--ignore <code>` — remove diagnostics whose exact finding code is in the selected set ([§FS-check.1.4](FS-check.md#14-selecting-diagnostics-with---only-and---ignore)).
- `--full` — walk the whole config root past `[scan] include`, reporting unresolved references on their own tier ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full), [§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only)) plus the scanner-invariant `section-outside-declaration` error ([§FS-check.3.23](FS-check.md#323-section-outside-a-declaration)). It only ever *adds* findings; the in-scope report is unchanged.
- `--format text|json` — output shape, per [§FS-errors.5](FS-errors.md#5-json-format). The global flags `--version` and `--help` are handled before any scan ([§FS-cli](FS-cli.md#fs-cli-grunds-command-line-surface-conventions)).

### 1.1 Recognized citations

Per [§DF-reference-marker](../decisions/functional/DF-reference-marker.md#df-reference-marker-use--as-the-reference-marker-with--as-the-typing-trigger), a citation is the marker followed by an ID, e.g. `§FS-check.3.1`. The default marker is `§`; configurable via `grund.toml`.

In default mode (`[reference] strict = true`), only marker-prefixed citations are recognized — bare tokens are treated as plain text and do not trigger dangling-ref errors. Repositories that still rely on bare citations may set `[reference] strict = false` as a compatibility mode after checking the migration surface with `grund fmt --marker` ([§FS-fmt](FS-fmt.md#fs-fmt-grund-normalizes-references-in-bulk)).

Citations may appear in markdown prose, in source-file line/block comments, and in language doc-comments (Javadoc, JSDoc, Rustdoc, Python docstrings, etc.) — see [AR-scanner.2.3](../architecture/AR-scanner.md#23-citation-detection) and [AR-scanner.4](../architecture/AR-scanner.md#4-inline-declarations-in-language-doc-comments) for the exact contexts. `E2E` citations (`§E2E-<name>`) resolve against case directories under the configured `E2E` kind home per [AR-scanner.6](../architecture/AR-scanner.md#6-e2e-case-declarations).

#### 1.1.1 Off-grammar citations

An off-grammar citation that a catalog declaration backs ([§FS-config.3.2](FS-config.md#32-id--id-grammar)) participates in section and dangling checks, inbound counts, grounding, citation directions, `refs`, formatting, and editor navigation exactly like a conforming citation. The declaration's mismatch is reported at its heading ([§FS-check.4.6](FS-check.md#46-declaration-near-miss)), not at every citation.

#### 1.1.2 Named-section candidates

When `[id] named_sections = true`, the scanner consumes a whole ID-and-dot-tail candidate before deciding what it means ([§FS-config.3.2](FS-config.md#32-id--id-grammar)). A marker-prefixed legal named or mixed coordinate is a citation, including when the named section is missing. An unmarked candidate with a letter-bearing tail is one prose token and is suppressed whole even under `strict = false`; it never falls back to a bare-ID citation. A reserved `number.name` candidate is likewise never truncated to its numeric prefix. Full IDs claim candidates before number-only shorthand ([§FS-check.1.2.3](FS-check.md#123-the-full-id-always-wins)), and shorthand form and section existence stay independent facts ([§FS-check.1.2.4](FS-check.md#124-a-resolved-shorthand-is-a-real-edge)), exactly as for numeric coordinates.

#### 1.1.3 String literals in source files

In source files, a **bare** ID-shaped token whose start column falls inside a string literal is not treated as a citation (the same deterministic quote-tracking rule `grund fmt` uses — [§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule), [AR-scanner.2.3](../architecture/AR-scanner.md#23-citation-detection)), so an ID-shaped substring inside runtime data does not raise a false dangling-ref. A marker-prefixed **unqualified** citation is recognized everywhere, string or not, except inside the assigned Python data span bounded by [§FS-check.1.1.3.1](FS-check.md#1131-assigned-python-triple-quoted-data) — the marker remains the signal of intent in every other string context. Markdown files have no string literals and neither source-file carve-out applies there. Decided in [§DF-python-assigned-triple-quoted-data](../decisions/functional/DF-python-assigned-triple-quoted-data.md#df-python-assigned-triple-quoted-data-module-level-assigned-triple-quoted-strings-are-data-not-docstrings).

##### 1.1.3.1 Assigned Python triple-quoted data

In a `.py` file scanned with `[scan] docstring_python = true`, a simple unindented assignment whose right-hand side begins with a triple-quoted string opens an **assigned-data span**, not a docstring. Its left-hand side is one Python identifier, optionally followed by a same-line annotation and then `=`; whitespace may surround the annotation separator and `=`. The string may use `'''` or `"""` and may carry, case-insensitively, no prefix or one of `r`, `u`, `b`, `f`, `t`, `br`/`rb`, `fr`/`rf`, or `tr`/`rt`. The prefix and delimiter must be the first non-whitespace text after `=`. Parenthesized assignments, destructuring targets, computed targets, indented assignments, and a string reached only after another expression are outside this rule: this is a bounded lexical distinction, not Python scope or AST analysis ([§FS-non-goals.3](FS-non-goals.md#3-code-ast-parsing)).

The span begins at the prefix when present, otherwise at the opening delimiter, and ends immediately after the next matching delimiter that is not escaped by an odd-length run of backslashes. It may open and close on one line or cross lines; quote-like text using the other delimiter and an escaped matching delimiter do not close it. No citation form, declaration, section heading, value marker or binding, or inline citation site is recognized inside the span. Source columns remain columns in the raw file. Text before the span retains ordinary source treatment, text after a same-line close resumes ordinary source treatment at its raw column, and the next line after a multiline close begins in neutral state. A later delimiter at the first non-whitespace column therefore opens the same real module, function, or method docstring it would have opened had the assignment not existed.

With `docstring_python = false`, no assigned-data state is introduced and every line retains the existing raw-line source-string behavior. A leading triple-quote with no qualifying assignment remains the existing docstring form when the gate is enabled. Ordinary one-line strings, other Python triple-quoted contexts, other source languages, Markdown, strict-mode bare-token handling, and namespace-qualified citation suppression retain their existing rules.

#### 1.1.4 Markdown link destinations

In a Markdown file, the parallel carve-out is the link destination: a **bare** ID-shaped token whose start column falls inside an inline link's `(…)` half — `[text](…)` — is likewise not treated as a citation, because `grund fmt` never rewrites a link destination ([§FS-fmt.2.3](FS-fmt.md#23-what-is-never-rewritten)) and a finding whose only named fix the tool refuses to perform is one a repository can never clear ([§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)). The exclusion is total, not just a withheld error: such a token is not a citation for `refs`, for unused-declaration counting ([§FS-check.4.1](FS-check.md#41-unused-declaration)), or for grounding ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)) either. A marker-prefixed citation inside a link destination is unaffected — the marker is the signal of intent there too, exactly as it is inside a source-file string literal.

#### 1.1.5 Contexts read as neither prose nor code

Two contexts are read as neither prose nor code, so nothing inside them is a citation. A **fenced code block** in Markdown is skipped entirely: this is what makes an example ID safe to write in documentation without the `<§>` escape, and it is why the illustrations throughout these specs resolve to nothing. A fence opens with at most three leading spaces followed by a run of at least three backticks or tildes; it closes only on a run of the **same character** at least as long as the opener, again with at most three leading spaces and only whitespace after the run. A backtick opener cannot carry a backtick in its info string. An unclosed fence runs to end of file. In **source files only**, the namespace-qualified form `§<alias>/<ID>` is additionally skipped inside an inline-code span or a string literal, because `alias/ID` is shaped like a path, module reference, or URL ([AR-scanner.2.3](../architecture/AR-scanner.md#23-citation-detection)); the unqualified form stays live there, and neither skip applies in Markdown. Every other skip is a property of the *walk* rather than of the text — a file the walk skips, for any of the reasons [§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded) lists, is never read at all ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)).

#### 1.1.6 Value bindings

An exact explicit value binding additionally records its authored component, but its marker-prefixed token remains one citation under every rule above. The binding grammar and its narrower recognized source-comment contexts are [§FS-values.3](FS-values.md#3-explicit-value-bindings); `[reference] strict = false` never removes the binding's marker requirement.

#### 1.1.7 Fetched snapshots

A snapshot written by [§FS-fetch](FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot) is not a special input. Its configured file or folder home is in the ordinary scan scope ([§FS-config.3.5](FS-config.md#35-scan--what-gets-walked)); its declaration, body, sections, and marked body citations are recognized by the rules above. `check` never invokes its configured integration.

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

`[scan] include` and every walked kind home ([§FS-config.3.5](FS-config.md#35-scan--what-gets-walked)) are the roots the walk starts from, so a citation in a file outside them is not merely unchecked — it is invisible: it does not resolve, does not dangle, and appears in no report, so dangling IDs accumulate there unnoticed. A clean `check` then means "clean *within* the scope" but reads as "clean" — a false negative in the one command the workflow trusts, invisible by construction, because the citations that most need checking are the ones somebody forgot to bring into scope. [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration) calls that class a bug, so it must be *seen* without first editing the config to guess where to look; [§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded) accepts the bounded blind spot only because this flag exists to look into it.

`grund check --full` is that way. It cancels `[scan] include` for the walk, and with it the `scan = false` prune of an unwalked kind home ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-walked)) — and nothing else. Decided in [§DF-check-full-scope](../decisions/functional/DF-check-full-scope.md#df-check-full-scope-check---full-walks-past-scan-include-and-reports-unresolved-references-plus-orphaned-section-headings-out-there).

#### 1.3.1 The walk covers the whole config root

`[scan] exclude`, `.gitignore` and every other ignore file, hidden names — directory and file alike ([§FS-config.3.5](FS-config.md#35-scan--what-gets-walked)) — workspace member boundaries ([§FS-workspace.6](FS-workspace.md#6-nested-project-boundary)), the E2E case-directory boundary wherever a citable `E2E` kind is configured ([AR-scanner.6](../architecture/AR-scanner.md#6-e2e-case-declarations)), and `[scan] extensions` all still apply exactly as they do without the flag. A file type `grund` does not scan stays unscanned; widening `extensions` is a config decision, and one flag that widened both would make "what did this run read" unanswerable. A boundary is a *declared* member, not any nested `grund.toml`: a project directory the workspace never declared is ordinary tree to both walks, so `--full` reads it and judges its citations under *this* project's grammar and kinds. That is what the plain walk does with it too, but the flag is what makes it reachable by default — a vendored, generated, or example project belongs in `[scan] exclude`, or in `[workspace] members` if it is one of ours, before a run adds `--full`.

#### 1.3.2 The wider walk reads a superset, each file once

Every root the plain walk starts from — each `[scan] include` entry and every kind home it walks ([§FS-config.3.5](FS-config.md#35-scan--what-gets-walked)) — is walked under `--full` too, whether or not `exclude`, an ignore file, or the hidden-directory rule would otherwise prune it: those three rules prune *descendants*, never the directory a walk starts at, so a gitignored, excluded, or hidden root is read by the plain run and must be read here. Without that, `--full` could read *fewer* files than `grund check` and hide a finding instead of adding one. A hidden **file** is the exception, read by neither run even as a root ([§FS-config.3.5](FS-config.md#35-scan--what-gets-walked)); additivity survives it because the file is missing from both walks, not from one. Overlapping roots — an `include` entry inside another, or inside the config root the flag adds — name one file once; a file read twice would be a declaration duplicated with itself ([§FS-check.3.3](FS-check.md#33-duplicate-declaration)). "Once" is per *file*, not per path: an `include` root that is a symlink to a directory inside the config root, or a case alias of one on a case-insensitive filesystem, reaches its files under a spelling the config-root walk never produces, so a byte-identical compare cannot see the reread. The walk therefore starts at those roots *before* the config root and keeps the first spelling of each file — the one `grund check` prints without the flag. Every in-scope line is the plain run's, character for character; outside reference errors add the `outside [scan] include:` tier prefix, while [§FS-check.3.23](FS-check.md#323-section-outside-a-declaration)'s scanner invariant keeps its ordinary code and message.

#### 1.3.3 Two scopes, two rule sets

Inside the default scope, the report is the ordinary one. Outside it, [§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only)'s reference-resolution errors and [§FS-check.3.23](FS-check.md#323-section-outside-a-declaration)'s single scanner-invariant exception are reported. No style, grounding, placement, direction, duplicate, unused, or other rule is applied there. A `--full` that failed on conventions in directories that never opted into them would be run once and never again.

#### 1.3.4 Purely additive

The findings inside the configured scope are exactly the ones `grund check` reports on the same tree, so `--full` can only ever turn a green run red, never the reverse. It is the ordinary check plus unresolved references and section-like headings that violate declaration-body ownership in the wider walk.

#### 1.3.5 The unused-declaration warning is unchanged out there

A declaration inside `include` cited *only* from outside it keeps its `declared but never cited` warning ([§FS-check.4.1](FS-check.md#41-unused-declaration)) under `--full`, and the citation that would have retired it resolves and is reported by nothing. That is what additivity costs, and it is the right side of the trade: counting the wider walk's citations toward [§FS-check.4.1](FS-check.md#41-unused-declaration) would *remove* an in-scope finding, the one direction this flag must never move, and it would make the warning mean something different depending on a flag. The remedy is the one the tier already names — widen `include` so the citing file is governed, and the edge counts everywhere.

#### 1.3.6 An explicit path still narrows

`--full` cancels `include`, never a path the caller typed: `grund check <path> --full` scans exactly `<path>` ([§FS-config.3.5](FS-config.md#35-scan--what-gets-walked)), which is already outside `include`'s reach, and has no out-of-scope tier. A path that *resolves to the config root* is the root scope and does widen — `grund check .`, `grund check ./`, and `grund check <abs-root>` are the bare `grund check --full`, tier included. Any other path leaves the flag nothing to cancel.

#### 1.3.7 A path the flag cannot widen earns a caution, not a refusal

When `--full` is passed with an explicit path that is not the config root, the run emits one CLI-level `warning:` ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)) on **stderr** and reports the same findings, on the same streams, with the same exit code as the run without the flag:

```
warning: --full has no effect with an explicit PATH — it cancels [scan] include, and sim already bypasses it
```

Silently accepting the flag is the failure this mode exists to end in miniature: the caller asked for the wider search and got the ordinary run, with no signal. Rejecting it would be worse — the run is a valid one, and a script that passes `--full` uniformly would start failing on the invocation where it happens to be redundant. It is a warning like any other: the exit code is untouched, it stands in place of the `success` marker on an otherwise clean run ([§FS-check.2.1](FS-check.md#21-report-format)), and under `--format json` it is one diagnostic object on stderr, so a clean run's **stdout** stays empty either way.

#### 1.3.8 Workspaces widen per project

Run at a workspace root, `--full` applies to the root project and to every member ([§FS-workspace.5](FS-workspace.md#5-command-scope)): each walks its own tree past its own `[scan] include` and tiers its findings against its own configured scope, because `include` is a per-project statement. It widens the projects a run already has and never invents one, so under `[workspace] include_root = false` ([§FS-workspace.2](FS-workspace.md#2-workspace-configuration)) a file at the workspace root outside every member is read by nothing, with or without the flag — there is no root project whose `include` there would be to cancel.

#### 1.3.9 An empty default scope is still reported

A `--full` run whose *default* scope read no files gets the [§FS-check.2.2](FS-check.md#22-empty-scan) caution as well as its out-of-scope findings: the tier says where the citations actually are, the caution says the config has not been told.

#### 1.3.10 A flag, never a key

`--full` is a flag, never a `grund.toml` key. A project that wants its whole tree governed widens `include` and gets the whole rule set; `--full` exists for the tree whose config has drifted from where the code moved, and a config key for it would be a second, weaker `include`.

### 1.4 Selecting diagnostics with `--only` and `--ignore`

Both flags repeat: repeated values form a union, duplicate values are harmless, and each occurrence takes exactly one code — comma-separated values are not split into a selector language. An absent `--only` set retains every code. The two compose: a diagnostic is retained when its code is in `--only` (or no `--only` was given) and is not in `--ignore`, so ignore wins when both sets name the same code.

Codes are the documented lowercase kebab-case vocabulary in [§FS-errors.5](FS-errors.md#5-json-format), not categories or message fragments. Selecting an opt-in code such as `oversized-lead` is valid even when its config key is absent, but selection never activates the finding ([§FS-check.4.13](FS-check.md#413-oversized-lead-opt-in)).

Selector values are validated before config discovery or scanning. Lowercase kebab-case means `[a-z0-9]+(?:-[a-z0-9]+)*`: no uppercase, underscore, comma, empty segment, or leading/trailing hyphen. A missing or empty value is rejected as `error: --only requires a finding code` or `error: --ignore requires a finding code`; a value outside that grammar is rejected as `error: invalid finding code "<value>" (expected lowercase kebab-case)`; and a well-formed value outside the public catalog is rejected as ``error: unknown check finding code "<value>"; run `grund check --help` for supported codes``. These are CLI failures: stdout is empty and the exit is `2` ([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)). Validation is independent of flag spelling: both `--only value` / `--ignore value` and `--only=value` / `--ignore=value` have the same behavior.

## 2. Outputs

A report on **stdout** — `check` is a linter and its findings are its output ([§FS-errors.1](FS-errors.md#1-streams)) — plus an exit code:

- `0` — no retained errors. Retained warnings and suggestions are allowed (they do not affect the exit code).
- `1` — at least one retained error.
- `2` — scan or CLI failure (I/O, malformed file, invalid `grund.toml`, or invalid invocation).

The three value findings are ordinary fixed-severity errors and exit `1`; home JSON input that [§FS-values.5.3](FS-values.md#53-incomplete-input-and-deterministic-output) counts as incomplete leaves the scan incomplete and exits `2`. What `--only` and `--ignore` may select is [§FS-check.2.1.2](FS-check.md#212-selection-filters-the-complete-report), and what an incomplete run prints is [§FS-check.2.4](FS-check.md#24-an-incomplete-run). For verbose text and JSON report examples, including empty JSON scans and global diagnostic ordering, see [§FS-output-shapes](FS-output-shapes.md#fs-output-shapes-machine-readable-output-shapes).

### 2.1 Report format

Findings are written to **stdout**, one per line, in the form:

```
<path>:<line>: error: <message>
<path>:<line>: warning: <message>
<path>:<line>: suggestion: <message>
```

`<path>` is relative to the base `[output] relative_paths` selects — the config root under the default ([§FS-config.3.6](FS-config.md#36-output--report-format)). `<line>` is 1-indexed. The `<path>:<line>:` prefix is mandatory on every finding so editors and agents can jump unmodified — this is the contract from [§GOAL-friendliness-first.1](../goals.md#1-hard-requirements).

Every retained located text diagnostic carries its lowercase channel after that jump-friendly prefix: `error:` for [§FS-check.3](FS-check.md#3-errors-detected), `warning:` for [§FS-check.4](FS-check.md#4-warnings), and `suggestion:` for an enabled [§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in) advisory. The marker is report structure rather than part of the diagnostic message; `<message>` retains its ordinary bytes. Text reports follow the grouped order [§FS-errors.4](FS-errors.md#4-determinism) fixes. Every retained diagnostic remains present and unabridged.

When a finding inherently spans multiple sites (e.g., duplicate declarations, [§FS-check.3.3](FS-check.md#33-duplicate-declaration)), the message is anchored at the lexicographically-first site (sort by `path`, then `line`) and the other sites are listed parenthetically inside the message.

Selection happens before the fixed per-format sort, render, and exit decision ([§FS-check.2.1.2](FS-check.md#212-selection-filters-the-complete-report)). An otherwise empty selected report makes the default text form write exactly `success` plus a trailing newline ([§FS-check.2.1.3](FS-check.md#213-the-success-marker)); with `--format=json`, the retained findings are emitted as NDJSON on stdout instead ([§FS-check.2.1.4](FS-check.md#214-json)).

#### 2.1.1 CLI-level messages

Lines that are about the run rather than a finding at a site in the repo — unknown subcommand, malformed flag, invalid `grund.toml` schema (when the config itself parses but a value is wrong), a per-file read failure mid-walk ([§FS-check.2.4](FS-check.md#24-an-incomplete-run)), the empty-scan caution ([§FS-check.2.2](FS-check.md#22-empty-scan)), the citation-obligation caution ([§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing)), the nothing-recognized caution ([§FS-check.4.5](FS-check.md#45-nothing-recognized)) — are CLI-level messages: on **stderr**, never on stdout, with the shape, prefix and exit rules, and JSON treatment of [§FS-errors.2.2](FS-errors.md#22-cli-level-message). A `grund.toml` schema error is one of the CLI-level messages that still point at a line, beside the workspace warnings of [§FS-check.4.7](FS-check.md#47-a-workspace-member-swallows-the-blocks-own-scan) and [§FS-check.4.10](FS-check.md#410-include_root--false-leaves-the-blocks-own-files-unread), in the form and for the reason [§FS-config.4.3](FS-config.md#43-invalid-config-behavior) gives. [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) was in that list until its ramp ended: in `check` it is now a located error on stdout, and the CLI-level line it still prints on the five other walking surfaces is the one described there.

#### 2.1.2 Selection filters the complete report

`check` always completes the ordinary scan and every checker pass before applying `--only` and `--ignore` ([§FS-check.1.4](FS-check.md#14-selecting-diagnostics-with---only-and---ignore)): selection is a query over that complete report, not a way to skip declaration, citation, maintenance, warning, or suggestion work. Coded errors, warnings, and enabled suggestions are selected by the same rule, after `--suggestions`, `--full`, and `--require-grounding` have decided which diagnostics exist, and before the fixed per-format sort, render, and exit decision. Every retained diagnostic keeps its message, location, code, sites, and channel, and selectors cannot alter its relative order within its text severity group or the global JSON order; flag order and duplication cannot affect output ([§FS-errors.4](FS-errors.md#4-determinism)). CLI and config failures happen outside that selectable report, and a mid-scan failure's `io` diagnostic cannot be hidden: either kind of incomplete run ([§FS-check.2.4](FS-check.md#24-an-incomplete-run)) remains visible and exits `2` regardless of selectors. The unselected default run is unchanged, and no third exit status exists.

#### 2.1.3 The `success` marker

When there are zero retained errors and zero retained warnings, and no retained suggestion or unselectable run-level line exists, the default text form writes exactly `success` plus a trailing newline to stdout. The marker is only emitted for an otherwise empty selected report; a run that has a retained warning or suggestion prints that line instead. It says that the selected report is empty, not that the repository has no findings. There is no summary footer — the exit code is still the machine-readable verdict, and the per-finding lines are the human-readable detail.

#### 2.1.4 JSON

With `--format=json`, the retained findings are emitted as NDJSON on stdout instead — same stream, machine shape per [§FS-errors.5](FS-errors.md#5-json-format). JSON remains byte-, shape-, and order-compatible: its objects keep the global order [§FS-errors.4](FS-errors.md#4-determinism) fixes rather than inheriting text's severity groups. JSON remains diagnostics-only: stdout is empty when the selected report has no retained diagnostics, so `grund check --format=json | jq …` sees only diagnostic objects. CLI-level `error:` / `warning:` lines, when there are any, go to stderr ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)), so a clean JSON run is empty on *both* streams and a `2` always means something on stderr.

### 2.2 Empty scan

A walk that read **no scannable files** at all, and turned up no findings (no errors, no warnings — including the agent-entrypoint check of [§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block), which still runs and still reports even when nothing is scanned), is almost always a misconfigured scope rather than a clean repo. Rather than print `success` and exit `0` — which reads as "all clear" — `check` emits one CLI-level `warning:` line ([§FS-errors.2.2](FS-errors.md#22-cli-level-message)) to **stderr**: it is a caution about the run, not a finding about the repo, so it does not belong on stdout with the findings. Its message names the likely cause ([§FS-check.2.2.2](FS-check.md#222-the-message-names-the-likely-cause)); its exit code, JSON form, and what suppresses it are [§FS-check.2.2.3](FS-check.md#223-exit-code-json-and-what-suppresses-it). This is the friendliness-first counterpart to the explicit success marker ([§GOAL-friendliness-first.1](../goals.md#1-hard-requirements)): the run that scanned nothing is one of the cases where `success` would be the wrong answer ([§FS-check.2.1.3](FS-check.md#213-the-success-marker)).

#### 2.2.1 Citation-direction obligation applies to nothing

When `[citations.<kind>]` contains at least one `must` or `should` obligation, but the citing kind has no unit for the obligation to evaluate, `check` emits one CLI-level `warning:` on **stderr**. This is a run-level fact, not a finding at a repository site: the warning has no path, line, or sites, and it does not change the exit code or emit `success` in text mode. In `--format=json`, it is one standard warning diagnostic on **stderr** with `path`, `line`, and `sites` all `null` ([§FS-errors.5](FS-errors.md#5-json-format)). Its stable diagnostic code is `empty-citation-obligation`. The warning is independent of other findings: another warning or error does not suppress it.

The warning is emitted once per configured kind when all of the conditions [§FS-check.2.2.1.1](FS-check.md#2211-when-it-fires) lists hold; a workspace asks it per project, and an explicit path of the files it scans ([§FS-check.2.2.1.3](FS-check.md#2213-workspaces-and-explicit-paths)). Its messages distinguish the two kinds of missing unit ([§FS-check.2.2.1.2](FS-check.md#2212-the-messages)).

##### 2.2.1.1 When it fires

The warning fires for a kind when all of these conditions hold:

1. The table has a non-empty `must` or `should` list. A table containing only `must-not` or `should-not` entries has no obligation unit and does not warn. A kind with both levels is named by `must`; a `should`-only table is named by `should`.
2. The citing kind has a `folder` home, and that kind is walked. File homes, the homeless kind, and `scan = false` kinds do not warn.
3. The run successfully scanned at least one file that belongs to that folder, excluding the folder's entry file. For a citable kind, the entry is its effective `index` (`README.md` when the key is omitted); for a non-citable kind, the entry is the literal `README.md`. `index = false` excludes no file. Files not successfully scanned, files outside the home, and files hidden or excluded by the walk do not count.
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

When the explicit path is a **file whose own name begins with `.`** and whose extension is one `[scan] extensions` lists, the message names the hidden-name rule instead of [§FS-check.2.2.2.2](FS-check.md#2222-an-explicit-path)'s ([§FS-config.3.5](FS-config.md#35-scan--what-gets-walked)). The extension list is the one rule that did *not* skip that file, so naming it would send the reader to edit config that was never the cause:

```
warning: nothing to scan — `docs/.notes.md` is a hidden file. grund reads no file whose own name begins with `.`, whatever `[scan] extensions` says. Rename it, or move what needs checking into a file that is not hidden.
```

A hidden file whose extension is *also* unlisted keeps [§FS-check.2.2.2.2](FS-check.md#2222-an-explicit-path)'s message: there the list is a true reason, and naming one of two causes would be its own misdirection. This case answers for a handed **file** only — a handed *directory* whose only listed-extension content is hidden keeps [§FS-check.2.2.2.2](FS-check.md#2222-an-explicit-path)'s message too, though the hidden-name rule is the sole reason it read nothing, because the walk does not record that it met candidates and rejected every one of them by name.

##### 2.2.2.4 No project in scope

When a `[workspace]` block put no project in scope at all — `include_root = false`, and every member it has is an optional one this checkout does not have ([§FS-workspace.2.2](FS-workspace.md#22-a-member-that-may-be-legitimately-absent)) — the message says exactly that. The messages of [§FS-check.2.2.2.1](FS-check.md#2221-the-repo-root) and [§FS-check.2.2.2.2](FS-check.md#2222-an-explicit-path) would both be false here, because the walk never looked under `[scan] include` and the tree `grund init --docs` scaffolds is not what is missing. This one names no remedy either, for the reason [§FS-check.4.9](FS-check.md#49-a-workspace-member-declared-optional-is-absent)'s announcement names none: nothing is misconfigured, and the only thing that changes the answer is a fuller checkout.

#### 2.2.3 Exit code, JSON, and what suppresses it

This is a warning, not an error: the exit code stays `0` (a genuinely empty tree is not a failure), and `--format=json` emits the warning as one diagnostic JSON object on stderr, the stream of the text `warning:` line rather than of the findings on stdout. A repo that *does* have a stale `AGENTS.md` block or any other finding **about the configured scope** gets that finding (on stdout) and **no** empty-scan notice. Three findings are not about that scope and do not suppress it ([§FS-check.2.2.3.1](FS-check.md#2231-findings-that-do-not-suppress-it)).

##### 2.2.3.1 Findings that do not suppress it

- The redundant-config pair ([§FS-check.4.3](FS-check.md#43-redundant-config-pair)) is about which file the run read rather than what it walked, so a repository mid-migration keeps the scope diagnostic beside its config pair.
- The out-of-scope tier ([§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only)) is about the tree *outside* the scope, and is the case the caution is worth most ([§FS-check.1.3.9](FS-check.md#139-an-empty-default-scope-is-still-reported)).
- The absent-member announcement ([§FS-check.4.9](FS-check.md#49-a-workspace-member-declared-optional-is-absent)) is about the namespaces the run skipped rather than the scope it walked: the block whose last project went missing is exactly the run that has nothing to read, and it must not lose the line saying so to the line saying why.

### 2.3 Suggestions channel *(opt-in)*

The `should` / `should-not` levels of `[citations]` ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)) produce **suggestions** — findings carried on the suggestions channel rather than at a third severity ([§FS-terms.terms.5](FS-terms.md#terms5-findings), [§FS-distribution.3.0.1](FS-distribution.md#301-report-and-finding), [§FS-rules.7](FS-rules.md#7-findings-and-channels)): they are advisory by RFC-2119 definition and grund has no per-site suppression mechanism, so surfacing them in the default run would replace the `success` marker ([§FS-check.2.1.3](FS-check.md#213-the-success-marker)) on a repo that has consciously accepted a deviation, and it would never recover. They are therefore withheld from the default run and live on a separate channel, decided in [§DF-citation-directions](../decisions/functional/DF-citation-directions.md#df-citation-directions-encode-citation-directions-as-checked-config-with-rfc-2119-levels).

`grund check --suggestions` ([§FS-check.1](FS-check.md#1-inputs)) emits them. A suggestion is a third report channel, **not** a third severity: [§FS-config.6](FS-config.md#6-what-is-not-configured-here) freezes the severity set at `{error, warning}`, so a suggestion carries `"channel": "suggestion"` rather than a `severity` ([§FS-errors.5](FS-errors.md#5-json-format)). The codes are `suggested-citation` (a `should` obligation a declaration does not meet), `discouraged-citation` (a `should-not` citation site), and `escaped-citation-resolves` ([§FS-check.2.3.1](FS-check.md#231-escaped-citation-resolves)).

`--only` and `--ignore` select suggestions only after `--suggestions` has enabled this channel; that selection, the text and JSON rendering, and the exit code suggestions never affect are [§FS-check.2.3.2](FS-check.md#232-selecting-and-printing-suggestions). `grund gap` is the standing home for these records once it ships ([§FS-check.2.3.3](FS-check.md#233-grund-gap-is-their-standing-home)).

#### 2.3.1 Escaped citation resolves

A citation whose marker is bracketed — the schematic `<§>alias/ID` shape — is deliberately inert: the `§` is not immediately followed by the ID, so no pass treats it as a citation ([§FS-workspace.1](FS-workspace.md#1-citation-syntax)). That is how a citation's *shape* is written in prose without `grund check` resolving it. It also makes an escape of an ID that *does* exist ambiguous: usually a deliberate illustration, but also exactly what a live citation looks like once the marker is bracketed by accident, a slip that raises no dangling error ([§FS-check.3.1](FS-check.md#31-dangling-citation)) and navigates nowhere. So when an escaped citation's ID resolves to a real declaration, grund emits an `escaped-citation-resolves` suggestion at the escape site, naming the live `§`-form to switch to. It never replaces `success` or changes the exit code ([§FS-check.2.3.1.1](FS-check.md#2311-never-a-warning-or-error)), and it reads both escape forms ([§FS-check.2.3.1.2](FS-check.md#2312-both-escape-forms)).

##### 2.3.1.1 Never a warning or error

It is a suggestion, never a warning or error: illustrating a real ID is legitimate, so it must never replace `success` ([§FS-check.2.1.3](FS-check.md#213-the-success-marker)) or change the exit code. It is the mirror of the [§FS-check.3.1](FS-check.md#31-dangling-citation) dangling check — that flags a live citation whose ID does not resolve; this flags an escaped one whose ID does.

##### 2.3.1.2 Both escape forms

Unqualified `<§>ID` and qualified `<§>alias/ID` escapes are both covered. The ID is parsed with the citing project's grammar, so a cross-namespace target under an unusual grammar may be skipped, which only ever withholds a suggestion.

#### 2.3.2 Selecting and printing suggestions

`--only` and `--ignore` select suggestions only after `--suggestions` has enabled this channel: a selector cannot surface a suggestion the run did not request. Selecting away every enabled suggestion restores the ordinary empty selected-report behavior from [§FS-check.2.1.3](FS-check.md#213-the-success-marker).

- **Text** — `--suggestions` prints each suggestion in the located-finding shape `<path>:<line>: suggestion: <message>` ([§FS-check.2.1](FS-check.md#21-report-format)), after the error and warning groups in the same deterministic within-group order ([§FS-errors.4](FS-errors.md#4-determinism)). Without the flag, suggestions are not printed, and the `success` marker still appears for a run with zero errors and zero warnings even if suggestions exist — a suggestion says the graph is thinner than recommended, not that it is malformed.
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

An invalid `grund.toml` aborts before any file is read ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)): exit `2`, a single `error:` line on stderr, nothing on stdout. A per-file failure *during* the walk — a file that cannot be read or decoded, an unreadable directory, or a link the walk cannot resolve ([§FS-config.3.5.5](FS-config.md#355-a-link-the-walk-cannot-resolve-is-reported-and-not-walked-into)) — does not abort: the offending path is reported as `error: <path>: <reason>` on stderr, in the CLI-level shape of [§FS-errors.2.2](FS-errors.md#22-cli-level-message) because the path has no line to point at and "I could not read this" is about the run, not a finding about the graph; the walk continues over the remaining files; every finding collected from the readable files is still printed to stdout in the normal located `check` form; and the run exits `2` because the view of the tree was incomplete. A `2` therefore always means "do not trust this report as complete"; the printed findings are still real. Malformed input is answered with a diagnostic naming the path and a truthful code, never an abort ([§REQ-never-crashes](../requirements/REQ-never-crashes.md#req-never-crashes-garbage-in-diagnostic-out)).

## 3. Errors detected

Each of the following is an error and contributes to a non-zero exit code.

### 3.1 Dangling citation

A recognized citation (per [§FS-check.1.1](FS-check.md#11-recognized-citations)) for which no declaration is found: `unknown reference <ID>`. A near ID ([§FS-check.3.1.1](FS-check.md#311-a-near-id)) or an inline-code context ([§FS-check.3.1.2](FS-check.md#312-an-illustration-in-inline-code)) adds a hint, and a fetch-enabled kind adds the fetch action ([§FS-check.3.1.3](FS-check.md#313-a-kind-that-fetches)).

A number-only shorthand citation ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)) is exempt from this rule and reported by [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation) instead — never both, because `unknown reference FS-042` would name a token that is not a full ID under the repo's own grammar. For a value binding, this ordinary resolution finding suppresses value comparison at the same site ([§FS-values.5.1](FS-values.md#51-resolve-before-comparison)).

#### 3.1.1 A near ID

If the target namespace contains a declared ID of the same kind that is close by deterministic edit distance, the diagnostic appends one hint: `unknown reference FS-chek; did you mean FS-check?`. If no same-kind candidate is close enough, the message stays `unknown reference <ID>` so unrelated missing IDs do not produce noisy guesses.

#### 3.1.2 An illustration in inline code

When the dangling citation sits inside a Markdown inline-code span — where a `§`-citation is as often an illustration as a live reference — the diagnostic also offers the `<§>` escape ([§FS-check.2.3.1](FS-check.md#231-escaped-citation-resolves)): `unknown reference api/FS-zzz; write <§>api/FS-zzz if this is an illustration`. The two hints combine when a near-ID match and an inline-code context apply at once: `unknown reference api/FS-login; did you mean api/FS-logout? (or write <§>api/FS-login if this is an illustration)`. Outside inline code the escape hint is withheld, so an ordinary prose typo is nudged toward the near ID, not toward escaping.

#### 3.1.3 A kind that fetches

This is also the fixed finding for a fetch-enabled kind whose effective target-side resolution is `must` ([§FS-config.3.4.10](FS-config.md#3410-format-resolve-and-fetch--external-snapshot-kinds)). In that case the exact message is `unknown reference <qualified-ID>; no snapshot in <home> — run grund fetch <qualified-ID>`, its JSON code remains `dangling`, its severity is `error`, and it contributes exit 1. For a kind without `fetch`, the historical `unknown reference <qualified-ID>` bytes remain unchanged.

The near-ID and escaped-inline-code hints ([§FS-check.3.1.1](FS-check.md#311-a-near-id), [§FS-check.3.1.2](FS-check.md#312-an-illustration-in-inline-code)) take precedence over the fetch action. The message retains `unknown reference <qualified-ID>; no snapshot in <home>` and substitutes the existing conditional tail for the em-dash fetch tail: `; did you mean <candidate>?`, `; write <§><qualified-ID> if this is an illustration`, or their existing combined form. One citation site still produces one finding.

### 3.2 Missing section

A citation with a section suffix (`§FS-<user-login>.3.1` or, in an opted-in repository, `§FS-<user-login>.goals`) where the declaration exists but the requested section heading does not. A missing marker-prefixed named coordinate is never shortened to its declaration; it produces the ordinary `section not found` error and adds `write <§> before it to show the shape without citing it` to the message. A number-only shorthand carrying a missing named section produces both the existing shorthand finding and this finding: the persisted ID form and the requested target are independent facts ([AR-checker.2.12](../../crates/grund-core/src/checker/report.rs)).

An owned declaration-local numeric candidate ([§FS-check.1.1.8](FS-check.md#118-declaration-local-numeric-section-candidates)) follows the same rule. Its
[§FS-check.3.24](FS-check.md#324-declaration-local-section-citation) form
finding and this missing-section finding are independent, so a missing local path reports both.

For a value binding the explicit numeric component must resolve here before comparison; a missing component produces this finding alone, not a mismatch ([§FS-values.5.1](FS-values.md#51-resolve-before-comparison)).

### 3.3 Duplicate declaration

The same ID declared more than once: any two declarations that are not stubs ([§FS-check.3.4](FS-check.md#34-broken-inline-spec-stub)), whether headings or inline doc-comment declarations and whether in one file or several. Reported per [§FS-check.2.1](FS-check.md#21-report-format): one error anchored at the lexicographically-first site, with the remaining sites listed in the message.

Duplicate JSON keys, cross-file JSON IDs, Markdown/JSON collisions, and overlapping opted-in ownership feed this same ambiguity rule even when their components agree. A duplicate target cannot be value-compared ([§FS-values.2.3](FS-values.md#23-duplicates-and-ownership)).

### 3.4 Broken inline-spec stub

A `docs/` file whose H1 has the stub shape `# <ID>: [<text>](<path>)` where either the path does not exist, or the file at that path contains no inline declaration of the same ID. Relative stub links resolve as normal Markdown links first — relative to the stub file's directory — so `lychee` and rendered docs see the same target. If that path does not exist, `grund` falls back to resolving the path relative to the config root for compatibility with older stubs that wrote repo-root paths.

### 3.5 Invalid agent entrypoint init block

If `<path>/AGENTS.md` exists, `check` verifies the versioned `grund init` block defined by [§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints); it also verifies the companion agent entrypoints [§FS-check.3.5.1](FS-check.md#351-which-entrypoints-are-verified) names. A missing managed block when one is required, an older block version, a newer unsupported block version, or a config-derived section that no longer byte-matches its re-render from the live config ([§FS-init.2.3.5](FS-init.md#235-citation-directions), [§FS-init.2.3.6](FS-init.md#236-clickable-citations)) is an error in scaffolded-entrypoint mode; a legacy block is an older version and broken delimiters are a distinct error ([§FS-check.3.5.2](FS-check.md#352-legacy-and-malformed-blocks)). This lets CI catch repos whose managed agent entry points were initialized and later drifted or need to be refreshed with `grund init`. Every variant is an error with code `agents-init`, with the reporting order, message text, and selector behavior of [§FS-check.3.5.3](FS-check.md#353-code-and-message).

#### 3.5.1 Which entrypoints are verified

`check` verifies known companion agent entrypoints whenever they exist and are not symlinks to `AGENTS.md` — `.rules` only where the owning `.zed/` directory or a managed block already in it says so, the evidence [§FS-init.2.1.1](FS-init.md#211-one-entrypoint-per-agent) attributes that too-generic name by; for example, existing standalone `AGENTS.override.md`, `CLAUDE.md`, `.claude/CLAUDE.md`, `GEMINI.md`, and `.github/copilot-instructions.md` files must carry the managed block as rendered for that file's own agent ([§FS-init.2.3.6](FS-init.md#236-clickable-citations)), while `CLAUDE.md -> AGENTS.md` is already covered by the canonical file. `check` does not require absent agent-directory-triggered companions from [§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place); once `grund init` creates one, it is validated because it exists.

If `AGENTS.md` does not exist, existing companion agent files without a managed block are treated as project-owned instructions and are not validated by `grund check`; this keeps config-only adoption from modifying or policing an existing agent setup. A companion that already contains a managed `grund init` block is still version-checked even without `AGENTS.md`, so repos initialized directly into `CLAUDE.md`, `GEMINI.md`, or another explicit entrypoint still get drift detection.

#### 3.5.2 Legacy and malformed blocks

A legacy H2-bounded block from v3 or earlier ([§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)) is still recognized and reported as an *older block version* — `` run `grund init` `` is its transition path to the delimited form, so existing repositories are told how to migrate rather than treated as malformed. Broken delimiters are a distinct error: a file whose delimiters [§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints) defines as **malformed** is reported as a malformed managed block, anchored at the offending delimiter line and naming the defect; `check` never rewrites the file, and `grund init` refuses to splice against broken delimiters for the same reason.

#### 3.5.3 Code and message

Every variant remains an error with code `agents-init`, and the default run still reports it after every content pass completes. Its text follows the three-release migration in [§FS-errors.3](FS-errors.md#3-message-text): during the two compatibility releases the legacy message is a verbatim contiguous prefix followed by the fixed maintenance tail; in `0.15.0` the final text explicitly classifies the work as repository maintenance and states that citation validity is unaffected. A selector may retain or remove this ordinary coded error, but never changes the checker pass that produced it.

### 3.6 Ungrounded unit *(opt-in)*

Off by default. Two config keys decide it, each written in `[reference]` as the default for every `[[kinds]]` row and settable on the row itself ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)): `require_grounding` says **whether** a place's files must be grounded ([§FS-check.3.6.1](FS-check.md#361-which-files-a-row-governs)), and `grounding_level` says **what the unit is** inside each of them ([§FS-check.3.6.2](FS-check.md#362-the-unit)). `grund check --require-grounding` ([§FS-check.1](FS-check.md#1-inputs)) sets the same global default, and an explicit `require_grounding = false` on a row wins over it.

A unit is **grounded** when it contains at least one recognized citation ([§FS-check.1.1](FS-check.md#11-recognized-citations)) whose ID resolves to a declaration — **or**, in a source file outside every non-citable home, when it declares an ID inline (a spec home is grounded in the spec it *is*, [AR-scanner.4](../architecture/AR-scanner.md#4-inline-declarations-in-language-doc-comments)). A unit that is neither is an error ([§FS-check.3.6.3](FS-check.md#363-findings)). A unit whose only citation is dangling ([§FS-check.3.1](FS-check.md#31-dangling-citation)) is *not* grounded — it gets both findings; fixing the citation clears both. The rule is a pure function of `(tree, config)`: it reads no git history and parses no code ([§FS-check.3.6.4](FS-check.md#364-a-pure-function-of-the-tree)). Decided in [§DF-require-grounding](../decisions/functional/DF-require-grounding.md#df-require-grounding-an-opt-in-check-that-every-source-file-cites-a-spec).

#### 3.6.1 Which files a row governs

Every scanned file resolves to exactly one `[[kinds]]` row, and that row's effective `require_grounding` decides whether the file is checked at all:

- **A non-citable kind's home** ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)) governs **every** scanned file in it, `.md` included — a `folder` home's files, or the one document of a `file` home ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)), which is a place a maintainer declared like any other and takes both keys on its row ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)).
- **A citable kind's folder home** governs the **source files** in it — a file the walk reads whose extension is not `.md` ([AR-scanner.1](../architecture/AR-scanner.md#1-tree-walk)).
- **The homeless kind** ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)) governs the source files no home claims, and a file claimed by two overlapping homes falls to it as well, the way its citing side already does ([AR-scanner.2.4](../architecture/AR-scanner.md#24-citing-side-classification)).

So Markdown is exempt except inside a non-citable home, and `require_grounding = true` on an unwalked home's row is a config error ([§FS-check.3.6.1.1](FS-check.md#3611-markdown-is-governed-only-in-a-non-citable-home)). A repository that sets only the global key keeps every level `1` and the file as the unit ([§FS-check.3.6.1.2](FS-check.md#3612-the-global-key-alone-is-the-rule-it-always-was)).

##### 3.6.1.1 Markdown is governed only in a non-citable home

Markdown is therefore exempt except inside a non-citable home, and that exception is a home rather than an extension. The Markdown exemption reasons about implementation versus document; a non-citable home is neither guess — it is a directory the maintainer declared matters, and it is usually *all* Markdown, a skill, a runbook, a prompt library. Inheriting the exemption there would switch the rule off exactly where it was turned on. An unwalked home ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-walked)) has no scanned files, so the rule never reaches it — which is why `require_grounding = true` on such a row is a config error.

##### 3.6.1.2 The global key alone is the rule it always was

A repository that sets only `[reference] require_grounding = true` and configures no non-citable kind sees this rule exactly as it did before these keys existed: every row inherits the global `true`, every level is `1`, and the unit is the file.

#### 3.6.2 The unit

`grounding_level` is an integer in Markdown heading levels ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)). Each level **contains the one below it**, so the file itself is always a unit and nothing passes vacuously for lacking structure. In a Markdown file, level `L` adds a unit for every heading subtree of level `2` to `L` ([§FS-check.3.6.2.1](FS-check.md#3621-in-a-markdown-file)); in a source file, the units below the file are doc-comment blocks, ranked by indentation ([§FS-check.3.6.2.2](FS-check.md#3622-in-a-source-file)); and an inline declaration grounds its doc-comment block and its file, except inside a non-citable home ([§FS-check.3.6.2.3](FS-check.md#3623-the-inline-declaration-escape)).

##### 3.6.2.1 In a Markdown file

Level `L` makes a unit of the whole file and of every heading subtree whose level is between `2` and `L`. A subtree runs from its heading to the line before the next heading at the same or a higher level, so a parent is satisfied by any descendant and a leaf must cite directly; text before the first heading belongs to the file rather than to a section. At level `1` there are no section units and the file is the only one, which is the unit every config had before the key existed. A file with no heading at the level is one unit — the file — for the same reason.

##### 3.6.2.2 In a source file

There are two ranks, and they are read by indentation rather than by syntax ([§FS-non-goals.3](FS-non-goals.md#3-code-ast-parsing)): at level `2` every **unindented** doc-comment block is a unit — a parse-free stand-in for a top-level item, which holds across Rust, Python, Java, Go, and Kotlin — and at any higher level every doc-comment block is. What counts as a doc comment is the per-language rule of [§FS-inline-citation-style.1.1](FS-inline-citation-style.md#11-doc-comments-are-not-sites), already read once per file by the scanner. The file is a unit at every level, as in Markdown.

##### 3.6.2.3 The inline-declaration escape

The inline-declaration escape of [§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in) applies per unit: a doc-comment block that declares an ID is grounded by that declaration, and so is the file it sits in. It has no effect inside a non-citable home, where a declaration is a misplaced declaration to begin with ([§FS-check.3.7](FS-check.md#37-misplaced-declaration-configured-kind-home)) — there the only way to ground a unit is to cite one.

#### 3.6.3 Findings

A finding is anchored at its unit — line 1 for a file, the heading line for a section, the block's first line for a doc comment — and names the unit and, when the unit sits in a non-citable home, the home:

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

The rule is a pure function of `(tree, config)` like every other `check` rule ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)): it reads no git history ([§FS-non-goals.6](FS-non-goals.md#6-decision-database-audit-log-history-tracking)) and parses no code ([§FS-non-goals.3](FS-non-goals.md#3-code-ast-parsing)) — "source file" is decided by extension, a unit by heading level or comment indentation, and "grounded" by the citations the scanner already collected. It is the floor of the grounding discipline — the verification-at-rest layer of [§GOAL-agent-grounding.1](../goals.md#1-the-three-layers), on top of which `grund cover` exposes the citation graph ([§FS-cover](FS-cover.md#fs-cover-grund-groups-citations-by-scanned-file)) and [§RM-cochange-gate](../roadmap.md#rm-cochange-gate-a-pre-commit--ci-recipe--no-impl-change-without-spec-and-test) tracks the diff-aware co-change gate.

### 3.7 Misplaced declaration (configured kind home)

A declaration that sits where a configured kind home does not allow it is a misplaced-declaration error, anchored at the declaration line. Three placements are refused: a single-file kind's declaration outside its file ([§FS-check.3.7.1](FS-check.md#371-a-single-file-kind)), a declaration inside another kind's home ([§FS-check.3.7.2](FS-check.md#372-another-kinds-home)), and any declaration in a non-citable home ([§FS-check.3.7.3](FS-check.md#373-a-non-citable-home)). The home rules ([§FS-check.3.7.2](FS-check.md#372-another-kinds-home), [§FS-check.3.7.3](FS-check.md#373-a-non-citable-home)) apply to declaration lines and stub lines, not citations or prose mentions. A file that belongs to no configured home, or that matches several because configured homes overlap or nest, is not checked by them, because its expected kind is ambiguous.

#### 3.7.1 A single-file kind

A kind configured with `file = "<path>"` in [[kinds]] ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)) is a *single-file kind*: every declaration of that kind must live in that exact document, and one whose H1/H2 is found in any other scanned file is reported:

```
docs/notes.md:42: GOAL-foo must be declared in docs/goals.md (single-file kind)
```

Stubs (`# <ID>: [<text>](<path>)`) are exempt from this exact-file requirement: a stub points from a kind's home folder to an inline declaration elsewhere, a multi-file-kind feature, and a single-file kind has no folder to redirect from. This rule is the canonical mechanism that keeps `GRUND`, `GOAL`, and `RM` declarations in their documents, and what makes "one file, all goals inline" a checked invariant rather than a convention.

#### 3.7.2 Another kind's home

Every configured `file` and `folder` is also a declaration-home boundary: a declaration line in a file that belongs to exactly one configured kind home must declare that home's kind. A `file` home matches only that exact path; a `folder` home matches files below that directory. The error names the declared kind, the expected home kind, and the configured home:

```
docs/functional-spec/FS-lsp.md:42: AR-router declares kind AR inside FS home docs/functional-spec
```

#### 3.7.3 A non-citable home

A **non-citable home** ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)) admits no declaration of any kind. It has no kind an author could have declared instead, so the message names the place and says why rather than pointing at a kind that does not exist:

```
skills/review/SKILL.md:1: FS-review must not be declared in skills/ (not a citable home)
```

That is the rule working as designed, not a gap in it: `citable = false` says the directory is a place, and a place with a declaration in it is one of the two facts in conflict.

### 3.8 Cross-project citation failure

An alias-qualified citation whose alias path is unknown is reported at the citation site in every run, with or without a `[workspace]` ([§FS-workspace.1](FS-workspace.md#1-citation-syntax)); in a workspace run, so is one whose target declaration or target section is missing. The namespace and resolution rules live in [§FS-workspace.4](FS-workspace.md#4-resolution).

An unknown alias path names the projects it could have meant, so the fix is in the diagnostic rather than in the config: the outermost workspace root searches every project in scope ([§FS-check.3.8.2](FS-check.md#382-candidates-at-the-outermost-root)); a run narrowed to a subtree searches only for a strict extension of its scope ([§FS-check.3.8.1](FS-check.md#381-a-strict-extension-of-the-narrowed-scope-is-safe-to-hint)) and otherwise names the subtree it covers ([§FS-check.3.8.3](FS-check.md#383-a-narrowed-run-offers-no-candidate)), in the wording [§FS-check.3.8.4](FS-check.md#384-the-scope-only-message-across-three-releases) stages. The citation remains unresolved and the run still fails.

#### 3.8.1 A strict extension of the narrowed scope is safe to hint

A narrowed run may search for a candidate only when its non-empty scope path is a strict, segment-wise prefix of the written alias path: the written path starts with every scope segment and has at least one segment after them. Thus `group/alph` is eligible in scope `group`, and `group/alpha/bet` in scope `group/alpha`. The search sees only the aliases the narrowed run loaded, so every candidate it can name is inside the subtree the run can judge.

An eligible path gets the outermost root's ordered tiers, sorting, three-result limit, prose joining, and `; did you mean …?` rendering ([§FS-check.3.8.2](FS-check.md#382-candidates-at-the-outermost-root)); when the loaded aliases yield no candidate, the message is the bare `unknown project alias <path>` form. Outermost-root runs keep their unconditional candidate search.

Shorter and equal paths, paths whose segments do not begin with the scope, and merely lexical prefixes remain ineligible: in scope `group`, that excludes `alpha`, `group`, `outside/alpha`, and `grouped/alpha` alike. Each gets the scope-only message of [§FS-check.3.8.4](FS-check.md#384-the-scope-only-message-across-three-releases) with `<scope>` = `group` — in `0.13.2` and `0.14.0` the staged form with its exact compatibility suffix, from `0.15.0` the final template. Eligibility, candidate selection, resolution, and the failing verdict do not change in any of the three releases.

#### 3.8.2 Candidates at the outermost root

At the outermost workspace root — the scope CI runs, and the only one that can see every project a path could name — candidates are taken from the projects in scope in one tier only, best first: a project whose slash-separated path has every written segment as a **proper prefix** and at least one further segment, else one whose path **ends with** what was written (a dropped prefix — the mistake full alias paths invite, [§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)), else one whose **last segment** matches (a wrong prefix), else one a **typo** away under the near-match rule of [§FS-check.3.1.1](FS-check.md#311-a-near-id). Thus `group` matches `group/alpha`, and `group/alpha` matches `group/alpha/beta`; an exact `group`, `grouped/alpha`, and `other/group` do not match the proper-prefix tier. The first non-empty tier alone wins, without candidates from any lower tier. Its candidates are deduplicated, sorted by alias bytes, and then limited to three — `grund list` is the catalogue, a finding is not — and a path with no candidate reports on its own, unchanged.

```text
docs/FS-root.md:3: unknown project alias sprayer; did you mean hardware/sprayer?
docs/FS-root.md:4: unknown project alias api; did you mean left/api or right/api?
docs/FS-root.md:5: unknown project alias group; did you mean group/alpha?
```

#### 3.8.3 A narrowed run offers no candidate

A run narrowed to a subtree ([§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)) holds only part of the tree, so a path naming a project outside it is unknown *here* while being exactly right at the workspace root. Except for the visibly in-scope case of [§FS-check.3.8.1](FS-check.md#381-a-strict-extension-of-the-narrowed-scope-is-safe-to-hint), such a run therefore offers **no candidate at all**. It cannot tell a dropped prefix from a path that correctly names a project outside its subtree, and every tier reads it as the former: the dropped-prefix tier included, because a *shorter* written path is itself a complete alias path whenever a top-level project carries that name, so re-pointing it at a deeper namesake rewrites a citation CI accepts into a different project's — green before and green after, so nothing catches it.

In every ineligible case it names the subtree it covers — as a *subtree*, since that scope's own alias path is one project among the several it holds — rather than reporting the path bare, which is neither "delete this" nor "re-prefix this" but "check this from the root."

#### 3.8.4 The scope-only message across three releases

In `0.13.2` and `0.14.0`, the complete legacy diagnostic remains a contiguous prefix for consumers that match it, and the exact suffix clarifies that a subtree includes the named project and its descendants while warning when the wording changes:

```text
unknown project alias <path>; only the <scope> subtree is in scope here — check from the workspace root for a path outside it — here, the <scope> subtree means the <scope> project and its descendants; this wording changes in grund 0.15.0
```

In `0.15.0`, that compatibility form must be replaced with exactly:

```text
unknown project alias <path>; the <scope> project and its descendants are in scope here — check from the workspace root for a path outside that subtree
```

### 3.9 Section heading level mismatch

`[id] section_heading_levels` ([§FS-config.3.3.2](FS-config.md#332-section_heading_levels--heading-depth-against-path-depth)) sets how a citable section heading's Markdown depth must match its dotted path ([AR-scanner.2.2](../architecture/AR-scanner.md#22-section-detection)), and whether a mismatch is an error, a warning, or not reported. A mismatch the mode reports is anchored at the heading line. This point judges the depth of headings that already carry coordinates; the project-wide in-body Markdown ATX rule for a heading that carries none is [§FS-check.4.14](FS-check.md#414-unmarked-markdown-heading), independent of this mode. Bold labels are not headings and remain unchecked.

### 3.10 Inline citation style violation

A citation site in a code comment that violates the configured inline citation style — `inline_style = "citation-only"` with prose present, an inline note that exceeds `inline_note_max_lines`, or one that exceeds `inline_note_max_columns`. A site is an *inline* comment block, never a doc-comment block, so nothing in this rule reaches a citation written inside a `///`, a `/** … */`, a docstring, or a comment documenting the definition below it ([§FS-inline-citation-style.1.1](FS-inline-citation-style.md#11-doc-comments-are-not-sites)). The full mode and budget contract, and how multi-cap violations split into multiple findings, lives in [§FS-inline-citation-style.4.1](FS-inline-citation-style.md#41-errors--hard-caps); the schema for the controlling keys is in [§FS-config.3.1](FS-config.md#31-reference--citation-form). An opt-in layout check adds one further form ([§FS-check.3.10.1](FS-check.md#3101-layout-deviation)).

#### 3.10.1 Layout deviation

With `[reference] inline_note_layout` set to a layout and `inline_note_layout_check = "error"`, each line that [§FS-inline-citation-style.3.3.1](FS-inline-citation-style.md#331-per-line-not-per-site--and-only-where-a-note-opens) judges, in a citation site that carries a note ([§FS-inline-citation-style.3.3.2](FS-inline-citation-style.md#332-only-sites-that-carry-a-note)), and that does not match the configured form is an error anchored at that line ([§FS-inline-citation-style.3.3](FS-inline-citation-style.md#33-inline_note_layout--where-the-citations-sit), [§FS-inline-citation-style.4.4](FS-inline-citation-style.md#44-warnings-and-errors--opt-in-layout-deviations)). The same deviation is a warning under `inline_note_layout_check = "warn"` ([§FS-check.4.4](FS-check.md#44-inline-note-layout-deviation-opt-in)) and silent at the default `off`. It is the one member of this rule that anchors per line rather than at the site's first line, because a layout deviation is a property of the line an author has to edit.

### 3.11 Missing required citation

When `[citations]` ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)) sets a `must` obligation for a citing kind, every top-level declaration of that kind must carry at least one citation satisfying each `must` entry, anywhere in its body. A declaration that does not is an error anchored at the declaration line, naming the unmet target:

```
docs/architecture/AR-router.md:1: AR-router must cite FS or GOAL (citation direction)
```

The body extent and the citing-side classification come from the scanner ([AR-scanner.2.4](../architecture/AR-scanner.md#24-citing-side-classification)); the obligation pass is [AR-checker.2.9](../../crates/grund-core/src/checker/report.rs). The homeless kind ([§FS-check.3.11.1](FS-check.md#3111-the-homeless-kind)) and a non-citable kind ([§FS-check.3.11.2](FS-check.md#3112-a-non-citable-kind)) are asked per file instead, at the row's `grounding_level` ([§FS-check.3.11.3](FS-check.md#3113-the-unit-follows-grounding_level)); every failing unit is reported ([§FS-check.3.11.4](FS-check.md#3114-every-failing-unit-is-reported)), a file with no citation is no unit ([§FS-check.3.11.5](FS-check.md#3115-a-file-with-no-citation-is-no-unit)), and an `E2E` case has its own unit ([§FS-check.3.11.6](FS-check.md#3116-an-e2e-case)). The parallel `should` obligation is not an error; it is a suggestion ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)).

The ordinary rule sentence `<subject> must cite at least one <target-set>.`
reuses this code only when its actual count is zero. Rule-to-rule and the narrow
config-to-rule bridge deduplicate as
[§FS-rules.6](FS-rules.md#6-semantic-deduplication) specifies; when config
participates, this section's existing message stays byte-for-byte unchanged.

#### 3.11.1 The homeless kind

A **homeless-kind** obligation ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)) — `code`, or whatever the project named it — is per file rather than per declaration: a source file that contains at least one citation but none satisfying the obligation is the error, anchored at line 1.

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

Units are built from citations, so a file carrying none produces no unit and `must` cannot fire on it — except that a walked folder with real non-entry content earns the run-level warning of [§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing). The same zero-unit boundary [§FS-config.3.9.2](FS-config.md#392-the-homeless-kind) states for the homeless kind remains intentionally unwarned. In a non-citable home `require_grounding` closes the per-file grounding hole ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)): there the grounding rule follows the home rather than the file extension, so "cite something" and "cite an `FS`" are two keys that compose, while the warning of [§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing) points at the row's key when grounding is off.

#### 3.11.6 An `E2E` case

An `E2E`-kind obligation ([§FS-config.3.9.1](FS-config.md#391-levels)) is per case declaration, can be satisfied by the case's `spec.refs` manifest entries, and remains an error when the case has no scanned citations or matching manifest reference.

### 3.12 Forbidden citation

When `[citations]` ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)) sets a `must-not` prohibition for a citing kind, every citation site of that kind to a prohibited target is an error anchored at the citation site:

```
docs/functional-spec/FS-login.md:42: FS must not cite AR (citation direction) — re-point the citation or downgrade it to a plain Markdown link
```

The prohibition pass is [AR-checker.2.10](../../crates/grund-core/src/checker/report.rs); how it reads the citing and the cited kind is [§FS-check.3.12.1](FS-check.md#3121-how-the-two-kinds-are-read). The parallel `should-not` prohibition is not an error; it is a suggestion ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)). The sanctioned way to keep a discouraged downward pointer is a plain Markdown link, which is not a citation under `strict = true` and so is exempt from this rule.

A rule sentence `<subject> must not cite any <target-set>.` reuses this code and
the exact citation-site anchor. Its message replaces only the fixed `(citation
direction)` authority tail with `(<RULE-ID>)`; a config-derived duplicate keeps
this section's bytes unchanged ([§FS-rules.6](FS-rules.md#6-semantic-deduplication)).

#### 3.12.1 How the two kinds are read

The citing kind is the site's resolved `source_kind` ([AR-scanner.2.4](../architecture/AR-scanner.md#24-citing-side-classification)), named by kind for a citable kind, by **home** for a non-citable one the way [§FS-check.3.11.2](FS-check.md#3112-a-non-citable-kind) names it (`skills/ must not cite AR`), and by name for the homeless kind, which has no home to name it by ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)). The cited kind and namespace come from the citation token, matched against the rule's namespace grammar ([§FS-config.3.9.3](FS-config.md#393-namespace-matching)).

### 3.13 Number-only shorthand citation

A recognized shorthand citation ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)) persisted in a scanned file is governed by the target project's `[reference] shorthand` policy ([§FS-config.3.1](FS-config.md#31-reference--citation-form)). Under the default `canonical` policy, a uniquely resolving site is reported with its replacement text and `grund fmt --write` applies that mechanical fix in bulk ([§FS-fmt.2.4](FS-fmt.md#24-shorthand-to-canonical)). Under `accepted`, the same marker-origin site is valid and produces no shorthand-form finding; its full canonical citation may coexist in the same file. The policy gate changes only the unique result: a shorthand matching no declaration or several still earns the unknown or ambiguous form of [§FS-check.3.13.3](FS-check.md#3133-one-finding-per-site-in-three-forms), and no policy permits grund to guess.

Under `canonical`, an error rather than a warning or a suggestion: a warning leaves the exit code alone, so a repo could accumulate shorthand citations forever while CI stayed green ([§DF-number-only-citation-shorthand.2.3](../decisions/functional/DF-number-only-citation-shorthand.md#23-it-is-an-error-not-a-warning-or-a-suggestion)). A citation of a kind whose effective format has no `{number}` or no `{slug}` never earns this finding, because that kind has no shorthand ([§FS-check.1.2.1](FS-check.md#121-the-shorthand-shape)). Where the text itself forbids the rewrite, the resolving form does not fire ([§FS-check.3.13.1](FS-check.md#3131-where-the-text-forbids-the-rewrite), [§FS-check.3.13.2](FS-check.md#3132-a-python-docstring-is-not-a-string-literal)).

#### 3.13.1 Where the text forbids the rewrite

A shorthand inside inline code, a Markdown link destination, or a source string literal is exempt from the resolving form of this error, because [§FS-fmt.2.3](FS-fmt.md#23-what-is-never-rewritten) forbids the rewrite in those three contexts, where the text is legitimate as it stands, and an error whose only named fix the tool declines to perform there is one a repository can never clear. A suppressed scope and an external file-symlink target are not among them: `fmt` will not rewrite there either, but the error still fires and the author clears it by hand ([§FS-fmt.2.5](FS-fmt.md#25-suppressed-scopes), [§FS-fmt.2.3.2](FS-fmt.md#232-a-link-that-leaves-the-config-root-is-not-written-through)). The exempt citation is untouched in every other respect — it resolves, `refs` lists it, and it keeps its declaration from being reported unused ([§FS-check.1.2.4](FS-check.md#124-a-resolved-shorthand-is-a-real-edge)). The exemption is for the *mechanical* form only: a shorthand matching zero or several declarations is still reported in those contexts, because that is a dangling reference rather than a formatting nit.

#### 3.13.2 A Python docstring is not a string literal

A Python docstring is not a string literal for this exemption: its delimiters are doc-comment syntax, so the question is asked of its content ([§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule)), and a shorthand anywhere inside one — the opening line, a one-line docstring, an interior line, the closing line — is reported and rewritten exactly like one in a `#` comment. A shorthand inside a `"…"` or `'…'` literal on a **code** line is what stays exempt.

#### 3.13.3 One finding per site, in three forms

At most one *shorthand* finding per site, in one of three forms. Other rules judge the site on their own terms — a bad section ([§FS-check.3.2](FS-check.md#32-missing-section)) or a forbidden direction ([§FS-check.3.12](FS-check.md#312-forbidden-citation)) is a separate fact about the same citation and is reported separately:

```
docs/notes.md:5: shorthand citation §FS-042; write §FS-042-user-login
docs/notes.md:6: shorthand citation §FS-999 matches no declaration
docs/notes.md:7: shorthand citation §FS-042 is ambiguous: FS-042-user-login, FS-042-user-logout
```

The candidate list in the ambiguous form is sorted and complete — `grund` names every match and resolves none, because choosing one would be a guess and `check` reports facts about the tree ([§FS-check.5](FS-check.md#5-what-grund-does-not-check), [§GOAL-agent-grounding.3](../goals.md#3-what-this-rules-out), [§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution)). Duplicate *numbers* are not otherwise an error: [§FS-check.3.3](FS-check.md#33-duplicate-declaration) catches duplicate full IDs, and a repo may legitimately hold `FS-042-user-login` alongside `FS-042-user-logout` as long as nothing abbreviates them. The marker rendered in the message is the configured one ([§FS-config.3.1](FS-config.md#31-reference--citation-form)), and the qualified form names its namespace (`<§>api/FS-042`, escaped here because this repo has no `api` member) so the replacement can be pasted as written.

### 3.14 Out-of-scope unresolvable citation *(`--full` only)*

Under `--full` ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)), a citation in a file outside the configured scope whose reference resolves to nothing — the ID is declared nowhere ([§FS-check.3.1](FS-check.md#31-dangling-citation)), the declaration exists but the cited section does not ([§FS-check.3.2](FS-check.md#32-missing-section)), the namespace alias is unknown ([§FS-check.3.8](FS-check.md#38-cross-project-citation-failure)), or a number-only shorthand matches zero or several declarations ([§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)) — is an error ([§FS-check.3.14.1](FS-check.md#3141-an-error-not-a-warning)) judged on resolution alone ([§FS-check.3.14.2](FS-check.md#3142-only-resolution-plus-one-scanner-invariant-is-judged)). The site is reported in the ordinary located-finding shape, with the tier named first and the rule's own message after it:

```
sim/world.py:12: outside [scan] include: unknown reference RES-061-world-arable-basin-screen
render/prompts.md:4: outside [scan] include: missing section DA-002-general-field-service-scope.1.4
```

#### 3.14.1 An error, not a warning

It moves the exit code to `1` like every other reference failure. A warning would leave `--full` exit-code-neutral, and a finding no CI run can fail on is one a repository accumulates behind forever — the argument [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation) already makes. Nothing turns red without being asked: the flag is opt-in.

#### 3.14.2 Only resolution, plus one scanner invariant, is judged

The style, placement, grounding, direction, duplicate, and unused rules say how a project organizes the files it has chosen to govern, and `[scan] include` is exactly that choice; a directory nobody configured has agreed to none of them ([§FS-check.1.3.3](FS-check.md#133-two-scopes-two-rule-sets)). The sole exception is [§FS-check.3.23](FS-check.md#323-section-outside-a-declaration): a numeric or enabled named heading outside every declaration body is invalid scanner structure before any project convention applies, so it retains the untiered `section-outside-declaration` code and its ordinary message outside scope too.

#### 3.14.3 Resolution sees the whole walk

An out-of-scope citation whose declaration is also out of scope resolves normally. The tier reports references that point at *nothing*, not references that point outside the configured scope.

#### 3.14.4 The mechanical shorthand rewrite is withheld

[§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)'s resolving form names `grund fmt --write` as its fix, and `fmt` scopes by `[scan] include`, so out there the error would name a fix the formatter declines to apply — the same reason [§FS-check.3.13.1](FS-check.md#3131-where-the-text-forbids-the-rewrite) withholds it at an unrewritable site. A shorthand that matches zero or several declarations is a resolution failure, not a formatting nit, and is reported.

#### 3.14.5 A compound code per rule

A finding here carries `out-of-scope-` followed by the code its in-scope equivalent carries: `out-of-scope-dangling`, `out-of-scope-missing-section`, `out-of-scope-unknown-project`, `out-of-scope-shorthand-citation`. A `--format=json` consumer ([§FS-errors.5](FS-errors.md#5-json-format)) then filters the tier by prefix and the rule by exact match, both on the `code` field the shape already carries; one code for all four would have left the rule readable only by parsing the message prose. The JSON shape gains no field.

#### 3.14.6 The tier leads the message

`outside [scan] include: ` comes first, before the rule's own text, because it is the fact that changes what to do: out there the usual fix is to widen the key, not to edit the citation, and a rule's own fix-it hint — `did you mean …?`, `or write <§>… if this is an illustration` — is likelier to be the wrong advice and would otherwise be read first. Naming the key is the whole remedy the message carries; it does not also spell out "widen `[scan] include`", because every finding in the tier would repeat the same sentence and [§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full) states the remedy once.

#### 3.14.7 A wider walk can fail wider

The flag also puts files the configured scope never touched into the walk, so one that cannot be read or decoded out there is reported as the `error: <path>: <reason>` of [§FS-check.2.4](FS-check.md#24-an-incomplete-run) on stderr and the run exits `2` — "I could not read this" is a fact about the run, not about the tier, and it holds for a file in either scope. A tree whose plain `check` exits `0` can therefore exit `2` under `--full`; that is the wider walk reporting what it found, not a regression.

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

Under `--full` ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)) the site is outside `[scan] include`, where [§FS-check.3.14.4](FS-check.md#3144-the-mechanical-shorthand-rewrite-is-withheld) withholds the mechanical shorthand rewrite for the same reason: `fmt` scopes by `include` too, so the finding would name an edit no run in that scope is asking for.

### 3.16 Duplicate section path

Two or more citable section headings inside one declaration claiming the same dotted path ([AR-scanner.2.2](../architecture/AR-scanner.md#22-section-detection)) — either two `## 1. …` headings or two `## goals: …` headings under one `# FS-001-login`. Reported per [§FS-check.2.1](FS-check.md#21-report-format) in [§FS-check.3.3](FS-check.md#33-duplicate-declaration)'s shape: one error anchored at the first heading in file order, with every other heading line named in the message. Named and numeric coordinates use the same `duplicate-section` code ([§FS-errors.5](FS-errors.md#5-json-format)), the same multi-site `sites` record [§FS-check.3.3](FS-check.md#33-duplicate-declaration) carries, and the same no-ranking rule.

```
docs/functional-spec/FS-001-login.md:5: duplicate section FS-001-login.1 (also declared at docs/functional-spec/FS-001-login.md:9)
```

This is [§FS-check.3.3](FS-check.md#33-duplicate-declaration) one level down. A section path is a citation target, so two headings claiming it give `§FS-001-login.1` two destinations, and picking one silently is the guess [§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution) forbids by name. Decided in [§DF-duplicate-section-path](../decisions/functional/DF-duplicate-section-path.md#df-duplicate-section-path-a-section-coordinate-names-one-heading-or-the-run-says-so). The collision is scoped to one declaration ([§FS-check.3.16.1](FS-check.md#3161-scoped-to-one-declaration)) and its body ([§FS-check.3.16.2](FS-check.md#3162-scoped-to-that-declarations-body)), independent of the heading-level mode ([§FS-check.3.16.3](FS-check.md#3163-independent-of-id-section_heading_levels)), and read from the record `show` reads ([§FS-check.3.16.4](FS-check.md#3164-the-same-record-show-reads)).

#### 3.16.1 Scoped to one declaration

Section paths are addressed as `<ID>.<path>`, so the same `1.` under two different declarations is two distinct coordinates and not a finding. Only headings sharing a declaration collide.

#### 3.16.2 Scoped to that declaration's body

The headings judged are the ones inside the body [§FS-show.2.1](FS-show.md#21-whole-declaration-default) and [§FS-show.2.3.1](FS-show.md#231-what-counts-as-the-comment-block) delimit — in Markdown down to the next same-or-shallower heading, in a source file to the end of the comment block the declaration line opens. A `## 1.` further down the file — in the *next* item's doc-comment, or under a later unrelated heading — is not one of this declaration's sections: `grund <ID>.1` never reaches it, and reporting it would ask for a renumbering that changes what nothing points at. A stub ([§FS-check.3.4](FS-check.md#34-broken-inline-spec-stub)) is one link line whose tail is a path rather than a body, so it declares no sections at all and is never reported here; the headings that count are the inline home's, which is also the file `grund <ID>.<path>` reads.

#### 3.16.3 Independent of `[id] section_heading_levels`

The mode ([§FS-config.3.3](FS-config.md#33-section-paths--arbitrary-nesting-depth)) governs how deep a heading must sit for the path it writes, which is a different fact; `## 1.` and `### 1.` under an H1 declaration both claim path `1` and are a duplicate in every mode, `"loose"` included. [§FS-check.3.9](FS-check.md#39-section-heading-level-mismatch) judges only the first heading, the one the path resolves to; a later claimant is no section target and is not additionally judged for depth, so it yields this finding alone ([§DF-duplicate-section-path.2.4](../decisions/functional/DF-duplicate-section-path.md#24-the-heading-level-rule-judges-only-the-heading-the-path-resolves-to)).

#### 3.16.4 The same record `show` reads

This rule and [§FS-show.2.2.2](FS-show.md#222-ambiguous-section) answer from one recorded section set, so `grund <ID>.<path>` refuses exactly when this rule reports `<ID>.<path>` and returns a body exactly when it does not. Two readers that each decided for themselves would disagree — a fenced example, a heading past the end of the body — and a coordinate `check` calls clean but `show` will not resolve is [§REQ-no-wrong-citation](../requirements/REQ-no-wrong-citation.md#req-no-wrong-citation-a-citation-never-resolves-to-a-guess) failing quietly in the other direction.

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
- outside every zone `fmt` never writes in ([§FS-fmt.2.3](FS-fmt.md#23-what-is-never-rewritten), [§FS-fmt.6.4](FS-fmt.md#64-what-is-never-wrapped)): an inline-code span, a Markdown link destination — a bare ID-shaped token inside `](…)` is a URL, not an entry — a fenced block, a declaration heading line, and an index file reached as an external file-symlink target, which `--write` reads and does not write through ([§FS-fmt.2.3.2](FS-fmt.md#232-a-link-that-leaves-the-config-root-is-not-written-through)). A bare citation in such an index is consequently not an entry and falls to [§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index), while a full link already present there still satisfies the entry obligation; an ordinary index, including a file symlink whose target stays inside the root, remains repairable and reaches this rule;
- and **naming a section that exists**, when it names one at all. The pass has to compute a link target ([§FS-fmt.6.2](FS-fmt.md#62-form)), and a citation whose section no declaration declares has none, so `fmt` passes over the line. Such a citation is already reported by [§FS-check.3.2](FS-check.md#32-missing-section), and adding a second finding whose named command answers `rewrote 0 lines` would be the trap [§FS-check.3.17.5](FS-check.md#3175-anything-else-is-not-an-entry) exists to avoid.

#### 3.17.5 Anything else is not an entry

Anything else in the index is **not an entry**: it neither satisfies [§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index) nor is reported here, and the ID falls to [§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index), whose fix is a human edit rather than a command. The alternative is an error whose named fix the tool declines to perform, which is an error a repository can never clear — the same trap [§FS-check.3.13.1](FS-check.md#3131-where-the-text-forbids-the-rewrite) stays out of, and by the same predicate ([§DF-index-entry-form.2.3](../decisions/functional/DF-index-entry-form.md#23-one-link-per-id-not-every-mention)).

The condition runs one way only: an entry that already *is* a link satisfies [§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index) whatever `fmt` would do with it. Off strict mode, where an unmarked token is a citation ([§FS-config.3.1](FS-config.md#31-reference--citation-form)), that means a hand-written `[FS-x](…)` around one is a correct entry and is left alone; under `strict = true` the same line carries no citation at all, so the ID has no entry and is [§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)'s.

#### 3.17.6 One finding per ID

Only an ID the index already cites in a form `fmt` would wrap ([§FS-check.3.17.4](FS-check.md#3174-only-a-citation-fmt-would-wrap-reaches-this-rule)) reaches this rule; an ID with neither that nor an entry is [§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)'s, and one cause never yields both findings. Where several citations of one ID sit in the index and none is a link, the finding anchors at the first of them in file order.

### 3.18 Declaration missing from its kind's index

A kind configured with a `folder` and an index file — `README.md` unless `index` names another or opts out ([§FS-config.3.4.2](FS-config.md#342-index--the-kinds-index-file)) — promises that the index lists that folder's declarations; nothing verified it before. Every covered declaration the index does not name is one error, anchored at the **declaration's heading** and naming the index file:

```
docs/decisions/functional/DF-md-link-emission.md:1: DF-md-link-emission is not listed in docs/decisions/functional/README.md — became an error in grund 0.13.0
```

Its children say which kinds ([§FS-check.3.18.1](FS-check.md#3181-which-kinds-are-covered)) and declarations ([§FS-check.3.18.2](FS-check.md#3182-which-declarations-are-covered)) are covered, how an external inline declaration enrolls ([§FS-check.3.18.3](FS-check.md#3183-an-external-inline-declaration-enrolls-by-its-canonical-link), [§FS-check.3.18.4](FS-check.md#3184-what-does-not-enroll)), what an entry is ([§FS-check.3.18.5](FS-check.md#3185-what-an-entry-is), [§FS-check.3.18.6](FS-check.md#3186-and-nothing-more)), how a missing or unscanned index is judged ([§FS-check.3.18.7](FS-check.md#3187-a-missing-index-file), [§FS-check.3.18.8](FS-check.md#3188-a-run-that-cannot-see-the-index-does-not-judge-it)), and why this is an error ([§FS-check.3.18.9](FS-check.md#3189-an-error-because-the-deadline-the-warning-named-has-arrived)). Decided in [§DF-index-entry-form](../decisions/functional/DF-index-entry-form.md#df-index-entry-form-an-index-entry-is-one-full-link-per-id-and-nothing-else-about-the-page), [§DF-index-compatibility-ramp](../decisions/functional/DF-index-compatibility-ramp.md#df-index-compatibility-ramp-a-findings-ramp-follows-its-fix-command-not-the-size-of-the-offence), and [§DF-index-not-an-inbound-citation](../decisions/functional/DF-index-not-an-inbound-citation.md#df-index-not-an-inbound-citation-an-index-entry-is-navigation-not-use).

- **Code:** `missing-index-entry` ([§FS-errors.5](FS-errors.md#5-json-format)).

#### 3.18.1 Which kinds are covered

Folder kinds that declare IDs. A `citable = false` kind ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)) has no declarations, so it has no index and this rule never reaches it — which is why setting `index` on one is a config error rather than a silent no-op ([§FS-config.3.4.2](FS-config.md#342-index--the-kinds-index-file)).

#### 3.18.2 Which declarations are covered

Every ID of that kind with at least one declaration site anywhere under `folder` — the whole subtree, not its top level, because a kind's folder routinely holds a directory per topic or per year (`DISC`'s proposals all live in `docs/discussions/proposals/`). A stub-and-inline pair collapses the way [§FS-list.2](FS-list.md#2-behaviour) collapses it: the stub under `folder` is what puts the ID in the folder, and **one** entry for the ID satisfies the rule — pointing at wherever the body lives, which for an inline home is the source file. A declaration of some *other* kind sitting inside the folder is a misplaced declaration ([§FS-check.3.7](FS-check.md#37-misplaced-declaration-configured-kind-home)) and is not additionally demanded here.

#### 3.18.3 An external inline declaration enrolls by its canonical link

An index may also **enroll one external inline declaration directly**, with no stub under `folder`. The enrollment is deliberately a stricter form than an ordinary entry: an unqualified, marker-prefixed citation of the bare ID (no section) whose declaration is in a non-Markdown source file outside `folder`, wrapped as a Markdown link whose destination is exactly the one `grund fmt --cross-refs` derives from that index to the source home ([§FS-fmt.6.2](FS-fmt.md#62-form)). The same canonical link is both the act of membership and the satisfying entry, so `check` requires no third artifact. Where two kinds share one `folder` and one `index`, each enrolls its own: the link is matched to the kind its ID names, so configuration order never lets one kind hide another's external entry. `grund show` and `grund list` still see the source declaration as the only home; enrollment creates no declaration record and no synthetic stub ([§FS-show.2.3](FS-show.md#23-inline-declarations-in-code-and-doc-comments), [§FS-list.2](FS-list.md#2-behaviour)).

#### 3.18.4 What does not enroll

Every condition distinguishes enrollment from surrounding prose. A foreign-kind or qualified citation, a citation of a section, a **number-only shorthand** ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)) — enrollment is the *persisted* whole ID, and a shorthand stays authoring sugar until `grund fmt --write` expands it ([§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)) — an unlinked mention, a link **nested inside another link's destination** or any other zone `fmt` never writes in ([§FS-check.3.17.4](FS-check.md#3174-only-a-citation-fmt-would-wrap-reaches-this-rule)), a link with a different destination, and a link to a Markdown declaration outside `folder` are ordinary references. They neither enroll the ID nor become navigational for [§FS-check.4.1.2](FS-check.md#412-an-index-entry-does-not-count). A marker-prefixed bare-ID mention that `grund fmt --cross-refs` turns into the exact canonical link becomes an enrollment when that form is written — the stored link is the unambiguous signal. Removing it removes the external membership, so there is no missing-entry finding for an external declaration that the index no longer claims. Decided in [§DF-index-entry-form.2.7](../decisions/functional/DF-index-entry-form.md#27-a-canonical-bare-id-link-enrolls-an-external-inline-declaration).

#### 3.18.5 What an entry is

For an ID covered by a declaration under `folder`, one recognized citation ([§FS-check.1.1](FS-check.md#11-recognized-citations)) of the ID in the index file, written as a full Markdown link. The two conditions are the entry's contract and either one unmet is a finding: this rule is the first, and [§FS-check.3.17](FS-check.md#317-index-entry-is-not-a-link) is the second. For an external inline declaration, the canonical link of [§FS-check.3.18.3](FS-check.md#3183-an-external-inline-declaration-enrolls-by-its-canonical-link) establishes coverage and satisfies the entry simultaneously.

Between them sits a third case, and it lands here: a citation `grund fmt --write` would not wrap ([§FS-check.3.17.4](FS-check.md#3174-only-a-citation-fmt-would-wrap-reaches-this-rule)) is not an entry at all, so an index that mentions the ID only that way is reported *here*, where the fix is to write an entry, and never under [§FS-check.3.17](FS-check.md#317-index-entry-is-not-a-link), where the command the message names would decline to act ([§FS-check.3.17.5](FS-check.md#3175-anything-else-is-not-an-entry)).

#### 3.18.6 And nothing more

Layout is free: table or list, grouped or flat, in any order, with any prose around it. `docs/functional-spec/README.md` groups its entries under six curated headings, and a rule that dictated a table would break the best index in the tree. One link per ID is enough — every other occurrence of the ID in the index is untouched and is never a finding ([§DF-index-entry-form.2.3](../decisions/functional/DF-index-entry-form.md#23-one-link-per-id-not-every-mention)).

#### 3.18.7 A missing index file

A missing index file is this same finding class, once per declaration in the folder: a folder whose index nobody wrote is the strongest form of the same fact, not a different one. It is also why the finding is anchored at the declaration and not at the index — an index file that does not exist has no line to point at, and every declaration has one. The message says which way it failed, in a parenthesis after the file name: `(the index file does not exist)`, `(the index file is a directory)` for a path that is one, and `(the index file could not be read)` for a file that is there and would not open. Three phrasings rather than one because "does not exist", said about a directory that plainly does, is a diagnosis the reader has to argue with before they can act on it.

#### 3.18.8 A run that cannot see the index does not judge it

Which IDs the index names comes from the scan, while the form of each entry comes from re-reading the file, and the two can disagree about whether the index was read at all: a narrowed `grund check <one-file>` ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)), or an index the `[scan]` set excludes, leaves the index unscanned while the declarations under the folder are still in view. Reporting every one of them as unlisted would be a finding about the scope, not about the tree, so this rule is skipped for an index file the run did not scan. An index file that is *not there* — missing, or a directory wearing the name — is a fact about the tree and is still reported.

#### 3.18.9 An error, because the deadline the warning named has arrived

No `grund` command writes a missing entry — rendering the index is not a pass `fmt` has — so this rule never had [§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)'s licence for a same-release verdict flip, and took [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)'s deprecation path instead. It arrived in `0.12.0` as a warning whose own text named the release it would become an error in, `0.13.0`, and that release ended the ramp. So the finding is an error like every other in this section — it contributes to the exit code ([§FS-check.3](FS-check.md#3-errors-detected)) and it stands in place of the `success` marker ([§FS-check.2.1](FS-check.md#21-report-format)). The ramp bought what it was for: a repository that never listed its declarations was told, by the tool, in `0.12.0`, exactly which run would start failing.

The deadline clause is spent: a release still ahead is a date a reader can act on, and one that has arrived is not. What replaces it reports rather than promises, `— became an error in grund 0.13.0`, which is the past-tense half of the closed vocabulary [§FS-distribution.4.2](FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name) defines. A landed ramp names its release for the same reason a pending one names its deadline, and for one more: that clause is the only record of the flip a user reads without the changelog, and it is what makes the release path able to refuse a version this error would contradict.

### 3.19 Orphan name-bearing section path

With `[id] named_sections = true`, every proper prefix of a name-bearing section path must be recorded in the same declaration before the descendant can be addressed. `### goals.performance: Performance` therefore requires a recorded `goals`; `### missing.performance: Performance` is one error at that heading's line even when another Markdown heading visually contains it. The check uses the complete recorded path set and is independent of heading-depth validation, so a heading with both a missing prefix and a wrong level produces both findings. Purely numeric paths are unchanged and are never judged by this rule.

The message names the orphan coordinate and its first absent prefix. Its code is `orphan-section`. This is a declaration-side structural error, not a missing-citation error: it is reported even when nobody cites the orphan, and a citation to it may independently be present and resolve to the recorded coordinate.

### 3.20 Invalid value declaration

In a kind opted into whole values, a readable Markdown or JSON declaration that violates [§FS-values.2](FS-values.md#2-value-declarations) is an error at the exact invalid heading, key, or element. An exact embedded marker in an invalid location or a marked root with an invalid shape is the same error, located at the marker for the root/authority failures and at the offending line for content/shape failures ([§FS-values.2.4](FS-values.md#24-embedded-section-value-roots)). Inside a declared value chapter the same error covers content the chapter may not hold and a root whose run is invalid, located at the offending line, or at the root heading for a root-level failure a chapter root has no marker to carry ([§FS-values.2.5](FS-values.md#25-chapter-declared-value-roots)). The code is `invalid-value-declaration`. Duplicate declarations retain [§FS-check.3.3](FS-check.md#33-duplicate-declaration) instead; a duplicate section may independently carry this finding, and home JSON input that [§FS-values.5.3](FS-values.md#53-incomplete-input-and-deterministic-output) counts as incomplete retains exit `2` ([§FS-check.2](FS-check.md#2-outputs)).

### 3.21 Invalid value binding

An attempted backtick-delimited binding that violates the exact grammar in [§FS-values.3.1](FS-values.md#31-the-only-binding-grammar), targets a value root of either origin or a declared value chapter heading itself, or descends below one of that root's immediate components is an error at the attempted form. Its code is `invalid-value-binding`. Unbackticked adjacency, bare citations, a delimited form aimed at an ordinary unmarked section or at a named section outside every declared chapter, and the same shape for a whole declaration whose kind lacks `values = true` are not attempts and remain ordinary prose/citations.

### 3.22 Value mismatch

After ordinary citation and section resolution succeeds uniquely, a binding whose authored component differs from its whole declaration or marked root under [§FS-values.4](FS-values.md#4-exact-equality) is an error at the binding and names the component site. Its code is `value-mismatch`; the canonical text and NDJSON parity are fixed by [§FS-values.5](FS-values.md#5-resolution-diagnostics-and-exit-status). An unknown alias, dangling ID, duplicate or invalid declaration, duplicate or missing section, invalid root, or noncanonical shorthand suppresses this comparison so one bad reference is never also reported as a mismatch. Ordinary declaration and section-structure findings run before comparison.

### 3.23 Section outside a declaration

When the scanner encounters a numeric section heading, or an enabled named section heading, that is deeper than its stale declaration context but whose line lies inside no declaration body, `check` emits one located error at the heading. A numeric heading's exact message is `numbered section outside any declaration`; an enabled named heading's is `named section outside any declaration`. Both use the public code `section-outside-declaration`. Ownership is the declaration's body span ([§FS-check.3.23.1](FS-check.md#3231-ownership-is-the-body-span)), the rejected heading leaves the section map every consumer reads ([§FS-check.3.23.2](FS-check.md#3232-the-rejected-heading-leaves-the-section-map)), and the finding is reported like any other hard error ([§FS-check.3.23.3](FS-check.md#3233-an-ordinary-hard-finding)).

#### 3.23.1 Ownership is the body span

Ownership is the body span already used for extraction and citing-side classification, not a second section-only approximation. In Markdown, a same-or-higher heading ends a declaration body even when that boundary heading is plain; a later deeper section-like heading is outside. In source, the end of a doc-comment or docstring ends ownership. A later declaration in the same comment block ends the earlier body and begins its own. An inline-spec stub owns only its single heading line, so numbered prose below the stub belongs to no stubbed declaration. A deeper section-like heading before any of those boundaries stays valid. A heading inside a Markdown fence is content and emits nothing ([§FS-show.2.5](FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example)). Legal unmarked or plain headings are not errors under this point; adopting a general unmarked-heading policy is a separate change ([§FS-check.4.14](FS-check.md#414-unmarked-markdown-heading)).

#### 3.23.2 The rejected heading leaves the section map

The rejected heading is excluded from the shared body-local section map before any consumer runs ([§FS-show.2.1.2](FS-show.md#212-section-map---toc)). It cannot resolve a citation or query, enter completion or list/size output, become an embedded-value root, participate in duplicate-section detection, or acquire an LSP navigation target. `show` and other failed queries retain their ordinary missing-section semantics rather than printing this check-only message.

#### 3.23.3 An ordinary hard finding

This is an ordinary hard finding under §[§FS-check.2](FS-check.md#2-outputs)–3. Text uses the located `<path>:<line>: error: <message>` form. JSON emits `{"severity":"error","path":<path>,"line":<line>,"code":"section-outside-declaration","message":<message>,"sites":null}`. `--only section-outside-declaration` retains it and `--ignore section-outside-declaration` removes it; a retained finding contributes exit `1`, while selecting it away restores the ordinary selected-report result. Parallel and workspace scans merge the record once under the workspace-relative path, never once per stale declaration. A narrowed scan judges the complete selected file, and `--full` applies the same code and message to otherwise out-of-scope files it adds. The LSP transports the same error severity, code, message, and heading range through its shared snapshot ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)).

### 3.24 Declaration-local section citation

Every candidate from [§FS-check.1.1.8](FS-check.md#118-declaration-local-numeric-section-candidates) receives one whole-token form verdict with code
`local-section-citation`:

- An owned numeric path is an error `local section citation <token>; write
  <marker><owner><separator><path>`. It remains a real edge for every graph consumer. A missing
  target section independently receives [§FS-check.3.2](FS-check.md#32-missing-section).
- An ownerless or genuinely ambiguous site is an error that says no enclosing declaration can be
  chosen and instructs the author to write a full citation or escape the illustration. It has no
  guessed ID or navigation target.
- A digit-starting mixed, named, or glued tail is an error naming the complete unsupported token
  and giving the same full-citation-or-escape guidance. No numeric prefix becomes an edge.

The finding covers the complete authored token for CLI and LSP ranges. An owned citation in a
location protected from automatic writing still names its manual full replacement; formatter
eligibility changes what can be rewritten, not whether persisted local form is canonical. The
same rule applies under configured markers and both strict modes.

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

A directory that declares `[workspace]` and that **no enclosing `[workspace]` block lists among its `members`** is claimed by nobody: the enclosing project's scan absorbs its subtree when it reaches it, while a run started *at* it names every project from itself ([§FS-workspace.6.1.8](FS-workspace.md#618-a-block-no-enclosing-block-lists-is-outside-the-chain)). The two scopes then spell the same projects differently — `c/FS-c` inside the block, `root/FS-c` at the repository root — so a citation passes the inner check and fails the run CI does, which is [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration) failing in the one place the alias-path model exists to hold it. Every run whose tree walk meets such a block says so. In `check` it is an **error**: it reaches exit `1` and prints on **stdout** behind the `<path>:<line>: error:` prefix of [§FS-check.2.1](FS-check.md#21-report-format), located at the block's `[workspace]` line ([§FS-check.3.29.13](FS-check.md#32913-in-check-one-of-the-reports-errors)) — the deprecation ramp the warning announced, landing on the release it named ([§FS-check.3.29.14](FS-check.md#32914-an-error-because-the-deadline-the-warning-named-has-arrived)). On the five other walking surfaces, which have no error channel for a fact about the run, it stays one CLI-level `warning:` on **stderr** ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages), [§FS-errors.2.2](FS-errors.md#22-cli-level-message)). Decided in [§DF-unlisted-workspace-block](../decisions/functional/DF-unlisted-workspace-block.md#df-unlisted-workspace-block-an-unlisted-workspace-block-is-reported-by-the-walk-that-meets-it).

What counts is [§FS-check.3.29.1](FS-check.md#3291-what-counts), and its edges are [§FS-check.3.29.2](FS-check.md#3292-an-unanswered-claim-is-not-reported-and-is-asked-quietly) to [§FS-check.3.29.5](FS-check.md#3295-include_root--false-changes-nothing). The message is [§FS-check.3.29.6](FS-check.md#3296-the-message) to [§FS-check.3.29.8](FS-check.md#3298-the-second-remedy-is-an-outcome-not-a-key); which commands report it, over what walk, is [§FS-check.3.29.9](FS-check.md#3299-every-command-that-walks-reports-it-not-check-alone) to [§FS-check.3.29.12](FS-check.md#32912-an-unlisted-block-outside-the-walk-is-unreported); its severity and rendering are [§FS-check.3.29.13](FS-check.md#32913-in-check-one-of-the-reports-errors) to [§FS-check.3.29.15](FS-check.md#32915-every-frontend-renders-it-anchored-at-the-blocks-workspace-line).

- **Code:** `unlisted-workspace-block` ([§FS-errors.5](FS-errors.md#5-json-format)).

#### 3.29.1 What counts

A directory the run's own walk reached, carrying a config under either discovery name ([§FS-config.1](FS-config.md#1-file-location-and-discovery)), whose config declares a `[workspace]` table, and whose canonical root no `[workspace]` block above it names among its `members` or `optional_members` entries. The claim question is the ancestor climb the claimed chain already runs ([§FS-workspace.6.1.7.2](FS-workspace.md#6172-the-quiet-climb-asks-the-same-ancestors)) — asked of a directory the walk found rather than of the run's own root — so it is read from `members` and `optional_members` entries alone and it climbs past the run's root exactly as that climb does. Both discovery names are probed at each walked *directory*, which is what finds the `.agents/grund.toml` form: the walk never descends into a hidden directory ([§FS-config.3.5](FS-config.md#35-scan--what-gets-walked)), so watching instead for walked *files* named `grund.toml` would find half the blocks and call the other half claimed. A tree with no enclosing `[workspace]` block anywhere above it is the same case rather than a milder one — nothing claims the block, so nothing gives the projects under it a stable alias path, and the enclosing scan absorbs them just the same.

#### 3.29.2 An unanswered claim is not reported, and is asked quietly

An ancestor that *names* the candidate among its `members` and then cannot answer it — a member list that will not expand, a config that will not parse — leaves the claim unanswered ([§FS-workspace.6.1.8.1](FS-workspace.md#6181-two-shapes-stay-unreported)) and the block unreported, because no answer is not the answer that nothing claims it. That silence has a floor: the claim is read off the entry text before anything is expanded, so a block no ancestor *names* is never silenced by an ancestor's breakage, however broken that ancestor is. And the question is asked **quietly** — [§FS-workspace.6.1.7.5](FS-workspace.md#6175-an-unobtainable-members-value-leaves-the-claim-undecidable)'s undecidable-claim warning belongs to the climb that spells an alias path out of the chain, which this rule does not do, so a run that would otherwise never ask the chain anything gains no line from having asked.

#### 3.29.3 Three neighbouring shapes are not this finding

A nested directory carrying a plain `grund.toml` with no `[workspace]` table declares no projects to absorb: it is ordinary tree to the enclosing walk ([§FS-check.1.3.1](FS-check.md#131-the-walk-covers-the-whole-config-root)) and nothing reports it. A block that *is* listed is inside the claimed chain at every depth, whatever the nesting. And **a project root of this run is never a candidate** — the run's own root, and each member root the walk stops at ([§FS-workspace.6](FS-workspace.md#6-nested-project-boundary)) — because those are the scopes the run names everything else from, and a block absorbs nothing into a namespace it is the namespace of. Without that exemption the rule would fire on the run's own root the moment `--full` made it a walk root ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)), which is every workspace repository that sits under no enclosing one — that is to say, almost all of them.

#### 3.29.4 Only the outermost block of a chain

A `[workspace]` block below an unlisted one *is* claimed — by the unlisted block — so the claim test answers it on its own, and listing the outer block puts the whole chain back in the claimed chain. One finding for one edit. One block the walk reached twice — under its own path and under a directory symlink to it — is one finding for the same reason: the claim test resolves both spellings to one root, one edit clears both, and the spelling reported is the first the walk met. Two unlisted blocks neither of which lists the other are two findings, because they are two edits.

#### 3.29.5 `include_root = false` changes nothing

The key answers "is this block's root a project?"; the finding asks "does anything claim this block?". Same finding, same message. What that key costs the block's *own* files is [§FS-check.4.10](FS-check.md#410-include_root--false-leaves-the-blocks-own-files-unread), a separate finding on a separate condition: the two can fire on one block, because being claimed by nobody and being read by nobody are different holes with different repairs.

#### 3.29.6 The message

The message carries the block, what the absorption costs, the two config edits that clear it, and the release it became an error in. In `check` the location is the prefix and the text opens at the finding ([§FS-check.3.29.7](FS-check.md#3297-the-location-sits-in-the-prefix-where-the-finding-is-located-and-inside-the-text-where-it-is-not)):

```
b/grund.toml:3: error: this [workspace] is listed by no enclosing workspace — the projects under it are absorbed into `root` instead of named under their own alias path; add "b" to [workspace] members in grund.toml, or keep it out of that project's [scan] — an unlisted [workspace] became an error in grund 0.15.0
```

On the five other walking surfaces of [§FS-check.3.29.9](FS-check.md#3299-every-command-that-walks-reports-it-not-check-alone) the same sentence keeps the location inside it, on stderr:

```
warning: b/grund.toml:3: this [workspace] is listed by no enclosing workspace — the projects under it are absorbed into `root` instead of named under their own alias path; add "b" to [workspace] members in grund.toml, or keep it out of that project's [scan] — an unlisted [workspace] became an error in grund 0.15.0
```

Two texts for two channels, and one word of the sentence is all the flip changed in either: `becomes` became `became` ([§FS-distribution.4.2](FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)).

#### 3.29.7 The location sits in the prefix where the finding is located, and inside the text where it is not

Where the finding is located — `check`'s report error ([§FS-check.3.29.13](FS-check.md#32913-in-check-one-of-the-reports-errors)) — the location is the bare `<path>:<line>:` prefix every other [§FS-check.3](FS-check.md#3-errors-detected) error wears, and it is not repeated inside the message text. That is [§FS-errors.2.2.1](FS-errors.md#221-a-location-inside-the-message-text)'s rule followed rather than an exception to it: a line opening with that prefix is the signal of a per-site finding on stdout, which is now what this is. Where the finding is *not* located — the CLI-level `warning:` the five other surfaces print — the location sits *inside* the message text, the shape [§FS-check.4.3.1](FS-check.md#431-a-cli-level-warning) and [§FS-config.4.3](FS-config.md#43-invalid-config-behavior) already use and the shape the undecidable-claim warning of [§FS-workspace.6.1.7.5](FS-workspace.md#6175-an-unobtainable-members-value-leaves-the-claim-undecidable) already prints for a neighbouring fact about a `[workspace]` block. `<line>` is the block's `[workspace]` line either way: the reader has two files to open and this is the one that is wrong. Every path in the line is rendered against the run's report base ([§FS-errors.3](FS-errors.md#3-message-text)), and `"b"` is the block's directory relative to the enclosing project's root, which is where both remedies are written. The absorbing project is named by the alias path this run spells it with, so the message can be matched against what [§FS-list](FS-list.md#fs-list-grund-lists-every-declared-id) printed.

#### 3.29.8 The second remedy is an outcome, not a key

The second remedy is stated as an outcome rather than as a key because which key carries it depends on the tree: `[scan] exclude` prunes descendants and never the directory a walk starts at ([§FS-check.1.3.2](FS-check.md#132-the-wider-walk-reads-a-superset-each-file-once)), so a block that is itself an `include` root leaves `include` as the edit, and a block below one takes `exclude`. Naming a key that clears the finding in one shape and not the other would be a remedy the reader has to argue with.

#### 3.29.9 Every command that walks reports it, not `check` alone

Every command whose run walks a project tree reports it: `check`, [§FS-list](FS-list.md#fs-list-grund-lists-every-declared-id), [§FS-refs](FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id), [§FS-cover](FS-cover.md#fs-cover-grund-groups-citations-by-scanned-file), [§FS-fmt](FS-fmt.md#fs-fmt-grund-normalizes-references-in-bulk), and the ID read of [§FS-show](FS-show.md#fs-show-grund-reads-a-single-declaration-body-by-id) — as an error in `check`, the one of the six with an error channel and the one where the alias-path guarantee is gated, and as the `warning:` line of [§FS-check.3.29.6](FS-check.md#3296-the-message) on the other five, whose exit codes [§FS-cli.5](FS-cli.md#5-exit-code-mapping-is-fixed) freezes and whose business a fact about the run is not. That is a deliberate choice, because [§FS-check.4.3.2](FS-check.md#432-reported-by-check-config-validate-and-config-show) draws the opposite line for the redundant config pair, and three reasons make it. The absorbed spelling is what `list` prints, so a `list` that shows `root/FS-c` where the block shows `c/FS-c` and says nothing is the same silence the finding exists to break. `refs` and `fmt --cross-refs` resolve qualified citations against the same project map, so they are equally wrong under an absorbed block. And [§FS-check.4.3.2](FS-check.md#432-reported-by-check-config-validate-and-config-show)'s line does not reach this fact: a redundant config pair is about *which file the run read*, while this is about *how every command in the tree spells its projects*, which is not a question only `check` asks.

#### 3.29.10 Knowable only from the walk

This finding differs from the neighbouring `[workspace]` cautions in one respect: a fact knowable at workspace-boundary population is knowable before any walk, so every command that merely *loads* the workspace can carry it. This one is knowable only from the walk that meets the nested config, so it is a property of **what the run walked** — carried by every command that walks, and silent wherever the walk does not reach the block. That is the earliest point at which the fact exists, not a second rule picked for convenience.

#### 3.29.11 The scope is the walk the run already makes

The roots `[scan] include` and the walked `[[kinds]]` homes give it, and their subtrees, minus member boundaries and what `[scan] exclude`, the ignore files, and hidden directories prune below a root — never a root itself ([§FS-config.3.5](FS-config.md#35-scan--what-gets-walked), [§FS-workspace.6](FS-workspace.md#6-nested-project-boundary)). No second walk is made for config files: the entries are already being enumerated, and the added work is one config probe per walked directory, which is what keeps [§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible) affordable here. It is also exactly the tree that gets absorbed — a block the scan never reaches absorbs nothing into the enclosing namespace.

#### 3.29.12 An unlisted block outside the walk is unreported

An unlisted block outside the run's walk is still unreported, a real limitation recorded rather than papered over. A narrowed `grund check <path>` ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)), a directory `[scan] exclude` prunes, a gitignored one, and a subtree behind a member boundary all leave a block unmet, and a run that cannot see something does not judge it — the same stance [§FS-check.3.18.8](FS-check.md#3188-a-run-that-cannot-see-the-index-does-not-judge-it) takes for an index the run did not scan. `grund check --full` widens the walk, so it reaches blocks the plain run does not and may report one more; that is the flag being additive ([§FS-check.1.3.4](FS-check.md#134-purely-additive)) about a caution rather than about a located finding, and it is the same edit [§FS-check.1.3.1](FS-check.md#131-the-walk-covers-the-whole-config-root) already recommends for a vendored or example project sitting inside the config root.

#### 3.29.13 In `check`, one of the report's errors

In `check` it is one of the report's errors. Like every error it reaches the exit code — a run whose walk meets an unlisted block exits `1` ([§FS-check.3](FS-check.md#3-errors-detected)) — and like every finding it stands in place of the `success` marker ([§FS-check.2.1.3](FS-check.md#213-the-success-marker)), which it already displaced as a warning, so nothing moves for a repository gating on that word. It is a **located** finding: it prints on **stdout** behind the `<path>:<line>: error:` prefix [§FS-check.2.1](FS-check.md#21-report-format) makes mandatory, and under `--format=json` it is one diagnostic on stdout whose `path` is the block's own config and whose `line` is its `[workspace]` line, with `sites` `null` ([§FS-errors.5](FS-errors.md#5-json-format)). Those two fields were `null` for as long as this was a CLI-level warning; populating them is the schema move [§DF-unlisted-workspace-error-shape](../decisions/functional/DF-unlisted-workspace-error-shape.md#df-unlisted-workspace-error-shape-check-and-the-editor-take-the-located-shape-the-five-walking-surfaces-keep-the-cli-level-line) argues under [§REQ-backwards-compatibility.4](../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise). `--ignore unlisted-workspace-block` suppresses the finding and with it the exit code, because selection applies to an error exactly as it did to the warning ([§FS-check.1.4](FS-check.md#14-selecting-diagnostics-with---only-and---ignore)); the `code` is unchanged, because a code identifies a rule and does not travel with a severity.

#### 3.29.14 An error, because the deadline the warning named has arrived

No `grund` command writes either remedy — both are config edits and a judgement about which one the repository wants — so this rule never had [§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)'s licence for a same-release verdict flip, and took [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)'s deprecation path instead. It arrived in `0.13.0` as a warning whose own text named the release it would become an error in, and `0.15.0` is that release, reached. So the finding is an error like every other in this section — it contributes to the exit code ([§FS-check.3](FS-check.md#3-errors-detected)) and it stands in place of the `success` marker ([§FS-check.2.1](FS-check.md#21-report-format)). The ramp bought what it was for: every `0.14.x` binary told a repository with an unlisted block, in the tool's own output, which release would start failing it. The same ramp, for the same reason, as [§FS-check.3.18.9](FS-check.md#3189-an-error-because-the-deadline-the-warning-named-has-arrived) — argued in [§DF-index-compatibility-ramp](../decisions/functional/DF-index-compatibility-ramp.md#df-index-compatibility-ramp-a-findings-ramp-follows-its-fix-command-not-the-size-of-the-offence).

The deadline clause is spent: a release still ahead is a date a reader can act on, and one that has arrived is not. What replaces it reports rather than promises, `— an unlisted [workspace] became an error in grund 0.15.0`, which is the past-tense half of the closed vocabulary [§FS-distribution.4.2](FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name) defines. It stays on both texts of [§FS-check.3.29.6](FS-check.md#3296-the-message) — the error's and the five surfaces' warning — because that clause is the only record of the flip a reader meets without the changelog, and it is what lets the release path refuse a version this finding would contradict. The ramp constant that held the named release ahead of the running version goes with the promise, and so does the unit test that read it: a test can hold only the pending half ([§FS-distribution.4.2.1](FS-distribution.md#421-a-test-can-hold-only-the-pending-half)), so what holds the landed half is an assertion on the clause's own bytes.

#### 3.29.15 Every frontend renders it, anchored at the block's `[workspace]` line

In `check` it is a located report error ([§FS-check.3.29.13](FS-check.md#32913-in-check-one-of-the-reports-errors)), and the location is the finding's own field: `path` is the block's config, `line` is its `[workspace]` line. **The anchor is the one thing the flip did not move** — it is the same `<path>:<line>` the warning named, and it is still carried as a field rather than left to be read back out of message text. What moved is which channel carries it.

On the five other surfaces of [§FS-check.3.29.9](FS-check.md#3299-every-command-that-walks-reports-it-not-check-alone) the diagnostic travels in the run's warning channel and states its location *inside* the text ([§FS-check.3.29.7](FS-check.md#3297-the-location-sits-in-the-prefix-where-the-finding-is-located-and-inside-the-text-where-it-is-not)) and nowhere else: the CLI is the only frontend those five have, and it prints the sentence rather than placing it, so a location field there would be carried for nobody. The editor therefore takes this finding from `check`'s report, which is where the anchor is, and publishes it as an **error** on that `[workspace]` line ([§FS-lsp.1.1.3](FS-lsp.md#113-workspace-warnings-on-the-runs-warning-channel), [§FS-lsp.4](FS-lsp.md#4-determinism-and-parity-with-the-cli)) — one text and one location reaching every frontend from one place ([§FS-distribution.3.1](FS-distribution.md#31-rust-grund-core-crate)), as they already did, with the channel changed underneath. The three sibling `[workspace]` cautions stay on the run's channel and keep their anchors there, having no error channel to move into.

## 4. Warnings

**`4.8` is vacant.** The unlisted-`[workspace]` rule held that number until the ramp of [§FS-check.3.29.14](FS-check.md#32914-an-error-because-the-deadline-the-warning-named-has-arrived) ended and the finding became an error; it is [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) now, under the heading its verdict belongs to. Its siblings kept the numbers they had rather than closing the gap: a section number is an address inside this repository and not a promise outside it, so a vacancy costs a reader this sentence while renumbering `4.9` to `4.14` would have moved 371 citations in 152 files to say the same thing.

### 4.1 Unused declaration

An ID that is declared but never cited. Reported as a warning, not an error — newly declared IDs may not yet have citations. Warnings never affect the exit code ([§FS-check.2](FS-check.md#2-outputs)). A resolving shorthand counts as a citation ([§FS-check.4.1.1](FS-check.md#411-a-resolving-shorthand-counts)), a kind's own index entry does not ([§FS-check.4.1.2](FS-check.md#412-an-index-entry-does-not-count)), and `E2E` declarations are exempt ([§FS-check.4.1.3](FS-check.md#413-e2e-declarations-are-exempt)).

#### 4.1.1 A resolving shorthand counts

A number-only shorthand citation that resolves counts here like any other citation ([§FS-check.1.2.4](FS-check.md#124-a-resolved-shorthand-is-a-real-edge)): a declaration abbreviated as `§FS-042` everywhere is cited, and reporting it as unused would state the opposite of the truth.

#### 4.1.2 An index entry does not count

A citation that is a kind's own **index entry** ([§FS-check.3.18.5](FS-check.md#3185-what-an-entry-is)) does not count here. An index names every declaration in its folder by construction, so counting its entries would leave every ID in an indexed folder permanently cited and delete the signal this warning exists to give ([§DF-index-not-an-inbound-citation](../decisions/functional/DF-index-not-an-inbound-citation.md#df-index-not-an-inbound-citation-an-index-entry-is-navigation-not-use)). The exclusion is exactly the entry: a citation in an index file of an ID whose home lies *outside* that folder is an ordinary citation and counts like any other **unless that exact site is the canonical link that enrolls an external inline declaration** ([§FS-check.3.18.3](FS-check.md#3183-an-external-inline-declaration-enrolls-by-its-canonical-link)). Other citations of the enrolled ID on the same page still count. `grund refs` is unaffected and still lists every index entry — they are real citations, and a reader asking who points at an ID wants to be told that its index does.

#### 4.1.3 `E2E` declarations are exempt

`E2E` declarations ([AR-scanner.6](../architecture/AR-scanner.md#6-e2e-case-declarations)) are exempt: an end-to-end case is exercised by being run, not by being cited, so a `§E2E-<name>` that nothing references is not a warning. Every other kind is subject to this rule, including Markdown and JSON value declarations; the citation inside a recognized value binding counts as a use ([§FS-values.3.2](FS-values.md#32-recognized-text-contexts)). `grund list --unused` ([§FS-list](FS-list.md#fs-list-grund-lists-every-declared-id)) uses the same default signal and suppresses uncited `E2E` cases unless `E2E` is explicitly selected with `--kind` (including a multi-kind filter such as `--kind FS,E2E`).

### 4.2 Inline note soft-cap overrun *(opt-in)*

Off by default. When `[reference] warn_on_suggested = true` is set in the project's `grund.toml` ([§FS-config.3.1](FS-config.md#31-reference--citation-form)), an inline citation site whose line count exceeds `inline_note_suggested_lines` but stays within `inline_note_max_lines` is reported as a warning. The full contract — what counts as a soft-cap overrun and how it interacts with the hard-cap error in [§FS-check.3.10](FS-check.md#310-inline-citation-style-violation) — lives in [§FS-inline-citation-style.4.2](FS-inline-citation-style.md#42-warnings--opt-in-soft-cap). Off by default because the soft cap is primarily agent-facing guidance ([§FS-inline-citation-style.5](FS-inline-citation-style.md#5-agent-facing-rendering)); flipping the toggle escalates it to a `check`-time signal.

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

Off by default. When `[reference] inline_note_layout` names a layout and `[reference] inline_note_layout_check = "warn"` ([§FS-config.3.1](FS-config.md#31-reference--citation-form)), every line of an inline citation site that [§FS-inline-citation-style.3.3.1](FS-inline-citation-style.md#331-per-line-not-per-site--and-only-where-a-note-opens) judges and that does not match the configured form is reported as a warning, anchored at that line. Setting the key to `"error"` reports the identical message as an error instead ([§FS-check.3.10](FS-check.md#310-inline-citation-style-violation)); `"off"`, or a `inline_note_layout` left at `any`, reports nothing. The form itself, the per-line rule, and the exemption for sites that carry no note live in [§FS-inline-citation-style.3.3](FS-inline-citation-style.md#33-inline_note_layout--where-the-citations-sit), and the channel table in [§FS-inline-citation-style.4.4](FS-inline-citation-style.md#44-warnings-and-errors--opt-in-layout-deviations).

Off by default because a layout is a house style rather than a correctness property, and the two levels exist so a repository can migrate on `warn` before it gates on `error` — the same ladder [§FS-check.4.2](FS-check.md#42-inline-note-soft-cap-overrun-opt-in) gives the soft cap.

### 4.5 Nothing recognized

A walk that read at least one file and recognized **nothing in it** — no declaration and no citation — is [§FS-check.2.2](FS-check.md#22-empty-scan)'s empty scan one step further in: the scope was right and the files were read, and the grammar matched none of their content. `check` then emits one CLI-level `warning:` line ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)) on **stderr**, whose text is [§FS-check.4.5.2](FS-check.md#452-the-message). It has two usual causes ([§FS-check.4.5.1](FS-check.md#451-the-usual-causes)), is asked per project ([§FS-check.4.5.3](FS-check.md#453-asked-per-project)) and only of a run over that project's root ([§FS-check.4.5.4](FS-check.md#454-asked-only-of-the-whole-project)), and is withheld beside any other finding about the scope ([§FS-check.4.5.5](FS-check.md#455-a-warning-withheld-beside-any-other-finding-about-the-scope)); the per-heading half is [§FS-check.4.6](FS-check.md#46-declaration-near-miss)'s ([§FS-check.4.5.6](FS-check.md#456-the-per-heading-half)). Decided in [§DF-nothing-recognized](../decisions/functional/DF-nothing-recognized.md#df-nothing-recognized-a-run-that-recognized-nothing-says-so-and-says-it-as-a-warning).

- **Code:** `nothing-recognized` ([§FS-errors.5](FS-errors.md#5-json-format)), with `path` and `line` null like every CLI-level diagnostic.

#### 4.5.1 The usual causes

The usual causes are a tree nobody has declared in yet and a docs tree whose headings open with a prefix that is no configured kind ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)), either of which leaves every heading in the tree a non-declaration and the run's verdict `success` over a repository where nothing is grounded.

#### 4.5.2 The message

The line names how many files were read, the shape a declaration heading and a citation take under the configured format, and the configured `[[kinds]]` prefixes:

```
warning: nothing recognized — grund read 3 files and found no declaration and no citation in them. A declaration heading reads `# <KIND>-<NNN>-<slug>: <title>` and a citation `<marker><KIND>-<NNN>-<slug>`, under [id] format = "{kind}-{number}-{slug}" with <KIND> one of {AR, FS}. Either nothing is declared yet, or the headings are written to a different shape than that.
```

The shapes are rendered from the `[id] format` template, the same substitution [§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints) makes for the managed entrypoint block, and the citation shape carries the configured marker ([§FS-config.3.1](FS-config.md#31-reference--citation-form)). The closing sentence offers both readings of the fact, because the run cannot tell them apart without judging a line: a tree written to another format and a `grund init` scaffold nobody has declared in yet produce the identical report, and naming only the first would send a fresh adopter looking for a bug in a config that is fine. No example ID is built from `[id] number_pattern` and `[id] slug_pattern` and no corrected ID is proposed for any heading: `check` reports facts about the tree and the config ([§FS-check.3](FS-check.md#3-errors-detected) vs [§FS-check.4](FS-check.md#4-warnings)), and an ID assembled from those patterns would be a guess at what they accept.

#### 4.5.3 Asked per project

The question is asked **per project**, like [§FS-check.2.2](FS-check.md#22-empty-scan): in a workspace ([§FS-workspace.5](FS-workspace.md#5-command-scope)) each project is judged against its own config, since one project's grammar mismatch says nothing about another's. It asks *recognized*, not *declared* — a member that only cites another member's specs declares nothing and is working as intended, so a citation anywhere in the project answers the question.

#### 4.5.4 Asked only of the whole project

It is asked only of a run whose scope **is** that project's root (no path argument, or a path that resolves to it). A narrowed `grund check <dir>` is a slice the caller chose, and a slice holding no declaration and no citation is an answer rather than a misconfiguration — the claim this caution makes is about a whole project, and a run that read part of one cannot make it.

#### 4.5.5 A warning, withheld beside any other finding about the scope

Like [§FS-check.2.2](FS-check.md#22-empty-scan) it is a warning, and like [§FS-check.2.2](FS-check.md#22-empty-scan) it is withheld from a run that has any other finding about the configured scope: the exit code stays `0` (a tree with nothing in it yet is the ordinary first day of a repository), and a report that already says something about that scope is not the silent verdict this rule exists to break. It inherits every exception [§FS-check.2.2.3.1](FS-check.md#2231-findings-that-do-not-suppress-it) lists unchanged, and for the same reasons. A redundant-config pair ([§FS-check.4.3](FS-check.md#43-redundant-config-pair)) is a fact about which file was read — and a repository mid-migration between the two config names is exactly where a mismatched `[id] format` hides, in the file that is no longer read. The out-of-scope tier ([§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only)) is a fact about the tree beyond the scope, and a `--full` run that reports every citation out there while the configured scope holds nothing is the strongest form of this diagnosis, not a reason to withhold half of it. What it buys is the `success` marker — a warning stands in its place ([§FS-check.2.1](FS-check.md#21-report-format)), so the run that recognized nothing stops printing the same word as the run that checked everything.

#### 4.5.6 The per-heading half

The per-heading half — naming each heading that looks like a declaration and does not match — is [§FS-check.4.6](FS-check.md#46-declaration-near-miss), a different rule asking a different question: this one is arithmetic over what the scan recorded, that one is about what a single line came close to being. Where both could speak, [§FS-check.4.6](FS-check.md#46-declaration-near-miss) does and this one is withheld under the rule above, because "these two headings, at these lines" is the same fact said usefully.

### 4.6 Declaration near miss

A heading that opens the way a declaration does and does not match its effective ID format remains a declaration for read compatibility ([§FS-config.3.2](FS-config.md#32-id--id-grammar)). The classic stumble is `# FS-login: …` under the default `{kind}-{number}-{slug}` — the `-NNN-` left out. Before grund 0.15.0, `check` emits one **warning** per such declaration, at the line a contributor has to edit:

```
docs/spec.md:1: `FS-login` resolves for compatibility but does not match [id] format = "{kind}-{number}-{slug}" — rename it or change the effective format; this warning becomes an error in grund 0.15.0
```

What counts is [§FS-check.4.6.1](FS-check.md#461-what-counts): the declaration colon is its discriminator ([§FS-check.4.6.2](FS-check.md#462-the-declaration-colon-is-the-discriminator)), and inline code, prose and fenced blocks never count ([§FS-check.4.6.3](FS-check.md#463-never-in-inline-code-prose-or-a-fenced-block)). The message states facts rather than a guessed rename ([§FS-check.4.6.4](FS-check.md#464-facts-not-a-guessed-rename)), the warning becomes an error in 0.15.0 ([§FS-check.4.6.5](FS-check.md#465-a-warning-before-0150-an-error-in-it)), and there is no opt-out ([§FS-check.4.6.6](FS-check.md#466-no-opt-out-no-rewrite)).

- **Code:** `declaration-near-miss` ([§FS-errors.5](FS-errors.md#5-json-format)).

#### 4.6.1 What counts

A line in declaration position — a Markdown heading, or a comment-prefixed line in a source file under the rules of [AR-scanner.4](../architecture/AR-scanner.md#4-inline-declarations-in-language-doc-comments) — whose first token unambiguously begins with a configured citable kind ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)), which the effective ID grammar rejects, and which is **followed by the declaration colon**. This also covers a per-kind format whose first literal after `{kind}` differs from the persisted token.

#### 4.6.2 The declaration colon is the discriminator

A line opening with an ID-shaped token and no colon is prose far more often than it is a declaration attempt — a comment wrapped across lines whose continuation begins with one is the case that proved it, in this repository's own source. So the rule reads exactly the shape a declaration attempt has, `<KIND>-…: <title>`, and says nothing about the rest. A near miss written without a title is not reported; that is the cost, and it buys a rule that stays quiet on prose.

#### 4.6.3 Never in inline code, prose, or a fenced block

The token stops at a backtick, so an inline-code mention is not a near miss. The position rules are the declaration rules exactly, so a near miss is only ever read where a declaration would have been: a bare `FS-login: …` in Markdown prose is not one ([§DF-code-declarations-drop-hash](../decisions/functional/DF-code-declarations-drop-hash.md#df-code-declarations-drop-hash-code-resident-declarations-may-drop-the--prefix)), and neither is anything inside a fenced block.

#### 4.6.4 Facts, not a guessed rename

The message names the token as written and the effective template, states that lookup remains compatible, and offers the two real migration choices: rename the declaration and its citations, or change the effective format. It does **not** propose a corrected ID; assembling one from component patterns would guess what the author meant.

#### 4.6.5 A warning before 0.15.0, an error in it

Before 0.15.0 it is a warning, so like every warning it leaves the exit code alone ([§FS-check.2](FS-check.md#2-outputs)): a run with no errors exits successfully but prints the located warning and no `success` marker ([§FS-check.2.1.3](FS-check.md#213-the-success-marker)). The declaration still appears in `list` and resolves through every reader; severity never changes recognition. In grund 0.15.0 the same code and location become an error, the deadline clause becomes the past-tense release report required by [§FS-distribution.4.2](FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name), and `check` exits `1`. The release guard and [§RM-off-grammar-declaration-error](../roadmap.md#rm-off-grammar-declaration-error-make-off-grammar-declarations-a-check-error-in-0150) prevent shipping the warning at or beyond that version.

#### 4.6.6 No opt-out, no rewrite

There is no line-oriented opt-out or automatic rewrite: the position and colon rules bound recognition, and migration remains the repository author's choice.

### 4.7 A workspace member swallows the block's own scan

A `[workspace]` block every one of whose walk roots lies inside one of its own members ([§FS-workspace.2.1](FS-workspace.md#21-a-member-that-swallows-the-blocks-own-scan)) reads nothing at all, and said nothing about it: `grund list` completed silently and exited `0`, and `check` offered only the empty-scan caution of [§FS-check.2.2](FS-check.md#22-empty-scan) — which names `[scan] include`, the key that is usually already correct, so the one message the run did produce pointed away from the entry that caused it.

`grund` emits one CLI-level `warning:` line ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)) on **stderr**, whose content is [§FS-check.4.7.1](FS-check.md#471-the-message), from every command that walks ([§FS-check.4.7.2](FS-check.md#472-every-command-that-walks-says-it-not-just-check)), with every block spelled from the run's own root ([§FS-check.4.7.3](FS-check.md#473-one-run-spells-the-whole-tree-from-one-place)), `grund init` included ([§FS-check.4.7.4](FS-check.md#474-grund-init-names-the-blocks-below-its-target-from-the-target)). It stands beside the empty-scan caution ([§FS-check.4.7.5](FS-check.md#475-beside-the-empty-scan-caution-not-in-place-of-it)), keeps its text under `--format json` ([§FS-check.4.7.6](FS-check.md#476-a-launch-time-message-keeps-its-text-under---format-json)), travels as one of the run's warnings to every frontend ([§FS-check.4.7.7](FS-check.md#477-one-of-the-runs-warnings-rendered-by-every-frontend)), and becomes an error in the next release ([§FS-check.4.7.8](FS-check.md#478-a-warning-in-this-release-an-error-in-the-next)).

#### 4.7.1 The message

The line carries the block's `members` line as its breadcrumb the way a config error does ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)), then each covered root and the member entry it is inside, in config order. The member entry is named **as the config wrote it**; the covered root is named by its path **under the block root**, which is that spelling normalized rather than the spelling itself — an `include = ["./docs/"]` entry is named `docs`. Neither is the resolved path: that renders as nothing when it equals the render base and as an absolute path when it does not, and an author can edit neither ([§FS-errors.4](FS-errors.md#4-determinism)):

```
warning: grund.toml:16: [workspace] members swallows this project's whole scan — every scan root is inside a member: `docs` in `docs` — so its declarations are unreachable and its citations are never checked. Point [scan] include at a directory that is not a member, or set include_root = false. This becomes an error in grund 0.15.0.
```

#### 4.7.2 Every command that walks says it, not just `check`

The question is asked where a run populates a block's member boundary, so `grund check`, `list`, `refs`, `cover`, `fmt`, and every other command that resolves that boundary carry it. A silent-scan defect only `check` reports is half-reported: the other surfaces are exactly where the repository looks fine. It is emitted **once per block per run**, ahead of any finding, and asked of every block in a nested tree against that block's own `members` line, rendered from the run's own root ([§FS-check.4.7.3](FS-check.md#473-one-run-spells-the-whole-tree-from-one-place)).

#### 4.7.3 One run spells the whole tree from one place

Every line is rendered against the root this run was launched at, like every diagnostic from a block above the run's root ([§FS-errors.3](FS-errors.md#3-message-text)), and that base is the same for every block the run reaches — the ones above it, the one it is rooted at, and the ones below it alike. Most commands never have to think about it: a run narrowed into a member is re-rooted onto that member before it walks, so the top of the tree it expands *is* where it was launched.

#### 4.7.4 `grund init` names the blocks below its target from the target

`grund init` is the exception to [§FS-check.4.7.3](FS-check.md#473-one-run-spells-the-whole-tree-from-one-place)'s rooting, because it expands the outermost workspace above its target in order to teach the alias set. Its blocks below the target are still named from the target: run inside a member of a three-deep absorbed tree, the lines read `../grund.toml`, `grund.toml`, `sub/grund.toml`, and a reader resolves every one of them against the directory they are standing in. Re-basing them onto the workspace root instead would print a path that exists from there and is the wrong file, which is the defect [§FS-workspace.6.1.7](FS-workspace.md#617-a-claiming-block-that-cannot-answer-fails-the-run) already forbids for an ancestor's `members` line.

#### 4.7.5 Beside the empty-scan caution, not in place of it

[§FS-check.2.2](FS-check.md#22-empty-scan) is about a walk that read nothing; this is about a configuration that can read nothing. A `check` over an absorbed block prints both, this one first, and [§FS-check.2.2](FS-check.md#22-empty-scan)'s text is unchanged — it is stable phrasing ([§FS-errors.3](FS-errors.md#3-message-text)) and a repository grepping for it keeps what it had.

#### 4.7.6 A launch-time message keeps its text under `--format json`

It is a launch-time message carried in the run's warning channel ([§FS-check.4.7.7](FS-check.md#477-one-of-the-runs-warnings-rendered-by-every-frontend)) rather than in `check`'s report, so it keeps its text under `--format json` ([§FS-errors.5.2.2](FS-errors.md#522-launch-time-messages-stay-text)), like the undecidable-claim warning of [§FS-workspace.6.1.7.5](FS-workspace.md#6175-an-unobtainable-members-value-leaves-the-claim-undecidable) and every other warning that channel carries. It therefore carries no JSON `code` and no selector of its own ([§FS-check.1.4](FS-check.md#14-selecting-diagnostics-with---only-and---ignore)). Like every warning it leaves the exit code alone ([§FS-check.2](FS-check.md#2-outputs)), and like every warning it stands in place of the `success` marker ([§FS-check.2.1.3](FS-check.md#213-the-success-marker)).

#### 4.7.7 One of the run's warnings, rendered by every frontend

The fact is settled during workspace expansion, before any report exists, but what the engine hands back is still a diagnostic in the run's warning channel — carried on whatever the walking command returns, and rendered by whichever frontend asked for it, exactly as [§FS-check.2.2](FS-check.md#22-empty-scan)'s empty-scan caution already is ([§FS-distribution.3.1](FS-distribution.md#31-rust-grund-core-crate)). It **anchors at the block's `members` line**: the `grund.toml:<line>` breadcrumb of [§FS-check.4.7.1](FS-check.md#471-the-message) is the finding's own location too, so a frontend places it without parsing the message, while [§FS-check.2.1](FS-check.md#21-report-format)'s `<path>:<line>:` prefix stays off it, because a fact about the run's configuration is not a finding at a site in the citation graph. No byte a reader has today moves: the CLI prints the line of [§FS-check.4.7.1](FS-check.md#471-the-message) on stderr in [§FS-check.2.1.1](FS-check.md#211-cli-level-messages)'s shape, `--format json` keeps that same text ([§FS-check.4.7.6](FS-check.md#476-a-launch-time-message-keeps-its-text-under---format-json)), and a run that earns it still prints no `success`. What changes is who else hears it — an editor publishes it on that `grund.toml` line, where before it published nothing at all ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)).

#### 4.7.8 A warning in this release, an error in the next

No `grund` command repairs it — the fix is a choice between repointing `[scan] include` and declaring the block no project — so [§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)'s single-release licence does not apply and [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)'s deprecation path does: the message names the release the finding becomes an error in. That release is [§RM-workspace-absorbed-scan-error](../roadmap.md#rm-workspace-absorbed-scan-error-flip-the-absorbed-scan-warning-to-an-error), and a test holds it ahead of the running version so the deadline cannot pass unnoticed — the same guard [§FS-check.3.18.9](FS-check.md#3189-an-error-because-the-deadline-the-warning-named-has-arrived) carried until its own release arrived, for the same reason. Decided in [§DF-absorbed-scan-warning](../decisions/functional/DF-absorbed-scan-warning.md#df-absorbed-scan-warning-a-scan-its-own-members-swallowed-is-a-warning-with-a-named-release-not-an-error).

#### 4.7.9 A later expansion refusal does not discard it

Once the run's root block settles this warning, a later workspace-expansion refusal does not discard it: the warning is returned and rendered exactly once, before the refusal ([§FS-check.4.7.7](FS-check.md#477-one-of-the-runs-warnings-rendered-by-every-frontend)). This is the root-side counterpart to [§FS-check.4.10.8](FS-check.md#4108-a-failed-workspace-expansion-withholds-it)'s bounded silence for questions below the root that a failed expansion leaves unanswered.

### 4.9 A workspace member declared optional is absent

A member listed in `[workspace] optional_members` whose path is not a directory in this checkout ([§FS-workspace.2.2](FS-workspace.md#22-a-member-that-may-be-legitimately-absent)). The namespace it would have contributed was not read: no declaration in it reached a catalog, and no citation into it was resolved or reported ([§FS-workspace.4](FS-workspace.md#4-resolution)). The run is a report about less of the repository than it looks like, and saying so is this finding's whole job.

It is one located warning per absent entry ([§FS-check.4.9.1](FS-check.md#491-one-warning-per-absent-entry)), naming the entry and its namespace ([§FS-check.4.9.2](FS-check.md#492-the-entry-as-written-the-namespace-by-its-alias-path)) and no remedy ([§FS-check.4.9.3](FS-check.md#493-it-names-no-remedy-because-nothing-here-is-broken)). Unlike [§FS-check.4.7](FS-check.md#47-a-workspace-member-swallows-the-blocks-own-scan) and [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) it is printed on stdout ([§FS-check.4.9.4](FS-check.md#494-a-located-finding-on-stdout-not-a-cli-level-caution)), because the exit code cannot carry it ([§FS-check.4.9.5](FS-check.md#495-the-exit-code-forces-the-shape)); it names no release ([§FS-check.4.9.6](FS-check.md#496-it-names-no-release)), and its JSON form is [§FS-check.4.9.7](FS-check.md#497-its-json-form-is-on-stdout).

- **Code:** `optional-member-absent` ([§FS-errors.5](FS-errors.md#5-json-format)).

#### 4.9.1 One warning per absent entry

One warning per absent entry, in the report's ordinary sort ([§FS-errors.4](FS-errors.md#4-determinism)) — by entry name, since all of them share one path and line — anchored at the `optional_members` line of the block that holds it:

```
grund.toml:5: optional workspace member `vendored` is absent — citations into namespace `vendored` were not checked, so this run does not cover it
```

#### 4.9.2 The entry as written, the namespace by its alias path

The entry is named **as the config wrote it** and the namespace by the **whole alias path this run spells it with** — `vendored` and `sub/vendored` for one entry inside a nested block — which is the pair [§FS-check.4.7.1](FS-check.md#471-the-message) and [§FS-check.3.29.7](FS-check.md#3297-the-location-sits-in-the-prefix-where-the-finding-is-located-and-inside-the-text-where-it-is-not) already use, for the same two reasons: the entry is what an author can edit ([§FS-workspace.2](FS-workspace.md#2-workspace-configuration)), and the alias path is what a citation has to write ([§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)).

#### 4.9.3 It names no remedy, because nothing here is broken

Most warnings in [§FS-check.4](FS-check.md#4-warnings) end in an edit, because they report a configuration that says something its author did not mean. This one reports a state the author declared in advance and a checkout that happens to be partial; the only thing that would "fix" it is a checkout with the member in it, which is not grund's to ask for and is often not available where the run happens. So the message stops at the fact.

#### 4.9.4 A located finding on stdout, not a CLI-level caution

It is a located finding on stdout, which departs deliberately from its two nearest neighbours — [§FS-check.4.7](FS-check.md#47-a-workspace-member-swallows-the-blocks-own-scan) and [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) both point at a line of their block's config, and [§FS-check.4.7](FS-check.md#47-a-workspace-member-swallows-the-blocks-own-scan) prints as a CLI-level `warning:` on stderr ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)) — as [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) still does on the five walking surfaces that have no report to put it in ([§FS-check.3.29.9](FS-check.md#3299-every-command-that-walks-reports-it-not-check-alone)), though in `check` it is an error now ([§FS-check.3.29.13](FS-check.md#32913-in-check-one-of-the-reports-errors)). Those two report a **misconfiguration**, and the reason for that CLI-level form is that every command in the tree is wrong under them: a block that reads nothing makes `grund list` silent, an unlisted block makes every command spell the same project two ways. So both are carried by every command that walks — [§FS-check.4.7](FS-check.md#47-a-workspace-member-swallows-the-blocks-own-scan) emitted where the workspace loads, [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) where the walk meets the block ([§FS-check.3.29.10](FS-check.md#32910-knowable-only-from-the-walk)). This finding is not that. Nothing is misconfigured — the repository said this may happen, and it happened — and no other command's output is wrong, because a namespace that is not there has nothing to list and nothing to point at. What is at stake is only the verdict `check` renders, and a statement about the coverage of a report belongs in the report.

#### 4.9.5 The exit code forces the shape

[§FS-check.2](FS-check.md#2-outputs) gives grund one way to say "do not trust this report as complete", and it is exit `2`. This is the one case where a run is deliberately incomplete and still exits `0` ([§FS-workspace.2.2](FS-workspace.md#22-a-member-that-may-be-legitimately-absent)), so the exit code carries nothing here and stdout has to. A CLI-level line would leave stdout saying only that the `success` marker was withheld — a signal made of an absence, which under `2>/dev/null` or a folded CI log is indistinguishable from silence, and which is exactly the warning that scrolls past. The `<path>:<line>:` prefix is also honest here in a way it is not for [§FS-check.4.3.1](FS-check.md#431-a-cli-level-warning): there is one line, in a file the repository wrote, and it is the line a reader has to open to understand the run. `success` is withheld all the same, because it is withheld for every warning ([§FS-check.2.1.3](FS-check.md#213-the-success-marker)) — but that is now a consequence of the finding rather than the whole of it.

#### 4.9.6 It names no release

[§FS-check.4.7](FS-check.md#47-a-workspace-member-swallows-the-blocks-own-scan) is a warning on the way to being an error ([§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)), and [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) has become one ([§FS-check.3.29.14](FS-check.md#32914-an-error-because-the-deadline-the-warning-named-has-arrived)). This one is permanent. An unverified namespace is not a state to be migrated off; it is the standing price of the opt-out, paid on every run of every checkout that takes it, and a repository that stops wanting to pay it deletes the entry.

#### 4.9.7 Its JSON form is on stdout

Under `--format=json` it is one warning diagnostic on **stdout** with `path` and `line` set and `sites` null, like every other located finding ([§FS-errors.5](FS-errors.md#5-json-format)) — which is the other half of what the CLI-level shape would have cost, since a consumer filtering the report for coverage facts would have had to parse text on a second stream to find this one.

### 4.10 `include_root = false` leaves the block's own files unread

A `[workspace]` block that sets `include_root = false` is not a project, and its own files are read by nobody ([§FS-check.4.10.1](FS-check.md#4101-nobody-reads-the-blocks-own-files)): a declaration there reaches no catalog and a citation there is never checked, which is [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration) failing through one config key. `grund` emits one CLI-level `warning:` ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)) on **stderr** for such a block **whose own tree actually holds a file a scan would have read**. Decided in [§DF-unread-opted-out-block](../decisions/functional/DF-unread-opted-out-block.md#df-unread-opted-out-block-the-unread-files-of-an-opted-out-block-are-a-conditional-warning-that-never-ramps).

What counts is [§FS-check.4.10.2](FS-check.md#4102-what-counts), and [§FS-check.4.10.3](FS-check.md#4103-a-block-with-no-member-in-scope-is-not-this-finding) and [§FS-check.4.10.4](FS-check.md#4104-what-stays-silent-and-why-the-silence-is-the-point) are what does not. The message is [§FS-check.4.10.5](FS-check.md#4105-the-message), and it names one root ([§FS-check.4.10.6](FS-check.md#4106-one-root-not-every-root)). Every command that walks says it, once per block ([§FS-check.4.10.7](FS-check.md#4107-every-command-that-walks-says-it-and-each-block-says-it-once)), except over a failed workspace expansion ([§FS-check.4.10.8](FS-check.md#4108-a-failed-workspace-expansion-withholds-it)). It keeps its text under `--format json` ([§FS-check.4.10.9](FS-check.md#4109-a-launch-time-message-keeps-its-text-under---format-json)), stands in place of `success` without moving the exit code ([§FS-check.4.10.10](FS-check.md#41010-it-stands-in-place-of-the-success-marker-and-never-moves-the-exit-code)), reaches every frontend ([§FS-check.4.10.11](FS-check.md#41011-one-of-the-runs-warnings-rendered-by-every-frontend)), and never becomes an error ([§FS-check.4.10.12](FS-check.md#41012-a-warning-permanently-with-no-release-it-becomes-an-error-in)).

#### 4.10.1 Nobody reads the block's own files

An intermediate opted-out block is still a segment in every alias path below it ([§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)), yet its own files are read by nobody: it has no scan of its own, an enclosing scan, where there is one, stops at the member boundary ([§FS-workspace.6](FS-workspace.md#6-nested-project-boundary)), and `--full` ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)) widens a project's scope and has no project to widen here. No run said so: over a grouping directory holding two dangling citations, `check`, `check --full` and `list` were all silent and exited `0` ([grund#71](https://github.com/agent-grounds/grund/issues/71)).

#### 4.10.2 What counts

**What counts is one question: would this block have read something, had it been a project?** That is the mirror of [§FS-workspace.2.1](FS-workspace.md#21-a-member-that-swallows-the-blocks-own-scan), which asks whether a block that *is* a project reads nothing — so one notion of "this block's own scope" answers both, and a `[[kinds]]` home or an unwalked home moves the two rules together. Take the block's **default scope** — the roots `[scan] include` and the walked `[[kinds]]` homes give it ([§FS-config.3.5](FS-config.md#35-scan--what-gets-walked)), the set [§FS-workspace.6](FS-workspace.md#6-nested-project-boundary)'s boundary prunes, asked of the default scope whatever `--full` says, because this is a property of the configuration and not of one walk. Drop a root that is not on disk: the walk skips it before it prunes, so it is read by nobody and costs nobody anything. Drop a root at or inside one of the block's own expanded member roots: those files *are* read, by the member — compared as canonical paths, the way the walk's own prune compares them. On what is left, probe for one file the scan would have read, under the block's own `[scan] extensions`, `exclude`, ignore files and hidden-name rules applied exactly as the scanner applies them ([§FS-config.3.5](FS-config.md#35-scan--what-gets-walked)). The probe **stops at the first hit**: the cost is "is there one file here", not the size of the tree, and only a block that opted out ever pays it ([§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible)).

#### 4.10.3 A block with no member in scope is not this finding

`include_root = false` with no members — an absent `members` key, an empty list, or a non-empty list of globs that match no directories, beside no `optional_members` entry ([§FS-workspace.2.2](FS-workspace.md#22-a-member-that-may-be-legitimately-absent)) — is already a config error at that block's own line ([§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)). A configuration the run refuses is not one it also cautions about, and the caution's two remedies are not the repair that block needs.

#### 4.10.4 What stays silent, and why the silence is the point

Four shapes are deliberately not this finding, and each is a correct configuration that [§FS-check.4.10.2](FS-check.md#4102-what-counts) answers "no" for. A grouping directory that only groups — `grund.toml` and its members and nothing else — has no root on disk to read. A block whose own roots are all inside its members is [§FS-workspace.2.1](FS-workspace.md#21-a-member-that-swallows-the-blocks-own-scan)'s shape read from the other side. A tree whose only files are of unscanned types, or excluded, gitignored, or hidden, is a tree the block would not have read as a project either. And a root the config names that is not on disk rescues nothing. Firing on any of them would be a warning with no edit that clears it, which is permanent output tools learn to filter — the outcome [§DF-absorbed-scan-warning](../decisions/functional/DF-absorbed-scan-warning.md#df-absorbed-scan-warning-a-scan-its-own-members-swallowed-is-a-warning-with-a-named-release-not-an-error) already rejected once for a neighbouring finding.

#### 4.10.5 The message

The message carries the config line the reader should open, the tree that is unread, what that costs, and the two remedies:

```
warning: group/grund.toml:17: no project scans `docs`, so its citations are never checked. Set include_root = true, or point another project's [scan] include at it.
```

The breadcrumb is the block's own `include_root` line — the key that decided is the line to open — falling back to its `[workspace]` line if the key is absent, which the default `true` makes unreachable. It renders against the root this run was launched at, like every diagnostic about a block above or below it ([§FS-errors.3](FS-errors.md#3-message-text)). The unread tree is named by its path **under the block root**, that spelling normalized rather than the spelling itself, exactly as [§FS-check.4.7.1](FS-check.md#471-the-message) names a covered root, and never by the resolved path, for [§FS-check.4.7.1](FS-check.md#471-the-message)'s reason.

#### 4.10.6 One root, not every root

[§FS-check.4.7.1](FS-check.md#471-the-message) lists every covered root because its claim is universal — *every* one is inside a member — and the list is the evidence for it. This claim is existential: one unread root is the whole finding, one edit clears all of them, and probing the rest would buy nothing the answer depends on. The one named is the first in scope order, `[scan] include` in config order and then the `[[kinds]]` homes, which is fixed; the first *file* under it is whatever the filesystem handed back, which is not, and is therefore never named ([§FS-errors.4](FS-errors.md#4-determinism)).

#### 4.10.7 Every command that walks says it, and each block says it once

The question is asked where a run populates a block's member boundary — the block the run is rooted at, and each block below it — so `check`, [§FS-list](FS-list.md#fs-list-grund-lists-every-declared-id), [§FS-refs](FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id), [§FS-cover](FS-cover.md#fs-cover-grund-groups-citations-by-scanned-file), [§FS-fmt](FS-fmt.md#fs-fmt-grund-normalizes-references-in-bulk) and every other command that resolves that boundary carry it, on exactly the surfaces [§FS-check.4.7](FS-check.md#47-a-workspace-member-swallows-the-blocks-own-scan) is carried on ([§FS-check.4.7.2](FS-check.md#472-every-command-that-walks-says-it-not-just-check)). The reproduction in the ticket used `check` and `list`, which is the whole argument: a silent-scope defect only `check` reports is half-reported. It is **once per block per run** — a workspace-wide run that expands the same block a second time still says it once — and a tree with two opted-out blocks earns one line each, the block the run is rooted at first and the blocks below it after, in the order the run reaches them. A run narrowed inside a member never populates the enclosing block's boundary, so it stays silent about a block it is not reading through.

#### 4.10.8 A failed workspace expansion withholds it

A run whose **workspace expansion fails** does not carry this finding for the blocks below its root, a silence declared and bounded ([§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded)) rather than a gap: the answer depends on where the run's other projects are, and a failed expansion is exactly the case where that list was never produced, so the alternative is not an earlier warning but a wrong one. The run is exiting `2` on a config error the reader has to repair before anything else it says is worth reading, and the finding returns on the next run. [§FS-check.4.7](FS-check.md#47-a-workspace-member-swallows-the-blocks-own-scan) survives such a run, because what it asks is answered by the block's own members alone.

#### 4.10.9 A launch-time message keeps its text under `--format json`

It is a launch-time message, so it keeps its text under `--format json` ([§FS-errors.5.2.2](FS-errors.md#522-launch-time-messages-stay-text)) and carries no JSON `code` and no selector of its own ([§FS-check.1.4](FS-check.md#14-selecting-diagnostics-with---only-and---ignore)). That is [§FS-check.4.7](FS-check.md#47-a-workspace-member-swallows-the-blocks-own-scan)'s rendered shape ([§FS-check.4.7.6](FS-check.md#476-a-launch-time-message-keeps-its-text-under---format-json)) rather than [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block)'s, and the difference between them is *when the fact exists*. [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block)'s is knowable only from the walk that meets a nested config ([§FS-check.3.29.10](FS-check.md#32910-knowable-only-from-the-walk)), so it is one of `check`'s report errors and renders as a JSON diagnostic with `path` and `line` set and `sites` null ([§FS-check.3.29.13](FS-check.md#32913-in-check-one-of-the-reports-errors)). This one is settled at boundary population, before a walk and before a report exists, and is carried by every command that resolves that boundary ([§FS-check.4.10.7](FS-check.md#4107-every-command-that-walks-says-it-and-each-block-says-it-once)), not by `check` alone — so the shape it renders in is fixed here rather than borrowed from `check`'s, and giving `check` a different one would make the text a consumer greps for depend on which command produced it.

#### 4.10.10 It stands in place of the `success` marker, and never moves the exit code

[§FS-check.2.1.3](FS-check.md#213-the-success-marker) is unchanged: a run with a warning prints the warning rather than `success`. This is the first `[workspace]` caution that has to say so out loud, because it is the first that can fire on an otherwise clean run — [§FS-check.4.7](FS-check.md#47-a-workspace-member-swallows-the-blocks-own-scan)'s block always earns the empty-scan caution beside it ([§FS-check.2.2](FS-check.md#22-empty-scan), [§FS-check.4.7.5](FS-check.md#475-beside-the-empty-scan-caution-not-in-place-of-it)), and [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block)'s is in the report already, as an error ([§FS-check.3.29.13](FS-check.md#32913-in-check-one-of-the-reports-errors)). The exit code stays where it was ([§FS-check.2](FS-check.md#2-outputs)): opting out is a legitimate choice, and [§FS-non-goals.9](FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization) fixes what a warning means.

#### 4.10.11 One of the run's warnings, rendered by every frontend

Like [§FS-check.4.7](FS-check.md#47-a-workspace-member-swallows-the-blocks-own-scan), and for the same reason ([§FS-check.4.7.7](FS-check.md#477-one-of-the-runs-warnings-rendered-by-every-frontend)): the fact is settled before any report exists, but what the engine hands back is a diagnostic in the run's warning channel — carried on whatever the walking command returns and rendered by whichever frontend asked ([§FS-distribution.3.1](FS-distribution.md#31-rust-grund-core-crate)) — rather than a line the engine wrote to a stream. It **anchors at the block's `include_root` line**, falling back to its `[workspace]` line exactly as the breadcrumb of [§FS-check.4.10.5](FS-check.md#4105-the-message) does, and carries that anchor as the finding's own location so a frontend never parses the message for one. Every byte is the byte it is today, on stderr and under `--format json` alike, and the `success` marker it stands in place of is unaffected. The editor is the surface that gains a reader: it publishes the warning on that `include_root` line ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)), which is where the one edit that clears it is written.

#### 4.10.12 A warning permanently, with no release it becomes an error in

This is one of the two `[workspace]` findings that do not ramp, beside [§FS-check.4.9](FS-check.md#49-a-workspace-member-declared-optional-is-absent) ([§FS-check.4.9.6](FS-check.md#496-it-names-no-release)), and the difference is not how wrong the repository is. A finding is eligible to become an error only when **every repository in that state is wrong** *and* **the configuration has a way to say "I meant it"** — the rule for all four, argued in [§DF-unread-opted-out-block.2.3](../decisions/functional/DF-unread-opted-out-block.md#23-what-makes-a-workspace-finding-ramp-and-what-makes-one-permanent). [§FS-check.4.7](FS-check.md#47-a-workspace-member-swallows-the-blocks-own-scan) and [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) pass both: a block that claims to be a project and reads nothing, and a block whose projects are spelled two ways, are wrong on every reading of them, and each has an edit that records the intent instead — `include_root = false` for one, listing the block for the other — so an error is a verdict their author can act on before it lands, and [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)'s deprecation path gets them there — as it has already got [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) ([§FS-check.3.29.14](FS-check.md#32914-an-error-because-the-deadline-the-warning-named-has-arrived)). [§FS-check.4.9](FS-check.md#49-a-workspace-member-declared-optional-is-absent) fails the first: an absent optional member is a state its repository declared in advance ([§FS-check.4.9.3](FS-check.md#493-it-names-no-remedy-because-nothing-here-is-broken)). This finding passes neither. A grouping directory holding a README nobody needs checked is doing exactly what its author meant, and neither remedy records that: making the block a project and pointing another project's `[scan] include` at it both change what the repository *is*. That the finding is a property of the configuration **and** the tree — the same `grund.toml` silent on Monday and reportable on Tuesday because somebody added `group/docs/notes.md` — is the symptom that makes it recognizable, not the reason: [§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index) depends on the tree in the same way and ramped anyway. There is therefore no version constant, no roadmap milestone, and no clause in the message naming a release.

### 4.11 Config read from the deprecated `.agents/` location

The config this run read is an `.agents/grund.toml` ([§FS-config.1.2](FS-config.md#12-the-agents-location-is-deprecated)). The file is still read and still governs the project — the location is deprecated, not withdrawn — and the run says so, naming the file it read and the bare `grund.toml` beside it that should hold it instead:

```
warning: .agents/grund.toml is a deprecated config location — move it to grund.toml
```

It is [§FS-check.4.3](FS-check.md#43-redundant-config-pair)'s finding in everything but its trigger ([§FS-check.4.11.1](FS-check.md#4111-fs-check43s-finding-in-everything-but-its-trigger)), on the same three surfaces and no more ([§FS-check.4.11.2](FS-check.md#4112-the-same-three-surfaces-as-fs-check43-and-no-more)), once per config ([§FS-check.4.11.3](FS-check.md#4113-once-per-config-not-once-per-scope)), and never beside [§FS-check.4.3](FS-check.md#43-redundant-config-pair) about one directory ([§FS-check.4.11.4](FS-check.md#4114-a-directory-carrying-both-names-earns-fs-check43-and-not-this)). Its JSON form is [§FS-check.4.11.5](FS-check.md#4115-under---formatjson-one-warning-diagnostic-on-stderr). It stands in place of the `success` marker ([§FS-check.4.11.6](FS-check.md#4116-it-stands-in-place-of-the-success-marker)) and never becomes an error ([§FS-check.4.11.7](FS-check.md#4117-a-warning-permanently-with-no-release-it-becomes-an-error-in)).

#### 4.11.1 [§FS-check.4.3](FS-check.md#43-redundant-config-pair)'s finding in everything but its trigger

This is [§FS-check.4.3](FS-check.md#43-redundant-config-pair)'s finding in everything but its trigger, and for [§FS-check.4.3.1](FS-check.md#431-a-cli-level-warning)'s reason: it is a fact about *which file the run read*, not about that file's content and not about a site in the citation graph. So it is a CLI-level `warning:` on **stderr** ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)) and never a per-finding line — there is no offending line to point at, the whole file is the subject. Both paths are report paths and follow `[output] relative_paths` as [§FS-check.4.3.1](FS-check.md#431-a-cli-level-warning)'s do ([§FS-config.3.6](FS-config.md#36-output--report-format)), so the message names the move as the `git mv` a reader can type from the report's base. Like every warning it leaves the exit code alone ([§FS-check.2](FS-check.md#2-outputs)), because the location it names is a supported one and [§FS-non-goals.9](FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization) fixes what a warning means.

#### 4.11.2 The same three surfaces as [§FS-check.4.3](FS-check.md#43-redundant-config-pair), and no more

`grund check`, `grund config validate` and `grund config show` ([§FS-config.4.1](FS-config.md#41-grund-config-validate-path), [§FS-config.4.2](FS-config.md#42-grund-config-show-path)) carry it, byte-identically. `list`, `refs`, `cover`, `fmt` and a bare ID read stay silent, on [§FS-check.4.3.2](FS-check.md#432-reported-by-check-config-validate-and-config-show)'s argument unchanged. The silence is not an oversight to be widened later — a fact that never changes between runs, printed by every command, is permanent output tools learn to filter, and the surface a user reaches for to ask *where is my config* is `config show`.

#### 4.11.3 Once per config, not once per scope

A workspace names the root's config and every member's, each at the path that project's config was loaded under ([§FS-errors.4](FS-errors.md#4-determinism)) — `packages/beta/.agents/grund.toml`, moving to `packages/beta/grund.toml`. Root and member are separate configs and a workspace may mix the two forms ([§FS-workspace.2](FS-workspace.md#2-workspace-configuration)), so a member on the old path earns its own line while a member on the bare form earns none, and a root and a member both on the old path earn one line each.

#### 4.11.4 A directory carrying both names earns [§FS-check.4.3](FS-check.md#43-redundant-config-pair) and not this

The bare `grund.toml` won the tie ([§FS-config.1.1](FS-config.md#11-when-one-directory-carries-both)), so the config in force is on the home path already and nothing about it is deprecated; the `.agents/` file beside it is the one read by nothing, which is exactly what [§FS-check.4.3](FS-check.md#43-redundant-config-pair) reports. Emitting both would name one move twice and disagree about which of the two files is the problem.

#### 4.11.5 Under `--format=json`, one warning diagnostic on stderr

Under `--format=json` it is one warning diagnostic on stderr, carrying the `code` `deprecated-config-location`, with `path`, `line` and `sites` null ([§FS-errors.5](FS-errors.md#5-json-format)) — [§FS-check.4.3](FS-check.md#43-redundant-config-pair)'s shape, because it arrives the same way: one of the report's warnings, knowable from the config the run loaded rather than from the walk.

#### 4.11.6 It stands in place of the `success` marker

It stands in place of the `success` marker ([§FS-check.2.1.3](FS-check.md#213-the-success-marker)), as every warning does, and that is the whole cost of the finding rather than a detail of it: a clean repository on the old path now prints this line where it printed `success`. [§REQ-backwards-compatibility.1](../requirements/REQ-backwards-compatibility.md#1-what-is-covered) governs that as a verdict change and permits it, since the exit code does not move. This repository pays it in its own e2e corpus, which is why the fixtures that had no reason to be on `.agents/` are on the bare form and the ones that remain are the ones whose subject is discovery itself.

#### 4.11.7 A warning permanently, with no release it becomes an error in

The promise lives in [§FS-config.1.2](FS-config.md#12-the-agents-location-is-deprecated), because it is a fact about the location rather than about this message: the fallback is never removed, so there is no version constant, no roadmap milestone, and no clause in the text naming a release.

### 4.12 Missing snapshot

A recognized citation has no declaration and targets a fetch-enabled kind whose effective `resolve` is `should` ([§FS-config.3.4.10](FS-config.md#3410-format-resolve-and-fetch--external-snapshot-kinds)). It is a distinct fixed warning, not a softened dangling error:

```text
<path>:<line>: no snapshot for <qualified-ID> in <home> — run grund fetch <qualified-ID>
```

The em-dash remedy tail is exactly `— run grund fetch <qualified-ID>`, with no inner backticks. The JSON code is `missing-snapshot`, severity is `warning`, and the message field is the text after `<path>:<line>: `. A warning-only run exits 0 and prints no `success` marker. Every site gets one finding, remains in the reference and coverage indexes, and may resolve after an explicit [§FS-fetch](FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot). The hints that take the fetch tail's place are [§FS-check.4.12.1](FS-check.md#4121-a-hint-takes-the-fetch-tails-place), and its workspace spelling is [§FS-check.4.12.2](FS-check.md#4122-in-a-workspace).

#### 4.12.1 A hint takes the fetch tail's place

The near-ID and escaped-inline-code hints of [§FS-check.3.1.1](FS-check.md#311-a-near-id) and [§FS-check.3.1.2](FS-check.md#312-an-illustration-in-inline-code) take precedence over the fetch action. The message retains `no snapshot for <qualified-ID> in <home>` and substitutes the existing conditional `; did you mean …`, `; write <§>…`, or combined tail for the em-dash fetch tail. That precedence never produces both a dangling and a missing-snapshot finding for one site.

#### 4.12.2 In a workspace

In a workspace, the ID and remedy use the complete alias-qualified spelling while `<home>` is rendered from the run's report base ([§FS-workspace.8.1](FS-workspace.md#81-grund-aliasid)).

### 4.13 Oversized lead *(opt-in)*

When the effective project config contains the opt-in key from [§FS-config.3.1](FS-config.md#31-reference--citation-form):

```toml
[reference]
lead_size_warning = { max = <N>, unit = "<unit>" }
```

`check` measures each declaration and citable section lead in that project by [§FS-list.3.4](FS-list.md#34---size--per-point-lead-and-full-body-measurements). A lead whose selected measurement is strictly greater than `max` produces one warning at that site's heading line. Equality passes. A broken stub has no measurable lead and produces no size warning; duplicate declaration homes and duplicate section claimants are judged separately from their own site-local slices.

The message is [§FS-check.4.13.1](FS-check.md#4131-the-message) and its exit and rendering [§FS-check.4.13.2](FS-check.md#4132-exit-code-and-rendering). Only the key activates it ([§FS-check.4.13.3](FS-check.md#4133-only-the-key-activates-it)), and which sites it judges is [§FS-check.4.13.4](FS-check.md#4134-which-sites-it-judges).

#### 4.13.1 The message

The fixed code is `oversized-lead`, severity is `warning`, and the exact text after `<path>:<line>: ` is:

```text
<coordinate> lead is <actual> <unit>, over the configured maximum of <max>; move detail into citable child sections, or promote a child section to its own ID after running grund refs <coordinate> --summary
```

The coordinate is local for a member-local check and workspace-qualified for a workspace-root check. The two remedies preserve grounding and citation stability; the message never suggests shortening or deleting it.

#### 4.13.2 Exit code and rendering

A warning-only run exits `0` and replaces the text `success` marker; JSON uses the ordinary located diagnostic object ([§FS-errors.5](FS-errors.md#5-json-format)). The LSP publishes the same message, line, code, and warning severity as the CLI ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)). Any simultaneous error, including `duplicate` or `duplicate-section`, still decides exit `1`; the warning neither suppresses it nor changes its priority.

#### 4.13.3 Only the key activates it

The absent key activates no measurement or finding and leaves the existing text and JSON check output byte-identical. `--only oversized-lead` and `--ignore oversized-lead` select the finding after the complete check and never activate it.

#### 4.13.4 Which sites it judges

An explicit-path check judges only declaration and section sites scanned at that path. `--full` adds its existing out-of-scope reference findings but does not extend this repository policy beyond declarations in the configured scan scope. In a workspace, each member's effective key governs only that member's sites; a member without the key remains silent even when another member opts in.

### 4.14 Unmarked Markdown heading

Before grund 0.15.0, a Markdown ATX heading that is deeper than a declaration heading and whose line is still inside that declaration's body is a fixed warning when it is neither another declaration nor a recognized numeric or enabled named section. The containing declaration is the nearest enclosing body, so a plain heading beneath a deeper child declaration names that child, not an overlapping ancestor. This is a project-wide rule with no configuration, severity selector, or permanent opt-out ([§DF-unmarked-markdown-headings](../decisions/functional/DF-unmarked-markdown-headings.md#df-unmarked-markdown-headings-in-body-markdown-atx-headings-participate-in-the-knowledge-graph)).

Which headings participate is [§FS-check.4.14.1](FS-check.md#4141-which-headings-participate). The message is [§FS-check.4.14.2](FS-check.md#4142-the-message) and its suggested coordinate [§FS-check.4.14.3](FS-check.md#4143-the-suggested-coordinate); its rendering is [§FS-check.4.14.4](FS-check.md#4144-rendering-exit-and-selection), its flip to an error [§FS-check.4.14.5](FS-check.md#4145-an-error-in-grund-0150), and what it leaves unchanged [§FS-check.4.14.6](FS-check.md#4146-no-command-numbers-the-heading).

#### 4.14.1 Which headings participate

Only ATX headings in scanned Markdown files participate. A heading inside a backtick or tilde fence is content. A file title before the first declaration, a same-or-shallower heading that closes a declaration body, source doc-comment text, setext text, and a bold label are outside the rule. A deeper declaration and a valid numeric or enabled named section already participate in the graph and are not unmarked. `--full` keeps this convention-scoped warning narrowed to the configured scan scope, as it does other convention findings ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)).

#### 4.14.2 The message

The warning is anchored at the heading line, uses code `unmarked-heading`, and has this text:

```text
unmarked heading inside <ID>; number it (<suggested heading>) as <ID>.<path>, declare an ID, or use a bold label; this warning becomes an error in grund 0.15.0
```

#### 4.14.3 The suggested coordinate

The suggested coordinate is guidance, not a rewrite. Its path depth follows the written ATX depth relative to the containing declaration. At each depth, grund uses the nearest preceding citable or already-suggested parent and appends one above the largest existing or earlier-suggested numeric sibling; it never fills a hole or reuses a coordinate. With no parent it starts one above the largest root numeric coordinate, or at `1` when none exists. If the authored heading skips a depth, missing parents are filled with `.1`. A named parent may therefore receive a numeric child such as `goals.1`. The suggested heading preserves the authored `#` depth and title and inserts the complete coordinate in that valid numeric or mixed form. A titleless ATX heading has no authored title to preserve, so its otherwise-identical suggestion uses the literal title `Untitled`; applying that complete suggested heading produces a recognized section and clears the warning.

#### 4.14.4 Rendering, exit, and selection

Text output uses the `<path>:<line>: warning: <message>` form. A warning leaves the exit at `0` but stands in place of the `success` marker ([§FS-check.2.1.3](FS-check.md#213-the-success-marker)). JSON emits the same path, line, code, and message with `"severity":"warning"` and `"sites":null`. `--only unmarked-heading` retains it and `--ignore unmarked-heading` removes it through the ordinary exact-code selection rules. The LSP carries the same core finding with warning severity and the complete ATX heading as its range ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)).

#### 4.14.5 An error in grund 0.15.0

In grund 0.15.0 the same site, code, suggestion, and all non-severity bytes stay stable except that the deadline clause becomes `this became an error in grund 0.15.0`; severity becomes error and a retained finding contributes exit `1`. [§RM-unmarked-heading-error](../roadmap.md#rm-unmarked-heading-error-make-unmarked-markdown-headings-errors-in-0150) owns that scheduled flip. Until then the warning window serves [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path).

#### 4.14.6 No command numbers the heading

No command numbers the heading: `grund fmt` remains unchanged. `show`, `list`, `refs`, `cover`, formatting, section resolution, and source scanning otherwise keep their current behavior, including the body boundary and rejected section behavior of [§FS-check.3.23](FS-check.md#323-section-outside-a-declaration). `grund_config_version` stays `1`; the managed agent block is the existing mechanical repair surface and moves to v10 under `grund init` ([§FS-init.2.3.4.5](FS-init.md#2345-declaration-forms)).

## 5. What grund does not check

See [§FS-non-goals](FS-non-goals.md#fs-non-goals-what-grund-will-deliberately-not-do) — in particular [§FS-non-goals.1](FS-non-goals.md#1-markdown-link-validation) (markdown links / URLs), [§FS-non-goals.2](FS-non-goals.md#2-spelling-grammar-prose-quality) (spelling/grammar outside the explicit value form), and the convention that ID numbers are stable handles, not ordinal positions. Value checking adds only the exact binding in [§FS-values.3.1](FS-values.md#31-the-only-binding-grammar): no surrounding-number inference, bare-literal lint, range/unit semantics, rendering, fingerprint/history check, or new reconciliation verb is performed ([§FS-values.9](FS-values.md#9-compatibility-and-explicit-exclusions)). The near misses that used to be listed here are [§FS-check.5.1](FS-check.md#51-the-near-misses-are-no-longer-here).

### 5.1 The near misses are no longer here

The declaration-side near miss is **no longer** in this section: a heading shaped like `# <KIND>-…: <title>` whose ID does not match the configured `[id] format` is reported per heading by [§FS-check.4.6](FS-check.md#46-declaration-near-miss), and a tree in which every heading misses that way says so twice over — once per line, and once as the run that recognized nothing ([§FS-check.4.5](FS-check.md#45-nothing-recognized)). Neither guesses the corrected ID. The citation-side near miss — a `§`-marked token in the shorthand shape — left this section earlier, when [§FS-check.1.2](FS-check.md#12-the-number-only-shorthand) and [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation) began recognizing and reporting it.

## 6. Watch mode (`--watch`)

Status: planned — implementation tracked under [§RM-watch](../roadmap.md#rm-watch-implement-grund-check---watch).

When implemented, `grund check --watch [<path>]` will run the check once, then stay resident and re-run it whenever a file under the scanned tree (or the discovered `grund.toml`) changes. It is the editor-less counterpart to the optional LSP server ([§FS-lsp](FS-lsp.md#fs-lsp-grund-ships-an-optional-lsp-server)): the LSP integrates `grund` into an editor's diagnostics; `--watch` is the plain-terminal "every save" loop that [§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible) exists for. Until [§RM-watch](../roadmap.md#rm-watch-implement-grund-check---watch) lands, `grund check --watch` is a CLI error (`error: unknown flag \`--watch\``, exit 2).

How it notices a change is [§FS-check.6.1](FS-check.md#61-change-detection), what each run prints is [§FS-check.6.2](FS-check.md#62-each-run-is-a-plain-grund-check), how it ends is [§FS-check.6.3](FS-check.md#63-lifecycle), and which command takes the flag is [§FS-check.6.4](FS-check.md#64-scope).

### 6.1 Change detection

Filesystem notifications where the OS provides them; a debounce window coalesces a burst of writes into one re-check. No polling loop is required, and there is no configurable interval — the watcher reacts, it does not sample.

### 6.2 Each run is a plain `grund check`

Output and exit-status semantics of an individual run are exactly [§FS-check.2](FS-check.md#2-outputs)/[§FS-check.2.1](FS-check.md#21-report-format) on the tree's state at that moment — byte-identical to what a non-`--watch` invocation would print ([§FS-errors.4](FS-errors.md#4-determinism)). Before each run, and only when stdout is a terminal, the previous run's output is cleared so the terminal always shows the current report; piped output and `--format=json` carry no clearing bytes and stay byte-identical to a plain run, and with `--format=json` each run emits the same diagnostic NDJSON as non-watch mode, scoped to that run.

### 6.3 Lifecycle

The process runs until interrupted (Ctrl-C / SIGINT). On interrupt it exits with the exit code of the most recently completed run (`0`/`1`/`2`), so `grund check --watch &` followed by a later signal is still a meaningful CI-ish probe. There is no TUI, no key bindings, no prompt — it is non-interactive per [§FS-non-goals.10](FS-non-goals.md#10-interactive-mode), just a re-printing checker. No network I/O ([§FS-non-goals.11](FS-non-goals.md#11-network-access-during-a-check)); the only files touched are the ones the walk already reads.

### 6.4 Scope

`--watch` will be a `check` flag spelled as `grund check --watch [<path>]` ([§FS-cli](FS-cli.md#fs-cli-grunds-command-line-surface-conventions)). Other subcommands will not take it; a one-shot `grund fmt` or ID query has nothing to keep watching.
