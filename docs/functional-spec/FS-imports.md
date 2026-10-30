# FS-imports: a project cites a sibling project its build places beside it, read live from that tree

A config may declare a project that already sits at a path its build placed it as an
**import**, and cite that project's declarations as `<alias>/<ID>`, the same qualified form
a workspace member answers ([§FS-workspace.1](FS-workspace.md#1-citation-syntax)). The import is read live from that tree,
read-only, as a mounted root under its alias ([§FS-remote-projects.mount.rule](FS-remote-projects.md#mountrule-a-mounted-root-resolves-natively-and-is-shown-under-its-alias)); no commit pins
it, and `check` says so. That lets a repository cite a project another repository holds from
its own config alone, on the command line and in the editor, with no shared parent config, no
second revision pin and no committed copy ([§GOAL-small-and-large](../goals.md#goal-small-and-large-start-small-configure-for-big)), while every
cross-repository citation stays a checked one ([§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration)). The decision is
[§DF-imports](../decisions/functional/DF-imports.md#df-imports-an-import-is-a-named-input-read-live-from-a-sibling-tree-never-a-remote).

The command line ships it whole: the table is read, every import whose path names a
directory is mounted under its alias ([§FS-imports.mount](FS-imports.md#mount-an-import-is-a-mounted-root-read-live-and-read-only)), each failure of [§FS-imports.failures](FS-imports.md#failures-config-failures) is
refused where it says, and the two checks, selection, the writers and `fetch` treat an import
as this spec says. The editor ships beside it, answering through every import ([§FS-imports.editor](FS-imports.md#editor-the-editor-reads-the-import-graph-the-command-line-reads)). A config that declares no import reads nothing this spec names
([§FS-imports.declaration.none](FS-imports.md#declarationnone-a-config-with-no-imports-reads-nothing-new)).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, body, section, catalog), [§FS-terms.terms.2](FS-terms.md#terms2-citations)
(citation, qualified citation, citation site), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, config root,
workspace, member, alias), [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, verdict), and
[§FS-remote-projects.terms](FS-remote-projects.md#terms-terms) (mounted root, standalone run).

- **import** — A project held outside the consumer's tree, declared by a
  `[workspace.imports.<alias>]` table and read live from the directory its `path` names.
- **consumer** — The project whose config declares an import.
- **target** — The directory an import's `path` names, and the config at its root.
- **dependency** — The project an import loads at its target, with everything it imports in
  turn.
- **import edge** — One import table: the config that declares it, its alias and its target.
- **canonical alias path** — Of a file reached under several alias paths: the one a surface
  that reports each site once reports it under ([§FS-imports.graph.diamond](FS-imports.md#graphdiamond-one-tree-reached-under-two-alias-paths)).

## declaration: Declaration

### declaration.table: The import table

An import is declared in the workspace block of a version-2 config:

```toml
[workspace.imports.native-image]
path = "../graal/substratevm"
project = "native-image"
```

`path` and `project` are both required strings, and any other key in the table is a config
error ([§FS-imports.failures](FS-imports.md#failures-config-failures)). The table needs nothing beside it: a `[workspace]` that holds
only imports is valid, its projects are the root and its imports, and the consumer stays the
current project for an unqualified ID ([§FS-workspace.8.9](FS-workspace.md#89-the-current-project)). A config with imports loads as a
workspace does, so `grund list` qualifies every row ([§FS-workspace.8.3](FS-workspace.md#83-grund-list)).

The table is version 2 only. A version-1 config that declares it keeps today's refusal,
`unknown config section`, with exit 2, and `grund config migrate --to 2` maps nothing to it
and refuses such a file the same way. A target's config may be either version: a version-1
target is read as version 1 and imported as any target is.

`grund config show` prints the table as written, with no resolved path, and `grund config
explain` answers for the table and for both keys. The config record the bindings expose
carries `workspace.imports` only where an import is declared, so a config without one keeps
its bytes there too. The managed block `grund init` writes does not list imports: a
`[workspace]` that holds only imports renders it exactly as a config without `[workspace]`
does.

### declaration.alias: The alias is the table key

An import's alias is its table key and must match the alias grammar of [§FS-workspace.1.1](FS-workspace.md#11-the-alias-path). It
shares the sibling level of [§FS-workspace.3](FS-workspace.md#3-aliases) with the root, the members and the optional
members of the same block, and once they ship with its mounted members and remotes. A
collision with any of them is a config error at the table line, in the words a member
collision uses today: the root by its role, a member or an optional member by its entry,
and the import by its path, each as written. With a member `tools/native-image` whose
`project_name` is `native-image`:

```text
duplicate workspace project alias `native-image` (workspace member `tools/native-image` and import `../graal/substratevm`)
```

So an import can never stand in for a remote. The alias may differ from the target's
`project_name`: it is the consumer's name for the target, and inside the mount the target
still answers to its own ([§FS-remote-projects.mount.rule](FS-remote-projects.md#mountrule-a-mounted-root-resolves-natively-and-is-shown-under-its-alias)). Where an enclosing workspace
claims the consumer, the consumer's imports are named under the consumer's segment from the
outermost root, as its members are ([§FS-workspace.6.1.5](FS-workspace.md#615-alias-paths-do-not-change-with-scope)).

### declaration.path: The path is the consumer's, relative and literal

`path` is relative to the consumer's config root, the directory discovery found the config
for, including when the config lives under `.agents/`; it is never relative to `.agents/`
itself. It is written with `/` and may use `..`. Nothing in it is expanded: a glob, `~`, an
environment variable or a URL is a directory name like any other. It is a config error at
the `path` line when it:

- is absolute, or holds a backslash, on every platform;
- resolves to the consumer's own config root;
- resolves to a directory that contains the consumer's config root;
- resolves to a directory inside the consumer's config root, which is what a member is for.

Each test is made on the canonical path, so a symlink is followed exactly as far as the
directory whose identity is then checked, and the target's own scope rules decide what is
read beneath it ([§FS-scan.scope.links](FS-scan.md#scopelinks-a-symlink-in-the-tree-is-followed)). Two spellings of one canonical root are one target.

### declaration.root: The target's own config, at exactly that root

The target must hold its own config at exactly the directory `path` names. Discovery's
precedence chooses between the two names there, a bare `grund.toml` before
`.agents/grund.toml` ([§FS-config.1.1](FS-config.md#11-when-one-directory-carries-both)), and nothing walks upward from the target to find
one.

A target its own repository claims is mounted all the same as its own outermost root: with
`graal/grund.toml` listing `substratevm` in its `members`, an import of
`../graal/substratevm` reads no config above `substratevm`. Neither that claim nor the other
members of the block that makes it reach the consumer, and the target's citations into those
siblings are the target's own findings, suppressed in a consumer run
([§FS-imports.findings](FS-imports.md#findings-dependency-findings-are-suppressed-consumer-citations-are-not)).

### declaration.identity: The project is named, never inferred

The target's config must declare `project_name`, and it must equal the import's `project`.
A target that declares none, or another one, is a config error at the `project` line, even
where the wrong directory happens to declare every cited ID: the mapping from an alias to a
project is authored, never inferred ([§REQ-no-wrong-citation](../requirements/REQ-no-wrong-citation.md#req-no-wrong-citation-a-citation-never-resolves-to-a-guess)).

### declaration.none: A config with no imports reads nothing new

A config that declares no `[workspace.imports]` table reads nothing outside its tree it did
not read before, and prints no byte it did not print before. A sibling directory is never
searched, not even one whose `project_name` matches the alias a citation spells: that
citation keeps today's `unknown project alias` finding and exit 1, and the run's effective
inputs ([§FS-check.6.1.3](FS-check.md#613-effective-input-inventory)) are the same with the sibling present or gone.

## mount: An import is a mounted root, read live and read-only

An import is loaded as a mounted root under its alias ([§FS-remote-projects.mount.rule](FS-remote-projects.md#mountrule-a-mounted-root-resolves-natively-and-is-shown-under-its-alias)): its own
config, scan, grammar, sections, values and rules; its own members and imports named beneath
its alias; nothing of the consumer visible inside it; and no claim from above reaching in. It
is read from the target's tree as that tree stands when the run reads it, and it is
read-only ([§FS-imports.writes](FS-imports.md#writes-the-consumer-writes-only-its-own-files)).

It is walked with the ignore files at and below the target's root, whether or not the target
is a git checkout, and with none above it ([§FS-remote-projects.ignore](FS-remote-projects.md#ignore-ignore-isolation)): no `.gitignore` or
`.ignore` of the target's repository above the target, no `.git/info/exclude`, no global
`core.excludesFile`, and none of the consumer's. A dependency whose own run hides a file
through one of those shows that file through the import; the fix belongs in the dependency,
as an ignore file at or below its root.

A run narrowed to a path inside the consumer still resolves through its imports, and a run
whose path is the target, `grund check ../graal/substratevm`, is the target's standalone
run. A member run of one of the consumer's own members loads neither the block's imports
nor its siblings ([§FS-workspace.5.1](FS-workspace.md#51-a-member-run)).

## graph: The import graph

### graph.transitive: Imports nest beneath their importer

A dependency's own imports are loaded beneath its alias, as its members are: `native-image`
importing `org` gives `native-image/org`, and a member `compiler` of that import gives
`native-image/org/compiler`. Only declared imports are followed. Two different trees that
share a `project_name` under two importers are two projects, `a/shared` and `b/shared`, as
two pins of one remote are ([§FS-remote-projects.mount.nested](FS-remote-projects.md#mountnested-nested-pins-stay-isolated)).

### graph.cycle: A cycle is a config error

An import whose target's canonical root is already a root on the chain of imports that
reached it closes a cycle. That is a config error, located at the import that closes it,
which names the chain from the consumer, the consumer by its `project_name` and each project
after it by the alias its importer gave it. The chain already names every import that
reached the closing table, so no `imported as` note follows it
([§FS-imports.failures](FS-imports.md#failures-config-failures)):

```text
error: ../graal/substratevm/grund.toml:4: import cycle: enterprise → native-image → enterprise
```

### graph.diamond: One tree reached under two alias paths

Two import edges whose targets share one canonical root load that tree once, and it is
reached under one alias path per route: a consumer importing `lib-a` and `lib-b`, each of
which imports `core`, reaches `core` as `lib-a/core` and `lib-b/core`. It is one tree and
one set of files, so each surface says per alias path only what is about the alias path:

- `list` prints one row per alias path, `lib-a/core/FS-x` and `lib-b/core/FS-x`, at the same
  file and line with the same `refs` count.
- `grund lib-a/core/FS-x` and `grund lib-b/core/FS-x` print the same body, and
  `refs lib-a/core/FS-x` and `refs lib-b/core/FS-x` list the same sites.
- `refs` and `cover` report each citation site once, and `cover` each file once, under the
  file's canonical alias path, which is also the `project` their JSON objects carry.
- `check` reports nothing twice, and a declaration reached under two alias paths is not a
  `duplicate`: it is one declaration, in one file.

The canonical alias path is the one with the fewest segments, and among those the first in
byte order: `lib-a/core` above. It depends only on the declared configs, never on the order
a config lists its imports or a run reads them. The same rule names the one report of a file
an enclosing workspace also lists as an ordinary member: the member and the import are two
alias paths of one tree, the member's findings are reported, the import's are suppressed,
and nothing is reported twice.

### graph.reverse: A dependency never learns its consumer

A dependency's standalone run reads nothing of any consumer and answers exactly as it does
when no consumer exists. Inside a consumer run, nothing of the consumer is visible from the
dependency. A directory no import names is never searched for a matching alias, ID or
`project_name`, and a missing import never falls back to one ([§REQ-no-wrong-citation](../requirements/REQ-no-wrong-citation.md#req-no-wrong-citation-a-citation-never-resolves-to-a-guess)).

## findings: Dependency findings are suppressed, consumer citations are not

In a consumer run, a content finding located inside an import is suppressed in every
channel, text, JSON and the editor, as a remote's is
([§FS-remote-projects.differences.suppression](FS-remote-projects.md#differencessuppression-remote-owned-content-findings-are-suppressed)), and so is every caution the dependency's own
config earns, such as `config-version-1` or `deprecated-config-location`. A run whose path is
the dependency reports them, because that is its standalone run. Suppression never removes
data: `list`, `refs`, `cover` and their counts include each import.

Two rules keep suppression honest, as they do for a remote
([§FS-remote-projects.differences.consumer-site](FS-remote-projects.md#differencesconsumer-site-suppression-never-lets-a-consumer-citation-pass)). A dependency defect that breaks a consumer
citation, such as a duplicate ID the consumer cites, fails at the consumer's citation site.
And `import-live` and `import-missing` are never suppressed, including where a transitive
import puts them in a dependency's config ([§FS-imports.checks](FS-imports.md#checks-checks)).

## paths: Imported files are reported where they are

A file of an import is reported at its own path, relative to the run's report base, which is
the consumer's config root by default: `../graal/substratevm/docs/functional-spec/FS-x.md`.
It is never absolute and never relocated, and under `relative_paths = false` its base is the
one every command uses ([§FS-workspace.8.7.3](FS-workspace.md#873-paths)). A cross-ref link `fmt` writes into an import
points at that file, relative to the consumer document that holds it ([§FS-workspace.8.5](FS-workspace.md#85-grund-fmt---cross-refs)): it
works in a checkout and in an editor, not on a forge page.

## writes: The consumer writes only its own files

Writers walk only the consumer's own projects: `fmt --write`, `fmt --check`,
`fmt --cross-refs` and `init` never visit, rewrite or list a file of an import. A write into
the consumer's own files computed from an import's facts is ordinary: a cross-ref link from a
consumer document into the imported file, and the consumer's section-lock entries under its
own `.grund/sections/<alias>/` ([§FS-section-lock.file.members](FS-section-lock.md#filemembers-citations-into-another-project)). A run with a missing import
leaves the consumer's `.grund/sections/<alias>/` entries for it as they are, as it leaves an
alias the run does not load.

`grund fetch` of an ID in an import is refused before its integration runs, with exit 2,
naming the target as the place to run it:

```text
error: `native-image/TICKET-1` is in import `native-image`, read-only here — run grund fetch TICKET-1 in ../graal/substratevm
```

Read-only belongs to the import edge, not to the tree: run from the dependency itself, every
write is ordinary.

## editor: The editor reads the import graph the command line reads

`grund-lsp` answers through imports from the graph `check` reads ([§REQ-one-answer](../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree)): hover,
completion, definition, references, highlights and document links resolve through them, and
completion offers each import's alias and every alias reached through it from the same alias
list, by the rule `complete ids` offers an alias path by ([§FS-workspace.8.4.1](FS-workspace.md#841-a-prefix-containing-)). An import
needs no folder of its own: a session holding only the consumer's folder, or only a folder inside it whose
upward walk finds the consumer's config, answers through every import.

- **Watching.** The server follows each import's tree, config and ignore files that lie
  outside every open folder, from the effective inputs `check --watch` records
  ([§FS-check.6.1.3](FS-check.md#613-effective-input-inventory)): through watchers it registers with the client where the client can,
  and through its own watcher where it cannot. `check --watch` follows the same inputs.
  The server's own watcher makes a burst of changes one rebuild, bounded as `check --watch`
  bounds its debounce ([§FS-check.6.1.2](FS-check.md#612-bounded-debounce-and-serialized-work)), and a request that arrives while that rebuild is
  pending is answered from the current snapshot: a tree that never stops changing neither
  postpones the rebuild nor holds a request in any project behind it.
- **Unsaved buffers.** An unsaved buffer of an imported file is that file's text for the
  consumer, under whichever spelling the editor opened it ([§FS-scan.identity.buffers](FS-scan.md#identitybuffers-a-buffer-is-the-file-at-its-physical-location)).
- **Ownership.** A document under an import root that no folder's project owns is answered by
  that import's own project, read-only, loaded once per canonical root and shared by every
  consumer that imports it, so two consumers in one session are not two claims
  ([§FS-lsp.2.2.3](FS-lsp.md#223-a-document-no-project-answers)). It gets no diagnostics, because its findings are the dependency's, and it
  is offered no edits. Opened as its own folder, the dependency owns its files and edits
  them normally.
- **Failures.** A dependency that disappears rebuilds the consumer without it: navigation into
  it is gone, and `import-missing` sits on the import line. A dependency config that will
  not load leaves the consumer on its last good scan, with the error on the import line
  ([§FS-lsp.requirements.config-errors](FS-lsp.md#requirementsconfig-errors-a-config-mistake-reaches-the-editor-as-loudly-as-the-command-line--realized)).

## checks: Checks

Each section below is one check `grund check` enforces about an import, and its name is the
finding code verbatim, as [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog) publishes it and [§REQ-spec-section-names.code](../requirements/REQ-spec-section-names.md#code-a-check-is-named-by-its-diagnostic-code)
requires; each code's row in that catalog carries its severity. Neither `--only` nor
`--ignore` hides either code: both stay in the report whatever the selectors, as an `io`
finding does ([§FS-check.2.1.2](FS-check.md#212-selection-filters-the-complete-report)), so a run whose import is missing never exits `0` and a run
that read an import always says so. Being unhideable changes no exit code.

### checks.import-live: An import is read live, unpinned

Once per import edge the run reads, a transitive one included, one warning is located at the
table line of the config that declares it:

```text
grund.toml:4: warning: import `native-image` is read live from `../graal/substratevm`, unpinned
```

It names the import by the alias path this run spells it with and the path as its config
wrote it, and nothing else: its text depends only on the config, never on a commit, a dirty
state or a clock, and no output of `check`, a query, `list`, `refs`, `cover`, `fmt` or `id`
carries any other provenance. It is on stdout for the reason [§FS-check.4.9.5](FS-check.md#495-the-exit-code-forces-the-shape) gives: exit `0`
cannot say that the verdict depends on a tree no commit pins. Its JSON form is one ordinary
warning object:

```json
{"severity":"warning","path":"grund.toml","line":4,"code":"import-live","message":"import `native-image` is read live from `../graal/substratevm`, unpinned","sites":null,"authority":null}
```

Because it is a warning, `success` is never printed while an import is declared, and the
exit code is the one the rest of the report gives.

### checks.import-missing: An import names no directory

An import whose `path` names no directory is one error at its table line, and the run exits
`1`:

```text
grund.toml:4: error: import `native-image` is missing: `../graal/substratevm` is not a directory — citations into namespace `native-image` were not checked
```

Every citation into a missing import is unverified, as a citation into an absent optional
member is ([§FS-workspace.4.3](FS-workspace.md#43-unverified-is-reported-once-per-member)), and the import absorbs every alias path that begins with its
own ([§FS-workspace.2.2.2.3](FS-workspace.md#2223-an-absent-member-absorbs-the-alias-paths-beneath-it)): no `unknown project alias`, `dangling` or other finding is
reported at those sites, and nothing is searched for in the import's place. A query into it
fails with exit 2, naming it: `error: import `native-image` is missing:
`../graal/substratevm` is not a directory`.

## failures: Config failures

Every failure below is a config error. The run stops before any report, prints `error:
<path>:<line>: <message>` on stderr, and exits 2, the exit of an incomplete run
([§FS-check.2.4](FS-check.md#24-an-incomplete-run)); none is a selectable code. A failure about an import is located in the
config that declares it, at the line named below. A failure inside a target's own config,
whether it cannot be parsed, has an unsupported version or breaks any rule a standalone run
would refuse, is located at the target's file and line, at the report base, as a nested
member's config error is today. The `error:` line is the one the target's standalone run
prints, with its path taken from the report base, and one `note:` line after it names the
import that reached the target, by the alias path the run spells it with and the table that
declares it, as a recovery breadcrumb ([§FS-errors.1.2](FS-errors.md#12-what-stderr-carries)):

```text
error: ../graal/substratevm/grund.toml:3: unknown key `strcit` at the top level (expected grund_config_version or project_name)
note: imported as `native-image` from grund.toml:4
```

| Failure | Line | Message |
|---|---|---|
| unknown key | the key | ``unknown key `ref` in [workspace.imports.native-image] (expected path or project)`` |
| missing key | the table | ``[workspace.imports.native-image] needs `project` `` |
| a key that is not a string | the key | `expected string` |
| absolute or backslashed path | `path` | ``invalid import path `/srv/graal` for `native-image` (expected a relative path written with /)`` |
| the consumer itself | `path` | ``import `native-image` resolves to the config root that declares it`` |
| a directory containing the consumer | `path` | ``import `native-image` resolves to `..`, which contains the config root that declares it`` |
| a directory inside the consumer | `path` | ``import `native-image` resolves inside the config root that declares it; list it in [workspace] members instead`` |
| an alias collision | the table | as [§FS-imports.declaration.alias](FS-imports.md#declarationalias-the-alias-is-the-table-key) shows |
| no config at the target | the table | ``import `native-image`: `../graal/substratevm` holds no grund.toml or .agents/grund.toml`` |
| a target that cannot be read | the table | ``import `native-image`: cannot read `../graal/substratevm`: <reason>`` |
| no `project_name` at the target | `project` | ``import `native-image` expects project `native-image`, but `../graal/substratevm` declares no project_name`` |
| another `project_name` | `project` | ``import `native-image` expects project `native-image`, but `../graal/substratevm` declares `svm` `` |
| a cycle | the closing table | as [§FS-imports.graph.cycle](FS-imports.md#graphcycle-a-cycle-is-a-config-error) shows |
