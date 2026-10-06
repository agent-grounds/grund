# Roadmap

What `grund` plans to ship next, in priority order. Each item has a stable ID — `RM-<slug>` under this repo's `[id] format` ([§FS-config.3.2](functional-spec/FS-config.md#32-id--id-grammar)); `RM` is a configured `[[kinds]]` prefix ([§FS-config.3.4](functional-spec/FS-config.md#34-kinds--recognized-kinds)), so `grund check` validates `§RM-…` citations like any other. Items may be cited from anywhere — commits, PRs, the changelog, other specs. A shipped item is removed: its record is the changelog and the spec it landed in, and whatever cited the milestone cites that spec instead. A cancelled item stays in place with a `~~strikethrough~~` title and a one-line reason. Where an item has a GitHub issue, the item names it.

The check engine, the retrieval surface (`grund <ID>`, `grund refs`, including E2E case manifests), the coverage index (`grund cover`), bulk normalization (`grund fmt`, including `--marker` and `--cross-refs`), config loading (`grund.toml` plus `grund config show` / `grund config validate`), `grund init`, `grund id`, the opt-in grounding floor ([§FS-check.3.6](functional-spec/FS-check.md#36-ungrounded-unit-opt-in)), the token-cheap read surfaces ([§DF-show-default-token-cheap](decisions/functional/DF-show-default-token-cheap.md#df-show-default-token-cheap-grund-show-defaults-to-the-cheap-read-the-full-body-is-opt-in)), the e2e corpus, the benchmark baseline/gate ([§AR-benchmarks](architecture/AR-benchmarks.md#ar-benchmarks-instruction-counting-benchmarks-for-the-hot-cli-commands)), the live registry-name guard ([§FS-distribution.4](functional-spec/FS-distribution.md#4-release-process)), the `grund-core` / `grund-cli` workspace split with data-returning core APIs ([§AR-bindings.2](architecture/AR-bindings.md#2-grund-core-the-only-place-logic-lives)), the optional Cargo LSP server ([§FS-lsp](functional-spec/FS-lsp.md#fs-lsp-grund-ships-an-optional-lsp-server)), and parallel per-file scanning ([§AR-scanner.1](architecture/AR-scanner.md#1-tree-walk)) are all shipped — see `docs/changelog.md`. Two arcs remain. The **distribution arc**: publish on npm and PyPI alongside cargo, including the npm/PyPI LSP packages, and add `grund check --watch`. And the **grounding arc** (the third layer of [§GOAL-agent-grounding.1](goals.md#1-the-three-layers), diff-gated enforcement): build on [§FS-check.3.6](functional-spec/FS-check.md#36-ungrounded-unit-opt-in) and [§FS-cover](functional-spec/FS-cover.md#fs-cover-grund-groups-citations-by-scanned-file) toward a diff-aware co-change gate — implementation cannot change without the spec it grounds in and without a test of it — via a pre-commit / CI recipe that consumes `grund cover` ([§RM-cochange-gate](roadmap.md#rm-cochange-gate-an-opt-in-commit-msg--ci-recipe-for-spec-and-test-edits)). Six standalone items sit outside both arcs: [§RM-doc-comment-declarations](roadmap.md#rm-doc-comment-declarations-declarations-only-in-classmethod-doc-comments) tightens code-declaration recognition so a declaration is only seen inside a class/method doc-comment and never a plain inline comment, [§RM-lsp-completion-tab](roadmap.md#rm-lsp-completion-tab-lsp-id-autocomplete-accepted-with-tab) adds LSP ID completion that works with editor Tab acceptance, [§RM-lsp-trigger-conversion-fix](roadmap.md#rm-lsp-trigger-conversion-fix-fix-the-lsp-trigger-conversion) fixes the LSP `$$` trigger conversion, [§RM-positioning](roadmap.md#rm-positioning-the-lychee-contrast-and-the-instruction-count-framing-in-readme-and-landing-copy) keeps the README/landing pitch paired with the benchmark story, [§RM-gap-report](roadmap.md#rm-gap-report-orphan-and-uncovered-id-reports) inverts the [§FS-cover](functional-spec/FS-cover.md#fs-cover-grund-groups-citations-by-scanned-file) index into an orphan / uncovered-ID report, and [§RM-positioning-trace-tools](roadmap.md#rm-positioning-trace-tools-position-grund-against-requirements-traceability-tools-in-readme) extends the README positioning to the requirements-traceability neighbourhood (OFT, Sphinx-Needs, TRLC, Doorstop, Duvet, SARA). The IDed milestones below project both arcs onto reviewable units of work.

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

Per [§FS-check.6](functional-spec/FS-check.md#6-watch-mode---watch). The terminal "every save" loop [§GOAL-fast-feedback](goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible) exists for — re-run `grund check` on effective local-input changes, with terminal screen ownership governed by [§FS-check.6.2.2](functional-spec/FS-check.md#622-owned-terminal-screen).

Together with [§FS-lsp](functional-spec/FS-lsp.md#fs-lsp-grund-ships-an-optional-lsp-server), this ships the live feedback loop: LSP for editor users, `grund check --watch` for terminal users and editor setups that do not speak LSP.

### 1. What

`--watch` on `grund check`: filesystem-notification-driven, debounced, no polling and no configurable interval. Each run is byte-identical to a plain `grund check` on the tree's current state; on Ctrl-C the process exits with the last completed run's exit code. Non-interactive — no TUI, no key bindings ([§FS-non-goals.10](functional-spec/FS-non-goals.md#10-interactive-mode)), no network ([§FS-non-goals.11](functional-spec/FS-non-goals.md#11-network-access-during-a-check)).

### 2. Why now

`grund-lsp` ([§FS-lsp](functional-spec/FS-lsp.md#fs-lsp-grund-ships-an-optional-lsp-server)) covers editor users; `--watch` covers everyone else with zero editor configuration, and it is small now that the engine is a library ([§AR-bindings.2](architecture/AR-bindings.md#2-grund-core-the-only-place-logic-lives)). The watcher calls `grund-core::scan`/`check` rather than re-implementing the walk.

### 3. Measurable

Bounded subprocess tests start `grund check --watch` and compare the immediate and subsequent reports with one-shot checks: clean text prints `success`, a dangling citation prints the ordinary finding, and repair returns to `success`. SIGINT returns the last fully flushed status. JSON emits only ordinary finding NDJSON; clean runs and some run boundaries are invisible ([§FS-check.6.2.1](functional-spec/FS-check.md#621-exact-stream-contract)). A private test-build completion observer proves empty JSON recovery without changing production streams ([§AR-bindings.3](architecture/AR-bindings.md#3-cratesgrund-cli-the-cli-binary)).

Fake-clock/event and barrier tests pin 100 ms quiet / 500 ms maximum debounce, startup coverage and serialized pending work. Real filesystem tests cover atomic saves, creation/rename/deletion, effective config/ignore/catalog/member/link changes and root replacement. Injected backend failures and PTYs pin loss recovery, fatal exit, interrupt cleanup and independent-stream screen restoration ([§FS-check.6.1](functional-spec/FS-check.md#61-change-detection), [§FS-check.6.3](functional-spec/FS-check.md#63-lifecycle), [§FS-check.6.2.2](functional-spec/FS-check.md#622-owned-terminal-screen)). Existing one-shot goldens stay authoritative; help and captures follow implementation.

## RM-lsp-completion-tab: LSP ID autocomplete accepted with Tab

Implemented in source under [§FS-lsp.1.6](functional-spec/FS-lsp.md#16-declared-id-completion): type a marker or trigger prefix, narrow to the ID, and accept one canonical citation with the editor's acceptance key. Core and protocol acceptance coverage is present; execution and release validation remain pending. This does not assign release membership or timing.

### 1. What

`grund-lsp` returns completion items for declared IDs in the document's resolved project config. Completion triggers include the configured marker prefix (`§F` under defaults), the configured typing trigger prefix (`$$F` under defaults), and a partially typed ID immediately after either prefix. Applying a completion replaces only the active prefix/token range with the configured marker plus the chosen ID; it does not touch surrounding prose, existing markdown links, or another citation on the same line. Completion details show the declaration title and source path, and items sort by exact prefix match, then kind order, then ID for deterministic output ([§FS-lsp.1.6.2](functional-spec/FS-lsp.md#162-candidates-resolution-and-ordering)). Clients own their acceptance keys: the [setup guide](user-facing/lsp.md#write-a-citation) records default bindings and optional mappings for Helix, Neovim, Zed, VSCode, and eglot/lsp-mode. Helix cycles with Tab and accepts with Enter.

### 2. Why now

The shipped LSP already gives diagnostics, navigation, hover, links, and trigger formatting ([§FS-lsp](functional-spec/FS-lsp.md#fs-lsp-grund-ships-an-optional-lsp-server)). The remaining daily friction is remembering exact IDs while writing a citation. Completion turns the LSP from a checker into an authoring aid without changing the CLI contract.

### 3. Measurable

LSP tests open a fixture workspace, request completions after `§F`, `$$F`, and a longer prefix, and assert the returned labels, sort order, and text-edit ranges. Applying the edit produces exactly one canonical `§<ID>` citation, `grund check` resolves it, and the same fixture covers a workspace member with a non-default marker/trigger.

## RM-lsp-trigger-conversion-fix: fix the LSP trigger conversion

Completion interaction coverage is implemented alongside [§FS-lsp.1.6.4](functional-spec/FS-lsp.md#164-snapshot-and-live-transform-interaction). Baseline investigation reproduced no generic trigger defect, so the shipped live transform [§FS-lsp.1.4](functional-spec/FS-lsp.md#14-live-trigger-transform) retains its behavior. The new request-sequence tests await execution; this is not a release commitment.

### 1. What

`textDocument/onTypeFormatting` converts the configured typing trigger (`$$` by default) to the configured marker only when the text after the trigger is a valid citation start for the document's resolved config. It must handle the common typing paths: `$$FS-foo` typed continuously, `$$` followed by a completion choice, trigger text at the start of a line, trigger text inside a comment, and trigger text next to another citation on the same line. The returned edit is minimal, UTF-16-correct, idempotent, and never rewrites literal money/prose `$$` that is not followed by a recognized ID prefix. Workspace-member config and non-default markers/triggers follow the same lookup path as `grund fmt` and diagnostics ([§FS-workspace.5](functional-spec/FS-workspace.md#5-command-scope)).

### 2. Why now

[§FS-lsp.1.4](functional-spec/FS-lsp.md#14-live-trigger-transform) is what makes `§` practical to type without leaving the keyboard. If the conversion is flaky, the LSP's most basic authoring workflow feels broken even when diagnostics and navigation are correct.

### 3. Measurable

Focused LSP tests cover continuous typing, completion-adjacent typing, line-start and comment positions, UTF-16 ranges, adjacent citations, non-default trigger/marker config, and negative `$$` prose cases. The same fixture should pass `grund fmt --check` after applying the LSP edit, proving live conversion and bulk normalization agree.

## RM-cochange-gate: an opt-in commit-msg / CI recipe for spec and test edits

The optional discipline serves [§GOAL-agent-grounding.1](goals.md#1-the-three-layers): every changed configured source file must directly cite an eligible, resolved declaration, and the comparison must contain both a declaring-file edit and a configured test-file edit for one shared target, unless bounded commit trailers waive the missing evidence. [§FS-cochange-recipe](functional-spec/FS-cochange-recipe.md#fs-cochange-recipe-an-opt-in-git-recipe-reports-related-declaration-and-test-edits) specifies this file-level recipe. Git comparison and Grund queries inspect the same exact trees; the recipe lives outside the engine under [§FS-non-goals.6](functional-spec/FS-non-goals.md#6-decision-database-audit-log-history-tracking) and [§FS-non-goals.12](functional-spec/FS-non-goals.md#12-surfaces-outside-grund-core-and-the-lsp-transport). Ordinary [§FS-check.3.6](functional-spec/FS-check.md#36-ungrounded-unit-opt-in) remains independent.

GitHub: [#474](https://github.com/agent-grounds/grund/issues/474), following [#26](https://github.com/agent-grounds/grund/issues/26).

[§FS-cover](functional-spec/FS-cover.md#fs-cover-grund-groups-citations-by-scanned-file) supplies citation facts, not changed-line coverage. This workflow reports related edits and visible waiver reasons; it proves neither semantic correctness nor test execution.

### 1. What

A copyable Python recipe under `examples/cochange/`, documented beside README hook/CI guidance, supports Grund 0.16.1. Commit-msg evaluates an isolated index tree with an explicit base and supplied final message; CI evaluates the whole PR from a unique merge base and reads its commits. Direct targets match by project and declaration root. Both evidence classes must match one shared target. JSON `Grund-Cochange:` trailers name exact changed paths, missing `spec`/`test` obligations and reasons ([§FS-cochange-recipe.waivers](functional-spec/FS-cochange-recipe.md#waivers-json-commit-trailers-and-commit-local-scopes)). They never waive eligible grounding, ordinary checks or incomplete data. No shipped command, core history API, rule-language extension or standing verdict changes are introduced.

### 2. Why now

File-level evidence can accept an unrelated edit inside a matched file and reject a valid unchanged-contract fix until explicitly waived ([§FS-cochange-recipe.evidence](functional-spec/FS-cochange-recipe.md#evidence-both-edits-for-one-resolved-direct-target)). An unchanged-contract fix supplies a related test edit and a reason-bearing spec waiver. A refactor may waive both evidence classes, but still needs eligible grounding. Runtime test results stay with CI. Adoption is explicit; demonstrations in Grund CI do not enable this gate for Grund contributions.

### 3. Measurable

Shared-runner goldens and synthetic Git tests exercise missing grounding/evidence, spec-only and test-only refusals, unrelated and split targets, complete evidence, bounded waiver remedies, exact staged/CI parity and truthful data refusals ([§FS-cochange-recipe.examples](functional-spec/FS-cochange-recipe.md#examples-maintained-walkthrough-tests-and-opt-in-guidance)). The maintained walkthrough enters CI through [§FS-examples.5.4](functional-spec/FS-examples.md#54-commandexternal-invokes-an-explicit-json-argv). The broader matrix includes histories, renames/deletions, workspace identities and malformed/stale trailers.

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

Output is sorted lexicographically by `(kind, id)` for byte-identical reproducibility ([§FS-errors.4](functional-spec/FS-errors.md#4-determinism)). The command never changes its exit code on found gaps — it is a report, not a check; severity/exit-code customization stays out of the engine ([§FS-non-goals.9](functional-spec/FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization)). CI use is a recipe (same shape as [§RM-cochange-gate](roadmap.md#rm-cochange-gate-an-opt-in-commit-msg--ci-recipe-for-spec-and-test-edits)): pipe the JSON, gate on the count. Dangling citations are already `grund check` errors and are not duplicated here.

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
