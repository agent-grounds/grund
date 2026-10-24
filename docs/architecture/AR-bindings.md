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
│       ├── pyproject.toml   # selects maturin
│       ├── python/grund/    # the public API, types and py.typed
│       └── src/             # PyO3 glue
├── docs/
└── tests/
    └── bindings/     # shared corpus and oracle
        ├── node/     # Node adapter, tests and runner
        └── python/   # Python adapter, tests and runner
```

Each binding is one directory: everything it owns is under its crate, and its
acceptance tests sit beside the other binding's under `tests/bindings/`.

All four frontend crates take their engine logic from `grund-core` alone and depend on none of each other. `tests/integration/test_frontend_isolation.py` holds this on the resolved dependency graph `cargo metadata` reports rather than on the manifests' intent: the CLI's tree carries no LSP transport, the server's no CLI, and the engine's no frontend. That is what lets [§DA-lsp-optional](../decisions/architectural/DA-lsp-optional.md#da-lsp-optional-lsp-server-ships-as-a-separate-optional-binary) hold: no JSON-RPC machinery or LSP type reaches `grund-core`, so none is in `grund-cli`'s tree.

Python joins the resolved graph proof in `tests/bindings/python/test_isolation.py`:
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
One selection, `CheckFindingSelection::select`, narrows a check for the CLI and
both bindings, and appends the `selected-check-off` lines from the switches the
hidden `check_with_run_warnings` path says the run armed
([§FS-check.1.4.1](../functional-spec/FS-check.md#141-a-selected-check-the-run-leaves-off-says-so)).
Existing Rust entry points and CLI rendering/defaults stay compatible
([§FS-distribution.3.1](../functional-spec/FS-distribution.md#31-rust-grund-core-crate)). Historical signatures below are not a frozen Python schema.

- `grund_core::scan(root: &Path) -> Result<Catalog>`
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
- `grund_core::integrations(opts: IntegrationsOpts) -> Result<IntegrationsOutput, IntegrationsError>` ([§FS-distribution.3.1.3](../functional-spec/FS-distribution.md#313-integrations-is-the-one-integrations-entry-point))
- `grund_core::effective_project(path) -> Result<Project>`, and `grund_core::effective_config(path)` / `grund_core::validate_config(path)` beside it, all three `#[doc(hidden)]` seams outside the 1.0 promise ([§AR-bindings.2.1](AR-bindings.md#21-the-crate-root-is-the-10-promise-and-the-frontends-seams))
- The `Catalog`, `Declaration`, `Citation`, `Report` data types, and the configuration records `Project`, `Run` and `Compiled` ([§AR-config.1](AR-config.md#1-the-records)), the three records `#[doc(hidden)]` like the readers above.
- Not `Findings`, `Config`, `KindConfig` or `CitationRules`: deprecated since `0.17.0`, they were removed in `0.19.0`. `Catalog` replaces `Findings`, and the `Config` façade [§AR-config.5](AR-config.md#5-the-config-façade-and-the-list-that-only-shrinks) builds from the records is what `effective_config` still returns, unnamed at the root.

Every name on the crate's public surface returns data and writes to no stream; callers decide what to do with it ([§FS-distribution.3.1](../functional-spec/FS-distribution.md#31-rust-grund-core-crate)). The former `grund_core::main_entry()` exception and its `compat/` renderer are absent ([§AR-system.2.9.1](README.md#291-no-process-frontend-lives-in-the-engine)). The published `grund` CLI owns command parsing, terminal rendering and exit-code policy for every command, and imports no `grund_core::command_*` symbol and no engine renderer under any spelling — `run_integrations` and `print_config_warnings` are the CLI's own ([§FS-integrations.1](../functional-spec/FS-integrations.md#1-user-facing-command), [§FS-config.4.2](../functional-spec/FS-config.md#42-grund-config-show-path), [§DA-engine-renders-nothing](../decisions/architectural/DA-engine-renders-nothing.md#da-engine-renders-nothing-the-engine-renders-nothing-so-the-deprecated-compat-frontend-retires)). `tests/integration/test_engine_boundary.py` holds the boundary: no engine source writes to a stream or exits a process, no process-entry export or compatibility frontend test machinery remains, and neither live frontend references an engine renderer.

### 2.1 The crate root is the 1.0 promise and the frontends' seams

`crates/grund-core/src/lib.rs` exports each component's names in two groups. A plain `pub use` is the promise [§FS-distribution.3.1.2](../functional-spec/FS-distribution.md#312-what-10-freezes) states: the entry points and every type their signatures reach, 115 names, `InitAllocation` and `InitAllocationHome` of [§FS-init.2.2.2.3](../functional-spec/FS-init.md#2223-the-kind-the-guidance-names) among them. A `#[doc(hidden)] pub use` is a seam one frontend reads — the configuration records that still move, the editor's snapshot and hover API, the Python and Node carriers, the CLI's `*_with_run_warnings` twins and the `test-binding-writes` hooks — 150 names that link and work and are no part of the promise. The grouping is in `lib.rs` itself, not at each definition, so `tests/integration/test_public_surface_inventory.py` reads which group a name is in from the one file that exports it. There is no third group. The 30 integrations names [§FS-distribution.3.1.3](../functional-spec/FS-distribution.md#313-integrations-is-the-one-integrations-entry-point) removed in `0.21.0` were one, re-exported from `writers/integrations_retiring.rs`, where each forwarded to the crate-private original the engine calls. That file went with them, and so did the two originals only it called, `known_clients_line` and `block_outcome_verb`; the rest are still `integrations`' own. In their place `lib.rs` keeps one comment line per removed name and method, beside the four aliases removed in `0.19.0`.

`#[non_exhaustive]` is at each definition rather than in `lib.rs`: every promised struct and enum carries it, and no other type and no variant does ([§FS-distribution.3.1.2](../functional-spec/FS-distribution.md#312-what-10-freezes)). The frontends are other crates, so each `match` they make on a promised enum has a wildcard arm, and every variant of today renders as it did. That arm renders what core gave that does not depend on the variant. An event whose note this release does not name prints its bare `<verb> <path>` line, because the write happened. A refusal or a fetch failure of a kind it does not name is an operational failure in core's own words: one `error: <message>` line on stderr and exit `2` ([§REQ-never-crashes.2](../requirements/REQ-never-crashes.md#2-exit-codes-are-the-api), [§FS-errors.2.2.2](../functional-spec/FS-errors.md#222-exit-codes)). Only where nothing is left, as for an `IdProposalOutcome`, does the arm refuse the variant itself, with the same line and exit naming the type it does not know. `tests/integration/test_frontend_wildcard_arms.py` holds every such `match` to naming each variant of today in its own patterns, with comments and literals unread, so a variant added to `grund-core` is red there until the frontend renders it. `test_promised_type_shapes.py` reads the attribute from the definitions and holds it to the inventory's `frozen at 1.0` rows, and it runs `cargo test --doc` over the crate, whose root carries four `compile_fail` doctests: they show a removed function, a removed method, a record literal and an exhaustive match failing to compile the way an embedder's code does, each beside a twin that differs in one line and compiles.

### 2.2 `integrations` is the one integrations orchestration

`writers/integrations_api.rs` holds `integrations`, the only sequence that refuses, detects, describes and installs for `grund integrations` ([§FS-integrations.1.3.1](../functional-spec/FS-integrations.md#131-the-engines-answer)). It answers with the records of `writers/integrations_output.rs`, and both frontends read that one answer: the CLI renders its text from it, and `writers/integrations_data.rs` projects it into the JSON the Python and Node packages return. The projection keeps the hosts' own name parsing and their `invalid-client`, `invalid-agent` and `invalid-preference` codes, and decides nothing the typed operation does not.

## 3. crates/grund-cli: the CLI binary

`crates/grund-cli`, the Cargo package published as `grund`: what `cargo install grund` produces and what the npm/PyPI packages wrap. It owns the installed binary, prints help and version, restores SIGPIPE, and routes top-level commands to CLI-local wrappers over the data APIs. Every command renders through one of those wrappers, `integrations` included: its argument parsing, detection and artifact printing, and its `--write` reports are `cli_integrations.rs` and `cli_integrations_write.rs`, which render the answer of the engine's `integrations` and carry no write sequence of their own ([§FS-integrations.1.3](../functional-spec/FS-integrations.md#13-which-side-owns-what), [§FS-distribution.3.1.3](../functional-spec/FS-distribution.md#313-integrations-is-the-one-integrations-entry-point)). Synchronous; no async runtime, no LSP types, no JSON-RPC.

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

The grund-py crate is the whole Python binding, as `crates/grund-node/` is the whole
Node one. Its Rust half owns only PyO3 conversion and exception policy over supported core
data APIs ([§FS-distribution.3.3](../functional-spec/FS-distribution.md#33-python-grund-pypi-package)). Frozen dataclasses, tuples and read-only config
mappings in `crates/grund-py/python/grund` expose the public schema; private
`grund._native` holds the abi3-py310 extension. `crates/grund-py/pyproject.toml` selects
maturin, which puts the workspace manifest, lockfile and `grund-core` into the sdist for
independently installing it unpacked, with accurate types/py.typed.

Rust work runs with the GIL released; calls check Python interrupts on return and
carry per-call scope rather than changing cwd ([§FS-distribution.3.3.4](../functional-spec/FS-distribution.md#334-silent-synchronous-calls)).
Core supplies warning/failure/path/install adapters; Python does not walk, resolve,
check, parse or reproduce managed-write behavior. No dependency on CLI/LSP/Node is
allowed. `tests/bindings/python/` compares complete data and canonical bytes against the
shared oracle in `tests/bindings/`, separately
from the frozen CLI wire projection ([§FS-distribution.3.0.3](../functional-spec/FS-distribution.md#303-complete-data-and-canonical-parity)).
The Python gate runs that acceptance on every push and pull request ([§AR-bindings.6.2](AR-bindings.md#62-the-python-acceptance-runs-in-the-gate)).

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

### 6.1 Native extension build setup

`crates/grund-py/build.rs` owns the platform linker setup for raw Cargo extension
builds ([§FS-distribution.3.3.7.1](../functional-spec/FS-distribution.md#3371-raw-cargo-extension-builds)), beside its existing interpreter/ABI checks. Use
PyO3's supported extension-module link-argument mechanism, scoped to the cdylib,
so maturin and the same-source command-agreement host need no separate copies
of Darwin flags. Ordinary workspace builds retain their executable/test linking.

### 6.2 The Python acceptance runs in the gate

The Python binding's acceptance, `tests/bindings/python/run.py`, is a test of the Python
gate ([§AR-ci.3.4](AR-ci.md#34-the-python-gates-inputs)), as the Node acceptance is ([§AR-bindings.5](AR-bindings.md#5-grund-node-the-napi-rs-binding)): the module
`tests/integration/test_python_binding_acceptance.py` runs it, so a change that moves
what the binding returns fails its own pull request rather than the next person who runs
the suite by hand. The module is the `--only-python-acceptance` selection of [§AR-ci.3.4.3](AR-ci.md#343-selecting-the-binding-acceptances-and-the-slow-tests),
so it is still a test of the Python gate, read as the gate's jobs as the Node acceptance is:
the Python acceptance jobs of [§AR-ci.13.1](AR-ci.md#131-one-job-per-language), Linux and Windows, run it on every push, pull
request and merge group, beside the Python jobs rather than inside them, and the local
`python-test` hook runs it too. Every build it makes shares one persistent Cargo target,
`GRUND_PYTHON_TARGET` ([§AR-ci.3.4.5](AR-ci.md#345-the-python-acceptances-shared-build-target)), rather than a cold temporary one.

The module installs this checkout's `crates/grund-py` from its source directory into a
virtual environment of its own under the wrapper's scratch, outside the checkout, once
per run, and runs `run.py` with that environment's interpreter. Every build of the run,
that install and the clean-checkout and sdist installs `run.py` makes itself, is at the
dev profile: the same source as a release without its LTO, because what the suite proves
is what the package returns and that it installs, not how it was optimized. The installed package is
the one `binding()` admits: a local-source install of this checkout, never a registry
package or whatever the gate's own interpreter has installed. The Rust side is the gate's
`GRUND_BINDINGS_ORACLE` ([§AR-ci.3.4](AR-ci.md#34-the-python-gates-inputs)), which the shared oracle side in
`tests/bindings/support.py` reads when it is set, so the Python and the Node adapters are
compared against one same-source build; a run on its own still falls back to
`target/debug/grund-binding-oracle`.

What [§AR-ci.3.4](AR-ci.md#34-the-python-gates-inputs) asks of an input it asks of this install: a build or install that fails
fails the test, with its output, and never skips it. The test passes only when every
module `run.py` discovers passes, and on failure it prints `run.py`'s whole output, so
the failing acceptance test is named in the gate's own log.

#### 6.2.1 Pending-interrupt acceptance evidence

The POSIX acceptance in `tests/bindings/python/test_boundary.py` proves
[§FS-distribution.3.3.4](../functional-spec/FS-distribution.md#334-silent-synchronous-calls),
then checks an independent root under
[§FS-distribution.3.3.3](../functional-spec/FS-distribution.md#333-per-call-scope-and-path-encoding).
Its isolated child explicitly installs `signal.default_int_handler` for SIGINT
and unblocks SIGINT. Regression runs exercise inherited SIG_IGN and, where
`pthread_sigmask` is available, a blocked SIGINT mask, without changing the test
runner's lasting signal state or directing SIGINT at it.

Bounded synchronization establishes that SIGINT became pending during the real
native operation before checking its return. Interpreter startup, elapsed time,
a larger workload, or a sender merely being launched do not establish this fact.
KeyboardInterrupt must come from that operation's return: fixture setup, signal
setup, synchronization, cleanup, and later Python work cannot satisfy the check.
Ordinary completion and retrying until one attempt succeeds cannot satisfy it
either. The independent follow-up call remains part of the evidence.

If delivery cannot be established, fail within a bound with `interrupt evidence
unavailable` and the failed synchronization stage, rather than claim a swallowed
interrupt or pass without evidence. A held-sender regression exercises this
negative control against the live acceptance harness; it must confirm that the
helper was held before interpreting the result. Positive scheduling evidence
also confirms delivery during native work. Protocol changes must adapt the
control, rather than pass because an old source-text pattern stopped matching.
Keep setup/transport failures distinct from a library delivery failure.

Bound every wait and reap helpers, close descriptors, and remove fixtures on
success and failure. Preserve POSIX acceptance coverage; a Linux-only validation
instrument or a blanket skip cannot replace it. These are private test mechanics,
not a public testing API or a change to the Rust return boundary.

#### 6.2.2 Every step fails with its output, and the one fetch is retried

"With its output" holds for every command the Python acceptance runs, not only the
install of [§AR-bindings.6.2](AR-bindings.md#62-the-python-acceptance-runs-in-the-gate). `tests/bindings/support.py` has one helper, `checked`, that
runs a command with its output captured and, when it exits non-zero, fails the test with
the command, its exit status, its stdout and its stderr in the failure's own text. Every
captured command in `tests/bindings/support.py` and `tests/bindings/python/` goes through
it, the oracle call of `rust_call` included; none passes `check=True`, whose
`CalledProcessError` prints the exit status and drops what the command said. A failed
step on a CI runner is then diagnosable from the job log alone.

The acceptance has one registry fetch: `test_build`'s `pip install build`, the build
frontend its sdist check needs. `test_build.install_build` makes it, and retries it
only for what a retry can cure, as [§AR-ci.3.4.1.3](AR-ci.md#3413-the-typescript-compiler-a-prepared-input) does for npm. An install whose pip
output names a broken or refused connection, a timeout or a failed name resolution is
tried again after a short pause, up to three attempts in all. Any other failure, and a
third network failure, fails the test at once with pip's own output and never turns into
a skip ([§AR-ci.3.4](AR-ci.md#34-the-python-gates-inputs)). pip's own `--retries` does not replace this: it had already run on
the CI attempt that failed.

`tests/integration/test_python_acceptance_steps.py` holds the harness to this without
the network: each helper's failure carries a failing command's stdout and stderr, no
captured `check=True` call is left, and `install_build`, against a local index, is
tried three times when every connection is dropped and once when the index answers
that it has no such package, and fails both times with pip's message.

## 7. Why this shape

- **One source of truth for behavior.** Bug fixes and new rules land in `grund-core` and reach all three ecosystems on the next release.
- **No re-implementation.** Neither Node nor Python developers need to maintain a parallel parser or a parallel rule set.
- **Fast everywhere.** The compiled engine is the same in all three; the bindings add only a thin marshalling layer.
- **Independent release cadence per crate when needed.** A Node-only fix in `grund-node` does not require a `grund-core` version bump. Such a candidate is scoped `binding-only`: its manifest records the binding's own package version beside the unchanged engine and CLI version, and it never claims ecosystem completeness ([§FS-distribution-candidate.6.3](../functional-spec/FS-distribution-candidate.md#63-a-binding-only-candidate-never-claims-completeness)).
