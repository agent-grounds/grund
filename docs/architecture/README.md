# AR-system: one engine, nine components, three frontends

`grund` is one pipeline. A single tree walk reads every file once and produces `Findings`; rules turn `Findings` into a `Report`; queries and writers answer from the same `Findings`; and thin frontends render or transport what the engine returns. Everything that decides lives in the engine crate `grund-core`, so the CLI, the LSP server and the planned bindings are the same verdicts behind different surfaces ([§GOAL-multi-language](../goals.md#goal-multi-language-same-engine-three-platforms), [§FS-distribution](../functional-spec/FS-distribution.md#fs-distribution-grund-distribution-targets)). Speed is set by the walk, which is why the walk happens once ([§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible)). This page is the whole system on one page: every architecture page in the index below carries a `placement` chapter that names its box here and reads no wider than it, so the system is described once and each page describes one part.

## 1. The system

Nine components in one crate, stacked in the order they may read each other, and the frontends above them:

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
             │ 2.5  scanner                                       │
             ├────────────────────────────────────────────────────┤
             │ 2.4  workspace                                     │
             ├────────────────────────────────────────────────────┤
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
  tree ──► scanner ──► Findings ──► checker ──► Report ─┐
                          │                             │
                          ├──► queries ──► data ────────┤
                          └──► writers ──► edits ───────┤
                                                        ▼
                                                       api
                                                        │
                            ┌──────────────┬────────────┴───────────┐
                           cli            lsp           node, py (planned)
```

Data flows along the arrows and so does knowledge: a component knows only what the arrows into it carry (section 4).

## 2. Components

One subsection per box. Each says what the box consumes, what it produces, what it must not know, and where its design is written. A component whose subsection is its whole architecture has no page of its own; it gets one when it has invariants beyond its placement. The names at the end of each subsection are today's file-name categories in `crates/grund-core/src/` ([§AR-core-module-layout.1](AR-core-module-layout.md#1-module-categories)); the second phase of this shape turns each component into a Rust module, so the compiler holds section 4 instead of a test.

### 2.1 grammar

Consumes text. Produces the lexical facts every other component shares: the ID grammar and its near-miss detection, comment-line and comment-block recognition, fenced-block boundaries, the number-only shorthand, inline-note layout, and the never-rewrite predicates ([§FS-fmt.2.3](../functional-spec/FS-fmt.md#23-what-is-never-rewritten)). Knows no file, no config and no rule. Module: `crates/grund-core/src/grammar/`.

### 2.2 model

Consumes nothing. Produces the data every component passes along: `Findings`, `Declaration`, `Citation`, `Report`, and the value records ([§FS-values.2](../functional-spec/FS-values.md#2-value-declarations)). Knows nothing else; it is types and tiny helpers. Module: `crates/grund-core/src/model/`.

### 2.3 config

Consumes `grund.toml` and the defaults. Produces one validated `Config` per project ([§FS-config](../functional-spec/FS-config.md#fs-config-grund-reads-a-toml-config-file-found-by-walking-up)). Knows nothing of the tree it describes. Module: `crates/grund-core/src/config/`.

### 2.4 workspace

Consumes configs. Produces the multi-project scope — member expansion, claims, scope narrowing — and the one resolver that maps a citation to its target project ([§FS-workspace](../functional-spec/FS-workspace.md#fs-workspace-grund-validates-cross-project-citations-in-a-workspace)). Knows no rule and no rendering; its own invariants are [§AR-workspace](AR-workspace.md#ar-workspace-how-the-resolver-config-loader-and-scanner-compose-across-projects). Module: `crates/grund-core/src/workspace/`.

### 2.5 scanner

Consumes the scope and the grammar. Produces `Findings`: every declaration, section, citation, value binding and grounding unit in the tree, from one walk ([§FS-check.1](../functional-spec/FS-check.md#1-inputs)). Knows no rule and no frontend, and never asks whether it is in a workspace. Design: [§AR-scanner](AR-scanner.md#ar-scanner-how-grund-discovers-declarations-and-citations). Module: `crates/grund-core/src/scanner/`.

### 2.6 checker

Consumes `Findings`. Produces the `Report`: errors, warnings and suggestions, each rule one pass over part of the findings ([§FS-check](../functional-spec/FS-check.md#fs-check-grund-validates-every-reference-in-a-repo)). Reads no file except in the two rules that must, and knows no frontend. Design: [§AR-checker](../../crates/grund-core/src/checker/report.rs). Module: `crates/grund-core/src/checker/`.

### 2.7 queries

Consume `Findings`. Produce data for one question each: a declaration body ([§FS-show](../functional-spec/FS-show.md#fs-show-grund-reads-a-single-declaration-body-by-id)), the citers of an ID ([§FS-refs](../functional-spec/FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id)), the catalog ([§FS-list](../functional-spec/FS-list.md#fs-list-grund-lists-every-declared-id)), per-file coverage ([§FS-cover](../functional-spec/FS-cover.md#fs-cover-grund-groups-citations-by-scanned-file)), shell completions ([§FS-completions](../functional-spec/FS-completions.md#fs-completions-grund-completes-declared-ids-in-shells)), and the editor's snapshot, hover and on-type answers ([§FS-lsp](../functional-spec/FS-lsp.md#fs-lsp-grund-ships-an-optional-lsp-server)). Know no rendering; the text and JSON shapes belong to the frontends. Module: `crates/grund-core/src/queries/`.

### 2.8 writers

Consume `Findings` and the tree. Produce edits: citation normalization and cross-reference links ([§FS-fmt](../functional-spec/FS-fmt.md#fs-fmt-grund-normalizes-references-in-bulk)), a proposed ID ([§FS-id](../functional-spec/FS-id.md#fs-id-grund-proposes-ids-for-new-declarations)), the init scaffold and the managed agent-entrypoint block ([§FS-init](../functional-spec/FS-init.md#fs-init-grund-bootstraps-a-new-grund-conformant-repo)), an external fact snapshot ([§FS-fetch](../functional-spec/FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot)), and the clickable-citation client artifacts ([§FS-integrations](../functional-spec/FS-integrations.md#fs-integrations-grund-prints-and-installs-its-rendering-layer-integrations)). The only components that write to the tree, and each writes only what its spec names ([§REQ-no-data-loss](../requirements/REQ-no-data-loss.md#req-no-data-loss-grund-never-eats-user-content)). Module: `crates/grund-core/src/writers/`. The two deprecated command adapters of the last two, `init_cmd.rs` and `integrations_cmd*.rs`, are deliberately still flat beside it, waiting for `compat/` with the other renderers ([§AR-system.2.9](README.md#29-api)).

### 2.9 api

Consumes everything above. Produces the embedding surface: data-returning functions and the public types, in `crates/grund-core/src/api.rs` and its siblings ([§AR-bindings.2](AR-bindings.md#2-grund-core-the-only-place-logic-lives), [§FS-distribution.3](../functional-spec/FS-distribution.md#3-api-surfaces)). Writes to no stream, exits no process and knows no frontend. The deprecated `main_entry()` path is the one exception, kept for 0.4 consumers until the deprecation path of [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) closes it; it renders inside the engine through the `compat` and `output` categories, a closed set that may shrink and never grow. Categories: `api`, `compat`, `output`.

## 3. Frontends

Three today, two planned, and none has engine logic ([§AR-bindings](AR-bindings.md#ar-bindings-target-shape-for-exposing-the-rust-engine-on-three-platforms)). `grund-cli` parses arguments, renders text and JSON, and maps exit codes ([§AR-bindings.3](AR-bindings.md#3-grund-cli-the-cli-binary), [§FS-cli](../functional-spec/FS-cli.md#fs-cli-grunds-command-line-surface-conventions)). `grund-lsp` speaks LSP over stdio and translates every request into an api call ([§AR-lsp](AR-lsp.md#ar-lsp-how-the-lsp-server-is-built)). `grund-node` and `grund-py` will marshal the same functions ([§AR-bindings.5](AR-bindings.md#5-grund-node-the-napi-rs-binding), [§AR-bindings.6](AR-bindings.md#6-grund-py-the-pyo3-binding)). Each depends on `grund-core` and on nothing of the others, so the CLI carries no JSON-RPC and the server no terminal renderer ([§DA-lsp-optional](../decisions/architectural/DA-lsp-optional.md#da-lsp-optional-lsp-server-ships-as-a-separate-optional-binary)).

## 4. Dependency direction

One rule: **no component reads one above it.** The stack in section 1 is the rule drawn: the frontends sit above api; api above the queries and the writers, which are siblings and read nothing of each other; those above the checker; the checker above the scanner; the scanner above workspace; workspace above config; config above grammar; and grammar above model, which reads nothing but std. Three consequences are held by tests today, and by the compiler once the categories are modules:

- The engine writes no stream and exits no process; the frontends render (`tests/integration/test_engine_boundary.py`).
- The engine names no frontend's protocol: no LSP types in `grund-core`, no CLI in `grund-lsp` (`tests/integration/test_frontend_isolation.py`).
- A frontend re-implements nothing: every regex, walk and rule is in the engine ([§AR-bindings.2](AR-bindings.md#2-grund-core-the-only-place-logic-lives)).

## 5. What holds the shape

- **Placement.** Every page in the index below opens with a `## placement:` chapter — a named section, so `grund <ID>.placement` is the question — that opens with a diagram in the notation of section 1 — what feeds the component on the left, its box in the middle, what it feeds on the right — and then says, in four facts, its box in section 2 or 3, what it takes and from whom, what it gives and to whom, and what it must not know. Anything wider than that belongs on this page. `tests/integration/test_architecture_placement.py` holds it: every page but this one has the chapter, the chapter opens with a fenced diagram, and it cites this page.
- **Files.** How the engine's files are named, owned and sized is [§AR-core-module-layout](AR-core-module-layout.md#ar-core-module-layout-core-implementation-is-split-by-category); `tests/integration/test_module_categories.py` holds the ownership table and `fissile` holds the size.
- **Assurance is not a component.** [§AR-ci](AR-ci.md#ar-ci-ci-mirrors-the-local-pre-commit-gate), [§AR-benchmarks](AR-benchmarks.md#ar-benchmarks-instruction-counting-benchmarks-for-the-hot-cli-commands) and [§AR-goal-measurement](AR-goal-measurement.md#ar-goal-measurement-goal-and-requirement-meters-live-outside-goals) measure the system rather than sit in it, and are listed apart below; their placement chapters say what each measures.

# Index

One file per page; each H1 declares an `AR-<slug>` ID and the body is its contract, and `§AR-<slug>.<section>` from anywhere in the tree resolves into it. A page may live inline in the doc-comment of the file it describes: its canonical link in this index enrolls it directly, with no stub file ([§FS-check.3.18](../functional-spec/FS-check.md#318-declaration-missing-from-its-kinds-index)), and `grund <ID>` resolves the source declaration and strips its comment markers ([§AR-scanner.4](AR-scanner.md#4-inline-declarations-in-language-doc-comments) lists the doc-comment forms). [§AR-checker](../../crates/grund-core/src/checker/report.rs) is the worked example: its only declaration is the doc-comment of `fn check` in [`crates/grund-core/src/checker/report.rs`](../../crates/grund-core/src/checker/report.rs).

The system:

| ID | Subject |
|---|---|
| [§AR-system](README.md#ar-system-one-engine-nine-components-three-frontends) | one engine, nine components, three frontends — this page |

The components and frontends:

| ID | Subject |
|---|---|
| [§AR-scanner](AR-scanner.md#ar-scanner-how-grund-discovers-declarations-and-citations) | how grund discovers declarations and citations |
| [§AR-checker](../../crates/grund-core/src/checker/report.rs) | how grund validates the scanner's findings — declared and enrolled directly from `crates/grund-core/src/checker/report.rs` |
| [§AR-workspace](AR-workspace.md#ar-workspace-how-the-resolver-config-loader-and-scanner-compose-across-projects) | how the resolver, config loader, and scanner compose across projects |
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
