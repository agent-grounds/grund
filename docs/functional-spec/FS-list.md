# FS-list: grund lists every declared ID

The `list` subcommand prints the repo's ID catalog: every declaration, where it lives, and its one-line title — or, narrowed with `--kind`/`--summary`, just the slice an agent asked for. It is the same catalog `grund <ID>` reads from and the broad counterpart of `grund refs` — `refs` answers "who cites *this* ID?", `list` answers "what IDs are there?". An agent that has been told to ground itself with `grund <ID>` needs a way to discover the `<ID>`s; a human auditing a spec tree needs the same. Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible) (no `grep` for `^# [A-Z]+-` across the tree), [§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file), and the agent-grounding loop in [§GRUND-grund](../grund.md#grund-grund-agents-stay-grounded-in-the-spec).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, home, citable, body, section, coordinate,
lead, index, catalog), [§FS-terms.terms.2](FS-terms.md#terms2-citations) (citation), [§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (source declaration,
stub, doc-comment), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope, workspace, member, alias),
[§FS-terms.terms.6](FS-terms.md#terms6-rules-and-directions) (rule), and [§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations) (value, fetcher, snapshot).

- **coordinate-size** — One row of `--size`, for one declaration or citable
  section: its lead and full-body measurement, or the site alone where there
  is nothing to measure.

## requirements: What every listing holds to

Two requirements on `list` as a whole, each carrying a mark as [§FS-config.requirements](FS-config.md#requirements-what-the-config-contract-holds-to) defines them. There is no third for what the tool states once for every query: listing exactly the declarations and sections `check` validates and `show` resolves, and counting what `refs` and `show` answer, is [§REQ-one-answer.contract](../requirements/REQ-one-answer.md#contract-one-question-one-tree-one-answer) with [§REQ-one-answer.identity.declarations](../requirements/REQ-one-answer.md#identitydeclarations-every-command-reads-one-declaration-set) and [§REQ-one-answer.sections.map](../requirements/REQ-one-answer.md#sectionsmap-every-surface-reads-one-section-map); reading offline, read-only and to the same bytes on every run is [§REQ-runs-offline.1](../requirements/REQ-runs-offline.md#1-read-and-verification-paths-execute-nothing), [§REQ-no-data-loss.1](../requirements/REQ-no-data-loss.md#1-the-query-surface-never-writes) and [§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes); and a narrowed listing as a rung of the escalation ladder is [§GOAL-token-economy.1](../goals.md#1-what-this-requires).

### requirements.flagged-rows: A finding marks a row and never removes one — realized

Whatever `check` reports about a declaration, `list` still shows it: a duplicated ID keeps one flagged line per home and picks no winner ([§FS-list.2.6](FS-list.md#26-duplicate-declarations)), each claimant of a duplicated section keeps its own marked size row ([§FS-list.2.7](FS-list.md#27-duplicate-sections)), a broken stub is listed at its own site with its target ([§FS-list.2.5](FS-list.md#25-inline-homes-stay-canonical), [§FS-list.3.4.3](FS-list.md#343-text-rows)), and an off-grammar declaration stays a normal row ([§FS-list.2.3](FS-list.md#23-off-grammar-declarations)). Nor does a finding fail the run: `list` has no exit `1` ([§FS-list.4](FS-list.md#4-exit-codes)). The located error is left to `check`, so the catalog shows the situation `check` judges, never a tree tidied of what it objects to. Serves [§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution), which reports duplicates rather than ranking them, and [§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work).

### requirements.one-selection: Every filter narrows every mode alike — realized, directional for the next filter

The filters make one selection and every output mode reads it: `--kind`, `--project`, `--unused`, `--selector` and `--glob` narrow the per-declaration rows, the `--summary` counts, the `--total` count and the `--size` rows alike, compose with one another by intersection, and apply before `--top` keeps its largest rows ([§FS-list.1.2](FS-list.md#12---project), [§FS-list.1.4](FS-list.md#14---summary), [§FS-list.3.3.1](FS-list.md#331-narrowed-summaries), [§FS-list.3.4.5](FS-list.md#345-order-and---top), [§FS-rules.8](FS-rules.md#8-command-surfaces)). A refusal reads it too: the `known kinds:` line is built from the same selection in every mode ([§FS-list.1.2](FS-list.md#12---project)). This binds a filter added from here on: it narrows that one selection, and so reaches every mode at once, rather than being a predicate one mode applies and another forgets. Serves [§GOAL-token-economy.1](../goals.md#1-what-this-requires), whose ladder needs a slice to name one set at every rung.

## 1. Inputs

```
grund list [<path>] [--selector <selector>] [--glob <pattern>]… [--kind <KIND>[,<KIND>…]]… [--project <alias>[,<alias>...]] [--unused]
           [--summary | --total] [--size[=<unit>[,<unit>…]]] [--top <N>] [--format text|json]
```

Which entries `<path>` reports, and what they resolve against, is [§FS-scan.path.list](FS-scan.md#pathlist-list-reports-the-paths-entries-and-resolves-against-the-project). `--format text|json` picks the output shape ([§FS-list.3](FS-list.md#3-outputs)), `text` by default.

`--kind` selects configured citable kinds and refuses an unknown one, or a non-citable one with the two-line message giving its reason ([§FS-list.1.1](FS-list.md#11---kind)). `--project` narrows a workspace catalog to the named projects ([§FS-list.1.2](FS-list.md#12---project)). `--unused` keeps only uncited declarations, discounting a kind's index entries and leaving out a configured `E2E` kind's cases unless `--kind` selects them ([§FS-list.1.3](FS-list.md#13---unused), [§FS-list.1.3.1](FS-list.md#131-a-configured-e2e-kind-is-left-out-unless-selected)). `--summary` prints one count per kind under those same filters ([§FS-list.1.4](FS-list.md#14---summary)). `--size` and `--top` switch to coordinate-size rows over a closed unit set ([§FS-list.1.5](FS-list.md#15---size), [§FS-list.1.6](FS-list.md#16---top)); their syntax and combination errors are launch errors, decided before config discovery or scanning ([§FS-list.1.7](FS-list.md#17-size-selector-errors)).

`list` is a query, like `show` and `refs` — non-interactive, no prompts ([§FS-non-goals.10](FS-non-goals.md#10-interactive-mode)).

`--selector <selector>` filters the shared catalog to the declaration and
named-chapter units matched by a phase-1 rule subject, and is always a flag so
the existing positional remains the scan path. Its composition, text/JSON
rows, empty-success result, and exit-2 refusals are
[§FS-rules.8](FS-rules.md#8-command-surfaces)'s.

`--glob <pattern>` is a second unit source beside the selector: repeatable, its patterns
union, and it selects the catalog coordinates they match, sections at any depth included
([§FS-coordinate-globs](FS-coordinate-globs.md#fs-coordinate-globs-a-coordinate-glob-selects-catalog-coordinates-by-pattern)). It composes with every other filter and mode exactly as `--selector`
does, by intersection, with the same rows in the same order, and both together intersect
([§FS-coordinate-globs.scope.filters](FS-coordinate-globs.md#scopefilters-filters-intersect-and-never-widen)). It is always a flag: a pattern typed as the positional
stays a scan path, and its failure gains a hint ([§FS-coordinate-globs.errors.hint](FS-coordinate-globs.md#errorshint-a-glob-typed-as-path-gets-a-hint)). `--total`
prints the count of the rows the same filters list instead of the rows ([§FS-list.total](FS-list.md#total---total-a-count-rung)).

### 1.1 `--kind`

`--kind <KIND>[,<KIND>…]` lists only declarations of the named kinds, each a configured *citable* `[[kinds]]` entry ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)). It accepts a comma-separated list (`--kind FS,AR`) and may be repeated (`--kind FS --kind AR`); the selections union.

An unknown kind *anywhere* in the selection is a CLI-level error ([§FS-list.4](FS-list.md#4-exit-codes)): a typo'd `--kind` must not silently produce an empty — or merely short — catalog. A configured **non-citable** kind ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)) is refused the same way and for the same reason — it would select nothing, every time — but with the reason instead of "unknown", because it is a real row in the table rather than a typo:

```text
error: kind `skill` declares no IDs — skills/ is not a citable home
known kinds: GRUND, GOAL, FS, AR, DF, DA, RM
```

`code`, the homeless kind's default name, in a project whose `[[kinds]]` declares no homeless kind ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)), is not citable either. It has no row, so it has no home to name, and `--kind code` is refused with [§FS-errors.2.5.1](FS-errors.md#251-a-kind-that-declares-no-ids)'s reason in its form with no home:

```text
error: kind `code` declares no IDs
known kinds: GRUND, GOAL, FS, AR, DF, DA, RM
```

Through `0.18.0` that headline followed the legacy ``unknown kind `code` `` and ended in `; this wording changes in grund 0.19.0`; the window of [§FS-errors.3.8](FS-errors.md#38-the-one-grammar-wording-migration) closed for `code` in `0.19.0` ([§FS-errors.3.8.1](FS-errors.md#381-the-final-templates)).

The `known kinds:` line lists the citable kinds only: they are the whole set this selector accepts. Under `--project` that set is the selected projects' kinds ([§FS-list.1.2](FS-list.md#12---project)).

Selected workspace projects may configure one kind differently. The kind is accepted when any selected project has it citable, which is exactly when the `known kinds:` line names it, and it is refused as non-citable only when none does. In a workspace whose `root` project configures `SKILL` with `citable = false` and whose member `api` has `SKILL` citable, `grund list --kind SKILL` lists `api/SKILL-intro`, while `grund list --project root --kind SKILL` is refused with the headline above and `known kinds: FS, AR`. The verdict is the one every command reads for a kind name in its scope ([§FS-scan.projects.kinds](FS-scan.md#projectskinds-whose-kinds-a-run-reads)), and the headline is [§FS-errors.2.5.1](FS-errors.md#251-a-kind-that-declares-no-ids)'s.

#### 1.1.1 A scope with no citable kind

Where the scope has no citable kind at all, as in a project whose config declares no kinds ([§FS-config.3.4.4.5](FS-config.md#3445-kinds---declares-no-kinds)), the line does not end on an empty list. It reads `known kinds: none — declare one in grund.toml first`, wherever a refusal prints it: `--kind`, `--selector` ([§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector)), and `grund id` ([§FS-id.1.1](FS-id.md#11-unknown-and-non-citable-kinds)). The headline above it is unchanged, so the kind the user typed is still named:

```text
$ grund id FS "Password reset"
error: unknown kind `FS`
known kinds: none — declare one in grund.toml first
```

### 1.2 `--project`

`--project <alias>[,<alias>...]` lists, in workspace mode, only declarations from the named projects, and composes with `--kind` by intersection. Outside workspace mode it is a CLI-level error (exit `2`); an unknown alias is also a CLI-level error (exit `2`), the same shape as an unknown `--kind` ([§FS-list.4](FS-list.md#4-exit-codes)). Workspace qualification, size behavior, and member-local invocation are specified in [§FS-workspace.8.3](FS-workspace.md#83-grund-list).

A narrowed run accepts only the kinds its selected projects configure, so `--kind` is looked up in those projects alone, accepted when any of them has it citable ([§FS-list.1.1](FS-list.md#11---kind)), and a kind only another loaded project configures is unknown here. Its refusal ([§FS-list.1.1](FS-list.md#11---kind)) ends in a `known kinds:` line that lists the citable kinds of the selected projects only, once each, in configuration order. In a workspace whose `root` project configures `FS` and `AR` and whose member `api` configures `API`:

```text
$ grund list --project root --kind API
error: unknown kind `API`
known kinds: FS, AR
```

Without `--project` every loaded project is selected, and the same refusal lists `FS, AR, API`. Every `list` mode that ends a refusal in this line, `--size` and `--selector` included ([§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector)), builds it from the same selection.

### 1.3 `--unused`

`--unused` lists only declarations that no recognised citation points at — the same set `check` warns on ([§FS-check.4.1](FS-check.md#41-unused-declaration)) — except the cases of a configured `E2E` kind, which it leaves out unless `--kind` selects them ([§FS-list.1.3.1](FS-list.md#131-a-configured-e2e-kind-is-left-out-unless-selected)). The default kinds declare no such kind, so there it has no case to leave out ([§FS-list.1.3.2](FS-list.md#132-the-default-kinds-have-no-e2e-kind)).

A citation that is a kind's own index entry ([§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)) is discounted here too, for the reason `check` discounts it and within the same bounds ([§FS-check.4.1.2](FS-check.md#412-an-index-entry-does-not-count)): a section citation or a value binding's citation in that index is not the entry and still counts ([§FS-check.4.1.2.1](FS-check.md#4121-a-section-citation-or-a-value-binding-in-the-index-is-not-the-entry)).

#### 1.3.1 A configured `E2E` kind is left out unless selected

Where a config declares an `E2E` case kind ([§FS-config.3.4.4.3](FS-config.md#3443-e2e-is-configured-not-a-default)), `--unused` leaves its cases out **unless `E2E` is explicitly selected with `--kind`**. An e2e case is a proof artifact, exercised by being run, not a citation target, so it is uncited by construction and would only ever bury the actionable signal (uncited specs, decisions, goals) in a bare `--unused` query. In such a config `--unused --kind E2E` lists uncited cases only, while `--unused --kind FS,E2E` lists uncited `FS` declarations plus uncited `E2E` cases because `E2E` was explicitly requested. The rule is the same under config version 1 and version 2: it follows the configured `E2E` kind, not the version that spells it.

#### 1.3.2 The default kinds have no `E2E` kind

The default kinds, and the config `grund init` writes from them, put e2e in the non-citable folder kind `e2e` at `tests/e2e` ([§FS-config.3.4.4](FS-config.md#344-the-default-kinds)). Its files declare no IDs, so a bare `--unused` there has no e2e case to leave out and none to list. `--kind E2E` names no kind such a config has and is refused as an unknown kind, and `--kind e2e` as a kind that declares no IDs, both with exit `2` ([§FS-list.1.1](FS-list.md#11---kind)). An inventory of uncited cases therefore exists only where a config declares the `E2E` kind.

A query taught for any repository, as the examples of `grund list --help` are ([§FS-cli.2.3](FS-cli.md#23-a-subcommands-help-page)), therefore works under the default kinds. It states the `E2E` rule as one for a configured `E2E` kind, and an example that composes `--unused` with `--kind` selects a kind the default kinds declare, such as `grund list --unused --kind FS`, never `--kind E2E`.

### 1.4 `--summary`

`--summary` prints one line per kind instead of one per declaration, narrowed by `--kind` and `--unused` as the per-declaration form is; [§FS-list.3.3](FS-list.md#33---summary) gives its shape.

### 1.5 `--size`

`--size[=<unit>[,<unit>…]]` switches from declaration rows to the coordinate-size rows in [§FS-list.3.4](FS-list.md#34---size--per-coordinate-lead-and-full-body-measurements). The optional value is accepted only in the same argument with `=`; a following bare word remains the existing `<path>` positional. Bare `--size` selects `lines,words,bytes`, in that order. An explicit list is case-sensitive, preserves caller order, and collapses repeated units to their first occurrence. The closed unit set is `lines`, `words`, and `bytes`: an empty item or any other value, including `tokens`, is a CLI-level error, and there is no recognized or reserved token-counting spelling. The flag may appear once and cannot be combined with `--summary`.

### 1.6 `--top`

`--top <N>` / `--top=<N>` retains at most the `N` rows with the largest lead in the first selected size unit, after every ordinary filter ([§FS-list.3.4.5](FS-list.md#345-order-and---top)). `N` is one positive base-10 integer. The flag may appear once, requires `--size`, and cannot be combined with `--summary`.

### 1.7 Size-selector errors

Size-selector syntax and combination errors are validated before config discovery or scanning. They write one raw `error:` line to stderr and exit `2` ([§FS-list.4](FS-list.md#4-exit-codes)): `--size may only appear once`; `--size requires one or more units`; `unknown size unit \`<unit>\` (expected lines, words, or bytes)`; `--top may only appear once`; `--top requires a positive integer`; `--top requires --size`; `--size cannot be combined with --summary`; or `--top cannot be combined with --summary`.

## 2. Behaviour

`list` runs the same scan as `check` ([AR-scanner](../architecture/AR-scanner.md#ar-scanner-how-grund-discovers-declarations-and-citations)) and emits, for every declaration the scan found, one catalog line. In size mode it additionally emits every numbered or named section the same scan makes citable. The set of declarations and sections is exactly the set `check` validates and `show` can attempt to resolve, so the three never disagree on what exists. This includes Markdown and source declarations, value-source declarations ([§FS-values.terms](FS-values.md#terms-terms)) and their sections, and configured `E2E` case declarations.

In that catalog, the entries of an opted-in kind's value sources are declarations ([§FS-list.2.1](FS-list.md#21-value-source-values)), a committed fetched snapshot is an ordinary declaration ([§FS-list.2.2](FS-list.md#22-per-kind-formats-and-fetched-snapshots)), and a retained off-grammar declaration is a normal catalog row ([§FS-list.2.3](FS-list.md#23-off-grammar-declarations)). Declarations come out sorted by ID ([§FS-list.2.4](FS-list.md#24-order)). A stub-and-inline pair shows one line, at the source file holding the body ([§FS-list.2.5](FS-list.md#25-inline-homes-stay-canonical)). An ID declared in several homes keeps one flagged line per home, and in size mode each claimant of a duplicated section keeps its own row ([§FS-list.2.6](FS-list.md#26-duplicate-declarations), [§FS-list.2.7](FS-list.md#27-duplicate-sections)). `list` prints no bodies or citations, modifies nothing, and never lists an ID that is cited but not declared ([§FS-list.2.8](FS-list.md#28-what-it-is-not)).

### 2.1 Value-source values

The entries of an opted-in kind's value sources ([§FS-values.terms](FS-values.md#terms-terms)) are declarations in the same catalog. `list` shows their ID and exact location without inventing a title; duplicate and unused filters apply as they do to Markdown values. A marked section value remains metadata on its enclosing declaration row, never a synthetic declaration or second row ([§FS-values.6](FS-values.md#6-shared-catalog-consumers)).

### 2.2 Per-kind formats and fetched snapshots

Per-kind formats ([§FS-config.3.4.10](FS-config.md#3410-format-resolve-and-fetch--external-snapshot-kinds)) are parsed through that shared catalog. A committed fetched snapshot appears as an ordinary declaration; a missing snapshot does not appear because there is no declaration to list. Listing never executes a fetcher.

### 2.3 Off-grammar declarations

An exact off-grammar declaration retained under [§FS-config.3.2](FS-config.md#32-id--id-grammar) is a normal catalog row, rendered exactly as written with its body-independent metadata, sections, citation count, duplicate flag, and stable sort position. Text and JSON keep their existing schemas. The row cannot disappear merely because `check` also reports its conformance mismatch ([§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss)).

### 2.4 Order

Declarations come out sorted by ID — kind, then number, then slug. The result is deterministic for a given tree.

### 2.5 Inline homes stay canonical

When an ID's home is a declaration with a stub in a kind's home pointing at it ([§FS-declarations.stubs.shape](FS-declarations.md#stubsshape-what-a-stub-looks-like), the [§FS-declarations.checks.broken-stub](FS-declarations.md#checksbroken-stub-broken-inline-spec-stub) / [§FS-show.2.3](FS-show.md#23-source-declarations-in-code-and-doc-comments) arrangement), `list` shows **one** line for that ID, naming the target's declaration, its `<path>:<line>` and its title, wherever the target sits, inside `[scan] include` or outside it ([§FS-declarations.stubs.pairing.4](FS-declarations.md#stubspairing4-every-surface-shows-a-paired-declaration-at-its-target)). An external source declaration enrolled directly by its kind's index ([§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)) likewise appears once at the source home: the index link creates no declaration to collapse. A *broken* stub (its target missing, or the target has no matching source declaration) is not paired with anything, so it does appear, listed at the stub's own location with a `→ <target>` note; `check` reports the breakage in located form.

### 2.6 Duplicate declarations

When an ID is declared in more than one independent home — the [§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration) error — `list` prints one line per home, each flagged so the duplication is visible at a glance. `list` does not pick a winner; it shows the situation and leaves the located error to `check`.

### 2.7 Duplicate sections

Size mode likewise prints one row per site that claims a duplicated section coordinate and marks each row duplicate. Declaration and section duplicates are measured from that site only: rows never merge bodies or imply a winner, and `show` continues to refuse the ambiguous coordinate ([§FS-show.2.2](FS-show.md#22-section)).

### 2.8 What it is not

`list` does not print declaration *bodies* (that is `grund <ID>`), and it does not list *citations* (that is `grund refs <ID>`). It does not modify anything. An ID that is cited but never declared does **not** appear in `list` — it has no declaration to catalog; `grund refs <ID>` and `grund check` are where a dangling citation surfaces.

## 3. Outputs

### 3.1 `--format text` (default)

One line per catalog entry on **stdout** (this is a result a caller consumes and pipes, like `grund <ID>` / `grund id` / `grund config show`, not report output):

```
$ grund list
AR-event-bus    src/bus.rs:14                 In-process event broadcaster
FS-check        docs/functional-spec/FS-check.md:1    grund validates every citation in a repo
FS-login        docs/functional-spec/FS-login.md:1    A player can log in with email
G-no-dangling-refs  docs/goals.md:7     every cited ID resolves to a declaration
```

The columns are: the ID (rendered in the repo's `[id] format`, or in its kind's own `format` where the kind sets one ([§FS-config.3.4.10.1](FS-config.md#34101-format)); left-padded so the column aligns — capped so one very long ID does not blow out the table), then `<path>:<line>` of the home declaration (for a collapsed stub-and-inline pair, the source file the body is in), then the title — the heading text the author wrote after `<ID>:`. A heading with no `: <text>` tail leaves the title column empty, a broken stub shows `→ <target>` in its place, a duplicated ID's lines carry a duplicate note, and a row with marked roots appends a suffix naming them in canonical order ([§FS-list.3.1.1](FS-list.md#311-row-notes)). `--kind` and `--unused` select the lines as [§FS-list.1.1](FS-list.md#11---kind) and [§FS-list.1.3](FS-list.md#13---unused) say. An empty catalog (or an empty filter result) prints nothing — that is not an error.

Stderr is empty on success.

#### 3.1.1 Row notes

A declaration with no title has an empty title column: one whose heading carries no `: <text>` tail, and an E2E case, which no heading declares ([§FS-declarations.e2e-case.id](FS-declarations.md#e2e-caseid-a-case-directory-and-the-id-it-declares)). A broken stub shows `→ <target>` in place of a title. A duplicated ID's lines carry a `(duplicate declaration — grund check)` note. A row with value roots — marked ([§FS-values.2.4](FS-values.md#24-embedded-section-value-roots)) or chapter-declared ([§FS-values.2.5](FS-values.md#25-chapter-declared-value-roots)) — appends ` [value roots: <ID.path>, <ID.path> (invalid)]`, ordered by canonical section path and not by origin; a row without them is byte-identical to its prior form.

### 3.2 `--format json`

NDJSON on stdout — one object per catalog entry, same order as the text form:

```json
{"id":"AR-event-bus","kind":"AR","path":"src/bus.rs","line":14,"title":"In-process event broadcaster","stub":false,"defines":null,"refs":3,"duplicate":false}
{"id":"FS-login","kind":"FS","path":"docs/functional-spec/FS-login.md","line":1,"title":"A player can log in with email","stub":false,"defines":null,"refs":7,"duplicate":false}
```

Fields: `id` (rendered ID), `kind`, `path` and `line` of the home declaration, `title` (`null` when the declaration has no title — a heading with no title tail, or an E2E case ([§FS-declarations.e2e-case.id](FS-declarations.md#e2e-caseid-a-case-directory-and-the-id-it-declares)) — or when the home is a broken stub), `stub` (true when this entry's home is a stub heading — only ever true for a *broken* stub, since a healthy one collapses into its source declaration), `defines` (the `<target>` of a stub heading, else `null`), `refs` (the count of recognised citations of this ID — exactly the number `grund refs` would list, and not the `--unused` predicate; [§FS-list.3.2.1](FS-list.md#321-refs-counts-citations)), and `duplicate` (true when the ID has more than one home). A row with marked roots adds a `value_roots` member in canonical section-path order, omitted when there are none ([§FS-list.3.2.2](FS-list.md#322-value_roots)).

#### 3.2.1 `refs` counts citations

`refs` is the count of recognised citations of this ID across the scanned tree — exactly the number `grund refs` would list, carried on every entry so a tool need not run `grund refs` per ID to learn it. It is a *count of citations*, not the `--unused` predicate: where a kind's index is checked ([§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)) that count includes the ID's index entry, which `--unused` discounts ([§FS-list.1.3](FS-list.md#13---unused)), so an entry selected by `--unused` may carry `refs: 1`. A tool filtering for uncited IDs should pass `--unused` and read the rows, not threshold on `refs`.

#### 3.2.2 `value_roots`

A row with value roots of either origin adds `"value_roots":[{"id":"FS-pricing.2","valid":true},{"id":"FS-pricing.4","valid":false}]` in canonical section-path order. The member records no origin. It is omitted, rather than emitted as an empty array, when the declaration has no value roots. No root changes `refs`, creates a row, or changes `--summary`. The additive conditional member and all existing wire fields are stable per [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path).

### 3.3 `--summary`

`grund list --summary` prints one line per kind — in the configured `[[kinds]]` order, for the kinds that have at least one declaration in scope:

```
$ grund list --summary
AR    7    docs/architecture
DA    3    docs/decisions/architectural
DF    9    docs/decisions/functional
FS   18    requirements.md
…
```

Columns: the kind, the count of declarations of that kind the scan found (after `--kind` / `--unused` narrowing, if any: `--kind` keeps only the selected kinds' rows and `--unused` counts only uncited declarations, [§FS-list.3.3.1](FS-list.md#331-narrowed-summaries)), and that kind's configured home (`file` for single-file kinds, otherwise `folder`; [§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)) — so one line tells an agent both how big each slice of the catalog is and where to look. A kind with zero declarations in scope is omitted, which is every non-citable kind by construction; an empty result (every kind empty, or `--kind` narrowed to kinds with no declarations) prints nothing — not an error. `--format json` together with `--summary`: NDJSON, one object per kind, `{"kind":<kind>,"title":<[[kinds]] title>,"home":<file-or-folder>,"count":<n>}`, same order. Exit codes ([§FS-list.4](FS-list.md#4-exit-codes)) are unchanged.

#### 3.3.1 Narrowed summaries

With `--kind FS,AR --summary` only those rows appear; with `--unused --summary` the counts are of uncited declarations — the same set the per-declaration `--unused` lists, under [§FS-list.1.3.1](FS-list.md#131-a-configured-e2e-kind-is-left-out-unless-selected)'s `E2E` rule.

### 3.4 `--size` — per-coordinate lead and full-body measurements

Size mode emits one row for each declaration site and each citable section site. A healthy stub collapses onto its inline home as in [§FS-list.2.5](FS-list.md#25-inline-homes-stay-canonical), wherever that home's file sits ([§FS-list.3.4.6](FS-list.md#346-a-home-outside-the-scan)). A broken stub remains a row at the stub site, with no measurement. Independent duplicate declaration homes and duplicate section claimants remain separate, marked site-local rows ([§FS-list.2.6](FS-list.md#26-duplicate-declarations), [§FS-list.2.7](FS-list.md#27-duplicate-sections)).

Each row measures exactly the lead and the full body `show` returns for that site ([§FS-list.3.4.1](FS-list.md#341-measured-strings)), in `lines`, `words` and `bytes` counted over its UTF-8 bytes with no locale or Unicode-table input ([§FS-list.3.4.2](FS-list.md#342-units)). Text rows are headerless `<coordinate>  <path>:<line>  <unit>=<lead>/<full>` lines ([§FS-list.3.4.3](FS-list.md#343-text-rows)); JSON rows are NDJSON in a fixed field order, without the catalog's `title` and `refs` ([§FS-list.3.4.4](FS-list.md#344-json-rows)). Rows come in normal catalog order, and `--top` keeps the largest leads in the first selected unit after every filter ([§FS-list.3.4.5](FS-list.md#345-order-and---top)).

#### 3.4.1 Measured strings

The measured strings are exactly the text bodies returned for that site by the corresponding default and `--full` show modes ([§FS-show.2.1](FS-show.md#21-whole-declaration-default), [§FS-show.2.3](FS-show.md#23-source-declarations-in-code-and-doc-comments)): a declaration excludes its declaration heading; a section includes its section heading as `show` prints it, so a selected relative heading is measured with its canonical path ([§FS-declarations.section.relative.shown.selected](FS-declarations.md#sectionrelativeshownselected-a-selected-sections-own-heading-names-its-path)), which adds bytes but no words or lines; a default read stops before the first child heading; `--full` includes descendants; doc-comment prefixes are stripped; cross-reference wrappers are flattened in ordinary prose and preserved inside Markdown fences ([§FS-show.3.2](FS-show.md#32-cross-reference-links-are-flattened-in-text-and-json)); and an empty lead is the empty string. Value-source declaration and section rows measure their exact available source slice, for which lead equals full. Preserving a fenced destination can increase byte and word counts and can therefore cross the opt-in [§FS-declarations.checks.oversized-lead](FS-declarations.md#checksoversized-lead-oversized-lead-opt-in) warning threshold; it does not change line counts. A selected relative heading's canonical path can cross a byte threshold of that warning the same way; a word threshold does not move.

#### 3.4.2 Units

Every unit is defined over the UTF-8 bytes of that measured string, with no locale or Unicode-table input:

- `lines` is the count of non-blank LF-delimited segments, including a final segment without a terminating LF. A segment is blank only when it contains ASCII whitespace bytes (`09`–`0D`, `20`).
- `words` is the count of maximal non-empty byte runs separated by those same ASCII whitespace bytes.
- `bytes` is the UTF-8 byte length.

#### 3.4.3 Text rows

Text output is headerless and line-oriented:

```text
<coordinate>  <path>:<line>  <unit>=<lead>/<full> [<unit>=<lead>/<full> ...]
```

The coordinate and location use the ordinary list padding and relative-path rules. A duplicate row ends in `  (duplicate — site-local)`. An unmeasurable broken stub instead renders every selected pair as `-/-` and ends in `  (broken stub → <target>)`.

#### 3.4.4 JSON rows

JSON output is NDJSON with fields in this order: optional workspace `project`, then `id`, `section`, `kind`, `path`, `line`, `stub`, `defines`, `duplicate`, followed by `lead_<unit>` and `full_<unit>` pairs in requested unit order. `id` is the rendered declaration ID, workspace-qualified when applicable; `section` is `null` for its declaration row or the exact section path; unmeasurable values are `null`; unselected unit fields are absent. Size rows deliberately omit the ordinary catalog's `title` and `refs` fields ([§FS-output-shapes.5](FS-output-shapes.md#5-list---formatjson)).

#### 3.4.5 Order and `--top`

Normal row order is existing workspace/project order and parsed-ID order, then the declaration before its sections, byte-sorted section path, then byte-sorted path and line to distinguish sites. `--top` is applied after `--project`, `--kind`, and `--unused`; `--unused` selects declarations by its ordinary rule and retains all section rows belonging to those declarations. Top mode sorts descending by the lead value of the first requested unit and uses normal row order as the total tie-break. It omits rows without measurements; fewer than `N` measurable rows returns all of them. An empty filtered result writes nothing and succeeds.

#### 3.4.6 A home outside the scan

A healthy stub collapses onto its inline home even where the home's file lies outside `[scan] include`. Size mode finds that home as `show` does, by the stub's ID in the target file, whether or not the walk reaches it ([§FS-show.2.3.7](FS-show.md#237-a-stubs-target-is-found-by-its-id)). Where that read finds the ID's one home, the rows are the ones the same tree prints with the target scanned: the declaration row at the home's `<path>:<line>`, then a row for each of the home's sections, every one measured, and in JSON `stub` is `false` and `defines` is `null` ([§FS-list.3.4.4](FS-list.md#344-json-rows)). Where that read finds the ID twice, the rows are again the scanned tree's: one declaration row at each of the target's declarations, each measured and each marked duplicate, the two declarations `check` reports as a duplicate ([§FS-declarations.stubs.pairing.1](FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned)), never one unmeasured row at the stub. No such row is labelled `broken stub`: the unmeasured stub-site row of [§FS-list.3.4.3](FS-list.md#343-text-rows) is the row of exactly the stubs `check` reports broken, read off the verdict every command reads ([§FS-declarations.stubs.verdict](FS-declarations.md#stubsverdict-one-verdict-per-stub-reached-once-per-scan)).

Measuring a home does not bring its file into the scan. `check`'s opt-in lead warning measures leads by this mode but still judges only sites in the configured scan scope ([§FS-declarations.checks.oversized-lead.4](FS-declarations.md#checksoversized-lead4-which-sites-it-judges)), so a home outside `[scan] include` draws no size warning.

## total: `--total`, a count rung

`--total` prints one count in place of the rows: the number of rows `list` would print under
the same path, `--selector`, `--glob`, `--kind`, `--project` and `--unused`, with or without a
pattern. Text is the bare number and a newline, JSON is `{"count":<n>}` and a newline
([§FS-output-shapes.5](FS-output-shapes.md#5-list---formatjson)), and an empty selection prints `0`, so a caller compares it with `wc -l` of
the rows. It is the cheap rung before the bodies: count a checklist, then read it
([§GOAL-token-economy.1](../goals.md#1-what-this-requires)). It is not `refs --total`'s sentence ([§FS-refs.3.4](FS-refs.md#34---total)): that is the hover's
words for a site and file pair, and a listing has neither. A count beside a fold or a
measurement asks two questions, so `--total` with `--summary`, `--size` or `--top` is a launch
error, decided before config discovery and before the size-selector errors, in that order:
`--total cannot be combined with --summary`, `--total cannot be combined with --size`,
`--total cannot be combined with --top` ([§FS-list.1.7](FS-list.md#17-size-selector-errors), [§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)). Repeating it is harmless.

## 4. Exit codes

- `0` — the scan succeeded; the listed catalog (possibly empty) is the result.
- `2` — scan / I/O error ([§FS-check.2](FS-check.md#2-outputs) partial-scan semantics apply: an incomplete scan exits `2` and the catalog may be short), an unknown `--kind` (any value in a comma-separated or repeated `--kind`), an invalid size selector or combination ([§FS-list.1.7](FS-list.md#17-size-selector-errors)), a refused coordinate glob ([§FS-coordinate-globs.errors](FS-coordinate-globs.md#errors-refusals-notes-and-hints)), an unsupported `--format`, or any other CLI-level error ([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)).

There is no `1`: `list` is a query that always returns *its* answer (a possibly-empty catalog), never "found something other than one body" — unlike `show`, it has no single-result expectation to violate.

## 5. Why this exists

`grep -RhoE '^#+ [A-Z]+-[a-z0-9-]+'` across `docs/` gives a contributor a rough list of declaration headings but cannot: reach source declarations inside doc-comments; collapse a stub onto the source declaration it points at; honour the configured `[id]` grammar in a repo that customised it; tell which declarations are uncited; or produce a stable, machine-shaped result an agent can program against. `list` is the scheme's own answer, sharing the scanner with `check` so the catalog and the validator never disagree on what a declaration is. Multi-kind `--kind FS,AR` and `--summary` give an agent a scoped slice or a bird's-eye count of the catalog instead of the full dump — the discovery half of token-cheap grounding. With `show` and `refs` it completes the read surface: `list` enumerates the IDs, `show` reads the body one promises, `refs` enumerates who took the promise.
