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

Node's adapter depends directly on the core, never the CLI, LSP or Python
frontend. Native transport and JS validation introduce no scanner, resolver,
checker or write-policy implementation ([§FS-distribution.3.2.1](../functional-spec/FS-distribution.md#321-operations-options-and-refusals)).

The frontends' side of [§AR-system.3](README.md#3-frontends) and the api's contract of [§AR-system.2.9](README.md#29-api). Every frontend takes the data the api returns and gives back a rendering or a transport of it; none holds a regex, a walk or a rule, and none depends on another. The engine's side of the contract is section 2; each frontend has a section of its own below.

The approved Python addition is an independent frontend over this boundary
([§FS-distribution.3.3](../functional-spec/FS-distribution.md#33-python-grund-pypi-package)); its acceptance tests initially fail until the local extension
exists. Registry distribution remains pending. No Python operation
delegates to a CLI process or imports another frontend.

## terms: Terms

Leans on [§FS-terms.terms.1](../functional-spec/FS-terms.md#terms1-declarations-and-coordinates) (section), [§FS-terms.terms.4](../functional-spec/FS-terms.md#terms4-scanning-and-project-structure) (scan), and
[§FS-terms.terms.5](../functional-spec/FS-terms.md#terms5-findings) (severity).

## 1. Target workspace layout

Resolved dependency evidence includes `grund-node` and excludes napi/Node
dependencies from core, CLI and LSP. CLI/LSP Cargo builds remain independent of
Node/npm tooling.

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

Python joins the resolved graph proof in `tests/bindings/test_isolation.py`:
grund-py depends directly on core and transitively on no frontend. The ordinary
Cargo CLI graph excludes PyO3/Python build dependencies ([§FS-distribution.3.3.7](../functional-spec/FS-distribution.md#337-local-source-and-typing-handoff)).
This extends, rather than replaces, the existing CLI/LSP isolation proof.

## 2. grund-core: the only place logic lives

Additive `ApiOutcome<T> { run_cautions, result: Result<T, ApiFailure> }`
carriers preserve cautions before failures and attach classification at the
source, with unchanged Display output ([§FS-distribution.3.1](../functional-spec/FS-distribution.md#31-rust-grund-core-crate)). Typed batch data,
schema-keyed config values, path-base carriers and integration-install
orchestration reuse current contexts and policy. Existing supported signatures
remain available; no regex or substring classification belongs in a binding.
Expected errors carry query/config/io/operation kinds, actual source coordinates,
sites, authority, causes, details and typed partial outputs. Unknown recoverable
errors retain an operation fallback. Compatible #470 carriers are reused.

Every check, every show, every regex, every walker invocation lives in `grund-core`. The crate exposes:

Bindings reuse current warning-preserving scoped APIs, selection and batch queries,
size/coverage/config/completion APIs, snapshot materialization and managed writers.
Additive failure carriers classify errors at source without parsing Display text;
core owns non-Unicode workspace preflight and integration-install orchestration.
Existing Rust entry points and CLI rendering/defaults stay compatible
([§FS-distribution.3.1](../functional-spec/FS-distribution.md#31-rust-grund-core-crate)). Historical signatures below are not a frozen Python schema.

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

For [§FS-check.6](../functional-spec/FS-check.md#6-watch-mode---watch), a synchronous CLI coordinator owns native notifications, bounded debounce, serialized scans/publication, screen ownership and shutdown. It reuses `check_with_run_warnings` and the one-shot selection/sorting/rendering/status path. Core config/workspace/scanner helpers expose a data-only effective-input inventory, including failed discovery; they do not own notifications or a public binding watch API. Coverage is installed before reads and refreshed before obsolete subscriptions retire ([§FS-check.6.1.1](../functional-spec/FS-check.md#611-subscribe-before-reading)). A directory is registered at most once per native backend, because ReadDirectoryChangesW keys registrations by path and a second one orphans the first request: a shallow anchor that becomes recursive coverage is moved to a replacement backend, which is complete before the old one is released ([§FS-check.6.1.3](../functional-spec/FS-check.md#613-effective-input-inventory), [§FS-check.6.3.3](../functional-spec/FS-check.md#633-interrupt-and-completion)).

Private test builds provide an observer outside stdout/stderr for subscription readiness and completed publication (including both stream flushes, sequence, captured bytes and status). They also provide controllable discovery/scan/publication barriers, fake clock/events and injected setup/runtime/loss/saturation/recovery failures. Tests use these to prove startup ordering, unpublished-result discard, empty JSON completion and cleanup ([§FS-check.6.1.2](../functional-spec/FS-check.md#612-bounded-debounce-and-serialized-work), [§FS-check.6.1.4](../functional-spec/FS-check.md#614-uncertain-notifications-and-recovery), [§FS-check.6.2.1](../functional-spec/FS-check.md#621-exact-stream-contract), [§FS-check.6.3.3](../functional-spec/FS-check.md#633-interrupt-and-completion)). Production builds contain no observer, barrier, injection control or completion record; enabling test machinery cannot change report bytes.

## 4. grund-lsp: the LSP server binary

Speaks LSP over stdio ([§AR-lsp.4](AR-lsp.md#4-transport)). Imports `grund-core` for scan/check/show/fmt-backed state, `lsp-server` for the stdio JSON-RPC loop and `lsp-types` for protocol data shapes. Published on Cargo as `grund-lsp`, with npm and PyPI packages planned ([§FS-distribution.1](../functional-spec/FS-distribution.md#1-targets)). Neither it nor `grund-cli` pulls the other in. Its architecture is [§AR-lsp](AR-lsp.md#ar-lsp-how-the-lsp-server-is-built).

## 5. grund-node: the napi-rs binding

Implements [§FS-distribution.3.2.1](../functional-spec/FS-distribution.md#321-operations-options-and-refusals) through [§FS-distribution.3.2.4](../functional-spec/FS-distribution.md#324-acceptance-evidence). The crate
owns napi-rs conversion and AsyncTask dispatch; owned inputs and records cross
the worker boundary. Compute does all engine/filesystem work on libuv workers;
resolve marshals data on the live JS thread. No Tokio runtime or CLI subprocess.
Roots/options are captured and copied before the first await, without chdir.

Unwind-capable builds catch panics at compute and native conversion/resolve/
reject boundaries. One hook dispatcher suppresses only guarded binding requests
and chains the previous hook elsewhere. A bounded addon-owned Rayon pool marks
its workers with the same guard, preventing scanner panics from printing before
their unwind is caught. No global core pool or Node dependency enters the engine.
Environment and in-flight-job ownership keep Rust resources alive through
worker_threads teardown, discard callbacks to destroyed environments and release
leases after running work finishes. No abort/OOM recovery or rollback promise.

An atomic lease per loaded addon rejects overlapping actual writers across roots
with busy/writer-busy. fmt(write), init unless check/dryRun, fetch and
integrations(write) acquire it before side effects; read/preview calls do not.
RAII releases the lease on success/error/unwind. No pool-worker mutex queue,
cross-process lock or external-write snapshot is introduced.

The private `grund.node` exports invoke and matching schema/engine/package/
target/N-API metadata. A shared lazy CJS loader and explicit ESM facade preserve
GrundError identity on Node 22/24, N-API 8. A matching local payload wins over an
exact-version platform payload; corrupt/incompatible local payloads fail without
fallback. Imports without native payload work; calls reject structured load
failures without downloading or compiling ([§FS-distribution.3.2.3.2](../functional-spec/FS-distribution.md#3232-moduleruntimeload-interface)).

#469 owns JS/TS assets, `package-api.json`, native/source builders and a
disposable API-only staging helper. #471 owns the single assembly manifest,
grund executable launcher, optional platform packages and installed-platform
evidence. The fragment is consumed by that recipe, never a second committed
umbrella manifest. The explicit bundled locked source workflow in
[§FS-distribution.3.2.3.3](../functional-spec/FS-distribution.md#3233-source-buildlocal-package-ownership) uses an external Cargo target directory. #470 owns
the common corpus/oracle; Node supplies its adapter and fixtures. Complete-data
and frozen CLI-byte parity are distinct proofs ([§FS-distribution.3.0.3](../functional-spec/FS-distribution.md#303-complete-data-and-canonical-parity)).

#471's recipe is `scripts/distribution/candidate.py`. It reads `package-api.json`
unchanged and adds only the name, version, `bin`, the exact-version optional platform
packages and `grund.platformPackages` ([§FS-distribution-candidate.2.3](../functional-spec/FS-distribution-candidate.md#23-one-assembly-recipe-consumes-the-bindings-fragment)). Each platform
package's `index.cjs` gives the loader `{addonPath, metadata}` and the launcher
`executablePath`. The launcher is a plain Node script that spawns the payload with
inherited stdio and forwards its status and signals ([§FS-distribution-candidate.3.2](../functional-spec/FS-distribution-candidate.md#32-the-npm-launcher-is-invisible)); it
loads no addon and never compiles or downloads ([§FS-distribution-candidate.3.3](../functional-spec/FS-distribution-candidate.md#33-a-missing-payload-is-an-error-never-a-substitute)).

Later “Adapt Node to #466” changes internals while retaining this approved host
contract and supported Rust consumers. #459/#463 unshipped behavior is deferred;
publication, registry changes and version bumps are outside local readiness.

## 6. grund-py: the PyO3 binding

The planned grund-py crate owns only PyO3 conversion and exception policy over supported core
data APIs ([§FS-distribution.3.3](../functional-spec/FS-distribution.md#33-python-grund-pypi-package)). Frozen dataclasses, tuples and read-only config
mappings in `python/grund` expose the public schema; private `grund._native` holds
the abi3-py310 extension. Root `pyproject.toml` selects maturin, with source inclusion
for independently installing an unpacked sdist and accurate types/py.typed.

Rust work runs with the GIL released; calls check Python interrupts on return and
carry per-call scope rather than changing cwd ([§FS-distribution.3.3.4](../functional-spec/FS-distribution.md#334-silent-synchronous-calls)).
Core supplies warning/failure/path/install adapters; Python does not walk, resolve,
check, parse or reproduce managed-write behavior. No dependency on CLI/LSP/Node is
allowed. `tests/bindings/` compares complete data and canonical bytes, separately
from the frozen CLI wire projection ([§FS-distribution.3.0.3](../functional-spec/FS-distribution.md#303-complete-data-and-canonical-parity)).

Local build readiness does not claim published wheels. #471 owns cibuildwheel
release assembly and CLI payload placement; the binding adds no console entrypoint.
Release assembly builds one abi3 extension per registry row with its own profile and
places the separately PGO-built `grund` executable as wheel data in `scripts/`, so
installers put the payload itself on `PATH`. cibuildwheel then installs that one wheel
into CPython 3.10 through 3.14 and runs the acceptance modules against each
([§FS-distribution-candidate.1.2](../functional-spec/FS-distribution-candidate.md#12-every-registry-row-proves-every-runtime-it-promises), [§FS-distribution-candidate.3.1](../functional-spec/FS-distribution-candidate.md#31-payloads-sit-where-each-registry-runs-them)). `grund-lsp` wheels
carry only the server and are tagged `py3-none`.
Carry “Adapt Python marshalling to #466/#453/#454” without changing the approved
Python schema. #469 supplies the later Node adapter; neither coordination is a
prerequisite for this local frontend.

## 7. Why this shape

- **One source of truth for behavior.** Bug fixes and new rules land in `grund-core` and reach all three ecosystems on the next release.
- **No re-implementation.** Neither Node nor Python developers need to maintain a parallel parser or a parallel rule set.
- **Fast everywhere.** The compiled engine is the same in all three; the bindings add only a thin marshalling layer.
- **Independent release cadence per crate when needed.** A Node-only fix in `grund-node` does not require a `grund-core` version bump. Such a candidate is scoped `binding-only`: its manifest records the binding's own package version beside the unchanged engine and CLI version, and it never claims ecosystem completeness ([§FS-distribution-candidate.6.3](../functional-spec/FS-distribution-candidate.md#63-a-binding-only-candidate-never-claims-completeness)).
