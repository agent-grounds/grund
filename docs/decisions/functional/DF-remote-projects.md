# DF-remote-projects: a remote project is a workspace member whose bytes were fetched, mounted as its own root

**Status:** Accepted
**Date:** 2026-10-07

## 1. Context

An organization's shared goals, principles and requirements belong in one repository, and
every project should cite them as `<alias>/<ID>` while staying checkable on a disconnected
machine. [§DF-subproject-namespaces.3.4](DF-subproject-namespaces.md#34-external-repos-require-a-separate-offline-cache-design) deferred external repositories until "a committed
lockfile or cached declaration index" kept `grund check` offline, and [§FS-workspace.7](../../functional-spec/FS-workspace.md#7-neighboring-repos)
described them as missing. [§DISC-remote-projects](../../discussions/proposals/2026-10-07-remote-projects.md#disc-remote-projects-a-remote-project-is-a-workspace-member-whose-bytes-were-fetched) argued the design in a cross-family agora
on 2026-10-07; the owner ruled its three disputes the same day, and approved the plan that
lands it as four pieces under agent-grounds/grund#495.

## 2. Decision

A remote is a workspace member whose bytes were fetched, specified in
[§FS-remote-projects](../../functional-spec/FS-remote-projects.md#fs-remote-projects-a-project-cites-another-repositorys-declarations-from-a-committed-pinned-projection):

- it is declared as `[workspace.remotes.<alias>]` with `source` and `ref`
  ([§FS-remote-projects.declaration.table](../../functional-spec/FS-remote-projects.md#declarationtable-the-remote-table));
- `grund fetch --remote <alias>` writes its complete tracked tree, read from git objects,
  to a committed `.grund/remotes/<alias>/` ([§FS-fetch.remote.selector](../../functional-spec/FS-fetch.md#remoteselector---remote-alias-projects-one-declared-remote),
  [§FS-remote-projects.projection.tree](../../functional-spec/FS-remote-projects.md#projectiontree-the-complete-tracked-tree)), and pins the commit and a digest of the exact
  bytes in `grund.lock` ([§FS-remote-projects.lock.keys](../../functional-spec/FS-remote-projects.md#lockkeys-one-table-per-remote));
- it is mounted as its own chain root, resolving natively and shown outside under the
  consumer's alias, and a local member may opt into the same mount with `mounted_members`
  ([§FS-remote-projects.mount.rule](../../functional-spec/FS-remote-projects.md#mountrule-a-mounted-root-resolves-natively-and-is-shown-under-its-alias), [§FS-remote-projects.mount.local](../../functional-spec/FS-remote-projects.md#mountlocal-a-local-member-opts-in-with-mounted_members));
- it is walked with its own committed ignore files only ([§FS-remote-projects.ignore](../../functional-spec/FS-remote-projects.md#ignore-ignore-isolation));
- it answers every operation as a locally mounted twin does, except as five listed
  differences say ([§FS-remote-projects.differences](../../functional-spec/FS-remote-projects.md#differences-the-closed-list-of-differences)).

### 2.1 The rulings

- **The write exception (V-A08): accepted.** A projection is read-only, and a write whose
  target lies in one is a named refusal whose remedy names the writable owning remote
  ([§FS-remote-projects.differences.writes](../../functional-spec/FS-remote-projects.md#differenceswrites-writes-into-a-projection-are-refused)). It is the one deliberate exception to "every
  operation, equivalently": editing a fetched project in place would make the committed
  bytes disagree with the commit the lock names.
- **Lock-side outcomes (V-D01).** An orphan lock entry is the warning `remote-orphan`,
  whose remedy never suggests fetching, because it breaks nothing the consumer reads and
  fetching cannot repair it ([§FS-remote-projects.checks.remote-orphan](../../functional-spec/FS-remote-projects.md#checksremote-orphan-state-no-declaration-names)). A lock grund cannot
  interpret is `remote-lock-invalid`, and an unsupported projection format and an
  unreadable projected config carry their own codes; all three exit 2, because exit 2
  already means the report cannot be trusted as complete
  ([§FS-remote-projects.failures](../../functional-spec/FS-remote-projects.md#failures-operational-failures)).
- **Digest coverage and checkout protection (V-D02).** The digest is over exact bytes,
  never normalized content, and fetch uses a checkout protection verified effective,
  including against `.gitattributes` the remote itself carries, or refuses and keeps the old
  pair ([§FS-remote-projects.bytes.digest](../../functional-spec/FS-remote-projects.md#bytesdigest-the-digest-is-over-exact-bytes), [§FS-remote-projects.bytes.protection](../../functional-spec/FS-remote-projects.md#bytesprotection-checkout-transforms-are-prevented-or-the-fetch-is-refused)). v1 refuses
  symlinks and gitlinks and leaves the executable bit out of the digest
  ([§FS-remote-projects.bytes.entries](../../functional-spec/FS-remote-projects.md#bytesentries-regular-files-only-and-no-executable-bit)).

### 2.2 The choices the agora left open

The approved plan took four further choices as proposed:

1. A config that declares no `[workspace.remotes]` table never reads `grund.lock` or
   `.grund/remotes/`, so leftovers after the last declaration is deleted earn no
   `remote-orphan` warning; the alternative puts a new warning into repositories that never
   opted in ([§FS-remote-projects.declaration.none](../../functional-spec/FS-remote-projects.md#declarationnone-a-repository-with-no-remotes-reads-nothing-new)).
2. Fetch prints the projected paths the consumer's git would not commit, with the exact
   `git add --force` that commits them, and never touches the index
   ([§FS-fetch.remote.commit](../../functional-spec/FS-fetch.md#remotecommit-paths-the-consumers-git-would-not-commit-are-named)).
3. `fetch --remote --check` exits 0 when current, 1 on any difference, and 2 when the ref
   cannot be obtained ([§FS-fetch.remote.check](../../functional-spec/FS-fetch.md#remotecheck---check-is-a-read-only-drift-signal)).
4. The managed `AGENTS.md` block is not changed by any piece: changing its template flags
   every repository's block as stale, so teaching remotes there is a later change through
   the block's own ramp.

### 2.3 What stays refused

- **A live working-tree override**, pointing a remote at a checkout without fetching: two
  machines would read different bytes under one lock.
- **A committed local `source` path**: colleagues and CI could not reproduce the pin, so a
  local clone is used only through `fetch --remote --from` ([§FS-fetch.remote.from](../../functional-spec/FS-fetch.md#remotefrom---from-path-projects-a-committed-revision-of-a-local-clone)).
- **A lock-only mode** reading an uncommitted cache: a fresh clone would not check offline
  ([§FS-remote-projects.projection.committed](../../functional-spec/FS-remote-projects.md#projectioncommitted-committed-with-no-lock-only-mode)).
- **`scope` narrowing** of a projection: a partial tree breaks equivalence by construction
  ([§FS-remote-projects.projection.format](../../functional-spec/FS-remote-projects.md#projectionformat-format-1-with-no-narrowing)).

## 3. Consequences

- The contract lands first, with no behaviour: this decision, [§FS-remote-projects](../../functional-spec/FS-remote-projects.md#fs-remote-projects-a-project-cites-another-repositorys-declarations-from-a-committed-pinned-projection),
  [§REQ-remote-equivalence](../../requirements/REQ-remote-equivalence.md#req-remote-equivalence-a-fetched-remote-answers-every-operation-as-a-local-project-does), the `--remote` chapter of [§FS-fetch](../../functional-spec/FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot), and the rewritten
  [§REQ-runs-offline.2](../../requirements/REQ-runs-offline.md#2-materialization-is-explicit) and [§REQ-runs-offline.3](../../requirements/REQ-runs-offline.md#3-no-implicit-freshness). Three pieces build it, each blocked by the
  one before: **Mount** (`mounted_members`, ignore isolation, the operation inventory and
  the standalone-against-mounted comparison), **Remotes** (the table, the lock, `fetch
  --remote` and `--from`, the projection writer, exact bytes with a Windows
  `core.autocrlf=true` test, the `remote-*` diagnostics, refused writes, suppression, and
  both remaining comparisons), and **Drift** (`--all`, `--check`, the upstream permalink).
- [§FS-workspace](../../functional-spec/FS-workspace.md#fs-workspace-grund-validates-cross-project-citations-in-a-workspace) keeps ordinary members on the outermost claim; a mounted root is the one
  exception ([§FS-workspace.6.1.6](../../functional-spec/FS-workspace.md#616-a-path-is-read-from-the-outermost-claim)).
- A repository that declares nothing sees no new read, finding or byte
  ([§FS-remote-projects.declaration.none](../../functional-spec/FS-remote-projects.md#declarationnone-a-repository-with-no-remotes-reads-nothing-new)), and an older grund refuses a config that adopts
  the feature loudly ([§FS-remote-projects.declaration.older](../../functional-spec/FS-remote-projects.md#declarationolder-an-older-grund-refuses-the-table-loudly)), so no verdict moves quietly
  and no release note is owed.
- The v2 configuration carries `remotes` and `mounted_members` in its envelope beside
  `members`, unchanged; the lock is not configuration.

## 4. Alternatives considered

| Approach | Why rejected |
|---|---|
| A rendered declaration index of the remote | A second reader that loses locations, bodies, values and citation sites. |
| Extended per-ID snapshots through `fetch <ID>` | Loses the project, its namespace, and any browsing before citing. |
| A git submodule as a member | Fails on a plain clone, needs setup in every worktree, reports the remote's findings in the consumer's gate, and verifies nothing it read. |
| Nesting the remote as an ordinary member | Its own names for its members change with who is outermost, so its own citations stop resolving. |
| Writing each piece's spec with its code | The contract would be reviewed four times, and the rulings would have no public home until the last piece. |
