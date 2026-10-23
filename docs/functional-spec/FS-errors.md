# FS-errors: grund emits messages in fixed shapes

This spec defines the style every `grund` subcommand uses when it speaks to a user or to a downstream tool. It is cross-cutting: [§FS-check](FS-check.md#fs-check-grund-validates-every-citation-in-a-repo), [§FS-show](FS-show.md#fs-show-grund-reads-a-single-declaration-body-by-id), [§FS-list](FS-list.md#fs-list-grund-lists-every-declared-id), [§FS-refs](FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id), [§FS-cover](FS-cover.md#fs-cover-grund-groups-citations-by-scanned-file), [§FS-fmt](FS-fmt.md#fs-fmt-grund-normalizes-citations-in-bulk), [§FS-fetch](FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot), [§FS-init](FS-init.md#fs-init-grund-bootstraps-a-new-grund-conformant-repo), [§FS-id](FS-id.md#fs-id-grund-proposes-ids-for-new-declarations), [§FS-config](FS-config.md#fs-config-grund-reads-a-toml-config-file-found-by-walking-up), and [§FS-completions](FS-completions.md#fs-completions-grund-completes-declared-ids-in-shells) all conform to it, and the global-flag behaviour in [§FS-cli](FS-cli.md#fs-cli-grunds-command-line-surface-conventions) routes its errors through [§FS-errors.2.2](FS-errors.md#22-cli-level-message) here. Serves [§GOAL-friendliness-first.1](../goals.md#1-hard-requirements) (errors point at the line, no surprises) and [§GOAL-no-silent-breakage.1](../goals.md#1-what-counts-as-user-visible) (the message shapes are user-visible output).

The shapes are **frozen** by the same logic as [§FS-non-goals.9](FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization): two correctly-configured installs must agree on what they print. A subcommand that needs to say something new picks one of the shapes below; it does not invent an ad hoc one.

For verbose implementer examples of JSON objects, empty-output behavior, stream split, and ordering, see [§FS-output-shapes](FS-output-shapes.md#fs-output-shapes-machine-readable-output-shapes). This file defines the general rules; that appendix pins representative wire examples.

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, home, body, section, catalog),
[§FS-terms.terms.2](FS-terms.md#terms2-citations) (citation, shorthand), [§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (stub), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan,
scope, workspace, member, alias), [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, severity, suggestion, caution,
verdict), [§FS-terms.terms.6](FS-terms.md#terms6-rules-and-directions) (rule), and [§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations) (value, binding, snapshot).

- **located finding** — A finding about the repository, printed with its `<path>:<line>: `
  prefix — as against a CLI-level message, which is about the run and carries no location.
- **code catalog** — The sorted, published set of `check` finding codes. A compound of this
  file's own: it is not the declaration catalog.
- **report base** — The directory every `path` in a report is rendered against, selected by
  `--path-base` when it is passed, else by `relative_paths`.

## requirements: What every message holds to

Two requirements on the shapes as a whole, carrying the marks of [§FS-config.requirements](FS-config.md#requirements-what-the-config-contract-holds-to). Left out because a tool-wide requirement states them: two runs over one input print the same bytes on both streams, which is [§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes) and is restated at [§FS-errors.4](FS-errors.md#4-determinism); the exit code each shape accompanies is [§REQ-never-crashes.2](../requirements/REQ-never-crashes.md#2-exit-codes-are-the-api)'s mapping; a finding at a site points at `path:line` under [§GOAL-friendliness-first.1](../goals.md#1-hard-requirements); and a message's text is user-visible bytes that move only through [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)'s path, so the second point below says how that path is walked for a message rather than restating it.

### requirements.fields: What a program needs from a finding is a field — realized, directional for the next finding

A consumer never parses `message` to act on a finding. What it is, is its `code`, the exact selector vocabulary ([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)); how serious, its `severity` or `channel` ([§FS-errors.3.4](FS-errors.md#34-the-check-channel-prefix), [§FS-errors.5.1](FS-errors.md#51-on-stdout--the-commands-output)); which rule said it, its `authority` ([§FS-errors.5.1](FS-errors.md#51-on-stdout--the-commands-output)); and every site its text names is in `sites` ([§FS-errors.5.1](FS-errors.md#51-on-stdout--the-commands-output), [§FS-errors.5.4](FS-errors.md#54-value-findings)), which for an ambiguity refusal is exactly the pairs the message lists, in its order ([§FS-errors.5.2.1](FS-errors.md#521-sites-on-an-ambiguity-refusal)). Where one code has two shapes, a field tells them apart, as `sites` tells the two `ambiguous` refusals ([§FS-errors.5.2.1](FS-errors.md#521-sites-on-an-ambiguity-refusal)). The two exceptions are about the run rather than the graph: a run-level warning carries its location in its text alone ([§FS-errors.5.2.3](FS-errors.md#523-run-level-findings-in-checks-report)), and a launch-time message stays text under every `--format` and is told by its exit code ([§FS-errors.5.2.2](FS-errors.md#522-launch-time-messages-stay-text)). A finding added from here on that names a site in its text carries it in `sites`, and one that shares a code with a sibling differs from it in a field. Serves [§GOAL-friendliness-first.1](../goals.md#1-hard-requirements).

### requirements.prefix-windows: A wording change keeps the old message whole until its window closes — realized, directional for the next change

Every wording change this file records, but the one exception named below, kept the text a consumer matched as a verbatim contiguous prefix through its window, appended whatever it added after that prefix, named the release the window ends in, and said what an exact-line consumer moves to meanwhile; the code, the severity and the exit verdict did not move ([§FS-errors.3.3](FS-errors.md#33-the-narrowed-run-unknown-project-wording-migration), [§FS-errors.3.6](FS-errors.md#36-the-agents-init-messages), [§FS-errors.3.7](FS-errors.md#37-the-rule-site-unknown-alias-wording-migration), [§FS-errors.3.8](FS-errors.md#38-the-one-grammar-wording-migration)). Bytes appended after a surviving prefix are no change at all and ship in one release ([§FS-errors.3](FS-errors.md#3-message-text)). One change is the named exception: the stale-block template of [§FS-errors.3.6.1](FS-errors.md#361-the-final-templates) kept only `repo maintenance: stale grund init block: ` when it was corrected in place in 0.20.0, under the pre-1.0 licence, with no window, because a window would have shown the ungrammatical sentence on purpose to every project migrating that release while its consumers already match `code == "agents-init"` ([§DF-stale-section-differs-in-0-20](../decisions/functional/DF-stale-section-differs-in-0-20.md#df-stale-section-differs-in-0-20-the-stale-block-finding-names-its-section-as-one-subject-corrected-in-place-in-0200)). Apart from it, a wording change from here on takes the same route — prefix kept, last release named, migration named, verdict unmoved — which is what the new form beside the old of [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) means for the bytes of a message. Serves [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path).

## 1. Streams

`grund` follows the **linter convention** (`eslint`, `ruff`, `shellcheck`, `golangci-lint`): a checker's findings *are* its output, so they go to **stdout** — `grund check | grep …`, `grund check > findings.txt`, and `grund check --format=json | jq …` all work with no stream redirection. `stderr` is reserved for what the command says *about* the run, not *as* its output. What stdout carries is [§FS-errors.1.1](FS-errors.md#11-what-stdout-carries), and what stderr carries is [§FS-errors.1.2](FS-errors.md#12-what-stderr-carries).

The two are never mixed: `grund check 2>/dev/null` shows you the findings and only the findings; `grund check >/dev/null` shows you only the run-level errors; `grund <ID> | …` is the body and nothing else.

### 1.1 What stdout carries

**stdout** carries the command's output:

- a query result — the body printed by `grund <ID>`, the catalog from `grund list`, the citations from `grund refs`, the file graph from `grund cover`, the ID from `grund id`, the config from `grund config show`, and the candidate from `grund config migrate`, whose report goes to stderr ([§FS-config-migrate.2.1](FS-config-migrate.md#21-stdout-is-the-candidate));
- a checker report — every located finding from `grund check` ([§FS-check.2.1](FS-check.md#21-report-format)), the text-mode `success` line from a clean `grund check`, and the would-change / did-change report from `grund fmt` ([§FS-fmt.3](FS-fmt.md#3-outputs)).
- `grund check --format=json` is findings-only: on success with nothing to report, stdout is empty.

### 1.2 What stderr carries

**stderr** carries everything else:

- `error:` lines — a launch-time failure or an I/O failure that means the run could not do its job ([§FS-errors.2.2](FS-errors.md#22-cli-level-message)), always with a non-zero exit;
- `warning:` lines about the run itself, not its content — e.g. an empty scan ([§FS-check.2.2](FS-check.md#22-empty-scan)) — exit unchanged;
- `note:` / `hint:` recovery breadcrumbs ([§FS-refs.2](FS-refs.md#2-behaviour), [§FS-show.3.4](FS-show.md#34-what-a-failed-query-prints)), a coordinate glob's empty-scope `note:` and its PATH `hint:` among them ([§FS-coordinate-globs.errors.note](FS-coordinate-globs.md#errorsnote-a-pattern-no-kind-can-start-gets-a-note), [§FS-coordinate-globs.errors.hint](FS-coordinate-globs.md#errorshint-a-glob-typed-as-path-gets-a-hint));
- the bare message a *failed query* prints when it has no result to put on stdout ([§FS-errors.2.3](FS-errors.md#23-bare-query-failure) — an ID query on a missing ID);
- `grund init`'s file-by-file transcript ([§FS-errors.6](FS-errors.md#6-the-grund-init-transcript) — `init`'s real output is the scaffold on disk; the transcript is progress), and `grund integrations --write`'s transcript in the same shape ([§FS-errors.6](FS-errors.md#6-the-grund-init-transcript)).

## 2. The Fixed Shapes

### 2.1 Located finding

A finding that points at a specific source site:

```
<path>:<line>: <message>
<path>:<line>: <channel>: <message>
```

- `<path>` is rendered against the report base that `--path-base` selects, else `relative_paths` ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from), [§FS-config.3.6](FS-config.md#36-output--report-format)).
- `<line>` is 1-indexed.
- `<channel>` is the lowercase `error`, `warning`, or `suggestion` prefix used
  by `grund check`; other commands omit it.
- `<message>` is a single line — no embedded newlines, no terminal period — and
  does not include the channel prefix.
- The `<path>:<line>:` prefix is mandatory: editors and agents jump on this exact shape.

Emitted on **stdout** — it is the command's output ([§FS-errors.1.1](FS-errors.md#11-what-stdout-carries)). Which form each command uses is [§FS-errors.2.1.1](FS-errors.md#211-which-form-each-command-uses).

#### 2.1.1 Which form each command uses

Every finding from
`grund check` uses the channel-bearing form ([§FS-check.2.1](FS-check.md#21-report-format)); every would-change line from `grund fmt` ([§FS-fmt.3](FS-fmt.md#3-outputs)) and every citation from `grund refs` ([§FS-refs.3.1](FS-refs.md#31---format-text-default)) keeps the channel-less form. The optional LSP server likewise keeps its existing diagnostic content ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)); the CLI text prefix does not become part of an LSP message. The `<path>:<line>:` prefix is shared; the command, stream, and `check` channel distinguish the meanings.

### 2.2 CLI-level message

A line that is about the *run*, not a finding at a site in the repo:

```
error: <message>
warning: <message>
```

- On **stderr** ([§FS-errors.1.2](FS-errors.md#12-what-stderr-carries)) — it is not the command's output.
- The literal leading `error: ` / `warning: ` prefix is what distinguishes a
  CLI-level message from a located finding, whose severity prefix follows its
  `<path>:<line>:` prefix. CI scripts grep for the leading `error:` to tell a
  launch-time failure from a clean run that found findings on stdout.

Where the message text still carries a location is [§FS-errors.2.2.1](FS-errors.md#221-a-location-inside-the-message-text); the exit code each prefix accompanies is [§FS-errors.2.2.2](FS-errors.md#222-exit-codes); who uses the shape is [§FS-errors.2.2.3](FS-errors.md#223-who-uses-it); how it renders under `--format=json` is [§FS-errors.2.2.4](FS-errors.md#224-under---formatjson).

#### 2.2.1 A location inside the message text

The `<path>:<line>:` *prefix* a located finding wears ([§FS-errors.2.1](FS-errors.md#21-located-finding)) is never used
here — a line beginning with that prefix is the signal of a per-site finding
on stdout, whether a `check` channel prefix or the message follows it. The
message *text* may still carry a location: a `grund.toml` schema error is
reported `error: <path>:<line>: <message>` ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)) — the leading `error:` marks it CLI-level (stderr, non-zero exit), and the `<path>:<line>:` inside the text is the breadcrumb to the bad line, since a config file has one where a bad flag does not. Other CLI-level messages carry a location in the text the same way when they have one — a config line (`warning: b/grund.toml:3: …`, which is what [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) still prints on the five scanning surfaces that have no error channel — `check` reports the same fact as a located error on stdout, with this prefix and without the location in the text) or a whole file (`error: <path>: <reason>`, [§FS-check.2](FS-check.md#2-outputs)) — or name the file in prose (e.g. `error: read grund.toml: Permission denied (os error 13)`).

#### 2.2.2 Exit codes

`error:` always accompanies a non-zero exit — exit `2` unless a section names
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
[§FS-errors.2.3](FS-errors.md#23-bare-query-failure) with no `error:` prefix.

#### 2.2.3 Who uses it

Used by [§FS-cli.4](FS-cli.md#4-errors-with-no-source-location) (unknown subcommand / bad flag), [§FS-id.6](FS-id.md#6-exit-codes) (unknown kind, unknown `--format`, scan / I/O error), [§FS-config.4.3](FS-config.md#43-invalid-config-behavior) (config validation), [§FS-check.2.1.1](FS-check.md#211-cli-level-messages) (a malformed config or a per-file read failure mid-scan), [§FS-check.2.2](FS-check.md#22-empty-scan) (the empty-scan `warning:`), [§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing) (the empty citation-obligation `warning:`, a table's or a rule's), [§FS-check.1.4.1](FS-check.md#141-a-selected-check-the-run-leaves-off-says-so) (the selected-check-off `warning:`), and any subcommand reporting a launch-time failure.

#### 2.2.4 Under `--format=json`

A *launch-time* `error:` (bad flag, unreadable config, missing path) is printed as raw text and is never JSON-ified; a *mid-scan* per-file failure collected by `grund check` is one of the report's findings and is rendered in `--format=json` like the others ([§FS-errors.5.2.3](FS-errors.md#523-run-level-findings-in-checks-report)), still on stderr because it is not a finding about the spec graph.

### 2.3 Bare query failure

When a subcommand established its query context but has no result to put on
stdout — an ID query on a missing ID, a missing section, an invalid ID under the
selected grammar, an ambiguous ID or section, or a broken stub; `grund refs` when
the selected resolver rejects an invalid ID or ambiguous number-only
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

A missing-target stub beside another home appends repair guidance after the complete
ambiguity prefix, as [§FS-declarations.stubs.repair](FS-declarations.md#stubsrepair-a-missing-target-beside-another-declaration-explains-the-repair) defines.
The appended explanation does not add, repeat, annotate, or reorder entries in JSON `sites`;
that array continues to represent the original ambiguity list.

#### 2.3.2 Who uses it

Used by ID queries (missing or invalid ID and missing section — [§FS-show.3](FS-show.md#3-outputs); ambiguous ID — [§FS-show.2.2.1](FS-show.md#221-ambiguous-id); ambiguous section — [§FS-show.2.2.2](FS-show.md#222-ambiguous-section); broken stub — [§FS-show.2.3.4](FS-show.md#234-broken-stub)), by `refs` resolver rejections after its compatibility window ([§FS-refs.4](FS-refs.md#4-exit-codes)), and by `grund id` for a satisfiable query context whose requested allocation has no result (empty slug — [§FS-id.3](FS-id.md#3-slug-derivation); proposed-ID collision — [§FS-id.5](FS-id.md#5-collision-check)), and by `grund config explain` for a target the version it answers in does not know ([§FS-config-explain.5](FS-config-explain.md#5-a-target-the-version-does-not-know)). `check` does not use this shape — every line it prints is a located finding (stdout) or a CLI-level message (stderr).

### 2.4 Text success line

A text-mode `grund check` run with zero retained errors and zero retained warnings, and no retained suggestion or unselectable run-level line ([§FS-check.2.1.3](FS-check.md#213-the-success-line)), prints exactly:

```
success
```

One trailing newline follows the line. The line is on **stdout** because it is the command's output ([§FS-errors.1.1](FS-errors.md#11-what-stdout-carries)), exits `0`, and appears only when the selected report is otherwise empty: it says that report is empty, not that the repository has no findings ([§FS-check.2.1.3](FS-check.md#213-the-success-line)). It is not emitted in `--format=json`, where stdout remains findings-only.

### 2.5 Refused input

When a command refuses an input the user typed because it is not in the
grammar — a rule sentence given to `check --rule`, a selector given to
`list --selector`, a coordinate glob given to `--glob` — the refusal names what
failed and, where it can, gives the input that is accepted when it is pasted back:

```
error: <reason>; accepted <form|selector|glob>: <input>
error: <reason>
```

- It is a CLI-level message ([§FS-errors.2.2](FS-errors.md#22-cli-level-message)): on **stderr**, with nothing on stdout,
  and exit `2` ([§FS-errors.2.2.2](FS-errors.md#222-exit-codes)).
- `<reason>` names the production that failed, and does not change with what
  follows it. After `; ` stands exactly one label and what it labels: a rule
  sentence after `accepted form:`, several quoted after `accepted forms:`, a
  selector after `accepted selector:`, a double-quoted glob after `accepted glob:`,
  and several joined with `or` after `accepted globs:` ([§FS-coordinate-globs.errors.shape](FS-coordinate-globs.md#errorsshape-one-refusal-line)).
  A label that ends in
  `after enabling it:` says the input is accepted only once the configuration
  enables what the reason names.
- Where nothing can be offered, the reason stands alone. Where what could not
  be supplied is a kind, the next line is `known kinds: <kinds>`, and a refusal
  that has a next step to name ends in one `hint:` line
  ([§FS-errors.1.2](FS-errors.md#12-what-stderr-carries)).
- A configured rule declaration refused for the same reason is a located
  finding ([§FS-errors.2.1](FS-errors.md#21-located-finding)), in the CLI and in the editor alike, whose message carries
  the same reason and accepted input after `is not a valid rule: ` and never the
  lines after it.

Which inputs each surface refuses, and how the accepted input is built from
what was typed, is [§FS-rule-refusals.sentences](FS-rule-refusals.md#sentences-strict-refusals) for rule sentences and [§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector)
for selectors.

#### 2.5.1 A kind that declares no IDs

A configured kind that is not citable ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)) is refused by every
surface that takes a kind name with one reason, built in one place from the
kind and its home, in the quoting that surface already uses: a backtick on
`list --kind` and `grund id`, a double quote on `list --selector` and on a
rule's subject and object.

```text
kind `<KIND>` declares no IDs — <home>/ is not a citable home
kind "<KIND>" declares no IDs — <home>/ is not a citable home
```

The verdict is [§FS-scan.projects.kinds](FS-scan.md#projectskinds-whose-kinds-a-run-reads)'s, read in the asking surface's scope,
and on the command line the next line is `known kinds: <kinds>`, naming the
citable kinds of that scope. A configured rule declaration's `invalid-rule`
finding carries the reason alone after `is not a valid rule: `
([§FS-errors.2.5](FS-errors.md#25-refused-input)). A `[citations]` target keeps the wording [§FS-config.3.9.5](FS-config.md#395-validation)
documents for it, *a non-citable target kind*.

`list --kind` and `grund id` print the first form ([§FS-list.1.1](FS-list.md#11---kind),
[§FS-id.1.1](FS-id.md#11-unknown-and-non-citable-kinds)). The selector and rule surfaces reach the second through the
window of [§FS-errors.3.8](FS-errors.md#38-the-one-grammar-wording-migration): until `0.19.0` they kept the reason they printed
before and appended this one, and since `0.19.0` they print it alone.

The homeless kind under its default name, `code`, in a project whose
`[[kinds]]` declares no homeless kind ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)), has no row, yet it
is the kind `[citations]` refuses as not citable, so every surface above
refuses it with this reason too. It has no home, so the reason ends at
`declares no IDs`:

```text
kind `code` declares no IDs
kind "code" declares no IDs
```

All four surfaces called it unknown before, `list --kind` included, so all
four reach it through the window of [§FS-errors.3.8](FS-errors.md#38-the-one-grammar-wording-migration).

## 3. Message text

The shape is structural; the text is human-readable. Style rules apply to every shape:

- **Lowercase first letter.** `unknown reference <ID>` — not `Unknown reference <ID>`.
- **No terminal period.** A single-clause message does not end in `.` or `!`. A run-level caution that runs to more than one clause, such as the empty-scan warning ([§FS-check.2.2.2](FS-check.md#222-the-message-names-the-likely-cause)), is written in sentences and punctuates each, its last included.
- **No ANSI colors yet.** Once colored output lands, the `[output] color` key, whose default is `auto` ([§FS-config.3.6](FS-config.md#36-output--report-format)), may add them ([§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path) applies); until then plain bytes are the contract.
- **Stable phrasing.** The exact text of each message is part of the user-visible output covered by [§GOAL-no-silent-breakage.1](../goals.md#1-what-counts-as-user-visible): changing it goes through a deprecation path. Tools grep on it. Appending to a message is not changing it: bytes added after text that survives as a verbatim contiguous prefix leave every prefix and `code` consumer reading what it read, so they ship in one release, and only an exact-line consumer is affected — whose migration is the stable `code` of [§FS-errors.3.6](FS-errors.md#36-the-agents-init-messages). [§FS-check.3.24.2](FS-check.md#3242-an-append-not-a-wording-change) is the worked case.
- **Quoted user input** appears in double quotes when the input could be confused with surrounding prose: `"<original title>"`, not `<original title>`.
- **One base for every path in the line** — [§FS-errors.3.1](FS-errors.md#31-one-base-for-every-path-in-the-line).

[§FS-errors.3.2](FS-errors.md#32-the-unknown-project-recovery-shape) to [§FS-errors.3.7](FS-errors.md#37-the-rule-site-unknown-alias-wording-migration) pin particular messages: the unknown-project recovery shape and its wording migration, `check`'s channel prefix, a missing fetch-backed declaration, `agents-init`, and the rule-site unknown alias.

### 3.1 One base for every path in the line

A path written *inside* the message text — a duplicate declaration's other homes, an ambiguous ID's competing sites, the stub a broken-stub refusal names — is a report path like the `<path>` the shape is located at, and is rendered against the same base ([§FS-config.3.6](FS-config.md#36-output--report-format)). In a workspace that base is the root the run reports from, never the member the finding came out of ([§FS-workspace.8.1](FS-workspace.md#81-grund-aliasid)): a line whose two halves are relative to two different roots sends the reader — and an editor following it — to a file that is not there. The path the message quotes back from the user's own text, such as a stub's link target, is not a resolved path and stays verbatim.

### 3.2 The unknown-project recovery shape

The unknown-project recovery shape in [§FS-check.3.8](FS-check.md#38-cross-project-citation-failure) freezes the base `unknown project alias <written>` and, when its first non-empty candidate band supplies alternatives, appends `; did you mean <a>?`, `; did you mean <a> or <b>?`, or `; did you mean <a>, <b> or <c>?`. The base begins lowercase and has no period; the recovery clause has one terminal question mark. Text output carries the whole message, and JSON retains `code: "unknown-project"` while carrying the same bytes in `message` ([§FS-errors.5.1](FS-errors.md#51-on-stdout--the-commands-output)).

### 3.3 The narrowed-run unknown-project wording migration

The narrowed-run scope-only unknown-project message took a three-release wording
migration, which ended in `0.16.0`: its final template is fixed in
[§FS-check.3.8.4](FS-check.md#384-the-scope-only-message-across-three-releases), and the `0.13.2` compatibility suffix — part of the error
message, never a second warning finding — is gone. Exact-line consumers were
asked to migrate during the window to the stable `code == "unknown-project"`;
the code, error severity, sites, selectors, and exit verdict did not change.
Workspace-root candidate messages and bare unknown-project messages were
unchanged throughout.

### 3.4 The `check` channel prefix

For `grund check`, the fixed rule supplies the channel and every located text
line makes it explicit after the location prefix: [§FS-check.3](FS-check.md#3-errors-detected) is `error:`, [§FS-check.4](FS-check.md#4-warnings) is `warning:`, and enabled [§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in) advisories are `suggestion:`. That structural prefix does not change the finding's message bytes. JSON continues to carry the same distinction in its existing `severity` or `channel` field ([§FS-errors.5.1](FS-errors.md#51-on-stdout--the-commands-output)); `fmt`, `refs`, run-level messages, and LSP retain their existing shapes.

### 3.5 A missing fetch-backed declaration

A missing fetch-backed declaration has two frozen identities, `dangling` /
`error` ([§FS-check.3.1](FS-check.md#31-dangling-citation)) and
`missing-snapshot` / `warning` ([§FS-check.4.12](FS-check.md#412-missing-snapshot)); those sections own the exact
message bytes, the bare-text fetch remedy, and the precedence of an existing
near-ID or illustration hint over the fetch-action tail.

### 3.6 The `agents-init` messages

The five `agents-init` messages took a three-release migration, which ended in
`0.16.0`. In the first two releases each existing message stayed a verbatim
contiguous prefix and gained a fixed tail classifying it as repo maintenance,
saying the citation checks still ran, and naming `0.16.0` as the release its
wording would change in. Exact-line consumers were asked to migrate during that
window to the stable `code == "agents-init"`; code, error severity, and the
default exit verdict did not change. The messages are now the final templates
of [§FS-errors.3.6.1](FS-errors.md#361-the-final-templates).

#### 3.6.1 The final templates

Since `0.16.0`, with the compatibility tail removed, the templates are these five, and a sixth since the check compares the whole block ([§FS-check.3.5.5](FS-check.md#355-the-workspace-sections-are-judged-at-the-outermost-root-of-a-full-checkout), [§DF-check-compares-the-whole-block](../decisions/functional/DF-check-compares-the-whole-block.md#df-check-compares-the-whole-block-grund-check-holds-the-whole-managed-block-the-workspace-sections-at-the-outermost-root-of-a-full-checkout)):

```text
repo maintenance: malformed grund managed block: <detail> (does not affect citation validity)
repo maintenance: outdated grund init block v<found> — run `grund init` to update to v<current> (does not affect citation validity)
repo maintenance: unsupported grund init block v<found> — this grund supports v<current> (does not affect citation validity)
repo maintenance: stale grund init block: the <section> section differs from grund.toml — run `grund init` to refresh (does not affect citation validity)
repo maintenance: missing grund init block v<current> — run `grund init` to install it (does not affect citation validity)
repo maintenance: stale grund init block: the <section> section differs from the workspace (<detail>) — run `grund init` in <dir> to refresh (does not affect citation validity)
```

`<found>` is the version the block's heading carries. `<current>` is the version the live config selects in `outdated` and `missing`, and in `unsupported` the maximum this binary supports, the number `grund init`'s refusal of the same block names, never the config's selection ([§FS-init.2.3.7.3.1](FS-init.md#23731-above-the-maximum-unsupported)). `<section>` is one of the nouns of [§FS-check.3.5.3](FS-check.md#353-code-and-message), and the sixth template carries the two workspace sections. Both stale templates name the section as one subject, `the <section> section`, so the sentence reads the same whichever heading it names, a heading someone wrote into the block included. The fourth template was corrected once, in 0.20.0, from `<section> differ from grund.toml`, which a singular section such as `scoped guidance` could not agree with; only its leading `repo maintenance: stale grund init block: ` was kept ([§DF-stale-section-differs-in-0-20](../decisions/functional/DF-stale-section-differs-in-0-20.md#df-stale-section-differs-in-0-20-the-stale-block-finding-names-its-section-as-one-subject-corrected-in-place-in-0200)). `<dir>` is the root of the project that owns the block, from the finding's own base; where that project is the one the run started in, ` in <dir>` is left out. The first five templates are held stable, so where they report another project's block they grow at the end rather than change in the middle — `outdated`, `stale … differs from grund.toml` and `missing` each append `; run it in <dir>` after their last byte:

```text
packages/billing/AGENTS.md:3: error: repo maintenance: outdated grund init block v35 — run `grund init` to update to v39 (does not affect citation validity); run it in packages/billing
```

`malformed` and `unsupported` name no command and append nothing. The sixth template's `<detail>` is the first differing entry or section fallback defined by [§FS-check.3.5.3.1](FS-check.md#3531-first-differing-workspace-entry); it does not change the five stable templates. For example:

```text
AGENTS.md:3: error: repo maintenance: stale grund init block: workspace members differ from the workspace (expected - [`billing`](packages/billing/); found - [`billing`](packages/billing/) *(not yet initialized)*) — run `grund init` to refresh (does not affect citation validity)
```

These are message classifications only: `repo maintenance` is not a finding
category or selector value ([§FS-check.1](FS-check.md#1-inputs)).

### 3.7 The rule-site unknown-alias wording migration

A rule object kind the run holds no workspace vocabulary to resolve is reported
at the rule's own heading as `invalid-rule` ([§FS-rules.4.1](FS-rules.md#41-a-rule-this-scope-cannot-verify)). Until the correcting
release its reason named the kind as the defect — `unknown kind "<KIND>" in
namespace "<ALIAS>"` for a pinned object, `unknown kind "<KIND>" in any
workspace namespace` for `*/<KIND>` — in a scope that cannot know whether that
kind exists anywhere. What such a run actually cannot reach is the workspace.
Those are bytes a consumer may match, so the correction took the three-release
route of [§FS-errors.3.3](FS-errors.md#33-the-narrowed-run-unknown-project-wording-migration) and [§FS-errors.3.6](FS-errors.md#36-the-agents-init-messages) rather than changing outright
([§REQ-backwards-compatibility.1](../requirements/REQ-backwards-compatibility.md#1-what-is-covered), [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)).

Until `0.16.0` each complete legacy reason stayed a verbatim contiguous
prefix and gained the true clause after it, followed by a suffix
naming `0.16.0` as the release its wording would change in. In `0.16.0` the
prefix and the suffix went, and the reason is the true clause alone, in the
shape a narrowed run already uses for a citation it cannot place — say what
cannot be resolved here, offer no candidate, send the reader to the workspace
root ([§FS-check.3.8.3](FS-check.md#383-a-narrowed-run-offers-no-candidate)). Its templates are [§FS-errors.3.7.1](FS-errors.md#371-the-final-templates).

Exact-line consumers were asked to migrate during the window to the stable
`code == "invalid-rule"`; the code, the error severity, the selectors, and the
exit verdict did not change, and `--only invalid-rule` keeps selecting the
finding. An alias the run *can* judge and rejects keeps its legacy reason
unchanged, because there the kind really is what failed.

The reason belongs to the vocabulary check rather than to the finding, so the
same two forms also reach `check --rule`'s pre-scan refusal
([§FS-rules.4](FS-rules.md#4-validation-lifecycle)), where there is no rule heading to report at and therefore no
`code` to migrate to: that surface prints `error: <reason>` on stderr, writes
nothing to stdout, and exits 2. A consumer of it keys on the exit code, which
this migration did not move, and on the sentence it passed in.

#### 3.7.1 The final templates

Since `0.16.0`, with the compatibility prefixes removed, the two reasons are
exactly:

```text
unknown project alias <ALIAS>; no workspace is in scope here, so the alias cannot be resolved — check from the workspace root
no workspace is in scope here, so no namespace can be searched for <KIND> — check from the workspace root
```

### 3.8 The one-grammar wording migration

Two refusals changed the reason they print, and both are bytes a consumer may
match, so they took the warning release of [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)
rather than changing outright ([§DF-one-answer-rulings-grammar](../decisions/functional/DF-one-answer-rulings-grammar.md#df-one-answer-rulings-grammar-one-grammar-keeps-the-documented-refusal-bytes-and-order-and-one-kind-universe-keeps-each-commands-vocabulary), D10):

- A configured kind that is not citable, named on `list --selector`,
  `list --size --selector`, or in a rule's subject or object on `check`,
  `check --rule` or a configured rule declaration, was refused as though it
  were not configured: `unknown kind "<KIND>"`, or, for a selector spelled
  `KIND` or `KIND.NAME`, `literal subject "<selector>" does not match the
  configured ID grammar`. Its reason is §FS-errors.2.5.1's. So was `code`,
  the homeless kind's default name, in a project whose `[[kinds]]` declares no
  homeless kind, on those surfaces and on `list --kind`, where the legacy
  reason is ``unknown kind `code` ``. Its reason is [§FS-errors.2.5.1](FS-errors.md#251-a-kind-that-declares-no-ids)'s form
  with no home ([§DF-one-answer-rulings-grammar](../decisions/functional/DF-one-answer-rulings-grammar.md#df-one-answer-rulings-grammar-one-grammar-keeps-the-documented-refusal-bytes-and-order-and-one-kind-universe-keeps-each-commands-vocabulary), G09).
- A subject with a section-component wildcard in a sentence that joins clauses
  with `and` was refused as a conjunction. Under the order of
  [§FS-rule-refusals.sentences.order](FS-rule-refusals.md#sentencesorder-a-sentence-is-refused-for-the-first-stage-it-fails) the wildcard is refused first.

Its window opened in `0.18.0` and closed in `0.19.0` (#881): until then each
complete legacy reason, accepted forms included, stayed a verbatim contiguous
prefix — `unknown kind "e2e"`, `literal subject "e2e.terms" does not match the
configured ID grammar`, or the conjunction refusal with its accepted forms —
followed by ` — `, the reason it was moving to, and the suffix `; this wording
changes in grund 0.19.0`. In `0.19.0` the prefix and the suffix went, and each
refusal prints [§FS-errors.3.8.1](FS-errors.md#381-the-final-templates)'s reason alone ([§DF-one-grammar-lands-in-0-19](../decisions/functional/DF-one-grammar-lands-in-0-19.md#df-one-grammar-lands-in-0-19-the-one-grammar-window-closes-in-0190-and-a-pinned-target-the-run-cannot-accept-fails-the-load)).
Whatever followed the legacy reason on the next line — `known kinds:`, a
`hint:` — still follows the reason. The exit code, the `invalid-rule` code and
its severity, and what is refused did not move; only the reason's text did. In
a repository whose kinds are `FS`, `RULE` and the non-citable `e2e` homed at
`tests/e2e/`, with named sections on, these are the exact lines:

| Input | Exact reason |
|---|---|
| `list --selector e2e.terms` | `kind "e2e" declares no IDs — tests/e2e/ is not a citable home` |
| `list --selector 'Each e2e'` | `kind "e2e" declares no IDs — tests/e2e/ is not a citable home` |
| `check --rule 'Each e2e must have exactly one terms chapter.'` | `kind "e2e" declares no IDs — tests/e2e/ is not a citable home` |
| `check --rule 'Each FS must cite at least one e2e.'` | `kind "e2e" declares no IDs — tests/e2e/ is not a citable home` |
| `check --rule 'FS-x.* must cite at least one FS and must not cite any FS.'` | `section-component wildcards are not accepted in phase 1` |

The first four are each followed by `known kinds: FS, RULE`, as before. The
fifth is followed by nothing: once its subject is rebuilt the sentence is still
a conjunction, so no accepted form is offered ([§FS-errors.3.8.1](FS-errors.md#381-the-final-templates)). Exact-line
consumers were asked to migrate during the window to the reason after the
` — `, which is what `0.19.0` prints; a matcher of the legacy prefix no longer
finds it.

The same repository declares no homeless kind, so `code` has no row there.
Its window opened in `0.18.0` and closed in `0.19.0` (#680): each of these
refusals printed its legacy reason in front — `unknown kind "code"`,
``unknown kind `code` `` on `list --kind`, or `literal subject "code" does not
match the configured ID grammar` on a bare selector — then ` — `, the reason
below and the suffix. Each now prints [§FS-errors.3.8.1](FS-errors.md#381-the-final-templates)'s form with no home
alone, followed by `known kinds: FS, RULE`, as before, with exit `2`
([§DF-homeless-code-declares-no-ids](../decisions/functional/DF-homeless-code-declares-no-ids.md#df-homeless-code-declares-no-ids-code-with-no-row-is-refused-as-declaring-no-ids-alone-from-0190)):

| Input | Exact reason |
|---|---|
| `list --kind code` | ``kind `code` declares no IDs`` |
| `list --selector 'Each code'` | `kind "code" declares no IDs` |
| `list --selector code` | `kind "code" declares no IDs` |
| `check --rule 'Each code must have exactly one terms chapter.'` | `kind "code" declares no IDs` |
| `check --rule 'Each FS must cite at least one code.'` | `kind "code" declares no IDs` |

Retiring the window was [§RM-one-grammar-flip](../roadmap.md#rm-one-grammar-flip-end-the-one-grammar-window-in-0190).

#### 3.8.1 The final templates

Since `0.19.0`, with the legacy prefix and the suffix gone, the reasons are exactly
[§FS-errors.2.5.1](FS-errors.md#251-a-kind-that-declares-no-ids)'s double-quoted form and the wildcard reason of
[§FS-rule-refusals.sentences](FS-rule-refusals.md#sentences-strict-refusals):

```text
kind "<KIND>" declares no IDs — <home>/ is not a citable home
section-component wildcards are not accepted in phase 1
```

`code` with no row prints the form with no home, `kind "code" declares no IDs`,
and ``kind `code` declares no IDs`` on `list --kind`; its window has already
closed (#680).

The wildcard reason is then followed by the accepted form
[§FS-rule-refusals.sentences.rewrite](FS-rule-refusals.md#sentencesrewrite-an-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced) builds, where it builds one; for a
sentence that is still a conjunction once its subject is rebuilt it builds
none ([§FS-rule-refusals.sentences.rewrite.reparsed](FS-rule-refusals.md#sentencesrewritereparsed-a-form-is-parsed-before-it-is-offered)).

## 4. Determinism

Two runs of the same subcommand on the same input must produce byte-identical stdout *and* stderr ([§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)). This rules out:

- Wall-clock timestamps in messages.
- Process IDs, hostnames, or — in the reports of the tree-reading commands
  [§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes) names — an absolute path or one that escapes the loaded
  root other than the `..` path of a config above it that the run had to read, with
  no absolute fallback for an in-root target ([§FS-config.3.6](FS-config.md#36-output--report-format),
  [§DF-cli-base-parent-paths.2](../decisions/functional/DF-cli-base-parent-paths.md#2-decision)).
- Non-deterministic ordering — the order each report keeps is [§FS-errors.4.1](FS-errors.md#41-ordering).
- Platform-native path separators in repo-relative output. Any path that appears in a report, JSON field, e2e case manifest, duplicate-site list, stub-link note, or formatter summary is rendered with `/`, so Windows and Unix runs over the same tree compare byte-for-byte.

How `check --only` and `--ignore` keep the contract is [§FS-errors.4.2](FS-errors.md#42-finding-selection).

### 4.1 Ordering

Each completed engine check report stably sorts its `errors`, `warnings`, and
`suggestions` channels bytewise by `(path, line, message)`, after path narrowing
and caution insertion. An absent path sorts first; an absent line counts as zero;
ties retain their existing order. Finding selection retains this order, with
selection-generated `selected-check-off` advice appended afterwards
([§FS-check.1.4.1](FS-check.md#141-a-selected-check-the-run-leaves-off-says-so)). The compatibility decision is [§DF-completed-check-report-order](../decisions/functional/DF-completed-check-report-order.md#df-completed-check-report-order-completed-check-reports-share-one-order).

Text `check` findings are grouped as errors,
warnings, then enabled suggestions and sort bytewise by `(path, line,
message)` within each group. JSON `check` findings retain their global
bytewise `(path, line, message)` order across channels. Other reports retain
their existing documented order; multi-site findings are located at the
lexicographically-first site ([§FS-check.2.1](FS-check.md#21-report-format)).

A message that would otherwise be non-deterministic (e.g. the order of duplicate-declaration sites) is sorted before printing.

Rule findings obey this same ordering without a rule-specific sorter. In a
same-anchor `cite each` group, the fixed message puts the target ID immediately
after `<subject> cites ` and before its count, so bytewise message ordering also
orders shared-prefix target IDs bytewise
([§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits)).

### 4.2 Finding selection

`grund check --only` and `--ignore` preserve [§FS-errors.4](FS-errors.md#4-determinism)'s contract: selection precedes
the format-specific sort and rendering, retained findings keep their
locations, messages, codes, sites, and channels, and reordering or duplicating
selector flags cannot alter the result ([§FS-check.2.1](FS-check.md#21-report-format)).

## 5. JSON format

The subcommands with a machine-readable result or finding surface accept `--format=json`: `check`, `show`, `list`, `refs`, `cover`, `id`, and `integrations` ([§GOAL-friendliness-first.1](../goals.md#1-hard-requirements), [§FS-cli.3](FS-cli.md#3-cross-subcommand-flags), [§FS-integrations.5](FS-integrations.md#5-json-format)). Operational commands whose output is human text or generated files (`fmt`, `fetch`, `init`, `config`, `guide`, `completions`) do not accept `--format` unless their own spec adds a JSON surface later. JSON follows the same stream split as the text form ([§FS-errors.1](FS-errors.md#1-streams)): what goes to stdout is [§FS-errors.5.1](FS-errors.md#51-on-stdout--the-commands-output), what goes to stderr is [§FS-errors.5.2](FS-errors.md#52-on-stderr--what-is-not-output), and `show --batch` is the query-stream exception of [§FS-errors.5.3](FS-errors.md#53-show---batch).

So `grund check --format=json | jq …`, `grund <ID> --format=json | jq …`, `grund list --format=json | jq …` all work with no stream juggling, and `grund <missing> --format=json | jq …` does not choke because the finding is on stderr where the pipe does not see it.

The text-form messages defined above remain the default. JSON is opt-in.

Value findings are [§FS-errors.5.4](FS-errors.md#54-value-findings); the `code` catalog `check` selects on is [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog).

### 5.1 On stdout — the command's output

`grund check --format=json` emits its findings as NDJSON, one object per line, in the binding-level shape from [§FS-distribution.3.0](FS-distribution.md#30-language-neutral-data-shapes) (`{ severity, path, line, code, message, sites, authority }`); `severity` carries the `error`/`warning` distinction as a structured field, while text carries the explicit channel prefix after its location prefix ([§FS-errors.3.4](FS-errors.md#34-the-check-channel-prefix)), and `sites` is `null` for an ordinary single-site finding, a `[{ path, line }]` list naming every site for a multi-site finding (a duplicate declaration, [§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration)). `authority` is the last key and follows `sites` in the same shape: `null` for a finding no chapter rule authored, or a bytewise-sorted list of the rule origins that did ([§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits)) — `["RULE-security"]` for one declared rule, `["--rule"]` for an ad-hoc trial sentence, `["--rule","RULE-security"]` where both reached the same meaning ([§FS-rules.6](FS-rules.md#6-semantic-deduplication)). It is what lets a caller ask *which rule said this* without matching message text, and it is the field `check --only-rule` queries ([§FS-rules.8](FS-rules.md#8-command-surfaces)). A clean JSON check emits no `success` object. A **suggestion** ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)), emitted only under `grund check --suggestions`, carries `"channel": "suggestion"` in place of a `severity` — keeping the frozen `{error, warning}` severity set ([§FS-config.6](FS-config.md#6-what-is-not-configured-here)) intact, so a consumer filtering on `severity` never sees one. Query subcommands' results are [§FS-errors.5.1.1](FS-errors.md#511-query-results).

#### 5.1.1 Query results

Query subcommands emit their result on stdout too: one JSON object for a single-result command (`grund <ID> --format=json` — [§FS-show](FS-show.md#fs-show-grund-reads-a-single-declaration-body-by-id); `grund id --format=json` — [§FS-id](FS-id.md#fs-id-grund-proposes-ids-for-new-declarations)), NDJSON — one object per row — for a list command (`grund list` per declaration, `grund refs` per citation, `grund cover` per scanned file).

### 5.2 On stderr — what is not output

A *failed ID query* (`ID not found` / `ambiguous` / `broken stub` / `section not found` / `invalid ID`, exit `1`) emits its one finding object on stderr in the same `{ severity, path, line, code, message, sites, authority }` shape, with `path` and `line` `null` — there is no single site, and there is no result, so nothing goes to stdout. `authority` is present and always `null` there: no query failure is a chapter rule's, and the key set is one set across every record of this shape rather than a conditional a consumer has to branch on. This includes `refs`' invalid-ID and ambiguous-number-only rejections; their codes are respectively `invalid-id` and `ambiguous`, both carry `sites:null`, and neither carries the text-mode hint.

Which ambiguity refusals carry `sites` is [§FS-errors.5.2.1](FS-errors.md#521-sites-on-an-ambiguity-refusal); the messages that stay raw text under any `--format` are [§FS-errors.5.2.2](FS-errors.md#522-launch-time-messages-stay-text); run-level findings in `check`'s report are [§FS-errors.5.2.3](FS-errors.md#523-run-level-findings-in-checks-report).

#### 5.2.1 `sites` on an ambiguity refusal

`sites` carries the `[{ path, line }]` list the message names, the same pairs in the same order, for an `ambiguous` refusal naming an ID with two homes ([§FS-show.2.2.1](FS-show.md#221-ambiguous-id)) and for an `ambiguous-section` refusal ([§FS-show.2.2.2](FS-show.md#222-ambiguous-section)). The number-only shorthand's `ambiguous` refusal names candidate IDs rather than sites ([§FS-show.2.2.1](FS-show.md#221-ambiguous-id)), so — like every other query failure — it carries `sites: null`; a consumer tells the two `ambiguous` shapes apart by whether `sites` is `null`, not by `code`.

#### 5.2.2 Launch-time messages stay text

A *launch-time* CLI-level message ([§FS-errors.2.2](FS-errors.md#22-cli-level-message)) — an error such as a bad flag, unknown kind, unknown project alias, unreadable config or path, or a config the workspace refuses such as [§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan)'s (exit `2`), or a warning carried in the run's warning channel, such as [§FS-check.4.10](FS-check.md#410-include_root--false-leaves-the-blocks-own-files-unread)'s and [§FS-workspace.6.1.7.6](FS-workspace.md#6176-how-the-undecidable-claim-warning-travels)'s — stays as its `error:` / `warning:` text line on stderr regardless of `--format`. What decides is which channel carries the fact: a warning settled from the loaded config but carried as one of `check`'s report warnings, such as [§FS-check.4.3](FS-check.md#43-redundant-config-pair)'s and [§FS-check.4.11](FS-check.md#411-config-read-from-the-deprecated-agents-location)'s, is data, and renders as a JSON finding ([§FS-errors.5.2.3](FS-errors.md#523-run-level-findings-in-checks-report)).

#### 5.2.3 Run-level findings in `check`'s report

A run-level finding in `grund check`'s report — about the run, not a finding about the graph, such as the empty-scan `warning:` ([§FS-check.2.2](FS-check.md#22-empty-scan)), the nothing-recognized `warning:` ([§FS-check.4.5](FS-check.md#45-nothing-recognized)), or a per-file read failure collected mid-scan (a `line`-less finding) — is likewise on stderr in both forms; under JSON a run-level warning carries `path`, `line`, `sites`, and `authority` all `null`, any location being in its message text ([§FS-check.4.5](FS-check.md#45-nothing-recognized)). Two are the exceptions. A rule's line saying it applies to nothing is a diagnostic its rule produced, so it carries that rule's `authority` and a run scoped to the trial sentence keeps the sentence's line ([§FS-check.2.2.1.4](FS-check.md#2214-a-rules-obligation)); and the selected-check-off line is about the selection itself, so it passes both axes ([§FS-check.1.4.1](FS-check.md#141-a-selected-check-the-run-leaves-off-says-so)). Any other run-level finding is about the run rather than about a unit, so no rule authored it and `--only-rule` cannot retain it. A scoped run therefore drops it along with everything else no rule authored, exactly as the code axis drops it — `--ignore empty-scan` by name, `--only <any other code>` by omission — and only a mid-scan failure's `io` finding is beyond either axis ([§FS-check.2.1.2](FS-check.md#212-selection-filters-the-complete-report)). Say the consequence plainly, because this is where a scoped trial can still mislead: an author who scopes a sentence over a tree that scanned nothing gets `success` at exit `0`, and the warning that would have said *nothing was scanned, so the sentence never got a chance* is gone with the rest. The `success` there means what it always means — the selected report is empty, not that the repository has no findings ([§FS-check.2.1.3](FS-check.md#213-the-success-line)). The exit code is unmoved either way, these being warnings.

### 5.3 `show --batch`

`show --batch --format=json` is the explicit query-stream exception. Every
well-formed query, including a failed one, produces one ordered stdout envelope
([§FS-output-shapes.4.1](FS-output-shapes.md#41-show---batch---formatjson)); stderr
is reserved for invocation, batch-input, configuration, and scan failures that
abort the run. A batch-input error uses raw
`error: batch input line <N>: <reason>` text because no data-producing phase
began. The entire stream is validated before scanning, so this error leaves
stdout empty and produces no partial records.

### 5.4 Value findings

Value findings use the same object and streams. `invalid-value-declaration`, `invalid-value-binding`, and `value-mismatch` are fixed error codes; a mismatch's `sites` is the sorted declaration-site array, and its `message` is byte-identical to the text message after the primary `path:line:` prefix and its channel prefix ([§FS-errors.2.1](FS-errors.md#21-located-finding), [§FS-values.5](FS-values.md#5-resolution-findings-and-exit-status)). The `invalid-value-binding` that refuses a root-aimed binding for a space carries the offending component's declaration site as its one `sites` entry, the site its text names ([§FS-values.3.1.2](FS-values.md#312-a-binding-aimed-at-the-root)), so text and NDJSON name the same component. Home JSON input that [§FS-values.5.3](FS-values.md#53-incomplete-input-and-deterministic-output) counts as incomplete remains a run-level incomplete-scan failure at exit `2` rather than a semantic value finding.

### 5.5 The `check` code catalog

For `grund check`, `code` is also the exact public selector vocabulary for
[§FS-check.1](FS-check.md#1-inputs), apart from a row marked planned. The catalog is one table sorted by code, and one row is the whole of what the
catalog says about a code: what it means, the severity it carries, the release a ramp still
ahead of it changes that severity in, what has to be configured or passed for it
to fire at all, and the specification section that specifies it. Severity lives
here rather than in the section that specifies the check, so promoting a warning
to an error edits this cell and moves no coordinate
([§REQ-spec-section-names.code](../requirements/REQ-spec-section-names.md#code-a-check-is-named-by-its-diagnostic-code)). A code whose row names `—` under *Enabled by* fires on every
run; `Ramp` is `—` where no promotion is promised, a ramp already spent included.
The rows are what the installed `grund` says about a code, so how each cell is
written and read is [§FS-errors.5.5.1](FS-errors.md#551-how-a-row-is-read).

| Code | Meaning | Severity | Ramp | Enabled by | Check |
|---|---|---|---|---|---|
| `agents-init` | An agent entrypoint such as `AGENTS.md` lacks the `grund init` block it needs, or carries one this grund does not accept. | error | — | — | [§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block) |
| `broken-stub` | A stub's link does not lead to a declaration of its ID. | error | — | — | [§FS-declarations.checks.broken-stub](FS-declarations.md#checksbroken-stub-broken-inline-spec-stub) |
| `chapter-cardinality` | A declaration has more or fewer chapters of a name than a rule's `have` sentence allows. | error, or a suggestion for a `should` or `should not` rule | — | v2: `form = "rule"` is set on a `[schema.kinds]` table or `--rule` is passed · v1: `rules = true` is set on a `[[kinds]]` row or `--rule` is passed | [§FS-check.3.26](FS-check.md#326-chapter-cardinality) |
| `citation-cardinality` | A declaration cites a target more or fewer times than a rule allows. | error, or a suggestion for a `should` or `should not` rule | — | v2: `form = "rule"` is set on a `[schema.kinds]` table or `--rule` is passed · v1: `rules = true` is set on a `[[kinds]]` row or `--rule` is passed | [§FS-check.3.27](FS-check.md#327-citation-cardinality) |
| `config-version-1` | A config file is read as version 1; the warning names the command that migrates it. | v2: not in version 2 · v1: warning | — | v2: not in version 2 · v1: a config file is read as version 1 | [§FS-check.checks.config-version-1](FS-check.md#checksconfig-version-1-a-config-file-read-as-version-1) |
| `conflict-marker` | A scanned file holds an unresolved merge-conflict block. | error | — | — | [§FS-declarations.checks.conflict-marker](FS-declarations.md#checksconflict-marker-unresolved-merge-conflict) |
| `dangling` | A citation names an ID that no declaration declares. | error | — | — | [§FS-check.3.1](FS-check.md#31-dangling-citation) |
| `declaration-near-miss` | A heading opens like a declaration, but its ID does not match its kind's ID format. | error | — | — | [§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss) |
| `deprecated-config-location` | The config was read from `.agents/grund.toml`, a location that still works but is deprecated. | warning | — | — | [§FS-check.4.11](FS-check.md#411-config-read-from-the-deprecated-agents-location) |
| `discouraged-citation` | A citation goes to a target that a `should-not` entry or rule discourages. | none — suggestion | — | v2: `--suggestions` is passed and `[rules.citations.<kind>] should-not` or a `should not` rule is set · v1: `--suggestions` is passed and `[citations.<kind>] should-not` or a `should not` rule is set | [§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in) |
| `duplicate` | The same ID is declared more than once. | error | — | — | [§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration) |
| `duplicate-section` | Two section headings in one declaration claim the same section path. | error | — | — | [§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path) |
| `empty-citation-obligation` | A `must` or `should` citation obligation applies to a kind that has no unit for it to judge. | warning | — | v2: `[rules.citations.<kind>] must` or `should`, or a `must` or `should` rule, is set for a scanned kind with a folder home · v1: `[citations.<kind>] must` or `should`, or a `must` or `should` rule, is set for a scanned kind with a folder home | [§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing) |
| `empty-scan` | The scan read no files at all, which usually means its scope is misconfigured. | warning | — | — | [§FS-check.2.2](FS-check.md#22-empty-scan) |
| `escaped-citation-resolves` | An escaped citation names an ID or section that exists, so it may be a live citation escaped by mistake. | none — suggestion | — | `--suggestions` is passed | [§FS-check.2.3.1](FS-check.md#231-escaped-citation-resolves) |
| `excluded-stub-target` | A stub led to a file the scan excludes; that case is now reported as a broken stub. | none — spent from grund 0.19.0; select `broken-stub` instead | — | — | [§FS-declarations.checks.excluded-stub-target](FS-declarations.md#checksexcluded-stub-target-stub-to-a-target-the-walk-excludes-broken-from-grund-0190) |
| `forbidden-citation` | A citation goes to a target that a `must-not` entry or rule forbids. | v2: by strength: `must-not` an error, `warn-not` a warning · v1: error | — | v2: `[rules.citations.<kind>] must-not` or a `must not` rule is set; a `warn-not` entry turns it on too, as a warning · v1: `[citations.<kind>] must-not` or a `must not` rule is set | [§FS-check.3.12](FS-check.md#312-forbidden-citation) |
| `full-scope-ignored` | `--full` was given an explicit path, which it cannot widen, so it changed nothing. | warning | — | `--full` is passed; with an explicit path | [§FS-check.1.3.7](FS-check.md#137-a-path-the-flag-cannot-widen-earns-a-caution-not-a-refusal) |
| `glob-citation` | A marked citation is written as a pattern rather than as one ID. | error | — | — | [§FS-check.checks.glob-citation](FS-check.md#checksglob-citation-glob-citation) |
| `image-index-entry` | A citation-only image was all a kind's index held for an ID; such an image is now neither an entry nor an enrollment. | none — spent from grund 0.20.0; select `missing-index-entry` instead | — | — | [§FS-citations.checks.image-index-entry](FS-citations.md#checksimage-index-entry-a-citation-only-image-is-an-index-entry-for-one-more-release) |
| `inline-citation-style` | A citation's inline note in a code comment breaks the configured note style: too many lines, too wide, the wrong layout, or prose where only the citation is allowed. | v2: by strength: the `[schema.notes.lines]`, `[schema.notes.columns]`, `[schema.notes.layout]` or `[schema.notes.text]` key it breaks · v1: error; a warning for an overrun of `[reference] inline_note_suggested_lines` under `warn_on_suggested = true`, or for a layout deviation under `inline_note_layout_check = "warn"` | — | — | [§FS-check.3.10](FS-check.md#310-inline-citation-style-violation) |
| `invalid-field-value` | A field's value is not one of the values its kind admits for it. | v2: by strength: the field's `presence` · v1: not in version 1 | — | v2: `[schema.kinds.<kind>.fields.<field>] one_of` is set · v1: not in version 1 | [§FS-fields.checks.invalid-field-value](FS-fields.md#checksinvalid-field-value-invalid-field-value) |
| `invalid-rule` | A rule declaration cannot be read: its title, rationale, vocabulary or subject is invalid. | error | — | v2: `form = "rule"` is set on a `[schema.kinds]` table or `--rule` is passed · v1: `rules = true` is set on a `[[kinds]]` row or `--rule` is passed | [§FS-check.3.25](FS-check.md#325-invalid-rule) |
| `invalid-section-lock` | A line of a `.grund/sections/` lock file is not an entry grund can read. | error | — | a `.grund/sections/` lock file exists | [§FS-section-lock.checks.invalid-section-lock](FS-section-lock.md#checksinvalid-section-lock-a-lock-line-grund-cannot-read) |
| `invalid-value-binding` | A value binding breaks the binding grammar or aims at something it cannot bind. | error | — | v2: `form = "value"` is set on a `[schema.kinds]` table or `content = "values"` on one of its fields; a value tag the scan finds turns it on too · v1: `values = true` or `value_chapter` is set on a `[[kinds]]` row; a value tag the scan finds turns it on too | [§FS-check.3.21](FS-check.md#321-invalid-value-binding) |
| `invalid-value-declaration` | A declaration of a kind that holds values is not a valid value. | error | — | v2: `form = "value"` is set on a `[schema.kinds]` table or `content = "values"` on one of its fields; a value tag the scan finds turns it on too · v1: `values = true` or `value_chapter` is set on a `[[kinds]]` row; a value tag the scan finds turns it on too | [§FS-check.3.20](FS-check.md#320-invalid-value-declaration) |
| `io` | A file or directory could not be read, so the run's view of the tree is incomplete. | error | — | — | [§FS-check.2.4](FS-check.md#24-an-incomplete-run) |
| `local-section-citation` | A marker followed by a bare section number is not a citation grund can resolve; write the full ID. | error | — | — | [§FS-check.3.24](FS-check.md#324-declaration-local-section-citation) |
| `misplaced-declaration` | A declaration sits outside the home its kind is configured to live in. | error | — | v2: `folders` or `files` is set on a `[schema.kinds]` table · v1: `folder` or `file` is set on a `[[kinds]]` row | [§FS-declarations.checks.misplaced-declaration](FS-declarations.md#checksmisplaced-declaration-misplaced-declaration-configured-kind-home) |
| `misplaced-field` | A field does not stand where its kind places it. | v2: by strength: the field's `presence` · v1: not in version 1 | — | v2: `[schema.kinds.<kind>.fields.<field>] position` or `after` is set · v1: not in version 1 | [§FS-fields.checks.misplaced-field](FS-fields.md#checksmisplaced-field-misplaced-field) |
| `missing-citation` | A declaration lacks a citation that a `must` entry or rule requires. | v2: by strength: `must` an error, `warn` a warning · v1: error | — | v2: `[rules.citations.<kind>] must` or a `must` rule is set; a `warn` entry turns it on too, as a warning · v1: `[citations.<kind>] must` or a `must` rule is set | [§FS-check.3.11](FS-check.md#311-missing-required-citation) |
| `missing-field` | A declaration lacks a field its kind declares. | v2: by strength: the field's `presence` · v1: not in version 1 | — | v2: `[schema.kinds.<kind>.fields.<field>] presence = "must"` is set; a `warn` or `should` `presence` turns it on too, at that strength · v1: not in version 1 | [§FS-fields.checks.missing-field](FS-fields.md#checksmissing-field-missing-field) |
| `missing-index-entry` | A declaration in a kind's folder is not listed in that folder's index. | error | — | — | [§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index) |
| `missing-section` | A citation names a section its declaration does not have. | error | — | — | [§FS-check.3.2](FS-check.md#32-missing-section) |
| `missing-snapshot` | A citation targets a fetched kind whose declarations have not been fetched yet. | warning | — | v2: `[rules.resolution] <KIND> = "warn"` is set for a fetched kind · v1: `resolve = "should"` is set on a fetched `[[kinds]]` row | [§FS-check.4.12](FS-check.md#412-missing-snapshot) |
| `mistitled-field` | A field's heading is titled differently from the title its kind gives it. | v2: by strength: the field's `presence` · v1: not in version 1 | — | v2: `[schema.kinds.<kind>.fields.<field>] title` is set · v1: not in version 1 | [§FS-fields.checks.mistitled-field](FS-fields.md#checksmistitled-field-mistitled-field) |
| `nothing-recognized` | The scan read files but recognized no declaration and no citation in them. | warning | — | — | [§FS-check.4.5](FS-check.md#45-nothing-recognized) |
| `optional-member-absent` | A workspace member listed as optional is not in this checkout, so it was not read. | warning | — | `[workspace] optional_members` is set; in any `[workspace]` block the run expands, a nested one included | [§FS-check.4.9](FS-check.md#49-a-workspace-member-declared-optional-is-absent) |
| `orphan-section` | A named section path is used before every shorter prefix of it is recorded in the same declaration. | error | — | v2: — · v1: `[id] named_sections = true` is set | [§FS-declarations.checks.orphan-section](FS-declarations.md#checksorphan-section-orphan-name-bearing-section-path) |
| `out-of-scope-dangling` | Under `--full`, a file outside the default scope cites an ID that no declaration declares. | error | — | `--full` is passed | [§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only) |
| `out-of-scope-local-section-citation` | Under `--full`, a file outside the default scope writes a marker followed by a bare section number. | error | — | `--full` is passed | [§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only) |
| `out-of-scope-missing-section` | Under `--full`, a file outside the default scope cites a section its declaration does not have. | error | — | `--full` is passed | [§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only) |
| `out-of-scope-shorthand-citation` | Under `--full`, a file outside the default scope writes a number-only shorthand that matches no declaration or several. | error | — | `--full` is passed | [§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only) |
| `out-of-scope-unknown-project` | Under `--full`, a file outside the default scope cites through a project alias the workspace does not know. | error | — | `--full` is passed | [§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only) |
| `oversized-lead` | A declaration's lead is longer than the configured word limit. | v2: by strength: `[schema.leads.words]` · v1: warning | — | v2: `[schema.leads.words] warn = <N>` is set; a `must` or `should` limit turns it on too, at that strength · v1: `[reference] lead_size_warning = { max = <N>, unit = "words" }` is set | [§FS-declarations.checks.oversized-lead](FS-declarations.md#checksoversized-lead-oversized-lead-opt-in) |
| `redundant-config` | A directory holds both `grund.toml` and `.agents/grund.toml`, and only the first is read. | warning | — | — | [§FS-check.4.3](FS-check.md#43-redundant-config-pair) |
| `relative-section-parent` | A relative section heading has no parent section one level up for its path to extend. | error | — | — | [§FS-declarations.checks.relative-section-parent](FS-declarations.md#checksrelative-section-parent-a-relative-heading-without-its-parent) |
| `remote-missing` | A declared remote project has no pinned projection to read. | error | — | planned — a `[workspace.remotes]` table | [§FS-remote-projects.checks.remote-missing](FS-remote-projects.md#checksremote-missing-a-declared-remote-has-no-pinned-projection) |
| `remote-modified` | A remote project's projection is not the bytes its lock records. | error | — | planned — a `[workspace.remotes]` table | [§FS-remote-projects.checks.remote-modified](FS-remote-projects.md#checksremote-modified-the-projection-is-not-the-locked-bytes) |
| `remote-orphan` | Remote state remains that no declaration names. | warning | — | planned — a `[workspace.remotes]` table | [§FS-remote-projects.checks.remote-orphan](FS-remote-projects.md#checksremote-orphan-state-no-declaration-names) |
| `remote-stale` | A remote project's lock entry disagrees with its declaration. | error | — | planned — a `[workspace.remotes]` table | [§FS-remote-projects.checks.remote-stale](FS-remote-projects.md#checksremote-stale-the-lock-disagrees-with-the-declaration) |
| `section-address-changed` | A cited section's text has moved to another address, and the cited address now holds something else. | error | — | a citation names a relative heading's address | [§FS-section-lock.checks.section-address-changed](FS-section-lock.md#checkssection-address-changed-a-cited-sections-text-sits-at-another-address) |
| `section-heading-level` | A section heading's Markdown depth does not match the depth of its section path. | v2: by strength: `[schema] heading_depth` · v1: error, or a warning under `[id] section_heading_levels = "warn"` | — | v2: `[schema] heading_depth = "must"` is set; it is the default, and `warn` or `should` turns it on too, at that strength · v1: `[id] section_heading_levels = "strict"` is set; it is the default, and `"warn"` turns it on too | [§FS-declarations.checks.section-heading-level](FS-declarations.md#checkssection-heading-level-section-heading-level-mismatch) |
| `section-outside-declaration` | A section heading sits outside every declaration's body. | error | — | — | [§FS-declarations.checks.section-outside-declaration](FS-declarations.md#checkssection-outside-declaration-section-outside-a-declaration) |
| `selected-check-off` | A check that `--only` selected was off for the whole run, so it judged nothing. | warning | — | a code `--only` selects is armed by no project of the run | [§FS-check.1.4.1](FS-check.md#141-a-selected-check-the-run-leaves-off-says-so) |
| `shorthand-citation` | A number-only shorthand citation is written where the full ID is required, or matches no declaration or several. | error | — | — | [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation) |
| `shorthand-numeric-run` | A number-only shorthand citation is glued to another number, so `grund fmt` will not expand it. | error | — | — | [§FS-check.3.15](FS-check.md#315-shorthand-citation-in-a-numeric-run) |
| `stub-shape` | A heading whose title is one link was read as a stub by one test and as a declaration by another; that can no longer happen. | none — spent from grund 0.19.0 and never fires | — | — | [§FS-declarations.checks.stub-shape](FS-declarations.md#checksstub-shape-stub-shape-read-differently-from-grund-0190) |
| `suggested-citation` | A declaration lacks a citation that a `should` entry or rule suggests. | none — suggestion | — | v2: `--suggestions` is passed and `[rules.citations.<kind>] should` or a `should` rule is set · v1: `--suggestions` is passed and `[citations.<kind>] should` or a `should` rule is set | [§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in) |
| `uncited-unit` | A declaration or chapter is cited more or fewer times than a rule's `be cited by` sentence allows. | error, or a suggestion for a `should` or `should not` rule | — | v2: `form = "rule"` is set on a `[schema.kinds]` table or `--rule` is passed · v1: `rules = true` is set on a `[[kinds]]` row or `--rule` is passed | [§FS-check.3.28](FS-check.md#328-uncited-unit) |
| `undeclared-chapter` | A declaration of a closed kind has a chapter that none of the kind's fields declares. | v2: by strength: the kind's `closed` · v1: not in version 1 | — | v2: `[schema.kinds.<kind>] closed = "must"` is set; a `warn` or `should` `closed` turns it on too, at that strength · v1: not in version 1 | [§FS-fields.checks.undeclared-chapter](FS-fields.md#checksundeclared-chapter-undeclared-chapter) |
| `ungrounded` | A unit in a place that must be grounded neither cites nor declares an ID. | v2: by strength: the rung of the grounding ladder it falls short of · v1: error | — | v2: a `must` rung is set in `[rules.citations.<kind>.grounding]`, or in `[rules.citations.grounding]` or by `--require-grounding` for a kind with no ladder of its own; a `warn` or `should` rung turns it on too, at that strength · v1: `require_grounding = true` is set on a `[[kinds]]` row, or in `[reference]` or by `--require-grounding` for a kind whose row does not set it `false` | [§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in) |
| `unknown-project` | A citation names a project alias that is not known, or a declaration or section that project does not have. | error | — | — | [§FS-check.3.8](FS-check.md#38-cross-project-citation-failure) |
| `unlinked-index-entry` | An index entry cites a declaration bare rather than as a Markdown link. | error | — | — | [§FS-check.3.17](FS-check.md#317-index-entry-is-not-a-link) |
| `unlisted-workspace-block` | A directory declares `[workspace]`, but no enclosing workspace lists it among its members. | error | — | — | [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) |
| `unmarked-heading` | A heading inside a declaration's body is neither a declaration nor a recognized section heading. | error | — | — | [§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading) |
| `unreached-declaration` | A declaration lacks the chapter a chapter-scoped citation rule over its kind needs in order to reach it. | error, or a suggestion for a `should` or `should not` rule | — | v2: `form = "rule"` is set on a `[schema.kinds]` table or `--rule` is passed · v1: `rules = true` is set on a `[[kinds]]` row or `--rule` is passed | [§FS-rules.checks.unreached-declaration](FS-rules.md#checksunreached-declaration-unreached-declaration) |
| `unread-scan-root` | A file the config names as a scan root is not read, because its name is hidden or its extension is not scanned. | warning | — | v2: a `files` entry of a `[schema.kinds]` table names a file that is not read for its name · v1: a `[[kinds]]` `file` home or a `[scan] include` entry names a file that is not read for its name | [§FS-scan.checks.unread-scan-root](FS-scan.md#checksunread-scan-root-a-configured-file-root-is-not-read) |
| `unrecorded-section-citation` | A citation names a locked section address that has no entry in the section lock. | error | — | a citation names a relative heading's address | [§FS-section-lock.checks.unrecorded-section-citation](FS-section-lock.md#checksunrecorded-section-citation-a-locked-address-the-lock-does-not-hold) |
| `unused` | A declared ID is never cited. | warning | — | — | [§FS-check.4.1](FS-check.md#41-unused-declaration) |
| `value-mismatch` | A value binding's written value differs from the value its target declares. | error | — | v2: `form = "value"` is set on a `[schema.kinds]` table or `content = "values"` on one of its fields; a value tag the scan finds turns it on too · v1: `values = true` or `value_chapter` is set on a `[[kinds]]` row; a value tag the scan finds turns it on too | [§FS-check.3.22](FS-check.md#322-value-mismatch) |

The four `remote-*` rows are specified ahead of the release that introduces them
([§FS-remote-projects](FS-remote-projects.md#fs-remote-projects-a-project-cites-another-repositorys-declarations-from-a-committed-pinned-projection)), which their *Enabled by* cell marks `planned`; until it ships, a config declaring `[workspace.remotes]` is refused
at load ([§FS-remote-projects.declaration.older](FS-remote-projects.md#declarationolder-an-older-grund-refuses-the-table-loudly)), so none of them fires. Shipping them edits that cell and moves no row.

The five field codes `invalid-field-value`, `misplaced-field`, `missing-field`,
`mistitled-field` and `undeclared-chapter` take their channel from the strength
their field or kind writes, so one code can be an error, a warning or a
suggestion ([§FS-fields.strengths.table](FS-fields.md#strengthstable-which-strength-each-finding-takes)).

The chapter-rule codes `chapter-cardinality`, `citation-cardinality`,
`invalid-rule`, `uncited-unit`, and `unreached-declaration` are selectable on
the same surfaces as every other code
([§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits)).

Every future check finding code enters this catalog in the release that
introduces it; renaming or removing one requires compatibility treatment. The
catalog changes neither the NDJSON object nor the library/LSP report. `io` is a
recognized code but an incomplete-scan safety finding remains retained and
exit `2` even when `--ignore io` or an excluding `--only` set is present
([§FS-check.2](FS-check.md#2-outputs)).

#### 5.5.1 How a row is read

Every cell but *Check* is printed by the installed `grund`: `grund config explain`
answers for a code from its row ([§FS-config-explain.4](FS-config-explain.md#4-a-codes-answer)), and the off line of
[§FS-check.1.4.1](FS-check.md#141-a-selected-check-the-run-leaves-off-says-so) is built from its *Enabled by* and *Severity* cells. So no cell
names a specification ID or holds a link, and each is written by these rules.

- **Meaning** is one plain sentence saying what the code reports, in words that
  hold in both configuration versions.
- **Versions.** A *Severity* or *Enabled by* cell may split as `v2: … · v1: …`,
  version 2 first; each side holds for a config file read as that version, and a
  cell that does not split holds for both. `not in version N` is a value: a
  config file read as version N never turns the code on, though a project of
  another version in the same run may.
- **Severity.** `error` or `warning` alone is fixed. `none — suggestion` is no
  severity: the finding is a suggestion, shown only with `--suggestions`
  ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)). `none — spent …` is a code that never fires and that `--only` and
  `--ignore` still accept; the rest of the cell, after `none — `, says what
  became of it. `by strength: <setting>` is chosen by the strength the named
  setting is written under: `must` an error, `warn` a warning, `should` a
  suggestion, and `may` not checked. Any other cell names a severity and the
  setting or the rule strength that changes it, so it is chosen by a setting too.
- **Enabled by.** `—` is always on. `planned — …` marks a row specified ahead of
  its release: its code is in no selector vocabulary, `grund config explain`
  does not know it, and the rest of the cell is what will turn it on. Any other
  cell is one clause, or several joined by ` and `, every one of which the
  check needs, and no clause holds ` and ` within it. A clause that says
  `is set` is turned on by a setting, one that says `is passed` by a flag, one
  that says both by either, and one that says neither by a condition of the
  tree that the check itself judges. A clause may end in a remark after `; `,
  another way the same clause is met or what else the run needs, which
  `explain` prints and the off line leaves out. A split cell names the same
  clauses in the same order on each side, so one clause is one switch spelled in
  each version.
- **Printing.** Backticks mark a spelling, and a printed cell is its text without
  them.

## 6. The `grund init` transcript

`grund init` ([§FS-init.2.2](FS-init.md#22-stdout--stderr)) writes status lines to **stderr** — `wrote AGENTS.md`, `appended CLAUDE.md`, `exists grund.toml`, etc. — followed by the `next:` block. These are **not** the command's output: `init`'s output is the scaffold it wrote to disk; the transcript is progress, and nobody pipes `grund init`. They use a `<verb> <path>` shape. This is one of two carve-outs from [§FS-errors.1](FS-errors.md#1-streams); the other is `grund integrations --write`, which reports what it wrote on stderr in the same `<verb> <path>` shape ([§FS-integrations.4.1.6](FS-integrations.md#416-outcome-verbs)). Every other subcommand puts its output on stdout. In particular `grund fmt --write` ([§FS-fmt.3](FS-fmt.md#3-outputs)) does **not** use this shape: its `rewrote N line(s):` report is `fmt`'s output and goes to stdout, the same stream as its `--check` dry-run report. A subcommand with no output and no transcript stays silent and lets the exit code carry the verdict.

## 7. What this rules out

- A severity prefix (`error:`, `warning:`) ahead of a located finding's `<path>:<line>:` prefix — that leading position marks a CLI-level message ([§FS-errors.2.2](FS-errors.md#22-cli-level-message)); the channel prefix `check` places goes after the location ([§FS-errors.3.4](FS-errors.md#34-the-check-channel-prefix)).
- Multi-line messages. A finding that wants to elaborate uses `--format=json` and its `code` ([§FS-errors.5.1](FS-errors.md#51-on-stdout--the-commands-output)), not a wrapped paragraph.
- Interactive prompts, progress bars, or spinners. Per [§FS-non-goals.10](FS-non-goals.md#10-interactive-mode), every subcommand is non-interactive.
- Localization. Messages are English; translation is downstream's problem.
