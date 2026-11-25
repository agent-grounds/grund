# FS-cli: grund's command-line surface conventions

What no one subcommand owns: how the first word picks what `grund` runs, the two global flags
that answer before any work, the run flags several commands share, the errors that carry no
source location, and the fixed exit codes. Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible) and
[§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, body, section, coordinate),
[§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, workspace), [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, severity, verdict), and
[§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations) (snapshot).

## requirements: What every invocation holds to

Three more properties hold of every invocation and are stated where they are shared: its
exit-code mapping is fixed ([§GOAL-friendliness-first.2](../goals.md#2-what-this-rules-out), [§FS-cli.5](FS-cli.md#5-exit-code-mapping-is-fixed)), it changes only through a named deprecation
path ([§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path), [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)), and every address a help page
prints resolves where it lands ([§REQ-shipped-surfaces.1](../requirements/REQ-shipped-surfaces.md#1-no-shipped-or-printed-byte-names-a-declaration-of-this-repository)).

### requirements.before-load: A usage mistake is answered before the tree is read

What the arguments alone can refuse is refused before config discovery, before any
`grund.toml` is read and before any scan: an unsupported run-flag value ([§FS-cli.3.5](FS-cli.md#35-a-bad-run-flag-is-answered-before-the-load)), a
flag placed before its subcommand ([§FS-cli.4.1](FS-cli.md#41-a-flag-placed-before-the-subcommand)), an unknown or malformed flag, a missing
value, a conflicting pair ([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)), a second path ([§FS-cli.3.2](FS-cli.md#32-at-most-one-path)), and a bad `check`
selector ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)); `--version` and `--help` are answered before it as well ([§FS-cli.2](FS-cli.md#2-global-flags)).
So a mistake on the command line is reported as itself whatever tree it points at, and a
broken `grund.toml` never hides it. A flag added from here on is validated,
as far as its own spelling decides, before the load. Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

### requirements.as-written: An invocation runs as written or is refused

No invocation is quietly narrowed or read as another. A second path is refused, never
dropped ([§FS-cli.3.2](FS-cli.md#32-at-most-one-path)); a flag before a known subcommand is refused, never read as a query for
an ID spelled like the subcommand ([§FS-cli.4.1](FS-cli.md#41-a-flag-placed-before-the-subcommand)); a flag the command does not take is an
error ([§FS-cli.3](FS-cli.md#3-cross-subcommand-flags)); bare `grund` is an error, never a help page at exit `0` ([§FS-cli.1](FS-cli.md#1-the-default-subcommand));
and a first word that is neither a subcommand, `help` nor an ID fails as the query it was read as,
with a hint toward the subcommand it may have meant ([§FS-cli.1.2](FS-cli.md#12-a-first-word-that-is-not-an-id)). Where one word has two
readings, the order that picks is written down: help dispatch before the ID query
([§FS-cli.1.3](FS-cli.md#13-a-help-request-with-an-unknown-first-word)), and `--version` above everything ([§FS-cli.2.1](FS-cli.md#21---version)). A subcommand or flag added from
here on keeps this: a word it makes ambiguous is refused or given a written precedence,
never resolved by a guess. Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

## 1. The default subcommand

The first word that is not a flag decides what `grund` runs:

- A subcommand written first dispatches to it: `check`, `show`, `list`, `refs`, `cover`,
  `fmt`, `fetch`, `id`, `init`, `config`, `schema`, `guide`, `completions`, `integrations`.
  The one hidden subcommand, `complete`, is reserved for generated shell scripts
  ([§FS-completions.2](FS-completions.md#2-internal-dynamic-helper)). Written after leading flags, a subcommand is the error of [§FS-cli.4.1](FS-cli.md#41-a-flag-placed-before-the-subcommand).
- `help` written first is the help request of [§FS-cli.2.3](FS-cli.md#23-a-subcommands-help-page).
- Any other word is the ID query of [§FS-show.1](FS-show.md#1-inputs), byte for byte what the explicit `show`
  subcommand prints, show flags written before the ID included: `grund --toc FS-check` reads
  what `grund FS-check --toc` reads ([§DF-show-keep-explicit-form](../decisions/functional/DF-show-keep-explicit-form.md#df-show-keep-explicit-form-grund-keeps-show-as-a-subcommand-alongside-the-bare-id-default)).
- `grund` with no arguments is the CLI-level error of [§FS-cli.4](FS-cli.md#4-errors-with-no-source-location), not the help page
  ([§DF-bare-grund-lands-on-an-error](../decisions/functional/DF-bare-grund-lands-on-an-error.md#df-bare-grund-lands-on-an-error-bare-grund-lands-on-a-cli-level-error-not-the-top-level-help-page)).

### 1.2 A first word that is not an ID

A first word that is neither a subcommand nor `help` is read as an ID, so a mistyped
subcommand fails as that query: empty stdout, exit `1`, and on stderr the invalid-ID message
and the hints of [§FS-show.3.5](FS-show.md#35-the-hint-for-each-failure), the `grund check <word>` hint for an existing path among them,
then, when the word contains none of `-`, `/` and `.`, one more line pointing at the list of
subcommands. Those are the separators an ID, a workspace-qualified ID and a coordinate carry,
so a word with none of them cannot be an ID of the default shape and is most likely a botched
subcommand. So in a version-2 project whose `[schema] id_format` is `{kind}-{slug}`,
`grund bogus` prints:

```text
invalid ID `bogus`
hint: this repo's [schema] id_format is `{kind}-{slug}` (run `grund config show`); `grund list` shows the IDs that exist
hint: run `grund --help` for the list of subcommands
```

### 1.3 A help request with an unknown first word

Help dispatch comes before the ID query, so a help request that names an unknown command,
`grund help bogus` or `grund bogus --help`, is a CLI-level error ([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)), empty stdout and
exit `2`, rather than a query:

```text
error: unknown command: bogus
known commands: check, show, list, refs, cover, fmt, fetch, id, init, config, schema, guide, completions, integrations
```

## 2. Global flags

`--version` and `--help` are the two global flags: recognised whatever the subcommand, and
answered before any tree is read or any file written.

### 2.1 `--version`

`grund --version` (alias `grund -V`) prints exactly `grund <semver>` on stdout and exits `0`.
The `<semver>` is this build's version as [§FS-distribution.4.1.1](FS-distribution.md#411-a-development-build-names-its-commit) gives it, fixed when the
binary was compiled, so answering reads no git, environment or file. It wins over everything
else on the command line: with any subcommand it prints the version line, never that
command's run or help page, so `grund check --version` scans nothing, and an argument that is
not UTF-8 does not refuse it ([§FS-cli.5.1](FS-cli.md#51-non-utf-8-process-arguments-are-refused-before-dispatch)). It is how a user learns which release a
deprecation warning measures from ([§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)).

### 2.2 The top-level help page

`grund --help` (alias `grund -h`) prints the top-level help on stdout and exits `0`. The page,
top to bottom:

- **A statement.** One line saying what `grund` is.
- **A usage line.** Directly under the statement, with no blank line between them, one line
  naming the invocation forms `grund <ID>[.<section>]`, `grund <COMMAND> [ARGS]`,
  `grund <COMMAND> -h` and `grund -V`.
- **The commands, grouped by intent.** Three groups, in this order and with a blank line
  between them: *Query the catalog* (`show`, `list`, `refs`, `cover`), *Check and author*
  (`check`, `fmt`, `id`, `fetch`) and *Set up* (`guide`, `init`, `config`, `schema`,
  `integrations`, `completions`). Each row gives the command, a one-line description in the
  vocabulary of [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) and [§FS-terms.terms.2](FS-terms.md#terms2-citations), and a sample call.
- **A footer.** One line giving the public address of the user guides and their runnable
  examples, `https://github.com/agent-grounds/grund/tree/main/docs/user-facing`: an address a
  reader with only the installed binary can open, never a path in grund's own source tree.

There is no options block: a flag's place is its command's own page ([§FS-cli.2.3](FS-cli.md#23-a-subcommands-help-page)), and a flag
written before the subcommand gets the error of [§FS-cli.4.1](FS-cli.md#41-a-flag-placed-before-the-subcommand), which names where it goes.

The whole page fits one screen ([§GOAL-friendliness-first.1](../goals.md#1-hard-requirements)), which this spec sets at ≤ 24
lines of at most 100 columns, so each description is a single terse line. Some rows say more
than what the command is: `show`'s marks it as the default command, `fmt`'s names link and
section-lock maintenance ([§DF-fmt-maintenance-guidance](../decisions/functional/DF-fmt-maintenance-guidance.md#df-fmt-maintenance-guidance-teach-the-existing-post-rename-formatter-maintenance)), `integrations`' names `grund-lsp`
([§FS-integrations.2](FS-integrations.md#2-detection)), `guide`'s says it teaches setting up and using grund and lists its
sections, with the bare command as its sample call ([§DF-guide-command](../decisions/functional/DF-guide-command.md#df-guide-command-grund-agent-setup-instructions-becomes-grund-guide-whose-bare-run-prints-a-front-page-and-whose-sections-are-named-positionally), [§FS-init.5.6](FS-init.md#56-the-front-page)), and
`list`'s names the declarations nothing cites, with `--unused` as its sample call
([§FS-list.1.3](FS-list.md#13---unused)):

```text
  list          Every declared ID, or the ones nothing cites.           grund list --unused
```

### 2.3 A subcommand's help page

`grund help <subcommand>` and `grund <subcommand> --help` (and `grund <subcommand> -h`) print
*that subcommand's* page on stdout, exit `0`: its usage line, its arguments, every flag with a
one-line example, the exit-code meanings for that subcommand, and a one-line recovery hint
where the common failure has an obvious next step. `grund help` with no argument is the
top-level page, and a help request for an unknown command is the error of [§FS-cli.1.3](FS-cli.md#13-a-help-request-with-an-unknown-first-word).

A page is static: it discovers no configuration to render, so it describes scope truthfully
for every supported config version, in version-neutral terms or by labelling v1's
`[scan] include` and v2's places ([§FS-config-v2.schema.places](FS-config-v2.md#schemaplaces-rows-and-their-places)). `check`'s page keeps the
no-config default directories and describes `--full` as
[§FS-config-v2.schema.places.boundaries](FS-config-v2.md#schemaplacesboundaries-what-a-place-may-name) does, never as a way past source exclusions
([§FS-config-v2.schema.sources.exclude](FS-config-v2.md#schemasourcesexclude-exclude-is-a-gitignore-style-glob-list)).

Every flag the page names, a run flag of [§FS-cli.3](FS-cli.md#3-cross-subcommand-flags) included, has an example: a command line
that runs the page's subcommand (for `show`, the bare `grund …` too) and passes the flag,
spelled `--flag`, `--flag VALUE` or `--flag=VALUE`, written after `e.g.` on the flag's option
entry or as a line of a block whose heading ends in `examples:`. A bare value such as
`e.g. --section 3.1` says what the flag takes, not how the command runs, so it is no example.

The page of each command whose `<path>` narrows what it scans or reports, `check`, `fmt`,
`cover`, `list` and `refs`, says what a run given no path covers ([§FS-scan.path.none](FS-scan.md#pathnone-a-run-with-no-path-covers-the-project-discovery-finds)) in one
line below its usage, and none says PATH defaults to `.`:

```text
With no PATH, the run covers the whole project or workspace discovered from the current directory; a PATH narrows it.
```

A page whose path only chooses the project that answers, as `config`'s and `schema`'s do,
keeps the default it states. `show`'s path does both ([§FS-scan.path.show](FS-scan.md#pathshow-show-resolves-the-id-against-the-project-the-path-discovers)), so its `--path`
entry gives the one-coordinate read and the generated batch (`--batch --all`, `--glob`) each
its own default and names no `.` default.

`check`'s page documents every selector, `--only-rule` among them, and how they compose
([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore), [§FS-rules.8](FS-rules.md#8-command-surfaces)); that selection filters a complete report, so an operational
failure stays visible at exit `2` and exit `0` or `success` speaks only for the selected
report ([§FS-check.2.1.2](FS-check.md#212-selection-filters-the-complete-report), [§FS-check.2.1.3](FS-check.md#213-the-success-line)); and the complete supported-code catalog in sorted
order, so a caller who knows a message finds its code, with a line pointing at
`grund config explain CODE` ([§FS-config-explain.4](FS-config-explain.md#4-a-codes-answer)). Beside PATH it names what a PATH run still reports from
outside it, the root agent-entrypoint finding `agents-init` and a multi-site finding with a
site in PATH, such as a duplicate anchored at its twin ([§FS-check.1.3.6](FS-check.md#136-an-explicit-path-still-narrows)), and teaches
`grund check <PATH> --ignore agents-init`, there and in a PATH-bearing example, as the way to
leave `agents-init` out; it never promises that every reported location then lies in PATH. It documents `--watch [<path>]` with an example of its loop and the
statuses it exits with ([§FS-check.6.3](FS-check.md#63-lifecycle)), and links the public watch guide for that mode's limits
([§FS-check.6.1](FS-check.md#61-change-detection), [§FS-check.6.2](FS-check.md#62-each-run-is-a-plain-grund-check)).

`fmt`'s page teaches what to run after renaming a declaration or moving its file
([§DF-fmt-maintenance-guidance](../decisions/functional/DF-fmt-maintenance-guidance.md#df-fmt-maintenance-guidance-teach-the-existing-post-rename-formatter-maintenance)): an example `grund fmt --write` from the config or workspace root, which
re-derives generated Markdown link destinations and anchors where the link pass applies ([§FS-fmt.6.3](FS-fmt.md#63-idempotency-and-re-derive))
and makes the eligible section-lock updates ([§FS-section-lock.write.rules](FS-section-lock.md#writerules-add-refresh-drop)); how its report reads
([§FS-fmt.3.5](FS-fmt.md#35-the-dry-run-report), [§FS-fmt.3.7](FS-fmt.md#37-the---write-report), [§FS-section-lock.write.report](FS-section-lock.md#writereport-what-fmt-reports)); and what `fmt` does not do: rename an
authored citation ID, repair an arbitrary URL, rename a lock for a file move alone
([§FS-section-lock.file.shard](FS-section-lock.md#fileshard-one-file-per-cited-declaration)) or silently approve a protected section move
([§FS-section-lock.write.never-a-move](FS-section-lock.md#writenever-a-move-fmt-never-approves-a-move)).

`show`'s `--format` row says what each format prints for a section read as well as for a whole
declaration ([§FS-show.2.2](FS-show.md#22-section), [§FS-show.3.1](FS-show.md#31-format-variants), [§DF-block-names-section-heading](../decisions/functional/DF-block-names-section-heading.md#df-block-names-section-heading-the-blocks-first-rung-says-a-section-read-opens-with-its-heading-line)):

```text
  --format text|md|json  text (default) is the body, opening with a selected section's
                         own heading line; md adds the declaration heading above it;
                         json wraps the text body
```

Where a user guide or a runnable example covers the command, the page ends with `Guide:` and
then `Example:` lines giving their public addresses,
`https://github.com/agent-grounds/grund/blob/main/<file>` or
`https://github.com/agent-grounds/grund/tree/main/<directory>`. They name `main`, never a
version tag, which a development build lacks ([§REQ-shipped-surfaces.1](../requirements/REQ-shipped-surfaces.md#1-no-shipped-or-printed-byte-names-a-declaration-of-this-repository)), and every address
`grund` prints names a file (`blob`) or a directory (`tree`) that exists in the tree it was
built from, so a printed link cannot go stale.

## 3. Cross-subcommand flags

Two **run flags** are shared across commands, each a caller's choice for this one run, written
after the subcommand as `--flag value` or `--flag=value`; neither is global ([§FS-cli.4.1](FS-cli.md#41-a-flag-placed-before-the-subcommand)):

- `--format text|json` picks the output, `text` by default; `json` opts into the stable machine
  shapes of [§FS-errors.5](FS-errors.md#5-json-format), on the streams of [§FS-errors.1](FS-errors.md#1-streams). Passed, it outranks the v1
  `[output] format` key of a command that honours it ([§FS-config.3.6](FS-config.md#36-output--report-format)): flag, then key, then default.
- `--path-base project|invocation` picks the report base every rendered path is spelled from,
  `project` by default, and outranks the v1 `relative_paths` key as [§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from) says.

A bad value of either is answered before anything is loaded ([§FS-cli.3.5](FS-cli.md#35-a-bad-run-flag-is-answered-before-the-load)). Which command
takes which:

| command | `--format` | honours v1 `[output] format` | `--path-base` |
|---|---|---|---|
| `check` (incl. `--watch`), `list`, `refs`, `cover` | `text\|json` | yes | yes |
| `show` (incl. `--batch` and the bare-ID form), `id`, `integrations` | `text\|json` | no, so a tree that commits `format = "json"` never changes their bytes ([§REQ-backwards-compatibility.1](../requirements/REQ-backwards-compatibility.md#1-what-is-covered)) | yes |
| `fmt`, `config validate`, `config show`, `config migrate` | rejected | — | yes: they render paths and config-load errors |
| `schema` | its own `table\|mermaid\|dot` ([§FS-schema.1](FS-schema.md#1-the-command)); `text` and `json` are refused | no | yes: it renders config-load errors |
| `init`, `fetch`, `completions`, `guide` | rejected | — | rejected |

A rejected flag is the command's own `unknown flag` error ([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)).

A path argument means what [§FS-scan.path](FS-scan.md#path-what-a-path-argument-means) says, in every command that takes one, and each takes
at most one ([§FS-cli.3.2](FS-cli.md#32-at-most-one-path)). `--only <code>`, `--ignore <code>` and the boolean `--only-rule` are
`check`'s finding-query flags ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)), and every other command rejects them.

### 3.2 At most one path

A second path to any command that takes one, as in `grund check a b` or `grund id FS "t" a b`,
is a CLI-level error, exit `2` ([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)), never a silent use of one and a quiet drop of the
rest, so a typo'd path is reported, never absorbed. The line is
`error: <command> takes at most one path argument`, where `<command>` keeps an action or flag
that selects the form (`config migrate`, `show --batch`). A command that reads operands before
the path names them first: `error: show takes an ID and at most one path argument`, also for a
bare `grund <ID>` and with `refs` for `show` in ID mode ([§FS-refs.1.2](FS-refs.md#12-refusals)), and
`error: id takes <KIND>, <title>, and at most one path argument`. `guide` refuses a second
section the same way, as `error: guide takes at most one section` ([§FS-init.5](FS-init.md#5-the-guide)).

### 3.4 `--path-base` — where report paths are spelled from

`--path-base=project|invocation` selects the report base ([§FS-errors.terms](FS-errors.md#terms-terms)) for the whole run:

- **`project`** (the default) — the config root the run resolved against, the workspace root
  when a workspace is loaded: what v1 `[output] relative_paths = true` gives.
- **`invocation`** — the path you passed, or the current directory if you passed none: the
  path argument itself when it is a directory, its parent directory when it is a file, else the
  working directory. This is the CLI base of [§FS-config.3.6.1](FS-config.md#361-relative_paths--false--the-cli-base), so
  it prints byte for byte what `[output] relative_paths = false` prints, its bounded `..`
  components included ([§DF-cli-base-parent-paths](../decisions/functional/DF-cli-base-parent-paths.md#df-cli-base-parent-paths-relative_paths--false-keeps-one-cli-base-and-may-climb-within-the-loaded-root)). It is not the literal working directory:
  `cd / && grund check /abs/ws/src/lib.rs` reports `lib.rs`, never a path that climbs out of
  the loaded root ([§FS-errors.4](FS-errors.md#4-determinism)).

With no flag the v1 `[output] relative_paths` key decides; a passed flag outranks it, so
`--path-base=project` overrides a committed `relative_paths = false` ([§FS-config.3.6](FS-config.md#36-output--report-format)).

The flag sets one base for the whole run, every project it loads and every rerun of
`check --watch`, so a workspace whose root and member disagree on `relative_paths` reports
from the one base the flag names. It spells
every path a report renders: a finding's or a scan failure's location, every path inside its
message or reason, and JSON `path` and `sites[].path` ([§FS-workspace.8.7.3](FS-workspace.md#873-paths), [§FS-errors.2.2.5](FS-errors.md#225-one-scan-failure-one-rendering)); and
the config file a config-load error names ([§FS-errors.2](FS-errors.md#2-the-fixed-shapes)), the one path a config key can never
move, because the key is read only after that config loaded. The report base itself is
spelled `.`, never the empty string, wherever a report names it, a declaration record's
location and a message's text included, so no reported path reads empty ([§FS-workspace.5.3](FS-workspace.md#53-the-empty-scan-caution-names-each-project-from-the-report-base)).

Every command that takes the flag defines both values on its help page and names
`--path-base=invocation` as the replacement for `[output] relative_paths = false`, and a
command that honours `[output] format` names `--format json` as its replacement the same way.

### 3.5 A bad run flag is answered before the load

An unsupported `--format` or `--path-base` value is the caller's usage error, so every
command answers it **before anything is loaded** — before
config discovery, before a member's `grund.toml` is read, before any scan — whatever the other
flag's value: on stderr, with empty stdout and exit `2`. So `grund list --format bogus` prints
the first line below even in a tree whose `grund.toml` is broken, and
`grund check --path-base=x` the second:

```text
error: unsupported list format `bogus`
error: unsupported path base `x` (expected project or invocation)
```

An `[output] format` or `relative_paths` key carrying an unsupported value is a property of the
tree, so it is reported after the load, like any other config fault.

## 4. Errors with no source location

An unknown or malformed flag, a missing value, mutually exclusive flags, an unknown command in
a help request ([§FS-cli.1.3](FS-cli.md#13-a-help-request-with-an-unknown-first-word)), and no arguments at all are CLI-level errors ([§FS-errors.2.2](FS-errors.md#22-cli-level-message)):
`error: <message>` on stderr, which stays text under `--format json` ([§FS-errors.5.2.2](FS-errors.md#522-launch-time-messages-stay-text)), empty
stdout, exit `2`. `check`'s selector errors take the forms of [§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore). A first word that
is neither a subcommand, `help` nor an ID is not one of these but the failed query of [§FS-cli.1.2](FS-cli.md#12-a-first-word-that-is-not-an-id),
exit `1`.

`grund` with no arguments names both explicit forms
([§DF-bare-grund-lands-on-an-error](../decisions/functional/DF-bare-grund-lands-on-an-error.md#df-bare-grund-lands-on-an-error-bare-grund-lands-on-a-cli-level-error-not-the-top-level-help-page)):

```text
error: no command given; the bare `grund` fallback that ran `grund check .` was removed in grund 0.16.0
hint: run `grund check` to validate this repository
hint: run `grund --help` for the list of subcommands
```

### 4.1 A flag placed before the subcommand

A subcommand's flags follow the subcommand, since only `--version` and `--help` are global
([§FS-cli.2](FS-cli.md#2-global-flags)). So when the first argument is a flag and the first non-flag word after the leading
flags is a subcommand [§FS-cli.1](FS-cli.md#1-the-default-subcommand) dispatches on, the invocation is a CLI-level error that says
where the flag goes, never a `show` query that reads the subcommand as the ID and the next word
as a path. So `grund --format json refs FS-widget` prints:

```text
error: --format follows the subcommand: grund refs --format json FS-widget
```

The leading flags are skipped together with the values of the value-taking flags (`--format`,
`--path-base`, `--section`, `--path`), written as a separate word or as `--flag=value`, and the
first word left is the one tested; any leading flag counts, so `grund --brief list` and
`grund --path-base invocation list` are refused the same way. The message names the first
leading flag as written, without its value, then the corrected command: `grund`, the
subcommand, every leading argument in the order and spelling it was written, values included,
then the arguments that followed the subcommand, joined by single spaces. The error is about
placement only: the corrected command may still be refused by that subcommand's own flag
rules. A leading flag before a word that is not a subcommand is the `show` query of [§FS-cli.1](FS-cli.md#1-the-default-subcommand).

## 5. Exit-code mapping is fixed

Every command exits with one of three codes, and the precise meaning per command is in its own
spec:

- `0` — clean, or printed;
- `1` — findings, or a failed query;
- `2` — a scan or CLI-level failure.

The *mapping* is frozen ([§GOAL-friendliness-first.2](../goals.md#2-what-this-rules-out), [§FS-non-goals.9](FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization)): it is not
configurable, and a change to it goes through the [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path) deprecation path.
A `show --batch` exits `1` when any of its queries fails, after printing every record
([§FS-show.3.3](FS-show.md#33-batch-query)).

### 5.1 Non-UTF-8 process arguments are refused before dispatch

A literal `--version` or `-V` anywhere among the process arguments wins even when another argument is not UTF-8: the
normal compiled version line, empty stderr, exit `0` ([§FS-cli.2.1](FS-cli.md#21---version)). Otherwise an argument that
is not valid UTF-8 refuses the invocation before command dispatch, config discovery, scanning
or writing, help included: empty stdout, exit `2`, and one stderr line beginning `error: ` that
identifies the argument as invalid UTF-8 and shows its value with invalid units and control
characters escaped, so it stays one safe line. The argument is never silently dropped,
converted lossily to a replacement-character path, used as that substituted file, or reported
as that substituted path missing.

The refusal is CLI-level ([§FS-errors.2.2](FS-errors.md#22-cli-level-message)), the hidden completion helper's included: its quiet
config- and scan-failure rule ([§FS-completions.2](FS-completions.md#2-internal-dynamic-helper)) starts only after this boundary accepts the
arguments. Decided in [§DF-cli-non-utf8-arguments](../decisions/functional/DF-cli-non-utf8-arguments.md#df-cli-non-utf8-arguments-refuse-non-utf-8-argv-before-dispatch-while-preserving-version-precedence), for [§REQ-never-crashes](../requirements/REQ-never-crashes.md#req-never-crashes-garbage-in-diagnostic-out).

## 6. What is deliberately absent

- No generic `--quiet` / `--verbose` knobs — severity is fixed ([§FS-non-goals.9](FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization)), and a clean
  text `grund check` already has a single fixed `success` line ([§GOAL-friendliness-first.1](../goals.md#1-hard-requirements)).
  `check --only <code>` / `--ignore <code>` is a scoped query over stable finding identities,
  not a presentation mode or a project policy knob ([§FS-check.1](FS-check.md#1-inputs)).
- No `--color` — the inert v1 `[output] color` key has no run-flag counterpart
  ([§FS-config.3.6](FS-config.md#36-output--report-format)).
- No `--config <file>` override — config is discovered by walking up from the command path
  ([§FS-config.1](FS-config.md#1-file-location-and-discovery)), never pointed at, so two installs on one tree agree ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree));
  `grund config show [path]` reports what was discovered from that path.
- No interactive flags, no TUI, no prompts ([§FS-non-goals.10](FS-non-goals.md#10-interactive-mode)).
- No `grund graph`, no `grund new` — visualising the ID graph is a non-goal
  ([§FS-non-goals.6](FS-non-goals.md#6-decision-database-audit-log-history-tracking)), and creating the file for a new declaration is the caller's job after
  `grund id` ([§FS-id.7](FS-id.md#7-what-id-does-not-do)). `grund schema` draws the configuration, never declarations or
  their citations ([§FS-schema](FS-schema.md#fs-schema-grund-schema-shows-what-a-repository-is-held-to-read-from-its-configuration-alone)).
