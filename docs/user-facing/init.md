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
`GRUND`, `GOAL`, `FS`, `AR`, `DF`, `DA` and `RM` kinds, plus the non-citable
`e2e` and `integration` test homes. `grund init --template=software-development`
is the same run.

The managed block teaches Project namespaces only for workspace roots and
claimed members. Standalone repositories omit it. A discovered workspace keeps
that guidance even if a missing member or duplicate alias prevents its Workspace
members list from rendering
([§FS-init.2.3.4.16](../functional-spec/FS-init.md#23416-project-boundaries)).
For when and how to split off an independently checked app, package, service or
subproject, run `grund agent-setup-instructions` and read “Splitting off a member”
([§FS-init.5.4](../functional-spec/FS-init.md#54-splitting-off-a-member)).

## `--scaffold`: a fresh repository

```bash
grund init --scaffold
```

`--scaffold` also lays the template's starter files — `requirements.md`,
the `docs/requirements/` index for `REQ`, `docs/grund.md`, `docs/goals.md`, the architecture and decision READMEs, and the
two test homes — at the homes the config names ([§FS-init.2.1.3](../functional-spec/FS-init.md#213-the-software-development-starter-files)). An existing
starter file is left alone unless you pass `--force`. Use it on an empty
repository, or one that wants to move to the canonical layout.

`--scaffold` used to be spelled `--docs`. The old spelling still lays the same
files, after a warning that it is removed in grund 0.19.0 ([§FS-init.1.7](../functional-spec/FS-init.md#17---docs-is-the-deprecated-spelling-of---scaffold)):

```text
warning: --docs is deprecated and is removed in grund 0.19.0; use --scaffold, which lays the same starter files
```

## `--template=none`: a repository with its own kinds

A repository with a taxonomy of its own — ADRs, RFCs, runbooks — does not want
the nine default kinds. `--template=none` writes a `grund.toml` that declares
none ([§FS-init.1.6.2](../functional-spec/FS-init.md#162-none)):

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

The config holds `kinds = []`, the spelling of an empty kind set
([§FS-config.3.4.4.5](../functional-spec/FS-config.md#3445-kinds---declares-no-kinds)). It is not the same as leaving `[[kinds]]` out, which keeps
the built-in kinds. Until a kind is declared, nothing is an ID: `grund check`
passes and says why it found nothing, `grund id` answers `known kinds: none`,
and the managed block's Project map tells the agent how to add the first kind
instead of listing an empty one.

To declare the repository's kinds, replace `kinds = []` with one `[[kinds]]`
table per kind:

```toml
[[kinds]]
kind = "ADR"
folder = "docs/adr"
title = "Architecture decisions"
```

Then run `grund init` again, so the entrypoint's Project map lists `ADR`, and
`grund check` to see what the repository already cites.

`none` has no starter files, so `--template=none --scaffold` is refused, exit
`2`, and so is a template name grund does not know:

```text
error: --scaffold has nothing to lay: the none template has no starter files
error: unknown template `docs` — the built-in templates are software-development and none; a template from a path is not supported yet
```

## Scoped guidance in a version-2 config

In a version-2 config, `init` writes each row's guidance into the managed
block of an `AGENTS.md` in the deepest directory holding the row's places,
not into the root entrypoint ([§FS-init.2.3.5.11](../functional-spec/FS-init.md#23511-scoped-guidance-a-rows-guidance-lands-above-its-places)). A row's guidance is its
citation directions, its clause of the grounding sentence, its kind's
declaration shape and the chapter rules about its kind. Folders
`docs/spec/core` and `docs/spec/plugins` land in `docs/spec/AGENTS.md`, a
file place counts as its parent directory, and a row whose places share no
directory stays in the root block, as does a destination directory that does
not exist yet. The example under
[`examples/scoped-guidance`](../../examples/scoped-guidance/) records one run:

```text
wrote AGENTS.md
wrote docs/spec/AGENTS.md
wrote tests/AGENTS.md
exists grund.toml
```

A scoped file `init` creates holds the managed block and nothing else. The
root block lists every scoped file under `### Scoped guidance`
([§FS-init.2.3.5.13](../functional-spec/FS-init.md#23513-the--scoped-guidance-pointer)), and that list is what `init` owns: a later run updates
those files, and removes the ones the config no longer places, deleting a file
only when the block was all of it. Where the root `CLAUDE.md` or `GEMINI.md` is
a symlink to `AGENTS.md`, each scoped directory gets the same symlink
([§FS-init.2.3.5.14](../functional-spec/FS-init.md#23514-writing-scoped-files)), and the report names each one:
`linked docs/spec/CLAUDE.md -> AGENTS.md`, `removed old/AGENTS.md`.

A scoped file, or a declared field, moves the block from v19 to v21, or from
v20 to v22 where a rule kind is enabled ([§FS-init.2.3.7.1](../functional-spec/FS-init.md#2371-the-current-version)). To keep every
row in the root file, set `[presentation] rules = "root"`: the root block is
then what a version-1 config renders, plus the declaration shapes. A
version-1 config always renders at the root.

Whether an agent reads a scoped file by itself depends on the agent. For one
that does not, the root block's list tells it to read the file before writing
there. As read from each tool's own documentation
([§DF-scoped-guidance.loading](../decisions/functional/DF-scoped-guidance.md#loading-which-agents-load-a-scoped-file-by-themselves)):

| Agent | What its documentation says | Documentation, read 2026-10-14 |
|---|---|---|
| Claude Code | Loads a subdirectory's `CLAUDE.md` once it reads, writes or edits a file in that subdirectory; automatic through the symlink. | <https://code.claude.com/docs/en/memory> |
| Codex | Builds its instruction chain once at startup, from the project root down to the working directory, and none below it. Not automatic: it follows the root pointer, or is started in the directory (`--cd docs/spec`). | <https://learn.chatgpt.com/docs/agent-configuration/agents-md> |
| Gemini CLI | When a tool accesses a file, scans that file's directory and its ancestors for `GEMINI.md`; automatic through the symlink. `context.fileName` may name `AGENTS.md` instead. | <https://geminicli.com/docs/cli/gemini-md/> |
| Cursor | Applies an `AGENTS.md` in a subdirectory when working with files in that directory or its children; automatic. | <https://cursor.com/docs/context/rules> |
| Windsurf | Applies a subdirectory's `AGENTS.md` only when Cascade reads or edits files inside that directory; automatic. | <https://docs.devin.ai/desktop/cascade/agents-md> |
| Pi | Loads context files from the agent directory, the working directory and its parents at startup; nothing below the working directory. Not automatic: it follows the root pointer. | <https://github.com/badlogic/pi-mono/blob/main/packages/coding-agent/docs/configuration.md> |
| GitHub Copilot | Not verified. The repository-instructions page says the nearest `AGENTS.md` takes precedence but not when a nested one is read, and VS Code's nested `AGENTS.md` support is off by default. It follows the root pointer. | <https://docs.github.com/en/copilot/how-tos/configure-custom-instructions/add-repository-instructions>, <https://code.visualstudio.com/docs/copilot/customization/custom-instructions> |
| Zed | Not verified. Its instructions page describes one project-level file and nothing in subdirectories. It follows the root pointer. | <https://zed.dev/docs/ai/instructions> |

## A repository that already has a config

`init` never writes over an existing `grund.toml`, whichever template you name.
When you name one anyway, it says so rather than doing nothing silently
([§FS-init.1.6.5](../functional-spec/FS-init.md#165-a-config-that-is-already-there)):

```text
note: grund.toml exists, so the none template's grund.toml is not written — init never overwrites a config
```

The managed block is still rendered from the config the repository has. For
the `[scan]` settings that fit common repository shapes, see the
[repository-shape examples](init-repo-shapes.md).
