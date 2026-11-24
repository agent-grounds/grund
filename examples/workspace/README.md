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

From this example's `repo/` directory, try `grund FS-session --brief`. The
root has no such declaration, so the refusal suggests `api/FS-session` and
`web/FS-session`; `grund api/FS-session --brief` reads the selected member's
declaration. The same suggestion rule applies when the root config does not
even declare the member's kind ([§FS-workspace.8.1.1.4](../../docs/functional-spec/FS-workspace.md#8114-each-project-is-asked-in-its-own-grammar)).

Refs keeps its root-local meaning: here `grund refs FS-session` succeeds with
an empty answer because `FS` is valid at the root. If a kind exists only in a
member, an unqualified refs query instead keeps `invalid ID` and offers the
qualified `grund refs <alias>/<ID>` command as its text hint; JSON stays
`invalid-id` ([§FS-workspace.8.2.2.1](../../docs/functional-spec/FS-workspace.md#8221-a-member-only-kind-replaces-the-text-format-hint)).

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

## Repairing a missing alias

Run from the grund repository root with the updated `grund` on `PATH`:

```bash
bash examples/workspace/missing-alias.sh
```

The script copies the valid fixture into a disposable directory under `~/ag/tmp`,
checks the baseline, and changes only `<§>api/FS-session` to `<§>FS-session` in
the root document. Check still exits 1, now naming the actual declarations:

```text
docs/functional-spec/FS-platform.md:3: error: unknown reference FS-session; did you mean api/FS-session or web/FS-session?
```

Choose the intended project: the script restores `<§>api/FS-session`, checks
successfully and prints `check after repair: exit 0`. It asserts the diagnostic
and exit statuses, then removes its copy. The checked-in fixture and its empty
expected stdout stay valid. Its existing v1 migration cautions still appear on
stderr.

The suggestion uses only loaded projects and takes precedence over local typo
guesses; it does not make the unqualified citation resolve. A member-local run
cannot suggest unseen siblings. Unlike show's member-only-kind query above,
check recognizes only the citing project's configured grammar
([§FS-check.3.1.5](../../docs/functional-spec/FS-check.md#315-an-unqualified-id-declared-in-a-loaded-member)).

## Repairing an unlisted project

Run this walkthrough from the grund repository root with the updated `grund`
on `PATH`:

```bash
bash examples/workspace/unlisted-project.sh
```

The script builds a disposable v2 workspace under `~/ag/tmp`: `shop` lists
`packages/billing`, while `packages/notify` has its own config but no membership
entry. The first check exits 1 with an existing misplaced-declaration error:

```text
packages/notify/requirements.md:3: error: FS-email must be declared in requirements.md (single-file kind) — packages/notify/grund.toml (project notify) is not read: no [workspace] members entry lists packages/notify; kind homes from grund.toml (project shop) apply; add "packages/notify" to [workspace] members in grund.toml, or exclude "packages/notify/" with [schema.sources] exclude in grund.toml
```

The home in that message is shop's root-level file. The script adds only notify's
membership and checks successfully, keeping the declaration at heading level 2.
It then restores the unlisted configuration and demonstrates the exclusion
alternative. A nested config alone does not change ownership, and promoting the
declaration's heading does not repair it
([§FS-declarations.checks.misplaced-declaration](../../docs/functional-spec/FS-declarations.md#checksmisplaced-declaration-misplaced-declaration-configured-kind-home)).
In v1 use `[scan] exclude = ["notify"]`, excluding every directory named `notify`
at every depth. In v2 use `[schema.sources] exclude = ["packages/notify/"]` to
exclude this path.
If a source include or kind home makes notify or a descendant a scan root,
remove that configured root too, since exclusion cannot prune a scan root.
The script asserts the diagnostic and exit statuses and removes its fixture
when it exits.

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
AGENTS.md:3: error: repo maintenance: stale grund init block: the workspace members section differs from the workspace (expected - [`api`](packages/api/); found - [`api`](packages/api/) *(not yet initialized)*) — run `grund init` to refresh (does not affect citation validity)
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
