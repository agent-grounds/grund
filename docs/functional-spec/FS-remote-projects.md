# FS-remote-projects: a project cites another repository's declarations from a committed, pinned projection

A repository may declare another repository as a **remote** and cite its declarations as
`<alias>/<ID>`, the same qualified form a workspace member already answers
([§FS-workspace.1](FS-workspace.md#1-citation-syntax)). The remote's bytes are fetched once by the one network verb
([§FS-fetch.remote](FS-fetch.md#remote-remote-projections)), committed under `.grund/remotes/<alias>/`, and pinned in `grund.lock`;
from then on every operation reads them offline, through the same reader that reads a
local member. That lets an organization of separate repositories share one corpus of
goals and requirements with the citation contract a monorepo has ([§GOAL-small-and-large](../goals.md#goal-small-and-large-start-small-configure-for-big)),
keeps every cross-repository citation a checked one ([§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration)), and keeps
`grund check` local and fast ([§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible)). The decision is
[§DF-remote-projects](../decisions/functional/DF-remote-projects.md#df-remote-projects-a-remote-project-is-a-workspace-member-whose-bytes-were-fetched-mounted-as-its-own-root); equivalence with a local member is [§REQ-remote-equivalence](../requirements/REQ-remote-equivalence.md#req-remote-equivalence-a-fetched-remote-answers-every-operation-as-a-local-project-does).

Every behaviour below is specified ahead of the releases that build it. Until those land,
a config that declares `[workspace.remotes]` or `mounted_members` is refused at load
([§FS-remote-projects.declaration.older](FS-remote-projects.md#declarationolder-an-older-grund-refuses-the-table-loudly)), and a repository that declares neither reads
nothing this spec names ([§FS-remote-projects.declaration.none](FS-remote-projects.md#declarationnone-a-repository-with-no-remotes-reads-nothing-new)).

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

### declaration.table: The remote table

A remote is declared in the workspace block, beside `members`:

```toml
[workspace]
members = ["crates/*"]

[workspace.remotes.org]
source = "https://github.com/agent-grounds/org"
ref    = "main"
```

`source` and `ref` are both required strings. `source` is the portable forge source a
colleague or CI can fetch from; a local filesystem path is refused as a `source`, because
a pin nobody else can reproduce is not a pin (a local clone is used only through
[§FS-fetch.remote.from](FS-fetch.md#remotefrom---from-path-projects-a-committed-revision-of-a-local-clone)). `ref` names a branch, tag or commit at that source. Any other key
in the table is a config error. `grund config show` renders the table and `grund config
validate` validates it like any other workspace key; the lock is not configuration and
neither command shows it.

### declaration.alias: The alias is the table key

A remote's alias is its table key and must match the alias grammar of [§FS-workspace.1.1](FS-workspace.md#11-the-alias-path).
It shares the sibling alias level of [§FS-workspace.3](FS-workspace.md#3-aliases) with the members, the included root,
the mounted members and the other remotes of the same block: a collision with any of them
is a located config error, because a qualified citation would otherwise have two targets.
The remote's own `project_name` is its native name, used only inside its mount
([§FS-remote-projects.mount.rule](FS-remote-projects.md#mountrule-a-mounted-root-resolves-natively-and-is-shown-under-its-alias)).

### declaration.registration: Only a declaration registers a remote

A directory is a remote because a table registers it, never because it sits under
`.grund/remotes/`. Hidden-path pruning is unchanged: a walk never descends into `.grund/`,
and a projection is read because its table names it. Registration extends member expansion
and the ancestor-claim read at every depth: each `[workspace.remotes]` key claims
`.grund/remotes/<key>/` below the config root that declares it ([§FS-workspace.6.1.7.4](FS-workspace.md#6174-the-claim-is-read-from-members-entries-alone)),
including inside a projection, whose own remotes are its nested projections.

### declaration.none: A repository with no remotes reads nothing new

A config that declares no `[workspace.remotes]` table never reads `grund.lock` or
`.grund/remotes/`. An existing file of either name changes no catalog, finding, verdict or
byte of output: a `.grund/remotes/x/` holding declarations stays unread, an unrelated
`grund.lock` stays unparsed, and a `.grund/notes.md` the config scans explicitly is read
exactly as before. The price is accepted: a consumer that deletes its last declaration gets
no `remote-orphan` warning for the leftovers, rather than every repository that never
opted in being exposed to a new warning ([§REQ-backwards-compatibility.1](../requirements/REQ-backwards-compatibility.md#1-what-is-covered)).

### declaration.older: An older grund refuses the table loudly

A grund that does not support remotes refuses a config declaring `[workspace.remotes]` or
`mounted_members` as an unknown config section or key, with exit 2. Adopting the feature
therefore cannot silently change what an older grund reports: it stops rather than checks
the consumer without its remotes.

## projection: The projection

### projection.tree: The complete tracked tree

A projection is the remote's **complete tracked tree** at the locked commit, read from git
objects: its configs, its committed ignore files, its member subtrees, and its own nested
projections with their locks. It is not the remote's default scan set: a scan never
descends into hidden directories or member subtrees, so a scan-set copy would drop the
remote's own remotes and members, and an explicit-path read such as `grund cover
.grund/remotes/org/extras` must find the same files it would find in the remote itself.

### projection.committed: Committed, with no lock-only mode

The projection is committed in the consumer beside the lock, so a fresh clone checks
offline. There is no lock-only mode that reads an uncommitted cache. Fetch writes no
`.ignore` beside a projection, because grund honours `.ignore` and would lose the remote;
a search tool excludes the directory in its own invocation.

### projection.format: Format 1, with no narrowing

The projection format is `projection = 1`, recorded in the lock. No `scope` narrowing is
admitted: a partial tree breaks equivalence by construction, and a large remote narrows
its own tree upstream. A later, smaller format is admitted only once it proves it
preserves every supported operation, and it carries a new format number.

## lock: The lock

### lock.keys: One table per remote

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
object id the ref resolved to, `projection` is the format, and `digest` is the
SHA-256 of the projection ([§FS-remote-projects.lock.digest](FS-remote-projects.md#lockdigest-what-the-digest-covers)).

### lock.digest: What the digest covers

The digest covers the projection's entries in sorted order of their projection-relative,
`/`-separated paths, and for each entry its path, its entry type and its exact bytes
([§FS-remote-projects.bytes.digest](FS-remote-projects.md#bytesdigest-the-digest-is-over-exact-bytes)). It covers nothing outside `.grund/remotes/<alias>/`,
and no timestamp, mode or executable bit ([§FS-remote-projects.bytes.entries](FS-remote-projects.md#bytesentries-regular-files-only-and-no-executable-bit)).

### lock.agreement: Checked against the declaration both ways

`check` holds the lock and the declarations to each other in both directions: a declared
remote with no lock entry, or with a lock entry whose `source` or `ref` differs, is a
finding of [§FS-remote-projects.checks](FS-remote-projects.md#checks-checks), and so is a lock entry no declaration names. The
check reads only the lock and the committed bytes; it never runs `git` and never reaches
the network ([§REQ-runs-offline.1](../requirements/REQ-runs-offline.md#1-read-and-verification-paths-execute-nothing)).

### lock.ownership: The lock is grund's

The lock is written only by `grund fetch --remote`, which owns the lock entry of the
remote it fetches and nothing else in the file. Fetch refuses to replace a `grund.lock` it
did not write, and refuses a `.grund/remotes/<alias>/` that is not a projection it wrote,
with the old bytes left untouched ([§REQ-no-data-loss.2](../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own)).

## mount: The mount

### mount.rule: A mounted root resolves natively and is shown under its alias

A remote's projection, and a member listed in `mounted_members`, is a *mounted root*. Everything written inside it (citations, `[citations]` rule operands, value operands) resolves in the namespace its own standalone run builds. Its `project_name` names its root, its own members and remotes name theirs, and nothing outside the mount is visible from inside. Each identity it resolves to is shown outside under the mount's alias: the native root alias is replaced by the mount alias, and every other alias path gets the mount alias in front. No claim above a mounted root reaches inside it. The literal text of a citation is never rewritten; only fields that name a target carry the mapped identity.

So, mounted as `org`, a remote's own `payments/GOAL-refund` is `org/payments/GOAL-refund`
outside, and its bare `GOAL-x` is `org/GOAL-x`. Mounted as `o` while its `project_name`
is `org`, its own citation `org/GOAL-common` is `o/GOAL-common` outside. A consumer that
also has a `payments` member of its own still means that member by `payments/…`. The
remote's rule objects keep their native meaning.

### mount.local: A local member opts in with `mounted_members`

`mounted_members` is a list beside `members` and `optional_members` in a `[workspace]`
block. Each entry names a local directory that is read as a mounted root, so that a remote
and a local twin of the same bytes stay comparable, and so that a monorepo can mount a
checkout of another project and keep that project's own names. A mounted member's alias is
its entry's last path segment, as for an optional member ([§FS-workspace.2.2.2](FS-workspace.md#222-the-alias-of-an-optional-member)), because
its `project_name` is its native name rather than the consumer's choice. One entry belongs
to one list ([§FS-workspace.2.2.7](FS-workspace.md#227-one-entry-belongs-to-one-list)). Members in `members` and `optional_members` keep
today's semantics; mounting is opt-in per entry.

### mount.nested: Nested pins stay isolated

A projection's own remotes are mounted inside it under its alias. If `org` pins `payments`
at one commit and `policy` pins it at another, they are two projects, `org/payments` and
`policy/payments`. There is no version solver and no rebinding: rebinding would judge a
parent's citations against a commit the parent never saw.

### mount.inside: A run started inside a projection

A run or editor session started inside a projection climbs the ancestor claims only to
learn that it is in a mounted root and under which alias its identities are shown, then
resolves natively, as the mount rule says.

## ignore: Ignore isolation

A mounted root is walked with its own committed ignore files, as its own config sets them,
and with none of the consumer's or the machine's: no consumer `.gitignore` or `.ignore`
above it, no `.git/info/exclude`, and no global `core.excludesFile`. A consumer's `*.md`
line or one developer's global excludes would otherwise delete the remote's declarations
from view, and two clones of one commit would disagree. A mounted root's ignore state is
therefore a committed, portable input ([§REQ-deterministic-output.2](../requirements/REQ-deterministic-output.md#2-input-is-the-tree-the-config-and-the-invocation)).

## differences: The closed list of differences

A remote answers every operation as a locally mounted twin of its bytes does
([§REQ-remote-equivalence](../requirements/REQ-remote-equivalence.md#req-remote-equivalence-a-fetched-remote-answers-every-operation-as-a-local-project-does)). These five differences are the whole list.

### differences.writes: Writes into a projection are refused

A write whose target lies in a projection is a named refusal whose remedy names the
writable owning remote. `grund fetch org/TICKET-1` refuses before its integration runs;
`grund fmt --check .grund/remotes/org` reports the projection read-only rather than clean;
inside a nested projection the owning boundary wins, so `org/payments` is refreshed by
updating `org` upstream and refetching. Automatic writer traversals (`fmt --write`,
`fmt --cross-refs`, `init`, editor edits) edit only writable sites and name a read-only
exclusion wherever they would otherwise propose a rewrite inside a projection; a dry run
reports the same exclusions and refusals as write mode. Editors receive provenance and
read-only status, and a mutating action into a projection is unavailable with the reason
given. A rewrite of a consumer file computed from remote content is not a write into a
projection and is identical to the local-twin case.

### differences.suppression: Remote-owned content findings are suppressed

In an automatic consumer run, a content finding located inside a projection is suppressed
in every channel (text, JSON, editor). A run whose scope names the projection, such as
`grund check .grund/remotes/org`, reports them. Suppression never removes data: `refs`
sites, `cover` totals, `list --size` and counts are exactly as for a local project, and no
aggregate count of suppressed findings is printed.

### differences.ignore: Ignore inputs are isolated

A mounted root's ignore inputs are its own, as [§FS-remote-projects.ignore](FS-remote-projects.md#ignore-ignore-isolation) specifies.

### differences.paths: Paths are relocated

A remote's files are reported under `.grund/remotes/<alias>/`. A rendered cross-ref into a
remote links to the projected file by default ([§FS-workspace.8.5](FS-workspace.md#85-grund-fmt---cross-refs)). A consumer may opt into
an upstream permalink at the locked commit as a presentation setting, never one that
follows the floating `ref`; a provider grund does not know yields no guessed link.

### differences.integrity: Integrity is verified offline and recursively

Each projection is verified against its lock on every run that reads it, and every nested
projection against its own lock, by the checks of [§FS-remote-projects.checks](FS-remote-projects.md#checks-checks).

### differences.consumer-site: Suppression never lets a consumer citation pass

Two rules keep suppression honest. An upstream defect that invalidates a consumer citation,
such as a duplicate `ORG-x` the consumer cites, fails at the consumer's citation site. And
an absent optional member inside a remote that leaves a consumer citation unverified is
announced once, in the form of [§FS-check.4.9](FS-check.md#49-a-workspace-member-declared-optional-is-absent).

### differences.selectors: Integrity findings stay visible under content selectors

A selector that does not name an integrity finding does not hide it: `--only dangling`
over a modified projection still prints `remote-modified`, while `--ignore
remote-modified` names it and hides it. This is the one stated exception to
[§FS-check.2.1.2](FS-check.md#212-selection-filters-the-complete-report), because a filtered report over a namespace whose bytes are not the locked
ones is not a report about the remote at all.

### differences.cascade: An unavailable remote is one diagnostic

A remote that is missing, stale, modified or unloadable yields its one diagnostic, and
suppresses the unknown-alias finding of [§FS-check.3.8](FS-check.md#38-cross-project-citation-failure) for each consumer citation into it.

## bytes: Exact bytes

### bytes.digest: The digest is over exact bytes

A projection is the remote's exact bytes. The digest is never computed over normalized
content, because a digest over normalized line endings would let grund read bytes other
than the locked ones while reporting them as the locked ones.

### bytes.protection: Checkout transforms are prevented, or the fetch is refused

Fetch reads blobs from the object store and never checks the remote out, so no filter or
line-ending conversion runs on the way in. On the way out to the consumer's own checkouts,
fetch writes `.grund/remotes/.gitattributes` holding `* -text`, then asks git for each
projected path's effective attributes; when an attribute the remote itself carries, such as
a nested `*.md text eol=crlf`, would still convert a projected file, fetch refuses the
materialization and keeps the old projection and lock. A `core.autocrlf=true` checkout of a
committed projection reads the locked bytes.

### bytes.entries: Regular files only, and no executable bit

A remote tree containing a symlink or a gitlink is refused by fetch with an error naming
the entry; such entries may later be admitted as recorded inventory entries without
changing this contract. The digest covers entry types but not the executable bit, which
no grund operation reads and a Windows checkout does not preserve.

## checks: Checks

Each section below is one check `grund check` enforces about a remote, and its name is the
finding code verbatim, as [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog) publishes it and [§REQ-spec-section-names.code](../requirements/REQ-spec-section-names.md#code-a-check-is-named-by-its-diagnostic-code)
requires; each code's row in that catalog carries its severity. Every check applies
recursively to nested projections and stays visible under content selectors
([§FS-remote-projects.differences.selectors](FS-remote-projects.md#differencesselectors-integrity-findings-stay-visible-under-content-selectors)).

### checks.remote-missing: A declared remote has no pinned projection

A remote declared with no lock entry is reported at its `[workspace.remotes.<alias>]`
table; one locked with no projection directory is reported at its lock entry. The remedy
is `grund fetch --remote <alias>`.

### checks.remote-stale: The lock disagrees with the declaration

A lock entry whose `source` or `ref` differs from its declaration is reported at the
changed key of the declaration. The remedy is to refetch, or to restore the declaration.

### checks.remote-modified: The projection is not the locked bytes

A projection whose digest is not its lock entry's `digest` is reported at that `digest`.
The remedy is to restore the projection from version control, or to refetch.

### checks.remote-orphan: State no declaration names

A lock entry, or a directory under `.grund/remotes/`, that no declaration of the config
root names is reported where it stands. It breaks nothing the consumer reads and cannot be
repaired by fetching, so its remedy is to remove the stale state or restore the
declaration, and it never suggests fetching. It fires only in a config root that declares
at least one remote ([§FS-remote-projects.declaration.none](FS-remote-projects.md#declarationnone-a-repository-with-no-remotes-reads-nothing-new)).

## failures: Operational failures

Three failures leave the run unable to say what a remote declares. Each exits 2, the
incomplete-run exit of [§FS-check.2.4](FS-check.md#24-an-incomplete-run), so a namespace that was never established cannot
yield a clean filtered report. They keep today's stderr shape for an operational failure,
and none is a selectable check code.

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
`remote-unreadable`, naming the dependency path from the consumer to the input. It is the
failure an unloadable member config is today, which is equivalence rather than an
exception. The remedy is to upgrade grund for an unsupported config, or to restore damaged
input.
