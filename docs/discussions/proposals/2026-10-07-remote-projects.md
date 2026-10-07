# DISC-remote-projects: A remote project is a workspace member whose bytes were fetched

## 1. Status

Concluded, and decided in [§DF-remote-projects](../../decisions/functional/DF-remote-projects.md#df-remote-projects-a-remote-project-is-a-workspace-member-whose-bytes-were-fetched-mounted-as-its-own-root): its design questions were ruled, and the contract it argues for is specified in [§FS-remote-projects](../../functional-spec/FS-remote-projects.md#fs-remote-projects-a-project-cites-another-repositorys-declarations-from-a-committed-pinned-projection) and [§REQ-remote-equivalence](../../requirements/REQ-remote-equivalence.md#req-remote-equivalence-a-fetched-remote-answers-every-operation-as-a-local-project-does) under `agent-grounds/grund#495`. This proposes the external-repository layer that [§DF-subproject-namespaces.3.4](../../decisions/functional/DF-subproject-namespaces.md#34-external-repos-require-a-separate-offline-cache-design) deferred and [§FS-workspace.7](../../functional-spec/FS-workspace.md#7-neighboring-repos) still describes as missing: `<§>org/ORG-x` resolving to a declaration that lives in another repository, with `grund check` still offline, deterministic and fast.

The requirement it is built around is the owner's: **every operation must work on a fetched project exactly as it works on a local one.** Equivalence is not a list of commands taught to read a cache one by one; it has to hold by construction, or the next command added breaks it.

### 1.1 What accepting it accepts

Accepting this discussion accepts the model of [§DISC-remote-projects.3](2026-10-07-remote-projects.md#3-the-model), the equivalence contract of [§DISC-remote-projects.4](2026-10-07-remote-projects.md#4-equivalence) with its one owner-accepted exception, the diagnostics of [§DISC-remote-projects.5](2026-10-07-remote-projects.md#5-diagnostics) and the byte contract of [§DISC-remote-projects.6](2026-10-07-remote-projects.md#6-bytes). Each piece is then specified in [§FS-remote-projects](../../functional-spec/FS-remote-projects.md#fs-remote-projects-a-project-cites-another-repositorys-declarations-from-a-committed-pinned-projection), with pointers from `FS-workspace` and `FS-fetch`, owes [§REQ-backwards-compatibility](../../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly) on its own terms, and lands as its own change. Nothing here changes a key, a finding or a byte of output today.

### 1.2 How it was argued

A first draft was argued in a local cross-family agora on 2026-10-07 — two participants, one exchange round after the openings, a moderator's verdict both participants endorsed — which found five facts that made the draft wrong as written, agreed sixteen changes, and left two disputes plus one exception for the owner. The owner ruled the same day. As with [§DISC-core-concerns.1.2](2026-09-30-core-concerns.md#12-how-a-ruling-made-off-the-record-is-evidenced), the agora's artifacts stay unpublished, and `agent-grounds/grund#495` restates the design and the rulings in public; this document is the revision written against the verdict and the rulings, and [§DISC-remote-projects.8](2026-10-07-remote-projects.md#8-rulings) records both.

## 2. Context

### 2.1 The need

An organization's common goals, principles and requirements belong in one repository, and every project should cite them: a project `GOAL` that grounds in an organization principle, a `REQ` that refines an organization requirement. Agents working in a project should read the organization's corpus with the commands they use for the project's own — `grund <ID>`, `--toc`, `list`, `refs`, completion, hover — offline, on a fresh clone.

Every project must still remain independent: checkable on a disconnected machine ([§REQ-runs-offline](../../requirements/REQ-runs-offline.md#req-runs-offline-verification-never-depends-on-an-external-service)), and never moved by an upstream change it did not take deliberately.

### 2.2 What exists today

- **Workspace members** ([§FS-workspace](../../functional-spec/FS-workspace.md#fs-workspace-grund-validates-cross-project-citations-in-a-workspace)) already give every operation a cross-project form: resolution ([§FS-workspace.4](../../functional-spec/FS-workspace.md#4-resolution)), value bindings ([§FS-workspace.4.4](../../functional-spec/FS-workspace.md#44-value-bindings)), `grund <alias>/<ID>` ([§FS-workspace.8.1](../../functional-spec/FS-workspace.md#81-grund-aliasid)), `refs` ([§FS-workspace.8.2](../../functional-spec/FS-workspace.md#82-grund-refs)), `list` ([§FS-workspace.8.3](../../functional-spec/FS-workspace.md#83-grund-list)), completions ([§FS-workspace.8.4](../../functional-spec/FS-workspace.md#84-shell-completions)), cross-ref re-derivation ([§FS-workspace.8.5.2](../../functional-spec/FS-workspace.md#852-re-derive-crosses-projects)), `cover` ([§FS-workspace.8.6](../../functional-spec/FS-workspace.md#86-grund-cover)) and nested alias paths ([§FS-workspace.6.1](../../functional-spec/FS-workspace.md#61-nested-workspaces)). But a member must be authored in the tree ([§FS-workspace.2.3](../../functional-spec/FS-workspace.md#23-what-a-member-entry-may-name)), and its alias paths are relative to whichever root is outermost.
- **External facts** ([§FS-fetch](../../functional-spec/FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot), [§DISC-external-facts](2026-09-07-external-facts.md#disc-external-facts-external-facts-are-committed-declarations-materialized-explicitly)) materialize one foreign declaration as committed bytes through the one verb allowed to reach the network — one ID at a time ([§FS-fetch.6](../../functional-spec/FS-fetch.md#6-stability-and-ownership)), with no notion of the project it came from.
- **A submodule as a member** reads equivalently, but fails on a plain clone, needs setup in every worktree, reports the remote's own findings in the consumer's gate, and verifies nothing about what it read.

### 2.3 The principle the existing designs share

[§DISC-external-facts.2](2026-09-07-external-facts.md#2-context): **grund verifies snapshots; integrations own freshness.** [§DF-subproject-namespaces.3.4](../../decisions/functional/DF-subproject-namespaces.md#34-external-repos-require-a-separate-offline-cache-design) asked for "a committed lockfile or cached declaration index"; [§DF-subproject-namespaces.3.5](../../decisions/functional/DF-subproject-namespaces.md#35-network-io-must-be-one-explicit-verb) for one network verb. This proposal keeps all three and adds the step that makes equivalence structural: the snapshot is of the **project**, in the form the one reader already reads.

## 3. The model

### 3.1 One reader

A remote project is committed in the consumer as ordinary files and read by the same reader that reads local members. A rendered declaration index is rejected because it is a second reader that loses locations, bodies, values and citation sites; extended per-ID snapshots because they lose the project and any browsing before citing; a submodule because it fails on a plain clone and verifies nothing.

### 3.2 Declaration and registration

A remote is declared beside the members, in the workspace envelope:

```toml
[workspace]
members = ["crates/*"]

[workspace.remotes.org]
source = "https://github.com/agent-grounds/org"
ref    = "main"
```

The declaration is what makes it a project. A directory placed under `.grund/remotes/` is not a member by being there: today only `members` and `optional_members` register projects, and the ancestor climb reads only those entries. Remote registration therefore extends member expansion and the ancestor-claim read at every depth ([§FS-workspace.6.1.7.4](../../functional-spec/FS-workspace.md#6174-the-claim-is-read-from-members-entries-alone)). A remote's table key shares the sibling alias space of [§FS-workspace.3](../../functional-spec/FS-workspace.md#3-aliases): a collision with another remote, a member or an included root is a located config error.

A grund without remote support refuses a config that declares `[workspace.remotes]` loudly, so adopting the feature cannot silently change what an older grund reports.

In the v2 format of [§DISC-core-concerns](2026-09-30-core-concerns.md#disc-core-concerns-three-concerns-over-two-trees--how-the-configuration-the-core-spec-and-the-engine-are-organized), remotes sit in the envelope beside `members` — the envelope "decides which projects there are" before any concern is read — not under schema. The lock is not configuration.

### 3.3 The projection is the complete tracked tree

`grund fetch --remote org` writes `.grund/remotes/org/` from git objects: the remote's **complete tracked tree** at the locked commit — configs, committed ignore files, member subtrees, and its own nested projections with their locks. Format `projection = 1`.

It is not the remote's default scan set. A scan never descends into hidden directories or into member subtrees, so a scan-set copy would drop the remote's own remotes and members; and an explicit-path read such as `grund cover .grund/remotes/org/extras` must work on a remote exactly as on a local copy, which needs the files a default scan skips. No `scope = [...]` narrowing is admitted in v1: it breaks equivalence by construction, and a large remote should narrow its own tree upstream. A smaller future format must prove it preserves every supported operation.

The projection is committed — there is no lock-only mode — so a fresh clone checks offline. No `.ignore` is written beside it: grund honours `.ignore` too and would lose the remote. Search tools exclude the directory in their own invocation.

### 3.4 Each remote is mounted as its own root

Copying bytes is not enough. A project's names for its own members depend on who is outermost: checked on its own, `org` calls its member `payments`; nested as a member of a consumer, the same bytes are `org/payments`, and `org`'s own text citing `payments/GOAL-refund` no longer resolves.

So a remote is **mounted as its own chain root**. Its citations and its qualified rule and value operands resolve in its own namespace, exactly as its standalone run resolves them, and each resolved identity is exported under the consumer's alias: `org`'s `payments/GOAL-refund` resolves natively and is exported as `org/payments/GOAL-refund`. The consumer chooses the alias; the remote's `project_name` is kept as its native name, so a source whose `project_name = "org"` and whose own text cites `org/GOAL-common` still resolves when the consumer mounts it as `o`. Target-bearing output fields carry the mapping; the literal source text of a citation is unchanged. The remote's own rule objects keep their native meaning.

Nested pins stay isolated: if `org` pins `payments` at A and `policy` pins it at B, they are `org/payments` and `policy/payments`. There is no solver and no rebinding in v1 — rebinding would judge a parent's citations against a commit the parent never saw.

The same mount is available to a local member that opts into it, so that the remote and its local twin stay comparable. Existing members keep today's semantics; mounting is a new, explicit composition mode, not a change to monorepo resolution. A run or editor session started inside a projection uses the ancestor claim only to learn that it is in one and how its exports are prefixed, and still resolves natively.

### 3.5 Ignore isolation

A projection is walked with the remote's own committed ignore files, as its config sets them, and with none of the consumer's or the machine's: no consumer `.gitignore` or `.ignore` above it, no `.git/info/exclude`, no global `core.excludesFile`. Otherwise a consumer's `*.md` line or one developer's global excludes would delete remote declarations from view, and two clones of one commit would disagree. Ignore state remains an input ([§REQ-deterministic-output.2](../../requirements/REQ-deterministic-output.md#2-input-is-the-tree-the-config-and-the-invocation)); for a mounted project it becomes a committed, portable one. A locally mounted member gets the same scan context.

### 3.6 The lock

`grund.lock`, committed beside `grund.toml` and written only by grund:

```toml
[remotes.org]
source     = "https://github.com/agent-grounds/org"
ref        = "main"
commit     = "3f1c…"          # full id
projection = 1
digest     = "sha256:…"
```

The digest covers projection-relative, `/`-separated, sorted paths, entry types and exact bytes ([§DISC-remote-projects.6](2026-10-07-remote-projects.md#6-bytes)). The lock is checked against its declaration in both directions. Fetch validates the whole new projection before replacing the old pair, so a refused fetch leaves projection and lock exactly as they were; and it refuses to overwrite a `.grund/` or `grund.lock` it does not own.

### 3.7 One network verb

```sh
grund fetch --remote org                     # resolve ref, project, rewrite lock — one reviewable diff
grund fetch --remote org --from ../org/main  # project a committed revision of a local clone; dirty tree refused
grund fetch --remote --all                   # every remote this config root declares
grund fetch --remote org --check             # read-only drift: any commit difference, rewinds included
```

`fetch` stays the only command that reaches the network ([§DF-subproject-namespaces.3.5](../../decisions/functional/DF-subproject-namespaces.md#35-network-io-must-be-one-explicit-verb)), and [§REQ-runs-offline.2](../../requirements/REQ-runs-offline.md#2-materialization-is-explicit) is extended to name the selector. The committed `source` is always the portable forge source; a local clone is used only through `--from`, so colleagues and CI can reproduce the pin. `--check` never runs inside `grund check`; a ref it cannot obtain is an operational failure, not a clean result. `--all` updates only this config root's own declarations, never dependencies owned inside a parent's snapshot. No upstream script, checker or `fetch` integration runs during extraction.

Fetch refuses to lock a commit whose projected config or required content this grund cannot load — installing it would only defer the failure. It locks, and reports, a commit whose own content check has findings: gating a pin on upstream housekeeping is the coupling this design exists to avoid.

## 4. Equivalence

### 4.1 The contract

Proposed as a new requirement, `REQ-remote-equivalence`:

> For any project P and any project O, every operation `grund` offers — `check`'s findings about P's sites, `grund <ID>` in every form, `list`, `refs`, `cover`, value bindings, rule subjects and objects, completion, hover, definition, references, cross-ref rendering, and every write into P's own files computed from O's content — produces the same result when O is a fetched remote of P as when O's bytes are a locally mounted member of P under the same alias, and the same graph O's standalone run produces, except as [§DISC-remote-projects.4.2](2026-10-07-remote-projects.md#42-the-closed-list-of-differences) lists.

Writes are inside the contract: re-deriving a link after an upstream rename, normalizing a citation and inserting a completion produce the same diff against a remote as against a local mounted member.

### 4.2 The closed list of differences

1. **Writes into a projection are refused** — the one exception to the owner's requirement, accepted by the owner ([§DISC-remote-projects.8.1](2026-10-07-remote-projects.md#81-the-write-exception)). A write whose target lies in a projection is a named refusal whose remedy names the writable owning remote: `fetch org/TICKET-1` against a projection refuses before running anything, `fmt --check .grund/remotes/org` reports read-only rather than clean, and inside a nested snapshot the owning boundary wins — `org/payments` is refreshed by updating `org` upstream and refetching. Automatic writer traversals edit only writable sites and name a read-only exclusion wherever a projected rewrite would otherwise be proposed; a dry run reports the same exclusions and refusals as write mode. Editors receive provenance and read-only status, and mutating actions into projections are unavailable with the reason given.
2. **Remote-owned content findings are suppressed** in every channel of an automatic consumer run — text, JSON, editor — and an explicitly scoped run or audit reports them. Suppression never removes data: `refs` sites, `cover` totals, `list --size` and counts are exactly as for a local project. No aggregate count line is printed.
3. **Ignore inputs** follow [§DISC-remote-projects.3.5](2026-10-07-remote-projects.md#35-ignore-isolation).
4. **Paths are relocated** under `.grund/remotes/<alias>/`. A rendered cross-ref links to the projected file by default; a consumer may opt into an upstream permalink at the locked commit — never one that follows the floating `ref` — as a presentation setting, and an unsupported provider yields no guessed link.
5. **Remote integrity is verified offline and recursively** ([§DISC-remote-projects.5](2026-10-07-remote-projects.md#5-diagnostics)).

Two rules keep suppression from letting a consumer citation pass: an upstream defect that invalidates a consumer citation — a duplicate `ORG-x` the consumer cites — fails at the consumer's site; and an absent optional member inside a remote that leaves a consumer citation unverified is announced once (the form of [§FS-check.4.9](../../functional-spec/FS-check.md#49-a-workspace-member-declared-optional-is-absent)).

### 4.3 The proof

Two comparisons, both required, over an audited inventory of CLI, library and editor operations including writer diffs, preview/write parity and refusals:

1. **Source against projection:** O's standalone run against O as a remote of a consumer — catches lost inputs and namespace edges, which a comparison of two copies cannot, since both lose the same thing.
2. **Projection against local twin:** O as a remote against O's bytes as a locally mounted member — catches any special reading path.

Only declared relocation fields are normalized, never arbitrary text, and the source run uses the committed portable scan context of [§DISC-remote-projects.3.5](2026-10-07-remote-projects.md#35-ignore-isolation). A command added later is covered by entering the inventory, and an omission is visible there.

## 5. Diagnostics

`check` reads the lock and the projection and never the network. Every diagnostic applies recursively to nested remotes, stays visible under content selectors, and an unavailable remote suppresses the cascade of unknown-alias findings into it.

| Code | Class | When | Remedy |
|---|---|---|---|
| `remote-missing` | error, exit 1 | declared with no lock entry (sited at the table), or locked with no projection (sited at the lock entry) | fetch |
| `remote-stale` | error, exit 1 | the lock's `source` or `ref` differs from the declaration; sited at the changed key | refetch, or restore the declaration |
| `remote-modified` | error, exit 1 | the projection's digest is not the lock's; sited at the digest | restore from version control, or refetch |
| `remote-orphan` | warning | a lock entry or a `.grund/remotes/` directory no declaration names | remove the stale state or restore the declaration; never fetch |
| `remote-lock-invalid` | operational, exit 2 | a lock grund cannot parse or interpret | repair or restore the lock; no alias is guessed |
| `remote-projection-unsupported` | operational, exit 2 | a `projection` format newer than this grund | upgrade grund; never downgrade by refetching |
| `remote-unreadable` | operational, exit 2 | a projected config or required input this grund cannot load, with its dependency path | upgrade grund for an unsupported config; restore damaged input |

Exit 2 keeps its meaning — do not trust this report as complete ([§FS-check.2.4](../../functional-spec/FS-check.md#24-an-incomplete-run)) — so a namespace that was never established cannot yield a clean filtered report. An unloadable projected config is the same failure an unloadable member config is today, which is equivalence rather than an exception. Code identity is kept distinct from CLI rendering; operational failures keep today's stderr shape.

## 6. Bytes

The projection is **exact bytes**. The digest is never computed over normalized content: hashing normalized line endings would let grund read bytes other than the locked ones while reporting them as the locked ones.

So checkout transforms must be prevented, not forgiven. Fetch uses a protection verified effective for the tree it installs — including against `.gitattributes` the remote itself carries, which can override an outer `.grund/.gitattributes` with `-text` (a nested `*.md text eol=crlf` does) — or refuses the materialization and keeps the old pair. The mechanism is settled by a real Windows `core.autocrlf=true` checkout of a projection that carries its own attributes.

In v1 fetch refuses a remote tree containing symlinks or gitlinks, with a named error; they can later be admitted as recorded inventory entries without changing the contract. The digest covers entry types but not the executable bit, which no grund operation reads and which Windows checkouts do not preserve.

## 7. Determinism and what is refused

A projection is a function of the commit and the projection format; the lock records both, and the ignore inputs are committed. Two installs of one grund version over one committed tree read the same bytes and agree ([§FS-non-goals.13](../../functional-spec/FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)). Different grund versions may read one projection differently exactly as they may read the consumer's own files, and [§REQ-backwards-compatibility](../../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly) governs both. A repository with no remotes is unchanged: one that already scans `.grund/notes.md` keeps its catalog and verdict.

Refused for those reasons: a live override pointing a remote at a working tree without fetching, a committed local `source` path, and a lock-only mode reading an uncommitted cache.

## 8. Rulings

### 8.1 The write exception

Accepted. A projection is read-only, and the refusal of [§DISC-remote-projects.4.2](2026-10-07-remote-projects.md#42-the-closed-list-of-differences) item 1 is the one deliberate exception to "all operations, equivalently". Read-only, locked snapshots are the point of the design: editing a fetched project in place would make the committed bytes disagree with the commit the lock names.

### 8.2 Lock-side outcomes

An orphan lock entry is the warning `remote-orphan`: it breaks nothing the consumer reads and cannot be repaired by fetching, so it is stale state like an orphan directory. A lock grund cannot interpret is the operational `remote-lock-invalid`, exit 2, and the unsupported-format and unreadable-config cases carry their own codes, because exit 2 already means the report cannot be trusted as complete.

### 8.3 Exact bytes, and a small v1

Exact bytes with verified protection or refusal, never a normalized digest, because the verbatim promise is what equivalence rests on. Symlinks and gitlinks are refused in v1 and the executable bit is left out, because it keeps v1 portable and small and can be widened later without changing the contract.

## 9. The motivating use

The organization's corpus lives in its own small repository with an `ORG` kind. Each of grund, rhei, ephor and fissile declares `[workspace.remotes.org]`, fetches once, and cites `<§>org/ORG-…` from its goals and requirements; a root rule `[citations.GOAL] must = ["org/ORG"]` may require it, and is not satisfied by a local ORG-shaped declaration ([§FS-config.3.9.3](../../functional-spec/FS-config.md#393-alias-matching)). Each stays on its own pin — rhei at A while fissile moves to B — and checks offline with its own grund. A scheduled `grund fetch --remote --all --check` turns upstream movement into a matter for the project rather than a surprise in its gate.

One coupling is recorded for the organization's conventions: if it adopts a config feature a consumer's pinned grund cannot read, that consumer keeps checking on its old pin but cannot move to the new one until it upgrades grund.
