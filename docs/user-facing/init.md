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

## `--scaffold`: a fresh repository

```bash
grund init --scaffold
```

`--scaffold` also lays the template's starter files — `requirements.md`,
`docs/grund.md`, `docs/goals.md`, the architecture and decision READMEs, and the
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
