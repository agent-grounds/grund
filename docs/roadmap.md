# Roadmap

What `grund` plans to ship next, in priority order. Each item has a stable ID — `RM-<slug>` under this repo's `[id] format` ([§FS-config.3.2](functional-spec/FS-config.md#32-id--id-grammar)); `RM` is a configured `[[kinds]]` prefix ([§FS-config.3.4](functional-spec/FS-config.md#34-kinds--recognized-kinds)), so `grund check` validates `§RM-…` citations like any other. Items may be cited from anywhere — commits, PRs, the changelog, other specs. A shipped item is removed: its record is the changelog and the spec it landed in, and whatever cited the milestone cites that spec instead. A cancelled item stays in place with a `~~strikethrough~~` title and a one-line reason. Where an item has a GitHub issue, the item names it.

The check engine, the retrieval surface (`grund <ID>`, `grund refs`, including E2E case manifests), the coverage index (`grund cover`), bulk normalization (`grund fmt`, including `--marker` and `--cross-refs`), config loading (`grund.toml` plus `grund config show` / `grund config validate`), `grund init`, `grund id`, the opt-in grounding floor ([§FS-check.3.6](functional-spec/FS-check.md#36-ungrounded-unit-opt-in)), the token-cheap read surfaces ([§DF-show-default-token-cheap](decisions/functional/DF-show-default-token-cheap.md#df-show-default-token-cheap-grund-show-defaults-to-the-cheap-read-the-full-body-is-opt-in)), the e2e corpus, the benchmark baseline/gate ([§AR-benchmarks](architecture/AR-benchmarks.md#ar-benchmarks-instruction-counting-benchmarks-for-the-hot-cli-commands)), the live registry-name guard ([§FS-distribution.4](functional-spec/FS-distribution.md#4-release-process)), the `grund-core` / `grund-cli` workspace split with data-returning core APIs ([§AR-bindings.2](architecture/AR-bindings.md#2-grund-core-the-only-place-logic-lives)), the optional Cargo LSP server ([§FS-lsp](functional-spec/FS-lsp.md#fs-lsp-grund-ships-an-optional-lsp-server)), and parallel per-file scanning ([§AR-scanner.1](architecture/AR-scanner.md#1-tree-walk)) are all shipped — see `docs/changelog.md`. Two arcs remain. The **distribution arc**: publish on npm and PyPI alongside cargo, including the npm/PyPI LSP packages, and add `grund check --watch`. And the **grounding arc** (the third layer of [§GOAL-agent-grounding.1](goals.md#1-the-three-layers), diff-gated enforcement): build on [§FS-check.3.6](functional-spec/FS-check.md#36-ungrounded-unit-opt-in) and [§FS-cover](functional-spec/FS-cover.md#fs-cover-grund-groups-citations-by-scanned-file) toward a diff-aware co-change gate — implementation cannot change without the spec it grounds in and without a test of it — via a pre-commit / CI recipe that consumes `grund cover` ([§RM-cochange-gate](roadmap.md#rm-cochange-gate-a-pre-commit--ci-recipe--no-impl-change-without-spec-and-test)). Six standalone items sit outside both arcs: [§RM-doc-comment-declarations](roadmap.md#rm-doc-comment-declarations-declarations-only-in-classmethod-doc-comments) tightens code-declaration recognition so a declaration is only seen inside a class/method doc-comment and never a plain inline comment, [§RM-lsp-completion-tab](roadmap.md#rm-lsp-completion-tab-lsp-id-autocomplete-accepted-with-tab) adds LSP ID completion that works with editor Tab acceptance, [§RM-lsp-trigger-conversion-fix](roadmap.md#rm-lsp-trigger-conversion-fix-fix-the-lsp-trigger-conversion) fixes the LSP `$$` trigger conversion, [§RM-positioning](roadmap.md#rm-positioning-the-lychee-contrast-and-the-instruction-count-framing-in-readme-and-landing-copy) keeps the README/landing pitch paired with the benchmark story, [§RM-gap-report](roadmap.md#rm-gap-report-orphan-and-uncovered-id-reports) inverts the [§FS-cover](functional-spec/FS-cover.md#fs-cover-grund-groups-citations-by-scanned-file) index into an orphan / uncovered-ID report, and [§RM-positioning-trace-tools](roadmap.md#rm-positioning-trace-tools-position-grund-against-requirements-traceability-tools-in-readme) extends the README positioning to the requirements-traceability neighbourhood (OFT, Sphinx-Needs, TRLC, Doorstop, Duvet, SARA). The IDed milestones below project both arcs onto reviewable units of work.

## RM-unmarked-heading-error: make unmarked Markdown headings errors in 0.16.0

This milestone shipped in grund 0.16.0: `unmarked-heading` is an error, and its
record is the compatibility notice that landed it,
[§DF-unmarked-markdown-headings.release-note](decisions/functional/DF-unmarked-markdown-headings.md#release-note-release-note),
together with
[§FS-declarations.checks.unmarked-heading.5](functional-spec/FS-declarations.md#checksunmarked-heading5-an-error-in-grund-0160).
The item's own plan is removed, as a shipped item's is; this address, heading
text included, is kept while released changelogs still cite it.

## RM-off-grammar-declaration-error: make off-grammar declarations a check error in 0.16.0

This milestone shipped in grund 0.16.0: `declaration-near-miss` is an error,
and its record is the compatibility notice that landed it,
[§DF-off-grammar-declaration-compatibility.release-note](decisions/functional/DF-off-grammar-declaration-compatibility.md#release-note-release-note),
together with
[§FS-declarations.checks.declaration-near-miss.5](functional-spec/FS-declarations.md#checksdeclaration-near-miss5-a-warning-before-0160-an-error-in-it).
The item's own plan is removed, as a shipped item's is; this address, heading
text included, is kept while released changelogs still cite it.

## RM-unreached-declaration-error: make the unreached-declaration warning an error in 0.16.0

This milestone shipped in grund 0.16.0: `unreached-declaration` is an error on
the ordinary `must` channel, and its record is the compatibility notice that
landed it,
[§DF-chapter-rule-reaches-every-declaration.release-note](decisions/functional/DF-chapter-rule-reaches-every-declaration.md#release-note-release-note),
together with
[§FS-rules.checks.unreached-declaration](functional-spec/FS-rules.md#checksunreached-declaration-unreached-declaration).
The item's own plan is removed, as a shipped item's is; this address, heading
text included, is kept while the changelog and that notice still cite it —
#362's bullet in the 0.15.0 release, written before the promotion, and the
notice, which the 0.16.0 release publishes
([§FS-distribution.4.6.4](functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions)).
Retiring the address is a change of its own: the notice is a section of a
decision record and may be reworded like any other, so the repoint the lead
above prescribes need not wait for a release. It is not a deadline item.

GitHub: [#368](https://github.com/agent-grounds/grund/issues/368).

## RM-distribution: cargo + npm + pypi from one engine

Per [§FS-distribution](functional-spec/FS-distribution.md#fs-distribution-grund-distribution-targets) and [§AR-bindings](architecture/AR-bindings.md#ar-bindings-target-shape-for-exposing-the-rust-engine-on-three-platforms). Builds on the shipped workspace split ([§AR-bindings.2](architecture/AR-bindings.md#2-grund-core-the-only-place-logic-lives)).

### 1. What

napi-rs binding for npm; PyO3 binding for PyPI; CI publish jobs for all three registries (`grund-core` first, in dependency order). Each publish job builds the CLI binary with profile-guided optimization via `scripts/pgo-build.sh` ([§DA-pgo-release](decisions/architectural/DA-pgo-release.md#da-pgo-release-distributed-binaries-are-pgo-built-trained-on-the-benchmark-workload), [§FS-distribution.4](functional-spec/FS-distribution.md#4-release-process)) — wired for the crates.io `grund` and `grund-lsp` packages already, extended to the prebuilt npm and PyPI CLI/LSP binaries here.

### 2. Why now

`grund` is only viable as a CI dependency for non-Rust projects once it ships on their native package manager ([§GOAL-multi-language](goals.md#goal-multi-language-same-engine-three-platforms)).

### 3. Measurable

Integration test runs the same spec corpus through all three bindings and asserts byte-identical reports ([§AR-goal-measurement.2](architecture/AR-goal-measurement.md#2-goal-meters)).

## RM-watch: implement grund check --watch

Per [§FS-check.6](functional-spec/FS-check.md#6-watch-mode---watch). The editor-less "every save" loop [§GOAL-fast-feedback](goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible) exists for — re-run `grund check` on every change under the scanned tree, clearing prior output each run.

Together with [§FS-lsp](functional-spec/FS-lsp.md#fs-lsp-grund-ships-an-optional-lsp-server), this ships the live feedback loop: LSP for editor users, `grund check --watch` for terminal users and editor setups that do not speak LSP.

### 1. What

`--watch` on `grund check`: filesystem-notification-driven, debounced, no polling and no configurable interval. Each run is byte-identical to a plain `grund check` on the tree's current state; on Ctrl-C the process exits with the last completed run's exit code. Non-interactive — no TUI, no key bindings ([§FS-non-goals.10](functional-spec/FS-non-goals.md#10-interactive-mode)), no network ([§FS-non-goals.11](functional-spec/FS-non-goals.md#11-network-access-during-a-check)).

### 2. Why now

`grund-lsp` ([§FS-lsp](functional-spec/FS-lsp.md#fs-lsp-grund-ships-an-optional-lsp-server)) covers editor users; `--watch` covers everyone else with zero editor configuration, and it is small now that the engine is a library ([§AR-bindings.2](architecture/AR-bindings.md#2-grund-core-the-only-place-logic-lives)). The watcher calls `grund-core::scan`/`check` rather than re-implementing the walk.

### 3. Measurable

An e2e fixture starts `grund check --watch` on a clean fixture (asserts silent first run), writes a file that introduces a dangling ref (asserts the next run prints it), removes the bad citation (asserts the run goes silent again), then sends SIGINT (asserts exit code matches the last run). A second fixture asserts `--format=json` emits one self-contained report per run.

## RM-lsp-completion-tab: LSP ID autocomplete accepted with Tab

Implements the reserved `textDocument/completion` capability from [§FS-lsp.1.5](functional-spec/FS-lsp.md#15-capabilities-reserved-for-later), building on the shipped LSP server [§FS-lsp](functional-spec/FS-lsp.md#fs-lsp-grund-ships-an-optional-lsp-server). The result is the expected editor loop: type a marker or trigger prefix, narrow to the ID, accept the selected completion with the editor's normal Tab binding, and get the canonical citation inserted.

### 1. What

`grund-lsp` returns completion items for declared IDs in the document's resolved project config. Completion triggers include the configured marker prefix (`§F` under defaults), the configured typing trigger prefix (`$$F` under defaults), and a partially typed ID immediately after either prefix. Applying a completion replaces only the active prefix/token range with the configured marker plus the chosen ID; it does not touch surrounding prose, existing markdown links, or another citation on the same line. Completion details show the declaration title and source path, and items sort by exact prefix match, then kind order, then ID for deterministic output ([§FS-errors.4](functional-spec/FS-errors.md#4-determinism)). The server cannot own each client's Tab key, so the contract is that completion items use plain text edits and ranges that work with standard Tab acceptance in Helix, Neovim, Zed, VSCode, and eglot/lsp-mode; README snippets add the client-side Tab mapping only where the editor requires it.

### 2. Why now

The shipped LSP already gives diagnostics, navigation, hover, links, and trigger formatting ([§FS-lsp](functional-spec/FS-lsp.md#fs-lsp-grund-ships-an-optional-lsp-server)). The remaining daily friction is remembering exact IDs while writing a citation. Completion turns the LSP from a checker into an authoring aid without changing the CLI contract.

### 3. Measurable

LSP tests open a fixture workspace, request completions after `§F`, `$$F`, and a longer prefix, and assert the returned labels, sort order, and text-edit ranges. Applying the edit produces exactly one canonical `§<ID>` citation, `grund check` resolves it, and the same fixture covers a workspace member with a non-default marker/trigger.

## RM-lsp-trigger-conversion-fix: fix the LSP trigger conversion

Fixes and hardens the shipped live trigger transform [§FS-lsp.1.4](functional-spec/FS-lsp.md#14-live-trigger-transform). The existing LSP milestone is shipped ([§FS-lsp](functional-spec/FS-lsp.md#fs-lsp-grund-ships-an-optional-lsp-server)), but the `$$` authoring path needs to be reliable before completion and normal editing can depend on it.

### 1. What

`textDocument/onTypeFormatting` converts the configured typing trigger (`$$` by default) to the configured marker only when the text after the trigger is a valid citation start for the document's resolved config. It must handle the common typing paths: `$$FS-foo` typed continuously, `$$` followed by a completion choice, trigger text at the start of a line, trigger text inside a comment, and trigger text next to another citation on the same line. The returned edit is minimal, UTF-16-correct, idempotent, and never rewrites literal money/prose `$$` that is not followed by a recognized ID prefix. Workspace-member config and non-default markers/triggers follow the same lookup path as `grund fmt` and diagnostics ([§FS-workspace.5](functional-spec/FS-workspace.md#5-command-scope)).

### 2. Why now

[§FS-lsp.1.4](functional-spec/FS-lsp.md#14-live-trigger-transform) is what makes `§` practical to type without leaving the keyboard. If the conversion is flaky, the LSP's most basic authoring workflow feels broken even when diagnostics and navigation are correct.

### 3. Measurable

Focused LSP tests cover continuous typing, completion-adjacent typing, line-start and comment positions, UTF-16 ranges, adjacent citations, non-default trigger/marker config, and negative `$$` prose cases. The same fixture should pass `grund fmt --check` after applying the LSP edit, proving live conversion and bulk normalization agree.

## RM-cochange-gate: a pre-commit / CI recipe — no impl change without spec and test

The strong form of the discipline ([§GOAL-agent-grounding.1](goals.md#1-the-three-layers), diff-gated enforcement): a changed source file must be grounded ([§FS-check.3.6](functional-spec/FS-check.md#36-ungrounded-unit-opt-in)), and the change must also touch the spec it cites *or* a test of it, with an explicit escape hatch for refactors. This is diff-aware — a function of `(tree, base ref, config)`, not `(tree, config)` — and it leans on `grund cover` ([§FS-cover](functional-spec/FS-cover.md#fs-cover-grund-groups-citations-by-scanned-file)) plus a git diff, so it lives in the recipe layer, **not** in `grund-core` (a third first-party surface is out of scope, [§FS-non-goals.12](functional-spec/FS-non-goals.md#12-surfaces-outside-grund-core-and-the-lsp-transport); the engine reads no history, [§FS-non-goals.6](functional-spec/FS-non-goals.md#6-decision-database-audit-log-history-tracking)). Tiering rationale in [§DF-require-grounding](decisions/functional/DF-require-grounding.md#df-require-grounding-an-opt-in-check-that-every-source-file-cites-a-spec).

GitHub: [#26](https://github.com/agent-grounds/grund/issues/26).

[§FS-check.3.6](functional-spec/FS-check.md#36-ungrounded-unit-opt-in) proves files are grounded at rest; it does not prove that a behavior change came with a spec or test update. The co-change gate is therefore the highest-value remaining "agent discipline" item: use `grund cover` plus git diff to connect changed implementation files to the specs and tests that justify the change.

### 1. What

A documented pre-commit hook / CI step (a recipe alongside the `grund check` hook in the README, and a worked example under `examples/`), not a shipped binary. Given a base ref it: (a) lists changed source files; (b) for each, gets its cited IDs from `grund cover` and fails `ungrounded change` if a changed hunk falls under no citation; (c) requires the diff to also touch the declaring file of one of those IDs *or* a test / `§E2E-` case that cites one of them; (d) honours an escape hatch — a commit trailer (e.g. `Grund-Cochange: refactor`) or a `grund:no-cochange` pragma on a hunk — for legitimate refactors, kept greppable so a reviewer sees every waiver. Which paths count as "source" vs. "test", whether (c) needs spec *and* test or *either*, and how the base ref is chosen are knobs the repo sets in the recipe, not in `grund-core` — so the "two installs agree" contract ([§FS-non-goals.13](functional-spec/FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)) and the no-config-on-severity rule ([§FS-non-goals.9](functional-spec/FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization)) are untouched.

### 2. Why now

[§FS-check.3.6](functional-spec/FS-check.md#36-ungrounded-unit-opt-in) makes "every file is grounded" true at rest; this makes "every change stays grounded, and ships with a test" true at the diff. It is unsound by construction — without an AST it cannot tell a behavioral hunk from a cosmetic one — so the escape hatch is mandatory and the gate is advisory-strict, not a proof. That trade is the reason it is a recipe a repo opts into, not engine behavior.

### 3. Measurable

The recipe, run in this repo's CI on a synthetic branch, fails a commit that edits a `src/` file without touching its spec or a test, passes the same commit once a `Grund-Cochange:` trailer is added, and passes a commit that edits the spec and a test together. The `examples/` worked example carries golden output the e2e harness can diff.

## RM-doc-comment-declarations: declarations only in class/method doc-comments

Per [§DISC-doc-comment-declarations](discussions/proposals/2026-05-21-doc-comment-declarations.md#disc-doc-comment-declarations-declarations-live-only-in-classmethod-doc-comments-never-inline). Tightens the [§AR-scanner.4](architecture/AR-scanner.md#4-inline-declarations-in-language-doc-comments) recognizer so a code-resident declaration is seen only inside a doc-comment that documents the immediately-following definition (class, method, module, …), never a plain inline or trailing comment — with a default-on `[scan]` switch that restores today's any-comment behavior. Composes with [§FS-declarations.checks.declaration-near-miss](functional-spec/FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss): the gate drops the phantom declaration, the near-miss optionally surfaces "this looks like a declaration but is ignored." The marker/position classifier this milestone planned already exists — `comment_block.rs` decides doc comment or inline comment per block for the inline citation sites of [§FS-inline-citation-style.1.1](functional-spec/FS-inline-citation-style.md#11-doc-comments-are-not-sites) — so the declaration gate reuses it rather than building a second one.

### 1. What

The declaration recognizer splits the comment-prefix alternation in two: a *declaration-prefix* set holding only doc-comment markers (selected per file extension), and the existing any-comment set that citations keep using unchanged. For *marker* languages (Rust `///`/`//!`, Java/JS/TS `/** */`, C# `///`, …) a declaration is recognized only behind the doc marker; a bare `//`/`/* */` is a regular comment and never declares — closing the `//[/!]?` widening in `grammar.rs` that lets a plain `//` line declare today. For *position* languages where the regular marker is also the doc marker (Go `//`, Ruby `#`, …), the declaration is emitted only when its comment block is immediately followed by a line-anchored definition-starter (`func`, `class`, `def`, …) — recognition, not parsing ([§FS-non-goals.3](functional-spec/FS-non-goals.md#3-code-ast-parsing), [§AR-scanner](architecture/AR-scanner.md#ar-scanner-how-grund-discovers-declarations-and-citations)). Python docstrings ([§AR-scanner.4](architecture/AR-scanner.md#4-inline-declarations-in-language-doc-comments)) are unchanged. A new `[scan] declarations_in_doc_comments` key (default `true`, [§FS-config.3.5](functional-spec/FS-config.md#35-scan--what-gets-scanned)) restores the legacy any-comment recognizer when set `false`; it is a recognizer toggle, not a severity knob ([§FS-non-goals.9](functional-spec/FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization)), so the two-installs-agree contract ([§FS-non-goals.13](functional-spec/FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)) holds. Citations are untouched: a `§<ID>` in any comment still resolves and climbs.

### 2. Why now

Surfaced from a real LSP session: a citation note written in a plain `//` comment with the `§` marker dropped (`// FS-check.3.9 / …`) is silently read as a declaration of `FS-check` *inside a function body*, colliding with the real spec declaration and raising a `duplicate declaration` diagnostic. The recognizer is looser than [§AR-scanner.4](architecture/AR-scanner.md#4-inline-declarations-in-language-doc-comments)'s own framing, which already says an inline declaration lives in "the class, method, module, or package doc-comment." This realigns the recognizer with the spec and removes a sharp edge that bites authors and agents who write inline citation notes — directly serving [§GOAL-friendliness-first](goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible) and [§GOAL-zero-config](goals.md#goal-zero-config-works-on-any-conformant-tree) (default-on, no config).

### 3. Measurable

E2E fixtures: a plain `//` ID note inside a function body is *not* a declaration (`grund list` does not show it, no `duplicate declaration`); the same file with the switch off restores the declaration; a Rust `///` declaration is still recognized; a Go `//` block immediately above `func`/`type` is recognized while a Go `//` note not above a definition is not. The recognizer holds its shape across all three bindings ([§GOAL-multi-language](goals.md#goal-multi-language-same-engine-three-platforms)). Run on this repo (after the two `//` Rust fixtures move to `///`), `grund check` stays clean.

## RM-positioning: the Lychee contrast and the instruction-count framing in README and landing copy

`grund` and Lychee run together in this repository's CI
([§FS-non-goals.1](functional-spec/FS-non-goals.md#1-markdown-link-validation),
[§AR-ci.3](architecture/AR-ci.md#3-current-hooks)). The README already explains
their different jobs. This documentation milestone keeps that explanation factual
and connects the archival measurements to current reporting; it adds no benchmark
machinery or measurements ([§REQ-readme.evidence](requirements/REQ-readme.md#evidence-positioning-and-performance-claims-have-evidence)).

### 1. What

Reuse the existing README paragraph, correcting Lychee's scope to Markdown and HTML
as supported by the primary-source ledger in
[§REL-traceability-tools.work](related-work/REL-traceability-tools.md#work-what-the-work-is).
Describe Grund's citation resolution and declared constraints without implying proof
of semantic implementation correctness.

Retire the undated throughput badge and give a short path to [the archival report](benchmarks.md):
the 2026-05-20 local elapsed-time run and historical instruction-count snapshot remain
available with their provenance. Explain current generated-fixture reporting
([§AR-benchmarks.1.1](architecture/AR-benchmarks.md#11-generated-fixtures-never-this-repository))
against the PR base branch ([§AR-ci.5.1](architecture/AR-ci.md#51-pull-requests-and-pushes)).
Instruction count is a workload-cost proxy under binary/input/build assumptions,
including PGO comparability ([§AR-benchmarks.2](architecture/AR-benchmarks.md#2-why-instruction-counts-not-wall-clock),
[§AR-benchmarks.5](architecture/AR-benchmarks.md#5-comparing-two-revisions)); regression
limits are not enforced ([§AR-ci.5.2](architecture/AR-ci.md#52-regression-limits)).

### 2. Why now

Readers need to distinguish link validation from citation checking, and historical
timing evidence from current CI reporting. Reusing the introduction and linking to
the detailed evidence keeps the common path concise
([§REQ-readme.1](requirements/REQ-readme.md#1-what-it-must-say)).

### 3. Measurable

The README links to the archive and current reporting, with no throughput badge or
duplicate introductory block. The report preserves measured data and explains unlike
workloads, build assumptions and unenforced instruction-regression limits. Future
measurement instructions write elsewhere. Factual review and link/citation checks
support the text regression checks
([§REQ-readme.evidence](requirements/REQ-readme.md#evidence-positioning-and-performance-claims-have-evidence)).

## RM-gap-report: orphan and uncovered ID reports

A proposed inverse of [§FS-cover.2](functional-spec/FS-cover.md#2-behaviour):
instead of "what does this file cite?" it would answer "which declared IDs have
nothing climbing into them?" The command below is design work, not a shipped
capability or a commitment to parity with another tool. Current retrieval and
checking tasks are described in
[§REL-traceability-tools.work.matrix](related-work/REL-traceability-tools.md#workmatrix-reader-tasks).

The orphan half already exists as `grund list --unused` ([§FS-list](functional-spec/FS-list.md#fs-list-grund-lists-every-declared-id)), which lists the declarations nothing cites; what remains is the *unclimbed* view and the report shape below. GitHub: [#89](https://github.com/agent-grounds/grund/issues/89) asks for the deliberately-uncited marker that lets that list be driven to zero.

### 1. What

A new read-only command, `grund gap [--kind <K[,...]>] [--format text|json]`, that re-uses the existing citation graph and reports:

- *orphans*: declared IDs with zero inbound citations, ignoring kinds at the top of the climbing chain (`GRUND`, `GOAL` under the default config).
- *unclimbed*: declared IDs whose only inbound citations come from kinds that violate the climbing rule — e.g. an `FS-` that no `AR-`, `E2E-`, or code site cites.

Output is sorted lexicographically by `(kind, id)` for byte-identical reproducibility ([§FS-errors.4](functional-spec/FS-errors.md#4-determinism)). The command never changes its exit code on found gaps — it is a report, not a check; severity/exit-code customization stays out of the engine ([§FS-non-goals.9](functional-spec/FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization)). CI use is a recipe (same shape as [§RM-cochange-gate](roadmap.md#rm-cochange-gate-a-pre-commit--ci-recipe--no-impl-change-without-spec-and-test)): pipe the JSON, gate on the count. Dangling citations are already `grund check` errors and are not duplicated here.

### 2. Why now

[§FS-cover.2](functional-spec/FS-cover.md#2-behaviour) groups citations by scanned
file, including files with no citations, without judging sufficient coverage. An
inverted view could help a reader inspect inbound relationships directly. Its
usefulness must be assessed on that task, independently of the documentation
correction in [§RM-positioning-trace-tools](roadmap.md#rm-positioning-trace-tools-position-grund-against-requirements-traceability-tools-in-readme).

### 3. Measurable

E2E fixtures: a clean tree returns no orphans; deleting an `E2E-` that cited an `FS-` makes that `FS-` show up as `unclimbed` in the next `grund gap`. `--format=json` emits one NDJSON record per gap, sorted as above. Run on this repo, `grund gap` is silent (the repo self-hosts the floor).

## RM-positioning-trace-tools: position grund against requirements-traceability tools in README

[§RM-positioning](roadmap.md#rm-positioning-the-lychee-contrast-and-the-instruction-count-framing-in-readme-and-landing-copy)
covers links and performance evidence. This milestone connects the README to the
existing requirements-traceability explanation, organised around reader tasks
([§REL-traceability-tools.work.matrix](related-work/REL-traceability-tools.md#workmatrix-reader-tasks)).
It corrects documentation, not product behavior.

### 1. What

Add one concise README link and reuse the existing resolver examples. In the related
work, replace rankings and historical generalisations with sourced task-oriented prose:
read a cited section ([§FS-show.2.2](functional-spec/FS-show.md#22-section)), check
citation targets ([§FS-check.3.1](functional-spec/FS-check.md#31-dangling-citation),
[§FS-check.3.2](functional-spec/FS-check.md#32-missing-section)), and query citations
by scanned file ([§FS-cover.2](functional-spec/FS-cover.md#2-behaviour)). Resolving
a citation does not prove semantic implementation correctness or sufficient coverage.

Describe supported chapter and citation constraints
([§FS-rules.3.1](functional-spec/FS-rules.md#31-chapter-presence),
[§FS-rules.3.2](functional-spec/FS-rules.md#32-outbound-citation-count),
[§FS-rules.3.3](functional-spec/FS-rules.md#33-per-target-coverage),
[§FS-rules.3.4](functional-spec/FS-rules.md#34-inbound-citation-count-and-prohibition))
separately from fixed severity
([§FS-non-goals.9](functional-spec/FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization))
and absent arbitrary engine scripting
([§FS-non-goals.12.1](functional-spec/FS-non-goals.md#121-plugins-or-scripting-hooks-inside-the-engine)).
Do not derive a new interchange policy from those boundaries. The gap command above
and the design work in #458/#468 remain proposals, not shipped capabilities or
prerequisites for this correction.

### 2. Why now

Concrete tasks let a reader assess Grund beside an existing requirements tool without
blanket claims about that tool's purpose or data model. The detailed home already exists;
a short README path makes it discoverable without duplicating it
([§REQ-readme.1](requirements/REQ-readme.md#1-what-it-must-say)).

### 3. Measurable

The README links to the existing comparison. Each retained competitor capability claim
has scoped primary-source support with a checked date and version or unversioned status.
The prose preserves published section coordinates, describes shipped tasks and precise
boundaries, and makes no future gap or coverage-parity promise. Review the claims and
validate citations and links alongside text regression checks
([§REQ-readme.evidence](requirements/REQ-readme.md#evidence-positioning-and-performance-claims-have-evidence)).

## RM-workspace-absorbed-scan-error: flip the absorbed-scan warning to an error

This milestone shipped in grund 0.16.0: a `[workspace]` block whose members
swallow its own scan is a `members`-line config error, and its record is the
compatibility notice that landed it,
[§DF-absorbed-scan-warning.release-note](decisions/functional/DF-absorbed-scan-warning.md#release-note-release-note), together with [§FS-check.3.30](functional-spec/FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan).
The item's own plan is removed, as a shipped item's is; this address, heading
text included, is kept while released changelogs still cite it.
