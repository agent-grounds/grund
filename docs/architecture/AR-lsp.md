# AR-lsp: how the LSP server is built

Implements [§FS-lsp](../functional-spec/FS-lsp.md#fs-lsp-grund-ships-an-optional-lsp-server). The LSP server is a separate crate (`grund-lsp`) in the workspace defined by [§AR-bindings.1](AR-bindings.md#1-target-workspace-layout), depending only on `grund-core`. It has no shared runtime with `grund-cli`, no shared state with the bindings, and no own engine logic — everything it does delegates to `grund-core`.

## placement: Where the LSP server sits

```text
editor ◄── LSP over stdio ──► [ grund-lsp ] ──► api ──┬─► queries: snapshot, hover, on-type edits
                                                      └─► checker: diagnostics
                              grund-cli: no shared code, no shared dependency
```

A frontend ([§AR-system.3](README.md#3-frontends)) depending on `grund-core` and on nothing of `grund-cli`. It takes LSP requests over stdio and gives back what the api's editor queries return ([§AR-system.2.7](README.md#27-queries), [§AR-system.2.9](README.md#29-api)) — the snapshot, the hover body, the on-type edits — and the check report as diagnostics. It has no scanner, checker, `show` extraction or `fmt` planning of its own, no filesystem walk outside `grund_core::scan` and no second config loader: batch rendering asks `grund-core` for upward discovery and effective scan extensions, and a focused integration module owns only deterministic rendering and conflict-safe materialization. The JSON-RPC loop and protocol types live only here — `grund-core` references no `lsp-server` or `lsp-types`, and `grund-cli` pulls none in ([§AR-bindings.3](AR-bindings.md#3-grund-cli-the-cli-binary)) — which is what keeps the server optional ([§DA-lsp-optional](../decisions/architectural/DA-lsp-optional.md#da-lsp-optional-lsp-server-ships-as-a-separate-optional-binary), [§AR-system.4](README.md#4-dependency-direction)).

## 1. Crate boundary

`grund-lsp` is a binary crate with one product boundary: the optional server and the editor-client configuration that launches it. Its no-argument path speaks LSP over stdio and translates each request into a `grund-core` call; a thin pre-transport batch path lists, renders, or materializes the embedded integration artifacts of [§FS-lsp.2.4](../functional-spec/FS-lsp.md#24-installed-editor-integrations). What it does not contain is its placement chapter; its dependency cost stays in `grund-lsp`, and a user installing only the `grund` CLI pays none of it ([§DA-lsp-optional](../decisions/architectural/DA-lsp-optional.md#da-lsp-optional-lsp-server-ships-as-a-separate-optional-binary)).

## 2. State

The server holds one in-memory `LspSnapshot` per discovered Grund project, built by `grund-core` from the scan/check data [§AR-scanner.3](AR-scanner.md#3-output) produces, plus the resolved declaration, section-heading, stub, citation, and link ranges editor requests need and the set of files the scan read. The snapshots are the cache for everything else: hover, definition, references, document links, and diagnostics all answer from the snapshot of the document's owner (§2.2), and independent projects are never merged ([§FS-lsp.2.2](../functional-spec/FS-lsp.md#22-lifecycle)). An edit rebuilds only the projects that can see the edited file (§2.1).

### 2.1 What rebuilds a snapshot

- On `initialize`, the server turns the editor's folder anchors into deduplicated project roots by the discovery, fallback and skip rules of [§FS-lsp.2.2](../functional-spec/FS-lsp.md#22-lifecycle).
- During startup and again on `initialized`, it runs a full scan of every distinct project root, each on its own, so one that fails keeps its last good snapshot while the others refresh.
- On `textDocument/didChange`, it updates the file's in-memory copy (LSP delivers the new text) and re-scans only the projects that can see that file: one whose scan never reads the document cannot have changed verdicts, so the per-edit cost follows the edited project, not the number of open folders. A document *no* project claims rebuilds everything: that is what a file newly created under an external include root looks like, and treating it as nobody's would leave the editor silent on a file the CLI checks.
- On `textDocument/didSave`, it reconciles the in-memory copy against disk, for files another tool wrote.
- On `textDocument/didClose`, it drops the in-memory overlay and re-scans against disk.
- On `workspace/didChangeWatchedFiles`, it re-scans every project to pick up the creates and deletes the editor reported — a path no project has read yet belongs to none of them, so this cannot be narrowed to the changed document.
- On `workspace/didChangeWorkspaceFolders`, it rediscovers and deduplicates project roots from the remaining anchors, rebuilds their snapshots, and republishes diagnostics ([§FS-lsp.2.2](../functional-spec/FS-lsp.md#22-lifecycle)).

### 2.2 Which snapshot answers

Requests and diagnostics alike take their snapshot from one resolution of the document's owner under the rule of [§FS-lsp.2.2](../functional-spec/FS-lsp.md#22-lifecycle), so an editor cannot navigate against one project's view while reading another's errors, and a finding reaching two snapshots is published once, by the owner. That rule falls back from a containing root to a project whose scan reached the document, which is why `LspSnapshot` carries the set of files its scan read: a plain parent-relative include path may reach beyond the root, while a document reachable only through an outward directory symlink is absent from that set.

## 3. Scan strategy

### 3.1 Full re-scan on every change (v1)

Initial implementation: every `didChange` triggers `grund_core::scan(workspace_root)` and a fresh `grund_core::check`. This is simple and correct. The [§GOAL-fast-feedback.1](../goals.md#1-performance-targets) targets are the budget it rests on: a scan within them makes a full re-scan per keystroke invisible on small and medium projects, and acceptable per-save on large ones.

### 3.2 Incremental scan (v2, when budget breaks)

When the full-scan budget breaks (typically: large monorepos, slow disks, or per-keystroke debounce too tight), switch to incremental: rescan only the changed file and re-validate citations whose targets touch the changed file's declarations. This is the gradient [§GOAL-fast-feedback.2](../goals.md#2-how-we-get-there) endorses for the CLI's parallel scan — incremental is added when the simple version stops winning, not before.

The incremental path keeps the single source of truth in `grund_core::scan`; `grund-lsp` adds a thin "what changed" diff over scan inputs and reuses the rest.

## 4. Transport

LSP transport is **stdio only**: no TCP, no Unix socket, no named pipe. Stdio is what every LSP-aware editor expects by default, has no port-conflict surface, and needs no local listener another process could reach. Only an invocation with no arguments enters this transport: the editor invokes the server as a child process and it reads and writes JSON-RPC framed messages on stdin/stdout. Diagnostic logging goes to stderr in the LSP-canonical `[LEVEL] message` form, which editors that surface server logs render as-is.

Any argument is dispatched before the transport is constructed. Valid batch commands write only their documented CLI output, and help, malformed input, and batch failures return without reading protocol stdin or writing a JSON-RPC frame. This keeps protocol stdout pristine while letting the installed binary expose its own editor configuration ([§FS-lsp.2.4](../functional-spec/FS-lsp.md#24-installed-editor-integrations)).

## 5. Determinism and parity tests

The LSP must produce the same diagnostics for the same workspace state as `grund check` does — byte-for-byte on the message text, position-for-position on the line numbers ([§FS-lsp.4](../functional-spec/FS-lsp.md#4-determinism-and-parity-with-the-cli)). It does so as the same engine with a different transport, not a parallel implementation that could drift: `grund-core` does all engine work, returning the snapshot records of one scan/check pass (§5.1) and the hover answers read from them (§5.2), and `grund-lsp` only translates those into protocol ranges. Tests hold it, down to a child-process sweep against the CLI (§5.3).

### 5.1 The snapshot records

- `grund_core::lsp_snapshot` returns the report, declaration ranges, section-heading ranges, stub ranges, citation ranges, and resolved targets from one scan/check pass.
- Named sections and embedded value roots add no server-side parser or resolver: the snapshot carries their complete section paths, heading-title ranges, citation ranges, resolved targets, and checker diagnostics in the same records used for ordinary numeric sections. For a marked root, `section_range_parts` excludes the invisible marker from the title token while raw `show --toc` hover content keeps it; a binding definition still targets the existing component heading ([§FS-values.7](../functional-spec/FS-values.md#7-workspaces-and-editor-consumers)).
- Diagnostics, hover, definition, references, links, and highlights translate those records only.
- `textDocument/onTypeFormatting` calls the same configured trigger/marker and ID-grammar checks as `grund fmt`.

### 5.2 Hover

- `textDocument/hover` previews a citation's body by calling the same `show` engine as `grund <ID> --toc`, with open-document overlays applied; a declaration-side title returns the whole-title range and its usage counts ([§FS-lsp.1.2](../functional-spec/FS-lsp.md#12-hover-preview)).
- Those counts are read from the snapshot, not from a fresh `refs` query: `grund_core::refs` is the CLI's entry point and loads its own workspace context — one full scan per call — so calling it per hover would re-walk the tree on a keystroke-adjacent request and break [§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible). The snapshot comes from the same scan `refs` and `check` run (§2), so what is shared is the rule rather than the walk: `grund_core::citation_under_title` is the single definition of which citations belong to a declaration-side title, and `LspSnapshot::title_usage` and `LspSnapshot::title_citations` are the count and the list built from it, so the hover number and the `textDocument/references` result cannot drift apart ([§FS-lsp.1.3.1](../functional-spec/FS-lsp.md#131-references-from-declarations)). A test runs `grund_core::refs` over the same tree and compares.
- The hover body bytes are formed in `grund_core::lsp_title_hover_body`, beside the counts rather than in the transport, so the singular/plural and zero wording of [§FS-lsp.1.2](../functional-spec/FS-lsp.md#12-hover-preview) is unit-testable without a server process and cannot be re-worded by a second frontend.

### 5.3 Tests

- CLI/core/LSP parity fixtures cover embedded mismatch and title ranges alongside missing, duplicate, orphan, depth, and resolving named sections, rather than parallel behavior in the transport.
- Focused `grund-lsp` tests cover UTF-16 range conversion, hover linkification, configured trigger punctuation, member-local trigger/marker overrides, section-heading definition and references, embedded-root marker-free title ranges, raw previews, component navigation, CLI-equal value diagnostics, declaration-title usage counts end to end (with the citation hover left as the `--toc` body), whole-title stub document links, the absence of a self-pointing link on ordinary declaration titles (so the click resolves to go-to-definition usages, [§FS-lsp.1.3.2](../functional-spec/FS-lsp.md#132-document-links)), whole-token occurrence highlight ([§FS-lsp.1.3.3](../functional-spec/FS-lsp.md#133-occurrence-highlight)), and citation document-link line fragments.
- The declaration/citation matcher moved to `grund-core` with `citation_under_title`, so its unit cases live beside the counts they feed in `crates/grund-core/src/queries/tests_lsp_hover.rs`; this crate still covers it end to end, over a real server, in the navigation case.
- `tests/integration/lsp_cli_parity.rs` is the child-process sweep of [§FS-lsp.4](../functional-spec/FS-lsp.md#4-determinism-and-parity-with-the-cli): for every e2e case that is a plain `check` of a fixture carrying its own config, it compares the diagnostics the server publishes on `initialized` with the located findings `grund check --format json` prints — path, line, code, severity and message. The compared cases have a floor, so the sweep cannot shrink unnoticed; cases the CLI refuses and fixtures with no config at their root are counted, never silently skipped.

## 6. What this does not contain

- No editor-specific protocol code or wrapper. Per [§FS-lsp](../functional-spec/FS-lsp.md#fs-lsp-grund-ships-an-optional-lsp-server) and [§FS-non-goals](../functional-spec/FS-non-goals.md#fs-non-goals-what-grund-will-deliberately-not-do), no first-party VSCode/IntelliJ/Vim/Emacs plugin ships; the embedded LSP4IJ files are inert configuration data that the user explicitly imports.
- No process supervision. The editor owns the lifecycle ([§FS-lsp.2.2](../functional-spec/FS-lsp.md#22-lifecycle)); `grund-lsp` does not respawn itself, does not background, does not write a PID file.
- No telemetry, no auto-update, no crash reporter ([§FS-non-goals.11](../functional-spec/FS-non-goals.md#11-network-access-during-a-check) — no network I/O).
