# FS-cover: grund groups citations by scanned file

The `cover` subcommand exposes the citation graph as data: for each scanned file, which declaration IDs does it cite, and where? This is the plumbing surface for the diff-aware co-change recipe ([§RM-cochange-gate](../roadmap.md#rm-cochange-gate-an-opt-in-commit-msg--ci-recipe-for-spec-and-test-edits)): git decides what changed, `cover` says which IDs the changed files lean on. Asked for line ranges of one file with `--lines`, it also says which declaration and section own each line ([§FS-cover.6](FS-cover.md#6-line-ownership)), so a caller holding a diff hunk can name the spec points it edits. Serves [§GOAL-agent-grounding.1](../goals.md#1-the-three-layers) and keeps the policy layer out of `grund-core`.

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, home, section, catalog), [§FS-terms.terms.2](FS-terms.md#terms2-citations)
(marker, citation), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope, config root, workspace, member, alias),
[§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding), [§FS-terms.terms.6](FS-terms.md#terms6-rules-and-directions) (direction), and [§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations) (value,
binding, snapshot).

## 1. Inputs

```
grund cover [<path>] [--format text|json]
grund cover <file> --lines <N>|<N>-<M> [--lines …] [--format text|json]
```

- `<path>` — directory or file whose tree is scanned. Defaults to `.`. Discovery is the same as every other subcommand (walk up to a `grund.toml`, else defaults — [§FS-config.1](FS-config.md#1-file-location-and-discovery)). It bounds the **scan**, exactly as it does for `grund check`, and that is what decides the scope: every project at a workspace root, that member alone inside a member, that subtree alone below a config root ([§FS-workspace.8.6](FS-workspace.md#86-grund-cover)).
- `--lines <N>|<N>-<M>` — answer line ownership for that range of `<file>` instead of grouping citations ([§FS-cover.6.1](FS-cover.md#61-input)). Repeatable.
- `--format text|json` — output shape ([§FS-cover.3](FS-cover.md#3-outputs)). Default `text`. An unsupported value is answered before anything is loaded ([§FS-cover.1.1](FS-cover.md#11-an-unsupported---format-is-answered-before-the-load)).

`cover` is a query, like `list` and `refs` — non-interactive, no prompts ([§FS-non-goals.10](FS-non-goals.md#10-interactive-mode)). It reads no git history ([§FS-non-goals.6](FS-non-goals.md#6-decision-database-audit-log-history-tracking)) and parses no AST ([§FS-non-goals.3](FS-non-goals.md#3-code-ast-parsing)).

### 1.1 An unsupported `--format` is answered before the load

A `--format` value outside `text|json` is a usage error the caller can fix without touching the repository, so it is answered **before anything is loaded**: the scan can fail first ([§FS-cover.4](FS-cover.md#4-exit-codes)), and which of two errors a caller sees must not depend on the tree they happened to point at. A `[output] format` key carrying an unsupported value is a property of the tree, so it is reported after the load, like any other config fault. Every command answers a bad run flag this way ([§FS-cli.3.5](FS-cli.md#35-a-bad-run-flag-is-answered-before-the-load)); `cover` is where the rule was first stated.

## 2. Behaviour

`cover` runs the same scan as `check`, `list`, and `refs` ([AR-scanner](../architecture/AR-scanner.md#ar-scanner-how-grund-discovers-declarations-and-citations)) and only renders the `Catalog` the scanner already collected. It does not decide whether a file is sufficiently covered, whether a hunk is behavioral, or whether a spec/test co-change is required; those are recipe concerns ([§RM-cochange-gate](../roadmap.md#rm-cochange-gate-an-opt-in-commit-msg--ci-recipe-for-spec-and-test-edits)).

An owned declaration-local numeric citation ([§FS-check.1.1.8](FS-check.md#118-declaration-local-numeric-section-candidates)) is one ordinary resolved citation in this grouping: it carries the owner's full ID and exact section while its `text` remains the authored local spelling. It therefore also counts for unused-declaration, grounding, and citation-direction questions that consume the same graph. Ownerless and unsupported local candidates contribute no guessed edge.

Output is grouped by scanned file, sorted by path. Within a file, citations are sorted by `(line, column)`. Files with no recognised citations are still included, so a caller can distinguish "the file was scanned and cites nothing" from "the file was outside the scan scope." A citation object is the same shape `grund refs --format=json` emits ([§FS-cover.3.2](FS-cover.md#32---format-json)): path, line, column, rendered ID, optional section, marker boolean, and the verbatim token text.

Which citations count is [§FS-cover.2.1](FS-cover.md#21-a-cross-project-citation-counts) for a cross-project one, [§FS-cover.2.2](FS-cover.md#22-a-value-bindings-citation) for a value binding's, [§FS-cover.2.3](FS-cover.md#23-a-citation-whose-fetched-snapshot-is-missing) for one whose fetched snapshot is missing, and [§FS-cover.2.4](FS-cover.md#24-a-declaration-backed-exact-citation) for a declaration-backed exact one.

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
{"path":"src/login.rs","citations":[{"path":"src/login.rs","line":14,"column":5,"id":"FS-login","section":"2","marker":true,"text":"§FS-login.2","enclosing_declaration":"AR-login-flow","enclosing_section":"2"},{"path":"src/login.rs","line":28,"column":9,"id":"DF-password-policy","section":null,"marker":true,"text":"§DF-password-policy","enclosing_declaration":null,"enclosing_section":null}]}
{"path":"src/untouched.rs","citations":[]}
```

The nested citation objects intentionally carry `path` too, matching `refs` JSON byte shape so a caller can compare `cover` and `refs` without a field mapping layer. The parity is of **fields** — same names, same order, same types — not of every value: `refs` renders `id` for the target its query named, so an `<§>api/FS-login` site reads `"id":"FS-login"` there and `"id":"api/FS-login"` here. `refs` was handed the alias in the argument; `cover` was not, and dropping it would leave the row unable to say what it points at, since `project` names the *citing* project.

`enclosing_declaration` and `enclosing_section` name the unit the **citing** site sits in, and they sit after `text`. They are the citing-side counterpart of `id` and `section`, which name the target: `enclosing_declaration` is the nearest preceding declaration whose body range contains the site, and `enclosing_section` the nearest accepted section path containing it ([AR-scanner.2.4.3](../architecture/AR-scanner.md#243-the-enclosing-declaration), [AR-scanner.2.4.4](../architecture/AR-scanner.md#244-the-enclosing-accepted-chapter)) — numbered and named alike, `"1"` under `## 1. Inputs` and `"terms"` under `## terms: Terms`. Both are `string | null` and both are always present: `enclosing_declaration` is `null` where the site sits in no declaration body, and `enclosing_section` is `null` where no accepted section contains it, which includes every site in a declaration's lead above its first section. The first occurrence of a duplicated path is the primary and keeps its sites, so that path still appears here; a site under the duplicate occurrence, or under a rejected path, has none. `enclosing_declaration` renders bare, under the **citing** project's `[id]` config — the declaration it names sits in the citing file, and `project` already says which project that is ([§FS-workspace.8.6](FS-workspace.md#86-grund-cover)).

Together they are the `from` half of the `cites` edge ([§FS-rules.5.1](FS-rules.md#51-facts-and-identity)): a caller reads "`FS-cover`'s `terms` section cites `FS-terms`" out of the same scan that checked it, instead of re-parsing the tree to recover a unit the scan already held — the reconstruction [§FS-cover.5](FS-cover.md#5-why-this-exists) exists to remove. `cover` and `refs` pay the citing-side classification pass for them; the other read-only queries still skip it ([AR-scanner.2.4.2](../architecture/AR-scanner.md#242-citation-source-kind)).

In workspace mode both the per-file object and each nested citation object gain a leading `"project":"<alias>"` — the project that contains the file, which is also the citing project — exactly as `refs` does ([§FS-workspace.8.2](FS-workspace.md#82-grund-refs), [§FS-workspace.8.6](FS-workspace.md#86-grund-cover)). Outside workspace mode the field is absent and the bytes above are unchanged:

```json
{"project":"api","path":"apps/api/src/login.rs","citations":[{"project":"api","path":"apps/api/src/login.rs","line":14,"column":5,"id":"FS-login","section":"2","marker":true,"text":"§FS-login.2","enclosing_declaration":"AR-login-flow","enclosing_section":"2"}]}
```

## 4. Exit codes

- `0` — scan succeeded; the emitted file records are the result.
- `2` — the run could not read the tree it was asked about, in either of two ways:
  - **A config the run needs does not load** — including a `[workspace]` block whose members cannot be expanded (a duplicate or invalid alias, a missing member, a block with nothing in scope). `cover` spans the whole workspace ([§FS-workspace.8.6](FS-workspace.md#86-grund-cover)), so it fails on such a tree exactly where `grund list` does.
  - **A scan / I/O error in any project the run loaded** ([§FS-check.2](FS-check.md#2-outputs) partial-scan semantics apply: records found before or after the unreadable file may print, but the result is not trustworthy as complete). A member's unreadable file fails the run at the workspace root, because the grouping the run just printed is incomplete for the tree it claimed ([§FS-workspace.8.7](FS-workspace.md#87-output-and-exit-codes)).

With `--lines`, a third way to exit `2` is a usage error in the request itself, answered before any output ([§FS-cover.6.4](FS-cover.md#64-errors)).

There is no `1`: `cover` is a query over the current tree and has no finding class of its own.

## 5. Why this exists

`grund refs <ID>` answers "who cites this ID?" and `grund list` answers "what IDs exist?". The co-change gate needs the inverse grouping: "for this changed file, what IDs does it cite?" A shell script could run `grund refs` once per ID and regroup the output, but that is slower, loses files with zero citations, and makes every recipe reconstruct scanner state. `cover` provides that view directly while preserving the no-git, no-policy boundary: git diff is an input to the recipe, not to `grund cover`. Which declaration and section own a changed line is the same reconstruction one level down — a caller's own heading regex misplaces named sections, depth rules, fences and doc-comment ends — so `--lines` answers it from the scan as well ([§FS-cover.6](FS-cover.md#6-line-ownership)).

## 6. Line ownership

`grund cover <file> --lines <range>` reports, for each requested range, which declaration bodies and which sections the range's lines lie in, by the rules the scan already applies to a citation site's `enclosing_declaration` and `enclosing_section` ([§FS-cover.3.2](FS-cover.md#32---format-json)). A caller that holds a diff's line numbers can then name the spec points a hunk edits without parsing the file itself.

The input is a path and line numbers in the current tree, never a revision or a diff ([§FS-cover.5](FS-cover.md#5-why-this-exists), [§FS-non-goals.6](FS-non-goals.md#6-decision-database-audit-log-history-tracking)): the caller already has the line numbers from its diff, and they must be numbers of the tree on disk. Ownership is **structural** — which body a line lies in. It is not citation coverage and says nothing about whether a line is grounded, so the co-change recipe's rule against inferring coverage from a nearby citation is unaffected ([§FS-cochange-recipe.evidence](FS-cochange-recipe.md#evidence-both-edits-for-one-resolved-direct-target)).

### 6.1 Input

- Exactly one `<path>`, and it must be a file ([§FS-cover.6.4](FS-cover.md#64-errors)). Discovery and scope are those of any other `cover` run with that path ([§FS-cover.1](FS-cover.md#1-inputs)): a file is narrower than its config root, so the run is one narrowed scan of the enclosing project ([§FS-workspace.8.6.1](FS-workspace.md#861-a-narrower-path-is-one-narrowed-scan)).
- `--lines <N>` or `--lines <N>-<M>`, also spelled `--lines=<…>`, with 1-based line numbers and `N <= M`; `<N>` alone is `<N>-<N>`. The range is inclusive at both ends and its end must not pass the file's last line.
- `--lines` may be repeated, so a caller sends every hunk of one file in one scan. Each range is answered independently, in the order given; overlapping or repeated ranges are answered as often as they are asked.
- `--format` and `[output] format` apply as to any `cover` run, including [§FS-cover.1.1](FS-cover.md#11-an-unsupported---format-is-answered-before-the-load).

Under `--lines` the per-file record of [§FS-cover.3](FS-cover.md#3-outputs) is not emitted; the two record shapes never mix in one run. Without `--lines`, `cover` prints exactly what it printed before.

### 6.2 Ownership rules

A line's **owner** is the declaration whose body range contains it, the nearest preceding one when bodies nest; its **section** is the innermost accepted section of that declaration containing it, or none. These are the scan's own rules for a citation site, applied to every line, so a line that carries a citation is owned exactly as that citation's `enclosing_declaration` / `enclosing_section` say. At the edges:

- A declaration's heading line belongs to that declaration, under no section.
- A section heading line belongs to its own section.
- A blank line belongs to the unit it sits in: one just above a heading still belongs to the section above it.
- A Markdown body ends at the next heading of the same or a shallower level; from there on nothing owns the lines unless another declaration does.
- A rejected or duplicate section heading closes the section above it and opens nothing; its lines belong to the declaration under no section until an accepted heading opens one. The first occurrence of a duplicated path keeps its lines.
- A `#` line inside a fenced code block is not a heading and changes nothing.
- A stub owns only its one line.
- A source declaration owns its comment or docstring block up to the next declaration line in that block. The code below the block is owned by nothing: grund does not understand scopes ([§FS-non-goals.3](FS-non-goals.md#3-code-ast-parsing)).

### 6.3 Output

One record per `--lines`, in the order given. A record lists the range's **owner runs**: the maximal runs of consecutive lines owned by one declaration, clipped to the range, in line order. A range that crosses a declaration boundary has two or more, and one declaration appears twice when a nested declaration's body interrupts it. Each owner run is partitioned into **section runs**, the maximal runs of lines under one innermost section, where no section is `null` — the lead, or lines no accepted section contains. Lines owned by nothing appear in no owner run.

`--format json` writes one NDJSON object per range, in the shape [§FS-output-shapes.5.3](FS-output-shapes.md#53-cover---lines---formatjson) fixes:

```json
{"path":"docs/functional-spec/FS-config.md","start":115,"end":124,"owners":[{"declaration":"FS-config","start":115,"end":124,"sections":[{"section":"requirements.7","start":115,"end":118},{"section":"requirements.8","start":119,"end":122},{"section":"1","start":123,"end":124}]}]}
{"path":"src/report.rs","start":347,"end":358,"owners":[{"declaration":"AR-checker","start":347,"end":355,"sections":[{"section":"4","start":347,"end":355}]}]}
```

`"owners":[]` means no line of the range is owned. `path` renders as on the [§FS-cover.3.2](FS-cover.md#32---format-json) record of the same run, and `declaration` renders bare, as `enclosing_declaration` does.

`--format text` prints, per range, a heading `<path>:<range>`, then one row per run, in line order: two spaces, the run, two spaces, and its unit — `<declaration>` for a run under no section, `<declaration>.<section>` under one, `(no owner)` for lines nothing owns. A range or run of one line prints as `<N>`, a longer one as `<N>-<M>`:

```
$ grund cover docs/functional-spec/FS-config.md --lines 115-124
docs/functional-spec/FS-config.md:115-124
  115-118  FS-config.requirements.7
  119-122  FS-config.requirements.8
  123-124  FS-config.1
```

A file that exists but that the scan does not read — an excluded path, or a type grund does not scan — produces no record and exits `0`, which is how `cover` already tells "not scanned" from "scanned" ([§FS-cover.2](FS-cover.md#2-behaviour)).

### 6.4 Errors

Each of these is a usage error: one `error:` line on stderr, nothing on stdout, exit `2`, as for every CLI-level failure ([§FS-output-shapes.6](FS-output-shapes.md#6-cli-and-config-failures)):

- `--lines` with no `<path>`: `error: --lines needs a file path`.
- `--lines` with a directory: ``error: --lines needs a file path, and `<path>` is a directory``.
- `--lines` with no value: `error: --lines requires a value`.
- A range that is not `<N>` or `<N>-<M>` in decimal digits: ``error: --lines takes <N> or <N>-<M>, got `<range>` ``.
- A `0` in a range: ``error: --lines `<range>`: lines are numbered from 1``.
- An end before its start: ``error: --lines `<range>` ends before it starts``.
- An end past the file's last line, the commonest mistake, line numbers taken from the old side of a diff: ``error: --lines `<range>` ends past line <L>, the last line of `<path>` ``.

`<path>` and `<range>` are echoed as the caller wrote them; `<L>` is the file's line count. A path that does not exist exits `2` as it does without `--lines`, and so does a scan that cannot read the file ([§FS-cover.4](FS-cover.md#4-exit-codes)). There is still no exit `1`.
