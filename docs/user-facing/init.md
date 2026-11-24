# Setting up a repository with `grund init`

`grund init` answers two questions: **which template** to lay, and **how much
of it**. A template is a `grund.toml` plus a starter file for each home that
config names ([§FS-init.1.6](../functional-spec/FS-init.md#16-templates)). `--template <name>` picks one, and `--scaffold`
adds its starter files to what bare `init` writes. This page walks through the
three runs most repositories need; the example under
[`examples/init-template-none`](../../examples/init-template-none/) runs the
last one and records its output.

## Bare `init`: the config and the entrypoint

```bash
grund init
```

Bare `init` writes `grund.toml` when the repository has none, and the managed
`grund` block in its agent entrypoints, and **nothing else**: no file, no
directory and no `.gitkeep` under `docs/`, `tests/` or any kind home
([§FS-init.3.6](../functional-spec/FS-init.md#36-bare-init-writes-the-config-and-the-entrypoints-and-nothing-in-a-home)). That makes it safe to run on a repository that already has
specs. The config is the default template's, `software-development`: the
`GRUND`, `GOAL`, `FS`, `REQ`, `AR`, `DF`, `DA` and `RM` kinds, plus the
non-citable `e2e` and `integration` test homes.
`grund init --template=software-development` is the same run.

The file is short. It is version 2 (`grund_config_version = 2`) and writes only
what differs from a default: the project name, each kind's row with its places,
and each kind's title ([§FS-init.2.4.3](../functional-spec/FS-init.md#243-every-written-key-differs-from-its-default)). Every other setting is left to its
default, so a new project gets slug-only IDs such as `FS-login`, named sections,
and the whole tree scanned. `grund config show` prints the settings in effect,
defaults included, in the file's own version ([§FS-config.4.2](../functional-spec/FS-config.md#42-grund-config-show-path)); copy a
line from it into `grund.toml` to change that setting. An opt-in table that is
off, such as `[schema.leads.words]` while no lead is capped, is not printed:
`grund config explain` lists every table and answers for its keys
([§FS-config.4.2](../functional-spec/FS-config.md#42-grund-config-show-path)). A config file an
earlier `init` wrote keeps loading unchanged, and `grund config migrate` converts
it when you choose to.

## Your first declaration

Allocate an ID, write a declaration with a section, list it in that kind's
`README.md` index, then cite the section. See [the structure guide](structure.md)
for the declaration and citation rules.

```bash
$ grund id FS "store widgets"
FS-store-widgets
$ mkdir -p docs/functional-spec src
$ cat > docs/functional-spec/store-widgets.md <<'EOF'
# FS-store-widgets: Store widgets

The storage layer retains each widget record until it is removed.

## terms: Terms

## 1. Storage behavior
EOF
$ cat > docs/functional-spec/README.md <<'EOF'
- [§FS-store-widgets](store-widgets.md#fs-store-widgets-store-widgets)
EOF
$ cat > src/main.rs <<'EOF'
// §FS-store-widgets.1
fn main() {}
EOF
$ grund check
success
```

The managed block teaches Project namespaces only for workspace roots and
claimed members. Standalone repositories omit it. A discovered workspace keeps
that guidance even if a missing member or duplicate alias prevents its Workspace
members list from rendering
([§FS-init.2.3.4](../functional-spec/FS-init.md#234-managed-block-content-sections)).
Each other project in the workspace is one bullet that links its directory,
as in ``- [`billing`](packages/billing/)``, whether or not that project has been
initialized yet, so the order in which you run one `grund init` per project
does not matter
([§FS-init.2.3.4](../functional-spec/FS-init.md#234-managed-block-content-sections)).
In a checkout without one of the workspace's `optional_members`, `init` keeps
each block's Workspace members list as it is and says so in a note, so the list
is refreshed only from a full checkout
([§FS-init.2.3.4](../functional-spec/FS-init.md#234-managed-block-content-sections)).
For when and how to split off an independently checked app, package, service or
subproject, run `grund guide members`, which prints
“Splitting off a member”
([§FS-init.5.4](../functional-spec/FS-init.md#54-splitting-off-a-member)).

### The block's revision is not grund's release

The managed block opens with a labelled heading such as
`## Grounding with grund`. The revision in that heading describes the block's
text, not a grund release: `grund -V` prints the release you are running
([§FS-init.2.3.9](../functional-spec/FS-init.md#239-delimiters)). A
config selects one of four current revisions: v47, v48 once a kind declares rules,
and v49 or v50 when its guidance lands in a scoped `AGENTS.md` or a declaration
shape renders ([§FS-init.2.3.7](../functional-spec/FS-init.md#237-the-block-version)).
`grund check` reports an older block as outdated, and one `grund init` re-renders
it. A current block whose text no longer matches what `init` would write — after a
config edit, a migration, or a member joining the workspace — is reported stale,
naming the part that differs, and `grund init` refreshes it
([§FS-check.3.5](../functional-spec/FS-check.md#35-invalid-agent-entrypoint-init-block)).
`[schema.sources] fenced` is one such edit: the Rules bullet on how to mention an
ID without citing it then ends “or use a fenced code block not tagged `mermaid`”,
naming the fences left unread, and with `fenced = []` it reads as before
([Reading the citations in a diagram fence](sources.md#reading-the-citations-in-a-diagram-fence)).
`init` writes the labelled heading. A grund older than the block reports
it as malformed, so upgrade grund first rather than editing the block by hand.

## Exit codes and reports

`init` exits `1` for a rule finding in ordinary, `--check` and `--dry-run`
runs. Independently, `--check` exits `1` when a path is pending. Operational
errors earn `2` before either verdict. Otherwise the run exits `0`;
`--dry-run` alone keeps `0` even with pending paths
([§FS-init.4.1](../functional-spec/FS-init.md#41-what-earns-the-1),
[§FS-init.4](../functional-spec/FS-init.md#4-exit-codes)).

Read the two reports separately: located rule errors are on stdout; path
events are on stderr. Under `--check`, `would-write`, `would-append` and
`would-update` identify pending paths, alongside pending scoped-file events.
A rule finding and pending paths can coexist. A finding alone does not mean
paths are stale; `exists` or `updated` describes only that path, not the whole
run's success ([§FS-init.4.1](../functional-spec/FS-init.md#41-what-earns-the-1)).

An invalid rule withholds writes. A rule whose alias cannot be judged in member
scope is instead reported and rendered as authored
([§FS-rules.4.1](../functional-spec/FS-rules.md#41-a-rule-this-scope-cannot-verify)).
That member still exits `1` after writing, updating or finding every path
current, including under `--check` and `--dry-run`. The exit alone therefore
cannot serve as that member's staleness gate. Run `grund check` from the
workspace root to judge the alias-pinned rule with the full vocabulary
([§FS-init.4.1](../functional-spec/FS-init.md#41-what-earns-the-1)).

## `--scaffold`: a fresh repository

```bash
grund init --scaffold
```

`--scaffold` also lays the template's starter files — the
`docs/functional-spec/` index for `FS`, the `docs/requirements/` index for `REQ`, `docs/grund.md`, `docs/goals.md`, the architecture and decision READMEs, and the
two test homes — at the homes the config names ([§FS-init.2.1.3](../functional-spec/FS-init.md#213-the-software-development-starter-files)). An existing
starter file is left alone unless you pass `--force`. Use it on an empty
repository, or one that wants to move to the canonical layout.

`--scaffold` used to be spelled `--docs`. grund 0.18.0 still took the old
spelling with a warning; since grund 0.19.0 it is an unknown flag, and the run
writes nothing ([§FS-init.1](../functional-spec/FS-init.md#1-inputs)):

```text
error: unknown flag `--docs`
```

A script or CI step that still runs `grund init --docs` spells it `--scaffold`.

## `--template=none`: a repository with its own kinds

A repository with a taxonomy of its own — ADRs, RFCs, runbooks — does not want
the default template's kinds. `--template=none` writes a version-2 `grund.toml`
that declares none ([§FS-init.1.6](../functional-spec/FS-init.md#16-templates)):

```bash
$ grund init --template=none
wrote AGENTS.md
wrote grund.toml

next:
  1. declare a first kind in grund.toml, then run `grund init` again so AGENTS.md lists it
  2. run `grund check`
  3. allocate an ID:  ID=$(grund id <KIND> "…")  then add it to that kind's home
see AGENTS.md for the full workflow.
```

The config holds the version, the project name and a comment on declaring a
first kind, and no `[schema.kinds]` row
([§FS-init.2.4.9](../functional-spec/FS-init.md#249-the-none-templates-config)). Version 2 has no built-in kinds, so no row means
no kinds. Until a kind is declared, nothing is an ID: `grund check`
passes and says why it found nothing, `grund id` answers `known kinds: none`,
and the managed block's Project map tells the agent how to add the first kind
instead of listing an empty one.

When sources exist but no kind is declared, the warning names the offline help
for a kind's keys ([§FS-check.4.5](../functional-spec/FS-check.md#45-nothing-recognized)):

```bash
grund config explain '[schema.kinds.<NAME>]'
```

It teaches `folders`, `files` and every other kind key. You can also ask
`grund config explain schema.kinds`: that parent answer includes the same rows,
labelled as `[schema.kinds.<NAME>]` keys
([§FS-config-explain.1.2](../functional-spec/FS-config-explain.md#12-rungs)).

To declare the repository's kinds, add one `[schema.kinds.<KIND>]` table per
kind, and a `[presentation.kinds.<KIND>]` title for the Project map:

```toml
[schema.kinds.ADR]
folders = ["docs/adr"]

[presentation.kinds.ADR]
title = "Architecture decisions"
```

The first two lines are enough to declare `ADR`; the presentation table gives
it a readable title. Keep a `docs/adr/README.md` index linking each declaration,
such as `[ADR-use-postgres](use-postgres.md)` for a document headed
`# ADR-use-postgres: Use Postgres`.

Then run `grund init` again, so the entrypoint's Project map lists `ADR`. That
run refreshes the managed block and prints no `next:` block:

```text
updated AGENTS.md
exists grund.toml
```

The guidance only ever names a kind the config declares
([§FS-init.2.2.2](../functional-spec/FS-init.md#222-the-next-block)): `FS` when it has a home, otherwise the first citable
kind that has one, here `ADR`. Its home `docs/adr` already exists, so a
refresh has nothing left to teach. Allocate the first ID with
`grund id ADR "…"`, and run `grund check` to see what the repository already
cites. A run whose selected home is missing asks you to create it, as in
`create the ADR home (docs/adr)`, and so does every re-run until it exists,
even one that changes nothing. It never offers `--scaffold`, because
that lays grund's whole software-development starter, the files listed under
`--scaffold` above, rather than your kind's home alone. That is a tree your
project did not ask for, and here it would not even lay the home: it writes no
`docs/adr`.
A kind with no home at all is taught as a declaration in a source doc-comment.

`none` has no starter files, so `--template=none --scaffold` is refused, exit
`2`, and so is a template name grund does not know:

```text
error: --scaffold has nothing to lay: the none template has no starter files
error: unknown template `docs` — the built-in templates are software-development and none; a template from a path is not supported yet
```

## Scoped guidance in a version-2 config

In a version-2 config, `init` writes each row's guidance into the managed
block of an `AGENTS.md` in the deepest directory holding the row's places,
not into the root entrypoint ([§FS-init.2.3.5](../functional-spec/FS-init.md#235-citation-directions)). A row's guidance is its
citation directions, its clause of the grounding sentence, its kind's
declaration shape and the chapter rules about its kind. Folders
`docs/spec/core` and `docs/spec/plugins` land in `docs/spec/AGENTS.md`, a
file place counts as its parent directory, and a row whose places share no
directory stays in the root block, as does a destination directory that does
not exist yet. A scoped block holds the citation-directions legend, the
bullets of the rows that land there and the closing line; the root block keeps
the bullets of the rows that land at the root
([§FS-init.2.3.5](../functional-spec/FS-init.md#235-citation-directions)), so the single section the
[citation-directions page](citation-directions.md) shows is split by
directory. A config that declares no citation rule has no bullets and no
legend to split: a scoped block whose rows ground holds the grounding sentence
alone, `Every file under tests/e2e/ must cite one.`, and a block where no
grounding clause lands has no citation-directions section at all
([§FS-init.2.3.5](../functional-spec/FS-init.md#235-citation-directions)). The example under
[`examples/scoped-guidance`](../../examples/scoped-guidance/) records one run:

```text
wrote AGENTS.md
wrote docs/spec/AGENTS.md
wrote tests/AGENTS.md
exists grund.toml
```

A scoped file `init` creates holds the managed block and nothing else. The
root block lists every scoped file under `### Scoped guidance`
([§FS-init.2.3.5](../functional-spec/FS-init.md#235-citation-directions)), and that list is what `init` owns: a later run updates
those files, and removes the ones the config no longer places, deleting a file
only when the block was all of it. The entrypoint flag `--claude` creates a real
`CLAUDE.md`; it does not create a symlink to `AGENTS.md`. Where the root
`CLAUDE.md` or `GEMINI.md` is a symlink to `AGENTS.md`, `init` links the same
name in each scoped directory.

A scoped file, or a declared field, moves the block to its scoped revision; see
[the current revision list](#the-blocks-revision-is-not-grunds-release). To keep every
row in the root file, set `[presentation] rules = "root"`: the root block is
then the configured root guidance, plus the declaration shapes. `home` is the
default and writes scoped files; `rules = "root"` keeps every row in the root
block.

Whether an agent reads a scoped file by itself depends on the agent. For one
that does not, the root block's list tells it to read the file before writing
there. As read from each tool's own documentation
([§DF-scoped-guidance.loading](../decisions/functional/DF-scoped-guidance.md#loading-which-agents-load-a-scoped-file-by-themselves)):

| Agent | What its documentation says | Documentation, read 2026-10-14 |
|---|---|---|
| Claude Code | Loads a subdirectory's `CLAUDE.md` once it reads, writes or edits a file in that subdirectory; automatic through the symlink when the root `CLAUDE.md` points to `AGENTS.md`. A real `CLAUDE.md` from `--claude` falls back to the root pointer. | <https://code.claude.com/docs/en/memory> |
| Codex | Builds its instruction chain once at startup, from the project root down to the working directory, and none below it. Not automatic: it follows the root pointer, or is started in the directory (`--cd docs/spec`). | <https://learn.chatgpt.com/docs/agent-configuration/agents-md> |
| Gemini CLI | When a tool accesses a file, scans that file's directory and its ancestors for `GEMINI.md`; automatic when the nested file exists. `context.fileName` may name `AGENTS.md` instead. | <https://geminicli.com/docs/cli/gemini-md/> |
| Cursor | Applies an `AGENTS.md` in a subdirectory when working with files in that directory or its children; automatic. | <https://cursor.com/docs/context/rules> |
| Windsurf | Applies a subdirectory's `AGENTS.md` only when Cascade reads or edits files inside that directory; automatic. | <https://docs.devin.ai/desktop/cascade/agents-md> |
| Pi | Loads context files from the agent directory, the working directory and its parents at startup; nothing below the working directory. Not automatic: it follows the root pointer. | <https://github.com/badlogic/pi-mono/blob/main/packages/coding-agent/docs/configuration.md> |
| GitHub Copilot | Not verified. The repository-instructions page says the nearest `AGENTS.md` takes precedence but not when a nested one is read, and VS Code's nested `AGENTS.md` support is off by default. It follows the root pointer. | <https://docs.github.com/en/copilot/how-tos/configure-custom-instructions/add-repository-instructions>, <https://code.visualstudio.com/docs/copilot/customization/custom-instructions> |
| Zed | Not verified. Its instructions page describes one project-level file and nothing in subdirectories. It follows the root pointer. | <https://zed.dev/docs/ai/instructions> |

## A repository that already has a config

`init` never writes over an existing `grund.toml`, whichever template you name.
When you name one anyway, it says so rather than doing nothing silently
([§FS-init.1.6](../functional-spec/FS-init.md#16-templates)):

```text
note: grund.toml exists, so the none template's grund.toml is not written — init never overwrites a config
```

The managed block is still rendered from the config the repository has. For
the source settings that fit common repository shapes, see the
[repository-shape examples](init-repo-shapes.md).

## Removing grund from a repository

To remove grund's repository setup, first find the managed guidance blocks:

```bash
grep -rl --exclude-dir=.git 'BEGIN GRUND MANAGED BLOCK' .
```

Delete each block from files that also contain your own text; if `init` created
a file containing only the block, you can delete the whole file. Remove any
symlinks that `init` linked to those `AGENTS.md` files, then remove
`.grund/`, `grund.toml`, and any pre-commit or CI step you
added for grund. Citations and generated links are plain text and can stay or
be removed. For machine-level integration files, see
[Remove it](clickable-citations.md#7-remove-it).
