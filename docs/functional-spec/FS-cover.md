# FS-cover: grund groups citations by scanned file

The `cover` subcommand exposes the citation graph as data: for each scanned file, which declaration IDs does it cite, and where? It is the plumbing of a diff-aware co-change check ([§FS-cochange-recipe](FS-cochange-recipe.md#fs-cochange-recipe-an-opt-in-git-recipe-reports-related-declaration-and-test-edits)): git decides what changed, and `cover` says which IDs the changed files lean on. Given line ranges of one file with `--lines`, it says instead which declaration and section own each line ([§FS-cover.6](FS-cover.md#6-line-ownership)), so a caller holding a diff hunk can name the spec points it edits. Serves [§GOAL-agent-grounding.1](../goals.md#1-the-three-layers).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, home, section, catalog), [§FS-terms.terms.2](FS-terms.md#terms2-citations)
(marker, citation), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope, config root, workspace, member, alias),
[§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding), and [§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations) (value,
binding, snapshot).

## 1. Inputs

```
grund cover [<path>] [--glob <pattern> …] [--format text|json]
grund cover <file> --lines <N>|<N>-<M> [--lines …] [--format text|json]
```

- `<path>` — the tree to scan; with none, the project that discovery finds ([§FS-scan.path.none](FS-scan.md#pathnone-a-run-with-no-path-covers-the-project-discovery-finds)). An explicit path bounds the **scan**, and that decides the scope: every project at a workspace root, that member alone inside a member ([§FS-workspace.8.6](FS-workspace.md#86-grund-cover)), and one narrowed scan of the enclosing project for a path narrower than its config root ([§FS-scan.path.cover](FS-scan.md#pathcover-cover-runs-one-narrowed-scan)).
- `--glob <pattern>` — keep only the citations a pattern matches; every file still gets its record ([§FS-cover.globs](FS-cover.md#globs---glob-keeps-the-citations-a-pattern-matches)). Repeatable; refused beside `--lines` ([§FS-cover.6.4](FS-cover.md#64-errors)).
- `--lines <N>|<N>-<M>` — answer line ownership for that range of `<file>` instead of grouping citations ([§FS-cover.6.1](FS-cover.md#61-input)). Repeatable.
- `--format text|json` — the output shape ([§FS-cover.3](FS-cover.md#3-outputs)), default `text`. An unsupported value is answered before anything is loaded, as every command answers a bad run flag ([§FS-cli.3.5](FS-cli.md#35-a-bad-run-flag-is-answered-before-the-load)).

`cover` is a non-interactive query ([§FS-non-goals.10](FS-non-goals.md#10-interactive-mode)). It reads no git history ([§FS-non-goals.6](FS-non-goals.md#6-decision-database-audit-log-history-tracking)) and parses no AST ([§FS-non-goals.3](FS-non-goals.md#3-code-ast-parsing)).

A narrowed `cover` reads every E2E case one of whose files its `<path>` holds as the tree spells it under the kind's configured folder, whether the case directory or the folder itself is a link and wherever the link resolves. A failure reading such a file is held as [§FS-declarations.e2e-case.failure](FS-declarations.md#e2e-casefailure-a-case-that-cannot-be-read-fails-alone) says for any run narrowed to a path, so the run exits `2` naming the file at that spelling, as `grund check` on the same path does, though the remedy closing the line follows the path the run was handed ([§FS-check.2.4.3](FS-check.md#243-what-failed-and-where-decide-the-code-and-the-fix)). A path that holds none of the case's failing files, such as another readable file of the case or the resolved target of a case linked out of the folder, prints what it prints over a healthy case. One shape exits differently from `check`: where the folder itself is a link, a directory at, above or below its target as it resolves reads the cases there and names a failure at the configured spelling, exit `2`, where `grund check` on that path cautions and exits `0`. Decided in [§DF-cover-reads-the-case-its-path-holds](../decisions/functional/DF-cover-reads-the-case-its-path-holds.md#df-cover-reads-the-case-its-path-holds-a-narrowed-grund-cover-reads-the-e2e-case-its-path-holds-so-a-case-file-named-on-the-command-line-or-a-case-linked-out-of-the-cases-folder-fails-as-check-does).

## 2. Behaviour

`cover` renders the citations the scan recognized, with no grammar of its own, and judges none of them: whether a file is covered enough, a hunk behavioral or a spec and test co-change required is a question for a recipe built on it ([§FS-cochange-recipe](FS-cochange-recipe.md#fs-cochange-recipe-an-opt-in-git-recipe-reports-related-declaration-and-test-edits)).

It prints one record per scanned file, sorted by path, its citations sorted by `(line, column)`. A file that cites nothing still gets its record, so a caller can tell it from a file outside the scan.

Every citation the scan recognized in a file counts toward that file:

- A cross-project `<§><alias>/<ID>`, whether or not the alias resolves ([§FS-workspace.8.6.3](FS-workspace.md#863-qualified-citations-count-toward-the-citing-file), [§DF-cover-workspace-scope.2.2](../decisions/functional/DF-cover-workspace-scope.md#22-a-qualified-citation-counts-toward-the-citing-file)).
- An owned declaration-local numeric citation, as a citation of its owner's ID and numeric section whose `text` is the local spelling ([§FS-citations.recognized.local-sections](FS-citations.md#recognizedlocal-sections-declaration-local-numeric-section-candidates)).
- A value binding's citation; a value source is catalog input, not a scanned file, and adds no record or citation ([§FS-values.2.2](FS-values.md#22-declarations-from-value-sources), [§FS-values.3.2](FS-values.md#32-recognized-text-contexts)).
- A citation parsed under a per-kind format while its fetched snapshot is missing: fetching changes what resolves, not what a file cites, and `cover` never runs the integration ([§REQ-runs-offline.1](../requirements/REQ-runs-offline.md#1-read-and-verification-paths-execute-nothing)).

## globs: `--glob` keeps the citations a pattern matches

`grund cover [<path>] --glob <pattern>` keeps, in each file record, only the citations whose cited coordinate a pattern matches, as [§FS-coordinate-globs.matching.citations](FS-coordinate-globs.md#matchingcitations-on-refs-and-cover-a-glob-selects-citation-sites) matches it, judged against the project each citation targets ([§FS-coordinate-globs.scope.targets](FS-coordinate-globs.md#scopetargets-on-a-citation-the-qualifier-names-the-project-it-targets)). Repeated, the patterns union. Every scanned file keeps its record, in the same order and shape ([§FS-cover.3](FS-cover.md#3-outputs)), even with no citation left, and a pattern that matches nothing prints every record empty and exits `0`. A refused pattern exits `2` before anything is printed, and the note of a pattern no kind in scope can start goes to stderr ([§FS-coordinate-globs.errors](FS-coordinate-globs.md#errors-refusals-notes-and-hints)).

```text
$ grund cover --glob 'FS-audit{,.**}'
docs/fs/FS-audit.md:
  (no citations)
docs/fs/FS-ledger.md:
  3:24 §FS-audit
  7:13 §FS-audit.entries
docs/fs/FS-login.md:
  3:24 §FS-audit
  7:33 §FS-audit.entries
src/app.rs:
  4:4 §FS-audit
```

## 3. Outputs

### 3.1 `--format text` (default)

Text goes to stdout: a `<path>:` line per scanned file, then one line per citation, two spaces, `<line>:<column>`, a space and the token as written, or `(no citations)` under the same indent:

```
$ grund cover
docs/functional-spec/FS-001-widget.md:
  (no citations)
docs/functional-spec/README.md:
  5:4 §FS-001-widget
docs/notes.md:
  3:28 §FS-001-widget
```

### 3.2 `--format json`

NDJSON on stdout, one object per scanned file, `path` then `citations`:

```json
{"path":"docs/functional-spec/FS-001-alpha.md","citations":[{"path":"docs/functional-spec/FS-001-alpha.md","line":3,"column":42,"id":"FS-002-beta","section":null,"marker":true,"text":"§FS-002-beta","enclosing_declaration":null,"enclosing_section":null},{"path":"docs/functional-spec/FS-001-alpha.md","line":7,"column":50,"id":"FS-002-beta","section":null,"marker":true,"text":"§FS-002-beta","enclosing_declaration":"FS-001-alpha","enclosing_section":null},{"path":"docs/functional-spec/FS-001-alpha.md","line":11,"column":23,"id":"FS-002-beta","section":null,"marker":true,"text":"§FS-002-beta","enclosing_declaration":"FS-001-alpha","enclosing_section":"terms"},{"path":"docs/functional-spec/FS-001-alpha.md","line":15,"column":26,"id":"FS-002-beta","section":null,"marker":true,"text":"§FS-002-beta","enclosing_declaration":"FS-001-alpha","enclosing_section":"1"}]}
{"path":"docs/functional-spec/FS-002-beta.md","citations":[]}
```

Each citation object carries `path`, `line`, `column`, `id`, `section`, `marker`, `text`, `enclosing_declaration` and `enclosing_section`, in that order: the fields of a `refs --format json` record ([§FS-output-shapes.5.1](FS-output-shapes.md#51-refs---formatjson)), with the same names, order and types, so a caller can compare the two without a mapping layer. `id` says what the token says, keeping the alias a citation was written with and adding none to a local one ([§FS-workspace.8.6.4](FS-workspace.md#864-the-rendered-id-says-what-the-token-says)), while `refs <ID>` renders it as the target its query named, so an `<§>api/FS-login` site reads `"id":"FS-login"` there and `"id":"api/FS-login"` here.

`enclosing_declaration` and `enclosing_section` name the declaration and section the **citing** site sits in, as `id` and `section` name its target, by the ownership rules of [§FS-cover.6.2](FS-cover.md#62-ownership-rules), numbered and named alike (`"1"` under `## 1. Inputs`, `"terms"` under `## terms: Terms`). Both are always present, each `string | null`: `enclosing_declaration` is `null` where no declaration owns the site, and `enclosing_section` is `null` where no accepted section contains it, every site in a declaration's lead above its first section included. `enclosing_declaration` renders bare, under the citing project's `[id]` config, since `project` already names that project.

In workspace mode the file object and each citation object begin with `"project":"<alias>"`, the project that contains the file, which is the citing project ([§DF-cover-workspace-scope.2.3](../decisions/functional/DF-cover-workspace-scope.md#23-the-project-field-is-added-and-only-in-workspace-mode)); outside workspace mode the field is absent:

```json
{"project":"alpha","path":"packages/alpha/docs/FS-alpha-thing.md","citations":[{"project":"alpha","path":"packages/alpha/docs/FS-alpha-thing.md","line":3,"column":18,"id":"beta/FS-beta-thing","section":null,"marker":true,"text":"§beta/FS-beta-thing","enclosing_declaration":"FS-alpha-thing","enclosing_section":null}]}
```

## 4. Exit codes

- `0` — the scan succeeded, a `--glob` that matched nothing included.
- `2` — the run could not read the tree it was asked about, or refused the request:
  - **A config the run needs does not load**, a `[workspace]` block whose members cannot be expanded included (a duplicate or invalid alias, a missing member, a block with nothing in scope); spanning the whole workspace ([§FS-workspace.8.6](FS-workspace.md#86-grund-cover)), `cover` fails on such a tree exactly where `grund list` does.
  - **A scan or I/O failure in any project the run loaded**, reported as [§FS-check.2.4](FS-check.md#24-an-incomplete-run) says: records may print around it, but the result is not complete. A member's unreadable file fails a run at the workspace root, because the grouping it printed is incomplete for the tree it claimed ([§DF-cover-workspace-scope.4](../decisions/functional/DF-cover-workspace-scope.md#4-consequences)).
  - **A usage error in the request**, `--glob` beside `--lines` or a refused pattern among them, answered before any output ([§FS-cover.6.4](FS-cover.md#64-errors), [§FS-coordinate-globs.errors.shape](FS-coordinate-globs.md#errorsshape-one-refusal-line)).

There is no `1`: `cover` is a query over the current tree and has no finding of its own.

## 5. Why this exists

`grund refs <ID>` answers "who cites this ID?" and `grund list` answers "what IDs exist?". The co-change check needs the inverse grouping: "for this changed file, what IDs does it cite?" Regrouping `grund refs` once per ID is slower, loses the files that cite nothing, and makes every recipe reconstruct what the scan already holds. Which declaration and section own a changed line is the same reconstruction one level down, where a caller's own heading regex misplaces named sections, depth rules, fences and doc-comment ends.

## 6. Line ownership

`grund cover <file> --lines <range>` reports, for each requested range, which declaration bodies and which sections the range's lines lie in.

### 6.1 Input

- Exactly one `<path>`, and it must be a file ([§FS-cover.6.4](FS-cover.md#64-errors)). Discovery is that of any `cover` run with that path ([§FS-cover.1](FS-cover.md#1-inputs)), and a file is narrower than its config root, so the run is one narrowed scan of the enclosing project ([§FS-scan.path.cover](FS-scan.md#pathcover-cover-runs-one-narrowed-scan)).
- `--lines <N>` or `--lines <N>-<M>`, also spelled `--lines=<…>`, with 1-based line numbers and `N <= M`; `<N>` alone is `<N>-<N>`. The range is inclusive at both ends and must end at or before the file's last line. The numbers are those of the file on disk, never of a revision or a diff ([§FS-non-goals.6](FS-non-goals.md#6-decision-database-audit-log-history-tracking)).
- `--lines` may be repeated, so a caller sends every hunk of one file in one scan. Each range is answered independently, in the order given; overlapping or repeated ranges are answered as often as they are asked.

Under `--lines` the per-file record of [§FS-cover.3](FS-cover.md#3-outputs) is not emitted; the two record shapes never mix in one run.

### 6.2 Ownership rules

A line's **owner** is the declaration whose body range contains it, the nearest preceding one when bodies nest, and its **section** the innermost accepted section of that declaration containing it, or none; a line of a Markdown file's front matter has the owner and section [§FS-check.1.1.5.2.1](FS-check.md#11521-its-citations-belong-to-the-files-first-declaration) gives it. These are the scan's own rules for a citation site, applied to every line, so a line that carries a citation is owned exactly as that citation's `enclosing_declaration` and `enclosing_section` say. Ownership is structural: it says which declaration owns a line, not whether a citation covers it, so a recipe infers no coverage from it ([§FS-cochange-recipe.evidence](FS-cochange-recipe.md#evidence-both-edits-for-one-resolved-direct-target)). At the edges:

- A declaration's heading line belongs to that declaration, under no section; a section heading line belongs to its own section.
- A blank line belongs to the declaration and section it sits in: one just above a heading still belongs to the section above it.
- Past the end of a body ([§FS-declarations.body](FS-declarations.md#body-where-a-declarations-body-ends)), nothing owns a line unless another declaration does.
- A rejected or duplicate section heading closes the section above it and opens nothing; its lines belong to the declaration under no section until an accepted heading opens one. The first occurrence of a duplicated path keeps its lines.
- A stub owns only its one line.
- A source declaration owns its comment or docstring block up to the next declaration line in that block. The code below the block is owned by nothing: grund does not understand scopes ([§FS-non-goals.3](FS-non-goals.md#3-code-ast-parsing)).

### 6.3 Output

One record per `--lines`, in the order given. A record lists the range's **owner runs**: the maximal runs of consecutive lines owned by one declaration, clipped to the range, in line order. One declaration appears twice when a nested declaration's body interrupts it. Each owner run is partitioned into **section runs**, the maximal runs of lines under one innermost section, where no section is `null`: the lead, or lines no accepted section contains. Lines owned by nothing appear in no owner run.

`--format json` writes one NDJSON object per range, in the shape [§FS-output-shapes.5.3](FS-output-shapes.md#53-cover---lines---formatjson) fixes:

```json
{"path":"docs/functional-spec/FS-001-alpha.md","start":1,"end":14,"owners":[{"declaration":"FS-001-alpha","start":5,"end":14,"sections":[{"section":null,"start":5,"end":8},{"section":"terms","start":9,"end":12},{"section":"terms.words","start":13,"end":14}]}]}
{"path":"docs/functional-spec/FS-001-alpha.md","start":33,"end":49,"owners":[{"declaration":"FS-001-alpha","start":33,"end":34,"sections":[{"section":"2","start":33,"end":34}]},{"declaration":"FS-002-beta","start":39,"end":46,"sections":[{"section":null,"start":39,"end":42},{"section":"1","start":43,"end":46}]}]}
```

`--format text` prints, per range, a heading `<path>:<range>`, then one row per run, in line order: two spaces, the run, two spaces, and what owns it: `<declaration>` for a run under no section, `<declaration>.<section>` under one, `(no owner)` for lines nothing owns. A range or run of one line prints as `<N>`, a longer one as `<N>-<M>`:

```
$ grund cover docs/functional-spec/FS-001-alpha.md --lines 1-14 --lines 33-49
docs/functional-spec/FS-001-alpha.md:1-14
  1-4  (no owner)
  5-8  FS-001-alpha
  9-12  FS-001-alpha.terms
  13-14  FS-001-alpha.terms.words
docs/functional-spec/FS-001-alpha.md:33-49
  33-34  FS-001-alpha.2
  35-38  (no owner)
  39-42  FS-002-beta
  43-46  FS-002-beta.1
  47-49  (no owner)
```

A file that exists but that the scan does not read, an excluded path or a type grund does not scan, produces no record and exits `0` ([§FS-cover.2](FS-cover.md#2-behaviour)).

### 6.4 Errors

Each of these is a usage error: one `error:` line on stderr, nothing on stdout, exit `2`, as for every CLI-level failure ([§FS-output-shapes.6](FS-output-shapes.md#6-cli-and-config-failures)):

- `--lines` with no `<path>`: `error: --lines needs a file path`.
- `--lines` with a directory: ``error: --lines needs a file path, and `<path>` is a directory``.
- `--lines` with no value: `error: --lines requires a value`.
- `--lines` beside `--glob`, before the path is read: `error: --glob cannot be combined with --lines`.
- A range that is not `<N>` or `<N>-<M>` in decimal digits: ``error: --lines takes <N> or <N>-<M>, got `<range>` ``.
- A `0` in a range: ``error: --lines `<range>`: lines are numbered from 1``.
- An end before its start: ``error: --lines `<range>` ends before it starts``.
- An end past the file's last line: ``error: --lines `<range>` ends past line <L>, the last line of `<path>` ``.

`<path>` and `<range>` are echoed as the caller wrote them; `<L>` is the file's line count. A path that does not exist exits `2` as it does without `--lines`, and so does a scan that cannot read the file ([§FS-cover.4](FS-cover.md#4-exit-codes)).
