# FS-imports: a project cites a sibling project its build places beside it, read live from that tree

A config may declare a project its build already placed at a path as an **import**, and cite
its declarations as `<alias>/<ID>`, the qualified form a workspace member answers
([§FS-workspace.1](FS-workspace.md#1-citation-syntax)). The import is read live and read-only from that tree, as a mounted root
under its alias ([§FS-remote-projects.mount.rule](FS-remote-projects.md#mountrule-a-mounted-root-resolves-natively-and-is-shown-under-its-alias)); no commit pins it, and `check` says so. So a
repository cites a project another repository holds from its own config alone, with no shared
parent config, no second revision pin and no committed copy ([§GOAL-small-and-large](../goals.md#goal-small-and-large-start-small-configure-for-big)), and the
citation stays a checked one ([§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration)). The decision is [§DF-imports](../decisions/functional/DF-imports.md#df-imports-an-import-is-a-named-input-read-live-from-a-sibling-tree-never-a-remote).

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

`path` and `project` are required strings, and no other key is allowed
([§FS-imports.failures](FS-imports.md#failures-config-failures)). A `[workspace]` that holds only imports is valid, its projects the
root and its imports, and the consumer stays the current project for an unqualified ID
([§FS-workspace.8.9](FS-workspace.md#89-the-current-project)). A config with imports loads as a workspace does, so `grund list`
qualifies every row ([§FS-workspace.8.3](FS-workspace.md#83-grund-list)).

The table is version 2 only: a version-1 config that declares it is refused as an `unknown
config section`, exit 2, and so is `grund config migrate --to 2` of such a file. A target's
config may be either version. `grund config show` prints the table as written, with no
resolved path. The config record the bindings expose carries `workspace.imports` only where
an import is declared. The managed block `grund init` writes lists no import, so a
`[workspace]` that holds only imports renders the block exactly as a config without
`[workspace]` does.

### declaration.alias: The alias is the table key

An import's alias is its table key, in the alias grammar of [§FS-workspace.1.1](FS-workspace.md#11-the-alias-path), and it shares
the sibling level of [§FS-workspace.3](FS-workspace.md#3-aliases) with the root, the members and the optional members of
its block. A collision with any of them is a config error at the table line, in the words a
member collision uses, which name the root by its role, a member or an optional member by
its entry, and the import by its path, each as written:

```text
error: grund.toml:7: duplicate workspace project alias `native-image` (workspace member `tools/native-image` and import `../graal/substratevm`)
```

The alias is the consumer's name for the target and may differ from the target's
`project_name`, which the target still answers to inside the mount
([§FS-remote-projects.mount.rule](FS-remote-projects.md#mountrule-a-mounted-root-resolves-natively-and-is-shown-under-its-alias)). Where an enclosing workspace claims the consumer, its
imports are named beneath the consumer's segment from the outermost root, as its members are
([§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)).

### declaration.path: The path is the consumer's, relative and literal

`path` is relative to the consumer's config root, written with `/`, and may use `..`. Nothing
in it is expanded: a glob, `~`, an environment variable or a URL is a directory name like any
other. A path that is absolute or holds a backslash, on every platform, or that resolves to
the consumer's own config root, to a directory containing it or to one inside it, is refused
([§FS-imports.failures](FS-imports.md#failures-config-failures)). The tests that resolve the path use its canonical form, and two
spellings of one canonical root are one target.

### declaration.root: The target's own config, at exactly that root

The target must hold its own config at exactly the directory `path` names: discovery's
precedence chooses between the two names there ([§FS-config.1.1](FS-config.md#11-when-one-directory-carries-both)), and nothing walks upward
from the target to find one, so a workspace of the target's own repository that claims it
does not reach the consumer.

### declaration.identity: The project is named, never inferred

The target's config must declare `project_name`, and it must equal the import's `project`.
A target that declares none, or another one, is a config error at the `project` line, even
where the wrong directory happens to declare every cited ID: the mapping from an alias to a
project is authored, never inferred ([§REQ-no-wrong-citation](../requirements/REQ-no-wrong-citation.md#req-no-wrong-citation-a-citation-never-resolves-to-a-guess)).

## mount: An import is a mounted root, read live and read-only

An import is loaded as a mounted root under its alias ([§FS-remote-projects.mount.rule](FS-remote-projects.md#mountrule-a-mounted-root-resolves-natively-and-is-shown-under-its-alias)), with
its own config, scan, grammar, sections, values and rules. It is read from the target's tree
as that tree stands when the run reads it, and read-only from the consumer
([§FS-imports.writes](FS-imports.md#writes-the-consumer-writes-only-its-own-files)): a run whose path is the target is the target's standalone run, and
writes as any run does.

It is walked with the ignore files at and below the target's root and none above it
([§FS-remote-projects.ignore](FS-remote-projects.md#ignore-ignore-isolation)), so a file the dependency's own run hides through an ignore file
above its root, `.git/info/exclude` or a global `core.excludesFile` shows through the import;
the fix is an ignore file at or below the dependency's root.

## graph: The import graph

### graph.transitive: Imports nest beneath their importer

A dependency's own imports are loaded beneath its alias, as its members are: `native-image`
importing `org` gives `native-image/org`. Two different trees that share a `project_name`
under two importers are two projects, `a/shared` and `b/shared`
([§FS-remote-projects.mount.rule](FS-remote-projects.md#mountrule-a-mounted-root-resolves-natively-and-is-shown-under-its-alias)).

### graph.cycle: A cycle is a config error

An import whose target's canonical root is already a root on the chain of imports that
reached it is a config error at the table that closes the cycle, with no `imported as` note
([§FS-imports.failures](FS-imports.md#failures-config-failures)). It names the chain from the consumer, by the consumer's
`project_name` and then the alias each importer gave:

```text
error: ../graal/substratevm/grund.toml:4: import cycle: enterprise → native-image → enterprise
```

### graph.diamond: One tree reached under two alias paths

Two import edges whose targets share one canonical root load that tree once, under one alias
path per route: a consumer importing `lib-a` and `lib-b`, which each import `core`, reaches
`core` as `lib-a/core` and `lib-b/core`. `list` prints a row for each alias path, at the same
file and line with the same `refs` count, and a query or `refs` of either alias path gives
the same body or sites. `refs` and `cover` report each citation site once,
and `cover` each file once, under the file's canonical alias path, which their JSON carries
as `project`; `check` reports nothing twice, and a declaration reached under two alias paths
is not a `duplicate`.

The canonical alias path has the fewest segments, and among those comes first in byte order,
`lib-a/core` above, whatever order a config lists its imports or a run reads them in. A file
an enclosing workspace also lists as an ordinary member is reported once by the same rule:
the member's findings are reported, and the import's are suppressed.

### graph.reverse: A dependency never learns its consumer

A dependency's standalone run reads nothing of any consumer and answers exactly as it does
when no consumer exists. A directory no import names is never searched for a matching alias,
ID or `project_name`, and a missing import never falls back to one ([§REQ-no-wrong-citation](../requirements/REQ-no-wrong-citation.md#req-no-wrong-citation-a-citation-never-resolves-to-a-guess)),
so a run's effective inputs ([§FS-check.6.1](FS-check.md#61-change-detection)) are the same whether a directory no import names
is present or gone.

## findings: Dependency findings are suppressed, consumer citations are not

In a consumer run, a content finding located inside an import is suppressed in every channel,
text, JSON and the editor ([§FS-remote-projects.differences](FS-remote-projects.md#differences-the-closed-list-of-differences)), and so is every
caution the dependency's own config earns, such as `config-version-1` or
`deprecated-config-location`; its standalone run reports them. Suppression removes no data:
`list`, `refs`, `cover` and their counts include each import. It lets no consumer citation
pass ([§FS-remote-projects.differences](FS-remote-projects.md#differences-the-closed-list-of-differences)): a dependency defect that breaks one,
such as a duplicate ID the consumer cites, fails at the consumer's citation site. And
`import-live` and `import-missing` are never suppressed, even where a transitive import puts
them in a dependency's config.

## paths: Imported files are reported where they are

A file of an import is reported at its own path from the run's report base, never absolute
and never relocated under its alias: `../graal/substratevm/docs/functional-spec/FS-x.md` from
the consumer's config root ([§FS-workspace.8.7.3](FS-workspace.md#873-paths)). A cross-ref link `fmt` writes into an import
points at that file, relative to the consumer document that holds it ([§FS-workspace.8.5](FS-workspace.md#85-grund-fmt---cross-refs)), so
it works in a checkout and in an editor, not on a forge page ([§DF-imports.3](../decisions/functional/DF-imports.md#3-consequences)).

## writes: The consumer writes only its own files

Writers walk only the consumer's own projects: `fmt --write`, `fmt --check`,
`fmt --cross-refs` and `init` never visit, rewrite or list a file of an import. The
section-lock entries for citations into an import are the consumer's own, under its
`.grund/sections/<alias>/`, and a run with the import missing treats them as it treats those
of an alias it does not load ([§FS-section-lock.file.members](FS-section-lock.md#filemembers-citations-into-another-project)). `grund fetch` of an ID in an
import is refused before its integration runs, with exit 2, naming the target as the place to
run it ([§FS-imports.writes.answer](FS-imports.md#writesanswer-where-a-qualified-id-is-fetched-has-one-answer)):

```text
error: `native-image/TICKET-1` is in import `native-image`, read-only here — run grund fetch TICKET-1 in ../graal/substratevm
```

### writes.answer: Where a qualified ID is fetched has one answer

`grund fetch`'s refusal and the fetch tails of `grund check` ([§FS-check.3.1](FS-check.md#31-dangling-citation),
[§FS-check.4.12](FS-check.md#412-missing-snapshot)) render from one answer to where the fetch of a qualified ID runs, so
following either takes one command. The project is selected as the ID query selects it
([§FS-fetch.1](FS-fetch.md#1-input-and-project-selection)), reading the import edges on the alias path, nested ones and a member's
included, and every edge the run can reach only for an alias nothing answers. A fetch of a
local or member ID reads no import it does not name, so an import that cannot be read does
not fail it. The first of these that holds is the answer:

- **An alias path no project has** gets the refusal `grund show` prints for it, and **one
  into a missing import** the query refusal of [§FS-imports.checks.import-missing](FS-imports.md#checksimport-missing-an-import-names-no-directory). Neither
  names a command or says read-only, and both exit 2.
- **A workspace member's ID** is fetched from the root ([§FS-check.4.12](FS-check.md#412-missing-snapshot)).
- **An ID reached through an import** is fetched in the target of the last import edge on
  its alias path, spelled from the run's report base ([§FS-errors.4](FS-errors.md#4-determinism)), as the alias path below
  that edge: where `lb/in` is the last import, `lb/in/alpha/FS-a` is
  `run grund fetch alpha/FS-a in ../inner`, and `grund fetch` refuses it naming that import,
  `lb/in`.

Whether a command is named is asked of the project the ID is in, and where that project
refuses the fetch itself, neither surface names one. An ID it cannot parse, one of a kind it
does not declare among them, is `invalid ID`, exit 1, and a kind with no `fetch` is
``kind `<KIND>` has no fetch integration``, exit 2 ([§FS-fetch.1](FS-fetch.md#1-input-and-project-selection)). At a home `fetch` refuses,
`grund fetch` prints the refusal of [§FS-fetch.4](FS-fetch.md#4-file-home-write) or [§FS-fetch.4.2](FS-fetch.md#42-a-home-read-as-source-is-refused), exit 2, and `grund check`
the tail of [§FS-check.3.1](FS-check.md#31-dangling-citation).

One named command is refused. For a kind whose key has nothing to run in the target
([§FS-fetch.2.1](FS-fetch.md#21-a-value-that-names-no-executable)), `grund check` gives the tail of [§FS-check.3.1](FS-check.md#31-dangling-citation), which names no command, but
`grund fetch` keeps its read-only refusal, which stands in front of the key's test, so the
target refuses the command that refusal names, with the repair ([§DF-blank-fetch-check-tail.3](../decisions/functional/DF-blank-fetch-check-tail.md#3-consequences)).

## editor: The editor reads the import graph the command line reads

`grund-lsp` answers through imports from the graph `check` reads ([§REQ-one-answer](../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree)): hover,
completion, definition, references, highlights and document links resolve through them, and
completion offers each import's alias and every alias reached through it, as `complete ids`
offers an alias path ([§FS-workspace.8.4.1](FS-workspace.md#841-a-prefix-containing-)). An import needs no folder of its own: a session
holding only the consumer's folder, or only a folder inside it whose upward walk finds the
consumer's config, answers through every import, and an unsaved buffer of an imported file is
that file's text for the consumer, as any buffer is ([§FS-scan.identity.buffers](FS-scan.md#identitybuffers-a-buffer-is-the-file-at-its-physical-location)).

- **Watching.** The server follows each import's tree, config and ignore files outside every
  open folder, the effective inputs `check --watch` follows ([§FS-check.6.1](FS-check.md#61-change-detection)), through
  watchers it registers with the client where the client can and its own where it cannot.
  Its own watcher makes a burst of changes one rebuild, bounded as `check --watch` bounds its
  debounce ([§FS-check.6.1](FS-check.md#61-change-detection)), and the server answers a request that arrives meanwhile from the
  current snapshot, so a tree that never stops changing neither postpones the rebuild nor
  holds a request in any project.
- **Ownership.** A document under an import root that no folder's project owns is answered by
  that import's own project, read-only, with no diagnostics and no edits, loaded once per
  canonical root and shared by every consumer that imports it, so two consumers in one session
  are not two claims ([§FS-lsp.2.2.2](FS-lsp.md#222-one-project-answers-each-document)).
- **Failures.** A dependency that disappears rebuilds the consumer without it, with
  `import-missing` on the import line. A dependency config that will not load leaves the
  consumer on its last good scan, with the error on the import line
  ([§FS-lsp.requirements.config-errors](FS-lsp.md#requirementsconfig-errors-a-config-mistake-reaches-the-editor-as-loudly-as-the-command-line)).

## checks: Checks

Each section below is one finding code about the imports a run reads, whose row in
[§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog) carries its severity and fix, located and rendered as [§FS-check.2.1](FS-check.md#21-report-format) says, with the
exit status of [§FS-check.2](FS-check.md#2-outputs).

### checks.import-live: An import is read live, unpinned

Each import edge the run reads is reported once, at the table line of the config that
declares it:

```text
grund.toml:4: warning: import `native-image` is read live from `../graal/substratevm`, unpinned
```

It names the import by the alias path the run spells it with and the path as its config wrote
it, and nothing else: no output of `check`, a query, `list`, `refs`, `cover`, `fmt` or `id`
carries a commit, a dirty state, a clock or any other provenance. It is a located finding on
stdout rather than a CLI-level line, for the reason [§FS-check.4.9](FS-check.md#49-a-workspace-member-declared-optional-is-absent) gives an absent optional
member, and no selector hides it ([§FS-check.2.1.2](FS-check.md#212-selection-filters-the-complete-report)), so a run that read an import always says
so and never prints `success` ([§DF-imports.3](../decisions/functional/DF-imports.md#3-consequences)); the exit code is the one the rest of the report
gives.

### checks.import-missing: An import names no directory

An import whose `path` names no directory is reported once, at its table line, and the run
exits `1`:

```text
grund.toml:4: error: import `native-image` is missing: `../graal/substratevm` is not a directory — citations into namespace `native-image` were not checked
```

Every citation into it is unverified, as one into an absent optional member is
([§FS-workspace.4](FS-workspace.md#4-resolution)), and it absorbs every alias path that begins with its own
([§FS-workspace.2.2](FS-workspace.md#22-a-member-that-may-be-legitimately-absent)), so no finding is reported at those sites. No selector hides it
([§FS-check.2.1.2](FS-check.md#212-selection-filters-the-complete-report)), so a run whose import is missing never exits `0` ([§DF-imports.2.3](../decisions/functional/DF-imports.md#23-what-an-absent-import-gives)). A
query into it fails with exit 2:

```text
error: import `native-image` is missing: `../graal/substratevm` is not a directory
```

## failures: Config failures

Every failure below is a config error ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)), located at the line the table names,
in the config that declares the import. A failure in a target's own config, anything the
target's standalone run would refuse, is the `error:` line that run prints, its path spelled
from the report base, followed by one `note:` line naming the import that reached the
target, by the alias path the run spells it with and the table that declares it
([§FS-errors.1](FS-errors.md#1-streams)):

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
