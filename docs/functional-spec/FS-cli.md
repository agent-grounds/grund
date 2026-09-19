# FS-cli: grund's command-line surface conventions

The behaviour that is not owned by any one subcommand — how `grund` is invoked with no subcommand, the two global flags that short-circuit before any work, and the cross-subcommand flags. Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible) (one screen of help, no surprises) and [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path) (the CLI surface — subcommands, flags, exit-code mapping — is user-visible, so it changes only backward-compatibly or through a named deprecation window).

## 1. The default subcommand

- `grund` with no arguments keeps the historical `check .` behavior for the current deprecation window: it prints `warning: bare \`grund\` still runs \`grund check .\`; use \`grund check\` explicitly.` on stderr, then runs the same validation as `grund check .` with the same stdout and exit code.
- `grund <ID>[.<section>] …` (where the first non-flag word is not a known subcommand) is the ID-read query specified by [§FS-show.1](FS-show.md#1-inputs), byte-for-byte equivalent to the explicit `show` subcommand, including show flags written before the ID: `grund --toc FS-check` reads the same body as `grund FS-check --toc`. With no path, both resolve from `.`.
- `grund <subcommand> …` dispatches to that subcommand: `check`, `show`, `list`, `refs`, `cover`, `fmt`, `fetch`, `id`, `init`, `config`, `agent-setup-instructions`, `completions`, `integrations`. The hidden `complete` subcommand is reserved for generated shell scripts ([§FS-completions.2](FS-completions.md#2-internal-dynamic-helper)); `fetch` is the explicit one-ID writer specified by [§FS-fetch](FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot).

### 1.1 Why these defaults

Bare `grund` keeps running `check .` so that old CI scripts do not turn into a successful no-op while the default interactive form moves to `grund <ID>` ([§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path)). The ID shorthand exists because resolving a cited fact is the overwhelmingly common interactive invocation; new scripts spell validation `grund check [path]`. Why the `show` subcommand is kept alongside the bare-ID default is recorded in [§DF-show-keep-explicit-form](../decisions/functional/DF-show-keep-explicit-form.md#df-show-keep-explicit-form-grund-keeps-show-as-a-subcommand-alongside-the-bare-id-default).

### 1.2 A first word that is not an ID

Because the first non-flag word is read as a subcommand *or* an ID query, a mistyped subcommand would otherwise be reported as an invalid ID. So when `grund <word>` cannot be parsed as an ID, the message names the default ID-query reading and the explicit check form:

```
invalid ID `bogus`
hint: this repo's [id] format is `{kind}-{slug}` (run `grund config show`); `grund list` shows the IDs that exist
hint: run `grund check bogus` to validate a path
hint: run `grund --help` for the list of subcommands
```

The final `grund --help` hint is emitted only when the first word contains none of `-` / `/` / `.` — the three separators an ID, a workspace-qualified ID, or a section reference would carry — because a token without any of them cannot match the default `{kind}-{number}-{slug}` shape and is overwhelmingly a botched subcommand. The full known-command list stays in `grund --help` rather than being repeated on every query failure.

Stdout is empty and the exit is `1`: the default ID lookup is a failed query, not a CLI launch failure.

### 1.3 A help request with an unknown first word

A help request with an unknown first word remains an unknown-command error, because help dispatch happens before default ID dispatch:

```
error: unknown command: bogus
known commands: check, show, list, refs, cover, fmt, fetch, id, init, config, agent-setup-instructions, completions, integrations
```

Empty stdout, exit `2` — a CLI-level error like any other unknown subcommand (§4).

## 2. Global flags

`--version` (§2.1) and `--help` (§2.2, §2.3) are recognised regardless of subcommand and are handled *before* any tree scan or file write. When both a global flag and a subcommand are present, the global flag wins: `grund check --version` prints the version and exits `0` without scanning. `--version` outranks everything — with any subcommand present it is the version line, not that command's help page.

Help is never an error: it goes to stdout, exit `0`, so `grund --help | …` works.

### 2.1 `--version`

`grund --version` (alias `grund -V`) prints `grund <semver>` on stdout and exits `0`. Nothing else is printed; the output is one line and is deterministic for a given build. This is the affordance the [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path relies on — a warning that names "the release in which the old form stops working" is only actionable if the user can ask which release they are on.

### 2.2 The top-level help page

`grund --help` (alias `grund -h`) prints the top-level help on stdout and exits `0`. The page opens with a one-line statement of what `grund` is, then the three invocation forms (`grund <ID>`, `grund check <path>`, and `grund <command> …`), then a `Commands:` block — every subcommand on its own line with a one-line description and a sample invocation — then the cross-subcommand options. Every flag carries a one-line example.

The whole page fits one screen ([§GOAL-friendliness-first.1](../goals.md#1-hard-requirements)), which this spec sets at ≤ 24 lines, so each description is a single terse line: the `show` line still gestures at *why* the command exists ("Print one declaration body for agent context."), and `fetch` says that it materializes one configured external snapshot, with the full rationale on each command's own help page.

### 2.3 A subcommand's help page

`grund help <subcommand>` and `grund <subcommand> --help` (and `grund <subcommand> -h`) print *that subcommand's* page on stdout, exit `0`: its usage line, its arguments, every flag with a one-line example, the exit-code meanings for that subcommand, and a one-line recovery hint where the common failure has an obvious next step (e.g. `show`'s page says how to find an ID; `id`'s page shows the `$EDITOR` follow-up). `grund help` with no argument is the top-level page; `grund help <unknown>` is the unknown-command error (§4).

## 3. Cross-subcommand flags

- `--format text|json` — accepted by the subcommands with a machine-readable result or finding surface ([§FS-errors.5](FS-errors.md#5-json-format) lists them, [§FS-integrations.5](FS-integrations.md#5-json-format)). `text` is the default; `json` opts into the stable machine shapes, on the streams of §3.1. It is not a global flag: the operational commands [§FS-errors.5](FS-errors.md#5-json-format) lists, whose output is human text or generated files, reject `--format`.
- A path argument, when a subcommand takes one, defaults to `.` and is resolved the same way everywhere (config discovery walks up from it — [§FS-config.1](FS-config.md#1-file-location-and-discovery)). Every path-taking subcommand accepts at most one (§3.2).
- `--only <code>` and `--ignore <code>` are `check`-only diagnostic-query flags ([§FS-check.1](FS-check.md#1-inputs)); other subcommands reject them. The `check` help page documents them (§3.3).

### 3.1 The `--format json` streams

The stream split is the same as the text form ([§FS-errors.1](FS-errors.md#1-streams), [§FS-distribution.3.0](FS-distribution.md#30-language-neutral-data-shapes)): the command's output — `grund check`'s findings as NDJSON when diagnostics exist, a query result as one object (or NDJSON for a list command) — goes to stdout, while a failed ID query's diagnostic and any CLI-level `error:` go to stderr, except that `show --batch --format=json` keeps a failed query inside its stdout envelope ([§FS-errors.5](FS-errors.md#5-json-format)). `grund check --format=json` stays diagnostics-only and does not emit the text-mode `success` marker.

### 3.2 At most one path

A second positional — `grund check a b`, `grund <ID> a b`, `grund refs ID a b`, `grund cover a b`, `grund fmt a b`, `grund list a b`, `grund id FS "t" a b` — is a CLI-level error (`error: <subcommand> takes at most one path argument`, exit `2`, §4), never a silent use of one and a quiet drop of the rest. `config` and `agent-setup-instructions` already enforce this; the rule is uniform across the surface, so a typo'd path is reported, never absorbed.

### 3.3 The `check` selector help

The `check` help page documents both `--flag value` and `--flag=value`, repetition and composition, the fact that operational failures remain visible with exit `2`, selected-report exit semantics, one `--ignore agents-init` example, and the complete supported-code catalog in sorted order. The catalog is the discovery path for callers that know a message but not its code.

## 4. Errors with no source location

An unknown subcommand in help dispatch (`grund help <unknown>`), an unknown or malformed flag, or mutually-exclusive flags are CLI-level errors: `error: <message>` on stderr, empty stdout, exit `2` ([§FS-errors.2.2](FS-errors.md#22-cli-level-message), [§FS-check.2.1.1](FS-check.md#211-cli-level-messages)). A bare-word first argument that is neither a known subcommand nor a valid ID is not a CLI-level error but the failed default-`show` query of §1.2, exit `1`.

`check` selector errors use the exact forms in [§FS-check.1](FS-check.md#1-inputs). Missing, empty, malformed, and unknown values are rejected before config discovery or scanning, regardless of `--format`; they therefore always leave stdout empty, remain raw text on stderr, and exit `2`.

## 5. Exit-code mapping is fixed

`0` clean / printed, `1` findings or a failed query, `2` scan or CLI-level failure — the precise meaning per subcommand is in that subcommand's spec, but the *mapping* is frozen per [§GOAL-friendliness-first.2](../goals.md#2-what-this-rules-out) and [§FS-non-goals.9](FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization): it is not configurable, and a change to it goes through the [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path) deprecation path. The mapping is the machine-readable half of the verdict, so freezing it is what [§REQ-backwards-compatibility](../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly) rests on.

For `show --batch`, `1` is the aggregate failed-query verdict: all valid query
records are emitted before the process returns it. Exit `2` is reserved for a
malformed invocation or input stream, configuration failure, or scan failure that
prevents the batch from running ([§FS-show.2.6](FS-show.md#26-batch-resolution)).

## 6. What is deliberately absent

- No generic `--quiet` / `--verbose` knobs — severity is fixed ([§FS-non-goals.9](FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization)), and a clean text `grund check` already has a single fixed `success` line ([§GOAL-friendliness-first.1](../goals.md#1-hard-requirements)). The explicit `check --only <code>` / `--ignore <code>` surface is a scoped query over stable diagnostic identities, not a presentation mode or a project policy knob ([§FS-check.1](FS-check.md#1-inputs)).
- No `--config <file>` override — config is discovered by walking up from the command path ([§FS-config.1](FS-config.md#1-file-location-and-discovery)), not pointed at directly, to keep two installs on the same tree in agreement ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)). `grund config show [path]` reports what was discovered from that starting path.
- No interactive flags, no TUI, no prompts ([§FS-non-goals.10](FS-non-goals.md#10-interactive-mode)).
- No `grund graph`, no `grund new` — graph visualisation is a non-goal ([§FS-non-goals.6](FS-non-goals.md#6-decision-database-audit-log-history-tracking)), and file creation for a new declaration is the caller's job after `grund id` ([§FS-id.7](FS-id.md#7-what-id-does-not-do)).
