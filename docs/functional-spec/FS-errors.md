# FS-errors: grund emits messages in fixed shapes

Every `grund` command speaks in a few fixed shapes — the stream a line goes to, the prefix that sets a finding apart from a message about the run, the exit status beside it, the style of its text, the order its lines come in, and the JSON a program reads instead — so that an editor, a script and another install read the same bytes for the same input ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)). A command with something new to say uses one of these shapes rather than inventing one. Serves [§GOAL-friendliness-first.1](../goals.md#1-hard-requirements) (errors point at the line) and [§GOAL-no-silent-breakage.1](../goals.md#1-what-counts-as-user-visible) (the shapes are user-visible output). This page states the rules, and [§FS-output-shapes](FS-output-shapes.md#fs-output-shapes-machine-readable-output-shapes) shows examples of them on the wire.

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

Two requirements bind the shapes as a whole, beside the tool-wide ones every message already serves: the same bytes for the same input ([§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)), the exit-code mapping ([§REQ-never-crashes.2](../requirements/REQ-never-crashes.md#2-exit-codes-are-the-api)), and the deprecation path for user-visible bytes ([§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)).

### requirements.fields: What a program needs from a finding is a field

A consumer never parses `message` to act on a finding. What it is, is its `code`, the exact selector vocabulary ([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)); how serious, its `severity` or `channel`, and which rule said it, its `authority` ([§FS-errors.5.1](FS-errors.md#51-on-stdout--the-commands-output)); and every site its text names is in `sites`, in the order the text names them ([§FS-errors.5.1](FS-errors.md#51-on-stdout--the-commands-output), [§FS-errors.5.2](FS-errors.md#52-on-stderr--what-is-not-output)). Where one code has two shapes, a field tells them apart, as `sites` tells the two `ambiguous` refusals ([§FS-errors.5.2](FS-errors.md#52-on-stderr--what-is-not-output)). The exceptions are about the run rather than the graph: a run-level warning carries its location in its text, and also in `path` and `line` where it is raised at a line of a config file ([§FS-errors.5.2.3](FS-errors.md#523-run-level-findings-in-checks-report)), and a launch-time message stays text under every `--format` and is told by its exit code ([§FS-errors.5.2.2](FS-errors.md#522-launch-time-messages-stay-text)). Every finding holds to this, and so must every finding added: one that names a site in its text carries it in `sites`, and one that shares a code with a sibling differs from it in a field. Serves [§GOAL-friendliness-first.1](../goals.md#1-hard-requirements).

### requirements.prefix-windows: A wording change keeps the old message whole until its window closes

A message's exact text is user-visible output that tools grep ([§GOAL-no-silent-breakage.1](../goals.md#1-what-counts-as-user-visible)), so a change to it walks [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)'s path this way: through its window the text a consumer matched stays whole as a verbatim contiguous prefix, what the change adds follows that prefix, the message names the release the window ends in, and the change names what an exact-line consumer moves to meanwhile: the finding's stable `code`, or, for a refusal that has no code ([§FS-errors.2.5](FS-errors.md#25-refused-input)), the exit status and the new reason. The code, the severity and the exit verdict do not move. Bytes appended after a prefix that survives are no change at all and ship in one release, because every prefix and `code` consumer still reads what it read, and only an exact-line consumer has to move. A message is corrected in place instead, with no window, under the pre-release licence of [§REQ-backwards-compatibility.4](../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise), where a window would print on purpose a sentence that does not parse, a cause that is not the one, or a remedy that cannot work; each such correction is argued in its decision: [§DF-stale-section-differs-in-0-20](../decisions/functional/DF-stale-section-differs-in-0-20.md#df-stale-section-differs-in-0-20-the-stale-block-finding-names-its-section-as-one-subject-corrected-in-place-in-0200), [§DF-scan-failure-code-and-fix](../decisions/functional/DF-scan-failure-code-and-fix.md#df-scan-failure-code-and-fix-a-scan-failure-carries-its-own-code-and-a-fix-that-works-at-its-path), [§DF-fetch-source-home.3.4](../decisions/functional/DF-fetch-source-home.md#34-what-each-finding-names), [§DF-fetch-unread-home](../decisions/functional/DF-fetch-unread-home.md#df-fetch-unread-home-fetch-refuses-a-home-the-scan-does-not-read-before-the-integration-runs), [§DF-blank-fetch-check-tail](../decisions/functional/DF-blank-fetch-check-tail.md#df-blank-fetch-check-tail-check-offers-the-repair-not-a-command-that-can-only-refuse-for-a-fetch-key-with-nothing-to-run), [§DF-fetch-refusals-follow-the-scan](../decisions/functional/DF-fetch-refusals-follow-the-scan.md#df-fetch-refusals-follow-the-scan-four-fetch-refusals-say-what-the-scan-reads-corrected-in-place-and-an-empty-idmd-is-filled), [§DF-presence-or-refusal](../decisions/functional/DF-presence-or-refusal.md#df-presence-or-refusal-a-presence-name-holding-or-between-two-words-is-refused-for-the-or-corrected-in-place), [§DF-distinct-target-counts](../decisions/functional/DF-distinct-target-counts.md#df-distinct-target-counts-an-outbound-count-may-count-distinct-cited-declarations-spelled-distinct-after-the-count) and [§DF-reading-at-fetch-and-unread-homes.3.2](../decisions/functional/DF-reading-at-fetch-and-unread-homes.md#32-messages-corrected-in-place). Every other wording change holds to this, and the next one must. Serves [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path).

## 1. Streams

`grund` follows the linter convention of `eslint`, `ruff` and `shellcheck`: what a command produces goes to **stdout**, a checker's findings included, and **stderr** carries only what the command says about the run. The two never mix, so `grund check 2>/dev/null` shows the findings and only the findings, `grund check >/dev/null` only the run-level errors, and `grund <ID> | …` gets the body and nothing else.

**stdout** carries:

- a query result: the body `grund <ID>` prints, `grund list`'s catalog, `grund refs`' citations, `grund cover`'s file graph, `grund id`'s ID, `grund config show`'s config, and `grund config migrate`'s candidate, whose report goes to stderr ([§FS-config-migrate.2.1](FS-config-migrate.md#21-stdout-is-the-candidate));
- a checker report: every located finding of `grund check` ([§FS-check.2.1](FS-check.md#21-report-format)), the text-mode `success` line of a clean one ([§FS-errors.2.4](FS-errors.md#24-text-success-line)), and `grund fmt`'s would-change and did-change report ([§FS-fmt.3](FS-fmt.md#3-outputs)). `grund check --format=json` prints findings only, so a clean run leaves stdout empty.

**stderr** carries:

- `error:` lines, for a failure that means the run could not do its job, always with a non-zero exit ([§FS-errors.2.2](FS-errors.md#22-cli-level-message));
- `warning:` lines about the run rather than its content, such as an empty scan ([§FS-check.2.2](FS-check.md#22-empty-scan)), which leave the exit alone;
- `note:` and `hint:` recovery lines ([§FS-refs.2](FS-refs.md#2-behaviour), [§FS-show.3.4](FS-show.md#34-what-a-failed-query-prints)), among them a coordinate glob's empty-scope `note:` and its PATH `hint:` ([§FS-coordinate-globs.errors.note](FS-coordinate-globs.md#errorsnote-a-pattern-no-kind-can-start-gets-a-note), [§FS-coordinate-globs.errors.hint](FS-coordinate-globs.md#errorshint-a-glob-typed-as-path-gets-a-hint));
- the bare message of a failed query, which has no result to put on stdout ([§FS-errors.2.3](FS-errors.md#23-bare-query-failure));
- the progress transcripts of `grund init` and `grund integrations --write` ([§FS-errors.6](FS-errors.md#6-the-grund-init-transcript)).

`show --batch` keeps a failed query off stderr in every format ([§FS-show.3.3](FS-show.md#33-batch-query)): `text` and `md` print the failure's message and its `hint:` lines on stdout in the query's place among the answers ([§FS-show.3.3.2](FS-show.md#332-a-failure-stays-in-its-place)), and JSON carries the failure in that query's record of its envelope, with no hint ([§FS-errors.5.3](FS-errors.md#53-show---batch)).

## 2. The Fixed Shapes

### 2.1 Located finding

A finding at a site in the repository, on **stdout** because it is the command's output:

```
<path>:<line>: <message>
<path>:<line>: <channel>: <message>
```

- `<path>` is rendered against the report base ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from), [§FS-config.3.6](FS-config.md#36-output--report-format)), and `<line>` is 1-indexed.
- `<channel>` is `error`, `warning` or `suggestion`, written on every finding `grund check` prints ([§FS-check.2.1](FS-check.md#21-report-format)) and carried by JSON's `severity` or `channel` instead ([§FS-errors.5.1](FS-errors.md#51-on-stdout--the-commands-output)). `fmt`'s would-change lines ([§FS-fmt.3](FS-fmt.md#3-outputs)) and `refs`' citations ([§FS-refs.3.1](FS-refs.md#31---format-text-default)) take the form without it, and an LSP diagnostic keeps its own content, without the prefix ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)).
- `<message>` is a single line with no terminal period, and never includes the channel.
- The `<path>:<line>:` prefix is mandatory: editors and agents jump on this exact shape.

### 2.2 CLI-level message

A line about the run rather than a site in the repository, on **stderr** because it is not the command's output:

```
error: <message>
warning: <message>
```

The leading `error: ` or `warning: ` is what sets it apart from a located finding, whose channel follows its location, and CI scripts grep for a leading `error:` to tell a launch-time failure from a clean run that found findings. So a CLI-level message never begins with `<path>:<line>:`, though its text may name a location after the prefix: a config line, as in `error: <path>:<line>: <message>` for a `grund.toml` schema error ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)) or `warning: b/grund.toml:3: …`, an unlisted workspace block on the scanning surfaces other than `check` ([§FS-check.3.29](FS-check.md#329-unlisted-workspace-block)); a whole file, as in `error: <path>: <reason>` ([§FS-check.2](FS-check.md#2-outputs)); or a file in prose, as in `error: read grund.toml: Permission denied (os error 13)`. How it renders under `--format=json` is [§FS-errors.5.2.2](FS-errors.md#522-launch-time-messages-stay-text)'s for a launch-time message and [§FS-errors.5.2.3](FS-errors.md#523-run-level-findings-in-checks-report)'s for a failure `check` collects mid-scan.

#### 2.2.2 Exit codes

`error:` always accompanies a non-zero exit, `2` unless a section names another, for a failure that means the run could not do its job: a launch-time, setup or I/O failure that leaves no query context or trustworthy scan, or an operational command's own failure, such as `fetch`'s rejected integration output or ambiguous existing content ([§FS-fetch.7](FS-fetch.md#7-output-and-exits)). `grund config validate` is the named exception: it prints the same `error: <path>:<line>: <message>` for an invalid config and exits `1` ([§FS-config.4.1](FS-config.md#41-grund-config-validate-path)). `warning:` leaves the exit code alone; it is a caution, not a failure. Once context has selected a project's ID grammar, an operand its resolver rejects is a failed query instead, exit `1`, in the bare shape of [§FS-errors.2.3](FS-errors.md#23-bare-query-failure) ([§DF-refs-resolver-rejection](../decisions/functional/DF-refs-resolver-rejection.md#df-refs-resolver-rejection-an-id-rejected-by-a-selected-grammar-is-a-failed-query)). The exit-code mapping itself is [§FS-cli.5](FS-cli.md#5-exit-code-mapping-is-fixed).

#### 2.2.5 One scan failure, one rendering

A file the scan could not read or parse has one reported form in a run: its location, spelled from the report base with the base itself `.` ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)); its message, the reason followed by `; ` and its *Fix* ([§FS-check.2.4.3](FS-check.md#243-what-failed-and-where-decide-the-code-and-the-fix)); and its code. Every surface that publishes the failure publishes that form: the `error: <location>: <message>` line of `check`, `list`, `show`, `show --batch`, `refs`, `id` and `fetch`, `check`'s JSON record, the embedding API's record, and the failure a binding's `scan`, `show` and `showBatch` fail with, which carries the location as `path` ([§FS-workspace.8.7.3](FS-workspace.md#873-paths)). `proposeId` has no `path`, so its failure carries the same message with the location at its head. `init` stops on the first failure its scan meets and prints that failure's message, *Fix* included; where `init` locates it is not specified. No surface drops the *Fix*, and none repeats the location at the head of a message whose record carries it as `path`.

A file the reason names is part of what failed, so it is spelled like the location beside it, and the same tree prints the same bytes wherever it sits ([§FS-errors.4](FS-errors.md#4-determinism)): a run from a subdirectory under `--path-base=invocation` names it with the location's `..`, and a member's file is named from the workspace root, not from the member that scanned it. An `expected.exit` that does not parse at a workspace root with `members = ["m"]` is therefore `error: m/e2e/cases/a/expected.exit: parse m/e2e/cases/a/expected.exit: invalid digit found in string; …`.

### 2.3 Bare query failure

A command that established its query context but has no result to print says why in one line with no prefix, on **stderr**, and exits `1`, leaving stdout empty:

```
<message>
```

It has no `error:` prefix because the run did not fail; the request could not be satisfied ([§FS-errors.2.2](FS-errors.md#22-cli-level-message)). The shape answers an ID query that reads one coordinate, for an ID that is missing, invalid under the selected grammar or ambiguous, a missing or ambiguous section, or a broken stub ([§FS-show.3](FS-show.md#3-outputs), [§FS-show.2.2.1](FS-show.md#221-ambiguous-id), [§FS-show.2.2.2](FS-show.md#222-ambiguous-section), [§FS-show.2.3.4](FS-show.md#234-broken-stub)); `grund refs` when its resolver rejects an invalid ID or an ambiguous number-only shorthand ([§FS-refs.4](FS-refs.md#4-exit-codes)); `grund id` when a title slugifies to nothing or the proposed ID collides with a declaration ([§FS-id.3](FS-id.md#3-slug-derivation), [§FS-id.5](FS-id.md#5-collision-check)); and `grund config explain` for a target its version does not know ([§FS-config-explain.5](FS-config-explain.md#5-a-target-the-version-does-not-know)). `check` never uses it.

An ambiguity lists every site in the order of [§FS-errors.4.1.1](FS-errors.md#411-the-order-of-a-duplicates-sites), but an ambiguous number-only shorthand lists its candidate IDs in ID order ([§FS-show.2.2.1](FS-show.md#221-ambiguous-id)). A `hint:` line may follow where the next step is obvious, and a missing-target stub beside another home appends its repair after the complete ambiguity, leaving JSON `sites` as the ambiguity listed them ([§FS-declarations.stubs.repair](FS-declarations.md#stubsrepair-a-missing-target-beside-another-declaration-explains-the-repair)).

`show --batch` prints the failure on **stdout** instead, in its query's place among the answers ([§FS-errors.1](FS-errors.md#1-streams)), and exits with the batch's aggregate ([§FS-show.3.3](FS-show.md#33-batch-query)).

### 2.4 Text success line

`grund check`'s text form prints exactly this line on **stdout**, with one trailing newline, when its selected report is otherwise empty, and exits `0` ([§FS-check.2.1.3](FS-check.md#213-the-success-line)):

```
success
```

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
  and exit `2`.
- `<reason>` names the production that failed, and does not change with what
  follows it. After `; ` stands exactly one label and what it labels: a rule
  sentence after `accepted form:`, several quoted after `accepted forms:`, a
  selector after `accepted selector:`, a double-quoted glob after `accepted glob:`,
  and several joined with `or` after `accepted globs:` ([§FS-coordinate-globs.errors.shape](FS-coordinate-globs.md#errorsshape-one-refusal-line)).
  A label that ends in `after enabling it:` says the input is accepted only once
  the configuration enables what the reason names.
- Where nothing can be offered, the reason stands alone. Where what could not
  be supplied is a kind, the next line is `known kinds: <kinds>`, and a refusal
  that has a next step to name ends in one `hint:` line.
- A configured rule declaration refused for the same reason is a located
  finding ([§FS-errors.2.1](FS-errors.md#21-located-finding)) whose message carries the same reason and accepted
  input after `is not a valid rule: `, and never the lines after it.

Which inputs each surface refuses, and how the accepted input is built from
what was typed, is [§FS-rule-refusals.sentences](FS-rule-refusals.md#sentences-strict-refusals) for rule sentences and [§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector)
for selectors.

#### 2.5.1 A kind that declares no IDs

A configured kind that is not citable ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)) is refused by every
surface that takes a kind name with one reason, built in one place from the
kind and its home, in the quoting that surface already uses: a backtick on
`list --kind` and `grund id` ([§FS-list.1.1](FS-list.md#11---kind), [§FS-id.1](FS-id.md#1-inputs)), a double quote on
`list --selector`, whether the kind opens a sentence or a `KIND` or `KIND.NAME`
selector, and on a rule's subject and object.

```text
kind `<KIND>` declares no IDs — <home>/ is not a citable home
kind "<KIND>" declares no IDs — <home>/ is not a citable home
```

The verdict is [§FS-scan.projects.kinds](FS-scan.md#projectskinds-whose-kinds-a-run-reads)'s, read in the asking surface's scope,
and on the command line the next line is `known kinds: <kinds>`, naming the
citable kinds of that scope. A configured rule declaration's `invalid-rule`
finding carries the reason alone after `is not a valid rule: `
([§FS-errors.2.5](FS-errors.md#25-refused-input)). A `[citations]` target keeps its own wording, *a non-citable
target kind* ([§FS-config.3.9.5](FS-config.md#395-validation)).

`code`, the homeless kind's default name, has no row in a project whose
`[[kinds]]` declares no homeless kind ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)), yet `[citations]`
refuses it as not citable, so `list --kind`, `list --selector` and a rule's
subject and object refuse it with this reason too. It has no home, so the
reason ends at `declares no IDs`:

```text
kind `code` declares no IDs
kind "code" declares no IDs
```

## 3. Message text

The shape is structural and the text is for a reader. Every shape's text keeps these rules, and the sections below fix the exact text of the messages whose owners hand their wording here:

- **Lowercase first letter**: `unknown reference <ID>`, not `Unknown reference <ID>`.
- **No terminal period.** A single-clause message ends in no `.` or `!`. A run-level caution of more than one clause, such as the empty-scan warning ([§FS-check.2.2.2](FS-check.md#222-the-message-names-the-likely-cause)), is written in sentences and punctuates each, its last included.
- **No ANSI colors.** Output is plain bytes whatever `[output] color` is set to ([§FS-config.3.6](FS-config.md#36-output--report-format)).
- **Stable phrasing.** Tools grep a message's exact text, so it changes only as [§FS-errors.requirements.prefix-windows](FS-errors.md#requirementsprefix-windows-a-wording-change-keeps-the-old-message-whole-until-its-window-closes) says.
- **Candidates** are offered in one recovery clause after `; `: `did you mean <a>?`, `did you mean <a> or <b>?` or `did you mean <a>, <b> or <c>?`, with no comma before `or`, and the clause ends in one `?`.
- **Quoted user input** stands in double quotes where it could be confused with the prose around it: `"<original title>"`, not `<original title>`.
- **One base for every path in the line.** A path inside the message text — a duplicate's other homes, an ambiguous ID's competing sites, the stub a broken-stub refusal names — is a report path, rendered against the same base as the location ([§FS-config.3.6](FS-config.md#36-output--report-format)). In a workspace that base is the root the run reports from, never the member the finding came out of ([§FS-workspace.8.1](FS-workspace.md#81-grund-aliasid)), because a line whose two halves start from two roots sends the reader, and an editor following it, to a file that is not there. A path the message quotes back from the user's own text, such as a stub's link target, is not resolved and stays verbatim.

### 3.6 The `agents-init` messages

An `agents-init` finding ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)) prints one of six templates, the sixth because the check compares the whole block ([§FS-check.3.5.5](FS-check.md#355-the-workspace-sections-are-judged-at-the-outermost-root-of-a-full-checkout), [§DF-check-compares-the-whole-block](../decisions/functional/DF-check-compares-the-whole-block.md#df-check-compares-the-whole-block-grund-check-holds-the-whole-managed-block-the-workspace-sections-at-the-outermost-root-of-a-full-checkout)):

```text
repo maintenance: malformed grund managed block: <detail> (does not affect citation validity)
repo maintenance: outdated grund init block v<found> — run `grund init` to update to v<current> (does not affect citation validity)
repo maintenance: unsupported grund init block v<found> — this grund supports v<current> (does not affect citation validity)
repo maintenance: stale grund init block: the <section> section differs from grund.toml — run `grund init` to refresh (does not affect citation validity)
repo maintenance: missing grund init block v<current> — run `grund init` to install it (does not affect citation validity)
repo maintenance: stale grund init block: the <section> section differs from the workspace (<detail>) — run `grund init` in <dir> to refresh (does not affect citation validity)
```

`<found>` is the version the block's heading carries. `<current>` is the version the live config selects in `outdated` and `missing`, and in `unsupported` the highest this binary supports, the number `grund init`'s refusal of the same block names, never the config's selection ([§FS-init.2.3.7](FS-init.md#237-the-block-version)). `<section>` is one of the nouns of [§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block), named as one subject, `the <section> section`, so the sentence reads the same whichever heading it names, a heading someone wrote into the block included ([§DF-stale-section-differs-in-0-20](../decisions/functional/DF-stale-section-differs-in-0-20.md#df-stale-section-differs-in-0-20-the-stale-block-finding-names-its-section-as-one-subject-corrected-in-place-in-0200)). The sixth template names one of the two workspace sections, and its `<detail>` is the first differing entry of [§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block). `<dir>` is the root of the project that owns the block, spelled from the finding's own base, and ` in <dir>` is left out where that project is the one the run started in. The first five templates grow only at the end ([§FS-errors.requirements.prefix-windows](FS-errors.md#requirementsprefix-windows-a-wording-change-keeps-the-old-message-whole-until-its-window-closes)), so where they report another project's block, `outdated`, the first `stale` and `missing` append `; run it in <dir>`, while `malformed` and `unsupported` name no command and append nothing:

```text
packages/billing/AGENTS.md:3: error: repo maintenance: outdated grund init block v35 — run `grund init` to update to v51 (does not affect citation validity); run it in packages/billing
```

`repo maintenance` classifies the message only; it is not a finding category or selector value ([§FS-check.1](FS-check.md#1-inputs)).

### 3.7 The rule-site unknown-alias reasons

A rule whose object kind is pinned at an alias (`<ALIAS>/<KIND>`) or matched across every alias (`*/<KIND>`), in a run that holds no workspace vocabulary to resolve it, is unverifiable here ([§FS-rules.4.1](FS-rules.md#41-a-rule-this-scope-cannot-verify)). Its reason says what cannot be resolved in this scope, offers no candidate, and sends the reader to the workspace root, as a narrowed run does for a citation it cannot place ([§FS-check.3.8](FS-check.md#38-cross-project-citation-failure)). It is exactly the first line for a pinned object and the second for `*/<KIND>`:

```text
unknown project alias <ALIAS>; no workspace is in scope here, so the alias cannot be resolved — check from the workspace root
no workspace is in scope here, so no namespace can be searched for <KIND> — check from the workspace root
```

A configured rule declaration carries the reason in its `invalid-rule` finding at the rule's heading, and `check --rule` refuses the sentence with it as refused input ([§FS-errors.2.5](FS-errors.md#25-refused-input)). An alias the run can judge and rejects is refused for its kind instead, as `unknown kind "<KIND>" in namespace "<ALIAS>"` or `unknown kind "<KIND>" in any workspace namespace`, because there the kind really is what failed ([§FS-rules.4.1](FS-rules.md#41-a-rule-this-scope-cannot-verify)).

### 3.11 The missing-section remedy

Every `missing-section` finding ([§FS-check.3.2](FS-check.md#32-missing-section)) gives the `escape` case of its row's *Fix* in [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog), its `{escape}` the configured marker's escape: `<§>` under the default marker, `<@>` under `@`. Every missing section is owed it alike — numeric or named, a citation that carries its marker or a bare token a version 1 project reads without one, with `[id] named_sections` on or off — because a renumbered heading, a mistyped number and an example someone meant to escape all reach this one finding. The escape clears each of those sites, since an ID-shaped token right after the literal escape is no citation under either strict mode ([§FS-citations.recognized.escapes](FS-citations.md#recognizedescapes-an-id-in-an-escape-position)).

The remedy follows one of two heads:

- a marker-prefixed named coordinate, where named sections are on, is
  `section not found: <coordinate>; <escape>; <map>`, its `map` case naming the command that
  lists the handles the declaration has, because a handle is a name someone guessed;
- every other missing section is `missing section <coordinate>; <escape>`.

`<coordinate>` is the cited ID as the report renders it, alias included, then
`[id] section_separator` and the cited path, and the `map` case's `<ID>` is that cited ID alone.
Where the section lock names where the cited text went, its clause ([§FS-section-lock.checks.vanished](FS-section-lock.md#checksvanished-a-cited-address-that-is-gone-and-where-its-text-went))
takes the place of the map in the `section not found` form, and follows the coordinate in the
`missing section` form, the remedy last. Under `--full`, `out-of-scope-missing-section` prints
the same message after its scope prefix ([§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only)).

A declaration `FS-login` whose only section is `1`, cited as `<§>FS-login.session` and
`<§>FS-login.7` on one line, reports:

```text
docs/references.md:1: error: missing section FS-login.7; write <§> before it to show the shape without citing it
docs/references.md:1: error: section not found: FS-login.session; write <§> before it to show the shape without citing it; run `grund FS-login --toc` to print the lead with the section map
```

## 4. Determinism

Two runs of one command on the same input print byte-identical stdout *and* stderr ([§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)), so no report holds:

- a wall-clock timestamp, a process ID or a hostname;
- in the reports of the tree-reading commands [§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes) names, an absolute path, or a path that escapes the loaded root other than the `..` path of a config above it that the run had to read or of a file in an import, with no absolute fallback for a target inside the root ([§FS-config.3.6](FS-config.md#36-output--report-format), [§DF-cli-base-parent-paths.2](../decisions/functional/DF-cli-base-parent-paths.md#2-decision), [§FS-imports.paths](FS-imports.md#paths-imported-files-are-reported-where-they-are));
- an order that is not fixed ([§FS-errors.4.1](FS-errors.md#41-ordering));
- a platform-native path separator: every path in a report, a JSON field, an e2e case manifest, a duplicate's site list, a stub-link note or a formatter summary is written with `/`, so Windows and Unix runs over one tree compare byte for byte.

Finding selection keeps this contract: `check --only` and `--ignore` filter the complete report before the sort ([§FS-check.2.1.2](FS-check.md#212-selection-filters-the-complete-report)), and the order or repetition of their flags changes nothing.

### 4.1 Ordering

Each completed `check` report stably sorts its `errors`, `warnings` and
`suggestions` channels bytewise by `(path, line, message)`, after path narrowing
and caution insertion: an absent path sorts first, an absent line counts as zero,
and ties keep their order ([§DF-completed-check-report-order](../decisions/functional/DF-completed-check-report-order.md#df-completed-check-report-order-completed-check-reports-share-one-order)). Text prints errors,
then warnings, then enabled suggestions, each group in that order; JSON keeps one
global `(path, line, message)` order across channels. Selection keeps the order,
and the `selected-check-off` advice it generates is appended after it ([§FS-check.1.4.1](FS-check.md#141-a-selected-check-the-run-leaves-off-says-so)). A
message whose content would otherwise vary in order, such as a duplicate's sites, is sorted
before it is printed ([§FS-errors.4.1.1](FS-errors.md#411-the-order-of-a-duplicates-sites)). Rule findings take this order with no
sorter of their own ([§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits)). Every other report keeps the order its own
section documents.

#### 4.1.1 The order of a duplicate's sites

The sites of one duplicate, which `duplicate`, `ambiguous` and `ambiguous-section` list in the
message and in JSON `sites`, have one order whichever command names them
([§REQ-one-answer.identity.declarations](../requirements/REQ-one-answer.md#identitydeclarations-every-command-reads-one-declaration-set), [§DF-one-site-order](../decisions/functional/DF-one-site-order.md#df-one-site-order-a-duplicates-sites-have-one-order-checks-whichever-command-lists-them)): by path, then by line as a number.
The path is the name the site is reported by, relative to its project root, taken before it is
rendered against the report base and compared bytewise with `/` separators, so neither
`--path-base` nor the directory a run starts from moves a site. The **first site** is the first
in this order, and it is where `check` locates the finding ([§FS-check.2.1](FS-check.md#21-report-format)).

A file reached by two names is one site here, under the name
[§FS-scan.identity.findings.content](FS-scan.md#identityfindingscontent-a-content-finding-is-one-line-naming-every-name) settles on, and its other names take no part in the order. A
content finding's `(also at …)` names and a value mismatch's sites keep their own orders.

## 5. JSON format

`check`, `show`, `list`, `refs`, `cover`, `id` and `integrations` accept `--format=json` ([§FS-cli.3](FS-cli.md#3-cross-subcommand-flags), [§FS-integrations.5](FS-integrations.md#5-json-format)); `fmt`, `fetch`, `init`, `config`, `guide` and `completions`, whose output is human text or generated files, do not. JSON keeps the stream split of [§FS-errors.1](FS-errors.md#1-streams).

### 5.1 On stdout — the command's output

`grund check --format=json` prints its findings as NDJSON on stdout, one object per line, in the shape of [§FS-distribution.3.0](FS-distribution.md#30-language-neutral-data-shapes) with its keys in this order: `{ severity, path, line, code, message, sites, authority }` (examples in [§FS-output-shapes.1](FS-output-shapes.md#1-finding-object)).

- `severity` is `error` or `warning`. A suggestion, printed only under `--suggestions` ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)), carries `"channel": "suggestion"` in its place, so the severity set stays `{error, warning}` ([§FS-config.6](FS-config.md#6-what-is-not-configured-here)) and a consumer filtering on `severity` never sees one.
- `message` is the text line's bytes after its location and channel prefixes.
- `sites` is `null` for a finding at one site whose text names no other. A finding at several sites, such as a duplicate declaration ([§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration)), lists every site as `[{ path, line }]`, and so does a finding with one location whose text names other sites: its `sites` are what its text names, in the text's order, rather than its location. The section that specifies such a finding says which sites its text names.
- `authority` is `null` for a finding no chapter rule authored, else the bytewise-sorted list of the rule origins that did ([§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits)); it is what `check --only-rule` queries ([§FS-rules.8](FS-rules.md#8-command-surfaces)).

#### 5.1.1 Query results

Query commands print their result on stdout too: one JSON object for a single result (`grund <ID> --format=json`, [§FS-show](FS-show.md#fs-show-grund-reads-a-single-declaration-body-by-id); `grund id --format=json`, [§FS-id](FS-id.md#fs-id-grund-proposes-ids-for-new-declarations)), and NDJSON, one object per row, for a list (`grund list` per declaration, `grund refs` per citation, `grund cover` per scanned file).

### 5.2 On stderr — what is not output

A failed ID query (`ID not found`, `ambiguous`, `broken stub`, `section not found`, `invalid ID`; exit `1`) prints its one finding object on stderr, in the shape of [§FS-errors.5.1](FS-errors.md#51-on-stdout--the-commands-output) with `path` and `line` `null`: there is no single site and no result, so nothing goes to stdout. `authority` is there and always `null`, so every record of the shape has one key set ([§DF-rule-authority-is-a-field.2](../decisions/functional/DF-rule-authority-is-a-field.md#2-decision)). `refs`' invalid-ID and ambiguous number-only rejections are among them, with the codes `invalid-id` and `ambiguous`, and neither carries the text-mode hint.

`sites` is `null` on a query failure but two: an `ambiguous` refusal naming an ID with two homes ([§FS-show.2.2.1](FS-show.md#221-ambiguous-id)) and an `ambiguous-section` refusal ([§FS-show.2.2.2](FS-show.md#222-ambiguous-section)) carry the `[{ path, line }]` pairs the message names, in its order. The number-only shorthand's `ambiguous` refusal names candidate IDs rather than sites, so it carries `null`, and a consumer tells the two `ambiguous` shapes apart by whether `sites` is `null`, not by `code`.

#### 5.2.2 Launch-time messages stay text

A *launch-time* CLI-level message ([§FS-errors.2.2](FS-errors.md#22-cli-level-message)) stays its `error:` or `warning:` text line on stderr under every `--format`: an error such as a bad flag, an unknown kind or project alias, an unreadable config or path, or a config the workspace refuses ([§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan)), and a warning carried in the run's warning channel, such as [§FS-check.4.10](FS-check.md#410-include_root--false-leaves-the-blocks-own-files-unread)'s and [§FS-workspace.6.1.7](FS-workspace.md#617-a-claiming-block-that-cannot-answer-fails-the-run)'s. What decides is which channel carries the fact: a warning settled from the loaded config but carried as one of `check`'s report warnings, such as [§FS-check.4.3](FS-check.md#43-redundant-config-pair)'s and [§FS-check.4.11](FS-check.md#411-config-read-from-the-deprecated-agents-location)'s, is data, and renders as a JSON finding ([§FS-errors.5.2.3](FS-errors.md#523-run-level-findings-in-checks-report)).

#### 5.2.3 Run-level findings in `check`'s report

A run-level finding in `grund check`'s report is about the run rather than the graph, such as the empty-scan warning ([§FS-check.2.2](FS-check.md#22-empty-scan)), the nothing-recognized warning ([§FS-check.4.5](FS-check.md#45-nothing-recognized)), or a per-file read failure collected mid-scan, and it is on stderr in both formats.

Under JSON, a warning raised at a line of a version-2 config file ([§FS-config-v2.checks](FS-config-v2.md#checks-checks)) keeps its `warning: <path>:<line>:` text on stderr and carries that `path` and `line` as fields too, with `sites` `null`, because a program needs the line as a field; every other run-level warning carries `path`, `line` and `sites` `null`, any location being in its message ([§FS-check.4.5](FS-check.md#45-nothing-recognized)).

Selection treats two run-level lines apart: a rule's line saying it applies to nothing is its rule's diagnostic, so it carries that rule's `authority` and a run scoped to the trial sentence keeps it ([§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing)), and the selected-check-off line is about the selection itself, so it passes both axes ([§FS-check.1.4.1](FS-check.md#141-a-selected-check-the-run-leaves-off-says-so)). Every other run-level finding has `authority` `null`, because no rule authored it, so `--only-rule` drops it just as the code axis does, `--ignore empty-scan` by name and `--only <any other code>` by omission, and of those only a mid-scan failure is beyond either axis ([§FS-check.2.1.2](FS-check.md#212-selection-filters-the-complete-report)). An author who scopes a trial sentence over a tree that scanned nothing therefore gets `success` at exit `0`, and the warning that nothing was scanned is dropped with the rest.

### 5.3 `show --batch`

`show --batch` keeps its stream exception in JSON ([§FS-errors.1](FS-errors.md#1-streams)): every well-formed query, a failed one included, is one record of one ordered, hint-free envelope on stdout ([§FS-output-shapes.4.1](FS-output-shapes.md#41-show---batch---formatjson)). Its stderr carries only invocation, batch-input, configuration and scan failures, which abort the run. A batch-input error is the raw text `error: batch input line <N>: <reason>` and leaves stdout empty, because the whole input stream is validated before the scan.

### 5.5 The `check` code catalog

For `grund check`, the codes of this table are the exact public selector vocabulary of
[§FS-check.1](FS-check.md#1-inputs), but for a row marked planned. The table is sorted by code, and a code's row is the
one place the specification states what the code means, its severity, the release a ramp moves
that severity in, what turns it on, what clears its finding, and which section specifies it.
That section says what the check judges and names the code, leaving these cells to the row
([§REQ-spec-section-names.code](../requirements/REQ-spec-section-names.md#code-a-check-is-named-by-its-diagnostic-code)). `grund config explain <code>` prints a row as the config in
force reads it ([§FS-config-explain.4](FS-config-explain.md#4-a-codes-answer)), and a finding's message carries its *Fix* cell. How each
cell is written and read is [§FS-errors.5.5.1](FS-errors.md#551-how-a-row-is-read).

| Code | Meaning | Severity | Ramp | Enabled by | Fix | Note | Check |
|---|---|---|---|---|---|---|---|
| `agents-init` | An agent entrypoint such as `AGENTS.md` lacks the `grund init` block it needs, or carries one this grund does not accept. | error | — | — | outdated: run `grund init` to update to v{version} ‖ missing: run `grund init` to install it ‖ refresh: run `grund init` to refresh ‖ refresh-in: run `grund init` in {dir} to refresh ‖ elsewhere: run it in {dir} | — | [§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block) |
| `broken-stub` | A stub's link does not lead to a declaration of its ID. | error | — | — | v2: one-home: {fix of `duplicate`, case `there`} ‖ excluded-pattern: move the target out of what {pattern} matches, or take {pattern} out of [schema.sources] exclude ‖ ignored: move the target out of what {file} ignores, or stop ignoring it there · v1: one-home: {fix of `duplicate`, case `there`} ‖ excluded-entry: move the target out of {entry}, or take {entry} out of [scan] exclude ‖ ignored: move the target out of what {file} ignores, or stop ignoring it there | — | [§FS-declarations.checks.broken-stub](FS-declarations.md#checksbroken-stub-broken-inline-spec-stub) |
| `chapter-cardinality` | A declaration has more or fewer chapters of a name than a rule's `have` sentence allows. | error, or a suggestion for a `should` or `should not` rule | — | v2: `form = "rule"` is set on a `[schema.kinds]` table or `--rule` is passed · v1: `rules = true` is set on a `[[kinds]]` row or `--rule` is passed | count: add, remove or retitle chapters until the count fits ‖ named: write line {line} as {heading} | — | [§FS-check.3.26](FS-check.md#326-chapter-cardinality) |
| `citation-cardinality` | A declaration cites a target more or fewer times than a rule allows. | error, or a suggestion for a `should` or `should not` rule | — | v2: `form = "rule"` is set on a `[schema.kinds]` table or `--rule` is passed · v1: `rules = true` is set on a `[[kinds]]` row or `--rule` is passed | add or remove those citations until the count fits | — | [§FS-check.3.27](FS-check.md#327-citation-cardinality) |
| `config-fetch-names-nothing` | A `fetch` key in a version-2 config named no executable, and was read for one more release as a kind whose fetch could not run; such a key is now refused when the config loads. | none — spent from grund 0.23.0 and never fires | — | — | none — it never fires | — | [§FS-config-v2.checks.config-fetch-names-nothing](FS-config-v2.md#checksconfig-fetch-names-nothing-a-rows-fetch-names-no-executable) |
| `config-index-not-markdown` | A version-2 row's `index` named a file the configuration did not read as Markdown, and its folders were judged as under `index = false` for one more release; such an `index` is now refused when the config loads. | none — spent from grund 1.0.0 and never fires | — | — | none — it never fires | — | [§FS-config-v2.checks.config-index-not-markdown](FS-config-v2.md#checksconfig-index-not-markdown-a-named-index-is-not-read-as-markdown) |
| `config-md-not-markdown` | In version 2, an `md` entry of `[schema.sources.extensions]` mapped `.md` to a language that did not read Markdown, and was read that way for one more release; such an entry is now refused when the config loads. | none — spent from grund 1.0.0 and never fires | — | — | none — it never fires | — | [§FS-config-v2.checks.config-md-not-markdown](FS-config-v2.md#checksconfig-md-not-markdown-md-is-mapped-away-from-markdown) |
| `config-multiline-string` | A value in a version-2 config held a multi-line string, which version 2 does not read, and was read for one more release as before; such a value is now refused when the config loads. | none — spent from grund 0.22.0 and never fires | — | — | none — it never fires | — | [§FS-config-v2.checks.config-multiline-string](FS-config-v2.md#checksconfig-multiline-string-a-version-2-config-value-holds-a-multi-line-string) |
| `config-not-toml` | A version-2 config held text TOML refuses that grund read for one more release as the readers before it did; such text is now refused when the config loads. | none — spent from grund 0.22.0 and never fires | — | — | none — it never fires | — | [§FS-config-v2.checks.config-not-toml](FS-config-v2.md#checksconfig-not-toml-a-version-2-config-holds-text-toml-refuses) |
| `config-rule-file-not-markdown` | A version-2 rule kind's `files` entry was read by a `source` language, so a rule written as a Markdown heading there declared nothing, for one more release; such an entry is now refused when the config loads. | none — spent from grund 1.0.0 and never fires | — | — | none — it never fires | — | [§FS-config-v2.checks.config-rule-file-not-markdown](FS-config-v2.md#checksconfig-rule-file-not-markdown-a-rule-kinds-files-entry-is-read-as-source) |
| `config-rule-folder-not-markdown` | A version-2 rule kind's `folders` entry could hold no file read as Markdown, because no admitted language read Markdown, and loaded for one more release; such an entry is now refused when the config loads. | none — spent from grund 1.0.0 and never fires | — | — | none — it never fires | — | [§FS-config-v2.checks.config-rule-folder-not-markdown](FS-config-v2.md#checksconfig-rule-folder-not-markdown-a-rule-kinds-folder-can-hold-no-markdown) |
| `config-value-home-absent` | A version-2 value row's later home did not exist, and the row read its other homes for one more release; such a home is now refused when the config loads. | none — spent from grund 1.0.0 and never fires | — | — | none — it never fires | — | [§FS-config-v2.checks.config-value-home-absent](FS-config-v2.md#checksconfig-value-home-absent-a-value-rows-later-home-does-not-exist) |
| `config-value-home-invalid` | A version-2 value row's later home was the wrong shape, resolved outside the root or could not be looked up, and was read as before for one more release; such a home is now refused when the config loads. | none — spent from grund 1.0.0 and never fires | — | — | none — it never fires | — | [§FS-config-v2.checks.config-value-home-invalid](FS-config-v2.md#checksconfig-value-home-invalid-a-value-rows-later-home-is-the-wrong-shape-outside-the-root-or-not-looked-up) |
| `config-version-1` | A config file is read as version 1; the warning names the command that migrates it. | v2: not in version 2 · v1: warning | — | v2: not in version 2 · v1: a config file is read as version 1 | migrate {files} with grund config migrate --to 2 | — | [§FS-check.checks.config-version-1](FS-check.md#checksconfig-version-1-a-config-file-read-as-version-1) |
| `conflict-marker` | A scanned file holds an unresolved merge-conflict block. | error | — | — | resolve it by hand | — | [§FS-declarations.checks.conflict-marker](FS-declarations.md#checksconflict-marker-unresolved-merge-conflict) |
| `dangling` | A citation names an ID that no declaration declares. | error | — | — | near: did you mean {near}? ‖ near-escape: did you mean {near}? (or write {escape} if this is an illustration) ‖ escape: write {escape} if this is an illustration ‖ fetch: run grund fetch {id} ‖ value-source: declare it in that value source ‖ source-file: declare it in that source file ‖ source-file-value: move the kind to a Markdown file or folder ‖ fetch-names-nothing: name the program, or if this kind is not fetched remove the key and any {setting}, and its missing IDs are then errors ‖ pattern: a place is a path from the config root, not a pattern, so list each file or name the folder | — | [§FS-check.3.1](FS-check.md#31-dangling-citation) |
| `declaration-near-miss` | A heading opens like a declaration, but its ID does not match its kind's ID format. | error | — | — | rename it or change the effective format | — | [§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss) |
| `deprecated-config-location` | The config was read from `.agents/grund.toml`, a location that still works but is deprecated. | warning | — | — | move it to {home} | — | [§FS-check.4.11](FS-check.md#411-config-read-from-the-deprecated-agents-location) |
| `discouraged-citation` | A citation goes to a target that a `should-not` entry or rule discourages. | none — suggestion | — | v2: `--suggestions` is passed and `[rules.citations.<kind>] should-not` or a `should not` rule is set · v1: `--suggestions` is passed and `[citations.<kind>] should-not` or a `should not` rule is set | {fix of `forbidden-citation`, case `re-point`} | — | [§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in) |
| `duplicate` | The same ID is declared more than once. | error | — | — | rename: keep one declaration of the ID and rename or remove the others ‖ home: point the stub link at {site} to that real home ‖ there: point the stub link there | — | [§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration) |
| `duplicate-section` | Two section headings in one declaration claim the same section path. | error | — | — | headings: renumber or rename all but one of the headings ‖ keys: remove all but one of the keys | — | [§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path) |
| `empty-citation-obligation` | A `must` or `should` citation obligation applies to a kind that has no unit for it to judge. | warning | — | v2: `[rules.citations.<kind>] must` or `should`, or a `must` or `should` rule, is set for a scanned kind with a folder home · v1: `[citations.<kind>] must` or `should`, or a `must` or `should` rule, is set for a scanned kind with a folder home | v2: citable: did you mean `citable = false`? ‖ grounding: set `must = "file"` in [rules.citations.{kind}.grounding] to make that an error · v1: citable: did you mean `citable = false`? ‖ grounding: set require_grounding = true on the {place} row to make that an error | — | [§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing) |
| `empty-scan` | The scan read no files at all, which usually means its scope is misconfigured. | warning | — | — | v2: hidden: Rename it, or move what needs checking into a file that is not hidden ‖ admit: admit one in [schema.sources] languages, such as `markdown` ‖ unmap: [schema.sources.extensions] maps `{ext}` to `{language}`: drop that entry · v1: hidden: Rename it, or move what needs checking into a file that is not hidden ‖ no-kinds: replace `kinds = []` in `grund.toml` with a `[[kinds]]` table for its first kind, point `[scan] include` at your sources, or pass a path explicitly (`grund check <dir>`) ‖ root: Run `grund init --scaffold` to scaffold the canonical docs/ and e2e/ trees, point `[scan] include` in `grund.toml` at your sources, or pass a path explicitly (`grund check <dir>`) | — | [§FS-check.2.2](FS-check.md#22-empty-scan) |
| `escaped-citation-resolves` | An escaped citation names an ID or section that exists, so it may be a live citation escaped by mistake. | none — suggestion | — | `--suggestions` is passed | write {citation} for a live citation, or leave it escaped if it is only an illustration | — | [§FS-check.2.3.1](FS-check.md#231-escaped-citation-resolves) |
| `excluded-stub-target` | A stub led to a file the scan excludes; that case is now reported as a broken stub. | none — spent from grund 0.19.0; select `broken-stub` instead | — | — | none — it never fires | — | [§FS-declarations.checks.excluded-stub-target](FS-declarations.md#checksexcluded-stub-target-stub-to-a-target-the-walk-excludes-a-spent-check) |
| `forbidden-citation` | A citation goes to a target that a `must-not` entry or rule forbids. | v2: by strength: `must-not` an error, `warn-not` a warning · v1: error | — | v2: `[rules.citations.<kind>] must-not` or a `must not` rule is set; a `warn-not` entry turns it on too, as a warning · v1: `[citations.<kind>] must-not` or a `must not` rule is set | re-point: re-point the citation or downgrade it to a plain Markdown link ‖ index-rule: retain the marked index entry and change the prohibition in rule {rules} ‖ index-may: retain the marked index entry and add {kind} to the may list in {table}, preserving existing permissions ‖ index-prohibition: retain the marked index entry and change the matching {level} prohibition in {table} | — | [§FS-check.3.12](FS-check.md#312-forbidden-citation) |
| `front-matter-heading` | A heading claims the front-matter field's coordinate in a declaration whose kind reads that field from the file's front matter. | v2: error · v1: not in version 1 | — | v2: `[schema.kinds.<kind>.fields.front-matter]` is set · v1: not in version 1 | rename the section | The heading declares nothing, so it is no section and no rule unit. | [§FS-fields.checks.front-matter-heading](FS-fields.md#checksfront-matter-heading-front-matter-heading) |
| `full-scope-ignored` | `--full` was given an explicit path, which it cannot widen, so it changed nothing. | warning | — | `--full` is passed; with an explicit path | drop --full, or drop the path to check the whole project | — | [§FS-check.1.3.7](FS-check.md#137-a-path-the-flag-cannot-widen-earns-a-caution-not-a-refusal) |
| `glob-citation` | A marked citation is written as a pattern rather than as one ID. | error | — | — | cite the point, or escape it as {escape} | — | [§FS-check.checks.glob-citation](FS-check.md#checksglob-citation-glob-citation) |
| `image-index-entry` | A citation-only image was all a kind's index held for an ID; such an image is now neither an entry nor an enrollment. | none — spent from grund 0.20.0; select `missing-index-entry` instead | — | — | none — it never fires | — | [§FS-citations.checks.image-index-entry](FS-citations.md#checksimage-index-entry-a-citation-only-image-is-no-index-entry-a-spent-check) |
| `import-live` | An import is read live from the tree at its path, which no commit pins. | warning | — | v2: `[workspace.imports]` is set; in any config the run reads, a dependency's included · v1: not in version 1 | none — nothing here is broken: the verdict reads the tree the import names, and the line says that no commit pins it | — | [§FS-imports.checks.import-live](FS-imports.md#checksimport-live-an-import-is-read-live-unpinned) |
| `import-missing` | An import's path names no directory, so the citations into it were not checked. | error | — | v2: `[workspace.imports]` is set; in any config the run reads, a dependency's included · v1: not in version 1 | none — the build places the tree an import names, not grund; once the checkout holds it, the citations into it are checked | — | [§FS-imports.checks.import-missing](FS-imports.md#checksimport-missing-an-import-names-no-directory) |
| `inline-citation-style` | A citation's inline note in a code comment breaks the configured note style: too many lines, too wide, the wrong layout, or prose where only the citation is allowed. | v2: by strength: the `[schema.notes.lines]`, `[schema.notes.columns]`, `[schema.notes.layout]` or `[schema.notes.text]` key it breaks · v1: error; a warning for an overrun of `[reference] inline_note_suggested_lines` under `warn_on_suggested = true`, or for a layout deviation under `inline_note_layout_check = "warn"` | — | — | prose: move the prose into a comment block of its own ‖ lines: shorten the note, or move its detail into a doc-comment ‖ columns: shorten or rewrap its widest line ‖ layout: rewrite the line in that form | — | [§FS-check.3.10](FS-check.md#310-inline-citation-style-violation) |
| `invalid-field-value` | A field's value is not one of the values its kind admits for it. | v2: by strength: the field's `presence`: `must` an error, `warn` a warning, `should` a suggestion, `may` an error · v1: not in version 1 | — | v2: `[schema.kinds.<kind>.fields.<field>] one_of` is set · v1: not in version 1 | heading: retitle the heading to one of them ‖ value: change the value to one of them ‖ one-line: write one of them on the key's line | — | [§FS-fields.checks.invalid-field-value](FS-fields.md#checksinvalid-field-value-invalid-field-value) |
| `invalid-rule` | A rule declaration cannot be read: its title, rationale, vocabulary or subject is invalid. | error | — | v2: `form = "rule"` is set on a `[schema.kinds]` table or `--rule` is passed · v1: `rules = true` is set on a `[[kinds]]` row or `--rule` is passed | form: accepted form: {form} ‖ forms: accepted forms: {forms} ‖ form-enabled: accepted form after enabling it: {form} ‖ forms-enabled: accepted forms after enabling it: {forms} ‖ place: express this obligation in grund.toml: [{table}] with must = ["{target}"] ‖ place-grounding: express this obligation in grund.toml: [{table}] with must = ["{target}"], and {setting}, without which a file there that cites nothing is not checked ‖ rename: rename the chapter ‖ workspace: check from the workspace root | — | [§FS-check.3.25](FS-check.md#325-invalid-rule) |
| `invalid-section-lock` | A line of a `.grund/sections/` lock file is not an entry grund can read. | error | — | a `.grund/sections/` lock file exists | to rebuild the file, {fix of `unrecorded-section-citation`} | — | [§FS-section-lock.checks.invalid-section-lock](FS-section-lock.md#checksinvalid-section-lock-a-lock-line-grund-cannot-read) |
| `invalid-value-binding` | A value binding breaks the binding grammar or aims at something it cannot bind. | error | — | v2: `form = "value"` is set on a `[schema.kinds]` table or `content = "values"` on one of its fields; a value tag the scan finds turns it on too · v1: `values = true` or `value_chapter` is set on a `[[kinds]]` row; a value tag the scan finds turns it on too | join the literal and its citation | — | [§FS-check.3.21](FS-check.md#321-invalid-value-binding) |
| `invalid-value-declaration` | A declaration of a kind that holds values is not a valid value. | error | — | v2: `form = "value"` is set on a `[schema.kinds]` table or `content = "values"` on one of its fields; a value tag the scan finds turns it on too · v1: `values = true` or `value_chapter` is set on a `[[kinds]]` row; a value tag the scan finds turns it on too | correct what the reason names | — | [§FS-check.3.20](FS-check.md#320-invalid-value-declaration) |
| `io` | A file or directory could not be read, so the run's view of the tree is incomplete. | error | — | — | v2: entry: make the path readable, or keep it out of the scan ‖ root: make the path readable, or keep it out of the scan by removing it from the [schema.kinds] row that lists it, because exclude does not prune scan roots ‖ passed: make the path readable, or keep it out of the scan by not passing it, because exclude does not prune a path the command is given ‖ value: make the path readable, or keep it out of the scan by moving it out of the value home, because exclude does not reach a value source ‖ readable: make the path readable · v1: entry: make the path readable, or keep it out of the scan ‖ root: make the path readable, or keep it out of the scan by removing it from the [scan] include or [[kinds]] row that lists it, because exclude does not prune scan roots ‖ passed: make the path readable, or keep it out of the scan by not passing it, because exclude does not prune a path the command is given ‖ value: make the path readable, or keep it out of the scan by moving it out of the value home, because exclude does not reach a value source ‖ readable: make the path readable | — | [§FS-check.2.4.3](FS-check.md#243-what-failed-and-where-decide-the-code-and-the-fix) |
| `local-section-citation` | A marker followed by a bare section number is not a citation grund can resolve; write the full ID. | error | — | — | write: write {citation} ‖ escape: write a full citation or {escape} to show the shape without citing it ‖ fmt: {fix of `unlinked-index-entry`} | — | [§FS-check.3.24](FS-check.md#324-declaration-local-section-citation) |
| `misplaced-declaration` | A declaration sits outside the home its kind is configured to live in. | error | — | v2: `folders` or `files` is set on a `[schema.kinds]` table · v1: `folder` or `file` is set on a `[[kinds]]` row | v2: move: move it to a home of its own kind ‖ unlisted: add "{entry}" to [workspace] members in {enclosing}, or exclude "{entry}/" with [schema.sources] exclude in {enclosing} ‖ roots: also remove configured scan roots at or beneath "{entry}" in {enclosing} (source includes or kind homes), because exclude does not prune scan roots · v1: move: move it to a home of its own kind ‖ unlisted: add "{entry}" to [workspace] members in {enclosing}, or exclude "{component}" with [scan] exclude in {enclosing} (excludes all directories named "{component}" at every depth) ‖ roots: also remove configured scan roots at or beneath "{entry}" in {enclosing} (source includes or kind homes), because exclude does not prune scan roots | — | [§FS-declarations.checks.misplaced-declaration](FS-declarations.md#checksmisplaced-declaration-misplaced-declaration-configured-kind-home) |
| `misplaced-field` | A field does not stand where its kind places it. | v2: by strength: the field's `presence`: `must` an error, `warn` a warning, `should` a suggestion, `may` an error · v1: not in version 1 | — | v2: `[schema.kinds.<kind>.fields.<field>] position` or `after` is set · v1: not in version 1 | move the section there | — | [§FS-fields.checks.misplaced-field](FS-fields.md#checksmisplaced-field-misplaced-field) |
| `missing-citation` | A declaration lacks a citation that a `must` entry or rule requires. | v2: by strength: `must` an error, `warn` a warning · v1: error | — | v2: `[rules.citations.<kind>] must` or a `must` rule is set; a `warn` entry turns it on too, as a warning · v1: `[citations.<kind>] must` or a `must` rule is set | add such a citation within it | — | [§FS-check.3.11](FS-check.md#311-missing-required-citation) |
| `missing-field` | A declaration lacks a field its kind declares. | v2: by strength: the field's `presence` · v1: not in version 1 | — | v2: `[schema.kinds.<kind>.fields.<field>] presence = "must"` is set; a `warn` or `should` `presence` turns it on too, at that strength · v1: not in version 1 | chapter: add its {field} section ‖ block: add front matter at the top of its file ‖ not-first: only a file's first declaration has front matter; move it to a file of its own ‖ doc-comment: only a Markdown file's first declaration has front matter; move it to a Markdown file of its own ‖ byte-order-mark: remove the byte-order mark before the {delimiter} on line 1 ‖ key: add the key to the front matter | — | [§FS-fields.checks.missing-field](FS-fields.md#checksmissing-field-missing-field) |
| `missing-index-entry` | A declaration in a kind's folder is not listed in that folder's index. | error | — | — | write `{entry}` in {index} | — | [§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index) |
| `missing-section` | A citation names a section its declaration does not have. | error | — | — | escape: write {escape} before it to show the shape without citing it ‖ map: run `grund {id} --toc` to print the lead with the section map | — | [§FS-check.3.2](FS-check.md#32-missing-section) |
| `missing-snapshot` | A citation targets a fetched kind whose declarations have not been fetched yet. | warning | — | v2: `[rules.resolution] <KIND> = "warn"` is set for a fetched kind · v1: `resolve = "should"` is set on a fetched `[[kinds]]` row | near: {fix of `dangling`, case `near`} ‖ near-escape: {fix of `dangling`, case `near-escape`} ‖ escape: {fix of `dangling`, case `escape`} ‖ fetch: {fix of `dangling`, case `fetch`} ‖ value-source: {fix of `dangling`, case `value-source`} ‖ source-file: {fix of `dangling`, case `source-file`} ‖ source-file-value: {fix of `dangling`, case `source-file-value`} ‖ fetch-names-nothing: {fix of `dangling`, case `fetch-names-nothing`} | — | [§FS-check.4.12](FS-check.md#412-missing-snapshot) |
| `mistitled-field` | A field's heading is titled differently from the title its kind gives it. | v2: by strength: the field's `presence`: `must` an error, `warn` a warning, `should` a suggestion, `may` an error · v1: not in version 1 | — | v2: `[schema.kinds.<kind>.fields.<field>] title` is set · v1: not in version 1 | retitle the heading to match | — | [§FS-fields.checks.mistitled-field](FS-fields.md#checksmistitled-field-mistitled-field) |
| `nothing-recognized` | The scan read files but recognized no declaration and no citation in them. | warning | — | — | Declare its first kind in grund.toml to start | — | [§FS-check.4.5](FS-check.md#45-nothing-recognized) |
| `off-format-citation` | A marked citation opens with a configured kind, but its kind's ID format does not spell it and nothing declares it. | error | — | — | cite an ID in that format, or write {escape} for an illustration | — | [§FS-check.checks.off-format-citation](FS-check.md#checksoff-format-citation-off-format-citation) |
| `optional-member-absent` | A workspace member listed as optional is not in this checkout, so it was not read. | warning | — | `[workspace] optional_members` is set; in any `[workspace]` block the run expands, a nested one included | none — nothing here is broken: the member is optional, and only a checkout that holds it can check the citations into it | — | [§FS-check.4.9](FS-check.md#49-a-workspace-member-declared-optional-is-absent) |
| `orphan-section` | A named section path is used before every shorter prefix of it is recorded in the same declaration. | error | — | v2: — · v1: `[id] named_sections = true` is set | declare the prefix as a heading above it | — | [§FS-declarations.checks.orphan-section](FS-declarations.md#checksorphan-section-orphan-name-bearing-section-path) |
| `out-of-scope-dangling` | Under `--full`, a file outside the default scope cites an ID that no declaration declares. | error | — | `--full` is passed | {fix of `dangling`} | — | [§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only) |
| `out-of-scope-local-section-citation` | Under `--full`, a file outside the default scope writes a marker followed by a bare section number. | error | — | `--full` is passed | write: {fix of `local-section-citation`, case `write`} ‖ escape: {fix of `local-section-citation`, case `escape`} | — | [§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only) |
| `out-of-scope-missing-section` | Under `--full`, a file outside the default scope cites a section its declaration does not have. | error | — | `--full` is passed | {fix of `missing-section`} | — | [§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only) |
| `out-of-scope-off-format-citation` | Under `--full`, a file outside the default scope writes a marked citation its kind's ID format does not spell and nothing declares. | error | — | `--full` is passed | {fix of `off-format-citation`} | — | [§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only) |
| `out-of-scope-shorthand-citation` | Under `--full`, a file outside the default scope writes a number-only shorthand that matches no declaration or several. | error | — | `--full` is passed | {fix of `shorthand-citation`, case `full`} | — | [§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only) |
| `out-of-scope-unknown-project` | Under `--full`, a file outside the default scope cites through a project alias the workspace does not know. | error | — | `--full` is passed | {fix of `unknown-project`} | — | [§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only) |
| `oversized-lead` | A declaration's lead is longer than the configured word limit. | v2: by strength: `[schema.leads.words]` · v1: warning | — | v2: `[schema.leads.words] warn = <N>` is set; a `must` or `should` limit turns it on too, at that strength · v1: `[reference] lead_size_warning = { max = <N>, unit = "words" }` is set | move detail into citable child sections, or promote a child section to its own ID after running grund refs {coordinate} --summary | — | [§FS-declarations.checks.oversized-lead](FS-declarations.md#checksoversized-lead-oversized-lead-opt-in) |
| `reading-conflict` | Two in-tree names of a file whose own name claims no reading read it differently, so the run reads it under neither. | error | — | two in-tree names of a file whose own name claims no reading read differently | keep one name, or give the file a name that claims a reading | Neither axis of selection hides it, since a hidden one is a green run over an unread file. A citation elsewhere to what the file declares is also reported. | [§FS-scan.checks.reading-conflict](FS-scan.md#checksreading-conflict-two-names-of-one-file-read-it-differently) |
| `reading-moved` | A link is read as its target's own name reads, which differs from what the link's name read it as before grund 0.22.0. | warning | — | a scanned link's target has a name that claims another reading | rename the target or copy the file | It stands for one release and leaves the exit code alone. | [§FS-scan.checks.reading-moved](FS-scan.md#checksreading-moved-a-link-is-read-as-its-targets-own-name-reads) |
| `redundant-config` | A directory holds both `grund.toml` and `.agents/grund.toml`, and only the first is read. | warning | — | — | delete one | — | [§FS-check.4.3](FS-check.md#43-redundant-config-pair) |
| `relative-section-parent` | A relative section heading has no parent section one level up for its path to extend. | error | — | — | give the heading the parent it needs, or write its complete path | — | [§FS-declarations.checks.relative-section-parent](FS-declarations.md#checksrelative-section-parent-a-relative-heading-without-its-parent) |
| `remote-missing` | A declared remote project has no pinned projection to read. | error | — | planned — a `[workspace.remotes]` table | run `grund fetch --remote {alias}` | — | [§FS-remote-projects.checks.remote-missing](FS-remote-projects.md#checksremote-missing-a-declared-remote-has-no-pinned-projection) |
| `remote-modified` | A remote project's projection is not the bytes its lock records. | error | — | planned — a `[workspace.remotes]` table | restore the projection from version control, or refetch it | — | [§FS-remote-projects.checks.remote-modified](FS-remote-projects.md#checksremote-modified-the-projection-is-not-the-locked-bytes) |
| `remote-orphan` | Remote state remains that no declaration names. | warning | — | planned — a `[workspace.remotes]` table | remove the stale state, or restore the declaration | — | [§FS-remote-projects.checks.remote-orphan](FS-remote-projects.md#checksremote-orphan-state-no-declaration-names) |
| `remote-stale` | A remote project's lock entry disagrees with its declaration. | error | — | planned — a `[workspace.remotes]` table | refetch, or restore the declaration | — | [§FS-remote-projects.checks.remote-stale](FS-remote-projects.md#checksremote-stale-the-lock-disagrees-with-the-declaration) |
| `rule-folder-file-not-markdown` | A file in a version-2 rule kind's folder is read by a `source` language, so a rule written as a Markdown heading in it declares nothing. | v2: warning · v1: not in version 1 | — | v2: a file in a `folders` entry of a `form = "rule"` table is read by a `source` language · v1: not in version 1 | v2: give it a name a `document` language reads, or move it out of {folder} · v1: not in version 1 | — | [§FS-config-v2.checks.rule-folder-file-not-markdown](FS-config-v2.md#checksrule-folder-file-not-markdown-a-file-in-a-rule-kinds-folder-is-read-as-source) |
| `section-address-changed` | A cited section's text has moved to another address, and the cited address now holds something else. | error | — | a citation names the address of a relative heading or a mixed path | move it back, cite {new address} instead, or leave a pointer to it at {cited address}; if {new address} is a copy, edit the copy or cite it from {cited address} | `check` compares the text at a cited address with the fingerprint `.grund/sections/<ID>.lock` recorded for it, and finds that fingerprint at another address of the declaration. Only `grund fmt --write` writes a lock (`grund config explain unrecorded-section-citation` says where to run it). It never approves a move: the one thing it records at the old address is a pointer to the new one. Do not edit a lock by hand. | [§FS-section-lock.checks.section-address-changed](FS-section-lock.md#checkssection-address-changed-a-cited-sections-text-sits-at-another-address) |
| `section-heading-level` | A section heading's Markdown depth does not match the depth of its section path. | v2: by strength: `[schema] heading_depth` · v1: error, or a warning under `[id] section_heading_levels = "warn"` | — | v2: `[schema] heading_depth = "must"` is set; it is the default, and `warn` or `should` turns it on too, at that strength · v1: `[id] section_heading_levels = "strict"` is set; it is the default, and `"warn"` turns it on too | re-level the heading, or change its path to match its depth | — | [§FS-declarations.checks.section-heading-level](FS-declarations.md#checkssection-heading-level-section-heading-level-mismatch) |
| `section-outside-declaration` | A section heading sits outside every declaration's body. | error | — | — | move it inside a declaration's body, or drop its section path | — | [§FS-declarations.checks.section-outside-declaration](FS-declarations.md#checkssection-outside-declaration-section-outside-a-declaration) |
| `selected-check-off` | A check that `--only` selected was off for the whole run, so it judged nothing. | warning | — | a code `--only` selects is armed by no project of the run | off: the check is off until {switches} ‖ migrate: {fix of `config-version-1`} | — | [§FS-check.1.4.1](FS-check.md#141-a-selected-check-the-run-leaves-off-says-so) |
| `shorthand-citation` | A number-only shorthand citation is written where the full ID is required, or matches no declaration or several. | error | — | — | write: write {citation} ‖ full: write the full ID it means | — | [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation) |
| `shorthand-numeric-run` | A number-only shorthand citation is glued to another number, so `grund fmt` will not expand it. | error | — | — | write {citation}, or {escape} if these are old numbers | — | [§FS-check.3.15](FS-check.md#315-shorthand-citation-in-a-numeric-run) |
| `stub-shape` | A heading whose title is one link was read as a stub by one test and as a declaration by another; that can no longer happen. | none — spent from grund 0.19.0 and never fires | — | — | none — it never fires | — | [§FS-declarations.checks.stub-shape](FS-declarations.md#checksstub-shape-stub-shape-read-two-ways-a-spent-check) |
| `suggested-citation` | A declaration lacks a citation that a `should` entry or rule suggests. | none — suggestion | — | v2: `--suggestions` is passed and `[rules.citations.<kind>] should` or a `should` rule is set · v1: `--suggestions` is passed and `[citations.<kind>] should` or a `should` rule is set | {fix of `missing-citation`} | — | [§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in) |
| `uncited-unit` | A declaration or chapter is cited more or fewer times than a rule's `be cited by` sentence allows. | error, or a suggestion for a `should` or `should not` rule | — | v2: `form = "rule"` is set on a `[schema.kinds]` table or `--rule` is passed · v1: `rules = true` is set on a `[[kinds]]` row or `--rule` is passed | add or remove the citations of it until the count fits | — | [§FS-check.3.28](FS-check.md#328-uncited-unit) |
| `undeclared-chapter` | A declaration of a closed kind has a chapter that none of the kind's fields declares. | v2: by strength: the kind's `closed` · v1: not in version 1 | — | v2: `[schema.kinds.<kind>] closed = "must"` is set; a `warn` or `should` `closed` turns it on too, at that strength · v1: not in version 1 | chapter: rename it to one of them, number it, or add it to the kind's fields ‖ key: rename it to one of them, remove it, or add it to the field's keys ‖ undeclarable-key: rename it to one of them or remove it | — | [§FS-fields.checks.undeclared-chapter](FS-fields.md#checksundeclared-chapter-undeclared-chapter) |
| `ungrounded` | A unit in a place that must be grounded neither cites nor declares an ID. | v2: by strength: the rung of the grounding ladder it falls short of · v1: error | — | v2: a `must` rung is set in `[rules.citations.<kind>.grounding]`, or in `[rules.citations.grounding]` or by `--require-grounding` for a kind with no ladder of its own; a `warn` or `should` rung turns it on too, at that strength · v1: `require_grounding = true` is set on a `[[kinds]]` row, or in `[reference]` or by `--require-grounding` for a kind whose row does not set it `false` | cite a declared ID within it | — | [§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in) |
| `unknown-project` | A citation names a project alias that is not known, or a declaration or section that project does not have. | error | — | — | root: run `grund check` from the workspace root at `{root}` to validate cross-project citations ‖ subtree: check from the workspace root for a path outside that subtree ‖ near: did you mean {near}? ‖ unlisted: add "{entry}" to [workspace] members in {enclosing} | — | [§FS-check.3.8](FS-check.md#38-cross-project-citation-failure) |
| `unlinked-index-entry` | An index entry cites a declaration bare rather than as a Markdown link. | error | — | — | run `grund fmt --write` | — | [§FS-check.3.17](FS-check.md#317-index-entry-is-not-a-link) |
| `unlisted-workspace-block` | A directory declares `[workspace]`, but no enclosing workspace lists it among its members. | error | — | — | add "{entry}" to [workspace] members in {enclosing}, or keep it out of that project's {scan} | — | [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) |
| `unmarked-heading` | A heading inside a declaration's body is neither a declaration nor a recognized section heading. | error | — | — | number it ({heading}) as {address}, declare an ID, or use a bold label | — | [§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading) |
| `unparsable-file` | A file was read and does not parse, so the run's view of the tree is incomplete. | error | — | — | exit: write the case's expected exit status in the file as one integer ‖ value: make the file valid {format} ‖ ignore: write the line it names as a valid ignore pattern | — | [§FS-check.2.4.3](FS-check.md#243-what-failed-and-where-decide-the-code-and-the-fix) |
| `unreached-declaration` | A declaration lacks the chapter a chapter-scoped citation rule over its kind needs in order to reach it. | error, or a suggestion for a `should` or `should not` rule | — | v2: `form = "rule"` is set on a `[schema.kinds]` table or `--rule` is passed · v1: `rules = true` is set on a `[[kinds]]` row or `--rule` is passed | add the chapter, or narrow the rule to the declarations that have one | — | [§FS-rules.checks.unreached-declaration](FS-rules.md#checksunreached-declaration-unreached-declaration) |
| `unread-scan-root` | A file or folder the config names as a scan root is not read, because a file's name is hidden or its extension is not read for declarations. | warning | — | v2: a `files` entry of a `[schema.kinds]` table names a file that is not read for its name, or a `folders` entry names a folder whose files are all unread, one refused for its name · v1: a `[[kinds]]` `file` home or a `[scan] include` entry names a file that is not read for its name, or a `[[kinds]]` `folder` home names a folder whose files are all unread, one refused for its name | v2: language: Give the file an extension a listed language reads ‖ hidden: Rename the file to a name that is not hidden ‖ hidden-language: Rename the file to a name that is not hidden and an extension a listed language reads ‖ hidden-document: Rename the file to a name that is not hidden and an extension a listed document language reads ‖ document-admit: Add `markdown` to [schema.sources] languages, keeping the existing languages, and {rename} ‖ value-repair: Set `form = "value"` on the row, or {repair} ‖ value: Set `form = "value"` on the row, or give the file an extension a listed document language reads ‖ document: Give the file an extension a listed document language reads ‖ folder-add: Add `{language}` to [schema.sources] languages, or give the files in it an extension a listed language reads ‖ folder-add-document: Add `{language}` to [schema.sources] languages, or give the files in it an extension a listed document language reads ‖ folder-language: Give the files in it an extension a listed language reads ‖ folder-document: Give the files in it an extension a listed document language reads ‖ folder-value: Set `form = "value"` on the row, or give the files in it an extension a listed document language reads ‖ drop: drop the entry ‖ place: set `scan = false` on the row if it only lists a place · v1: add: Add "{ext}" to [scan] extensions ‖ drop: drop the entry ‖ rename: Rename the file to an extension the list holds ‖ give: give the file an extension the list holds ‖ hidden: Rename the file to a name that is not hidden ‖ hidden-list: Rename the file to a name that is not hidden and an extension the list holds ‖ folder-rename: Rename the files in it to an extension the list holds ‖ place: set `scan = false` on the row if it only lists a place | — | [§FS-scan.checks.unread-scan-root](FS-scan.md#checksunread-scan-root-a-configured-file-root-is-not-read) |
| `unreadable-front-matter` | The front matter of a declaration whose kind declares a `front-matter` field cannot be read as YAML or TOML keys. | v2: error · v1: not in version 1 | — | v2: `[schema.kinds.<kind>.fields.front-matter]` is set · v1: not in version 1 | yaml: fix the YAML ‖ toml: fix the TOML | The field is present, so no `missing-field` is reported beside it, and none of its keys is judged. | [§FS-fields.checks.unreadable-front-matter](FS-fields.md#checksunreadable-front-matter-unreadable-front-matter) |
| `unrecorded-section-citation` | A citation names a locked section address that has no entry in the section lock. | error | — | a citation names the address of a relative heading or a mixed path | run grund fmt --write with no path at the workspace root, or at the project root outside a workspace | — | [§FS-section-lock.checks.unrecorded-section-citation](FS-section-lock.md#checksunrecorded-section-citation-a-locked-address-the-lock-does-not-hold) |
| `unused` | A declared ID is never cited. | warning | — | — | cite it | — | [§FS-check.4.1](FS-check.md#41-unused-declaration) |
| `value-mismatch` | A value binding's written value differs from the value its target declares. | error | — | v2: `form = "value"` is set on a `[schema.kinds]` table or `content = "values"` on one of its fields; a value tag the scan finds turns it on too · v1: `values = true` or `value_chapter` is set on a `[[kinds]]` row; a value tag the scan finds turns it on too | change the literal to the declared value, or the declaration to the bound one | — | [§FS-check.3.22](FS-check.md#322-value-mismatch) |
| `yaml-value-source-deferred` | A finding at a YAML value source was a warning for one release, and a fetched kind's home skipped its YAML sources; such a source is now read, and each finding carries its own code and severity. | none — spent from grund 0.22.0 and never fires | — | — | none — it never fires | — | [§FS-values.2.2.3](FS-values.md#223-how-a-yaml-source-is-read) |

Every `check` finding code has a row here from the release that introduces it, and renaming or
removing one takes the compatibility treatment of [§REQ-backwards-compatibility.1](../requirements/REQ-backwards-compatibility.md#1-what-is-covered). No selector
hides `io`, `unparsable-file`, or the other codes [§FS-check.2.1.2](FS-check.md#212-selection-filters-the-complete-report) puts beyond both axes.

#### 5.5.1 How a row is read

Every cell but *Check* is printed by the installed `grund`: `grund config explain`
answers for a code from its row ([§FS-config-explain.4](FS-config-explain.md#4-a-codes-answer)), the off line of
[§FS-check.1.4.1](FS-check.md#141-a-selected-check-the-run-leaves-off-says-so) is built from its *Enabled by* and *Severity* cells, and a
finding's message carries its *Fix* cell filled in. So no cell
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
  became of it. `by strength:` is chosen by the strength a setting or rule is
  written under. A cell that lists what the strengths give lists every strength
  the code reports, and a strength it leaves out is not reported by this code.
  A cell that only names the setting takes the default: `must` an error, `warn`
  a warning, `should` a suggestion, and `may` not checked. Any other cell
  names a severity and the setting or the rule strength that changes it, so it
  is chosen by a setting too.
- **Ramp** names the release in which a promised change of severity lands, and
  is `—` where none is promised, a spent ramp included.
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
- **Fix** says what clears the finding, and no row leaves it blank. It is one
  of three things. A **template** is the remedy as the message prints it. A
  code whose message gives one of several remedies by context writes **cases**,
  `<label>: … ‖ <label>: …`, one template per remedy, each label a lowercase
  word or hyphenated words. **`none — <why>`** is a code that nothing the user
  does clears, and the rest of the cell says why; it is never an invented
  remedy.
- **Inside a template**, `{words}`, lowercase words in braces, is a slot the
  finding fills, such as `{new address}`, and `` {fix of `<code>`} `` stands
  for that row's *Fix* template, worded once there: its side of the same
  version, or its other side where that one is `not in version N`, and
  `` {fix of `<code>`, case `<label>`} `` stands for that side's one case so
  labelled. A *Fix* side that is nothing but `` {fix of `<code>`} `` is that
  row's side whole, cases and all, so a code whose remedies are another's
  names them rather than restating them. A reference to a code the catalog
  lacks, to a case its side does not name, or to a side that is `none`, or
  cases where one template is meant, is refused. A template holds no other
  brace, so a brace without its partner is refused, and text a message prints
  in braces is a slot's value. Backticks in a template are the message's own
  bytes and are kept, unlike any other cell's.
- **Note** is `—`, or plain sentences of what to know before applying the fix:
  what the check compares, what writes what it reads, what never to do instead.
- A *Fix* or *Note* cell may split by version as the cells above do, and `not
  in version N` is a value of it too.
- **A message's remedy.** A finding's message carries each *Fix* case it gives
  as one segment, the case's words verbatim and its slots filled from the
  finding, and holds no remedy words outside those segments; the finding's own
  words and a release clause may stand before, between or after them. Which
  cases a message gives is said where its code is specified: several where
  several remedies fit, and none where that point says so or the cell is `none`.
- **Printing.** Backticks mark a spelling, and a printed cell is its text without
  them, a *Fix* cell's excepted.

## 6. The `grund init` transcript

`grund init` ([§FS-init.2.2](FS-init.md#22-stdout--stderr)) and `grund integrations --write` ([§FS-integrations.4.1.6](FS-integrations.md#416-outcome-verbs)) report what they wrote on **stderr**, one `<verb> <path>` line per file — `wrote AGENTS.md`, `appended CLAUDE.md`, `exists grund.toml` — and `init` follows its lines with the `next:` block. The transcript is progress, not output: the command's output is what it wrote to disk. These are the two exceptions to [§FS-errors.1](FS-errors.md#1-streams). `grund fmt --write`'s `rewrote N line(s):` report is `fmt`'s output and goes to stdout, as its `--check` report does ([§FS-fmt.3](FS-fmt.md#3-outputs)). A command with no output and no transcript stays silent and lets the exit code carry the verdict.

## 7. What this rules out

- A severity prefix (`error:`, `warning:`) ahead of a located finding's `<path>:<line>:` prefix: that leading position marks a CLI-level message ([§FS-errors.2.2](FS-errors.md#22-cli-level-message)), and the channel `check` writes follows the location ([§FS-errors.2.1](FS-errors.md#21-located-finding)).
- Multi-line messages. A finding that wants to elaborate uses `--format=json` and its `code` ([§FS-errors.5.1](FS-errors.md#51-on-stdout--the-commands-output)), not a wrapped paragraph.
- Interactive prompts, progress bars or spinners: every command is non-interactive ([§FS-non-goals.10](FS-non-goals.md#10-interactive-mode)).
- Localization. Messages are English; translation is downstream's problem.
