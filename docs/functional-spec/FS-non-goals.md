# FS-non-goals: what grund will deliberately not do

This page holds the line against feature creep: each entry below is something `grund` could plausibly grow into and deliberately does not do. A non-goal is a commitment, not a postponement: turning one into a goal takes a decision record under `docs/decisions/architectural/` that overturns this spec.

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, lead), [§FS-terms.terms.2](FS-terms.md#terms2-citations) (citation),
[§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (stub, doc-comment), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, workspace), [§FS-terms.terms.5](FS-terms.md#terms5-findings)
(finding, severity, suggestion, anchor), and [§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations) (value, component, binding).

## 1. Markdown link validation

`grund` does **not** validate `[text](url)` links, anchor `#section` references inside markdown, or HTTP URLs. Use [`lychee`](https://github.com/lycheeverse/lychee) for those: there is no token-cheap reason to merge two lints into one binary.

This repository checks its own links with lychee, as a maintenance gate rather than a workflow required of `grund` users, and adds no network I/O to verification ([§FS-non-goals.11](FS-non-goals.md#11-network-access-during-a-check)). On each change, the gate checks every internal link in the tree without the network, relative paths and their fragments included, and sends to the network only the external URLs on lines the change adds or modifies; a scheduled sweep of the default branch checks every external URL in the tree and reports what it finds dead rather than failing anyone's change. A canonical link to this repository's default branch counts as internal: its target and fragment are validated locally, and a missing target or a bad fragment fails the run before any network request is made.

Wherever an external URL is checked, it passes only on a successful response. Its first attempt, a retry within a bounded timeout and retry policy, or a second check of a transient-shaped failure after the gate's own pass can obtain one; when a page on `github.com` answers a server error, a successful answer about the same resource from GitHub's API is that successful response.

Maintenance diagnostics must distinguish attempted API checks from URL shapes
that have no API endpoint. An unsupported shape remains a failure without a
successful response; its diagnostic must explain why no API check was attempted.

## 2. Spelling, grammar, prose quality

`grund` does not lint English. It reads spec text as opaque content between IDs, except for the explicit value binding [§FS-values.3](FS-values.md#3-explicit-value-bindings) defines, and never infers a value from surrounding prose. Use a general-purpose linter, such as `vale` or `ltex-ls`, alongside `grund`. Decided in [§DA-explicit-value-bindings](../decisions/architectural/DA-explicit-value-bindings.md#da-explicit-value-bindings-compare-only-authored-delimited-value-bindings) and [§DA-root-aimed-value-bindings](../decisions/architectural/DA-root-aimed-value-bindings.md#da-root-aimed-value-bindings-a-value-binding-may-aim-at-the-whole-root-its-components-joined-by-one-ascii-space).

## 3. Code AST parsing

`grund` does not parse code: it does not understand classes, methods, types, scopes or call graphs, and a stub's link is a file path, not a symbol reference ([§FS-declarations.stubs.shape](FS-declarations.md#stubsshape-what-a-stub-looks-like)). [§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible) rules out per-language parsers, and IDs are syntactic by design.

## 4. Cross-workspace ID renaming

`grund` does not provide a "rename ID" refactoring, and neither the optional LSP server ([§FS-lsp](FS-lsp.md#fs-lsp-grund-ships-an-optional-lsp-server)) nor any first-party editor wrapper ([§FS-non-goals.12.2](FS-non-goals.md#122-first-party-per-editor-plugins)) adds one. A rename is the author's edit of the declaration and of the citations that name it, the steps [§FS-init.2.3.4](FS-init.md#234-managed-block-content-sections) teaches.

## 5. Documentation generation

`grund` does not generate HTML, PDF, or any rendered documentation from specs. A static-site generator does that downstream, and `grund <ID>` is a building block it can use.

## 6. Decision database, audit log, history tracking

`grund` does not store decisions in a database, render decision graphs over time, track who changed what when, or visualize the ID graph: git is the audit log. `grund refs <ID>` ([§FS-refs](FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id)) answers who cites an ID now, from one scan of the current tree; it is a query, not a stored graph or a history view.

## 7. Inter-agent messaging or workflow

`grund` is a checker and a retriever. It does not coordinate agents, queue work, route messages, or implement any kind of state machine; tools like `rhei` do that.

## 8. Generalization beyond the ID scheme

`grund` does not validate citation syntaxes outside its configured kinds and the marked citation scheme ([§FS-citations](FS-citations.md#fs-citations-what-a-citation-is-and-where-its-text-is-never-rewritten)). A project may model an RFC, ticket, or incident as a committed Markdown declaration whose ID its kind's grammar parses, materialized explicitly with [§FS-fetch](FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot) and read by the ordinary scanner, but there is no URL-only external resolver, markerless ticket recognition, remote lookup during resolution, or second external-citation catalog. [§DISC-external-facts](../discussions/proposals/2026-09-07-external-facts.md#disc-external-facts-external-facts-are-committed-declarations-materialized-explicitly) records why that boundary preserves [§FS-non-goals.11](FS-non-goals.md#11-network-access-during-a-check).

## 9. Severity, exit code, or report-ordering customization

The set of severities, the exit-code mapping (`0`/`1`/`2`) and the report ordering are **not** configurable ([§GOAL-configurable.2](../goals.md#2-what-is-not-configurable)): a rule's strength only chooses which fixed channel its finding reaches ([§FS-config-v2.rules.strengths](FS-config-v2.md#rulesstrengths-one-strength-vocabulary-two-severities)), and the order, text groups and JSON alike, is [§FS-errors.4.1](FS-errors.md#41-ordering)'s, with no flag, key or legacy alternative.

## 10. Interactive mode

`grund` does not have a TUI, an interactive prompt, or a confirmation step. Every subcommand is non-interactive and CI-friendly, because an interactive flow would block CI and complicate scripting ([§GOAL-friendliness-first.2](../goals.md#2-what-this-rules-out)).

## 11. Network access during a check

`grund check` performs no network I/O and executes no configured integration, and neither do queries, formatting, completion, or LSP requests ([§REQ-runs-offline.1](../requirements/REQ-runs-offline.md#1-read-and-verification-paths-execute-nothing)). There is no telemetry. Only an explicit `grund fetch` ([§FS-fetch](FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot)) materializes an external fact, which is checked only once saved in the repository, as a declaration or a committed projection ([§REQ-runs-offline.2](../requirements/REQ-runs-offline.md#2-materialization-is-explicit)).

## 12. Surfaces outside `grund-core` and the LSP transport

`grund` ships exactly two kinds of surface over the engine, the bindings and the LSP server [§FS-distribution](FS-distribution.md#fs-distribution-grund-distribution-targets) targets, and nothing else: anything that would add a third, such as in-engine scripting, a per-editor wrapper or a marketplace plugin, is out of scope.

### 12.1 Plugins or scripting hooks inside the engine

`grund-core` has no plugin system, and no Lua hook or JavaScript user script runs in it: it is a library, and custom behavior calls its first-party bindings ([§FS-distribution](FS-distribution.md#fs-distribution-grund-distribution-targets)) from your own code. A plugin surface multiplies attack surface and breaks reproducibility, letting two installs disagree ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)).

### 12.2 First-party per-editor plugins

`grund` does not publish editor plugins to a marketplace, and does not ship or maintain an editor plugin that duplicates the LSP server, for VSCode, IntelliJ, Vim, Emacs or any other editor. The optional LSP server ([§FS-lsp](FS-lsp.md#fs-lsp-grund-ships-an-optional-lsp-server)) is the only first-party surface for what the protocol serves, and configuring an editor to use it is the user's one-time work ([§FS-lsp.2.3](FS-lsp.md#23-editor-configuration-one-time-per-editor)). The terminal-link extension the binary carries ([§FS-integrations.3.2](FS-integrations.md#32-editor-clients-vscode-codium)) stays in scope: it is never published to a marketplace, and it links `§<ID>` in the integrated terminal, which the LSP does not reach. Per-editor plugins would multiply maintenance across release cadences `grund` does not control, for capabilities the protocol already gives every modern editor ([§FS-lsp.1](FS-lsp.md#1-capabilities)). Decided in [§DA-lsp-optional](../decisions/architectural/DA-lsp-optional.md#da-lsp-optional-lsp-server-ships-as-a-separate-optional-binary).

## 13. Anything that would let two `grund` installs disagree

Any feature that could make two correctly-configured `grund` installs of one version (for a development build, one commit, [§FS-distribution.4.1.1](FS-distribution.md#411-a-development-build-names-its-commit)) disagree, on the same input ([§REQ-deterministic-output.2](../requirements/REQ-deterministic-output.md#2-input-is-the-tree-the-config-and-the-invocation)), about whether a repo is well-formed or about any output byte is permanently out of scope: this is the bright line.

## 14. Generated summaries; token-saving inside `check`

The token-cheap reads, `grund <ID>` ([§FS-show.2.1](FS-show.md#21-whole-declaration-default)), `--brief` ([§FS-show.2.1.1](FS-show.md#211-brief---brief)), `--toc` ([§FS-show.2.1.2](FS-show.md#212-section-map---toc)), `grund refs <ID> --summary` ([§FS-refs.3.3](FS-refs.md#33---summary)) and `grund list --summary` ([§FS-list.3.3](FS-list.md#33---summary)), are structural slices of the heading tree and the citation graph, never a generated summary or paraphrase: a slice prints the author's bytes with only the rewrites [§FS-show.requirements.authored-bytes](FS-show.md#requirementsauthored-bytes-a-slice-is-the-authors-bytes-selected-and-never-rewritten) names, so it stays byte-deterministic on its input ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)). Decided in [§DF-show-token-cheap-reads](../decisions/functional/DF-show-token-cheap-reads.md#df-show-token-cheap-reads-grund-show-keeps-the-full-body-default-token-cheap-slices-are-opt-in) and [§DF-show-default-token-cheap](../decisions/functional/DF-show-default-token-cheap.md#df-show-default-token-cheap-grund-show-defaults-to-the-cheap-read-the-full-body-is-opt-in).

The token economy applies only to the read and query surface: `grund check` never abridges its findings to save tokens, and prints every retained finding whole, in its located form ([§FS-check.2.1](FS-check.md#21-report-format), [§DF-show-token-cheap-reads.2.3](../decisions/functional/DF-show-token-cheap-reads.md#23-check-is-not-abridged)).
