# grund Node API

This is the native Node-API 8 frontend for the shared Rust engine
([§AR-bindings.5](../../docs/architecture/AR-bindings.md#5-grund-node-the-napi-rs-binding)).
Node 22.x and 24.x support ESM and CommonJS. This delivery provides a local,
API-only rehearsal of the planned `grund-cli` npm package. It is unpublished,
and the rehearsal has no executable launcher.

```js
import { check, show, GrundError } from 'grund-cli';
const report = await check('./repo', { suggestions: true });
console.log(report.report.errors);
try {
  const body = await show('FS-check', { root: './repo', mode: 'brief' });
  console.log(body.body);
} catch (error) {
  if (error instanceof GrundError) console.log(error.failure.kind, error.failure.code);
  else throw error;
}
```

All 18 operations return Promises: check, scan, show, showBatch, refs, list,
listSizes, cover, fmt, proposeId, init, completeIds, effectiveConfig,
validateConfig, fetch, integrations, referenceStyle, agentSetupInstructions.
The package exports readonly TypeScript records and strict option declarations.
CommonJS uses `const { check } = require('grund-cli')` and shares the ESM
`GrundError` class. Results carry `runCautions` separately from report findings.
Checks containing errors resolve; invocation failures reject `GrundError`.

## Local build and installation

Require Rust/Cargo 1.95.0, Node/npm, a linker and the target's SDK, plus dependency
access or a populated Cargo cache. No napi CLI, node-gyp or downloaded Node
headers are needed. From a checkout:

```sh
node crates/grund-node/stage.mjs \
  --out-dir "$HOME/ag/tmp/grund-node-package" \
  --target-dir "$HOME/ag/tmp/grund-node-target"
cd "$HOME/ag/tmp/grund-node-package"
npm run build:source -- --target-dir "$HOME/ag/tmp/grund-node-target"
npm pack
# In a fresh consumer, npm install --ignore-scripts <the packed tarball>.
```

Inside an unpacked installed package, `npm run build:source` rebuilds
`native/grund.node` using only `build/sources`. This is an explicit fallback;
installation, import and API calls never compile or download anything.
The Rust toolchain is pinned, dependencies are locked, builds use external
target directories, and the addon requires unwinding. A missing/corrupt addon
allows import but causes calls to reject with `load/native-load`.

## Writers and concurrency

Filesystem and engine work runs on libuv workers. Each call captures cwd and
copies inputs synchronously; concurrent reads have independent roots/options.
Default fmt previews; `fmt({root, write:true})` authorizes writes. Init writes
by default; `dryRun:true` or `check:true` previews. Fetch explicitly executes
the configured fetcher; other reads never do. Integration installation requires
`write:true` and retains core ownership and active-agent safeguards.

One loaded addon permits one active writer across all roots. Overlapping
writers reject `busy/writer-busy` before side effects. This is no filesystem
lock or snapshot, cancellation or rollback API. Separate processes/addon copies
and external writers require caller coordination. Recoverable unwinds become
silent native failures; abort/OOM are outside that recovery contract.

## Packaging handoff

`package-api.json` is an export/type/files/source-build fragment. The staging
helper creates a disposable manifest; packaging ticket #471 owns the sole
assembly manifest, CLI wrappers, platform packages and publication preparation.
The private addon exports `invoke` and `metadata`. Metadata matches
apiSchemaVersion=1, napiVersion=8, engineVersion, packageVersion and Rust target.
A matching local payload wins; a present corrupt/incompatible local payload
fails without platform fallback.

The assembly manifest can set `grund.platformPackages`, keyed by exact Rust
target. Each selected exact-version package exposes
`{addonPath: absolutePath, metadata}` and its package version. The fallback
name is `@agent-grounds/grund-cli-<rust-target>`; #471 chooses final package
names through that mapping. GNU Linux x64/arm64, macOS x64/arm64 and Windows x64
payload certification belongs to packaging, not to this local rehearsal.

## Maintainer checks

Binding acceptance is `python tests/bindings/node/run.py`; the existing
integration gate discovers it. A separate `--test-seams` build enables
`grund-node/test-node-runtime`; production builds expose no `__test`.
The native seams are private and accept no public-operation fault options.

The shared Rust oracle is sourced from #470's shared-core subset at
17e6647c84, extended here with Node framing
([§FS-distribution.3.0.3](../../docs/functional-spec/FS-distribution.md#303-complete-data-and-canonical-parity)).
The Python gate, `python scripts/run_python_gate.py`, builds it from `HEAD`
itself whenever `GRUND_BINDINGS_ORACLE` is unset
([§AR-ci.3.4](../../docs/architecture/AR-ci.md#34-the-python-gates-inputs)). To run `run.py` on its own, build and export it
after committing the tested source:

```sh
GRUND_BINDINGS_SOURCE_SHA="$(git rev-parse HEAD)" cargo +1.95.0 build \
  -p grund-core --example grund-binding-oracle --locked \
  --target-dir "$HOME/ag/tmp/grund-node-oracle"
export GRUND_BINDINGS_ORACLE="$HOME/ag/tmp/grund-node-oracle/debug/examples/grund-binding-oracle"
python tests/bindings/node/run.py
```

The oracle metadata must match the source commit and engine version. Complete
Rust/Node envelopes and the separate CLI stdout/stderr/status projections are
different proofs. Local compilation alone establishes neither.
“Adapt Node to #466” remains a later internal transition preserving this host
contract. Future #459/#463 behavior is not exposed until it exists in core.
