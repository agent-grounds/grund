# FS-cover: grund groups citations by scanned file

The `cover` subcommand exposes the citation graph as data: for each scanned file, which spec IDs does it cite, and where? This is the plumbing surface for the diff-aware co-change recipe ([§RM-cochange-gate](../roadmap.md#rm-cochange-gate-a-pre-commit--ci-recipe--no-impl-change-without-spec-and-test)): git decides what changed, `cover` says which IDs the changed files lean on. Serves [§GOAL-agent-grounding.1](../goals.md#1-the-three-layers) and keeps the policy layer out of `grund-core`.

## 1. Inputs

```
grund cover [<path>] [--format text|json]
```

- `<path>` — directory or file whose tree is scanned. Defaults to `.`. Discovery is the same as every other subcommand (walk up to a `grund.toml`, else defaults — [§FS-config.1](FS-config.md#1-file-location-and-discovery)). It bounds the **walk**, exactly as it does for `grund check`, and that is what decides the scope: every project at a workspace root, that member alone inside a member, that subtree alone below a config root ([§FS-workspace.8.6](FS-workspace.md#86-grund-cover)).
- `--format text|json` — output shape (§3). Default `text`. An unsupported value is answered before anything is loaded (§1.1).

`cover` is a query, like `list` and `refs` — non-interactive, no prompts ([§FS-non-goals.10](FS-non-goals.md#10-interactive-mode)). It reads no git history ([§FS-non-goals.6](FS-non-goals.md#6-decision-database-audit-log-history-tracking)) and parses no AST ([§FS-non-goals.3](FS-non-goals.md#3-code-ast-parsing)).

### 1.1 An unsupported `--format` is answered before the load

A `--format` value outside `text|json` is a usage error the caller can fix without touching the repository, so it is answered **before anything is loaded**: the scan can fail first (§4), and which of two errors a caller sees must not depend on the tree they happened to point at. A `[output] format` key carrying an unsupported value is a property of the tree, so it is reported after the load, like any other config fault.

## 2. Behaviour

`cover` runs the same scan as `check`, `list`, and `refs` ([AR-scanner](../architecture/AR-scanner.md#ar-scanner-how-grund-discovers-declarations-and-citations)) and only renders the `Findings` the scanner already collected. It does not decide whether a file is sufficiently covered, whether a hunk is behavioral, or whether a spec/test co-change is required; those are recipe concerns ([§RM-cochange-gate](../roadmap.md#rm-cochange-gate-a-pre-commit--ci-recipe--no-impl-change-without-spec-and-test)).

Output is grouped by scanned file, sorted by path. Within a file, citations are sorted by `(line, column)`. Files with no recognised citations are still included, so a caller can distinguish "the file was scanned and cites nothing" from "the file was outside the scan scope." A citation object is the same shape `grund refs --format=json` emits (§3.2): path, line, column, rendered ID, optional section, marker boolean, and the verbatim token text.

Which citations count is §2.1 for a cross-project one, §2.2 for a value binding's, §2.3 for one whose fetched snapshot is missing, and §2.4 for a declaration-backed exact one.

### 2.1 A cross-project citation counts

Every citation the scanner recognised in the file counts, **including a cross-project `<§><alias>/<ID>`**, and its rendered `id` keeps the alias it was written with and adds none of its own. `cover` does not judge whether the alias resolves; an unknown one is a `check` error, not a reason to drop the row. The rules and their reasons are [§FS-workspace.8.6.3](FS-workspace.md#863-qualified-citations-count-toward-the-citing-file) and [§FS-workspace.8.6.4](FS-workspace.md#864-the-rendered-id-says-what-the-token-says), decided in [§DF-cover-workspace-scope](../decisions/functional/DF-cover-workspace-scope.md#df-cover-workspace-scope-cover-indexes-the-whole-run-and-counts-cross-project-citations).

### 2.2 A value binding's citation

A recognized value binding contributes its ordinary citation to the citing scanned file. Home JSON is catalog input, not a scanned citing file, and therefore adds no file or citation row ([§FS-values.2.2](FS-values.md#22-json-declarations-from-the-kind-home), [§FS-values.3.2](FS-values.md#32-recognized-text-contexts)).

### 2.3 A citation whose fetched snapshot is missing

A citation parsed under a per-kind format is included even while its fetched
snapshot is missing. Fetching changes resolution, not whether the citing file
is covered. `cover` remains offline and never invokes the integration.

### 2.4 A declaration-backed exact citation

A declaration-backed exact marked citation retained through [§FS-config.3.2](FS-config.md#32-id--id-grammar) is
counted at its written site and rendered with its exact ID. `cover`
consumes the shared scanner result and adds no fallback grammar of its own.

## 3. Outputs

### 3.1 `--format text` (default)

Text output goes to stdout. It prints a heading line per scanned file, followed by either the citation line/column and token, or `(no citations)`:

```
$ grund cover src/
src/login.rs:
  14:5 §FS-login.2
  28:9 §DF-password-policy
src/untouched.rs:
  (no citations)
```

### 3.2 `--format json`

NDJSON on stdout — one object per scanned file:

```json
{"path":"src/login.rs","citations":[{"path":"src/login.rs","line":14,"column":5,"id":"FS-login","section":"2","marker":true,"text":"§FS-login.2"},{"path":"src/login.rs","line":28,"column":9,"id":"DF-password-policy","section":null,"marker":true,"text":"§DF-password-policy"}]}
{"path":"src/untouched.rs","citations":[]}
```

The nested citation objects intentionally carry `path` too, matching `refs` JSON byte shape so a caller can compare `cover` and `refs` without a field mapping layer. The parity is of **fields** — same names, same order, same types — not of every value: `refs` renders `id` for the target its query named, so an `<§>api/FS-login` site reads `"id":"FS-login"` there and `"id":"api/FS-login"` here. `refs` was handed the alias in the argument; `cover` was not, and dropping it would leave the row unable to say what it points at, since `project` names the *citing* project.

In workspace mode both the per-file object and each nested citation object gain a leading `"project":"<alias>"` — the project that contains the file, which is also the citing project — exactly as `refs` does ([§FS-workspace.8.2](FS-workspace.md#82-grund-refs), [§FS-workspace.8.6](FS-workspace.md#86-grund-cover)). Outside workspace mode the field is absent and the bytes above are unchanged:

```json
{"project":"api","path":"apps/api/src/login.rs","citations":[{"project":"api","path":"apps/api/src/login.rs","line":14,"column":5,"id":"FS-login","section":"2","marker":true,"text":"§FS-login.2"}]}
```

## 4. Exit codes

- `0` — scan succeeded; the emitted file records are the result.
- `2` — the run could not read the tree it was asked about, in either of two ways:
  - **A config the run needs does not load** — including a `[workspace]` block whose members cannot be expanded (a duplicate or invalid alias, a missing member, a block with nothing in scope). `cover` indexes the whole workspace ([§FS-workspace.8.6](FS-workspace.md#86-grund-cover)), so it fails on such a tree exactly where `grund list` does.
  - **A scan / I/O error in any project the run loaded** ([§FS-check.2](FS-check.md#2-outputs) partial-scan semantics apply: records found before or after the unreadable file may print, but the result is not trustworthy as complete). A member's unreadable file fails the run at the workspace root, because the index the run just printed is incomplete for the tree it claimed ([§FS-workspace.8.7](FS-workspace.md#87-output-and-exit-codes)).

There is no `1`: `cover` is a query over the current tree and has no finding class of its own.

## 5. Why this exists

`grund refs <ID>` answers "who cites this ID?" and `grund list` answers "what IDs exist?". The co-change gate needs the inverse grouping: "for this changed file, what IDs does it cite?" A shell script could run `grund refs` once per ID and regroup the output, but that is slower, loses files with zero citations, and makes every recipe reconstruct scanner state. `cover` provides that view directly while preserving the no-git, no-policy boundary: git diff is an input to the recipe, not to `grund cover`.
