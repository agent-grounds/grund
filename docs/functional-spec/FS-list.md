# FS-list: grund lists every declared ID

The `list` subcommand prints the repo's ID catalog: every declaration, where it lives, and its one-line title — or, narrowed with `--kind`/`--summary`, just the slice an agent asked for. It is the index that `grund <ID>` reads from and the broad counterpart of `grund refs` — `refs` answers "who cites *this* ID?", `list` answers "what IDs are there?". An agent that has been told to ground itself with `grund <ID>` needs a way to discover the `<ID>`s; a human auditing a spec tree needs the same. Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible) (no `grep` for `^# [A-Z]+-` across the tree), [§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file), and the agent-grounding loop in [§GRUND-grund](../grund.md#grund-grund-agents-stay-grounded-in-the-spec).

## 1. Inputs

```
grund list [<path>] [--kind <KIND>[,<KIND>…]]… [--project <alias>[,<alias>...]] [--unused] [--summary]
           [--size[=<unit>[,<unit>…]]] [--top <N>] [--format text|json]
```

`<path>` is the directory or file whose tree is scanned, `.` by default. Discovery is the same as every other subcommand (walk up to a `grund.toml`, else defaults — [§FS-config.1](FS-config.md#1-file-location-and-discovery)). `--format text|json` picks the output shape (§3), `text` by default.

`--kind` selects configured citable kinds and refuses an unknown one, or a non-citable one with the two-line message giving its reason (§1.1). `--project` narrows a workspace catalog to the named projects (§1.2). `--unused` keeps only uncited declarations, discounting a kind's index entries and leaving out `E2E` cases unless `--kind` selects them (§1.3). `--summary` prints one count per kind under those same filters (§1.4). `--size` and `--top` switch to measured point-size rows over a closed unit set (§1.5, §1.6); their syntax and combination errors are launch errors, decided before config discovery or scanning (§1.7).

`list` is a query, like `show` and `refs` — non-interactive, no prompts ([§FS-non-goals.10](FS-non-goals.md#10-interactive-mode)).

### 1.1 `--kind`

`--kind <KIND>[,<KIND>…]` lists only declarations of the named kinds, each a configured *citable* `[[kinds]]` entry ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)). It accepts a comma-separated list (`--kind FS,AR`) and may be repeated (`--kind FS --kind AR`); the selections union.

An unknown kind *anywhere* in the selection is a CLI-level error (§4): a typo'd `--kind` must not silently produce an empty — or merely short — catalog. A configured **non-citable** kind ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)) is refused the same way and for the same reason — it would select nothing, every time — but with the reason instead of "unknown", because it is a real row in the table rather than a typo:

```text
error: kind `skill` declares no IDs — skills/ is not a citable home
known kinds: GRUND, GOAL, FS, AR, DF, DA, RM
```

The `known kinds:` line lists the citable kinds only: they are the whole set this selector accepts.

### 1.2 `--project`

`--project <alias>[,<alias>...]` lists, in workspace mode, only declarations from the named projects, and composes with `--kind` by intersection. Outside workspace mode it is a CLI-level error (exit `2`); an unknown alias is also a CLI-level error (exit `2`), the same shape as an unknown `--kind` (§4). Workspace qualification, size behavior, and member-local invocation are specified in [§FS-workspace.8.3](FS-workspace.md#83-grund-list).

### 1.3 `--unused`

`--unused` lists only declarations that no recognised citation points at — the same set `check` warns on ([§FS-check.4.1](FS-check.md#41-unused-declaration)) — **excluding `E2E` cases unless `E2E` is explicitly selected with `--kind`**. An e2e case is a proof artifact, exercised by being run, not a citation target, so it is uncited by construction and would only ever bury the actionable signal (uncited specs, decisions, goals) in a bare `--unused` query. `--unused --kind E2E` lists uncited cases only, while `--unused --kind FS,E2E` lists uncited `FS` declarations plus uncited `E2E` cases because `E2E` was explicitly requested.

A citation that is a kind's own index entry ([§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)) is discounted here too, for the reason `check` discounts it ([§FS-check.4.1](FS-check.md#41-unused-declaration)).

### 1.4 `--summary`

`--summary` prints one line per kind instead of one per declaration, narrowed by `--kind` and `--unused` as the per-declaration form is; §3.3 gives its shape.

### 1.5 `--size`

`--size[=<unit>[,<unit>…]]` switches from declaration rows to the point-size rows in §3.4. The optional value is accepted only in the same argument with `=`; a following bare word remains the existing `<path>` positional. Bare `--size` selects `lines,words,bytes`, in that order. An explicit list is case-sensitive, preserves caller order, and collapses repeated units to their first occurrence. The closed unit set is `lines`, `words`, and `bytes`: an empty item or any other value, including `tokens`, is a CLI-level error, and there is no recognized or reserved token-counting spelling. The flag may appear once and cannot be combined with `--summary`.

### 1.6 `--top`

`--top <N>` / `--top=<N>` retains at most the `N` rows with the largest lead in the first selected size unit, after every ordinary filter (§3.4.5). `N` is one positive base-10 integer. The flag may appear once, requires `--size`, and cannot be combined with `--summary`.

### 1.7 Size-selector errors

Size-selector syntax and combination errors are validated before config discovery or scanning. They write one raw `error:` line to stderr and exit `2` (§4): `--size may only appear once`; `--size requires one or more units`; `unknown size unit \`<unit>\` (expected lines, words, or bytes)`; `--top may only appear once`; `--top requires a positive integer`; `--top requires --size`; `--size cannot be combined with --summary`; or `--top cannot be combined with --summary`.

## 2. Behaviour

`list` runs the same scan as `check` ([AR-scanner](../architecture/AR-scanner.md#ar-scanner-how-grund-discovers-declarations-and-citations)) and emits, for every declaration the scan found, one catalog line. In size mode it additionally emits every numbered or named section the same scan makes citable. The set of declarations and sections is exactly the set `check` validates and `show` can attempt to resolve, so the three never disagree on what exists. This includes Markdown, source-code doc-comment and inline declarations, JSON values and their sections, and configured `E2E` case declarations.

In that catalog, JSON entries from an opted-in kind home are declarations (§2.1), a committed fetched snapshot is an ordinary declaration (§2.2), and a retained off-grammar declaration is a normal catalog row (§2.3). Declarations come out sorted by ID (§2.4). A stub-and-inline pair shows one line, at the source file holding the body (§2.5). An ID declared in several homes keeps one flagged line per home, and in size mode each claimant of a duplicated section keeps its own row (§2.6, §2.7). `list` prints no bodies or citations, modifies nothing, and never lists an ID that is cited but not declared (§2.8).

### 2.1 JSON values

JSON entries from an opted-in kind home are declarations in the same catalog. `list` shows their ID and exact location without inventing a title; duplicate and unused filters apply as they do to Markdown values. A marked section value remains metadata on its enclosing declaration row, never a synthetic declaration or second row ([§FS-values.6](FS-values.md#6-shared-catalog-consumers)).

### 2.2 Per-kind formats and fetched snapshots

Per-kind formats ([§FS-config.3.4.10](FS-config.md#3410-format-resolve-and-fetch--external-snapshot-kinds)) are parsed through that shared catalog. A committed fetched snapshot appears as an ordinary declaration; a missing snapshot does not appear because there is no declaration to list. Listing never executes a fetcher.

### 2.3 Off-grammar declarations

An exact off-grammar declaration retained under [§FS-config.3.2](FS-config.md#32-id--id-grammar) is a normal catalog row, rendered exactly as written with its body-independent metadata, sections, reference count, duplicate flag, and stable sort position. Text and JSON keep their existing schemas. The row cannot disappear merely because `check` also reports its conformance mismatch ([§FS-check.4.6](FS-check.md#46-declaration-near-miss)).

### 2.4 Order

Declarations come out sorted by ID — kind, then number, then slug. The result is deterministic for a given tree.

### 2.5 Inline homes stay canonical

When an ID's home is an inline declaration in source code with a one-line stub under `docs/architecture/` pointing at it (the [§FS-check.3.4](FS-check.md#34-broken-inline-spec-stub) / [§FS-show.2.3](FS-show.md#23-inline-declarations-in-code-and-doc-comments) arrangement), `list` shows **one** line for that ID, naming the source file where the body lives. An external inline declaration enrolled directly by its kind's index ([§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)) likewise appears once at the source home: the index link creates no declaration to collapse. A *broken* stub (its target missing, or the target has no matching inline declaration) is not paired with anything, so it does appear, listed at the stub's own location with a `→ <target>` note; `check` reports the breakage in located form.

### 2.6 Duplicate declarations

When an ID is declared in more than one independent home — the [§FS-check.3.3](FS-check.md#33-duplicate-declaration) error — `list` prints one line per home, each flagged so the duplication is visible at a glance. `list` does not pick a winner; it shows the situation and leaves the located error to `check`.

### 2.7 Duplicate sections

Size mode likewise prints one row per site that claims a duplicated section coordinate and marks each row duplicate. Declaration and section duplicates are measured from that site only: rows never merge bodies or imply a winner, and `show` continues to refuse the ambiguous coordinate ([§FS-show.2.2](FS-show.md#22-section)).

### 2.8 What it is not

`list` does not print declaration *bodies* (that is `grund <ID>`), and it does not list *citations* (that is `grund refs <ID>`). It does not modify anything. An ID that is cited but never declared does **not** appear in `list` — it has no declaration to catalog; `grund refs <ID>` and `grund check` are where a dangling citation surfaces.

## 3. Outputs

### 3.1 `--format text` (default)

One line per catalog entry on **stdout** (this is a result a caller consumes and pipes, like `grund <ID>` / `grund id` / `grund config show`, not diagnostic output):

```
$ grund list
AR-event-bus    src/bus.rs:14                 In-process event broadcaster
FS-check        docs/functional-spec/FS-check.md:1    grund validates every reference in a repo
FS-login        docs/functional-spec/FS-login.md:1    A player can log in with email
G-no-dangling-refs  docs/goals.md:7     every cited ID resolves to a declaration
```

The columns are: the ID (rendered in the repo's `[id] format`, left-padded so the column aligns — capped so one very long ID does not blow out the table), then `<path>:<line>` of the home declaration (for a collapsed stub-and-inline pair, the source file the body is in), then the title — the heading text the author wrote after `<ID>:`. A heading with no `: <text>` tail leaves the title column empty, a broken stub shows `→ <target>` in its place, a duplicated ID's lines carry a duplicate note, and a row with embedded value roots appends a suffix naming them in canonical order (§3.1.1). `--kind` and `--unused` select the lines as §1.1 and §1.3 say. An empty catalog (or an empty filter result) prints nothing — that is not an error.

Stderr is empty on success.

#### 3.1.1 Row notes

A declaration whose heading carries no `: <text>` tail has an empty title column. A broken stub shows `→ <target>` in place of a title. A duplicated ID's lines carry a `(duplicate declaration — grund check)` note. A row with embedded value roots appends ` [value roots: <ID.path>, <ID.path> (invalid)]`, ordered by canonical section path; a row without them is byte-identical to its prior form.

### 3.2 `--format json`

NDJSON on stdout — one object per catalog entry, same order as the text form:

```json
{"id":"AR-event-bus","kind":"AR","path":"src/bus.rs","line":14,"title":"In-process event broadcaster","stub":false,"defines":null,"refs":3,"duplicate":false}
{"id":"FS-login","kind":"FS","path":"docs/functional-spec/FS-login.md","line":1,"title":"A player can log in with email","stub":false,"defines":null,"refs":7,"duplicate":false}
```

Fields: `id` (rendered ID), `kind`, `path` and `line` of the home declaration, `title` (`null` when the heading has no title tail or the home is a broken stub), `stub` (true when this entry's home is a stub heading — only ever true for a *broken* stub, since a healthy one collapses into its inline declaration), `defines` (the `<target>` of a stub heading, else `null`), `refs` (the count of recognised citations of this ID — exactly the number `grund refs` would list, and not the `--unused` predicate; §3.2.1), and `duplicate` (true when the ID has more than one home). A row with embedded value roots adds a `value_roots` member in canonical section-path order, omitted when there are none (§3.2.2).

#### 3.2.1 `refs` counts citations

`refs` is the count of recognised citations of this ID across the scanned tree — exactly the number `grund refs` would list, carried on every entry so a tool need not run `grund refs` per ID to learn it. It is a *count of citations*, not the `--unused` predicate: where a kind's index is checked ([§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)) that count includes the ID's index entry, which `--unused` discounts (§1.3), so an entry selected by `--unused` may carry `refs: 1`. A tool filtering for uncited IDs should pass `--unused` and read the rows, not threshold on `refs`.

#### 3.2.2 `value_roots`

A row with embedded roots adds `"value_roots":[{"id":"FS-pricing.2","valid":true},{"id":"FS-pricing.4","valid":false}]` in canonical section-path order. This member is omitted, rather than emitted as an empty array, when the declaration has no embedded roots. No root changes `refs`, creates a row, or changes `--summary`. The additive conditional member and all existing wire fields are stable per [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path).

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

Columns: the kind prefix, the count of declarations of that kind the scan found (after `--kind` / `--unused` narrowing, if any: `--kind` keeps only the selected kinds' rows and `--unused` counts only uncited declarations, §3.3.1), and that kind's configured home (`file` for single-file kinds, otherwise `folder`; [§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)) — so one line tells an agent both how big each slice of the catalog is and where to look. A kind with zero declarations in scope is omitted, which is every non-citable kind by construction; an empty result (every kind empty, or `--kind` narrowed to kinds with no declarations) prints nothing — not an error. `--format json` together with `--summary`: NDJSON, one object per kind, `{"kind":<prefix>,"title":<[[kinds]] title>,"home":<file-or-folder>,"count":<n>}`, same order. Exit codes (§4) are unchanged.

#### 3.3.1 Narrowed summaries

With `--kind FS,AR --summary` only those rows appear; with `--unused --summary` the counts are of uncited declarations — the same set the per-declaration `--unused` lists, under §1.3's `E2E` rule.

### 3.4 `--size` — per-point lead and full-body measurements

Size mode emits one row for each declaration site and each citable section site. A healthy stub collapses onto its inline home as in §2.5. A broken stub remains a row at the stub site, with no measurement. Independent duplicate declaration homes and duplicate section claimants remain separate, marked site-local rows (§2.6, §2.7).

Each row measures exactly the lead and the full body `show` returns for that site (§3.4.1), in `lines`, `words` and `bytes` counted over its UTF-8 bytes with no locale or Unicode-table input (§3.4.2). Text rows are headerless `<coordinate>  <path>:<line>  <unit>=<lead>/<full>` lines (§3.4.3); JSON rows are NDJSON in a fixed field order, without the catalog's `title` and `refs` (§3.4.4). Rows come in normal catalog order, and `--top` keeps the largest leads in the first selected unit after every filter (§3.4.5).

#### 3.4.1 Measured strings

The measured strings are exactly the text bodies returned for that site by the corresponding default and `--full` show modes ([§FS-show.2.1](FS-show.md#21-whole-declaration-default), [§FS-show.2.3](FS-show.md#23-inline-declarations-in-code-and-doc-comments)): a declaration excludes its declaration heading; a section includes its section heading; a default read stops before the first child heading; `--full` includes descendants; doc-comment markers are stripped; cross-reference wrappers are flattened; and an empty lead is the empty string. JSON declaration and section rows measure their exact available source slice, for which lead equals full.

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

## 4. Exit codes

- `0` — the scan succeeded; the listed catalog (possibly empty) is the result.
- `2` — scan / I/O error ([§FS-check.2](FS-check.md#2-outputs) partial-scan semantics apply: an incomplete scan exits `2` and the catalog may be short), an unknown `--kind` (any value in a comma-separated or repeated `--kind`), an invalid size selector or combination (§1.7), an unsupported `--format`, or any other CLI-level error ([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)).

There is no `1`: `list` is a query that always returns *its* answer (a possibly-empty catalog), never "found something other than one body" — unlike `show`, it has no single-result expectation to violate.

## 5. Why this exists

`grep -RhoE '^#+ [A-Z]+-[a-z0-9-]+'` across `docs/` gives a contributor a rough list of declaration headings but cannot: reach inline declarations inside source-code doc-comments; collapse a stub onto the inline declaration it points at; honour the configured `[id]` grammar in a repo that customised it; tell which declarations are uncited; or produce a stable, machine-shaped result an agent can program against. `list` is the scheme's own answer, sharing the scanner with `check` so the catalog and the validator never disagree on what a declaration is. Multi-kind `--kind FS,AR` and `--summary` give an agent a scoped slice or a bird's-eye count of the catalog instead of the full dump — the discovery half of token-cheap grounding. With `show` and `refs` it completes the read surface: `list` enumerates the IDs, `show` reads the body one promises, `refs` enumerates who took the promise.
