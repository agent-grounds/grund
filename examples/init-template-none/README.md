# `grund init --template=none`: adopting grund in a repository with its own kinds

This example teaches setting up a conformant repository that already has a
document taxonomy of its own ([§FS-examples.2](../../docs/functional-spec/FS-examples.md#2-canonical-use-cases)). It is for a maintainer, or an agent
running the `grund-init` skill, who wants grund's citations without grund's
nine default kinds: the existing-structure adoption model
([§FS-init.5.2](../../docs/functional-spec/FS-init.md#52-adopting-an-existing-docs-heavy-repo)).

Open `1-template-none/repo/` first. It holds an architecture decision under
`docs/adr/` and a runbook under `docs/runbooks/`, and no grund config. The
example pins two runs, one per step ([§FS-examples.5.5](../../docs/functional-spec/FS-examples.md#55-an-example-may-pin-several-runs)).

## Step 1: `grund init --template=none`

From the repository root, on a copy of the fixture:

```bash
cp -r examples/init-template-none/1-template-none/repo /tmp/adopt && git -C /tmp/adopt init -q
grund init /tmp/adopt --template=none
echo $?    # 0
```

The run writes two files and touches nothing under `docs/`
([§FS-init.3.6](../../docs/functional-spec/FS-init.md#36-bare-init-writes-the-config-and-the-entrypoints-and-nothing-in-a-home)), which `1-template-none/expected.repo/` records byte for byte:

- `grund.toml`, the `none` template's config ([§FS-init.2.4.9](../../docs/functional-spec/FS-init.md#249-the-none-templates-config)): a
  version-2 file holding the version, the project name and a comment on
  declaring a first kind. With no `[schema.kinds]` row it declares no kinds, and
  `grund config show` prints every setting it leaves at its default
  ([§FS-config.4.2](../../docs/functional-spec/FS-config.md#42-grund-config-show-path)).
- `AGENTS.md`, whose Project map teaches the first `[schema.kinds]` table rather
  than listing an empty one ([§FS-init.2.3.4.4.2](../../docs/functional-spec/FS-init.md#23442-no-kinds-yet)).

On stderr it reports those two paths, then a `next:` block whose first step is
to declare a first kind ([§FS-init.2.2.2.2](../../docs/functional-spec/FS-init.md#2222-a-project-with-no-kinds)). Stdout is empty.

## Step 2: declare the kinds, then `grund init` again

Add the repository's own kinds to `/tmp/adopt/grund.toml`, then run `grund init`
again so the entrypoint lists them. `2-declare-kinds/repo/` is step 1's tree with
these tables added:

```toml
[schema.kinds.ADR]
folders = ["docs/adr"]
[schema.kinds.runbook]
citable = false
folders = ["docs/runbooks"]

[presentation.kinds.ADR]
title = "Architecture decisions"
[presentation.kinds.runbook]
title = "Operational runbooks"
```

```bash
grund init /tmp/adopt
echo $?    # 0
```

The run rewrites the managed block, whose Project map now lists `ADR` and the
runbooks, and reports exactly this on stderr:

```text
updated AGENTS.md
exists grund.toml
```

No `next:` block follows. The guidance would name `ADR`, the first citable kind
with a home, and its home `docs/adr` already exists, so this refresh leaves
nothing to teach ([§FS-init.2.2.2.3](../../docs/functional-spec/FS-init.md#2223-the-kind-the-guidance-names)). Allocate the first ID in
it with `grund id ADR "…"`. The runbook kind is not citable, so the guidance
never names it.

## The trade-off

- **Benefit:** the config starts from the repository's taxonomy rather than
  grund's, so there is nothing to delete and no starter file lands over
  existing docs.
- **Cost:** until a kind is declared nothing is an ID, so `grund check` passes
  having checked nothing; its warning says so. `--template=none --scaffold` is
  refused, because the template has no starter files to lay.
