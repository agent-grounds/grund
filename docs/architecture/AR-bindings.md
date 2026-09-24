# AR-bindings: target shape for exposing the Rust engine on three platforms

Implements the planned distribution shape in [§FS-distribution](../functional-spec/FS-distribution.md#fs-distribution-grund-distribution-targets). Target state: a Cargo workspace with one core library and four frontends — three for batch use (CLI, Node, Python) and one for editor use (LSP). The release-blocking boundary is in place for Cargo: `grund-core` is the shared engine crate, `crates/grund-cli` the published package `grund`, and `crates/grund-lsp` the optional package `grund-lsp`. The planned `grund-node` and `grund-py` build on that boundary.

## placement: Where the frontends sit

```text
      ┌─► [ grund-cli ]  ─► text, JSON, exit codes
api ──┼─► [ grund-lsp ]  ─► LSP over stdio
      ├─► [ grund-node ] ─► Promise-returning functions   (planned)
      └─► [ grund-py ]   ─► Python functions              (planned)
      no frontend depends on another; every regex, walk and rule stays in the engine
```

The frontends' side of [§AR-system.3](README.md#3-frontends) and the api's contract of [§AR-system.2.9](README.md#29-api). Every frontend takes the data the api returns and gives back a rendering or a transport of it; none holds a regex, a walk or a rule, and none depends on another. The engine's side of the contract is section 2; each frontend has a section of its own below.

## 1. Target workspace layout

The shipped split ([§AR-system.1](README.md#1-the-system)) keeps one checked report behind every frontend and gives `grund-lsp` and the language bindings a library package to depend on. `grund-core` exposes the data-returning APIs of section 2 and the LSP snapshot; the binary, help, version, SIGPIPE setup, dispatch, flag parsing, text/JSON rendering and exit-code mapping live in `grund-cli` (section 3). Its renderer gives text and JSON deliberately distinct deterministic orders — severity groups for text, global location order for compatible JSON — without changing the shared report or LSP messages ([§FS-errors.4](../functional-spec/FS-errors.md#4-determinism)).

Final frontend layout:

```
grund/
├── crates/
│   ├── grund-core/   # the engine: scanner + checker + show + fmt + config. Pure Rust. No I/O policy.
│   ├── grund-cli/    # the CLI binary. Command parsing, exit codes, terminal formatting. Published to cargo as `grund`.
│   ├── grund-lsp/    # the LSP server binary. Speaks LSP over stdio. Published to Cargo as `grund-lsp`; npm/PyPI planned.
│   ├── grund-node/   # napi-rs binding. Published to npm as `grund-cli` (with the prebuilt CLI binary).
│   └── grund-py/     # PyO3 binding. Published to PyPI as `grund`.
├── docs/
└── tests/
```

All four frontend crates take their engine logic from `grund-core` alone and depend on none of each other. `tests/integration/test_frontend_isolation.py` holds this on the resolved dependency graph `cargo metadata` reports rather than on the manifests' intent: the CLI's tree carries no LSP transport, the server's no CLI, and the engine's no frontend. That is what lets [§DA-lsp-optional](../decisions/architectural/DA-lsp-optional.md#da-lsp-optional-lsp-server-ships-as-a-separate-optional-binary) hold: no JSON-RPC machinery or LSP type reaches `grund-core`, so none is in `grund-cli`'s tree.

## 2. grund-core: the only place logic lives

Every check, every show, every regex, every walker invocation lives in `grund-core`. The crate exposes:

- `grund_core::scan(root: &Path) -> Result<Findings>`
- `grund_core::check(root: &Path) -> Result<Report>`
- `grund_core::check_with_opts(opts: CheckOpts) -> Result<CheckOutput>`
- `grund_core::show(id: &str, opts: ShowOpts) -> Result<ShowOutput>`
- `grund_core::refs(opts: RefsOpts) -> Result<RefsOutput>` ([§FS-refs](../functional-spec/FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id))
- `grund_core::list(opts: ListOpts) -> Result<ListOutput>`
- `grund_core::cover(opts: CoverOpts) -> Result<CoverOutput>`
- `grund_core::format_references(opts: FmtOpts) -> Result<FmtOutput>`
- `grund_core::propose_id(kind, title, opts) -> Result<IdProposalOutcome>`
- `grund_core::init(opts: InitOpts) -> Result<InitOutput>`
- `grund_core::complete_ids(opts: CompleteIdsOpts) -> Result<Vec<String>>`
- `grund_core::effective_config(path)` / `grund_core::validate_config(path)`
- The `Findings`, `Declaration`, `Citation`, `Report` data types.

Every name on the crate's public surface returns data and writes to no stream; callers decide what to do with it ([§FS-distribution.3.1](../functional-spec/FS-distribution.md#31-rust-grund-core-crate)). The former `grund_core::main_entry()` exception and its `compat/` renderer are absent ([§AR-system.2.9.1](README.md#291-no-process-frontend-lives-in-the-engine)). The published `grund` CLI owns command parsing, terminal rendering and exit-code policy for every command, and imports no `grund_core::command_*` symbol and no engine renderer under any spelling — `run_integrations` and `print_config_warnings` are the CLI's own ([§FS-integrations.1](../functional-spec/FS-integrations.md#1-user-facing-command), [§FS-config.4.2](../functional-spec/FS-config.md#42-grund-config-show-path), [§DA-engine-renders-nothing](../decisions/architectural/DA-engine-renders-nothing.md#da-engine-renders-nothing-the-engine-renders-nothing-so-the-deprecated-compat-frontend-retires)). `tests/integration/test_engine_boundary.py` holds the boundary: no engine source writes to a stream or exits a process, no process-entry export or compatibility frontend test machinery remains, and neither live frontend references an engine renderer.

## 3. crates/grund-cli: the CLI binary

`crates/grund-cli`, the Cargo package published as `grund`: what `cargo install grund` produces and what the npm/PyPI packages wrap. It owns the installed binary, prints help and version, restores SIGPIPE, and routes top-level commands to CLI-local wrappers over the data APIs. Every command renders through one of those wrappers, `integrations` included: its argument parsing, detection and artifact printing, and its `--write` reports are `cli_integrations.rs` and `cli_integrations_write.rs`, over the client set, detection, agent surfaces and managed writes the engine returns as data ([§FS-integrations.1](../functional-spec/FS-integrations.md#1-user-facing-command)). Synchronous; no async runtime, no LSP types, no JSON-RPC.

## 4. grund-lsp: the LSP server binary

Speaks LSP over stdio ([§AR-lsp.4](AR-lsp.md#4-transport)). Imports `grund-core` for scan/check/show/fmt-backed state, `lsp-server` for the stdio JSON-RPC loop and `lsp-types` for protocol data shapes. Published on Cargo as `grund-lsp`, with npm and PyPI packages planned ([§FS-distribution.1](../functional-spec/FS-distribution.md#1-targets)). Neither it nor `grund-cli` pulls the other in. Its architecture is [§AR-lsp](AR-lsp.md#ar-lsp-how-the-lsp-server-is-built).

## 5. grund-node: the napi-rs binding

Re-exports the same operations as Promise-returning Node functions. The npm `grund-cli` package ships:

- The `grund` binary (so `npx grund-cli` works).
- A small JS module re-exporting `check`, `show`, etc. against the napi binding (so `import { check } from 'grund-cli'` works).

Prebuilt platform binaries are uploaded as separate npm packages (`@grund-cli/linux-x64`, etc.) per the `napi-rs` convention; the main package picks the right one at install time.

## 6. grund-py: the PyO3 binding

Same operations, exposed as Python functions. Built and packaged via `maturin`. Wheels are produced by `cibuildwheel` in CI for each release. Source distributions are also uploaded so unsupported platforms can build from source.

## 7. Why this shape

- **One source of truth for behavior.** Bug fixes and new rules land in `grund-core` and reach all three ecosystems on the next release.
- **No re-implementation.** Neither Node nor Python developers need to maintain a parallel parser or a parallel rule set.
- **Fast everywhere.** The compiled engine is the same in all three; the bindings add only a thin marshalling layer.
- **Independent release cadence per crate when needed.** A Node-only fix in `grund-node` does not require a `grund-core` version bump.
