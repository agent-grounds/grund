# Workspace member that cites no other project

A workspace root `graal` lists an open member, `ce/substratevm`, and a closed
one, `ee`. The open member must never cite the closed one. It says so in its
own config, without naming `ee` there: it closes its citation default and lists
its own kinds' targets under `may`
([§FS-config.3.9.3](../../docs/functional-spec/FS-config.md#393-alias-matching)).
The use-case is keeping one member of a workspace from citing any other
project.

```toml
# repo/grund.toml
grund_config_version = 2
project_name = "graal"

[workspace]
members = ["ce/substratevm", "ee"]
```

```toml
# repo/ce/substratevm/grund.toml, after the member's kinds
[rules.citations]
default = "must-not"

[rules.citations.FS]
may = ["FS"]

[rules.citations.code]
may = ["FS"]
```

It is for whoever maintains a workspace with an open/closed split. Open
`repo/ce/substratevm/grund.toml` first. The example pins four runs
([§FS-examples.5.5](../../docs/functional-spec/FS-examples.md#55-an-example-may-pin-several-runs)).
The [citation-directions guide](../../docs/user-facing/citation-directions.md#one-member-that-cites-no-other-project)
explains the recipe.

## Run 1: the root refuses the leaks

The open member cites the closed member's `FS-pgo` twice: from its `FS` file,
and from `docs/notes.md`, which is outside every kind home.

```bash
grund check examples/workspace-isolated-member/isolated-1-leaks/repo
echo $?    # 1
```

```text
ce/substratevm/docs/notes.md:1: error: code must not cite ee/FS (citation direction) — re-point the citation or downgrade it to a plain Markdown link
ce/substratevm/docs/requirements.md:6: error: FS must not cite ee/FS (citation direction) — re-point the citation or downgrade it to a plain Markdown link
```

A bare `FS` matches the member's own `FS` only, so `ee/FS` is on no list. It
falls to the closed default
([§FS-config.3.9.4](../../docs/functional-spec/FS-config.md#394-defaults-and-precedence)).
The same rule at the workspace root would say nothing here: a root is never
consulted for a member's sites
([§FS-config.principle.boundary](../../docs/functional-spec/FS-config.md#principleboundary-the-project-is-the-widest-committed-scope)).

## Run 2: the member's own run passes once the leaks are gone

```bash
grund check examples/workspace-isolated-member/isolated-2-member-clean/repo/ce/substratevm
echo $?    # 0
```

```text
success
```

The recipe names no other project, so the member's own run loads it.

## Run 3: a kind left off fails its own citations

The member's config drops the `[rules.citations.code]` row. Its homeless file
cites the member's own `FS-image`.

```bash
grund check examples/workspace-isolated-member/isolated-3-no-code-row/repo
echo $?    # 1
```

```text
ce/substratevm/docs/notes.md:1: error: code must not cite FS (citation direction) — re-point the citation or downgrade it to a plain Markdown link
```

## Run 4: a pinned sibling fails the member's own run

A third member, `ce/compiler`, joins the workspace. The open member allows it
with `[rules.citations.FS] may = ["FS", "compiler/FS"]`. The workspace root
accepts that. The member's own run cannot resolve the alias:

```bash
grund check examples/workspace-isolated-member/isolated-4-pinned-sibling/repo/ce/substratevm
echo $?    # 2
```

Stdout is empty, and stderr carries one line:

```text
error: grund.toml: [rules.citations.FS] may names `compiler/FS`: unknown project alias compiler; no workspace is in scope here, so the alias cannot be resolved — check from the workspace root; this became an error in grund 0.19.0
```

## The trade-off

- **The benefit.** The open member's config never names the closed project,
  and a citation of any other project fails the check from the workspace root.
- **The first cost.** Every citing kind must list its own targets under `may`,
  each non-citable kind and the homeless `code` kind included. A kind left off
  fails on its local citations, as run 3 shows:
  `ce/substratevm/docs/notes.md:1: error: code must not cite FS (citation direction)`.
- **The second cost.** Allowing one sibling but not another takes a pinned
  target. The member's own run then refuses its config, as run 4 shows, so
  that member is checked from the workspace root only.
