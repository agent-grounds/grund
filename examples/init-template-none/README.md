# `grund init --template=none`: adopting grund in a repository with its own kinds

This example teaches setting up a conformant repository that already has a
document taxonomy of its own ([§FS-examples.2](../../docs/functional-spec/FS-examples.md#2-canonical-use-cases)). It is for a maintainer, or an agent
running the `grund-init` skill, who wants grund's citations without grund's
nine default kinds: the existing-structure adoption model
([§FS-init.5.2](../../docs/functional-spec/FS-init.md#52-adopting-an-existing-docs-heavy-repo)).

Open `repo/` first. It holds an architecture decision under `docs/adr/` and a
runbook under `docs/runbooks/`, and no grund config.

## Run it

From the repository root, on a copy of the fixture:

```bash
cp -r examples/init-template-none/repo /tmp/adopt && git -C /tmp/adopt init -q
grund init /tmp/adopt --template=none
echo $?    # 0
```

The run writes two files and touches nothing under `docs/`
([§FS-init.3.6](../../docs/functional-spec/FS-init.md#36-bare-init-writes-the-config-and-the-entrypoints-and-nothing-in-a-home)), which `expected.repo/` records byte for byte:

- `grund.toml`, the `none` template's config ([§FS-init.2.4.9](../../docs/functional-spec/FS-init.md#249-the-none-templates-config)). It
  sets `kinds = []`, which declares no kinds
  ([§FS-config.3.4.4.5](../../docs/functional-spec/FS-config.md#3445-kinds---declares-no-kinds)), where leaving `[[kinds]]` out would
  have kept the nine defaults.
- `AGENTS.md`, whose Project map teaches the first `[[kinds]]` table rather than
  listing an empty one ([§FS-init.2.3.4.4.2](../../docs/functional-spec/FS-init.md#23442-no-kinds-yet)).

On stderr it reports those two paths, then a `next:` block whose first step is
to declare a first kind ([§FS-init.2.2.2.2](../../docs/functional-spec/FS-init.md#2222-a-project-with-no-kinds)). Stdout is empty.

## What to do next

Replace `kinds = []` with the repository's own kinds, then run `grund init`
again so the entrypoint lists them:

```toml
[[kinds]]
kind = "ADR"
folder = "docs/adr"
title = "Architecture decisions"

[[kinds]]
kind = "runbook"
folder = "docs/runbooks"
citable = false
title = "Operational runbooks"
```

## The trade-off

- **Benefit:** the config starts from the repository's taxonomy rather than
  grund's, so there is nothing to delete and no starter file lands over
  existing docs.
- **Cost:** until a kind is declared nothing is an ID, so `grund check` passes
  having checked nothing; its warning says so. `--template=none --scaffold` is
  refused, because the template has no starter files to lay.
