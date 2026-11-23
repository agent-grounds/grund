# FS-remote-projects: a project cites another repository's declarations from a committed, pinned projection

A repository may declare another repository as a **remote** and cite its declarations as
`<alias>/<ID>`, the qualified form a workspace member answers ([§FS-workspace.1](FS-workspace.md#1-citation-syntax)). The
remote's bytes are fetched once by the one network verb ([§FS-fetch.remote](FS-fetch.md#remote-remote-projections)), committed
under `.grund/remotes/<alias>/` and pinned in `grund.lock`; every other operation reads them
offline, through the reader that reads a local member. So separate repositories share one
corpus of goals and requirements with a monorepo's citation contract, checked locally and
fast ([§GOAL-small-and-large](../goals.md#goal-small-and-large-start-small-configure-for-big), [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration), [§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible)). The decision is
[§DF-remote-projects](../decisions/functional/DF-remote-projects.md#df-remote-projects-a-remote-project-is-a-workspace-member-whose-bytes-were-fetched-mounted-as-its-own-root); equivalence with a local member is [§REQ-remote-equivalence](../requirements/REQ-remote-equivalence.md#req-remote-equivalence-a-fetched-remote-answers-every-operation-as-a-local-project-does).

The mount layer is built, and an import is read through it ([§FS-imports](FS-imports.md#fs-imports-a-project-cites-a-sibling-project-its-build-places-beside-it-read-live-from-that-tree)): a mounted root
resolves natively and is shown under its alias ([§FS-remote-projects.mount.rule](FS-remote-projects.md#mountrule-a-mounted-root-resolves-natively-and-is-shown-under-its-alias)), and is
walked with its own ignore files alone ([§FS-remote-projects.ignore](FS-remote-projects.md#ignore-ignore-isolation)). Everything else here, a
run started inside a projection included, is specified ahead of its release. A config that
declares `[workspace.remotes]` or `mounted_members` is refused at load, and one that declares
neither reads nothing this page names ([§FS-remote-projects.declaration](FS-remote-projects.md#declaration-declaration-and-registration)).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, home, body, section, catalog),
[§FS-terms.terms.2](FS-terms.md#terms2-citations) (citation, qualified citation), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope, config
root, workspace, member, alias), [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, verdict), and
[§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations) (fetcher, snapshot).

- **remote** — A project another repository holds, declared by a `[workspace.remotes.<alias>]`
  table and read from its projection.
- **consumer** — The project whose config declares a remote.
- **projection** — The remote's complete tracked tree at one commit, written by fetch under
  `.grund/remotes/<alias>/` and committed in the consumer.
- **lock** — `grund.lock`: the committed record of which commit each projection holds and the
  digest of its bytes.
- **mounted root** — A projection, or a member listed in `mounted_members`, read as its own chain
  root and shown outside under the mount's alias.
- **standalone run** — A run of a project from its own config root, with nothing around it.

## declaration: Declaration and registration

A remote is declared in the workspace block, beside `members`:

```toml
[workspace]
members = ["crates/*"]

[workspace.remotes.org]
source = "https://github.com/agent-grounds/org"
ref    = "main"
```

`source` and `ref` are required strings, and any other key in the table is a config error.
`source` is the portable forge source a colleague or CI can fetch from, and a local
filesystem path is refused as a `source` ([§DF-remote-projects.2.3](../decisions/functional/DF-remote-projects.md#23-what-stays-refused); a local clone is read only
through [§FS-fetch.remote.from](FS-fetch.md#remotefrom---from-path-projects-a-committed-revision-of-a-local-clone)); `ref` names a branch, tag or commit at that source.

The table key is the remote's alias, held to [§FS-workspace.3](FS-workspace.md#3-aliases) like a member's, with the block's
mounted members and other remotes among its siblings, and a collision is a config error at
the table line ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)). The remote's own `project_name` names it only inside
its mount ([§FS-remote-projects.mount.rule](FS-remote-projects.md#mountrule-a-mounted-root-resolves-natively-and-is-shown-under-its-alias)).

A directory is a remote because a table registers it, never because it sits under
`.grund/remotes/`: a walk never descends into `.grund/` ([§FS-scan.scope.hidden](FS-scan.md#scopehidden-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent)), and each
`[workspace.remotes]` key claims `.grund/remotes/<key>/` below the config root that declares
it, in member expansion and the ancestor-claim read, at every depth and inside a projection
too ([§FS-workspace.6.1.7.4](FS-workspace.md#6174-the-claim-is-read-from-members-entries-alone)).

A config that declares no `[workspace.remotes]` table never reads `grund.lock` or
`.grund/remotes/`, so an existing file of either name changes no catalog, finding, verdict
or byte of output ([§REQ-backwards-compatibility.1](../requirements/REQ-backwards-compatibility.md#1-what-is-covered)). A grund that does not support remotes
refuses a config declaring `[workspace.remotes]` or `mounted_members` as an unknown config
section or key, exit 2 ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)), rather than check the consumer without its remotes
([§DF-remote-projects.3](../decisions/functional/DF-remote-projects.md#3-consequences)).

## projection: The projection

A projection is the remote's **complete tracked tree** at the locked commit, read from git
objects: its configs, its committed ignore files, its member subtrees, and its own nested
projections with their locks, not the remote's scan set. It is committed in the consumer
beside the lock, so a fresh clone checks offline, and there is no lock-only mode that reads
an uncommitted cache ([§DF-remote-projects.2.3](../decisions/functional/DF-remote-projects.md#23-what-stays-refused)). Fetch writes no `.ignore` beside a projection,
because grund honours `.ignore` and would lose the remote; a search tool excludes the
directory in its own invocation.

The projection format is `projection = 1`, recorded in the lock, and no `scope` narrowing is
admitted ([§DF-remote-projects.2.3](../decisions/functional/DF-remote-projects.md#23-what-stays-refused)): a large remote narrows its own tree upstream.

## lock: The lock

`grund.lock` sits beside the consumer's `grund.toml`, is committed, and holds one table per
remote:

```toml
[remotes.org]
source     = "https://github.com/agent-grounds/org"
ref        = "main"
commit     = "3f1c…"          # full id
projection = 1
digest     = "sha256:…"
```

`source` and `ref` are copied from the declaration at fetch time, `commit` is the full
object id the ref resolved to, `projection` is the format, and `digest` is the projection's
digest ([§FS-remote-projects.bytes](FS-remote-projects.md#bytes-exact-bytes)). The lock is not configuration, and neither `grund config show`
nor `grund config validate` shows it ([§DF-remote-projects.3](../decisions/functional/DF-remote-projects.md#3-consequences)). `check` holds the lock to the
declarations and the projections ([§FS-remote-projects.checks](FS-remote-projects.md#checks-checks)) from the lock and the committed bytes
alone, running no `git` and reaching no network ([§REQ-runs-offline.1](../requirements/REQ-runs-offline.md#1-read-and-verification-paths-execute-nothing), [§FS-fetch.remote.selector](FS-fetch.md#remoteselector---remote-alias-projects-one-declared-remote)).

The lock is written only by `grund fetch --remote`, which owns the lock entry of the remote
it fetches and nothing else in the file. Fetch refuses to replace a `grund.lock` it did not
write, and refuses a `.grund/remotes/<alias>/` that is not a projection it wrote, with the
old bytes left untouched ([§REQ-no-data-loss.2](../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own)).

## mount: The mount

### mount.rule: A mounted root resolves natively and is shown under its alias

A remote's projection, and a member listed in `mounted_members`, is a *mounted root*. Everything written inside it (citations, `[citations]` rule operands, value operands) resolves in the namespace its own standalone run builds. Its `project_name` names its root, its own members and remotes name theirs, and nothing outside the mount is visible from inside. Each identity it resolves to is shown outside under the mount's alias: the native root alias is replaced by the mount alias, and every other alias path gets the mount alias in front. No claim above a mounted root reaches inside it. The literal text of a citation is never rewritten; only fields that name a target carry the mapped identity.

So, mounted as `org`, a remote's own `payments/GOAL-refund` is `org/payments/GOAL-refund`
outside, and its bare `GOAL-x` is `org/GOAL-x`; mounted as `o` while its `project_name` is
`org`, its own citation `org/GOAL-common` is `o/GOAL-common` outside. A projection's own
remotes are mounted inside it the same way, so when `org` and `policy` pin `payments` at two
commits they are two projects, `org/payments` and `policy/payments`: there is no version
solver and no rebinding, which would judge a parent's citations against a commit the parent
never saw. A run or editor session started inside a projection climbs the ancestor claims
only to learn that it is in a mounted root and under which alias, then resolves natively.

### mount.local: A local member opts in with `mounted_members`

`mounted_members` is a list beside `members` and `optional_members` in a `[workspace]`
block. Each entry names a local directory read as a mounted root, so a local twin of a
remote's bytes stays comparable with it ([§REQ-remote-equivalence.4](../requirements/REQ-remote-equivalence.md#4-the-proof-is-two-comparisons-over-an-audited-inventory)), and a monorepo can mount a
checkout of another project and keep that project's own names inside it. A mounted member's alias is
its entry's last path segment, as for an optional member ([§FS-workspace.2.2.2](FS-workspace.md#222-the-alias-of-an-optional-member)), because its
`project_name` names it only inside its mount, and one entry belongs to one list
([§FS-workspace.2.2.7](FS-workspace.md#227-one-entry-belongs-to-one-list)).

## ignore: Ignore isolation

A mounted root is walked with its own committed ignore files, as its own config sets them,
and with none of the consumer's or the machine's: no consumer `.gitignore` or `.ignore`
above it, no `.git/info/exclude`, and no global `core.excludesFile`, so its ignore state is
a committed, portable input that two clones of one commit read alike
([§REQ-deterministic-output.2](../requirements/REQ-deterministic-output.md#2-input-is-the-tree-the-config-and-the-invocation)). Its own ignore files are the ones at and below the mounted
root, whether or not that root is a git checkout: an ignore file of its repository above it
is no more its own than the consumer's is.

## differences: The closed list of differences

A remote answers every operation as a locally mounted twin of its bytes does, except in
these five ways, the closed list of [§REQ-remote-equivalence.2](../requirements/REQ-remote-equivalence.md#2-the-closed-list-of-differences):

1. **Writes into a projection are refused.** A write whose target lies in a projection is a
   named refusal whose remedy names the writable owning remote ([§DF-remote-projects.2.1](../decisions/functional/DF-remote-projects.md#21-the-rulings)):
   `grund fmt --check .grund/remotes/org` reports the projection read-only rather than
   clean, and inside a nested projection the owning boundary wins, so `org/payments` is
   refreshed by updating `org` upstream and refetching. Automatic writer traversals
   (`fmt --write`, `fmt --cross-refs`, `init`, editor edits) edit only writable sites and
   name a read-only exclusion wherever they would otherwise propose a rewrite inside a
   projection. Editors receive provenance and read-only status, and a mutating action into a
   projection is unavailable with the reason given. A dry run reports the same refusals and
   exclusions as write mode, and a write into the consumer's own files computed from remote
   content produces the diff a local twin produces ([§REQ-remote-equivalence.3](../requirements/REQ-remote-equivalence.md#3-writes-are-inside-the-contract)).
2. **Remote-owned content findings are suppressed.** In an automatic consumer run, a content
   finding located inside a projection is suppressed in every channel (text, JSON, editor),
   and a run whose scope names the projection, such as `grund check .grund/remotes/org`,
   reports it. Suppression never removes data: `refs` sites, `cover` totals, `list --size`
   and counts are exactly as for a local project. Nor does it let a consumer citation pass:
   an upstream defect that invalidates a consumer citation, such as a duplicate `ORG-x` the
   consumer cites, fails at the consumer's citation site, and an absent optional member
   inside a remote that leaves a consumer citation unverified is announced once, in the form of [§FS-check.4.9](FS-check.md#49-a-workspace-member-declared-optional-is-absent).
3. **Ignore inputs are isolated**, as [§FS-remote-projects.ignore](FS-remote-projects.md#ignore-ignore-isolation) specifies.
4. **Paths are relocated.** A remote's files are reported under `.grund/remotes/<alias>/`,
   and a rendered cross-ref into a remote links to the projected file by default
   ([§FS-workspace.8.5](FS-workspace.md#85-grund-fmt---cross-refs)). A consumer may opt into an upstream permalink at the locked commit
   as a presentation setting, never one that follows the floating `ref`; a provider grund
   does not know yields no guessed link.
5. **Integrity is verified offline and recursively.** Each projection is verified against
   its lock on every run that reads it, and every nested projection against its own lock,
   by the checks of [§FS-remote-projects.checks](FS-remote-projects.md#checks-checks). A selector that does not name an integrity
   finding does not hide it: `--only dangling` over a modified projection still prints
   `remote-modified`, while `--ignore remote-modified` names it and hides it. This is the
   one stated exception to [§FS-check.2.1.2](FS-check.md#212-selection-filters-the-complete-report), because a filtered report over a namespace
   whose bytes are not the locked ones is not a report about the remote at all. A remote
   that is missing, stale, modified or unloadable yields its one diagnostic, and suppresses
   the unknown-alias finding of [§FS-check.3.8](FS-check.md#38-cross-project-citation-failure) for each consumer citation into it.

## bytes: Exact bytes

A projection is the remote's exact bytes, and its digest is never computed over normalized
content, which would let grund read bytes other than the locked ones while reporting them as
the locked ones ([§DF-remote-projects.2.1](../decisions/functional/DF-remote-projects.md#21-the-rulings)). The digest is the SHA-256 of the projection's entries in
sorted order of their projection-relative, `/`-separated paths, covering for each entry its
path, its entry type and its exact bytes. It covers nothing outside
`.grund/remotes/<alias>/`, and no timestamp, mode or executable bit, which no grund
operation reads and a Windows checkout does not preserve. Fetch refuses a remote tree
containing a symlink or a gitlink, with an error naming the entry.

Fetch never checks the remote out ([§FS-fetch.remote.selector](FS-fetch.md#remoteselector---remote-alias-projects-one-declared-remote)), so nothing converts the
bytes on the way in. On the way out to the consumer's own checkouts, fetch writes
`.grund/remotes/.gitattributes` holding `* -text`, then asks git for each projected path's effective attributes; when an
attribute the remote itself carries, such as a nested `*.md text eol=crlf`, would still
convert a projected file, fetch refuses the materialization and keeps the old projection and
lock. So a `core.autocrlf=true` checkout of a committed projection reads the locked bytes.

## checks: Checks

Each check below is reported as [§FS-check.2.1](FS-check.md#21-report-format) says, except that a selector that does not name it
does not hide it ([§FS-remote-projects.differences](FS-remote-projects.md#differences-the-closed-list-of-differences)), and its row is in [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog).

### checks.remote-missing: A declared remote has no pinned projection

A remote declared with no lock entry is reported at its `[workspace.remotes.<alias>]`
table, and one locked with no projection directory at its lock entry.

### checks.remote-stale: The lock disagrees with the declaration

A lock entry whose `source` or `ref` differs from its declaration is reported at the
changed key of the declaration.

### checks.remote-modified: The projection is not the locked bytes

A projection whose digest ([§FS-remote-projects.bytes](FS-remote-projects.md#bytes-exact-bytes)) is not its lock entry's `digest` is
reported at that `digest`.

### checks.remote-orphan: State no declaration names

A lock entry, or a directory under `.grund/remotes/`, that no declaration of the config
root names is reported where it stands. A config root that declares no remote reads
neither, so once its last declaration is deleted, the leftovers earn no warning
([§FS-remote-projects.declaration](FS-remote-projects.md#declaration-declaration-and-registration), [§DF-remote-projects.2.2](../decisions/functional/DF-remote-projects.md#22-the-choices-the-agora-left-open)).

## failures: Operational failures

Three failures leave the run unable to say what a remote declares. Each exits 2, the
incomplete-run exit of [§FS-check.2.4](FS-check.md#24-an-incomplete-run) ([§DF-remote-projects.2.1](../decisions/functional/DF-remote-projects.md#21-the-rulings)), with a CLI-level error on
stderr ([§FS-errors.2.2](FS-errors.md#22-cli-level-message)), and none is a selectable check code.

### failures.remote-lock-invalid: A lock grund cannot interpret

A `grund.lock` that cannot be parsed, or whose entry cannot be interpreted, fails the run
with `remote-lock-invalid`. No alias is guessed from it; the remedy is to repair or
restore the lock.

### failures.remote-projection-unsupported: A newer projection format

A lock entry whose `projection` format is newer than this grund supports fails the run
with `remote-projection-unsupported`. The remedy is to upgrade grund, never to downgrade
the projection by refetching.

### failures.remote-unreadable: A projected input this grund cannot load

A projected config or required input this grund cannot load fails the run with
`remote-unreadable`, naming the dependency path from the consumer to the input, as an
unloadable member config fails a local run ([§REQ-remote-equivalence.1](../requirements/REQ-remote-equivalence.md#1-the-contract)). The remedy is to
upgrade grund for an unsupported config, or to restore damaged input.
