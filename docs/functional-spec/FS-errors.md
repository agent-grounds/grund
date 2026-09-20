# FS-errors: grund emits messages in fixed shapes

This spec defines the style every `grund` subcommand uses when it speaks to a user or to a downstream tool. It is cross-cutting: [§FS-check](FS-check.md#fs-check-grund-validates-every-reference-in-a-repo), [§FS-show](FS-show.md#fs-show-grund-reads-a-single-declaration-body-by-id), [§FS-list](FS-list.md#fs-list-grund-lists-every-declared-id), [§FS-refs](FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id), [§FS-cover](FS-cover.md#fs-cover-grund-groups-citations-by-scanned-file), [§FS-fmt](FS-fmt.md#fs-fmt-grund-normalizes-references-in-bulk), [§FS-fetch](FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot), [§FS-init](FS-init.md#fs-init-grund-bootstraps-a-new-grund-conformant-repo), [§FS-id](FS-id.md#fs-id-grund-proposes-ids-for-new-declarations), [§FS-config](FS-config.md#fs-config-grund-reads-a-toml-config-file-found-by-walking-up), and [§FS-completions](FS-completions.md#fs-completions-grund-completes-declared-ids-in-shells) all conform to it, and the global-flag behaviour in [§FS-cli](FS-cli.md#fs-cli-grunds-command-line-surface-conventions) routes its errors through [§FS-errors.2.2](FS-errors.md#22-cli-level-message) here. Serves [§GOAL-friendliness-first.1](../goals.md#1-hard-requirements) (errors point at the line, no surprises) and [§GOAL-no-silent-breakage.1](../goals.md#1-what-counts-as-user-visible) (the message shapes are user-visible output).

The shapes are **frozen** by the same logic as [§FS-non-goals.9](FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization): two correctly-configured installs must agree on what they print. A subcommand that needs to say something new picks one of the shapes below; it does not invent an ad hoc one.

For verbose implementer examples of JSON objects, empty-output behavior, stream split, and ordering, see [§FS-output-shapes](FS-output-shapes.md#fs-output-shapes-machine-readable-output-shapes). This file defines the general rules; that appendix pins representative wire examples.

## 1. Streams

`grund` follows the **linter convention** (`eslint`, `ruff`, `shellcheck`, `golangci-lint`): a checker's findings *are* its output, so they go to **stdout** — `grund check | grep …`, `grund check > findings.txt`, and `grund check --format=json | jq …` all work with no stream redirection. `stderr` is reserved for what the command says *about* the run, not *as* its output. What stdout carries is [§FS-errors.1.1](FS-errors.md#11-what-stdout-carries), and what stderr carries is [§FS-errors.1.2](FS-errors.md#12-what-stderr-carries).

The two are never mixed: `grund check 2>/dev/null` shows you the findings and only the findings; `grund check >/dev/null` shows you only the run-level errors; `grund <ID> | …` is the body and nothing else.

### 1.1 What stdout carries

**stdout** carries the command's output:

- a query result — the body printed by `grund <ID>`, the catalog from `grund list`, the citations from `grund refs`, the file graph from `grund cover`, the ID from `grund id`, the config from `grund config show`;
- a checker report — every located finding from `grund check` ([§FS-check.2.1](FS-check.md#21-report-format)), the text-mode `success` marker from a clean `grund check`, and the would-change / did-change report from `grund fmt` ([§FS-fmt.3](FS-fmt.md#3-outputs)).
- `grund check --format=json` is diagnostics-only: on success with nothing to report, stdout is empty.

### 1.2 What stderr carries

**stderr** carries everything else:

- `error:` lines — a launch-time failure or an I/O failure that means the run could not do its job ([§FS-errors.2.2](FS-errors.md#22-cli-level-message)), always with a non-zero exit;
- `warning:` lines about the run itself, not its content — e.g. an empty scan ([§FS-check.2.2](FS-check.md#22-empty-scan)) — exit unchanged;
- `note:` / `hint:` recovery breadcrumbs ([§FS-refs.2](FS-refs.md#2-behaviour), [§FS-show.3.4](FS-show.md#34-what-a-failed-query-prints));
- the bare message a *failed query* prints when it has no result to put on stdout ([§FS-errors.2.3](FS-errors.md#23-bare-query-failure) — an ID query on a missing ID);
- `grund init`'s file-by-file transcript ([§FS-errors.6](FS-errors.md#6-the-grund-init-transcript) — `init`'s real output is the scaffold on disk; the transcript is progress), and `grund integrations --write`'s transcript in the same shape ([§FS-errors.6](FS-errors.md#6-the-grund-init-transcript)).

## 2. The Fixed Shapes

### 2.1 Located finding

A diagnostic that points at a specific source site:

```
<path>:<line>: <message>
<path>:<line>: <channel>: <message>
```

- `<path>` is rendered against the report base that `relative_paths` selects ([§FS-config.3.6](FS-config.md#36-output--report-format)).
- `<line>` is 1-indexed.
- `<channel>` is the lowercase `error`, `warning`, or `suggestion` marker used
  by `grund check`; other commands omit it.
- `<message>` is a single line — no embedded newlines, no terminal period — and
  does not include the channel marker.
- The `<path>:<line>:` prefix is mandatory: editors and agents jump on this exact shape.

Emitted on **stdout** — it is the command's output ([§FS-errors.1.1](FS-errors.md#11-what-stdout-carries)). Which form each command uses is [§FS-errors.2.1.1](FS-errors.md#211-which-form-each-command-uses).

#### 2.1.1 Which form each command uses

Every finding from
`grund check` uses the channel-bearing form ([§FS-check.2.1](FS-check.md#21-report-format)); every would-change line from `grund fmt` ([§FS-fmt.3](FS-fmt.md#3-outputs)) and every citation from `grund refs` ([§FS-refs.3.1](FS-refs.md#31---format-text-default)) keeps the channel-less form. The optional LSP server likewise keeps its existing diagnostic content ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)); the CLI text marker does not become part of an LSP message. The `<path>:<line>:` prefix is shared; the command, stream, and `check` channel distinguish the meanings.

### 2.2 CLI-level message

A line that is about the *run*, not a finding at a site in the repo:

```
error: <message>
warning: <message>
```

- On **stderr** ([§FS-errors.1.2](FS-errors.md#12-what-stderr-carries)) — it is not the command's output.
- The literal leading `error: ` / `warning: ` prefix is what distinguishes a
  CLI-level message from a located finding, whose severity marker follows its
  `<path>:<line>:` prefix. CI scripts grep for the leading `error:` to tell a
  launch-time failure from a clean run that found findings on stdout.

Where the message text still carries a location is [§FS-errors.2.2.1](FS-errors.md#221-a-location-inside-the-message-text); the exit code each prefix accompanies is [§FS-errors.2.2.2](FS-errors.md#222-exit-codes); who uses the shape is [§FS-errors.2.2.3](FS-errors.md#223-who-uses-it); how it renders under `--format=json` is [§FS-errors.2.2.4](FS-errors.md#224-under---formatjson).

#### 2.2.1 A location inside the message text

The `<path>:<line>:` *prefix* a located finding wears ([§FS-errors.2.1](FS-errors.md#21-located-finding)) is never used
here — a line beginning with that prefix is the signal of a per-site finding
on stdout, whether a `check` channel marker or the message follows it. The
message *text* may still carry a location: a `grund.toml` schema error is
reported `error: <path>:<line>: <message>` ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)) — the leading `error:` marks it CLI-level (stderr, non-zero exit), and the `<path>:<line>:` inside the text is the breadcrumb to the bad line, since a config file has one where a bad flag does not. Other CLI-level messages carry a location in the text the same way when they have one — a config line (`warning: b/grund.toml:3: …`, [§FS-check.4.8](FS-check.md#48-unlisted-workspace-block)) or a whole file (`error: <path>: <reason>`, [§FS-check.2](FS-check.md#2-outputs)) — or name the file in prose (e.g. `error: read grund.toml: Permission denied (os error 13)`).

#### 2.2.2 Exit codes

`error:` always accompanies a non-zero exit — exit `2` unless a point names
otherwise — for a failure that means the run could not do its job: a
launch-time, setup, or I/O failure that leaves no query context or
trustworthy scan, or an operational command's own failure, such as `fetch`'s
rejected integration output or ambiguous existing content
([§FS-fetch.7](FS-fetch.md#7-output-and-exits)). The named
exception is `grund config validate`, which prints the same located
`error: <path>:<line>: <message>` for an invalid config and exits `1`
([§FS-config.4.1](FS-config.md#41-grund-config-validate-path)). `warning:` leaves the exit code alone — it is a
caution, not a failure.

Once context has selected a project's ID grammar, a resolver-rejected operand
is instead an exit-`1` query failure, alongside `grund id`'s empty-slug /
collision and the other failed ID queries, so it takes the bare shape of
[§FS-errors.2.3](FS-errors.md#23-bare-query-failure) with no `error:` prefix. Grund 0.14.0 temporarily preserves `refs`' old
`error:` / exit-`2` shape and appends the migration warning in
[§FS-refs.4](FS-refs.md#4-exit-codes).

#### 2.2.3 Who uses it

Used by [§FS-cli.4](FS-cli.md#4-errors-with-no-source-location) (unknown subcommand / bad flag), [§FS-id.6](FS-id.md#6-exit-codes) (unknown kind, unknown `--format`, scan / I/O error), [§FS-config.4.3](FS-config.md#43-invalid-config-behavior) (config validation), [§FS-check.2.1.1](FS-check.md#211-cli-level-messages) (a malformed config or a per-file read failure mid-walk), [§FS-check.2.2](FS-check.md#22-empty-scan) (the empty-scan `warning:`), [§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing) (the empty citation-obligation `warning:`), and any subcommand reporting a launch-time failure.

#### 2.2.4 Under `--format=json`

A *launch-time* `error:` (bad flag, unreadable config, missing path) is printed as raw text and is never JSON-ified; a *mid-walk* per-file failure collected by `grund check` is one of the report's diagnostics and is rendered in `--format=json` like the others ([§FS-errors.5.2.3](FS-errors.md#523-run-level-diagnostics-in-checks-report)), still on stderr because it is not a finding about the spec graph.

### 2.3 Bare query failure

When a subcommand established its query context but has no result to put on
stdout — an ID query on a missing ID, a missing section, an invalid ID under the
selected grammar, an ambiguous ID or section, or a broken stub; `grund refs` from
0.15.0 when the selected resolver rejects an invalid ID or ambiguous number-only
shorthand; `grund id` when the title slugifies to nothing or the proposed ID
collides with an existing declaration:

```
<message>
```

- No prefix at all, on **stderr**, exit `1`. There is no single site to point at and no result to return, so stdout is empty; this line plus the exit code is what tells the caller what happened.
- Distinct from [§FS-errors.2.2](FS-errors.md#22-cli-level-message): there is no `error:` prefix, because this is not a launch/run failure — the command ran fine, the request was just unsatisfiable.

How ambiguity and hint lines read is [§FS-errors.2.3.1](FS-errors.md#231-ambiguity-and-hint-lines); who uses the shape is [§FS-errors.2.3.2](FS-errors.md#232-who-uses-it).

#### 2.3.1 Ambiguity and hint lines

Ambiguity messages list every site in lexicographic `path:line` order ([§FS-show.2.2.1](FS-show.md#221-ambiguous-id)), except an ambiguous number-only shorthand's, which lists its candidate IDs in ID order ([§FS-show.2.2.1.1](FS-show.md#2211-an-ambiguous-shorthand-names-its-candidates)). A `hint:` line may follow on stderr where the next step is obvious ([§FS-errors.1.2](FS-errors.md#12-what-stderr-carries)).

#### 2.3.2 Who uses it

Used by ID queries (missing or invalid ID and missing section — [§FS-show.3](FS-show.md#3-outputs); ambiguous ID — [§FS-show.2.2.1](FS-show.md#221-ambiguous-id); ambiguous section — [§FS-show.2.2.2](FS-show.md#222-ambiguous-section); broken stub — [§FS-show.2.3.4](FS-show.md#234-broken-stub)), by `refs` resolver rejections after its compatibility window ([§FS-refs.4](FS-refs.md#4-exit-codes)), and by `grund id` for a satisfiable query context whose requested allocation has no result (empty slug — [§FS-id.3](FS-id.md#3-slug-derivation); proposed-ID collision — [§FS-id.5](FS-id.md#5-collision-check)). `check` does not use this shape — every line it prints is a located finding (stdout) or a CLI-level message (stderr).

### 2.4 Text success marker

A text-mode `grund check` run with zero retained errors and zero retained warnings, and no retained suggestion or unselectable run-level line ([§FS-check.2.1.3](FS-check.md#213-the-success-marker)), prints exactly:

```
success
```

One trailing newline follows the line. The marker is on **stdout** because it is the command's output ([§FS-errors.1.1](FS-errors.md#11-what-stdout-carries)), exits `0`, and appears only when the selected report is otherwise empty: it says that report is empty, not that the repository has no findings ([§FS-check.2.1.3](FS-check.md#213-the-success-marker)). It is not emitted in `--format=json`, where stdout remains diagnostics-only.

## 3. Message text

The shape is structural; the text is human-readable. Style rules apply to every shape:

- **Lowercase first letter.** `unknown reference <ID>` — not `Unknown reference <ID>`.
- **No terminal period.** A single-clause message does not end in `.` or `!`. A run-level caution that runs to more than one clause, such as the empty-scan warning ([§FS-check.2.2.2](FS-check.md#222-the-message-names-the-likely-cause)), is written in sentences and punctuates each, its last included.
- **No ANSI colors yet.** Once colored output lands, the `[output] color` key, whose default is `auto` ([§FS-config.3.6](FS-config.md#36-output--report-format)), may add them ([§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path) applies); until then plain bytes are the contract.
- **Stable phrasing.** The exact text of each message is part of the user-visible output covered by [§GOAL-no-silent-breakage.1](../goals.md#1-what-counts-as-user-visible): changing it goes through a deprecation path. Tools grep on it.
- **Quoted user input** appears in double quotes when the input could be confused with surrounding prose: `"<original title>"`, not `<original title>`.
- **One base for every path in the line** — [§FS-errors.3.1](FS-errors.md#31-one-base-for-every-path-in-the-line).

[§FS-errors.3.2](FS-errors.md#32-the-unknown-project-recovery-shape) to [§FS-errors.3.6](FS-errors.md#36-the-agents-init-messages) pin particular messages: the unknown-project recovery shape and its wording migration, `check`'s channel marker, a missing fetch-backed declaration, and `agents-init`.

### 3.1 One base for every path in the line

A path written *inside* the message text — a duplicate declaration's other homes, an ambiguous ID's competing sites, the stub a broken-stub refusal names — is a report path like the `<path>` the shape anchors at, and is rendered against the same base ([§FS-config.3.6](FS-config.md#36-output--report-format)). In a workspace that base is the root the run reports from, never the member the finding came out of ([§FS-workspace.8.1](FS-workspace.md#81-grund-aliasid)): a line whose two halves are relative to two different roots sends the reader — and an editor following it — to a file that is not there. The path the message quotes back from the user's own text, such as a stub's link target, is not a resolved path and stays verbatim.

### 3.2 The unknown-project recovery shape

The unknown-project recovery shape in [§FS-check.3.8](FS-check.md#38-cross-project-citation-failure) freezes the base `unknown project alias <written>` and, when its first non-empty candidate tier supplies alternatives, appends `; did you mean <a>?`, `; did you mean <a> or <b>?`, or `; did you mean <a>, <b> or <c>?`. The base begins lowercase and has no period; the recovery clause has one terminal question mark. Text output carries the whole message, and JSON retains `code: "unknown-project"` while carrying the same bytes in `message` ([§FS-errors.5.1](FS-errors.md#51-on-stdout--the-commands-output)).

### 3.3 The narrowed-run unknown-project wording migration

The narrowed-run scope-only unknown-project message has a two-release wording
migration, whose `0.13.2` compatibility form and `0.14.0` final template are
fixed in [§FS-check.3.8](FS-check.md#38-cross-project-citation-failure). The `0.13.2` suffix is part of the existing error
message, not a second warning diagnostic. Exact-line consumers must migrate
during this window to the stable `code == "unknown-project"`; the code, error
severity, sites, selectors, and exit verdict do not change. Workspace-root
candidate messages and bare unknown-project messages remain unchanged
throughout this migration.

### 3.4 The `check` channel marker

For `grund check`, the fixed rule supplies the channel and every located text
line makes it explicit after the location prefix: [§FS-check.3](FS-check.md#3-errors-detected) is `error:`, [§FS-check.4](FS-check.md#4-warnings) is `warning:`, and enabled [§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in) advisories are `suggestion:`. That structural marker does not change the diagnostic's message bytes. JSON continues to carry the same distinction in its existing `severity` or `channel` field ([§FS-errors.5.1](FS-errors.md#51-on-stdout--the-commands-output)); `fmt`, `refs`, run-level messages, and LSP retain their existing shapes.

### 3.5 A missing fetch-backed declaration

A missing fetch-backed declaration has two frozen identities, `dangling` /
`error` ([§FS-check.3.1](FS-check.md#31-dangling-citation)) and
`missing-snapshot` / `warning` ([§FS-check.4.12](FS-check.md#412-missing-snapshot)); those points own the exact
message bytes, the bare-text fetch remedy, and the precedence of an existing
near-ID or illustration hint over the fetch-action tail.

### 3.6 The `agents-init` messages

The five `agents-init` messages migrate over two releases. In the first release,
each existing message stays as a verbatim contiguous prefix and gains exactly
` — repo maintenance; citation checks still ran; wording changes in grund
0.14.0`. This compatibility form both preserves prefix consumers and tells a
reader that the complete citation check already ran. Exact-line consumers must
migrate during this window to the stable `code == "agents-init"`; code, error
severity, and the default exit verdict do not change. The final `0.14.0` templates are [§FS-errors.3.6.1](FS-errors.md#361-the-final-templates).

#### 3.6.1 The final templates

In `0.14.0`, the compatibility tail is removed and the five final templates are:

```text
repo maintenance: malformed grund managed block: <detail> (does not affect citation validity)
repo maintenance: outdated grund init block v<found> — run `grund init` to update to v<current> (does not affect citation validity)
repo maintenance: unsupported grund init block v<found> — this grund supports v<current> (does not affect citation validity)
repo maintenance: stale grund init block: <section> differ from grund.toml — run `grund init` to refresh (does not affect citation validity)
repo maintenance: missing grund init block v<current> — run `grund init` to install it (does not affect citation validity)
```

These are message classifications only: `repo maintenance` is not a finding
category or selector value ([§FS-check.1](FS-check.md#1-inputs)).

## 4. Determinism

Two runs of the same subcommand on the same input must produce byte-identical stdout *and* stderr ([§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)). This rules out:

- Wall-clock timestamps in messages.
- Process IDs, hostnames, or — in the reports of the tree-reading commands
  [§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes) names — an absolute path or one that escapes the loaded
  root, with no absolute fallback for an in-root target ([§FS-config.3.6](FS-config.md#36-output--report-format),
  [§DF-cli-base-parent-paths.2](../decisions/functional/DF-cli-base-parent-paths.md#2-decision)).
- Non-deterministic ordering — the order each report keeps is [§FS-errors.4.1](FS-errors.md#41-ordering).
- Platform-native path separators in repo-relative output. Any path that appears in a report, JSON field, e2e case manifest, duplicate-site list, stub-link note, or formatter summary is rendered with `/`, so Windows and Unix runs over the same tree compare byte-for-byte.

How `check --only` and `--ignore` keep the contract is [§FS-errors.4.2](FS-errors.md#42-diagnostic-selection).

### 4.1 Ordering

Text `check` findings are grouped as errors,
warnings, then enabled suggestions and sort bytewise by `(path, line,
message)` within each group. JSON `check` findings retain their global
bytewise `(path, line, message)` order across channels. Other reports retain
their existing documented order; multi-site findings anchor at the
lexicographically-first site ([§FS-check.2.1](FS-check.md#21-report-format)).

A message that would otherwise be non-deterministic (e.g. the order of duplicate-declaration sites) is sorted before printing.

Rule findings obey this same ordering without a rule-specific sorter. In a
same-anchor `cite each` group, the fixed message puts the target ID immediately
after `<subject> cites ` and before its count, so bytewise message ordering also
orders shared-prefix target IDs bytewise
([§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits)).

### 4.2 Diagnostic selection

`grund check --only` and `--ignore` preserve [§FS-errors.4](FS-errors.md#4-determinism)'s contract: selection precedes
the format-specific sort and rendering, retained diagnostics keep their
locations, messages, codes, sites, and channels, and reordering or duplicating
selector flags cannot alter the result ([§FS-check.2.1](FS-check.md#21-report-format)).

## 5. JSON format

The subcommands with a machine-readable result or finding surface accept `--format=json`: `check`, `show`, `list`, `refs`, `cover`, `id`, and `integrations` ([§GOAL-friendliness-first.1](../goals.md#1-hard-requirements), [§FS-cli.3](FS-cli.md#3-cross-subcommand-flags), [§FS-integrations.5](FS-integrations.md#5-json-format)). Operational commands whose output is human text or generated files (`fmt`, `fetch`, `init`, `config`, `agent-setup-instructions`, `completions`) do not accept `--format` unless their own spec adds a JSON surface later. JSON follows the same stream split as the text form ([§FS-errors.1](FS-errors.md#1-streams)): what goes to stdout is [§FS-errors.5.1](FS-errors.md#51-on-stdout--the-commands-output), what goes to stderr is [§FS-errors.5.2](FS-errors.md#52-on-stderr--what-is-not-output), and `show --batch` is the query-stream exception of [§FS-errors.5.3](FS-errors.md#53-show---batch).

So `grund check --format=json | jq …`, `grund <ID> --format=json | jq …`, `grund list --format=json | jq …` all work with no stream juggling, and `grund <missing> --format=json | jq …` does not choke because the diagnostic is on stderr where the pipe does not see it.

The text-form messages defined above remain the default. JSON is opt-in.

Value diagnostics are [§FS-errors.5.4](FS-errors.md#54-value-diagnostics); the `code` catalog `check` selects on is [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog).

### 5.1 On stdout — the command's output

`grund check --format=json` emits its findings as NDJSON, one object per line, in the binding-level shape from [§FS-distribution.3.0](FS-distribution.md#30-language-neutral-data-shapes) (`{ severity, path, line, code, message, sites }`); `severity` carries the `error`/`warning` distinction as a structured field, while text carries the explicit channel marker after its location prefix ([§FS-errors.3.4](FS-errors.md#34-the-check-channel-marker)), and `sites` is `null` for an ordinary single-site finding, a `[{ path, line }]` list naming every site for a multi-site finding (a duplicate declaration, [§FS-check.3.3](FS-check.md#33-duplicate-declaration)). A clean JSON check emits no `success` object. A **suggestion** ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)), emitted only under `grund check --suggestions`, carries `"channel": "suggestion"` in place of a `severity` — keeping the frozen `{error, warning}` severity set ([§FS-config.6](FS-config.md#6-what-is-not-configured-here)) intact, so a consumer filtering on `severity` never sees one. Query subcommands' results are [§FS-errors.5.1.1](FS-errors.md#511-query-results).

#### 5.1.1 Query results

Query subcommands emit their result on stdout too: one JSON object for a single-result command (`grund <ID> --format=json` — [§FS-show](FS-show.md#fs-show-grund-reads-a-single-declaration-body-by-id); `grund id --format=json` — [§FS-id](FS-id.md#fs-id-grund-proposes-ids-for-new-declarations)), NDJSON — one object per row — for a list command (`grund list` per declaration, `grund refs` per citation, `grund cover` per scanned file).

### 5.2 On stderr — what is not output

A *failed ID query* (`ID not found` / `ambiguous` / `broken stub` / `section not found` / `invalid ID`, exit `1`) emits its one diagnostic object on stderr in the same `{ severity, path, line, code, message, sites }` shape, with `path` and `line` `null` — there is no single site, and there is no result, so nothing goes to stdout. This includes `refs`' invalid-ID and ambiguous-number-only rejections from 0.15.0; their codes are respectively `invalid-id` and `ambiguous`, both carry `sites:null`, and neither carries the text-mode hint.

Which ambiguity refusals carry `sites` is [§FS-errors.5.2.1](FS-errors.md#521-sites-on-an-ambiguity-refusal); the messages that stay raw text under any `--format` are [§FS-errors.5.2.2](FS-errors.md#522-launch-time-messages-stay-text); run-level diagnostics in `check`'s report are [§FS-errors.5.2.3](FS-errors.md#523-run-level-diagnostics-in-checks-report).

#### 5.2.1 `sites` on an ambiguity refusal

`sites` carries the `[{ path, line }]` list the message names, the same pairs in the same order, for an `ambiguous` refusal naming an ID with two homes ([§FS-show.2.2.1](FS-show.md#221-ambiguous-id)) and for an `ambiguous-section` refusal ([§FS-show.2.2.2](FS-show.md#222-ambiguous-section)). The number-only shorthand's `ambiguous` refusal names candidate IDs rather than sites ([§FS-show.2.2.1](FS-show.md#221-ambiguous-id)), so — like every other query failure — it carries `sites: null`; a consumer tells the two `ambiguous` shapes apart by whether `sites` is `null`, not by `code`.

#### 5.2.2 Launch-time messages stay text

A *launch-time* CLI-level message ([§FS-errors.2.2](FS-errors.md#22-cli-level-message)) — an error such as a bad flag, unknown kind, unknown project alias, or unreadable config or path (exit `2`), or a warning carried in the run's warning channel, such as [§FS-check.4.7](FS-check.md#47-a-workspace-member-swallows-the-blocks-own-scan)'s, [§FS-check.4.10](FS-check.md#410-include_root--false-leaves-the-blocks-own-files-unread)'s and [§FS-workspace.6.1.7.6](FS-workspace.md#6176-how-the-undecidable-claim-warning-travels)'s — stays as its `error:` / `warning:` text line on stderr regardless of `--format`. What decides is which channel carries the fact: a warning settled from the loaded config but carried as one of `check`'s report warnings, such as [§FS-check.4.3](FS-check.md#43-redundant-config-pair)'s and [§FS-check.4.11](FS-check.md#411-config-read-from-the-deprecated-agents-location)'s, is data, and renders as a JSON diagnostic ([§FS-errors.5.2.3](FS-errors.md#523-run-level-diagnostics-in-checks-report)). During 0.14.0 only, the two `refs` resolver rejections of [§FS-errors.5.2](FS-errors.md#52-on-stderr--what-is-not-output) retain that raw error and hint policy under JSON and append the raw warning fixed by [§FS-refs.4](FS-refs.md#4-exit-codes).

#### 5.2.3 Run-level diagnostics in `check`'s report

A run-level diagnostic in `grund check`'s report — about the run, not a finding about the graph, such as the empty-scan `warning:` ([§FS-check.2.2](FS-check.md#22-empty-scan)), the nothing-recognized `warning:` ([§FS-check.4.5](FS-check.md#45-nothing-recognized)), or a per-file read failure collected mid-walk (a `line`-less diagnostic) — is likewise on stderr in both forms; under JSON a run-level warning carries `path`, `line`, and `sites` all `null`, any location being in its message text ([§FS-check.4.8](FS-check.md#48-unlisted-workspace-block)).

### 5.3 `show --batch`

`show --batch --format=json` is the explicit query-stream exception. Every
well-formed query, including a failed one, produces one ordered stdout envelope
([§FS-output-shapes.4.1](FS-output-shapes.md#41-show---batch---formatjson)); stderr
is reserved for invocation, batch-input, configuration, and scan failures that
abort the run. A batch-input error uses raw
`error: batch input line <N>: <reason>` text because no data-producing phase
began. The entire stream is validated before scanning, so this error leaves
stdout empty and produces no partial records.

### 5.4 Value diagnostics

Value diagnostics use the same object and streams. `invalid-value-declaration`, `invalid-value-binding`, and `value-mismatch` are fixed error codes; a mismatch's `sites` is the sorted declaration-site array, and its `message` is byte-identical to the text message after the primary `path:line:` prefix and its channel marker ([§FS-errors.2.1](FS-errors.md#21-located-finding), [§FS-values.5](FS-values.md#5-resolution-diagnostics-and-exit-status)). Home JSON input that [§FS-values.5.3](FS-values.md#53-incomplete-input-and-deterministic-output) counts as incomplete remains a run-level incomplete-scan failure at exit `2` rather than a semantic value diagnostic.

### 5.5 The `check` code catalog

For `grund check`, `code` is also the exact public selector vocabulary for
[§FS-check.1](FS-check.md#1-inputs). The supported catalog is sorted and is:

```text
agents-init
broken-stub
dangling
declaration-near-miss
deprecated-config-location
discouraged-citation
duplicate
duplicate-section
empty-citation-obligation
empty-scan
escaped-citation-resolves
forbidden-citation
full-scope-ignored
inline-citation-style
invalid-value-binding
invalid-value-declaration
io
misplaced-declaration
missing-citation
missing-index-entry
missing-section
missing-snapshot
nothing-recognized
optional-member-absent
orphan-section
out-of-scope-dangling
out-of-scope-missing-section
out-of-scope-shorthand-citation
out-of-scope-unknown-project
oversized-lead
redundant-config
section-heading-level
section-outside-declaration
shorthand-citation
shorthand-numeric-run
suggested-citation
ungrounded
unknown-project
unlinked-index-entry
unlisted-workspace-block
unmarked-heading
unused
value-mismatch
```

Every future check diagnostic code enters this catalog in the release that
introduces it; renaming or removing one requires compatibility treatment. The
catalog changes neither the NDJSON object nor the library/LSP report. `io` is a
recognized code but an incomplete-scan safety diagnostic remains retained and
exit `2` even when `--ignore io` or an excluding `--only` set is present
([§FS-check.2](FS-check.md#2-outputs)).

## 6. The `grund init` transcript

`grund init` ([§FS-init.2.2](FS-init.md#22-stdout--stderr)) writes status lines to **stderr** — `wrote AGENTS.md`, `appended CLAUDE.md`, `exists grund.toml`, etc. — followed by the `next:` block. These are **not** the command's output: `init`'s output is the scaffold it wrote to disk; the transcript is progress, and nobody pipes `grund init`. They use a `<verb> <path>` shape. This is one of two carve-outs from [§FS-errors.1](FS-errors.md#1-streams); the other is `grund integrations --write`, which reports what it wrote on stderr in the same `<verb> <path>` shape ([§FS-integrations.4.1.6](FS-integrations.md#416-outcome-verbs)). Every other subcommand puts its output on stdout. In particular `grund fmt --write` ([§FS-fmt.3](FS-fmt.md#3-outputs)) does **not** use this shape: its `rewrote N line(s):` report is `fmt`'s output and goes to stdout, the same stream as its `--check` dry-run report. A subcommand with no output and no transcript stays silent and lets the exit code carry the verdict.

## 7. What this rules out

- A severity prefix (`error:`, `warning:`) ahead of a located finding's `<path>:<line>:` prefix — that leading position marks a CLI-level message ([§FS-errors.2.2](FS-errors.md#22-cli-level-message)); the channel marker `check` places goes after the location ([§FS-errors.3.4](FS-errors.md#34-the-check-channel-marker)).
- Multi-line messages. A finding that wants to elaborate uses `--format=json` and its `code` ([§FS-errors.5.1](FS-errors.md#51-on-stdout--the-commands-output)), not a wrapped paragraph.
- Interactive prompts, progress bars, or spinners. Per [§FS-non-goals.10](FS-non-goals.md#10-interactive-mode), every subcommand is non-interactive.
- Localization. Messages are English; translation is downstream's problem.
