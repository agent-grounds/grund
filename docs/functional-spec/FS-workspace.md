# FS-workspace: grund validates cross-project citations in a workspace

`grund` can treat a repository as a workspace of independent member projects. A local
citation is written as in a single project, while a cross-project citation names a stable
alias before the local ID, so a single project needs no configuration for it ([§GOAL-zero-config](../goals.md#goal-zero-config-works-on-any-conformant-tree))
and a larger repository gets an explicit resolver for its sub-projects without project names
in every ID ([§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable)). Decided in [§DF-subproject-namespaces](../decisions/functional/DF-subproject-namespaces.md#df-subproject-namespaces-alias-namespace-model-for-sub-projects-and-external-repos).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, home, citable, body, section, coordinate,
lead, catalog), [§FS-terms.terms.2](FS-terms.md#terms2-citations) (marker, citation, qualified citation, shorthand, citation
site), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope, config root, workspace,
member, alias), [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, caution, verdict, anchor), [§FS-terms.terms.6](FS-terms.md#terms6-rules-and-directions)
(grounded), and [§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations) (value, component, binding).

- **workspace root** — The project whose config carries the `[workspace]` block a run reads its
  members from.
- **boundary** — Where one project's scan stops because a member's own project begins.
- **claimed chain** — The `[workspace]` blocks from the outermost root down, each claiming the
  next, which is what an alias path is read along.
- **current project** — The project the run's own directory sits in, and what an unqualified
  citation resolves against.
- **absent** — Of an optional member: declared and not on disk. The block loads without it and
  the run continues.
- **unverified** — Of a citation into an absent member: neither resolved nor
  reported, a third outcome beside valid and dangling.

## requirements: What the workspace contract holds to

Four requirements that also hold here are stated where they are shared: a repository that
declares no workspace pays nothing for one ([§GOAL-small-and-large.3](../goals.md#3-layout-knobs-live-in-config)) beyond the qualified
shape it reads ([§FS-workspace.5.2](FS-workspace.md#52-recognition-in-a-run-that-loads-no-workspace)); a misconfigured block fails loudly at the line that
wrote it ([§FS-config.requirements.5](FS-config.md#requirements5-a-mistake-in-the-config-fails-loudly)); every surface answers a qualified citation as `check`
does ([§REQ-one-answer](../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree)); and a workspace run stays offline and its output deterministic
([§REQ-runs-offline](../requirements/REQ-runs-offline.md#req-runs-offline-verification-never-depends-on-an-external-service), [§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)).

### requirements.explicit: A cross-project citation names its target by alias path and resolves there alone

Which project a citation names is decided by its text: an unqualified ID resolves in the
current project only and a qualified one in the project its alias path names, never through
a file path or an unqualified ID another project happens to declare ([§FS-workspace.4](FS-workspace.md#4-resolution),
[§FS-workspace.8.1.1](FS-workspace.md#811-an-unqualified-id-another-project-declares), [§FS-workspace.8.9](FS-workspace.md#89-the-current-project)). The lookup then reads only the target's catalog,
grammar and shorthand policy ([§FS-workspace.1.2](FS-workspace.md#12-whose-grammar-reads-the-id), [§FS-workspace.4](FS-workspace.md#4-resolution)), so one ID in two projects
is two declarations rather than an ambiguity, and a candidate the run can name is message
text, never a pick ([§FS-workspace.8.1.1](FS-workspace.md#811-an-unqualified-id-another-project-declares)). Serves [§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution) and
[§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration).

### requirements.said: What a workspace does not check, it says

A qualified citation the run cannot resolve is an error at its site in every run, workspace
or not, whatever kind its tail names and whichever loaded grammar would accept it
([§FS-workspace.1.2](FS-workspace.md#12-whose-grammar-reads-the-id), [§FS-workspace.4](FS-workspace.md#4-resolution), [§FS-workspace.5.1](FS-workspace.md#51-a-member-run), [§FS-workspace.5.2](FS-workspace.md#52-recognition-in-a-run-that-loads-no-workspace)), unless the config
declared that its namespace may be absent. Every place a workspace leaves unchecked, or
names otherwise than its configs say, is declared by the config or announced by the run: an
absent optional member, declared one entry at a time and named once per member with
`success` withheld ([§FS-workspace.2.2](FS-workspace.md#22-a-member-that-may-be-legitimately-absent), [§FS-check.4.9](FS-check.md#49-a-workspace-member-declared-optional-is-absent)); the own files of an
`include_root = false` block ([§FS-workspace.6.1.2](FS-workspace.md#612-include_root-on-a-block)); a block no enclosing block lists, whose
projects are absorbed rather than named ([§FS-workspace.6.1.8](FS-workspace.md#618-a-block-no-enclosing-block-lists-is-outside-the-chain)); and alias paths an
unreadable ancestor claim may have shortened ([§FS-workspace.6.1.7](FS-workspace.md#617-a-claiming-block-that-cannot-answer-fails-the-run)). Only the two shapes
[§FS-workspace.6.1.8](FS-workspace.md#618-a-block-no-enclosing-block-lists-is-outside-the-chain) leaves unreported go unsaid. Serves [§REQ-no-missed-citation](../requirements/REQ-no-missed-citation.md#req-no-missed-citation-every-citation-the-run-reads-is-checked), whose
[§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded) declares the absent member's blind spot.

### requirements.own-catalog: A project's catalog holds its own declarations and no other project's

One project's scan never enters another's: the root scan stops at every member
([§FS-workspace.6](FS-workspace.md#6-nested-project-boundary)), and the boundary is mutual, belongs to the directory rather than to the
path that reaches it, symlinks included, and fences a member checked on its own by its
canonical root ([§FS-workspace.6.2](FS-workspace.md#62-the-boundary-is-mutual)). Member roots are compared canonically and may neither
overlap nor escape their block ([§FS-workspace.2.3](FS-workspace.md#23-what-a-member-entry-may-name)), and a block whose members would swallow
its whole scan is refused rather than left reading nothing ([§FS-workspace.2.1](FS-workspace.md#21-a-member-that-swallows-the-blocks-own-scan)). Serves
[§REQ-no-wrong-citation.2](../requirements/REQ-no-wrong-citation.md#2-no-false-alarms), since a declaration reported as a duplicate of itself is a false
alarm, and [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration).

### requirements.narrower-run: A narrower run is never more permissive than the outermost root

A citation that passes at a narrower scope passes at the outermost root, with one recorded
exception. A path below the config root and a project that declares no `[workspace]` load
no workspace, so every qualified citation there is an unknown alias ([§FS-workspace.5](FS-workspace.md#5-command-scope),
[§FS-workspace.5.1](FS-workspace.md#51-a-member-run), [§FS-workspace.6.1.7](FS-workspace.md#617-a-claiming-block-that-cannot-answer-fails-the-run)); a block in the claimed chain resolves a subset of
the root's alias paths, never a re-spelling of them ([§FS-workspace.6.1.5](FS-workspace.md#615-alias-paths-do-not-change-with-scope)). That is why a
block reads its paths from the outermost claim above it, a `grund.toml` above the checkout
included, and why a claiming block that cannot answer fails the run ([§FS-workspace.6.1.5](FS-workspace.md#615-alias-paths-do-not-change-with-scope),
[§FS-workspace.6.1.7](FS-workspace.md#617-a-claiming-block-that-cannot-answer-fails-the-run)): the enclosing-workspace case [§FS-config.requirements.8](FS-config.md#requirements8-a-projects-meaning-is-self-contained) names. The
recorded exception is a block no enclosing block lists, which names its subtree from itself
and is reported by every run whose scan meets it ([§FS-workspace.6.1.8](FS-workspace.md#618-a-block-no-enclosing-block-lists-is-outside-the-chain)). Serves
[§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration) at the outermost root, the run CI holds.

### requirements.ahead: Remotes, mounted roots and a relaxed standalone mode are specified ahead of their release

This point is deferred. A remote's and a mounted member's alias, a remote's place among a
block's siblings, the mounted-root exception to the outermost claim, the claim
`mounted_members` entries and `[workspace.remotes]` keys make, and a neighbouring repository
cited as a remote are [§FS-remote-projects](FS-remote-projects.md#fs-remote-projects-a-project-cites-another-repositorys-declarations-from-a-committed-pinned-projection) behaviour, owed to the Mount and Remotes pieces of
[§DF-remote-projects](../decisions/functional/DF-remote-projects.md#df-remote-projects-a-remote-project-is-a-workspace-member-whose-bytes-were-fetched-mounted-as-its-own-root); the mount layer they rest on is built ([§FS-remote-projects.mount.rule](FS-remote-projects.md#mountrule-a-mounted-root-resolves-natively-and-is-shown-under-its-alias),
[§FS-remote-projects.ignore](FS-remote-projects.md#ignore-ignore-isolation)). Until those pieces land, a config declaring `mounted_members`
or `[workspace.remotes]` is refused at load ([§FS-remote-projects.declaration](FS-remote-projects.md#declaration-declaration-and-registration)), and the
ancestor climb reads `members` and `optional_members` alone. A relaxed standalone mode that
downgrades a member-local unknown alias to a warning is owed to its own change
([§DF-subproject-namespaces.3.6](../decisions/functional/DF-subproject-namespaces.md#36-standalone-members-fail-loud-not-silent)). Serves [§REQ-remote-equivalence](../requirements/REQ-remote-equivalence.md#req-remote-equivalence-a-fetched-remote-answers-every-operation-as-a-local-project-does).

## 1. Citation syntax

A normal citation resolves inside the current project:

```text
§FS-login
§FS-login.2.1
```

A cross-project citation writes the target project's alias path, a slash, then
the target ID:

```text
§api/FS-login
§root/GOAL-compatibility
§payments/FS-refunds.3.2
§hardware/sprayer/FS-nozzle
```

The grammar is:

```text
§[alias/…/alias/]ID[.section]
```

The shape is read the same way in every repository, workspace or not ([§FS-workspace.5.2](FS-workspace.md#52-recognition-in-a-run-that-loads-no-workspace)).

### 1.1 The alias path

Each `alias` is a lowercase slug: it starts with a letter and then uses lowercase letters,
digits, or `-`. The alias path carries **one segment per workspace level**
([§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)); in a workspace with no nesting it is a single segment, so the common
form is `<§>alias/<ID>`. Since an ID never contains `/` (the ID grammar rejects one on
load, [§FS-config.3.2](FS-config.md#32-id--id-grammar)), the last slash in the token is always the boundary between the alias
path and the ID. So a marked file path is a citation: `<§>docs/functional-spec/FS-login.md`
is a two-segment alias path plus an ID, and marking it is what says "resolve this". A path
meant as a path is written without the marker, or outside the citation.

### 1.2 Whose grammar reads the ID

For an unqualified citation, the `ID[.section]` part uses the current project's ID and
section grammar. For a qualified citation, it uses the target project's grammar:
`<§>api/FS-001-session` is parsed with `api`'s `[id]` config, even when the citing project
uses a different ID format. A qualified citation whose alias path names no loaded project
has no target, so no project's grammar reads its tail, neither the citing project's nor
that of whichever loaded project declares the tail's kind: a workspace run reads it with
the fixed fallback shape of [§FS-workspace.5.2](FS-workspace.md#52-recognition-in-a-run-that-loads-no-workspace) and reports it at its site as
`unknown project alias <alias>` ([§FS-check.3.8](FS-check.md#38-cross-project-citation-failure)) whatever kind the tail names. So
`<§>org/GOAL-x` is an unknown-alias error in a workspace where no loaded project declares
`GOAL`, exactly as it is without `[workspace]` and as `<§>org/FS-a` is where one does.

## 2. Workspace configuration

A workspace is declared in the root project's `grund.toml` ([§FS-config.1](FS-config.md#1-file-location-and-discovery)):

```toml
project_name = "root"

[workspace]
members = ["apps/api", "packages/*"]
include_root = true
```

`members` is a list of paths or single-segment trailing globs, resolved relative to the
config root. `include_root` defaults to `true`; when false, a workspace-root `grund check`
checks only member projects ([§FS-workspace.6.1.2](FS-workspace.md#612-include_root-on-a-block)).

Each member is a separate project with its own IDs. A member with its own config, in either
discovery form, `.agents/grund.toml` or a bare `grund.toml` ([§FS-config.1](FS-config.md#1-file-location-and-discovery)), is configured by
that file; one with neither gets the canonical defaults with the member directory as the
config root. Root and members choose independently, so a workspace may mix the two forms.

### 2.1 A member that swallows the block's own scan

A member root may sit anywhere strictly inside the block that lists it, including on top of
the paths the block itself scans. When it covers **all** of them the block's own project
would read nothing: its declarations would reach no catalog and its dangling citations would
pass ([§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration)), so `grund` refuses the configuration. Take the block's **default
scope**, the roots `[scan] include` and the scanned `[[kinds]]` homes give it
([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)), which is the set [§FS-workspace.6](FS-workspace.md#6-nested-project-boundary)'s boundary prunes, and keep the ones that
exist on disk: when at least one remains and **every** one is at or inside an expanded
member root, the block fails to load with the config error of [§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan) at its
`members` line. Roots and member entries are compared as canonical paths, the way the scan's
own prune compares them, so a member reached through a symlink or a glob covers what it
actually lands on.

A block that omits `[scan] include` is not exempt: it is judged on that key's materialized
default roots, `requirements.md`, `docs`, `e2e` and `src` ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)). The question is
asked of the default scope whatever `--full` says ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)), since the member
boundary prunes the root `--full` adds as well. A partly covered scope is the boundary
working ([§FS-workspace.6](FS-workspace.md#6-nested-project-boundary)) and stays silent, and a block with `include_root = false` is not
a project, so it has no scan of its own to be covered; what its files cost is
[§FS-check.4.10](FS-check.md#410-include_root--false-leaves-the-blocks-own-files-unread). Decided in [§DF-absorbed-scan-warning](../decisions/functional/DF-absorbed-scan-warning.md#df-absorbed-scan-warning-a-scan-its-own-members-swallowed-is-a-warning-with-a-named-release-not-an-error).

### 2.2 A member that may be legitimately absent

`optional_members` is a sibling of `members` under `[workspace]`, with the same grammar, for
a member the repository knows may be missing, such as a submodule CI never fetches:

```toml
[workspace]
members          = ["apps/api", "packages/*"]
optional_members = ["vendored"]
```

Present, an optional member is an ordinary member under every rule of [§FS-workspace.2](FS-workspace.md#2-workspace-configuration).
Absent means **its path is not a directory**, the test [§FS-workspace.2.3](FS-workspace.md#23-what-a-member-entry-may-name) applies to every
member: a path that does not exist, a file, or a symlink that does not resolve to a
directory. An empty directory is present, even an uninitialized submodule's, which loads
under the canonical defaults with no declarations, so a citation into it is an
`unknown reference` error ([§FS-check.3.1](FS-check.md#31-dangling-citation)); such a repository lists the members under the
submodule instead, `optional_members = ["hardware/sprayer"]`. Every citation into an absent
member is **unverified**, reported once per member and never per site ([§FS-check.4.9](FS-check.md#49-a-workspace-member-declared-optional-is-absent)),
and nothing else moves: an alias that is not an optional member is unknown as in any run,
and a present member is checked to the letter ([§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded)).

The alias of an optional member is the **entry's last path segment**, present or not:
`sprayer` for `hardware/sprayer`, so a citation means the same thing in a full checkout and
a partial one. An absent member absorbs every alias path that begins with its segment at
its level, since it may itself have declared `[workspace]`: `<§>hardware/AR-bus` and
`<§>hardware/sprayer/FS-nozzle` are both unverified when `hardware` is the absent entry.
Such a citation is recognized by the fallback shape of [§FS-workspace.5.2](FS-workspace.md#52-recognition-in-a-run-that-loads-no-workspace), and a tail that
does not match it is unverified all the same.

These are config errors at the `optional_members` line ([§DF-optional-workspace-members](../decisions/functional/DF-optional-workspace-members.md#df-optional-workspace-members-an-absent-member-is-declared-in-a-sibling-list-and-the-run-announces-the-namespace-it-did-not-check)):

- a present member whose `project_name` disagrees with its entry's last segment, naming
  both;
- a last segment that is not a valid alias ([§FS-workspace.3](FS-workspace.md#3-aliases)), said before any directory is
  looked for;
- an entry ending in `/*`, since an absent parent names no members, with the message naming
  the shape to write instead;
- an entry naming a root that a `members` entry of the same block also names, compared
  canonically, so a `members` glob that expands onto an optional entry collides too.

```text
error: grund.toml:5: optional workspace member `vendored` declares project_name `warehouse`, but its alias is the entry's last segment `vendored` — make the two agree
error: grund.toml:5: [workspace] optional_members may not use a glob: `hardware/*` — an absent parent names no namespaces; list one concrete entry per namespace instead
```

A missing `members` entry fails as [§FS-workspace.2.3](FS-workspace.md#23-what-a-member-entry-may-name) says, and its message names this key:

```text
error: grund.toml:5: workspace member does not exist: vendored — list it in [workspace] optional_members if it may be legitimately absent
```

One optional entry written twice is one member, present or not: an absent entry is deduped
by the comparison [§FS-workspace.2.3](FS-workspace.md#23-what-a-member-entry-may-name) applies to `members`, taken from the entry text before
anything is announced. Every `[workspace]` block reads the key, at every depth and with no
privilege for the outermost: a nested block's entries are paths under its own config root,
validated by these rules, and its absent ones are announced at its own `optional_members`
line, rendered against the root the run was launched at ([§FS-errors.3](FS-errors.md#3-message-text)). The ancestor climb
reads `optional_members` beside `members` ([§FS-workspace.6.1.7](FS-workspace.md#617-a-claiming-block-that-cannot-answer-fails-the-run)), so a run started inside a
present optional member spells its alias path as the workspace root does. A block whose
last project is absent is not an empty block ([§FS-workspace.6.1.2](FS-workspace.md#612-include_root-on-a-block)), because emptiness is
read from the config text: it contributes no project, its absent members are announced,
and a run left with nothing to read earns the empty-scan caution too ([§FS-check.2.2](FS-check.md#22-empty-scan)).

### 2.3 What a member entry may name

Member paths must be relative, must not use `.` or `..`, must not use platform-specific
absolute forms or backslash separators, and must not overlap after glob expansion: one
member root cannot contain another. Each must also resolve, through symlinks, to a
directory *strictly inside* the config root that lists it: not that root itself, not an
ancestor of it, not another tree ([§DF-nested-workspaces.3.6](../decisions/functional/DF-nested-workspaces.md#36-termination-is-containment-not-a-depth-limit)). Members are compared as
canonical paths: two entries of one list that resolve to the same root are one member,
deduped rather than rejected, so a glob may name a directory an explicit entry also names
(`members = ["packages/*", "packages/api"]`).

An invalid entry, a missing member path, an escaping root and overlapping expanded roots are
config errors at the `members` line that introduced them ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)), naming the entry
as the config wrote it rather than the root it resolved to.

### 2.4 Trailing globs

`packages/*` means every direct child directory under `packages/`, including a child
symlink whose target is a directory; [§FS-workspace.2.3](FS-workspace.md#23-what-a-member-entry-may-name)'s canonical containment rule applies
to that target. If the glob's parent exists but cannot be read, that is a config error at
the `members` line naming the whole glob as written and preserving the filesystem reason:
“cannot read workspace member glob `packages/*`: `<I/O reason>`”.

## 3. Aliases

The root alias is the root config's `project_name`, or `root` when omitted. A member alias
is the member config's `project_name`, or the member directory's basename when omitted. An
optional member's alias is its entry's last path segment ([§FS-workspace.2.2](FS-workspace.md#22-a-member-that-may-be-legitimately-absent)), and an
import's is its table key ([§FS-imports.declaration.alias](FS-imports.md#declarationalias-the-alias-is-the-table-key)).

Aliases must match the lowercase slug grammar of [§FS-workspace.1.1](FS-workspace.md#11-the-alias-path) and must be unique
**among siblings**: the root project, the top-level members and the block's imports share
one level, and each nested `[workspace]` block's members share another ([§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)).
A duplicate or invalid alias is a launch-time error, because a qualified citation would
otherwise have two possible targets. Two projects under different parents may carry the
same alias: their alias paths differ, so nothing is ambiguous.

### 3.1 Project descriptions

A project's optional one-line `project_description` ([§FS-config.3](FS-config.md#3-keys-version-1)) follows the alias's
residency rule: a member's comes from the member's own config, the root's from the root
config, and a member without its own config has none ([§DF-workspace-member-descriptions](../decisions/functional/DF-workspace-member-descriptions.md#df-workspace-member-descriptions-member-side-project_description-for-workspace-member-lists)).
It is presentation only: generated workspace member lists render it beside the alias
([§FS-init.2.3.4](FS-init.md#234-managed-block-content-sections)), and it never takes part in alias derivation, citation resolution or
`check`.

## 4. Resolution

During `grund check`:

- `<§>ID` resolves only against declarations in the current project.
- `<§>alias/ID` resolves only against declarations in the named workspace project, and
  `<§>alias/ID.section` additionally requires that the target declaration contain the
  cited section.
- An unknown alias, and a known alias with no matching declaration, are errors at the
  citation site ([§FS-check.3.8](FS-check.md#38-cross-project-citation-failure)).
- An alias path that names, or descends into, an **absent optional member**
  ([§FS-workspace.2.2](FS-workspace.md#22-a-member-that-may-be-legitimately-absent)) is neither: the citation is *unverified*, nothing is reported at its
  site, and the run names the member once, at its `optional_members` entry ([§FS-check.4.9](FS-check.md#49-a-workspace-member-declared-optional-is-absent)).
- A finding for a known alias renders the `<ID>` and section separator with the target
  project's `[id]` config, not the citing project's; the literal source token remains the
  scanner's evidence.

A qualified citation is never resolved by path syntax such as `../FS-login` or
`packages/api/FS-login`; aliases are the stable names. One ID declared in two projects is
two declarations, not an ambiguity: the alias picks one.

Each lookup uses only the selected project's catalog, shorthand policy, and the effective
format of the candidate kind in that project ([§FS-config.3.1](FS-config.md#31-reference--citation-form), [§FS-config.3.2](FS-config.md#32-id--id-grammar)): an exact
local or qualified off-grammar declaration and its exact marked citations resolve, but the
same spelling is never borrowed from a sibling catalog. Raw spelling is preserved in
output, and the ordinary duplicate, shorthand and section ambiguity rules refuse rather
than guess. The target's shorthand policy governs a qualified number-only shorthand in
both `check` and `fmt`: a canonical root accepts and preserves a shorthand into an
`accepted` member, while an accepted root reports and expands a shorthand into a canonical
member. The policy changes only a unique persisted form; unknown and ambiguous shorthand
remain errors, and query and authoring input resolve or expand canonically
([§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation), [§FS-fmt.2.4](FS-fmt.md#24-shorthand-to-canonical)).

Value bindings use this resolver with no lookup path of their own: an unqualified binding
resolves locally, and an alias-qualified one uses the target project's grammar, `values`
opt-in, declaration and exact component and keeps the member-local unknown-alias error
([§FS-values.7](FS-values.md#7-workspaces-and-editor-consumers)).

## 5. Command scope

Which projects a run at the workspace root loads is [§FS-scan.projects.workspace-root](FS-scan.md#projectsworkspace-root-a-run-at-the-workspace-root-loads-every-project), and a
run narrower than the config root is [§FS-scan.projects.narrowed](FS-scan.md#projectsnarrowed-a-path-below-the-config-root-loads-what-a-run-from-it-would-see).

### 5.1 A member run

Which project a member run loads is [§FS-scan.projects.member](FS-scan.md#projectsmember-a-member-run-loads-the-member). Qualified citations such as
`§root/<ID>` or `§sibling/<ID>` cannot be resolved without the workspace context; each such
citation produces an `unknown project alias <path>` error at the citation site
([§FS-check.3.8](FS-check.md#38-cross-project-citation-failure)) rather than being skipped, which would let a passing member check ship a
qualified citation that fails at the workspace root ([§DF-subproject-namespaces.3.6](../decisions/functional/DF-subproject-namespaces.md#36-standalone-members-fail-loud-not-silent)). Run
`grund check` at the workspace root to validate cross-project citations.

#### 5.1.1 Verified enclosing-workspace guidance

When a member-local check refuses a qualified citation whose alias names the enclosing
workspace's root project, its `unknown project alias <path>` finding says so, says that the
checked project is a member of that workspace, and directs the reader to run `grund check`
at the workspace root, located relative to the invocation directory. This holds for a
member with its own config and for one using the canonical defaults through workspace
discovery:

```text
docs/api.md:3: error: unknown project alias root; root names the enclosing workspace's root project, and this project is a member of that workspace — run `grund check` from the workspace root at `.` to validate cross-project citations
```

The explanation requires verified membership in the enclosing workspace and the root
project's actual alias: a standalone project, an unlisted nested project and a genuinely
unknown alias do not receive it. Finding the enclosing configuration for it loads none of
its declarations and resolves nothing, so the guidance says where cross-project citations
are checked and promises neither that the written target or section exists there nor that a
run at the root passes.

### 5.2 Recognition in a run that loads no workspace

A marked `<§>alias/<ID>` token is a citation in **every** repository, including one with no
`[workspace]` ([§FS-citations.recognized.contexts](FS-citations.md#recognizedcontexts-contexts-read-as-neither-prose-nor-code), which skips it only inside a source file's
string literal or inline-code span): its alias path resolves against nothing there, so it is
reported as an unknown project alias at its site rather than skipped ([§FS-check.3.8](FS-check.md#38-cross-project-citation-failure)).

The scanner of a run that loads no workspace recognizes the `<§>alias/` prefix before
applying the local ID grammar, so `<§>root/FS-root` is an unknown-alias citation inside a
default `{kind}-{number}-{slug}` member rather than plain text. It reads the tail with a
fixed fallback shape, `KIND[-NUM]-SLUG` with an uppercase-or-digit kind and a non-empty
slug, because no target's `[id] format` is loaded. That shape does not decide whether the
citation is reported: the unknown thing is the alias, so a tail that does not match it is
an `unknown project alias` error all the same. The workspace-root run, which parses each
tail whose alias names a loaded project with that project's grammar, is the one place the
tail itself is checked.

### 5.3 The empty-scan caution names each project from the report base

A run at the workspace root asks [§FS-check.2.2](FS-check.md#22-empty-scan)'s question of each project it loads, and
names a project whose own scan read nothing by its root, in [§FS-check.2.2.2](FS-check.md#222-the-message-names-the-likely-cause)'s message, its
exclude-pattern form, or beside the outward link it names in their place. That root is
spelled from the report base the run's other paths use and never absolute
([§FS-config.3.6](FS-config.md#36-output--report-format)): the workspace root reads ``no files under `.` matched`` and a member
``no files under `m` matched``, wherever the checkout sits on disk ([§FS-errors.4](FS-errors.md#4-determinism)) and as a
run without `[workspace]` prints over the same root.

## 6. Nested project boundary

Workspace members are project boundaries. When the root project is checked as part of a
workspace, the root scan does not scan member project directories or their descendants,
even if the root project's `[scan] include` names a path inside a member; the member is
scanned separately under its own config and alias, so a child project's declaration never
becomes a duplicate or dependency of the root project. A version-2 config has no `include`
to prune: a place at or inside a member's root is a load error instead, located at the
place ([§FS-config-v2.schema.places.boundaries](FS-config-v2.md#schemaplacesboundaries-what-a-place-may-name)). A `members` list that prunes *every* one of
the block's own scan roots is [§FS-workspace.2.1](FS-workspace.md#21-a-member-that-swallows-the-blocks-own-scan).

### 6.1 Nested workspaces

A member may itself declare `[workspace]`. Its `members` are paths under *that* member's
config root, resolved and validated by the rules of [§FS-workspace.2](FS-workspace.md#2-workspace-configuration)
exactly as the outermost root's are, and the tree may nest to any depth: containment, not a
depth limit, is what makes expansion terminate ([§DF-nested-workspaces.3.6](../decisions/functional/DF-nested-workspaces.md#36-termination-is-containment-not-a-depth-limit)).

A project is named by its **whole alias path**: one segment per level, read from the
outermost workspace down ([§FS-workspace.1.1](FS-workspace.md#11-the-alias-path)), the outermost root contributing no segment.
Given a root with `members = ["hardware", "final"]` and `hardware` declaring
`members = ["sprayer", "pod"]`:

```text
§root/GOAL-x            the root project        §final/FS-x     a top-level member
§hardware/AR-bus        the node's own decl     §hardware/sprayer/FS-x   a leaf
```

Paths rather than globally unique names are what let two `pod` projects under different
parents coexist ([§DF-nested-workspaces](../decisions/functional/DF-nested-workspaces.md#df-nested-workspaces-a-nested-project-is-named-by-its-whole-alias-path)); a short leaf name does not resolve on its own, and
`grund check` at the outermost root names the projects the written path could have meant
([§FS-check.3.8](FS-check.md#38-cross-project-citation-failure)).

#### 6.1.2 `include_root` on a block

`include_root` is read per `[workspace]` block and governs that block's own project. Under
the default an intermediate node is a project like any other ([§DF-nested-workspaces.3.5](../decisions/functional/DF-nested-workspaces.md#35-the-intermediate-node-reuses-include_root-and-defaults-to-a-project)):
it derives an alias ([§FS-workspace.3](FS-workspace.md#3-aliases)), is scanned under its own config minus its members'
subtrees ([§FS-workspace.6](FS-workspace.md#6-nested-project-boundary)), and is citable at its own path. Under `include_root = false` it
is pure grouping, no project but a segment in every path below it, so
`<§>hardware/sprayer/<ID>` is unaffected either way. Its own files are then read by
**nobody**: the enclosing scan stops at the member boundary and no other scan covers the
directory, not even under `--full` ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)), so a declaration there reaches no
catalog and a citation there is never checked. A block whose own tree holds a file a scan
would have read earns the warning of [§FS-check.4.10](FS-check.md#410-include_root--false-leaves-the-blocks-own-files-unread); where it holds none, the run stays
silent.

Every block must put at least one project in scope. `include_root = false` with no members,
read from the config text with `optional_members` included and `members = []` counted as
none, is a config error at that block's `members` line, or at its `[workspace]` line when
there is no `members` key, so the error says which block is empty:

```text
error: grund.toml:16: workspace has no projects in scope (include_root = false and no members)
```

A non-empty list whose glob entries all match no directories is a config error at the
`members` line naming the first unmatched glob in list order. It belongs to the empty
block, not to each empty glob: an included root or another member that puts a project in
scope keeps the block valid.

```text
error: grund.toml:16: the glob `pkgs/*` matched no directories
```

#### 6.1.5 Alias paths do not change with scope

Discovery stops at the *nearest* config ([§FS-config.1](FS-config.md#1-file-location-and-discovery)), so a command invoked at an
intermediate node runs that node's subtree ([§FS-workspace.5](FS-workspace.md#5-command-scope)). **Alias paths do not change
with scope** for every scope in the **claimed chain**: a run started at any of its blocks
resolves a *subset* of the outermost root's paths rather than a re-spelled set of its own.
Inside `hardware/`, `<§>hardware/sprayer/<ID>` names what it names at the repository root
and `<§>final/<ID>` is unknown *here*, which is what the narrowed run's finding says instead
of proposing a project it does hold ([§FS-check.3.8](FS-check.md#38-cross-project-citation-failure)). The guarantee is quantified over the
scope, not the project: a block the chain never lists is not one of those scopes even when
the projects under it are reached by the chain, which a multi-segment entry (`grp/inner`)
makes routine ([§FS-workspace.6.1.8](FS-workspace.md#618-a-block-no-enclosing-block-lists-is-outside-the-chain)).

**A path is read from the outermost block that claims a directory:** a multi-segment
`members` entry (`grp/inner`) hops a directory that may itself declare `[workspace]` and
list the same child, and the outer claim is the one the scan down from the outermost root
follows. Ordinary nesting has one claim per directory, where the two agree.

#### 6.1.7 A claiming block that cannot answer fails the run

**A block that claims a directory and cannot answer** — a missing member, overlapping roots,
an invalid alias for the project below it, or a config that does not load at all — fails
the run with *its own error*, from its own `members` or `project_name` line, rendered
against the root **this run** was launched at, so a block above that root renders with
`..` and the reader lands on the file that holds the line ([§FS-errors.3](FS-errors.md#3-message-text)):

```text
error: ../grund.toml:16: workspace member does not exist: missing — list it in [workspace] optional_members if it may be legitimately absent
```

Only a run that reads a path carries this obligation: a run at a project that declares no
`[workspace]` of its own, a leaf member or any single-project repository, resolves one
project ([§FS-workspace.5](FS-workspace.md#5-command-scope)), so no claim above it, answered or not, can fail it. A block
that does not name this directory is not asked, so neither a `members` list it could not
expand nor a config that would not load is an error in a run below it.

The claim is read from the **`members` and `optional_members` entries alone**, never from a
loaded config: those values are parsed on their own, with no other key read and no shape
rule applied, so a config that fails to load is still asked whether it claims this
directory. Each value is read as its config's own version reads it
([§DF-ancestor-claim-read-by-its-version](../decisions/functional/DF-ancestor-claim-read-by-its-version.md#df-ancestor-claim-read-by-its-version-an-ancestors-workspace-claim-is-read-as-its-own-config-version-reads-it-and-one-that-cannot-be-read-answers-nothing)): a version-2 value as that reader reads an array
([§FS-config-v2.reader.5](FS-config-v2.md#reader5-an-array-may-span-lines)), and a version-1 value on its key's line ([§FS-config.3](FS-config.md#3-keys-version-1)), so a
version-1 list written across lines or with a trailing comma is not a list. Another key's
array, closed or not, never makes either value unobtainable.

A config whose `members` or `optional_members` text cannot be obtained, because the file
cannot be read or either value is not a list, leaves the claim undecidable in both
directions, and the run continues. The climb that spells this run's alias path says so in
one run-level warning located at that config and no line ([§FS-errors.2.2](FS-errors.md#22-cli-level-message)), rendered by each
frontend, an editor included ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)):

```text
warning: ../grund.toml: cannot read [workspace] members (`members` is not a list of strings); alias paths below it may be missing a segment
```

A second, **quiet** climb asks the same ancestors, by the same read, whether they list a
`[workspace]` block the run's own scan met, which decides whether [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) reports
it. It spells nothing, so it fails no run, and an ancestor it cannot read leaves the block
unreported, with no warning.

#### 6.1.8 A block no enclosing block lists is outside the chain

**A `[workspace]` block that no enclosing block lists is outside the chain:** at the outer
scope it is ignored, so its tree belongs to the enclosing project when that project's scan
reaches it and to nobody when it does not, while a run started **at** it names every path
from itself, so a citation can pass that block's check and fail the run CI does. A run
started at a block *below* it that the chain does list is back inside the guarantee. A run
whose own tree scan meets such a block reports it ([§FS-check.3.29](FS-check.md#329-unlisted-workspace-block)). Two shapes stay
unreported: a block the scan never reaches, behind `[scan] exclude`, an ignore file, a
member boundary or a narrowed scope; and a block an enclosing config names and cannot
answer for, the unanswered claim of [§FS-workspace.6.1.7](FS-workspace.md#617-a-claiming-block-that-cannot-answer-fails-the-run).

### 6.2 The boundary is mutual

**The boundary is mutual, and it belongs to the directory rather than to the path that
reaches it.** A member's own scan stops at every *other* project in the workspace exactly
as the root scan stops at the members: at a sibling member's files, and at the root
project's. Ordinary descent cannot cross the line, but a symlink is followed
([§FS-config.3.5.1](FS-config.md#351-a-symlink-in-the-tree-is-followed)), so the directory a link resolves to is asked which project owns it, the
innermost project root that contains it, and a directory owned by another project is not
descended into, whichever project's scan met it and under whatever name. This
loaded-workspace ownership rule is stronger than the physical-root rule: it also separates
another project whose root lies inside the current project's root.

A member checked on its own does not load the workspace map and needs none: its canonical
project root fences directory-link traversal ([§FS-config.3.5.1](FS-config.md#351-a-symlink-in-the-tree-is-followed)), so a link into a sibling
or the workspace root is pruned as an outward directory link and the member cannot absorb
another project's declarations. An ordinary parent-relative scan include is not a link
traversal and keeps the external scope [§FS-config.3.5.1](FS-config.md#351-a-symlink-in-the-tree-is-followed) gives it.

## 7. Neighboring repos

A neighbouring repository is cited as `<alias>/<ID>`. A neighbour the build already placed
beside the repository is declared as an import and read live from that tree, and `check`
says so on every run ([§FS-imports](FS-imports.md#fs-imports-a-project-cites-a-sibling-project-its-build-places-beside-it-read-live-from-that-tree)). A remote, fetched into a committed, pinned projection
and read offline ([§FS-remote-projects](FS-remote-projects.md#fs-remote-projects-a-project-cites-another-repositorys-declarations-from-a-committed-pinned-projection)), is specified ahead of its release
([§FS-workspace.requirements.ahead](FS-workspace.md#requirementsahead-remotes-mounted-roots-and-a-relaxed-standalone-mode-are-specified-ahead-of-their-release)), and a config that declares one is refused at load
([§FS-remote-projects.declaration](FS-remote-projects.md#declaration-declaration-and-registration)).

## 8. Other commands

Qualified `show` and `refs`, workspace `list`, the completion helper, `fmt --cross-refs` and
`cover` each resolve a qualified ID exactly as `grund check` does, under the target
project's per-kind format and with the catalog-backed read compatibility of [§FS-config.3.2](FS-config.md#32-id--id-grammar):
what a command adds is presentation, never a resolution rule of its own.

### 8.1 `grund <alias>/<ID>`

`grund <alias>/<ID>` reads a declaration in another workspace project, rendered exactly as
`grund <ID>` renders a local one ([§FS-show.2](FS-show.md#2-behavior)): the same slices (`--brief`, default,
`--toc`, `--full`), `text` and `md` heading behaviour, section selection and inline-code
extraction ([§FS-show.2.3](FS-show.md#23-source-declarations-in-code-and-doc-comments)). The alias prefix changes which tree is scanned, not what is
printed: `grund api/FS-login` prints the lead of `api`'s `FS-login`, and
`grund api/FS-login.3.1 --toc` the `3.1` section's lead and heading map.

#### 8.1.1 An unqualified ID another project declares

`grund <ID>` resolves against the current project alone ([§FS-workspace.4](FS-workspace.md#4-resolution)), so an ID only
another project declares is not found. In a workspace run the refusal names the projects
that do declare it and **still refuses**: exit `1`, stdout empty, no body printed, from
`grund show <ID>` and the bare `grund <ID>` ([§FS-cli.1](FS-cli.md#1-the-default-subcommand)) alike, since a printed body would
teach a citation `check` rejects ([§DF-subproject-namespaces.3.7](../decisions/functional/DF-subproject-namespaces.md#37-check-comes-first-query-commands-follow)).

```text
ID not found: SPEC-007-shipping; did you mean vendored/SPEC-007-shipping?
```

- The candidates come after the ID, so the line opens `ID not found: <ID>`, the prefix that
  selects the `not-found` code ([§FS-distribution.3.0](FS-distribution.md#30-language-neutral-data-shapes)); under `--format json` the finding
  keeps `"code":"not-found"` with the candidates inside `message`.
- Several projects declaring the ID is not an `ambiguous ID` error: the candidates are
  deduplicated, sorted, limited and joined as [§FS-check.3.8](FS-check.md#38-cross-project-citation-failure) treats its own, and none is
  chosen ([§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution)).
- The written ID is re-parsed and rendered in each candidate project's grammar, as a
  qualified citation is ([§FS-workspace.1.2](FS-workspace.md#12-whose-grammar-reads-the-id)), and the current grammar's rejection does not
  pre-empt the search, since a kind may exist only in a loaded member. An actual member
  declaration earns the same refusal with qualified candidates and `not-found`
  classification, for bare and explicit `show`, the programmatic show API and explicit
  batch queries; without one, the original failure stands, `invalid ID` and its format
  hint included ([§FS-show.3.5](FS-show.md#35-the-hint-for-each-failure)).
- A run that holds one project, member-local, a standalone repository, or a `<path>` that
  resolves inside a member ([§FS-workspace.5](FS-workspace.md#5-command-scope), [§FS-workspace.8.8](FS-workspace.md#88-discovery-follows-grund-checks-walk-up)), names no candidate and
  prints the bare line.
- The section is not carried: `grund SPEC-007-shipping.2` is offered
  `vendored/SPEC-007-shipping`, since the run never looked for the section.
- Where a candidate is named, the `ID not found` hint line is not printed, and where there
  is none it prints as [§FS-show.3.5](FS-show.md#35-the-hint-for-each-failure) says; the hint line has no JSON form either way.

#### 8.1.2 The JSON `path` base

Under `--format json` and the default `relative_paths = true`, the reported `path` is
relative to the config root the command resolved against, matching `grund list`
([§FS-config.3.6](FS-config.md#36-output--report-format)): `grund api/FS-login --format json` from the workspace root reports
`apps/api/…`, not the member-relative `…`.

#### 8.1.3 An unknown alias outside a workspace context

Outside a workspace context, a `<path>` argument that resolves member-local included,
`grund api/FS-login` exits `2` with two stderr lines:

```text
error: unknown project alias `api`
note: workspace aliases are defined in the root grund.toml under [workspace]
```

### 8.2 `grund refs`

`grund refs <alias>/<ID>` lists every citation of that qualified declaration across the
**whole workspace**: the cross-project form `<§><alias>/<ID>` anywhere, and the local form
`<§><ID>` **inside the named project's tree**. Both cite the same target
([§FS-workspace.4](FS-workspace.md#4-resolution)), so an author about to delete `api`'s `FS-login` learns about `api`'s own
files and every other project that wrote `<§>api/FS-login`. `--descendants` ([§FS-refs.1](FS-refs.md#1-inputs))
widens the section filter inside the target project's coordinate space only: it decides
which sections count, never which projects do.

#### 8.2.2 An unqualified query at the workspace root

`grund refs FS-login` at the workspace root lists citations of the *root's* `FS-login`
only, root being the current project, and those include the `<§>root/FS-login` sites in
members; `grund refs root/FS-login` is the same query there and the canonical form for
scripts.

When the current grammar rejects an unqualified ID but a loaded member actually declares
it, text ID-mode `refs` keeps `invalid ID`, exit `1` and empty stdout, and replaces its
format hint with the qualified ID and a usable `grund refs <alias>/<ID>` command. The
candidates are found as [§FS-workspace.8.1.1](FS-workspace.md#811-an-unqualified-id-another-project-declares) finds them, ordered and capped, each project
asked in its own grammar, within the loaded scope and without the section, and no member's
citation hits are searched. JSON keeps its hint-free `invalid-id` stderr finding
([§FS-refs.4](FS-refs.md#4-exit-codes)); a root-valid empty query succeeds with its note; qualified, member-local and
narrowed queries keep their scope; and without a real candidate the format hint stays.

#### 8.2.3 `--summary`

`--summary` ([§FS-refs.3.3](FS-refs.md#33---summary)) aggregates per file whichever member the file lives in, its
paths spelled as [§FS-workspace.8.7.3](FS-workspace.md#873-paths) says.

#### 8.2.4 `--format json`

`--format json` adds one field to each citation object, `"project": "<alias>"`: the alias
of the project that *contains* the citation site, not the target. The target is the query
argument itself, `<alias>` for a qualified lookup and the current project for an
unqualified one, and is not repeated per row. `"id"` is rendered with the target project's
`[id]` config and `"text"` remains the verbatim source citation. `"enclosing_declaration"`
is **bare**, rendered with the *citing* project's `[id]` config and never alias-qualified,
because the declaration it names sits in the citing file and `"project"` already names that
project ([§FS-cover.3.2](FS-cover.md#32---format-json)).

### 8.3 `grund list`

`grund list` at a workspace root prints the catalog of every project the workspace covers,
root plus members subject to `include_root` ([§FS-workspace.8.10](FS-workspace.md#810-include_root--false-leaves-the-root-alias-unknown)), one row per declaration
with no collisions even when two projects declare the same local ID.

#### 8.3.1 The ID column is always qualified

Whenever workspace mode is loaded, the ID column renders as `<alias>/<ID>` for every row,
the current project's and the rows `--project` narrows to included. The only way to
unqualified rows is a member-local `list`, a member-local `<path>` argument included.

#### 8.3.2 `--project`

`--project <alias>[,<alias>...]` narrows the catalog to the named projects, still rendered
with their prefix ([§FS-workspace.8.3.1](FS-workspace.md#831-the-id-column-is-always-qualified)). An unknown alias is a CLI-level error, exit `2`,
the same shape as `--kind` ([§FS-list.4](FS-list.md#4-exit-codes)). `--kind` composes with `--project` as an
intersection: `--kind FS --project api,payments` lists the FS declarations of those two
projects.

#### 8.3.3 `--size`

`--size` keeps the same project qualification, filtering, member-local configuration and
workspace-root-relative paths. Each size row begins with its project's alias as `project`
in JSON, and its `id` is the qualified coordinate the workspace list renders. `--top`
applies after project filtering and breaks ties by the workspace list order
([§FS-list.3.4](FS-list.md#34---size--per-coordinate-lead-and-full-body-measurements)).

#### 8.3.4 `--summary`

With a workspace loaded, `grund list --summary` ([§FS-list.3.3](FS-list.md#33---summary)) emits one row per
`(project, kind)` pair, sorted by alias path as the catalog's rows are ([§FS-list.2.4](FS-list.md#24-order)), then
by that project's configured kind order, in text and `--format json` alike. `--project`
narrows it to those projects' kinds, and the alias column is sized to the widest alias
among the rows left, capped as the ID column is ([§FS-list.3.1](FS-list.md#31---format-text-default)).

#### 8.3.5 `--format json`

`--format json` adds `"project": "<alias>"` to every object and renders `id` qualified
(`api/FS-login`). The `refs` count is the count under that project's qualified target,
computed as [§FS-workspace.8.2](FS-workspace.md#82-grund-refs) defines.

### 8.4 Shell completions

The helper `grund complete ids` ([§FS-completions.2](FS-completions.md#2-internal-dynamic-helper)) reads the prefix typed so far. With no
`/` in it, it offers the current project's bare IDs; invoked at the workspace root it
**also** offers one candidate per known alias path with a trailing `/`: `api/`,
`payments/`, `group/alpha/`, and `root/` only when `include_root = true`
([§FS-workspace.8.10](FS-workspace.md#810-include_root--false-leaves-the-root-alias-unknown)). The trailing slash is the continuation signal.

#### 8.4.1 A prefix containing `/`

A prefix containing `/` is split on the **last** `/`, the boundary a citation uses
([§FS-workspace.1.1](FS-workspace.md#11-the-alias-path)). The left side names a project, and the helper offers its IDs whose
qualified form matches the prefix **and** every known alias path continuing past the
prefix, again with a trailing `/`. So `group/` offers `group/FS-x` alongside `group/alpha/`
and `group/beta/`, and one more Tab after `group/alpha/` reaches that project's IDs. The
prefix itself is never re-offered, since it would stall the shell rather than advance it.

#### 8.4.2 Section candidates

`--sections` and the implicit section mode (a prefix containing the configured `[id]
section_separator`) compose with the alias prefix: `api/FS-login.` offers section
candidates against `api`'s declaration of `FS-login`.

#### 8.4.3 Helper errors stay quiet

A workspace that fails to load drops the helper back to single-project mode, so a user
typing in the shell never sees a stack trace ([§FS-completions.2](FS-completions.md#2-internal-dynamic-helper)).

#### 8.4.4 Shell script adjustments

A Tab from `api` advances to `api/` *without inserting a space*; each generated script
suppresses the space its own way:

- `bash` — the completion function calls `compopt -o nospace` for the current invocation
  whenever any candidate in the batch ends in `/`.
- `zsh` — the completion function partitions the candidate batch: slash-suffixed
  candidates are added with `compadd -S ''` (no auto-suffix), bare-ID candidates with the
  default suffix.
- `fish` — candidates are emitted via `complete -f -k`; fish appends no space when the
  candidate's last character is `/`.

### 8.5 `grund fmt --cross-refs`

At the workspace root the link wrapper ([§FS-fmt.6](FS-fmt.md#6-cross-reference-emission)) wraps a qualified citation
`<§>api/FS-login` in `docs/index.md` to
`[<§>api/FS-login](../apps/api/docs/functional-spec/FS-login.md#fs-login-...)`: the
relative path crosses the workspace into the member's home, and the anchor is computed
from the *member's* declaration heading under the member's configured anchor profile
([§FS-fmt.6.7](FS-fmt.md#67-configurability)). The link names the declaration `check` resolves
([§REQ-one-answer.identity.declarations](../requirements/REQ-one-answer.md#identitydeclarations-every-command-reads-one-declaration-set)) at the anchor `show` reports
([§REQ-one-answer.writes.anchors](../requirements/REQ-one-answer.md#writesanchors-shows-anchor-is-the-anchor-fmt-writes)).

#### 8.5.1 A member-local run leaves qualified citations untouched

A member-local run, inside `apps/api/` or through a `<path>` that resolves member-local
([§FS-workspace.8.8](FS-workspace.md#88-discovery-follows-grund-checks-walk-up)), **leaves qualified citations untouched**: it cannot resolve them
([§FS-workspace.5.1](FS-workspace.md#51-a-member-run)), and `--cross-refs` does not wrap a citation `check` would reject
([§FS-fmt.6.4](FS-fmt.md#64-what-is-never-wrapped)). A wrapper an earlier workspace-root pass left around a qualified citation,
such as `[<§>root/FS-x](../../docs/FS-x.md#...)`, is **preserved as written**, neither
stripped nor re-derived, until `fmt --cross-refs` next runs at the workspace root, the one
run that creates or refreshes cross-project wrappers.

#### 8.5.2 Re-derive crosses projects

A heading rename or a file move in `api` gives a one-line `fmt` diff ([§FS-fmt.6.3](FS-fmt.md#63-idempotency-and-re-derive)) in every
*other* project that wrapped a citation of the renamed thing, exactly as in `api`'s own
files, in the one workspace-root pass.

#### 8.5.3 Each member's `[fmt.cross_refs]` governs its targets

A citation that targets a member is wrapped under that member's own `[fmt.cross_refs]`
([§FS-fmt.6.7](FS-fmt.md#67-configurability)): its `anchor_format` wins for its own declarations, so a mixed-profile
workspace renders each project's anchors under that project's profile.

### 8.6 `grund cover`

`grund cover` at a workspace root, with no `<path>` or the config root itself, spans
**every project the workspace covers**, root plus members subject to `include_root`
([§FS-workspace.8.10](FS-workspace.md#810-include_root--false-leaves-the-root-alias-unknown)), with one entry per scanned file, as [§FS-workspace.8.3](FS-workspace.md#83-grund-list) defines the
catalog for `list`; a member-local invocation, or a `<path>` that resolves member-local,
covers that member alone ([§DF-cover-workspace-scope](../decisions/functional/DF-cover-workspace-scope.md#df-cover-workspace-scope-cover-indexes-the-whole-run-and-counts-cross-project-citations)). Whenever workspace mode is loaded,
`--format json` adds `"project": "<alias>"` to the per-file object and to each nested
citation object, in the shape [§FS-cover.3.2](FS-cover.md#32---format-json) specifies, and `enclosing_declaration` stays
**bare**, as in `refs` ([§FS-workspace.8.2.4](FS-workspace.md#824---format-json)); outside workspace mode no field is added.

#### 8.6.1 A narrower `<path>` is one narrowed scan

See [§FS-scan.path.cover](FS-scan.md#pathcover-cover-runs-one-narrowed-scan).

#### 8.6.4 The rendered `id` says what the token says

Every citation recognised in a file counts toward that file, a qualified
`<§><alias>/<ID>` included, listed at its `(line, column)` like any other: it is what the
file leans on, outside a workspace too, and calling its alias unknown is `check`'s job
([§FS-check.3.8](FS-check.md#38-cross-project-citation-failure)), not `cover`'s. The rendered `id` is canonical: `api/FS-login` for a
citation written qualified and the bare ID for a local one, under the **target** project's
`[id]` config as `refs` renders it ([§FS-workspace.8.2.4](FS-workspace.md#824---format-json)), so a row reports what the file
wrote. The target a row names is therefore `id` when it carries a `/`, and
`<project>/<id>` otherwise; `text` remains the verbatim source token either way.

### 8.7 Output and exit codes

Each command exits as its own page says: `show` ([§FS-show.3.4](FS-show.md#34-what-a-failed-query-prints)), `show --batch`
([§FS-show.3.3](FS-show.md#33-batch-query)), `refs` ([§FS-refs.4](FS-refs.md#4-exit-codes)), `list` ([§FS-list.4](FS-list.md#4-exit-codes)), `cover` ([§FS-cover.4](FS-cover.md#4-exit-codes)), the
completion helper ([§FS-completions.2](FS-completions.md#2-internal-dynamic-helper)) and `fmt --cross-refs` ([§FS-fmt](FS-fmt.md#fs-fmt-grund-normalizes-citations-in-bulk)).

#### 8.7.3 Paths

Paths in every command respect `[output] relative_paths` as `check` does. With a workspace
loaded and the default, they render from the workspace root, never from the first member,
`include_root = false` included, so a member's file is spelled the way `[workspace] members`
spells it and joins against the same base `git diff` reports. Under `relative_paths = false`
the base is the path argument or cwd, with in-workspace parent components as [§FS-config.3.6](FS-config.md#36-output--report-format)
specifies. That includes the `error: <path>: <reason>` line a per-file scan failure in any
project earns ([§FS-check.2](FS-check.md#2-outputs)): the same line `check` prints for the same tree, whichever
command met the failure first.

### 8.8 Discovery follows `grund check`'s walk-up

See [§FS-scan.projects.discovery](FS-scan.md#projectsdiscovery-the-nearest-config-decides) and [§FS-scan.path.discovery](FS-scan.md#pathdiscovery-a-path-is-where-discovery-starts).

### 8.9 The current project

The current project, what an unqualified ID resolves against ([§FS-workspace.4](FS-workspace.md#4-resolution)), is the root
project at the workspace root and the member project inside a member tree. When
`include_root = false` and the command is invoked at the workspace root there is none, so a
command that accepts a single ID rejects an unqualified `<ID>` and requires
`<alias>/<ID>`. Cross-project lookups always take the `<alias>/<ID>` form; there is no
`--all-projects` flag, because the alias *is* how the scope is named.

### 8.10 `include_root = false` leaves the root alias unknown

When the root project is excluded from the workspace ([§FS-workspace.2](FS-workspace.md#2-workspace-configuration)), it has no catalog
entry and its alias is not known: `<§>root/<ID>`, or whatever name `project_name` would
have assigned, is an unknown alias to every command in this section, never silently
reserved. Its files leave `cover`'s grouping with its catalog entry, and nothing else scans
them ([§FS-workspace.6.1.2](FS-workspace.md#612-include_root-on-a-block)).

### 8.11 A malformed alias path names the offending segment

A malformed alias path is rejected before the scan, and the finding names the offending
segment rather than the whole path, since the path is one slug per level
([§FS-workspace.1.1](FS-workspace.md#11-the-alias-path)) and the mistake is always in a segment:

```text
error: invalid project alias segment `Alpha` in `group/Alpha` (expected [a-z][a-z0-9-]*, one segment per workspace level)
```

An empty segment (`alpha//TICKET-1234`) has nothing to quote, so the finding says a
segment is empty, and a leading `/` says the path is empty, never quoting empty backticks.
A well-formed alias path that names no project is the unknown-alias error of
[§FS-workspace.8.1.3](FS-workspace.md#813-an-unknown-alias-outside-a-workspace-context) instead. The rule holds for every command that takes a qualified ID,
`grund fetch` included ([§FS-fetch.1](FS-fetch.md#1-input-and-project-selection)): one splitter reads the path, so an argument is
refused with the same words wherever it is typed.
