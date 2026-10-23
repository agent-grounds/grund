# Workspace: cross-project citation in a monorepo

A tiny three-project workspace: the root `root` declares `FS-platform`, and
two members `api` and `web` each declare their own `FS-session`. Both members
cite back to the root, and the root cites into each member. Every citation
that crosses a project boundary writes the target alias before the ID.

```toml
# repo/grund.toml
project_name = "root"

[workspace]
members      = ["packages/*"]
include_root = true
```

```markdown
# repo/docs/functional-spec/FS-platform.md
The root project pulls together its sub-projects: the API in §api/FS-session
and the web app in §web/FS-session.
```

```markdown
# repo/packages/api/docs/functional-spec/FS-session.md
The platform-level coordinator is §root/FS-platform.
```

## What this teaches

- **Workspace = one citation universe per member.** Both `packages/api` and
  `packages/web` declare `FS-session` independently; neither is a duplicate.
  Local citations stay short — inside `api`, `<§>FS-session` resolves to the
  api project; inside `web`, the same form resolves to the web project.
  (Every citation outside the fenced blocks above is written `<§>…`: the
  fixture this page describes lives under `[scan] exclude`, so these IDs
  resolve nowhere from *this* repository and a live marker would be a
  dangling citation under `grund check --full`.)
- **Cross-project citations are explicit.** Every reference that crosses a
  project boundary names the target's alias: `<§>api/FS-session`,
  `<§>root/FS-platform`. There is no `<§>../FS-platform` or implicit
  parent-of-parent lookup.
- **`grund check` orchestrates the whole tree.** At the workspace root, one
  `grund check examples/workspace/repo` validates the root and every member,
  with each member as its own namespace.

## Run it

```bash
grund examples/workspace/repo
echo $?    # 0
```

Then drop a dangling cross-project citation into any file (e.g. change
`<§>api/FS-session` to `<§>api/FS-missing`) and re-run. `grund` reports an
`unknown reference api/FS-missing` at the citation site, with the same
`path:line:` shape it uses for single-project checks.

To see the member-local refusal, keep the original fixture and run:

```bash
(cd examples/workspace/repo/packages/api && grund check)
echo $?    # 1: the member cannot resolve the root-qualified citation
(cd examples/workspace/repo && grund check)
echo $?    # 0: the workspace validates the original cross-project citations
```

The member's `unknown project alias root` finding identifies `root` as the
enclosing workspace's root project, explains the member relationship, and gives
`../..` as the location to run `grund check` from that invocation directory
([§FS-workspace.5.1.1](../../docs/functional-spec/FS-workspace.md#511-verified-enclosing-workspace-guidance)). The hint requires verified membership and alias identity;
it does not promise that a modified target exists when checked from the root.

## Finding and refreshing a stale entry

Run this walkthrough from the grund repository root with `grund` on `PATH`:

```bash
bash examples/workspace/stale-entry.sh
```

The script copies this fixture into a disposable directory under `~/ag/tmp`,
initializes every project twice so all blocks start current regardless of init
ordering, and checks that baseline from the workspace root. It then changes
only the root block's `api` entry from its directory link to the
not-yet-initialized mark an earlier grund wrote. The root check exits 1 and shows:

```text
AGENTS.md:3: error: repo maintenance: stale grund init block: workspace members differ from the workspace (expected - [`api`](packages/api/); found - [`api`](packages/api/) *(not yet initialized)*) — run `grund init` to refresh (does not affect citation validity)
```

`expected` comes from the current render; `found` is the stale body line on
disk. Both workspace sections report the first unequal pair in their existing
order, with `<missing>` for an absent side
([§FS-check.3.5.3.1](../../docs/functional-spec/FS-check.md#3531-first-differing-workspace-entry)).
The script runs `grund init` at the root, as this finding requests, and checks
again successfully. A finding in a member's block instead includes that
member's directory in its refresh command. The copied fixture is removed when
the script exits.

The neighboring-repo form `<§>payments/FS-refunds` resolving to another
repository is **not** implemented yet — see [[§FS-workspace.7](../../docs/functional-spec/FS-workspace.md#7-neighboring-repos)]
(../../docs/functional-spec/FS-workspace.md#7-neighboring-repos).
