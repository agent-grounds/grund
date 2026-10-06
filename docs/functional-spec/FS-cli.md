# FS-cli: grund's command-line surface conventions

The behaviour that is not owned by any one subcommand — how `grund` is invoked with no subcommand, the two global flags that short-circuit before any work, and the cross-subcommand flags. Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible) (one screen of help, no surprises) and [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path) (the CLI surface — subcommands, flags, exit-code mapping — is user-visible, so it changes only backward-compatibly or through a named deprecation window).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, body, section, coordinate),
[§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, workspace), [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, severity, verdict), and
[§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations) (snapshot).

## 1. The default subcommand

- `grund` with no arguments is a CLI-level error ([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)): the fallback that ran `grund check .` was removed in grund 0.16.0, and the message names the explicit forms to write instead.
- `grund <ID>[.<section>] …` (where the first non-flag word is not a known subcommand) is the ID-read query specified by [§FS-show.1](FS-show.md#1-inputs), byte-for-byte equivalent to the explicit `show` subcommand, including show flags written before the ID: `grund --toc FS-check` reads the same body as `grund FS-check --toc`. With no path, both resolve from `.`. A first non-flag word that *is* a known subcommand, after leading flags, is not this query but the misplaced-flag error of [§FS-cli.4.1](FS-cli.md#41-a-flag-placed-before-the-subcommand).
- `grund <subcommand> …` dispatches to that subcommand: `check`, `show`, `list`, `refs`, `cover`, `fmt`, `fetch`, `id`, `init`, `config`, `agent-setup-instructions`, `completions`, `integrations`. The hidden `complete` subcommand is reserved for generated shell scripts ([§FS-completions.2](FS-completions.md#2-internal-dynamic-helper)); `fetch` is the explicit one-ID writer specified by [§FS-fetch](FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot).

### 1.1 Why these defaults

Bare `grund` kept running `check .` so that old CI scripts did not turn into a successful no-op ([§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path)), and that same reason is what the removal lands on: the top-level help page would be stdout and exit `0` ([§FS-cli.2.2](FS-cli.md#22-the-top-level-help-page)), which is the successful no-op the window was bought to prevent, so the landing is the CLI-level error of [§FS-cli.4](FS-cli.md#4-errors-with-no-source-location) and a CI step still spelled `grund` goes red naming what to write instead ([§DF-bare-grund-lands-on-an-error](../decisions/functional/DF-bare-grund-lands-on-an-error.md#df-bare-grund-lands-on-an-error-bare-grund-lands-on-a-cli-level-error-not-the-top-level-help-page)). The ID shorthand exists because resolving a cited fact is the overwhelmingly common interactive invocation; new scripts spell validation `grund check [path]`. Why the `show` subcommand is kept alongside the bare-ID default is recorded in [§DF-show-keep-explicit-form](../decisions/functional/DF-show-keep-explicit-form.md#df-show-keep-explicit-form-grund-keeps-show-as-a-subcommand-alongside-the-bare-id-default).

### 1.2 A first word that is not an ID

Because the first non-flag word is read as a subcommand *or* an ID query, a mistyped subcommand would otherwise be reported as an invalid ID. So when `grund <word>` cannot be parsed as an ID, the message names the default ID-query reading and, for a likely mistyped subcommand, the help form:

```
invalid ID `bogus`
hint: this repo's [id] format is `{kind}-{slug}` (run `grund config show`); `grund list` shows the IDs that exist
hint: run `grund --help` for the list of subcommands
```

The final `grund --help` hint is emitted only when the first word contains none of `-` / `/` / `.` — the three separators an ID, a workspace-qualified ID, or a coordinate would carry — because a token without any of them cannot match the default `{kind}-{number}-{slug}` shape and is overwhelmingly a botched subcommand. The full known-command list stays in `grund --help` rather than being repeated on every query failure.

The `grund check <word>` migration breadcrumb is limited to an existing filesystem path that the bare query would otherwise refuse as an unknown project alias ([§FS-show.3.5.2](FS-show.md#352-the-filesystem-path-migration-breadcrumb)). An invalid ID or coordinate and any other query failure do not acquire that advice merely from using the bare form: `check` takes a filesystem path ([§FS-check.1](FS-check.md#1-inputs)), not an ID or section coordinate.

Stdout is empty and the exit is `1`: the default ID lookup is a failed query, not a CLI launch failure. A known subcommand word that only reaches this query because a flag was written before it is not such a word: it is refused earlier, by [§FS-cli.4.1](FS-cli.md#41-a-flag-placed-before-the-subcommand).

### 1.3 A help request with an unknown first word

A help request with an unknown first word remains an unknown-command error, because help dispatch happens before default ID dispatch:

```
error: unknown command: bogus
known commands: check, show, list, refs, cover, fmt, fetch, id, init, config, agent-setup-instructions, completions, integrations
```

Empty stdout, exit `2` — a CLI-level error like any other unknown subcommand ([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)).

## 2. Global flags

`--version` ([§FS-cli.2.1](FS-cli.md#21---version)) and `--help` ([§FS-cli.2.2](FS-cli.md#22-the-top-level-help-page), [§FS-cli.2.3](FS-cli.md#23-a-subcommands-help-page)) are recognised regardless of subcommand and are handled *before* any tree scan or file write. When both a global flag and a subcommand are present, the global flag wins: `grund check --version` prints the version and exits `0` without scanning. `--version` outranks everything — with any subcommand present it is the version line, not that command's help page.

Help is never an error: it goes to stdout, exit `0`, so `grund --help | …` works.

### 2.1 `--version`

`grund --version` (alias `grund -V`) prints `grund <semver>` on stdout and exits `0`. Nothing else is printed; the output is one line and is deterministic for a given build. This is the affordance the [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path relies on — a warning that names "the release in which the old form stops working" is only actionable if the user can ask which release they are on.

### 2.2 The top-level help page

`grund --help` (alias `grund -h`) prints the top-level help on stdout and exits `0`. The page, top to bottom:

- **A statement.** One line saying what `grund` is.
- **A usage line.** One line naming the invocation forms `grund <ID>[.<section>]`, `grund <COMMAND> [ARGS]`, `grund <COMMAND> -h` and `grund -V`.
- **The commands, grouped by intent.** Three groups, in this order and with a blank line between them: *Query the catalog* (`show`, `list`, `refs`, `cover`), *Check and author* (`check`, `fmt`, `id`, `fetch`) and *Set up* (`init`, `config`, `integrations`, `completions`, `agent-setup-instructions`). Each row gives the command, a one-line description in the vocabulary of [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) and [§FS-terms.terms.2](FS-terms.md#terms2-citations), and a sample call.
- **A footer.** One line giving the public address of the user guides and their runnable examples, `https://github.com/agent-grounds/grund/tree/main/docs/user-facing`: an address a reader with only the installed binary can open, never a path in grund's own source tree.

There is no options block. A flag's place is on each command's own page ([§FS-cli.2.3](FS-cli.md#23-a-subcommands-help-page)), and a flag written before the subcommand is answered by the error of [§FS-cli.4.1](FS-cli.md#41-a-flag-placed-before-the-subcommand), which names where it goes.

The whole page fits one screen ([§GOAL-friendliness-first.1](../goals.md#1-hard-requirements)), which this spec sets at ≤ 24 lines of at most 100 columns, so each description is a single terse line: the `show` row still gestures at *why* the command exists ("Print a coordinate's lead for agent context (default)."), and `fetch` says that it materializes one configured external snapshot, with the full rationale on each command's own help page.

### 2.3 A subcommand's help page

`grund help <subcommand>` and `grund <subcommand> --help` (and `grund <subcommand> -h`) print *that subcommand's* page on stdout, exit `0`: its usage line, its arguments, every flag with a one-line example, the exit-code meanings for that subcommand, and a one-line recovery hint where the common failure has an obvious next step (e.g. `show`'s page says how to find an ID; `id`'s page shows the `$EDITOR` follow-up). `grund help` with no argument is the top-level page; `grund help <unknown>` is the unknown-command error ([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)).

Where a user guide or a runnable example covers the command, its page ends with that guide's and example's public address: `Guide:` lines, then `Example:` lines, each giving a `https://github.com/agent-grounds/grund/blob/main/<file>` or `https://github.com/agent-grounds/grund/tree/main/<directory>` address. The address names `main`, never a version tag, because a development build has no tag ([§REQ-shipped-surfaces.1](../requirements/REQ-shipped-surfaces.md#1-no-shipped-or-printed-byte-names-a-declaration-of-this-repository)). Every such address `grund` prints names a file (`blob`) or a directory (`tree`) that exists in the tree it was built from, so a printed link cannot go stale.

## 3. Cross-subcommand flags

- `--format text|json` — accepted by the subcommands with a machine-readable result or finding surface ([§FS-errors.5](FS-errors.md#5-json-format) lists them, [§FS-integrations.5](FS-integrations.md#5-json-format)). `text` is the default; `json` opts into the stable machine shapes, on the streams of [§FS-cli.3.1](FS-cli.md#31-the---format-json-streams). It is not a global flag: the operational commands [§FS-errors.5](FS-errors.md#5-json-format) lists, whose output is human text or generated files, reject `--format`.
- A path argument, when a subcommand takes one, defaults to `.` and is resolved the same way everywhere (config discovery walks up from it — [§FS-config.1](FS-config.md#1-file-location-and-discovery)). Every path-taking subcommand accepts at most one ([§FS-cli.3.2](FS-cli.md#32-at-most-one-path)).
- `--only <code>`, `--ignore <code>`, and the boolean `--only-rule` are `check`-only finding-query flags ([§FS-check.1](FS-check.md#1-inputs)); other subcommands reject them. The `check` help page documents them ([§FS-cli.3.3](FS-cli.md#33-the-check-selector-help)).

### 3.1 The `--format json` streams

The stream split is the same as the text form ([§FS-errors.1](FS-errors.md#1-streams), [§FS-distribution.3.0](FS-distribution.md#30-language-neutral-data-shapes)): the command's output — `grund check`'s findings as NDJSON when there are any, a query result as one object (or NDJSON for a list command) — goes to stdout, while a failed ID query's finding and any CLI-level `error:` go to stderr, except that `show --batch --format=json` keeps a failed query inside its stdout envelope ([§FS-errors.5](FS-errors.md#5-json-format)). `grund check --format=json` stays findings-only and does not emit the text-mode `success` line.

### 3.2 At most one path

A second positional — `grund check a b`, `grund <ID> a b`, `grund refs ID a b`, `grund cover a b`, `grund fmt a b`, `grund list a b`, `grund id FS "t" a b` — is a CLI-level error (`error: <subcommand> takes at most one path argument`, exit `2`, [§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)), never a silent use of one and a quiet drop of the rest. `config` and `agent-setup-instructions` already enforce this; the rule is uniform across the surface, so a typo'd path is reported, never absorbed.

### 3.3 The `check` selector help

The `check` help page documents both `--flag value` and `--flag=value`, repetition and composition, the fact that operational failures remain visible with exit `2`, selected-report exit semantics, one `--ignore agents-init` example, and the complete supported-code catalog in sorted order. The catalog is the discovery path for callers that know a message but not its code.

It documents the third selector on the same footing: that `--only-rule` narrows the report to what the `--rule` sentence authored rather than by code, that the axes intersect, that it refuses with `error: --only-rule requires --rule` when no trial sentence was given, and that a `should`-level sentence also needs `--suggestions` to be seen ([§FS-rules.8](FS-rules.md#8-command-surfaces)). A selector reachable only from the specification is not discoverable at a terminal, which is where a sentence gets tried.

The same help documents `--watch [<path>]`, an example of the immediate-check/every-save loop, recoverable config/read errors, last-completed-status interruption and pre-first-completion/fatal-watcher status `2` ([§FS-check.6.3](FS-check.md#63-lifecycle)). It links the public watch guide for native-backend and notification-silent-filesystem limits, terminal/redirection behavior and invisible clean JSON runs ([§FS-check.6.1](FS-check.md#61-change-detection), [§FS-check.6.2](FS-check.md#62-each-run-is-a-plain-grund-check)). Help and captured README examples must describe working behavior; a future transcript must be captured after implementation, not invented while specifying the feature.

## 4. Errors with no source location

An unknown subcommand in help dispatch (`grund help <unknown>`), an unknown or malformed flag, mutually-exclusive flags, or no arguments at all are CLI-level errors: `error: <message>` on stderr, empty stdout, exit `2` ([§FS-errors.2.2](FS-errors.md#22-cli-level-message), [§FS-check.2.1.1](FS-check.md#211-cli-level-messages)). A bare-word first argument that is neither a known subcommand nor a valid ID is not a CLI-level error but the failed default-`show` query of [§FS-cli.1.2](FS-cli.md#12-a-first-word-that-is-not-an-id), exit `1`. A flag written before a known subcommand is a CLI-level error of its own, [§FS-cli.4.1](FS-cli.md#41-a-flag-placed-before-the-subcommand).

`grund` with no arguments names both explicit forms, because the fallback that ran `grund check .` was removed in grund 0.16.0 and a caller who wrote the bare form meant one of them:

```
error: no command given; the bare `grund` fallback that ran `grund check .` was removed in grund 0.16.0
hint: run `grund check` to validate this repository
hint: run `grund --help` for the list of subcommands
```

`check` selector errors use the exact forms in [§FS-check.1](FS-check.md#1-inputs). Missing, empty, malformed, and unknown values are rejected before config discovery or scanning, regardless of `--format`; they therefore always leave stdout empty, remain raw text on stderr, and exit `2`.

### 4.1 A flag placed before the subcommand

A subcommand's flags follow the subcommand: no flag but `--version` and `--help` ([§FS-cli.2](FS-cli.md#2-global-flags)) is global, and `--format` in particular is per-command ([§FS-cli.3](FS-cli.md#3-cross-subcommand-flags)). So when the first argument is a flag and the first non-flag word after the leading flags is a known subcommand — one of the names [§FS-cli.1](FS-cli.md#1-the-default-subcommand) dispatches on — the invocation is a CLI-level error that says where the flag goes, never a default-`show` query that reads the subcommand as the ID and the next word as a path:

```
$ grund --format json list
error: --format follows the subcommand: grund list --format json
$ grund --format json refs FS-cli
error: --format follows the subcommand: grund refs --format json FS-cli
```

- **Which word is the subcommand.** The leading flags are skipped together with the values of the value-taking `show` flags (`--format`, `--section`, `--path`), written either as a separate word or as `--flag=value`; the first word left is the one tested. Any leading flag counts, not only `--format`: `grund --brief list` is refused the same way.
- **What the message names.** The first leading flag as written, without its value, then the corrected command: `grund`, the subcommand, every leading argument in the order and spelling it was written (values included), then the arguments that followed the subcommand, joined by single spaces. The error is about placement only: the corrected command may still be refused by that subcommand's own flag rules.
- **How it is reported.** Stderr, empty stdout, exit `2`, regardless of `--format`, before config discovery or any scan, like every other error here.
- **What stays valid.** A leading flag before a word that is not a known subcommand is the default `show` query of [§FS-cli.1](FS-cli.md#1-the-default-subcommand), unchanged: `grund --format json FS-cli` reads `FS-cli` as JSON and exits `0`, and a word that is no ID stays the exit-`1` query failure of [§FS-cli.1.2](FS-cli.md#12-a-first-word-that-is-not-an-id). No name that collides with a subcommand could ever have resolved as an ID there, so nothing that worked before this rule stops working.

## 5. Exit-code mapping is fixed

`0` clean / printed, `1` findings or a failed query, `2` scan or CLI-level failure — the precise meaning per subcommand is in that subcommand's spec, but the *mapping* is frozen per [§GOAL-friendliness-first.2](../goals.md#2-what-this-rules-out) and [§FS-non-goals.9](FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization): it is not configurable, and a change to it goes through the [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path) deprecation path. The mapping is the machine-readable half of the verdict, so freezing it is what [§REQ-backwards-compatibility](../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly) rests on.

For `show --batch`, `1` is the aggregate failed-query verdict: all valid query
records are emitted before the process returns it. Exit `2` is reserved for a
malformed invocation or input stream, configuration failure, or scan failure that
prevents the batch from running ([§FS-show.2.6](FS-show.md#26-batch-resolution)).

## 6. What is deliberately absent

- No generic `--quiet` / `--verbose` knobs — severity is fixed ([§FS-non-goals.9](FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization)), and a clean text `grund check` already has a single fixed `success` line ([§GOAL-friendliness-first.1](../goals.md#1-hard-requirements)). The explicit `check --only <code>` / `--ignore <code>` surface is a scoped query over stable finding identities, not a presentation mode or a project policy knob ([§FS-check.1](FS-check.md#1-inputs)).
- No `--config <file>` override — config is discovered by walking up from the command path ([§FS-config.1](FS-config.md#1-file-location-and-discovery)), not pointed at directly, to keep two installs on the same tree in agreement ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)). `grund config show [path]` reports what was discovered from that starting path.
- No interactive flags, no TUI, no prompts ([§FS-non-goals.10](FS-non-goals.md#10-interactive-mode)).
- No `grund graph`, no `grund new` — graph visualisation is a non-goal ([§FS-non-goals.6](FS-non-goals.md#6-decision-database-audit-log-history-tracking)), and file creation for a new declaration is the caller's job after `grund id` ([§FS-id.7](FS-id.md#7-what-id-does-not-do)).
