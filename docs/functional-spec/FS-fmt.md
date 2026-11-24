# FS-fmt: grund normalizes citations in bulk

`grund fmt` rewrites citation markup across a tree: a typed trigger becomes the marker, a bare citation becomes a marked one under `--marker`, a number-only shorthand becomes its full ID where the `[reference] shorthand` policy asks, a declaration-local section citation whatever the policy says, and a citation in Markdown becomes a link to its declaration, whose destination every cross-reference pass re-derives. A root-scoped `--write` also maintains eligible section-lock entries ([§FS-section-lock.write.rules](FS-section-lock.md#writerules-add-refresh-drop)), without approving a protected section move ([§FS-section-lock.write.never-a-move](FS-section-lock.md#writenever-a-move-fmt-never-approves-a-move)). It is the batch counterpart of the LSP server's live trigger transform ([§FS-lsp.1.4](FS-lsp.md#14-live-trigger-transform)), and the one every install has, since the LSP server is opt-in. Implements [§DF-reference-marker](../decisions/functional/DF-reference-marker.md#df-reference-marker-use--as-the-reference-marker-with--as-the-typing-trigger).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, body, section, index, catalog),
[§FS-terms.terms.2](FS-terms.md#terms2-citations) (marker, citation, qualified citation, shorthand, canonical form),
[§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (source declaration, stub, doc-comment), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope,
config root, workspace, member, alias), [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, anchor), and
[§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations) (value, binding, fetcher, snapshot).

- **never-rewrite zone** — [§FS-citations.terms](FS-citations.md#terms-terms)'s.
- **suppressed scope** — A file `[fmt] exclude` names, or a region between a `grund:fmt off`
  directive and the `grund:fmt on` that closes it. The repository asked for it, so every finding
  there stays.
- **numeric run** — Digits adjoining a number-only shorthand that leave the token's end
  ambiguous, which is why the shorthand there is not rewritten.
- **dry run** — The run that reports what `--write` would change and writes nothing — `--check`,
  or no flag at all. It predicts `--write` exactly.

## requirements: What every fmt run holds to

The requirements `fmt` shares with other commands are stated where they are shared: [§REQ-no-data-loss.2](../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own), [§REQ-no-wrong-citation.3](../requirements/REQ-no-wrong-citation.md#3-no-wrong-write), [§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes) and [§REQ-one-answer.writes.anchors](../requirements/REQ-one-answer.md#writesanchors-shows-anchor-is-the-anchor-fmt-writes).

### requirements.fixed-point: A tree fmt has written is a fixed point

On a tree it can read whole, a second run with the same flags and configuration finds nothing to change: `--check` exits `0` and `--write` reports `rewrote 0 lines`, which [§FS-fmt.7.3](FS-fmt.md#73-preview-equivalence) makes one statement. Every pass arrives there, the section-lock pass included, which records the citations and section texts the run leaves rather than the ones it read ([§FS-section-lock.write.rules](FS-section-lock.md#writerules-add-refresh-drop)). Serves the CI gate and the pre-commit hook of [§FS-fmt.4](FS-fmt.md#4-why-this-exists), which agree only if what `--write` leaves `--check` passes, and [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

### requirements.next-class: A new rewrite class joins every list that names the classes

The rewrite classes are enumerated: the passes a fence stops ([§FS-fmt.2.3.3](FS-fmt.md#233-a-fenced-code-block-is-an-illustration)), the label order of the dry-run report ([§FS-fmt.3.5](FS-fmt.md#35-the-dry-run-report)) and the totals of the `--write` report ([§FS-fmt.3.7](FS-fmt.md#37-the---write-report)) each name them. A class joins each list in the change that adds it: it stops at every never-rewrite zone ([§FS-fmt.2.3](FS-fmt.md#23-what-is-never-rewritten)), takes its own label, a place in that order and a unit in the totals, and leaves the fixed point of [§FS-fmt.requirements.fixed-point](FS-fmt.md#requirementsfixed-point-a-tree-fmt-has-written-is-a-fixed-point). One that writes characters into an ID token or a link destination also names the text it writes ([§FS-fmt.3.6](FS-fmt.md#36-an-expansion-or-a-link-names-the-text-it-writes)) and runs only against a complete declaration set ([§FS-fmt.7.4](FS-fmt.md#74-no-write-without-a-complete-model)). The requirement is directional ([§FS-config.requirements](FS-config.md#requirements-what-the-config-contract-holds-to)): it binds each class as it is added and claims nothing of the five that exist. Serves [§REQ-no-data-loss.2](../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own) and [§REQ-no-wrong-citation.3](../requirements/REQ-no-wrong-citation.md#3-no-wrong-write).

## 1. Inputs

```
grund fmt [<path>] [--check] [--marker] [--cross-refs] [--write]
```

- `<path>` — the directory or file the run is narrowed to; with none, the project the current directory discovers ([§FS-scan.path.none](FS-scan.md#pathnone-a-run-with-no-path-covers-the-project-discovery-finds)).
- `--check` — report exactly what `--write` would change, exit non-zero if any change would be made, and write nothing.
- `--marker` — also rewrite bare citations (`FS-check`) to marked ones (`§FS-check`) ([§FS-fmt.2.2](FS-fmt.md#22-bare-to-marker-with---marker)). Off by default.
- `--cross-refs` — force the cross-reference pass ([§FS-fmt.6](FS-fmt.md#6-cross-reference-emission)) for this run. Without the flag the pass still runs, in both modes, wherever `[fmt.cross_refs] enabled` is set, as it is by default, and the scope holds Markdown ([§FS-fmt.6.7.2](FS-fmt.md#672-what-enabled-decides)).
- `--write` — write the changes, exiting `0` even when it made some ([§FS-fmt.3](FS-fmt.md#3-outputs)).

`--check` and `--write` are mutually exclusive; with neither, the run is `--check`.

## 2. Behavior

### 2.1 Trigger-to-marker

Wherever the configured trigger (default `$$`) is immediately followed by a token that matches its kind's effective ID format ([§FS-config.3.2](FS-config.md#32-id--id-grammar)) — `FS-007` under a numbered format, `FS-login` under the slug-only form `grund` itself uses — the trigger is replaced with the configured marker (default `§`). The trigger is consumed only where a real ID-shaped token follows it, so a bare `$$`, LaTeX display math or a shell variable, is left alone.

### 2.2 Bare-to-marker (with `--marker`)

With `--marker`, every recognized bare citation is also marked. This is how a repository leaves `[reference] strict = false`: run `grund fmt --marker --write` once, then remove the override or set the flag back to `true`.

### 2.3 What is never rewritten

This list is the ownership boundary of the one writer that rewrites text anywhere in a file it scans, rather than only inside a block it delimits ([§REQ-no-data-loss.2](../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own)): everything it names keeps its bytes.

- Declaration headings (the line that names the ID).
- The exact `<!-- grund:value -->` suffix on a numbered section heading, which is authored authority rather than formatter syntax: no pass inserts, canonicalizes, moves, removes or rewrites it or its trailing whitespace ([§FS-values.2.4](FS-values.md#24-embedded-section-value-roots)).
- Explicit section names. A pass may wrap citations to them and re-derive their anchors, but never infers a name from a title, renames one, or rewrites a named heading path.
- Relative section headings, never expanded, shortened or rewritten ([§FS-declarations.section.relative.shown](FS-declarations.md#sectionrelativeshown-how-a-relative-heading-is-shown)); `--write` writes only their section lock ([§FS-section-lock.write](FS-section-lock.md#write-who-writes-the-lock)).
- Citations in every zone [§FS-citations.zones](FS-citations.md#zones-where-citation-text-is-never-rewritten) names.
- Files outside the default scope.
- External file-symlink targets ([§FS-fmt.2.3.2](FS-fmt.md#232-a-link-that-leaves-the-config-root-is-not-written-through)).
- Everything in a **suppressed scope** ([§FS-fmt.2.5](FS-fmt.md#25-suppressed-scopes)). It alone is asked for by the repository rather than forced by the text, and it alone yields to a kind's index ([§FS-fmt.6.1](FS-fmt.md#61-scope)).

#### 2.3.1 String-literal exclusion rule

For every candidate rewrite site on a source-file line:

1. Scan the line left to right from column 0, in the state [§FS-languages.multi-line-string](FS-languages.md#multi-line-string-a-string-literal-that-spans-lines) gives it, to the candidate's start.
2. Track an open-quote state per `'`, `"`, and `` ` ``. An unescaped quote (no immediately preceding `\`) of a given kind toggles its state, but only while no other kind is open.
3. If any quote state is open at the candidate's start column, the candidate is inside a string literal and is **not** rewritten.

A file read as Markdown has no string literals and is not subject to this rule; every other scanned file is. Two readings, defined here and cited, never a suffix, decide which files a rule covers. A file is **read as Markdown** when its name resolves to an admitted `document` language ([§FS-config-v2.schema.sources.definitions](FS-config-v2.md#schemasourcesdefinitions-schemasourcesdefinitionsname-defines-a-language)), in version 1 when it is named `.md`; its **docstrings are read** when that language reads them, in version 1 when it is named `.py` under `[scan] docstring_python`. In version 2 a name no admitted language claims is neither, and a name the config writes is judged the same way as a file in the tree.

In a file whose docstrings are read, a `"""` / `'''` delimiter is doc-comment syntax rather than a quote ([§FS-check.1.1](FS-check.md#11-recognized-citations)), so the line this scan reads is the docstring's **content**, the delimiters stripped and the candidate's start column measured from there: a citation on the opening, a one-line or the closing line is judged like one on an interior line, where a quote inside the content still opens and closes and the delimiter never does, and code after the closing delimiter is judged on the raw line. A qualifying top-level assigned triple-quoted string is data, not a docstring ([§FS-citations.recognized.string-literals.assigned-data](FS-citations.md#recognizedstring-literalsassigned-data-assigned-python-triple-quoted-data)): every pass, and the LSP on-type transform, preserves its whole raw-column span byte for byte, and text after a same-line close is ordinary source again. Every other string literal on a code line, and every line of a file whose docstrings are not read, keeps the raw-line rule.

#### 2.3.2 A link that leaves the config root is not written through

The scan reads an external **file-symlink** target and checks its citations ([§FS-scan.scope.links](FS-scan.md#scopelinks-a-symlink-in-the-tree-is-followed)), but neither mode rewrites it, because the project does not own that file ([§REQ-no-data-loss.2](../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own)). Each refused link is named once on stderr, in the CLI-level shape of [§FS-errors.2.2](FS-errors.md#22-cli-level-message), with the exit code unchanged, and the dry run lists no rewrite for it ([§FS-fmt.7.3](FS-fmt.md#73-preview-equivalence)). Decided in [§DF-symlink-scan.2.5](../decisions/functional/DF-symlink-scan.md#25-fmt---write-refuses-an-external-file-link-external-directory-links-never-reach-it).

```text
warning: docs/functional-spec/FS-002-beta.md: not rewritten: the symlink target is outside the config root
```

Files below an external directory symlink draw neither a rewrite nor a warning: the shared scan prunes that directory. A link whose target is inside the config root is written through; a file reached under two spellings is read once, under the surviving one ([§FS-scan.identity.read-once](FS-scan.md#identityread-once-one-physical-file-is-read-once)), so a link `--cross-refs` writes into it is right only when the file is opened under that spelling.

A file refused because two names read it differently ([§FS-scan.checks.reading-conflict](FS-scan.md#checksreading-conflict-two-names-of-one-file-read-it-differently)) is left alone by both modes, narrowed or not, and named once on stderr, exit code unchanged; where it is also behind a link that leaves the config root, this line stands in place of that link's:

```text
warning: docs/license.md: not rewritten: docs/license.md and docs/license.rs read this file differently
```

#### 2.3.3 A fenced code block is an illustration

Every pass stops at a Markdown fence — trigger-to-marker, bare-to-marker, shorthand-to-canonical and the cross-reference wrap alike — because a fence is where a grounded repository writes an ID as an *illustration*: `check` reads nothing there ([§FS-check.1.1](FS-check.md#11-recognized-citations)), so an edit inside one would turn text that makes no claim into text that makes one. A fence whose citations `check` reads is no exception ([§DF-fenced-citations.2.3](../decisions/functional/DF-fenced-citations.md#23-citations-and-nothing-else)). What opens and closes a fence is the rule of [§FS-check.1.1](FS-check.md#11-recognized-citations).

#### 2.3.4 A link destination is the URL

That a link destination is never rewritten, except the destination of a wrap `fmt` itself maintains, is [§FS-citations.zones.link-destination](FS-citations.md#zoneslink-destination-a-link-destination-is-the-url).

#### 2.3.5 An inline code span closes on a run of its own length

Where an inline code span begins and ends, for every pass of this command and for `check`, is [§FS-citations.zones.inline-code](FS-citations.md#zonesinline-code-an-inline-code-span-closes-on-a-run-of-its-own-length).

### 2.4 Shorthand-to-canonical

Where a marker-origin number-only shorthand citation ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)) resolves to exactly one declaration, the target project's `[reference] shorthand` policy decides. Under `canonical` it is rewritten to the canonical full ID, keeping any `.<section>` suffix and any `<alias>/` prefix: `<§>FS-042` becomes `<§>FS-042-user-login`. This is the bulk fix for the [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation) error ([§DF-number-only-citation-shorthand.2.2](../decisions/functional/DF-number-only-citation-shorthand.md#22-where-the-shorthand-is-accepted-and-where-it-is-an-error)), except in a suppressed scope ([§FS-fmt.2.5](FS-fmt.md#25-suppressed-scopes)) or an external file-symlink target ([§FS-fmt.2.3.2](FS-fmt.md#232-a-link-that-leaves-the-config-root-is-not-written-through)), which `fmt` does not write. Under `accepted` neither mode offers the change, and the token keeps its bytes.

An owned declaration-local numeric citation ([§FS-check.1.1.8](FS-check.md#118-declaration-local-numeric-section-candidates)) is expanded, whatever the `shorthand` policy, to the configured marker, its one enclosing declaration's full ID, the configured ID/section separator and its numeric path, because declaration-local spelling is never an accepted stored form ([§DF-declaration-local-section-shorthand](../decisions/functional/DF-declaration-local-section-shorthand.md#df-declaration-local-section-shorthand-local-numeric-section-citations-are-recognized-but-never-canonical)). The expansion needs one owner, a site every writer rule permits, and a cited section that resolves against that owner: an ownerless, ambiguous or unsupported token is left byte-identical, and so is one whose owner records no heading at the cited path ([§FS-fmt.2.4.6](FS-fmt.md#246-an-absent-target-section-withholds-this-rewrite-and-only-this-one)).

A shorthand that matches no declaration, or more than one, is **left alone** for `check` to report: `fmt` does not guess. The never-rewrite zones apply to both expansions ([§FS-fmt.2.3](FS-fmt.md#23-what-is-never-rewritten)); where they withhold `check`'s number-only error is [§FS-citations.rewrite-exemption](FS-citations.md#rewrite-exemption-where-the-text-forbids-the-rewrite), while a declaration-local token in a protected site still draws [§FS-check.3.24](FS-check.md#324-declaration-local-section-citation), which names the replacement to write by hand where the owner has the cited section, and the escape where it has not.

The rewrite fires only on a whole token, one the next character cannot continue, so `<§>FS-042-User-Login`, `<§>FS-042_user_login` and `<§>FS-042abc` keep their bytes ([§FS-citations.shorthand.full-id-wins](FS-citations.md#shorthandfull-id-wins-the-full-id-always-wins)), and never in a numeric run ([§FS-fmt.2.4.1](FS-fmt.md#241-a-shorthand-in-a-numeric-run-is-not-rewritten)). A typed trigger expands under both policies, so `$$FS-042` becomes `<§>FS-042-user-login` in one step; origin is decided per candidate, so on a line that also holds a persisted `<§>FS-042`, `accepted` expands only the trigger. A qualified shorthand is matched and resolved with the **aliased project's** grammar and policy ([§FS-workspace.4](FS-workspace.md#4-resolution), [§FS-workspace.8.5](FS-workspace.md#85-grund-fmt---cross-refs)): a canonical `api` rewrites `<§>api/FS-042` to `<§>api/FS-042-session`, and an `api` on `accepted` keeps it; a member-local run has no workspace context and leaves qualified citations untouched. The declaration set is scanned the first time the run meets a shorthand, and that file is redone; the scan covers the whole project even under a narrowed path, and a project with no shorthand never pays for it.

The dry-run report labels a number-only expansion `shorthand → canonical` and a declaration-local one `local section → canonical`, each with the text it writes ([§FS-fmt.3.6](FS-fmt.md#36-an-expansion-or-a-link-names-the-text-it-writes)).

#### 2.4.1 A shorthand in a numeric run is not rewritten

A marked shorthand glued to a second number, as a changelog line recording an ID remapping writes it (`§SPEC-001→SPEC-003`, `§SPEC-001/003`, `§COMP-047/046/049`), is used as a numeral, not as a citation, and is left byte for byte. Decided in [§DF-shorthand-numeric-run](../decisions/functional/DF-shorthand-numeric-run.md#df-shorthand-numeric-run-a-marked-shorthand-glued-to-another-number-is-a-numeral-not-a-citation).

Reading forward from the end of the shorthand token, the site is a **numeric run** when both hold:

1. The maximal run of characters that can neither continue nor start an ID — anything not alphanumeric and not `_` — is non-empty, and carries no whitespace, no marker, and no bracket or quote (`(`, `)`, `[`, `]`, `{`, `}`, `"`, `'`, `` ` ``).
2. What follows that run begins with a `[id] number_pattern` match, or with the whole shorthand shape of the grammar that parsed the token ([§FS-config.3.2](FS-config.md#32-id--id-grammar)) written **unqualified**: an `<alias>/` prefix only ever precedes a citation ([§FS-workspace.1](FS-workspace.md#1-citation-syntax)), so a path that ends in an ID-shaped segment is no run.

The rule reads forward only: `SPEC-001→§SPEC-003`, whose tail carries the marker, is rewritten, because a glued number to the *left* is also what `2026-08-19/<§>FS-042` looks like. The site is reported by [§FS-check.3.15](FS-check.md#315-shorthand-citation-in-a-numeric-run) and stays a citation in every other respect ([§FS-citations.shorthand.numeric-run](FS-citations.md#shorthandnumeric-run-a-shorthand-in-a-numeric-run)).

#### 2.4.6 An absent target section withholds this rewrite, and only this one

A declaration-local candidate whose owner records no heading at the cited numeric path is left
byte-identical, in dry-run and in write mode alike. The section map the refusal reads is the one
[§FS-check.3.2](FS-check.md#32-missing-section) reads to report the absence
([§REQ-one-answer.sections.map](../requirements/REQ-one-answer.md#sectionsmap-every-surface-reads-one-section-map)).

The asymmetry with number-only ID shorthand is deliberate. `<§>FS-042` already names its declaration,
so expanding it to `<§>FS-042-user-login.99` changes only how that declaration is spelled and
[§FS-check.3.2](FS-check.md#32-missing-section) means the same thing on both sides of the rewrite;
nothing an author wrote is lost. The declaration-local rewrite is the one that picks an owner from
the site's *position* in the tree, and an absent section is the evidence that pick was wrong — the
author of a bare `<§>12` in a declaration whose sections stop at `2.1` meant some other document.
Writing the pick in destroys the only record that they did: the token stops looking unfinished and
becomes a full canonical citation of a section that is not there, and the target they meant is no
longer recoverable from the text. So number-only ID shorthand keeps its behaviour
([§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation) unchanged) and this rewrite alone
gains the precondition.

Target existence is a precondition of the **rewrite**, never of the **finding**. Both findings
still fire at a refused site — the canonical-form error and the missing-section error
([§FS-check.3.24](FS-check.md#324-declaration-local-section-citation),
[§DF-declaration-local-section-shorthand.2.4](../decisions/functional/DF-declaration-local-section-shorthand.md#24-resolved-local-forms-are-real-edges))
— and the site remains an ordinary edge. What goes with the rewrite is the offer of the command
that would have made it, for the reason
[§FS-check.3.24](FS-check.md#324-declaration-local-section-citation)
gives: no finding names a command that would answer `rewrote 0 lines`.

### 2.5 Suppressed scopes

A suppressed scope, a file or a region inside one, suppresses **every** rewrite this command performs, save a kind's index entries ([§FS-fmt.6.1](FS-fmt.md#61-scope)), and nothing else: the file is still scanned, its citations still resolve, dangle, count as edges and ground their file, and `grund check`, `grund refs`, `grund show` and `grund cover` report exactly what they would without it, every finding included. So a number-only shorthand inside one is a `check` error that `grund fmt --write` does not clear there, and the author writes the canonical ID or lifts the suppression around it ([§FS-citations.rewrite-exemption](FS-citations.md#rewrite-exemption-where-the-text-forbids-the-rewrite), [§DF-fmt-suppression.2.4](../decisions/functional/DF-fmt-suppression.md#24-check-is-unchanged-and-the-repository-pays-for-that)). Decided in [§DF-fmt-suppression](../decisions/functional/DF-fmt-suppression.md#df-fmt-suppression-fmt-suppression-is-per-file-and-per-region-and-the-index-carve-out-outranks-both).

#### 2.5.1 `[fmt] exclude` — a file at a time

A file the `[fmt] exclude` list matches is scanned, read and checked like any other file, and not rewritten, save a kind's index entries ([§FS-fmt.6.1](FS-fmt.md#61-scope)). Which files an entry names, and what a malformed one does, is [§FS-config.3.10](FS-config.md#310-fmt--suppressing-the-rewrite), and the key is optional and additive ([§FS-config.3.10](FS-config.md#310-fmt--suppressing-the-rewrite)). It governs every rewrite, not one pass, so it sits in `[fmt]` ([§DF-fmt-suppression.2.1](../decisions/functional/DF-fmt-suppression.md#21-one-general-fmt-exclude-not-one-exclude-per-pass)). In a workspace every project is rewritten under its own config, so the list is read from the project that owns the file and a member's entries never reach its siblings ([§FS-workspace.8.5](FS-workspace.md#85-grund-fmt---cross-refs)).

#### 2.5.2 `grund:fmt off` / `grund:fmt on` — a region at a time

A comment line whose entire content is `grund:fmt off` suppresses every rewrite from the **next** line onward; one whose content is `grund:fmt on` resumes it. In Markdown the comment is an HTML comment — `<!-- grund:fmt off -->` — and in a source file it is a comment line under the configured `[scan] comment_prefixes` ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)): `// grund:fmt off`, `# grund:fmt on`, `/* grund:fmt off */`, or a line of a Python docstring, read for its content ([§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule)). The directive line itself is never rewritten. A region with no `on` after it runs to the end of the file, every file starts with the rewrite on, and a redundant directive, a stray `on` included, changes nothing.

Only an exact content match is a directive: `<!-- grund:fmt off please -->` and `// grund:fmt-off` are ordinary comments. A directive toggles nothing inside any Markdown fence, inside an inline code span in either kind of file, or inside a recognized multi-line string literal of a source file ([§FS-languages.multi-line-string](FS-languages.md#multi-line-string-a-string-literal-that-spans-lines)). A fence drawn in a source file's doc-comment is live text, where a citation resolves ([§FS-check.1.1](FS-check.md#11-recognized-citations)), so a directive illustrated in one does open a region.

The region is for a citation whose *position* carries meaning, such as an edge label of an ASCII diagram in an HTML `<pre>` block or a four-space indented block, which a wrap would knock out of alignment ([§DF-fmt-suppression.2.2](../decisions/functional/DF-fmt-suppression.md#22-an-in-text-region-not-a-rule-keyed-by-declaration-section)).

## 3. Outputs

- `0` — no changes needed, **or** `--write` succeeded, whether or not it made changes.
- `1` — `--check` found at least one line `--write` would change. Never returned by `--write`.
- `2` — an I/O error, or a path in the scanned tree that could not be read — a broken symlink or a symlink loop the scan would otherwise have read through ([§FS-scan.scope.links.unresolvable](FS-scan.md#scopelinksunresolvable-a-link-the-scan-cannot-resolve-is-reported-and-not-scanned-into), [§FS-scan.scope.links.owed](FS-scan.md#scopelinksowed-which-unresolvable-links-are-owed-a-report)), an unreadable directory — each named on stderr; or, in a run that needs the whole declaration set, an unresolved merge conflict in a declaration ([§FS-declarations.body.conflict](FS-declarations.md#bodyconflict-a-body-still-in-the-middle-of-a-merge)).

The report goes to stdout ([§FS-errors.1](FS-errors.md#1-streams)), and `error:` and `warning:` lines to stderr ([§FS-errors.2.2](FS-errors.md#22-cli-level-message)).

### 3.1 An unreadable path is named, and a partial run still rewrites

A run that does not need the whole declaration set names each unreadable path in the shape, stream and exit `2` of [§FS-check.2.4](FS-check.md#24-an-incomplete-run), after its report:

```text
error: src/gone.rs: broken symlink: the target does not exist; make the path readable, or keep it out of the scan
```

The rewrite still ran on every file that could be read — under `--write` those changes are on disk and in the report — so the `2` says the view of the tree was incomplete, not that nothing happened.

### 3.2 A run that needs the whole declaration set rewrites nothing

A run needs the **whole declaration set** when it wraps or re-derives a cross-reference, whether `--cross-refs` asked or `[fmt.cross_refs] enabled` turned the pass on ([§FS-fmt.6.7.2](FS-fmt.md#672-what-enabled-decides)), when it expands a shorthand ([§FS-fmt.2.4](FS-fmt.md#24-shorthand-to-canonical)), or when it runs the section-lock pass over a project's config root or the workspace root ([§FS-section-lock.write.rules](FS-section-lock.md#writerules-add-refresh-drop)). Because the pass turns itself on wherever the scope holds Markdown and `enabled` is set, as it is by default, this is the ordinary path in both modes. Such a run refuses up front and rewrites nothing when its scan met an unreadable path or a conflict block ([§FS-declarations.body.conflict](FS-declarations.md#bodyconflict-a-body-still-in-the-middle-of-a-merge)), because a rewrite resolved against a partial set, or a declaration that is two texts at once, could name the wrong one ([§FS-fmt.7.4](FS-fmt.md#74-no-write-without-a-complete-model)). The refusal binds to the scope however it was named ([§FS-fmt.7.1](FS-fmt.md#71-scope-equivalence)), and reaches the aliased project a rewrite resolves against ([§FS-fmt.7.4](FS-fmt.md#74-no-write-without-a-complete-model)). A run that needs no declaration set reads none, so it does not see a conflict block either: a source-only path, or a version-1 project with cross-references off and no lock or shorthand in play, makes its trigger and `--marker` rewrites, and `check` still fails the tree.

Every line of a refused run says that nothing was rewritten, unlike a partial run's; a conflict block's line names its first line and carries the message [§FS-declarations.checks.conflict-marker](FS-declarations.md#checksconflict-marker-unresolved-merge-conflict) gives it:

```text
error: nothing was rewritten: docs/broken.md: broken symlink: the target does not exist; make the path readable, or keep it out of the scan
error: nothing was rewritten: docs/login.md:7: unresolved merge conflict in FS-login, lines 7–11: resolve it by hand
```

A refused run names every unreadable path its scan found, once each in the scan's deterministic order, and prints nothing on stdout. A workspace-root run completes that preflight across every in-scope project before it rewrites any, so a failure in a later member leaves earlier projects byte for byte unchanged; its failures are one list, the root's first, then each member's in workspace scan order. Conflict blocks are named one line each, in the order of the scan's files and then of lines, the root's blocks first and then each member's in workspace scan order. A run that meets both is refused for its unreadable paths alone, since a conflict in a file nobody could read cannot be known. A workspace-root run is one run here, so an unreadable path in one project and a conflict in another report only the path. The conflict is named by the first run after the path is made readable, and `check` names both.

### 3.5 The dry-run report

`--check`, or no flag, prints one `path:line: <kind>` row per changed line ([§FS-errors.2.1](FS-errors.md#21-located-finding)), where `<kind>` names the rewrite: `trigger → marker` ([§FS-fmt.2.1](FS-fmt.md#21-trigger-to-marker)), `bare → marker` ([§FS-fmt.2.2](FS-fmt.md#22-bare-to-marker-with---marker)), `local section → canonical` and `shorthand → canonical` ([§FS-fmt.2.4](FS-fmt.md#24-shorthand-to-canonical)), or `markdown link` ([§FS-fmt.6](FS-fmt.md#6-cross-reference-emission)). A line is one row however many rewrites and tokens it holds. It leads with the first kind in that order that fired on it, which is the order the rewrites run in, and names every other kind that fired, each once, in the same order, as one list joined by `, `; the list follows the label with `, ` when the label carries no detail and with `; ` when it does:

```text
docs/runbook.md:3: bare → marker, markdown link: §FS-reserve.1 → [§FS-reserve.1](../requirements.md#1-hold-time)
docs/notes.md:3: trigger → marker: §FS-042 → §FS-042-user-login; shorthand → canonical
```

The expansion detail ([§FS-fmt.3.6](FS-fmt.md#36-an-expansion-or-a-link-names-the-text-it-writes)) stays on the leading label and a listed kind carries none, except that a listed `markdown link` keeps its repeated-anchor detail ([§DF-repeated-heading-anchor.decision.row](../decisions/functional/DF-repeated-heading-anchor.md#decisionrow-the-changed-link-names-the-versions)) as `: <detail>` and then names the links it writes, which end the row. A section-lock row never joins the list; its `; creates the file` or `; deletes the file` ends its own row ([§FS-section-lock.write.report](FS-section-lock.md#writereport-what-fmt-reports)). A rewrite that changed nothing is never named: a withheld token, an excluded, suppressed or unwrapped citation, or a link already right.

### 3.6 An expansion or a link names the text it writes

A row whose line expands a shorthand names, after whichever label leads, each expansion as `<token> → <full form>`, number-only and declaration-local alike, in line order, joined by `, `, as after `trigger → marker` in `docs/notes.md:3` above, or `local section → canonical: <§>2.1 → <§>FS-001-nine.2.1, <§>FS-042.1 → <§>FS-042-user-login.1; shorthand → canonical` for a line that expands both kinds. A `markdown link` label, leading or listed, is followed by `: ` and one item per link the line writes, in source order, joined by `, `: an existing wrap the write re-points ([§FS-fmt.6.3](FS-fmt.md#63-idempotency-and-re-derive)) reads `[<§>ID](old destination) → [<§>ID](new destination)`, each destination shown whole, and a citation the write wraps ([§FS-fmt.6.2](FS-fmt.md#62-form)) reads `<§>ID → [<§>ID](destination)`, or `<§>alias/ID → [<§>alias/ID](destination)` when qualified. A repeated-anchor note comes first, then `; ` and the items.

```text
docs/sources/README.md:7: markdown link: [§SRC-503-gamma](SRC-503-gamma.md#stale) → [§SRC-503-gamma](SRC-503-gamma.md#src-503-gamma-gamma-source-report)
docs/use.md:3: markdown link: repeated heading anchor #1-overview → #1-overview-1, numbered since grund 0.18; [§FS-b.1](functional-spec/FS-ab.md#1-overview) → [§FS-b.1](functional-spec/FS-ab.md#1-overview-1)
```

Each item's left side is exactly the bytes of one span the link pass replaces, read on the line as the row's earlier kinds leave it: a fresh span is one whole citation token, a re-pointed span runs from the wrap's `[` to the `)` that closes it, and the spans stand in source order, each after the previous one ends, so replacing each with its right side gives the line `--write` leaves, and a row can be applied by hand; a text search for the left side cannot, since `<§>FS-1` also starts `<§>FS-10`. An item is read by its brackets, not by splitting the row, because an author's destination may hold `, ` or ` → `: each side ends at the `)` that closes its `(`, a fresh item's left side at the end of its one citation token, and the next item begins after the `, ` that follows the right side's closing `)`. A link already right, one withheld ([§FS-fmt.6.4](FS-fmt.md#64-what-is-never-wrapped)), an author's link, an image or a value binding writes nothing and is not listed.

Only `bare → marker` and `trigger → marker`, on a line that expanded nothing, carry no detail, since they write nothing into the ID token ([§DF-shorthand-numeric-run.2.7](../decisions/functional/DF-shorthand-numeric-run.md#27-invention-is-reported-in-full-whatever-the-rule-decides), [§DF-fmt-link-row-names-what-it-writes](../decisions/functional/DF-fmt-link-row-names-what-it-writes.md#df-fmt-link-row-names-what-it-writes-a-markdown-link-row-of-fmts-preview-names-each-link-before-and-after)).

### 3.7 The `--write` report

With `--write`, the report on stdout ([§FS-errors.6](FS-errors.md#6-the-grund-init-transcript)) names what changed on disk: a `rewrote N line(s):` line carrying the totals, joined by `, `, then one `  <path> (<count>)` line per file touched, in lexicographic path order, both counts being changed lines; an empty change set prints `rewrote 0 lines` with no list.

```text
rewrote 7 lines: 6 citations marked, 6 links written
  docs/architecture/AR-ledger.md (1)
  docs/architecture/README.md (1)
  docs/runbook.md (1)
  requirements.md (2)
  src/inventory/reserve.py (2)
```

The totals, in [§FS-fmt.3.5](FS-fmt.md#35-the-dry-run-report)'s order and the lock last, are triggers marked, citations marked, local sections expanded, shorthands expanded, links written and section-lock entries; a zero is left out and a one is singular (`1 section-lock entry`). Each occurrence counts: a citation marked and wrapped counts in both, and a typed trigger that expands counts as a trigger and a shorthand. A link counts when new or re-pointed. A section-lock entry counts once per add, refresh, drop or rebuild row, and each such row is also one of the N lines, so `rewrote 0 lines` still means nothing was written ([§FS-section-lock.checks.unrecorded-section-citation](FS-section-lock.md#checksunrecorded-section-citation-a-locked-address-the-lock-does-not-hold)). A partial run ([§FS-fmt.3.1](FS-fmt.md#31-an-unreadable-path-is-named-and-a-partial-run-still-rewrites)) counts what reached disk. A lock file the write created or deleted ends its line ` created` or ` deleted` ([§FS-section-lock.write.report](FS-section-lock.md#writereport-what-fmt-reports)).

## 4. Why this exists

`grund fmt --marker --write` adopts the marker on an existing repository in one pass. `grund fmt --check` in CI catches what a contributor who bypassed the editor left behind, such as a trigger typed in GitHub's web editor, and a pre-commit hook running `fmt` on staged files keeps the canonical form in version control.

## 5. Configurability

The marker, the trigger, the recognized `KIND` set and each kind's effective ID format are read from `grund.toml` ([§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable)); marker and trigger default to `§` and `$$` ([§DF-reference-marker](../decisions/functional/DF-reference-marker.md#df-reference-marker-use--as-the-reference-marker-with--as-the-typing-trigger)). Every pass selects the kind before applying its grammar. Formatting a missing or committed external snapshot never executes its configured fetcher. A file `fmt` leaves unwritten is named by `[fmt] exclude` ([§FS-fmt.2.5.1](FS-fmt.md#251-fmt-exclude--a-file-at-a-time)), a region by a `grund:fmt off` directive ([§FS-fmt.2.5.2](FS-fmt.md#252-grundfmt-off--grundfmt-on--a-region-at-a-time)), and the links it writes are set by `[fmt.cross_refs]` ([§FS-fmt.6.7](FS-fmt.md#67-configurability)).

## 6. Cross-reference emission

In a file read as Markdown, a marked citation becomes a clickable Markdown inline link to its declaration, save where [§FS-fmt.6.4](FS-fmt.md#64-what-is-never-wrapped) leaves it as written, while the citation stays the source of truth: `grund check`, `grund show` and `grund refs` resolve the citation, never the URL ([§FS-non-goals.1](FS-non-goals.md#1-markdown-link-validation)). Decided in [§DF-md-link-emission](../decisions/functional/DF-md-link-emission.md#df-md-link-emission-grund-fmt-may-emit-clickable-markdown-links-alongside--prefixed-citations).

### 6.1 Scope

The pass runs **only on files read as Markdown** ([§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule)) in the default scope. Source files are never touched, since their hosts do not render Markdown.

A kind's index — the file its `[[kinds]]` entry names, or the `README.md` it defaults to ([§FS-config.3.4.2](FS-config.md#342-index--the-kinds-index-file)) — is linkified whatever `[fmt.cross_refs] enabled` says and inside both suppressed scopes, in dry run and `--write` alike, so that `grund fmt --write` stays the fix for `check`'s demand that an index entry be a full link ([§FS-check.3.17](FS-check.md#317-index-entry-is-not-a-link), [§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)) in every configuration. Decided in [§DF-index-always-linkified](../decisions/functional/DF-index-always-linkified.md#df-index-always-linkified-the-cross-reference-pass-always-runs-on-a-kinds-index-file) and [§DF-index-carve-out-one-entry](../decisions/functional/DF-index-carve-out-one-entry.md#df-index-carve-out-one-entry-the-index-carve-out-writes-one-entry-per-owed-id-and-check-names-no-fix-fmt-declines).

Where `enabled = false` would otherwise have skipped the file, or a suppressed scope the file or region, the pass writes one entry for each ID the index owes ([§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)) and leaves every other citation there alone, an owed ID's other mentions included ([§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)). In a suppressed scope the entry is the ID's first link in the file, kept current when it lies in the scope; where no link exists and the ordinary pass outside the scope writes none, it is the ID's first citation in the scope that the pass can wrap ([§FS-fmt.6.4](FS-fmt.md#64-what-is-never-wrapped)). An external source declaration is owed only while its canonical enrollment link already exists: the carve-out keeps that form, and neither infers membership from an ordinary external citation nor repairs a link whose noncanonical destination makes it ordinary. Outside a suppressed scope, `enabled = true` scopes nothing, because the pass already runs there, and a marked bare-ID citation the ordinary pass wraps canonically takes on the enrollment meaning of its stored form.

### 6.2 Form

A marker-prefixed citation (illustrated as `§FS-<foo>.3.1`) — a bare one only once `--marker` has marked it ([§FS-fmt.6.5](FS-fmt.md#65-interaction-with---marker)) — becomes:

```
[§FS-<foo>.3.1](<relative-path>#<anchor>)
```

- `<relative-path>` — the path from the citing file to the declaring file, in POSIX form (`../functional-spec/FS-<foo>.md`), computed from where the two files are and never from how a file argument was spelled: `grund fmt NAME` and `grund fmt ./NAME` preview, check and write the same link, and `--write` reports the file by the same path ([§FS-config.1](FS-config.md#1-file-location-and-discovery)). A declaration whose home is source code (a stub pointing at `src/foo.rs`) is linked to that file with no anchor ([§DF-md-link-emission.2.3](../decisions/functional/DF-md-link-emission.md#23-source-file-links)).
- `#<anchor>` — a heading anchor, present whenever the declaration's home is a Markdown file and the active profile is not `none`: the cited heading ([§FS-fmt.6.2.1](FS-fmt.md#621-the-anchor-names-the-cited-heading)), slugified from its rendered text ([§FS-fmt.6.2.2](FS-fmt.md#622-the-anchor-slugifies-the-headings-rendered-text)).

The citation text inside the brackets is preserved verbatim, including the marker.

#### 6.2.1 The anchor names the cited heading

For a `.<section>` citation the anchor is the cited section's heading; for a bare-ID citation it is the declaration's own heading — `§GOAL-<x>` → `[§GOAL-<x>](goals.md#goal-x-the-title)` rather than a bare link to `goals.md` ([§DF-declaration-anchor](../decisions/functional/DF-declaration-anchor.md#df-declaration-anchor-a-bare-id-markdown-link-points-at-the-declarations-heading-anchor)).

##### 6.2.1.1 Through a stub, the declaration's heading is the target's

A stub stands for the declaration its target holds ([§FS-declarations.stubs.pairing.1](FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned)), so a bare-ID citation of the stub's ID anchors on that declaration's heading in the file the link points at, whether or not `[scan] include` reaches it: the link is the one the same tree takes with the target scanned ([§REQ-one-answer.stubs.outside-include](../requirements/REQ-one-answer.md#stubsoutside-include-a-target-outside-the-include-answers-as-if-scanned)), one stub or several to the same target ([§FS-declarations.stubs.pairing](FS-declarations.md#stubspairing-how-a-stub-pairs-with-what-it-points-at)), and its anchor is the one `show` reports ([§REQ-one-answer.writes.anchors](../requirements/REQ-one-answer.md#writesanchors-shows-anchor-is-the-anchor-fmt-writes)). A stub `# FS-a: [../notes/a.md](../notes/a.md)` in the walk, whose target outside it opens `# FS-a: A`, links `<§>FS-a` to `../notes/a.md#fs-a-a`. Where the target declares the ID once, which is the reading `show` takes, the stub's own heading, which names a file rather than a title, never lends the anchor: the link is never `#fs-a`. Where it declares the ID twice the ID is a duplicate, and the citation is not wrapped at all ([§FS-fmt.6.4](FS-fmt.md#64-what-is-never-wrapped)), scanned target or not. Why a link an older grund wrote to the stub's anchor now fails `fmt --check` is [§DF-stub-heading-from-unscanned-target](../decisions/functional/DF-stub-heading-from-unscanned-target.md#df-stub-heading-from-unscanned-target-a-bare-id-link-through-a-stub-anchors-on-its-targets-heading-whether-or-not-the-scan-reaches-it).

#### 6.2.2 The anchor slugifies the heading's rendered text

The anchor is the heading's **rendered text** ([§FS-declarations.heading.rendered-text](FS-declarations.md#headingrendered-text-a-headings-rendered-text)), its ID as declared ([§FS-config.3.2.8](FS-config.md#328-numeric-spelling-survives-reading-and-reporting)), slugified per the configured renderer profile ([§FS-fmt.6.7.1](FS-fmt.md#671-anchor_format-names-a-renderer-profile)) — for the default `github` profile, `### 6.2 Form` produces `#62-form`, and `## goals: Scope` produces `#goals-scope`. Retitling that heading `## goals: Intent` refreshes an existing wrap to `#goals-intent` on the next pass while the stored name and the citation text stay `goals`. The `github` (and `gitlab`) profile reproduces `github-slugger` byte for byte: disallowed characters are deleted in place and each remaining space becomes one `-`, with no run-collapsing and no trailing-`-` trim — `## A — B` → `#a--b`, `` ## 6. Watch mode (`--watch`) `` → `#6-watch-mode---watch` ([§DF-github-anchor-fidelity](../decisions/functional/DF-github-anchor-fidelity.md#df-github-anchor-fidelity-the-github-anchor-profile-reproduces-github-slugger-exactly)). A repeated slug in one file is numbered `-1`, `-2` ([§DF-repeated-heading-anchor.decision](../decisions/functional/DF-repeated-heading-anchor.md#decision-decision)).

### 6.3 Idempotency and re-derive

Every cross-reference pass recomputes the canonical URL inside each existing wrap and rewrites it where it differs ([§DF-md-link-anchor-strategy.2.2](../decisions/functional/DF-md-link-anchor-strategy.md#22-re-derive-on-every-pass-supersede-fs-fmt63)), so a heading rename or a file move that invalidates a wrap is a one-line `fmt` diff on the next pass, not a silently broken link, and the pass after it finds nothing to change ([§FS-fmt.requirements.fixed-point](FS-fmt.md#requirementsfixed-point-a-tree-fmt-has-written-is-a-fixed-point)). A link the run writes to a section heading whose own citation the same run expands ([§FS-fmt.2.4](FS-fmt.md#24-shorthand-to-canonical)) — from another file, its own, or another rewritten heading — carries the anchor of the text the run leaves on disk, not the text it read. An exact marked off-grammar citation backed by a catalog declaration ([§FS-config.3.2](FS-config.md#32-id--id-grammar)) is wrapped and re-derived like a conforming one; unmarked and declaration-less off-grammar candidates are not, since the formatter reads the shared scanner result and relaxes none of its grammar.

#### 6.3.1 An existing wrap is recognized by its brackets

How an existing wrap is recognized, for the re-derive and the no-double-wrap rule alike, is [§FS-citations.wrap.recognized](FS-citations.md#wraprecognized-an-existing-wrap-is-recognized-by-its-brackets).

### 6.4 What is never wrapped

The never-rewrite zones of [§FS-fmt.2.3](FS-fmt.md#23-what-is-never-rewritten) hold for this pass as for every other, except that a kind's index entry in a suppressed scope is wrapped ([§FS-fmt.6.1](FS-fmt.md#61-scope)). The citation bytes of a value binding, or of an attempt at one that `check` refuses, are not wrapped either ([§FS-values.8](FS-values.md#8-formatting-stability)).

Neither is a citation inside a link its author wrote, in any form [§FS-citations.wrap.enclosing.shape](FS-citations.md#wrapenclosingshape-which-links-enclose) lists ([§FS-citations.wrap.enclosing](FS-citations.md#wrapenclosing-a-citation-inside-a-link-its-author-wrote-is-left-to-the-author)), nor one that is an image's whole description ([§FS-citations.wrap.enclosing.images](FS-citations.md#wrapenclosingimages-an-image-is-not-a-link)): that markup stays as written, destination and all, in a kind's index as anywhere else. This protection outranks the index carve-out, because an enclosing link is already an entry ([§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)). It is not a never-rewrite zone: the earlier passes still rewrite the token inside the link.

The pass also leaves as written, in both modes and with any wrap already there, a citation whose target `check` reports missing or ambiguous, and the error is `check`'s:

- a dangling citation, whose declaration the scan cannot locate.
- a `.<section>` citation of a section its declaration does not have ([§FS-check.3.2](FS-check.md#32-missing-section)), anchor or not: a source-file home reached through a stub, in or outside `[scan] include`, and any home under the `none` profile skip it the same, while the bare ID and the sections the declaration has still link to the file. The scan decides which sections exist, in a stub's target outside the walk as well ([§FS-check.3.2](FS-check.md#32-missing-section)): a heading inside a fence ([§FS-show.2.5](FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example)) or after the body has closed ([§FS-declarations.checks.section-outside-declaration](FS-declarations.md#checkssection-outside-declaration-section-outside-a-declaration)) bounds no section of the cited declaration, and a fenced copy of the declaration's own heading opens nothing, so a real heading after it that belongs to the declaration above is no section of the cited ID.
- a citation of an ID that, once its stubs are paired, has one declaration that is a broken stub ([§FS-declarations.checks.broken-stub](FS-declarations.md#checksbroken-stub-broken-inline-spec-stub)), by the verdict the scan reaches on each stub ([§FS-declarations.stubs.verdict](FS-declarations.md#stubsverdict-one-verdict-per-stub-reached-once-per-scan)), or has more than one declaration, a broken stub beside a declaration and a target that declares the ID twice included ([§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration)): `fmt` picks none, as `show` does not ([§FS-show.2.2.1](FS-show.md#221-ambiguous-id), [§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution)).

### 6.5 Interaction with `--marker`

The link pass wraps only marked citations: with `--marker` the marker pass runs first and the link pass wraps what it marked; without it, bare citations stay bare and unwrapped.

### 6.7 Configurability

`[fmt.cross_refs]` holds the cross-reference settings; its keys and their defaults are [§FS-config.3.7](FS-config.md#37-fmtcross_refs--cross-reference-emission)'s.

#### 6.7.1 `anchor_format` names a renderer profile

`anchor_format` names one of the profiles of [§DF-md-link-anchor-strategy.2.3](../decisions/functional/DF-md-link-anchor-strategy.md#23-renderer-profiles):

- `github` (default) — GitHub's slugger; covers the most common host.
- `gitlab` — GitLab's slugger.
- `mkdocs` — MkDocs / Python-Markdown TOC extension's slugger.
- `pandoc` — Pandoc's `auto_identifiers` algorithm.
- `none` — emit no anchor; produce a file-level link with no fragment.

#### 6.7.2 What `enabled` decides

Generated configs and the built-in default set `enabled = true`, and a project with another renderer chooses its `anchor_format` ([§DF-md-link-default-on](../decisions/functional/DF-md-link-default-on.md#df-md-link-default-on-markdown-cross-reference-links-default-on-for-github-review-and-discovery)). With `enabled = true` the cross-reference pass runs, without `--cross-refs`, in every `grund fmt` run whose rewrite scope holds at least one Markdown file, dry run and `--write` alike. A source-only scope, such as `grund fmt src/app.rs --write`, stays on the marker and trigger path unless `--cross-refs` is passed, and skips the full-project link-target scan. With `enabled = false` both modes skip the pass unless `--cross-refs` is passed, apart from the index carve-out ([§FS-fmt.6.1](FS-fmt.md#61-scope)).

## 7. One model, and a write step

`fmt` is the model `grund check` verifies plus a write step, never a second implementation of it. Every claim it makes — what a token is, which lines will change, which paths could not be read, which files it may edit — is read from that model, and where the model is incomplete, or the proof that it is complete is missing, `fmt` refuses ([§FS-fmt.3.2](FS-fmt.md#32-a-run-that-needs-the-whole-declaration-set-rewrites-nothing)). Decided in [§DF-fmt-one-model](../decisions/functional/DF-fmt-one-model.md#df-fmt-one-model-fmt-is-the-shared-verified-model-plus-a-write-step-and-completeness-is-a-precondition-rather-than-a-convention).

The binding runs from `fmt` to `check`'s **model** — the same scan, the same token grammar, the same account of what could not be read — and never from `check`'s **findings** to what `fmt` can rewrite: `fmt` wraps citations `check` accepts as they stand, and `check` reports what `fmt` cannot fix. The one exception is ruled and closed: `check` withholds its resolving shorthand error in the contexts [§FS-citations.rewrite-exemption](FS-citations.md#rewrite-exemption-where-the-text-forbids-the-rewrite) names, at the accepted price that a shorthand inside a string literal that resolves to exactly one declaration is flagged by no command.

### 7.1 Scope-equivalence

`grund fmt` with no `<path>` and `grund fmt <path>` where `<path>` names the default scope ([§FS-fmt.1](FS-fmt.md#1-inputs)) are one run written two ways: identical stdout, stderr, exit code and effects on disk, in every mode and combination of flags. How a caller reached a scope is not part of it: a run that already holds a scan of the scope — a workspace-root run holding each member's, or the CLI reusing the scan it made resolving the current directory — may reuse it only after the completeness check a fresh scan would have made ([§FS-fmt.7.4](FS-fmt.md#74-no-write-without-a-complete-model)).

### 7.2 Reader-equivalence

The unreadable paths `fmt` reports are the ones `check` reports on the same scope: the same paths, spelled against the same config, with the same reasons, in the same deterministic order, in every form of the command — plain, refused, narrowed and workspace — because they are one scan. Equality is of the **(path, reason)** pairs, not of the bytes of the line: the one licensed difference is the `nothing was rewritten:` prefix of a refused run ([§FS-fmt.3.2](FS-fmt.md#32-a-run-that-needs-the-whole-declaration-set-rewrites-nothing)). Any other difference is a disagreement between two readers of one tree.

### 7.3 Preview-equivalence

The rows `fmt --check` prints are the change set `fmt --write` applies to the same tree: the same files, the same line numbers, and on each line every kind its row named ([§FS-fmt.3.5](FS-fmt.md#35-the-dry-run-report)) and the text each expansion and each link was named as writing ([§FS-fmt.3.6](FS-fmt.md#36-an-expansion-or-a-link-names-the-text-it-writes)). The write's totals count those kinds ([§FS-fmt.3.7](FS-fmt.md#37-the---write-report)), and a lock file previewed as created or deleted is created or deleted. The dry run is the write run with the write withheld, not a second computation, so neither mode holds a refusal, a carve-out or a scope rule the other does not: a rewrite the dry run lists and `--write` never performs would keep every `fmt --check` gate red.

### 7.4 No write without a complete model

Where a rewrite needs the whole declaration set — a shorthand to expand ([§FS-fmt.2.4](FS-fmt.md#24-shorthand-to-canonical)), a cross-reference to wrap or re-derive ([§FS-fmt.6](FS-fmt.md#6-cross-reference-emission)), or a section-lock entry to write ([§FS-section-lock.write.rules](FS-section-lock.md#writerules-add-refresh-drop)) — the completeness of that set is a **precondition of the rewrite**, not a habit of each caller: the rewrite is made only from a set whose scan met no error and found no conflict block ([§FS-declarations.body.conflict](FS-declarations.md#bodyconflict-a-body-still-in-the-middle-of-a-merge)), however the run reached it ([§DF-fmt-one-model.2.5](../decisions/functional/DF-fmt-one-model.md#25-completeness-is-a-type-the-consumer-demands-because-prose-has-failed-twice)). Where it fails, the run refuses as [§FS-fmt.3.2](FS-fmt.md#32-a-run-that-needs-the-whole-declaration-set-rewrites-nothing) says.

A run that writes works out every file's new text and every lock change before it writes any, then writes the files in walk order and the locks after them. A refusal therefore leaves the tree as the run found it wherever it comes — at the cross-reference pass, at the first shorthand whose scan was deferred ([§FS-fmt.2.4](FS-fmt.md#24-shorthand-to-canonical)), or in the lock pass. Two failures are not refusals, and each says what it reached: a file that cannot be read partway through the walk ends the run before anything is written, so its error lists no changes, nor does the partial output a binding carries ([§FS-distribution.3.3.2](FS-distribution.md#332-operational-failures-and-invalid-arguments)); a write that fails partway still reports the files written before it, because those are on disk.

A qualified citation resolves against the project its alias names ([§FS-workspace.8.5](FS-workspace.md#85-grund-fmt---cross-refs)), so rewriting one — a shorthand expanded, a link wrapped or re-pointed — needs that project's proof. Where the aliased project's scan met an error, the run writes nothing and refuses with [§FS-fmt.3.2](FS-fmt.md#32-a-run-that-needs-the-whole-declaration-set-rewrites-nothing)'s lines, naming that project's unreadable paths beside its own, each once, however it reached the rewrite: a narrowed `fmt --write docs` whose own walk is clean, a link only an ignore file covers, or a workspace-root run. A run with no rewrite resolved against that project — no qualified citation of it, one already right, or one in a never-rewrite zone — is not refused. A conflict block in the aliased project is not asked about, because it changes no recognition ([§FS-declarations.body.conflict](FS-declarations.md#bodyconflict-a-body-still-in-the-middle-of-a-merge)); the citing project's own blocks still refuse.
