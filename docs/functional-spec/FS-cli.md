# FS-cli: grund's command-line surface conventions

The behaviour that is not owned by any one subcommand — how `grund` is invoked with no subcommand, the two global flags that short-circuit before any work, and the cross-subcommand flags. Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible) (one screen of help, no surprises) and [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path) (the CLI surface — subcommands, flags, exit-code mapping — is user-visible, so it changes only backward-compatibly or through a named deprecation window).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, body, section, coordinate),
[§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, workspace), [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, severity, verdict), and
[§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations) (snapshot).

## requirements: What every invocation holds to

Two requirements on the command-line surface as a whole, each carrying a mark of
[§FS-config.requirements](FS-config.md#requirements-what-the-config-contract-holds-to). Three properties are left out because a goal or requirement already
states them: that the exit-code mapping is fixed is [§GOAL-friendliness-first.2](../goals.md#2-what-this-rules-out), which
[§FS-cli.5](FS-cli.md#5-exit-code-mapping-is-fixed) freezes by name; that the surface changes only through a named window is
[§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path) and [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path); and that every address a help
page prints resolves where it lands is [§REQ-shipped-surfaces.1](../requirements/REQ-shipped-surfaces.md#1-no-shipped-or-printed-byte-names-a-declaration-of-this-repository).

### requirements.before-load: A usage mistake is answered before the tree is read — realized, directional for the next flag

What the arguments alone can refuse is refused before config discovery, before any
`grund.toml` is read and before any scan: an unsupported run-flag value ([§FS-cli.3.5](FS-cli.md#35-a-bad-run-flag-is-answered-before-the-load)), a
flag placed before its subcommand ([§FS-cli.4.1](FS-cli.md#41-a-flag-placed-before-the-subcommand)), an unknown or malformed flag, a missing
value, a conflicting pair, a second path ([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location), [§FS-cli.3.2](FS-cli.md#32-at-most-one-path)), and a bad `check`
selector ([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)); `--version` and `--help` are answered before it as well ([§FS-cli.2](FS-cli.md#2-global-flags)).
So a mistake on the command line is reported as itself whatever tree it points at, and a
broken `grund.toml` never hides it. A flag added from here on is validated,
as far as its own spelling decides, before the load. Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

### requirements.as-written: An invocation runs as written or is refused — realized, directional for the next subcommand

No invocation is quietly narrowed or read as another. A second path is refused, never
dropped ([§FS-cli.3.2](FS-cli.md#32-at-most-one-path)); a flag before a known subcommand is refused, never read as a query for
an ID spelled like the subcommand ([§FS-cli.4.1](FS-cli.md#41-a-flag-placed-before-the-subcommand)); a flag the command does not take is an
error ([§FS-cli.3](FS-cli.md#3-cross-subcommand-flags)); bare `grund` is an error, never a help page at exit `0` ([§FS-cli.1.1](FS-cli.md#11-why-these-defaults));
and a first word that is neither a subcommand nor an ID fails as the query it was read as,
with a hint toward the subcommand it may have meant ([§FS-cli.1.2](FS-cli.md#12-a-first-word-that-is-not-an-id)). Where one word has two
readings, the order that picks is written down: help dispatch before the ID query
([§FS-cli.1.3](FS-cli.md#13-a-help-request-with-an-unknown-first-word)), and `--version` above everything ([§FS-cli.2](FS-cli.md#2-global-flags)). A subcommand or flag added from
here on keeps this: a word it makes ambiguous is refused or given a written precedence,
never resolved by a guess. Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

## 1. The default subcommand

- `grund` with no arguments is a CLI-level error ([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)): the fallback that ran `grund check .` was removed in grund 0.16.0, and the message names the explicit forms to write instead.
- `grund <ID>[.<section>] …` (where the first non-flag word is not a known subcommand) is the ID-read query specified by [§FS-show.1](FS-show.md#1-inputs), byte-for-byte equivalent to the explicit `show` subcommand, including show flags written before the ID: `grund --toc FS-check` reads the same body as `grund FS-check --toc`. With no path, both resolve from `.`. A first non-flag word that *is* a known subcommand, after leading flags, is not this query but the misplaced-flag error of [§FS-cli.4.1](FS-cli.md#41-a-flag-placed-before-the-subcommand).
- `grund <subcommand> …` dispatches to that subcommand: `check`, `show`, `list`, `refs`, `cover`, `fmt`, `fetch`, `id`, `init`, `config`, `schema`, `guide`, `completions`, `integrations`. The hidden `complete` subcommand is reserved for generated shell scripts ([§FS-completions.2](FS-completions.md#2-internal-dynamic-helper)), and is the only hidden one: `agent-setup-instructions`, `guide`'s old name, was removed in grund 0.20.0 and is no subcommand ([§FS-init.5.7](FS-init.md#57-agent-setup-instructions-is-the-deprecated-name-of-guide)); `fetch` is the explicit one-ID writer specified by [§FS-fetch](FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot).

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
known commands: check, show, list, refs, cover, fmt, fetch, id, init, config, schema, guide, completions, integrations
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
- **A usage line.** Directly under the statement, with no blank line between them, one line naming the invocation forms `grund <ID>[.<section>]`, `grund <COMMAND> [ARGS]`, `grund <COMMAND> -h` and `grund -V`.
- **The commands, grouped by intent.** Three groups, in this order and with a blank line between them: *Query the catalog* (`show`, `list`, `refs`, `cover`), *Check and author* (`check`, `fmt`, `id`, `fetch`) and *Set up* (`guide`, `init`, `config`, `schema`, `integrations`, `completions`). Each row gives the command, a one-line description in the vocabulary of [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) and [§FS-terms.terms.2](FS-terms.md#terms2-citations), and a sample call.
- **A footer.** One line giving the public address of the user guides and their runnable examples, `https://github.com/agent-grounds/grund/tree/main/docs/user-facing`: an address a reader with only the installed binary can open, never a path in grund's own source tree.

There is no options block. A flag's place is on each command's own page ([§FS-cli.2.3](FS-cli.md#23-a-subcommands-help-page)), and a flag written before the subcommand is answered by the error of [§FS-cli.4.1](FS-cli.md#41-a-flag-placed-before-the-subcommand), which names where it goes.

The whole page fits one screen ([§GOAL-friendliness-first.1](../goals.md#1-hard-requirements)), which this spec sets at ≤ 24 lines of at most 100 columns, so each description is a single terse line: the `show` row still gestures at *why* the command exists ("Print a coordinate's lead for agent context (default)."), and `fetch` says that it materializes one configured external snapshot, with the full rationale on each command's own help page. The `guide` row opens *Set up*, because it is where a newcomer starts: it says the guide teaches setting up and using grund and that it lists its sections, and its sample call is the bare command, whose front page is that list ([§FS-init.5.6](FS-init.md#56-the-front-page)):

```text
  guide         How to set up and use grund; lists its sections.        grund guide
```

The `fmt` row names both link and section-lock maintenance, so a reader can discover
more than marker conversion within that same budget. The full workflow belongs on
its own page ([§FS-cli.2.3.3](FS-cli.md#233-fmt-teaches-declaration-maintenance)).

The `list` row makes the declarations nothing cites reachable from the top level, because
"which specs does nothing cite?" is a question a reader asks of the catalog, and one that
`refs` answers only one ID at a time, with an empty answer ([§FS-refs.2.1](FS-refs.md#21-an-id-with-no-citations)). Its description
names that set beside the whole catalog, and its sample call is the flag that selects it,
`--unused` ([§FS-list.1.3](FS-list.md#13---unused)), so the one-call answer is found without opening `list`'s own page:

```text
  list          Every declared ID, or the ones nothing cites.           grund list --unused
```

### 2.3 A subcommand's help page

Static help describes configuration scope truthfully across supported versions, without discovering a configuration just to render help. In particular, `check` help preserves the no-config default directories, and either uses version-neutral scope terms or explicitly labels v1's `[scan] include` and v2's places ([§FS-config-v2.schema.places](FS-config-v2.md#schemaplaces-rows-and-their-places)). Its `--full` description widens past configured scope as [§FS-config-v2.schema.places.boundaries.7](FS-config-v2.md#schemaplacesboundaries7---full-widens-past-the-places) specifies; it does not suggest bypassing source exclusions ([§FS-config-v2.schema.sources.exclude](FS-config-v2.md#schemasourcesexclude-exclude-is-a-gitignore-style-glob-list)). An unqualified instruction to set `[scan] include` is not v2 repair advice.

`grund help <subcommand>` and `grund <subcommand> --help` (and `grund <subcommand> -h`) print *that subcommand's* page on stdout, exit `0`: its usage line, its arguments, every flag with a one-line example, the exit-code meanings for that subcommand, and a one-line recovery hint where the common failure has an obvious next step (e.g. `show`'s page says how to find an ID; `id`'s page shows the `$EDITOR` follow-up). `grund help` with no argument is the top-level page; `grund help <unknown>` is the unknown-command error ([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)).

Where a user guide or a runnable example covers the command, its page ends with that guide's and example's public address: `Guide:` lines, then `Example:` lines, each giving a `https://github.com/agent-grounds/grund/blob/main/<file>` or `https://github.com/agent-grounds/grund/tree/main/<directory>` address. The address names `main`, never a version tag, because a development build has no tag ([§REQ-shipped-surfaces.1](../requirements/REQ-shipped-surfaces.md#1-no-shipped-or-printed-byte-names-a-declaration-of-this-repository)). Every such address `grund` prints names a file (`blob`) or a directory (`tree`) that exists in the tree it was built from, so a printed link cannot go stale.

#### 2.3.1 Every flag has an example

A flag's one-line example is a command line that runs the page's subcommand — `grund <subcommand> …`, or for `show`, the default command, `grund …` as well — and passes that flag, spelled `--flag`, `--flag VALUE` or `--flag=VALUE`. It is written on the flag's own option entry after `e.g.`, or as one line of the page's examples block (a block whose heading ends in `examples:`). A bare value is not an example: `e.g. FS.requirements` or `e.g. --section 3.1` says what the flag takes, not how the command is run. The rule holds every flag the page names, whether in its usage lines or as an option entry, including the run flags every page shares ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)): a page that names a flag gives it an example.

#### 2.3.2 A page whose path narrows says what a run without one covers

The page of each command whose `<path>` narrows what it scans or reports, `check`, `fmt`, `cover`, `list` and `refs`, says what a run given no path covers ([§FS-scan.path.none](FS-scan.md#pathnone-a-run-with-no-path-covers-the-project-discovery-finds)) in one line below its usage, worded once for all five:

```text
With no PATH, the run covers the whole project or workspace discovered from the current directory; a PATH narrows it.
```

None of the five says PATH defaults to `.`: from below the config root, the run given `.` is a narrowed run and not the one given no path. A page whose path only chooses the project that answers, as `config`'s and `schema`'s do, keeps the default it states. `show`'s path does both ([§FS-scan.path.show](FS-scan.md#pathshow-show-resolves-the-id-against-the-project-the-path-discovers)), so its `--path` entry gives each form its own and names no `.` default: one coordinate answers alike with `.` and with no path, and `--batch --all` and `--glob`, which the path narrows, query with none the whole project or workspace discovered from the current directory.

#### 2.3.3 `fmt` teaches declaration maintenance

The `fmt` page explains that, after the author renames a declaration or moves its
file and updates affected authored citation IDs, `grund fmt --write` re-derives
generated Markdown link destinations and anchors when the link pass applies
([§FS-fmt.6.3](FS-fmt.md#63-idempotency-and-re-derive)). It names section-lock
maintenance and says to run from the project's config root or workspace root for
eligible lock updates ([§FS-section-lock.write.rules](FS-section-lock.md#writerules-add-refresh-drop)).
Its examples include a root-scoped `grund fmt --write` invocation whose explanation
connects it to this rename or move workflow, and the page says how that run's
report reads: a dry-run row names every kind of rewrite its line receives
([§FS-fmt.3.5.1](FS-fmt.md#351-a-line-with-several-rewrites-carries-the-first-label)), the `--write` summary is followed by its totals
([§FS-fmt.3.7](FS-fmt.md#37-the---write-report)), and a lock file the rename deletes or creates is said to be
deleted or created, on its dry-run row and on its `--write` line
([§FS-section-lock.write.report](FS-section-lock.md#writereport-what-fmt-reports)).

This teaching preserves the boundaries: `fmt` does not rename authored citation
IDs or repair arbitrary URLs, a file move alone leaves an ID-keyed lock's name
alone ([§FS-section-lock.file.shard](FS-section-lock.md#fileshard-one-file-per-cited-declaration)),
and `fmt` never silently approves a protected section move
([§FS-section-lock.write.never-a-move](FS-section-lock.md#writenever-a-move-fmt-never-approves-a-move)).
The preview/write contract remains [§FS-fmt.3](FS-fmt.md#3-outputs)'s.

#### 2.3.4 `show` says a section read opens with its heading

The `show` page's `--format` row describes what each format prints for a section read as well
as for a whole declaration, because an agent writes its checks against that row before it runs
the command. `text`, the default, is the body, which opens with a selected section's own heading
line ([§FS-show.2.2](FS-show.md#22-section), [§FS-show.3.1.1](FS-show.md#311-text)); `md` adds the declaration heading above it
([§FS-show.3.1.2](FS-show.md#312-md)); `json` wraps the text body ([§FS-show.3.1.3](FS-show.md#313-json)). The row reads exactly:

```text
  --format text|md|json  text (default) is the body, opening with a selected section's
                         own heading line; md adds the declaration heading above it;
                         json wraps the text body
```

A row that called `text` the body and `md` the format that keeps the heading said that a text
section read is heading-less, which no format is.

## 3. Cross-subcommand flags

- `--format text|json` — accepted by the subcommands with a machine-readable result or finding surface ([§FS-errors.5](FS-errors.md#5-json-format) lists them, [§FS-integrations.5](FS-integrations.md#5-json-format)). `text` is the default; `json` opts into the stable machine shapes, on the streams of [§FS-cli.3.1](FS-cli.md#31-the---format-json-streams). It is not a global flag: the operational commands [§FS-errors.5](FS-errors.md#5-json-format) lists, whose output is human text or generated files, reject `--format`.
- `--path-base project|invocation` — the report base every rendered path is spelled from ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)). `project` is the default.
- Both are **run flags**: a caller's choice for this one run, written after the subcommand as `--flag value` or `--flag=value`. When one is passed it outranks the v1 `[output]` key that sets the same default for the tree ([§FS-config.3.6](FS-config.md#36-output--report-format)), so the precedence is flag, then key, then default. A bad value of either is answered before anything is loaded ([§FS-cli.3.5](FS-cli.md#35-a-bad-run-flag-is-answered-before-the-load)).

  | command | `--format` | honours v1 `[output] format` | `--path-base` |
  |---|---|---|---|
  | `check` (incl. `--watch`), `list`, `refs`, `cover` | `text\|json` | yes | yes |
  | `show` (incl. `--batch` and the bare-ID form), `id`, `integrations` | `text\|json` | no — the key never reached them, and a tree that commits `format = "json"` must not change their bytes on upgrade ([§REQ-backwards-compatibility.1](../requirements/REQ-backwards-compatibility.md#1-what-is-covered)) | yes |
  | `fmt`, `config validate`, `config show`, `config migrate` | rejected | — | yes: they render paths and config-load errors |
  | `schema` | its own `table\|mermaid\|dot` ([§FS-schema.1](FS-schema.md#1-the-command)); `text` and `json` are refused | no | yes: it renders config-load errors |
  | `init`, `fetch`, `completions`, `guide` | rejected | — | rejected |

  A rejected flag is a CLI-level error of the command's own flag rules, `unknown flag`, per [§FS-cli.4](FS-cli.md#4-errors-with-no-source-location). `fmt` and `config` gain no JSON surface here: designing one is their own spec's business ([§FS-errors.5](FS-errors.md#5-json-format)). The inert `[output] color` key gets no run-flag counterpart and there is no `--color`.
- A path argument, when a subcommand other than `init` takes one, is resolved the same way everywhere: config discovery walks up from it, or from the current directory when none is given ([§FS-config.1](FS-config.md#1-file-location-and-discovery)). A run given none covers the project that discovery finds ([§FS-scan.path.none](FS-scan.md#pathnone-a-run-with-no-path-covers-the-project-discovery-finds)). `init`'s path is the directory it writes in, `.` when none is given ([§FS-init.1](FS-init.md#1-inputs)). Every path-taking subcommand accepts at most one ([§FS-cli.3.2](FS-cli.md#32-at-most-one-path)).
- `--only <code>`, `--ignore <code>`, and the boolean `--only-rule` are `check`-only finding-query flags ([§FS-check.1](FS-check.md#1-inputs)); other subcommands reject them. The `check` help page documents them ([§FS-cli.3.3](FS-cli.md#33-the-check-selector-help)).

### 3.1 The `--format json` streams

The stream split is the same as the text form ([§FS-errors.1](FS-errors.md#1-streams), [§FS-distribution.3.0](FS-distribution.md#30-language-neutral-data-shapes)): the command's output — `grund check`'s findings as NDJSON when there are any, a query result as one object (or NDJSON for a list command) — goes to stdout, while a failed ID query's finding and any CLI-level `error:` go to stderr, except that `show --batch --format=json` keeps a failed query inside its stdout envelope ([§FS-errors.5](FS-errors.md#5-json-format)). `grund check --format=json` stays findings-only and does not emit the text-mode `success` line.

### 3.2 At most one path

A second positional — `grund check a b`, `grund <ID> a b`, `grund refs ID a b`, `grund cover a b`, `grund fmt a b`, `grund list a b`, `grund schema a b`, `grund id FS "t" a b` — is a CLI-level error (`error: <subcommand> takes at most one path argument`, exit `2`, [§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)), never a silent use of one and a quiet drop of the rest. `config` already enforces this, and `guide` refuses a second section name the same way ([§FS-init.5](FS-init.md#5-the-guide)); the rule is uniform across the surface, so a typo'd path is reported, never absorbed.

### 3.3 The `check` selector help

The `check` help page documents both `--flag value` and `--flag=value`, repetition and composition, the fact that operational failures remain visible with exit `2`, selected-report exit semantics, one `--ignore agents-init` example, and the complete supported-code catalog in sorted order. The catalog is the discovery path for callers that know a message but not its code. One line beside it says that `grund config explain CODE` says what a code means, its severity and what turns it on ([§FS-config-explain.4](FS-config-explain.md#4-a-codes-answer)).

It documents the third selector on the same footing: that `--only-rule` narrows the report to what the `--rule` sentence authored rather than by code, that the axes intersect, that it refuses with `error: --only-rule requires --rule` when no trial sentence was given, and that a `should`-level sentence also needs `--suggestions` to be seen ([§FS-rules.8](FS-rules.md#8-command-surfaces)). A selector reachable only from the specification is not discoverable at a terminal, which is where a sentence gets tried.

The same help documents `--watch [<path>]`, an example of the immediate-check/every-save loop, recoverable config/read errors, last-completed-status interruption and pre-first-completion/fatal-watcher status `2` ([§FS-check.6.3](FS-check.md#63-lifecycle)). It links the public watch guide for native-backend and notification-silent-filesystem limits, terminal/redirection behavior and invisible clean JSON runs ([§FS-check.6.1](FS-check.md#61-change-detection), [§FS-check.6.2](FS-check.md#62-each-run-is-a-plain-grund-check)). Help and captured README examples must describe working behavior; a future transcript must be captured after implementation, not invented while specifying the feature.

### 3.4 `--path-base` — where report paths are spelled from

`--path-base=project|invocation` selects the report base ([§FS-errors.terms](FS-errors.md#terms-terms)) for the whole run:

- **`project`** (the default) — relative to the config root the run resolved against, the workspace root when a workspace is loaded. This is what v1 `[output] relative_paths = true` gives.
- **`invocation`** — relative to the path you passed, or the current directory if you passed none: the path argument itself when it is a directory, its parent directory when it is a file, else the working directory. This is the CLI base of [§FS-config.3.6.1](FS-config.md#361-relative_paths--false--the-cli-base), so `--path-base=invocation` prints byte for byte what `[output] relative_paths = false` prints today, including the bounded `..` components of [§DF-cli-base-parent-paths](../decisions/functional/DF-cli-base-parent-paths.md#df-cli-base-parent-paths-relative_paths--false-keeps-one-cli-base-and-may-climb-within-the-loaded-root). It is not the literal working directory: `cd / && grund check /abs/ws/src/lib.rs` reports `lib.rs`, never a path that climbs out of the loaded root, which [§FS-errors.4](FS-errors.md#4-determinism) forbids.

**One base for the whole run.** The flag sets the base for every project the run loads, members included: a workspace whose root and member disagree on `relative_paths` reports from the one base the flag names. It applies to every rendered path — the primary `<path>` of a text finding, the paths inside its message, JSON `path` and `sites[].path`, the `error: <path>: <reason>` line of a per-file scan failure ([§FS-workspace.8.7.3](FS-workspace.md#873-paths)), and the config file named by a config-load error ([§FS-errors.2](FS-errors.md#2-the-fixed-shapes)), which is the one path a config key can never move, because the key is read only after that config loaded. `check --watch` applies it on every rerun.

**Relation to the key.** When the flag is absent, the v1 `[output] relative_paths` key decides, exactly as before ([§FS-config.3.6](FS-config.md#36-output--report-format)). When the flag is present it wins: `--path-base=project` overrides a committed `relative_paths = false`, and `--path-base=invocation` overrides the default.

**Help.** The help page of every command that takes the flag shows it with both values defined in the words above, and names `--path-base=invocation` as the replacement for `[output] relative_paths = false`; the commands that honour `[output] format` name `--format json` as its replacement the same way.

### 3.5 A bad run flag is answered before the load

An unsupported `--format` or `--path-base` value is a usage error the caller can fix without touching the repository, so every command answers it **before anything is loaded** — before config discovery, before a member's `grund.toml` is read, before any scan — on stderr with empty stdout and exit `2`, regardless of the other flag's value:

```
$ grund list --format bogus
error: unsupported list format `bogus`
$ grund check --path-base=x
error: unsupported path base `x` (expected project or invocation)
```

Which of two errors a caller sees must not depend on the tree they happened to point at: `grund list --format bogus` in a tree whose `grund.toml` is broken reports the flag, as `check`, `cover`, `show` and `id` already do. A `[output] format` or `relative_paths` key carrying an unsupported value is a property of the tree, so it is reported after the load, like any other config fault. [§FS-cover.1.1](FS-cover.md#11-an-unsupported---format-is-answered-before-the-load) is this rule as `cover` first stated it.

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

- **Which word is the subcommand.** The leading flags are skipped together with the values of the value-taking flags (`--format`, `--path-base`, `--section`, `--path`), written either as a separate word or as `--flag=value`; the first word left is the one tested. Any leading flag counts, not only `--format`: `grund --brief list` is refused the same way. So is `grund --path-base invocation list`, which names `grund list --path-base invocation`; `--path-base` is counted as value-taking so that its value is never mistaken for the ID of a default `show` query.
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
- No `grund graph`, no `grund new` — visualising the ID graph is a non-goal ([§FS-non-goals.6](FS-non-goals.md#6-decision-database-audit-log-history-tracking)), and file creation for a new declaration is the caller's job after `grund id` ([§FS-id.7](FS-id.md#7-what-id-does-not-do)). `grund schema` ([§FS-schema](FS-schema.md#fs-schema-grund-schema-shows-what-a-repository-is-held-to-read-from-its-configuration-alone)) draws the configuration — kinds, places and the rules between them — never declarations or their citations.
