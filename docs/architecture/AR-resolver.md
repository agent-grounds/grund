# AR-resolver: how a run loads every project and resolves a citation to one of them

A workspace is two halves. Reading the configs — expanding one `members` list,
climbing the claims that spell an alias path, narrowing a scope, computing the
boundary roots a scan stops at — is [§AR-workspace](AR-workspace.md#ar-workspace-how-the-config-time-workspace-layer-composes-with-the-config-loader-and-the-scanner), and it happens
before any file is read. *Loading* those projects is this page: scanning every
one of them, reconciling off-grammar citations across the whole set, and
answering the questions that are functions of what a run loaded and of nothing
else ([§FS-workspace.8](../functional-spec/FS-workspace.md#8-other-commands)).

Because it runs scans, it sits **above** the scanner while the half that reads
configs stays below it. That is the whole reason the component exists: while
both halves were one box, the lower half reached up through the scanner for the
walk, the scan errors and the ID-argument resolver, and three components above
reached up past each other for a body slice and a link target that are neither a
rule nor a write ([§AR-system.4](README.md#4-dependency-direction)). The test for
which half a question belongs to is one question: does the answer need loaded
findings, or only configs?

## placement: Where the resolver sits

```text
workspace ─► project map ─┐
                          ▼
scanner ─► Findings ─► [ resolver ] ─┬─► loaded project set ─► checker, queries, writers
                                     ├─► citation ──► target project
                                     ├─► declaration ──► body by recorded span
                                     └─► declaration ──► link target
```

The tenth box of the pipeline ([§AR-system.2.10](README.md#210-resolver)). It takes the
project map workspace expanded out of configs ([§AR-system.2.4](README.md#24-workspace)) and each
project's `Findings` from the scanner ([§AR-system.2.5](README.md#25-scanner)), and gives the
checker, the queries and the writers the loaded project set with the four
answers above ([§AR-system.2.6](README.md#26-checker), [§AR-system.2.7](README.md#27-queries), [§AR-system.2.8](README.md#28-writers)). It
knows no rule and no rendering: it settles which project a coordinate lands in
and what text is there, never whether that is an error or how to print it
([§AR-system.4](README.md#4-dependency-direction)).

## 1. The resolver: one function

`target_for_citation(cite, local, local_config, workspace)` in
`resolver/citation_target.rs` is the single function any command calls to map a
citation to the project it resolves against — that project's `Findings` and the
`Config` its ID is parsed and rendered with ([§AR-workspace.2](AR-workspace.md#2-single-citation-grammar)):

- `cite.namespace == None` → resolves against `local` (the current project).
- `cite.namespace == Some(name)` → resolves against `workspace[name]`, or
  `None` if the alias is unknown.

Every consumer of citations — the checker (dangling, missing-section,
ungrounded), and any future qualified `show` / `refs` / `list` — must go
through this function rather than match on `citation.namespace` itself. The bug
shape it rules out is a command that learns qualified IDs but resolves them
slightly differently from `check`, leaving the editor jump and the CI verdict
out of sync.

A `None` return value at the resolver is never a silent skip; the calling
rule turns it into a located diagnostic (`unknown project alias <name>` at
the citation site). Section 2 is that consequence in full.

## 2. Standalone members fail loud, not silent

A `grund check` invoked at a member root cannot resolve qualified citations
to the workspace or to siblings — there is no project map. Per
[§DF-subproject-namespaces](../decisions/functional/DF-subproject-namespaces.md#df-subproject-namespaces-alias-namespace-model-for-sub-projects-and-external-repos) §3.6 and [§FS-workspace.5](../functional-spec/FS-workspace.md#5-command-scope), every such unresolved
qualified citation is an `unknown project alias <name>` error at the
citation site.

This is the architecturally honest default: the resolver (section 1) returns
`None` for the unknown alias, and the caller — a single rule, in one place — turns
`None` into a diagnostic. The opt-in to downgrade these to warnings
(`[reference] cross_project_when_standalone = "warn"`) is deferred follow-up
([§DF-subproject-namespaces](../decisions/functional/DF-subproject-namespaces.md#df-subproject-namespaces-alias-namespace-model-for-sub-projects-and-external-repos) §3.6); when it lands, it changes one branch in
the checker, not the scanner, not the loader, not the resolver shape.

## 3. Downstream commands compose, not duplicate

Query commands (`show`, `refs`, `list`, `cover`, completions) and the formatter
(`fmt --cross-refs`) consume the qualified-citation shape through a single
shared loader, `load_workspace_context`
([§FS-workspace.8](../functional-spec/FS-workspace.md#8-other-commands)).
That loader funnels through `resolve_workspace_config` so workspace
discovery and member-scope rewriting stay in one place
([§AR-workspace.5.1](AR-workspace.md#51-one-loader-one-parser)), and it
exposes:

1. The list of projects in scope (root + members in workspace mode; a
   single project member-local or standalone).
2. The "current" project for unqualified IDs (root at the workspace
   root; `None` when `include_root = false`, so unqualified queries are
   forced to qualify or fail loud).
3. `project_by_alias` for routing a qualified `<§>alias/<ID>` to the
   right config + findings; `aliases()` for completion candidates.

Each command then applies its own filter — `grund refs FS-x` invoked at
the workspace root scopes to the current (root) project; `grund list
--project api` narrows the catalog; `grund fmt --cross-refs` from a
member tree preserves any pre-existing qualified wraps as-is and emits
no new ones ([§FS-workspace.8.5](../functional-spec/FS-workspace.md#85-grund-fmt---cross-refs)).
No command re-implements the resolver, the citation regex, or the alias
derivation.

`grund show --batch` is one consumer invocation, not a loop around the public
single-query API. A non-empty explicit batch or `--all` calls the shared loader
exactly once, then resolves, slices, and renders every coordinate against the
returned context ([§FS-show.2.6](../functional-spec/FS-show.md#26-batch-resolution)).
The exhaustive coordinate collector reads the declarations and recorded section
maps already in that context; it performs no preliminary completion/list scan.
The loader exposes an opt-in test-only counting observer so focused black-box
tests count one load for many explicit queries and one for exhaustive discovery.

`grund cover` applies **no** filter: it is keyed by file, so every project
the loader returned contributes its scanned files and every citation in
them, qualified or not ([§FS-workspace.8.6](../functional-spec/FS-workspace.md#86-grund-cover), [§DF-cover-workspace-scope](../decisions/functional/DF-cover-workspace-scope.md#df-cover-workspace-scope-cover-indexes-the-whole-run-and-counts-cross-project-citations)). That
is the one command where dropping a row is indistinguishable from a file
having nothing to say, so it is the one command whose consumer-end filter
was a silent skip rather than a scope choice. A file belongs to exactly one
project by the boundary rule ([§AR-workspace.6](AR-workspace.md#6-the-workspace-boundary)),
so the per-file index needs no merge step and the alias attached to each entry
is unambiguous.

It reaches the loader through `load_narrowable_workspace_context`, which takes
the workspace-aggregate arm only when `scope_is_config_root` — the same test
`run_check` uses — and otherwise returns the one narrowed project
([§FS-workspace.8.6](../functional-spec/FS-workspace.md#86-grund-cover)). Both
arms build the single-project context from one helper, so "single project"
cannot come to mean two things.
