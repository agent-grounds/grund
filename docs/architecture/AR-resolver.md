# AR-resolver: how a run loads every project and resolves a citation to one of them

A workspace is two halves. Reading the configs — expanding one `members` list,
climbing the claims that spell an alias path, narrowing a scope, computing the
boundary roots a scan stops at — is [§AR-workspace](AR-workspace.md#ar-workspace-how-the-config-time-workspace-layer-composes-with-the-config-loader-and-the-scanner), and it happens
before any file is read. *Loading* those projects is this page: scanning every
one of them, reconciling off-grammar citations and cross-namespace shorthands
across the whole set, and answering the questions that are functions of what a
run loaded and of nothing else ([§FS-workspace.8](../functional-spec/FS-workspace.md#8-other-commands)).

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
                                     ├─► declaration ──► link target
                                     └─► shorthand token ──► the declaration it names
```

The tenth box of the pipeline ([§AR-system.2.10](README.md#210-resolver)). It takes the
project map workspace expanded out of configs ([§AR-system.2.4](README.md#24-workspace)) and each
project's `Findings` from the scanner ([§AR-system.2.5](README.md#25-scanner)), and gives the
checker, the queries and the writers the loaded project set with the four
answers above ([§AR-system.2.6](README.md#26-checker), [§AR-system.2.7](README.md#27-queries), [§AR-system.2.8](README.md#28-writers)). It
knows no rule and no rendering: it settles which project a coordinate lands in
and what text is there, never whether that is an error or how to print it
([§AR-system.4](README.md#4-dependency-direction)).

It is also where the run's warning channel is settled, for the same reason the
loaded project set is: [§FS-check.4.10](../functional-spec/FS-check.md#410-include_root--false-leaves-the-blocks-own-files-unread)
asks whether a block that opted out of being a project would have read something
had it been one, which only a walk can answer, so workspace *poses* that question
and `resolver/unread_block.rs` answers it ([§AR-workspace.placement](AR-workspace.md#placement-where-the-workspace-layer-sits)).
Assembling the four run-level `[workspace]` warnings there hands every walking
command one ordered list of `Diagnostic`s to carry on whatever it returns — it
renders none of them ([§FS-distribution.3.1](../functional-spec/FS-distribution.md#31-rust-grund-core-crate),
[§DA-engine-renders-nothing](../decisions/architectural/DA-engine-renders-nothing.md#da-engine-renders-nothing-the-engine-renders-nothing-so-the-deprecated-compat-frontend-retires)).

## terms: Terms

Leans on [§FS-terms.terms.1](../functional-spec/FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, body, section, coordinate,
catalog), [§FS-terms.terms.2](../functional-spec/FS-terms.md#terms2-citations) (citation, qualified citation, shorthand, canonical
form, citation site), [§FS-terms.terms.4](../functional-spec/FS-terms.md#terms4-scanning-and-project-structure) (scan, scope, workspace, member, alias),
[§FS-terms.terms.5](../functional-spec/FS-terms.md#terms5-findings) (finding, verdict), [§FS-terms.terms.6](../functional-spec/FS-terms.md#terms6-rules-and-directions) (rule), and
[§FS-terms.terms.8](../functional-spec/FS-terms.md#terms8-the-architectures-own-words) (box).

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
rule turns it into a located diagnostic (`unknown project alias <path>` at
the citation site). Section 2 is that consequence in full.

## 2. Standalone members fail loud, not silent

A `grund check` invoked at a member root cannot resolve qualified citations
to the workspace or to siblings — there is no project map. Per
[§DF-subproject-namespaces](../decisions/functional/DF-subproject-namespaces.md#df-subproject-namespaces-alias-namespace-model-for-sub-projects-and-external-repos) [§DF-subproject-namespaces.3.6](../decisions/functional/DF-subproject-namespaces.md#36-standalone-members-fail-loud-not-silent) and [§FS-workspace.5](../functional-spec/FS-workspace.md#5-command-scope), every such unresolved
qualified citation is an `unknown project alias <path>` error at the
citation site.

This is the `None` of section 1, turned into a diagnostic by a single rule in
one place. The opt-in to downgrade these to warnings
(`[reference] cross_project_when_standalone = "warn"`) is deferred follow-up
([§DF-subproject-namespaces](../decisions/functional/DF-subproject-namespaces.md#df-subproject-namespaces-alias-namespace-model-for-sub-projects-and-external-repos) [§DF-subproject-namespaces.3.6](../decisions/functional/DF-subproject-namespaces.md#36-standalone-members-fail-loud-not-silent)); when it lands, it changes one branch in
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
A `<path>` argument therefore means two different things, and [§AR-resolver.3.3](AR-resolver.md#33-two-scopes-and-which-narrowness-a-narrow-scope-keeps) is which.
The one-invocation batch loader and the unfiltered `cover` are [§AR-resolver.3.1](AR-resolver.md#31-grund-show---batch-loads-once) and [§AR-resolver.3.2](AR-resolver.md#32-grund-cover-filters-nothing).

### 3.1 `grund show --batch` loads once

`grund show --batch` is one consumer invocation, not a loop around the public
single-query API. A non-empty explicit batch or `--all` calls the shared loader
exactly once, then resolves, slices, and renders every coordinate against the
returned context ([§FS-show.2.6](../functional-spec/FS-show.md#26-batch-resolution)).
The exhaustive coordinate collector reads the declarations and recorded section
maps already in that context; it performs no preliminary completion/list scan.
The loader exposes an opt-in test-only counting observer, compiled in only by the
`test-workspace-load-count` feature, so focused black-box tests count one load
for many explicit queries and one for exhaustive discovery; in a build without
the feature those tests are ignored, naming it, rather than run against a log
nobody wrote ([§AR-ci.3.3](AR-ci.md#33-test-only-observers)).

### 3.2 `grund cover` filters nothing

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

### 3.3 Two scopes, and which narrowness a narrow scope keeps

For a query command a `<path>` selects a **project** and bounds no walk. For `grund check` it is the
**report scope** — the set of files a finding may be about — while the **resolution scope** it reads
stays the enclosing project's ordinary scope, `[scan] include` plus every walked kind home, union the path
([§FS-check.1.3.6.1](../functional-spec/FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution)). The run walks the resolution scope, runs every rule over the whole of it, and drops
the diagnostics anchored outside the report scope.

So "a scope narrower than the root has to stay narrow" is a statement about which **projects** a walk covers
and which **files** are reported. It is not a statement about which declarations resolution reads: widening
that set answers the question the caller asked rather than a wider one, because the caller asked whether their
path is clean and not whether it is self-contained. The narrowing that must stay is `cover`'s and
`scope_is_config_root`'s — a narrowed member run stays one project ([§FS-workspace.8.6](../functional-spec/FS-workspace.md#86-grund-cover)) — and the
two are kept at visibly different stages so neither drifts into the other: `retain_findings_in_scope` narrows
the scan result before any rule runs, which is what `--full` additivity needs ([§FS-check.1.3.4](../functional-spec/FS-check.md#134-purely-additive)), and the
path filter narrows the report after every rule has run.

## 4. The shorthand a whole run's catalog resolves

Recognizing a number-only shorthand is lexical, and stays in the grammar: the
shape a token wears, the "same kind, same number" candidate rule, and the index
a pass builds out of one declaration set so it pays one walk instead of one per
site ([§AR-system.2.1](README.md#21-grammar), [§FS-check.1.2](../functional-spec/FS-check.md#12-the-number-only-shorthand)). Resolving one lands here wherever the
answer needs more than the project the token sits in.

- **`§<alias>/FS-042` has no local grammar to expand it with.** A per-project
  scan sees only its own declarations, so the cross-namespace half of the rule
  runs once every project has been loaded, against the declaration set of the
  project the alias names ([§FS-workspace.1](../functional-spec/FS-workspace.md#1-citation-syntax), [§AR-scanner.2.6](AR-scanner.md#26-number-only-shorthand-citations)). The unqualified
  half stays inside each project's own walk, because there the token and the
  declarations it could name belong to one project.
- **What `fmt` writes in its place** is the *target's* canonical form — rendered
  under the target's `[id] format`, sectioned with the target's separator, and
  gated by the target's `[reference] shorthand` policy, because a workspace may
  mix all three ([§FS-fmt.2.4](../functional-spec/FS-fmt.md#24-shorthand-to-canonical), [§FS-workspace.8.5](../functional-spec/FS-workspace.md#85-grund-fmt---cross-refs)). The per-walk index that
  answers it, this project's declarations plus one per alias, is built here for
  the same reason.
- **What an editor offers** for a shorthand being typed is that same expansion
  asked of a list of declared IDs ([§FS-lsp.1.4](../functional-spec/FS-lsp.md#14-live-trigger-transform)), so the on-type edit and
  `grund fmt` cannot disagree about what resolves.

What is *not* here is the finding a shorthand site earns. Unique, ambiguous or
unknown is a verdict, and a verdict is the checker's ([§FS-check.3.13](../functional-spec/FS-check.md#313-number-only-shorthand-citation),
[§AR-checker.2.12](../../crates/grund-core/src/checker/report.rs)): it reads the candidate set downward out of the grammar's
index, and which project to read it in out of section 1.

## 5. A stub's sections, where the walk did not reach its target

A stub stands for its target's declaration, so a citation of one of its sections
resolves where the target declares that section, scanned or not
([§FS-check.3.2.1](../functional-spec/FS-check.md#321-a-stubs-sections-are-its-targets-scanned-or-not)). Where the walk reached the target, the target's record is among
the `Findings` and nothing is read. Where it did not, the only record of the ID is
the stub, which has no sections of its own, and a lookup that stopped at the
records would report a section the target plainly declares. So the lookup goes on
to `resolver/stub_home.rs`: the reading `show` already made of a target outside the
walk ([§FS-show.2.3.7](../functional-spec/FS-show.md#237-a-stubs-target-is-found-by-its-id)), moved down out of `queries/` so that both commands answer
from one record and count one section set ([§FS-show.2.2.2.2](../functional-spec/FS-show.md#2222-the-headings-check-counts)).

That reading is the scanner's own pass over the target, `scan_unwalked_file`, kept
to the declarations of the ID that are not themselves stubs. It reads the
declaration of the ID and never the file's headings, so a broken stub, whose target
declares no home of the ID, gains no section and stays a broken stub. It reads only
where the broken-stub rule reads ([§AR-checker.2.5](../../crates/grund-core/src/checker/report.rs)): a scannable file other than the
stub's own, at which no record of the ID already sits. And it lends nothing where `show`
refuses the ID as ambiguous: a target that declares the ID twice, and every target of an ID
with more than one home ([§FS-show.2.3.7](../functional-spec/FS-show.md#237-a-stubs-target-is-found-by-its-id), [§FS-show.2.2.1](../functional-spec/FS-show.md#221-ambiguous-id)). So `unscanned_stub_home`
answers one declaration or none, and every reader falls back to the stub's own record.

`section_resolves` in `resolver/citation_target.rs` is the one test, and every
reader in `check` asks it: the missing-section finding ([§AR-checker.2.4](../../crates/grund-core/src/checker/report.rs)), the
`grund fmt --write` clause that finding carries ([§FS-check.3.24.1](../functional-spec/FS-check.md#3241-the-release-attribution-and-where-the-command-clause-is-withheld)), the bare index
entry the index rule admits ([§FS-check.3.17.4](../functional-spec/FS-check.md#3174-only-a-citation-fmt-would-wrap-reaches-this-rule)), the local-section expansion `fmt`
writes ([§FS-fmt.2.4.6](../functional-spec/FS-fmt.md#246-an-absent-target-section-withholds-this-rewrite-and-only-this-one)), and the `cites` fact a rule reads ([§FS-rules.5.1](../functional-spec/FS-rules.md#51-facts-and-identity)). That last one
asks `section_home`, the same test answering where the section was found: where it was
found in a home outside the walk, the rule adapter mints that home's chapters as nodes on
this miss, from the record the lookup returned, so the fact's target is the chapter the
citation names, as on the scanned tree. They carry no `chapter` fact, so no subject or
count of chapters reaches them ([§FS-check.3.2.1](../functional-spec/FS-check.md#321-a-stubs-sections-are-its-targets-scanned-or-not), [§AR-rules.3](AR-rules.md#3-rulefacts)). A value
binding compares against `home_as_scanned` instead, the target's declaration in
place of the stub, so a mismatch names the target's line ([§FS-values.5.1](../functional-spec/FS-values.md#51-resolve-before-comparison)). The value
authority a malformed binding is refused by, and that `fmt --cross-refs` protects, is
read through it too, in the one predicate both share ([§FS-values.3.1.1](../functional-spec/FS-values.md#311-invalid-attempts-and-non-attempts), [§FS-values.8](../functional-spec/FS-values.md#8-formatting-stability)). So
is the heading a link anchors on, the declaration's own for a bare ID as much as a
`.<section>`'s: where the target declares the ID once, `fmt --cross-refs` reads it out
of the target's record, the one `show` anchors on, not off the stub's title-less line
([§FS-fmt.6.2.1.1](../functional-spec/FS-fmt.md#6211-through-a-stub-the-declarations-heading-is-the-targets)), and a
heading that record leaves out of the body gives a section no anchor
([§FS-fmt.6.4.1](../functional-spec/FS-fmt.md#641-a-section-the-declaration-does-not-have-is-not-wrapped)). A
workspace citation asks with the target project's `Config`, out of section 1,
because the stub's link resolves against that project's root.

Three properties hold the read to what a scan of the target would be:

- **Once, on a miss.** `Findings::stub_targets` keeps one slot per stub target,
  filled the first time a reader misses there and borrowed for the rest of the
  run. A section a record holds reads nothing, and a target many IDs and sections
  ask about is read once. A `--full` run narrows its findings after the slots are
  made ([§AR-resolver.3.3](AR-resolver.md#33-two-scopes-and-which-narrowness-a-narrow-scope-keeps)), which changes nothing a slot holds: whether a record
  sits at the target is asked of the current records on every lookup.
- **Observed.** The read goes through the input observation of [§FS-check.6.1.1](../functional-spec/FS-check.md#611-subscribe-before-reading), so
  a watching run re-checks when the target changes. The slots belong to the
  `Findings` a scan produced, so the next run starts from empty ones and reads the
  target again.
- **The editor's text.** The slots carry the overlays the walk was given, so a
  target open in an editor answers as a save would ([§FS-lsp.1.1](../functional-spec/FS-lsp.md#11-diagnostics)). `show` passes
  the overlays it was given straight to the same reading instead.

What stays in [§AR-scanner.4.6](AR-scanner.md#46-a-stubs-home-is-recorded-once-after-the-walk) is recording. Its post-walk pass records a home only
for an ID declared more than once; a lone stub's target is read here, and only when
a reader asks for what it declares.
