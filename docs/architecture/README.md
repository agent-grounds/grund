# AR-system: one engine, twelve components, three frontends

`grund` is one pipeline. A single tree walk reads every file once and produces
`Findings`; the resolver completes the structural model; chapter-rule parsing
and fact production meet only as `ParsedRule` and `RuleFacts` in the rules
engine; checker rules turn that model into a `Report`; queries and writers
answer from the same model; and thin frontends render or transport what the
engine returns. Everything that decides lives in the engine crate `grund-core`,
so the CLI, the LSP server and the planned bindings are the same verdicts behind
different surfaces ([§GOAL-multi-language](../goals.md#goal-multi-language-same-engine-three-platforms),
[§FS-distribution](../functional-spec/FS-distribution.md#fs-distribution-grund-distribution-targets),
[§FS-rules.11](../functional-spec/FS-rules.md#11-functional-architecture-constraint)).
Speed is set by the walk, which is why the walk happens once
([§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible)).
This page is the whole system, described once; each page in the index below
describes one part and names its box here in a `placement` chapter (section 5).

## 1. The system

Twelve components in one crate, stacked in the order they may read each other,
and the frontends above them:

```text
             ┌───────────┐  ┌───────────┐  ┌──────────────────────┐
 frontends   │ grund-cli │  │ grund-lsp │  │ grund-node, grund-py │  planned
             └─────┬─────┘  └─────┬─────┘  └──────────┬───────────┘
                   └──────────────┼───────────────────┘
                                  ▼
             ┌────────────────────────────────────────────────────┐
 grund-core  │ 2.9  api                                           │
             ├─────────────────────────┬──────────────────────────┤
             │ 2.7  queries            │ 2.8  writers             │
             │      show, refs, list,  │      fmt, id, init,      │
             │      cover, completions,│      fetch, integrations │
             │      editor answers     │                          │
             ├─────────────────────────┴──────────────────────────┤
             │ 2.6  checker                                       │
             ├────────────────────────────────────────────────────┤
             │ 2.12 rules                                         │
             ├────────────────────────────────────────────────────┤
             │ 2.10 resolver                                      │
             ├────────────────────────────────────────────────────┤
             │ 2.5  scanner                                       │
             ├─────────────────────────┬──────────────────────────┤
             │ 2.4  workspace          │ 2.11 templates           │
             ├─────────────────────────┴──────────────────────────┤
             │ 2.3  config                                        │
             ├────────────────────────────────────────────────────┤
             │ 2.1  grammar                                       │
             ├────────────────────────────────────────────────────┤
             │ 2.2  model                                         │
             └────────────────────────────────────────────────────┘
             a component reads only what is below it (section 4)
```

The same components as the data moves through them:

```text
               grund.toml ──► config ──► workspace ──┐
                                                     ▼
  tree ──► scanner ──► Findings ─────────────────► resolver ────────────────┐
                                                        │                  │
                             rule titles + vocabulary ──┤                  │
                                                        ▼                  ▼
                              ParsedRule + RuleFacts ─► rules ─► Diagnostic│
                                                                           ▼
                       loaded Findings ─────────────────► checker ─► Report ─┐
                              │                                             │
                              ├──► queries ──► data ────────────────────────┤
                              └──► writers ──► edits ───────────────────────┤
                                                                            ▼
                                                                           api
                                                                            │
                                                ┌──────────────┬────────────┴───────────┐
                                               cli            lsp           node, py (planned)
```

Data flows along the arrows and so does knowledge: a component knows only what the arrows into it carry (section 4).

## 2. Components

One subsection per box. Each says what the box consumes, what it produces, what it must not know, and where its design is written. A component whose subsection is its whole architecture has no page of its own; it gets one when it has invariants beyond its placement. Each subsection ends with the component's Rust module in `crates/grund-core/src/`, one directory per box, whose `mod.rs` re-exports all that crosses its boundary ([§AR-core-module-layout.1](AR-core-module-layout.md#1-module-categories), section 4).

### 2.1 grammar

Consumes text. Produces the lexical facts every other component shares: the ID grammar and its near-miss detection, comment-line and comment-block recognition, fenced-block boundaries, the number-only shorthand, inline-note layout, the never-rewrite predicates ([§FS-fmt.2.3](../functional-spec/FS-fmt.md#23-what-is-never-rewritten)), and the formatter's own syntax — the `--cross-refs` link wrapper a shown body is flattened back from ([§FS-fmt.6.2](../functional-spec/FS-fmt.md#62-form), [§DF-show-cross-ref-flattening](../decisions/functional/DF-show-cross-ref-flattening.md#df-show-cross-ref-flattening-grund-show-flattens-cross-reference-link-wrappers)) and the two scopes a repository takes out of a rewrite's reach ([§FS-fmt.2.5](../functional-spec/FS-fmt.md#25-suppressed-scopes), [§DF-fmt-suppression](../decisions/functional/DF-fmt-suppression.md#df-fmt-suppression-fmt-suppression-is-per-file-and-per-region-and-the-index-carve-out-outranks-both)). Knows no file and no rule, and takes no `Config`: what configuration decides reaches it as the compiled `Grammar` and the one settings record built beside it — the marker, `[reference] strict`, the comment prefixes, the inline-note keys — so every reader here reads a decision and makes none ([§FS-config.3.2](../functional-spec/FS-config.md#32-id--id-grammar), [§FS-config.3.1](../functional-spec/FS-config.md#31-reference--citation-form), [§FS-config.3.5](../functional-spec/FS-config.md#35-scan--what-gets-walked)). Module: `crates/grund-core/src/grammar/`.

### 2.2 model

Consumes nothing. Produces the data every component passes along: `Findings`, `Declaration`, `Citation`, `Report`, and the value records ([§FS-values.2](../functional-spec/FS-values.md#2-value-declarations)). Knows nothing else; it is types and tiny helpers. Module: `crates/grund-core/src/model/`.

### 2.3 config

Consumes `grund.toml` and the defaults. Produces one validated `Config` per project ([§FS-config](../functional-spec/FS-config.md#fs-config-grund-reads-a-toml-config-file-found-by-walking-up)). Knows nothing of the tree it describes. Module: `crates/grund-core/src/config/`.

### 2.4 workspace

Consumes configs. Produces the multi-project scope — member expansion, claims, scope narrowing, boundary roots ([§FS-workspace](../functional-spec/FS-workspace.md#fs-workspace-grund-validates-cross-project-citations-in-a-workspace)). Knows no rule, no rendering and no scan: loading the projects it names is the resolver's, one box up (section 2.10). Its own invariants are [§AR-workspace](AR-workspace.md#ar-workspace-how-the-config-time-workspace-layer-composes-with-the-config-loader-and-the-scanner). Module: `crates/grund-core/src/workspace/`.

### 2.5 scanner

Consumes the scope and the grammar. Produces `Findings`: every declaration, section, citation, value binding and grounding unit in the tree, from one walk ([§FS-check.1](../functional-spec/FS-check.md#1-inputs)) — and the one probe over the tree that is no part of that walk, which agent entrypoint files a repository has, because `init` and `check` both ask it and must not disagree ([§FS-init.2.1](../functional-spec/FS-init.md#21-files-written-updated-or-left-in-place), [§FS-check.3.5](../functional-spec/FS-check.md#35-invalid-agent-entrypoint-init-block)). Knows no rule and no frontend, and never asks whether it is in a workspace. Design: [§AR-scanner](AR-scanner.md#ar-scanner-how-grund-discovers-declarations-and-citations). Module: `crates/grund-core/src/scanner/`.

### 2.6 checker

Consumes the resolver's loaded `Findings` and diagnostics from the rules
component. Produces the `Report`: errors, warnings and suggestions, each check
one pass over its owned input ([§FS-check](../functional-spec/FS-check.md#fs-check-grund-validates-every-reference-in-a-repo),
[§FS-rules.11](../functional-spec/FS-rules.md#11-functional-architecture-constraint)).
It orchestrates rule parsing, fact production and evaluation without owning any
of their grammar or relational meaning. Reads no file except in the two checks
that must, and knows no frontend. Design:
[§AR-checker](../../crates/grund-core/src/checker/report.rs). Module:
`crates/grund-core/src/checker/`.

### 2.7 queries

Consume `Findings`. Produce data for one question each: a declaration body ([§FS-show](../functional-spec/FS-show.md#fs-show-grund-reads-a-single-declaration-body-by-id)), the citers of an ID ([§FS-refs](../functional-spec/FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id)), the catalog ([§FS-list](../functional-spec/FS-list.md#fs-list-grund-lists-every-declared-id)), per-file coverage ([§FS-cover](../functional-spec/FS-cover.md#fs-cover-grund-groups-citations-by-scanned-file)), shell completions ([§FS-completions](../functional-spec/FS-completions.md#fs-completions-grund-completes-declared-ids-in-shells)), and the editor's snapshot, hover and on-type answers ([§FS-lsp](../functional-spec/FS-lsp.md#fs-lsp-grund-ships-an-optional-lsp-server)). Know no rendering; the text and JSON shapes belong to the frontends. Module: `crates/grund-core/src/queries/`.

### 2.8 writers

Consume `Findings` and the tree. Produce edits: citation normalization and cross-reference links ([§FS-fmt](../functional-spec/FS-fmt.md#fs-fmt-grund-normalizes-references-in-bulk)), a proposed ID ([§FS-id](../functional-spec/FS-id.md#fs-id-grund-proposes-ids-for-new-declarations)), the init scaffold and the managed agent-entrypoint block — which files a run writes, the splice that puts the block in one, and the walk-up the block's workspace section needs, the block *text* being the templates' (section 2.11) ([§FS-init](../functional-spec/FS-init.md#fs-init-grund-bootstraps-a-new-grund-conformant-repo)) — an external fact snapshot ([§FS-fetch](../functional-spec/FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot)), and the clickable-citation client artifacts ([§FS-integrations](../functional-spec/FS-integrations.md#fs-integrations-grund-prints-and-installs-its-rendering-layer-integrations)). The only components that write to the tree, and each writes only what its spec names ([§REQ-no-data-loss](../requirements/REQ-no-data-loss.md#req-no-data-loss-grund-never-eats-user-content)). Module: `crates/grund-core/src/writers/`. The `grund` CLI calls these data-returning operations and owns their argv, rendered bytes and exit codes; for example, `integrations` is implemented by `crates/grund-cli/src/cli_integrations*.rs` over the `pub` block of `writers/mod.rs` ([§FS-integrations.1](../functional-spec/FS-integrations.md#1-user-facing-command), [§AR-bindings.3](AR-bindings.md#3-cratesgrund-cli-the-cli-binary)).

### 2.9 api

Consumes everything above. Produces the embedding surface: data-returning functions and the public types ([§AR-bindings.2](AR-bindings.md#2-grund-core-the-only-place-logic-lives), [§FS-distribution.3](../functional-spec/FS-distribution.md#3-api-surfaces)). Writes to no stream, exits no process and knows no frontend. Module: `crates/grund-core/src/api/`, one file per surface beside the private adapters that fill it. There is no process-frontend exception (section 2.9.1).

#### 2.9.1 No process frontend lives in the engine

No engine component parses argv, renders to a stream, or decides a process exit status. The former `grund_core::main_entry()` process frontend and its `compat/` module left after the deprecation path of [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) completed; `lib.rs` exports no replacement renderer ([§FS-distribution.3.1.1](../functional-spec/FS-distribution.md#311-main_entry-is-absent-from-the-embedding-api), [§DA-engine-renders-nothing](../decisions/architectural/DA-engine-renders-nothing.md#da-engine-renders-nothing-the-engine-renders-nothing-so-the-deprecated-compat-frontend-retires)). Run-level cautions, including the four `[workspace]` warnings that were last to move, travel as `Diagnostic` data for each frontend to render ([§FS-check.4.7](../functional-spec/FS-check.md#47-a-workspace-member-swallows-the-blocks-own-scan), [§FS-check.3.29](../functional-spec/FS-check.md#329-unlisted-workspace-block), [§FS-check.4.10](../functional-spec/FS-check.md#410-include_root--false-leaves-the-blocks-own-files-unread), [§FS-workspace.6.1](../functional-spec/FS-workspace.md#61-nested-workspaces)). `tests/integration/test_engine_boundary.py` holds the absence of the renderer and process-entry export; the direction ledger in section 4 covers only the twelve engine components.

### 2.10 resolver

Consumes the project map from workspace and every project's `Findings` from the scanner. Produces the loaded project set a run operates on — each project scanned, its off-grammar citations and its cross-namespace number-only shorthands reconciled across the whole set — and four answers that are functions of what a run loaded: which project a citation resolves against, a declaration's body sliced by the spans the scan recorded, the link target its ID resolves to, and which declaration a shorthand token names in whichever project's catalog answers for it ([§AR-resolver.4](AR-resolver.md#4-the-shorthand-a-whole-runs-catalog-resolves)) ([§FS-workspace.8](../functional-spec/FS-workspace.md#8-other-commands), [§FS-show.2](../functional-spec/FS-show.md#2-behavior), [§FS-fmt.6.2](../functional-spec/FS-fmt.md#62-form), [§FS-fmt.2.4](../functional-spec/FS-fmt.md#24-shorthand-to-canonical)). Knows no rule and no rendering; it runs scans, which is what puts it above the scanner while the config half of the workspace stays below it. Design: [§AR-resolver](AR-resolver.md#ar-resolver-how-a-run-loads-every-project-and-resolves-a-citation-to-one-of-them). Module: `crates/grund-core/src/resolver/`.

### 2.11 templates

Consumes config and the managed-block markers the grammar recognizes. Produces the text a managed block should say as a function of that config: the `AGENTS.md` block of [§FS-init.2.3](../functional-spec/FS-init.md#23-generated-agent-entrypoints) with its config-derived sections, the generated `grund.toml`, and the embedded scaffold payload ([§FS-init.2.1](../functional-spec/FS-init.md#21-files-written-updated-or-left-in-place), [§FS-init.2.4](../functional-spec/FS-init.md#24-generated-grundtoml), [§FS-init.5](../functional-spec/FS-init.md#5-agent-setup-instructions)). Rendering is deterministic, so a fresh render is the hash: `init` writes it and `check` re-renders its `### Citation directions` and `### Clickable citations` sections and byte-compares them for drift ([§FS-check.3.5](../functional-spec/FS-check.md#35-invalid-agent-entrypoint-init-block), [§AR-checker.2.7](../../crates/grund-core/src/checker/report.rs)) — two commands asking one component for the same answer from opposite directions, which is why it is a box rather than a corner of the writers. Knows no file, no rule and no rendering of a report: it writes nothing and reads no tree, and the one block section that needs a walk arrives already rendered from the run that walked for it ([§FS-init.2.3.4.15](../functional-spec/FS-init.md#23415-workspace-members)). Module: `crates/grund-core/src/templates/`.

### 2.12 rules

Consumes authored rule titles with config vocabulary and the resolver's complete
structural model. Produces located rule `Diagnostic`s for the checker through
exactly two internal data boundaries: `ParsedRule` from the sentence front end
and `RuleFacts` from the Markdown adapter. The logic engine evaluates only
those values and owns semantic deduplication; the scanner remains rule-blind
([§FS-rules.11](../functional-spec/FS-rules.md#11-functional-architecture-constraint)).
Knows no frontend, renderer or filesystem beyond repository-relative fact
anchors. Design: [§AR-rules](AR-rules.md#ar-rules-sentences-and-facts-meet-only-in-the-rule-engine).
Module: `crates/grund-core/src/rules/` (created with the implementation).

## 3. Frontends

Two today, two planned, and none has engine logic ([§AR-bindings](AR-bindings.md#ar-bindings-target-shape-for-exposing-the-rust-engine-on-three-platforms)). `grund-cli` parses arguments, renders text and JSON, and maps exit codes ([§AR-bindings.3](AR-bindings.md#3-cratesgrund-cli-the-cli-binary), [§FS-cli](../functional-spec/FS-cli.md#fs-cli-grunds-command-line-surface-conventions)). `grund-lsp` speaks LSP over stdio and translates every request into an api call ([§AR-lsp](AR-lsp.md#ar-lsp-how-the-lsp-server-is-built)). `grund-node` and `grund-py` will marshal the same functions ([§AR-bindings.5](AR-bindings.md#5-grund-node-the-napi-rs-binding), [§AR-bindings.6](AR-bindings.md#6-grund-py-the-pyo3-binding)). Each depends on `grund-core` and on nothing of the others, so the CLI carries no JSON-RPC and the server no terminal renderer ([§DA-lsp-optional](../decisions/architectural/DA-lsp-optional.md#da-lsp-optional-lsp-server-ships-as-a-separate-optional-binary)).

## 4. Dependency direction

One rule: **no component reads one above it.** The stack in section 1 is the
rule drawn: the frontends sit above api; api above the queries and the writers,
which are siblings and read nothing of each other; those above the checker; the
checker above rules; rules above resolver; resolver above scanner; scanner above
workspace and templates, which are siblings and read nothing of each other;
both above config; config above grammar; and grammar above model, which reads
nothing but std. Two things hold it. The compiler holds a component's privacy —
nothing outside a module directory can name what its `mod.rs` does not re-export
([§AR-core-module-layout.1.1](AR-core-module-layout.md#11-modrs-is-the-components-whole-boundary)) —
and `tests/integration/test_dependency_direction.py` holds the order across
those directories, with every read that still runs the other way listed one by
one and marked at its import, so the list can only shrink. The rules component's
internal direction and stronger parser/engine prohibitions are held by
`tests/integration/test_rules_architecture.py` ([§AR-rules.6](AR-rules.md#6-boundary-tests)).
Three consequences are held by tests of their own:

- The engine writes no stream and exits no process; the frontends render (`tests/integration/test_engine_boundary.py`).
- The engine names no frontend's protocol: no LSP types in `grund-core`, no CLI in `grund-lsp` (`tests/integration/test_frontend_isolation.py`).
- A frontend re-implements nothing: every regex, walk and rule is in the engine ([§AR-bindings.2](AR-bindings.md#2-grund-core-the-only-place-logic-lives)).

## 5. What holds the shape

- **Placement.** Every page in the index below opens with a `## placement:` chapter — a named section, so `grund <ID>.placement` is the question — that opens with a diagram in the notation of section 1 — what feeds the component on the left, its box in the middle, what it feeds on the right — and then says, in four facts, its box in section 2 or 3, what it takes and from whom, what it gives and to whom, and what it must not know. Anything wider than that belongs on this page. `tests/integration/test_architecture_placement.py` holds it: every page but this one has the chapter, the chapter opens with a fenced diagram, and it cites this page.
- **Files.** How the engine's files are named, owned and sized is [§AR-core-module-layout](AR-core-module-layout.md#ar-core-module-layout-core-implementation-is-split-by-category); two tests hold its module directories and the direction across them ([§AR-core-module-layout.1.4](AR-core-module-layout.md#14-two-tests-hold-the-layout-against-the-tree)), and `fissile` holds the size.
- **Assurance is not a component.** [§AR-ci](AR-ci.md#ar-ci-ci-mirrors-the-local-pre-commit-gate), [§AR-benchmarks](AR-benchmarks.md#ar-benchmarks-instruction-counting-benchmarks-for-the-hot-cli-commands) and [§AR-goal-measurement](AR-goal-measurement.md#ar-goal-measurement-goal-and-requirement-meters-live-outside-goals) measure the system rather than sit in it, and are listed apart below; their placement chapters say what each measures.

# Index

One file per page; each H1 declares an `AR-<slug>` ID and the body is its contract, and `§AR-<slug>.<section>` from anywhere in the tree resolves into it. A page may live inline in the doc-comment of the file it describes: its canonical link in this index enrolls it directly, with no stub file ([§FS-check.3.18](../functional-spec/FS-check.md#318-declaration-missing-from-its-kinds-index)), and `grund <ID>` resolves the source declaration and strips its comment markers ([§AR-scanner.4](AR-scanner.md#4-inline-declarations-in-language-doc-comments) lists the doc-comment forms). [§AR-checker](../../crates/grund-core/src/checker/report.rs) is the worked example: its only declaration is the doc-comment of `fn check` in [`crates/grund-core/src/checker/report.rs`](../../crates/grund-core/src/checker/report.rs).

The system:

| ID | Subject |
|---|---|
| [§AR-system](README.md#ar-system-one-engine-twelve-components-three-frontends) | one engine, twelve components, three frontends — this page |

The components and frontends:

| ID | Subject |
|---|---|
| [§AR-scanner](AR-scanner.md#ar-scanner-how-grund-discovers-declarations-and-citations) | how grund discovers declarations and citations |
| [§AR-checker](../../crates/grund-core/src/checker/report.rs) | how grund validates the scanner's findings — declared and enrolled directly from `crates/grund-core/src/checker/report.rs` |
| [§AR-workspace](AR-workspace.md#ar-workspace-how-the-config-time-workspace-layer-composes-with-the-config-loader-and-the-scanner) | how the config-time workspace layer composes with the config loader and the scanner |
| [§AR-resolver](AR-resolver.md#ar-resolver-how-a-run-loads-every-project-and-resolves-a-citation-to-one-of-them) | how a run loads every project and resolves a citation to one of them |
| [§AR-rules](AR-rules.md#ar-rules-sentences-and-facts-meet-only-in-the-rule-engine) | how sentence parsing and relational evaluation meet only through `ParsedRule` and `RuleFacts` |
| [§AR-bindings](AR-bindings.md#ar-bindings-target-shape-for-exposing-the-rust-engine-on-three-platforms) | the engine's contract with its frontends, and the shape of the planned ones |
| [§AR-lsp](AR-lsp.md#ar-lsp-how-the-lsp-server-is-built) | how the LSP server is built |
| [§AR-core-module-layout](AR-core-module-layout.md#ar-core-module-layout-core-implementation-is-split-by-category) | how the engine's files are named, owned and sized |

What measures the system:

| ID | Subject |
|---|---|
| [§AR-ci](AR-ci.md#ar-ci-ci-mirrors-the-local-pre-commit-gate) | CI mirrors the local pre-commit gate |
| [§AR-benchmarks](AR-benchmarks.md#ar-benchmarks-instruction-counting-benchmarks-for-the-hot-cli-commands) | instruction-counting benchmarks for the hot CLI commands |
| [§AR-goal-measurement](AR-goal-measurement.md#ar-goal-measurement-goal-and-requirement-meters-live-outside-goals) | goal and requirement meters live outside goals |

The index is navigational: cite a page's ID, or `AR-system` for the whole, never this file by path.
