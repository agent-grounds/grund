# Node binding acceptance

Run `python tests/bindings/node/run.py` from the checkout. This tests the local
API rehearsal of [§FS-distribution.3.2.4](../../../docs/functional-spec/FS-distribution.md#324-acceptance-evidence).
Node 22/24, npm, Rust/Cargo, linker/SDK and dependency access or populated caches
are prerequisites. Scratch packages and Cargo targets are under `~/ag/tmp`.
No registry publication or installed-prebuilt platform certification occurs.
The checkout currently has no Node frontend; missing capability is an assertion
failure, never a skip. These tests are the contract before implementation. The
one platform skip is `fetch.mjs`'s explicit fetch write: its fixture fetcher is a
POSIX script, so it is skipped off unix exactly where the CLI's case runner skips
the same `requires-unix-shell` case.

The staging invocation is `node crates/grund-node/stage.mjs --out-dir <package>
--target-dir <external-cargo-target>`. It materializes only a disposable package
from `package-api.json` and the workspace version. The tests then run the
declared `npm run build:source`, pack it, install with lifecycle scripts disabled
in a fresh consumer, and run ESM/CJS consumers. The stage helper basename and its
flags settle an unspecified test workflow detail; #471 still owns the sole
committed assembly manifest.

`build/source.mjs` accepts `--target-dir` and locates its bundled source closure
at `build/sources/`; neither install nor module import builds automatically.
The closure includes adjusted workspace manifests, Cargo.lock, pinned toolchain,
core/node sources and core assets. Tests remove the installed native payload and
rebuild from that closure. Typescript 5.9.3 is a test-only pinned consumer compiler,
not a production package dependency.

`adapter.mjs` extends #470's common corpus through newline JSON requests
`{operation,args}`. Set `GRUND_BINDINGS_ORACLE` to that same-source Rust oracle.
Its `--metadata` returns `protocolVersion:1`, `sourceSha` and `engineVersion`;
ordinary invocations read one request on stdin and emit the complete canonical
`failure,result,run_cautions` envelope. These small framing choices fill an
unspecified transport detail and do not define a competing common comparator.
Native-only fields are compared; there is no CLI-as-Rust-oracle substitute.
The Node adapter maps fixed host names only, leaving config/details authored keys
opaque. Arrays preserve engine order and keys sort by UTF-8 bytes. Corpus C0
escaping differs from CLI DEL/C1 escaping. CLI projections are separate test-only
renderers in `check-wire.mjs` and `query-wire.mjs`; they do not implement API
operations.

Runtime tests require a separate addon built with `--test-seams`, enabling only
the future `grund-node/test-node-runtime` Cargo feature. The production builder
does not enable it, and its addon has no `__test` export. The test-build
`__test` object controls native barriers/faults; no public operation takes a
fault-injection option. This file specifies the seam; it implements none of the
native behavior. Required methods:

- `holdNext(operation)`, `holdNextWriter(outcome)`, `waitStarted()`, `release()`:
  arm one job; readiness is signalled from active compute (and after writer lease
  acquisition for writers), before filesystem side effects. Hold only the armed
  job; subsequent reads must remain available. Outcomes are success/error/panic.
- `panicNext(stage)`, `failWorkerNext()`: inject an unwind at conversion,
  compute, Rayon, resolve or reject, or a worker scheduling/completion failure.
- `delayNext(operation, milliseconds)`: delay active native compute with no JS
  timer/port, so pending-liveness.cjs proves the native job keeps the host alive.
- `installOtherHook()`, `hookCalls()`, `panicOutsideGuard()`: an independent Rust
  caller's counting hook coexists with the binding dispatcher. The test harness
  arranges the prior hook before dispatcher installation, rather than replacing
  the binding dispatcher with a printing hook. Outside-guard panic is caught by
  that caller; its hook counts but writes nothing.
- `waitIdle()`, `resourceCounts()`: wait for Rust jobs after environment teardown,
  and count live job/environment/writer resources with stable baseline values.
- `failWriteAfter(operation, count)`: inject one test-only I/O failure after that
  many owned writes in init/fmt, retaining completed events/changes as partial
  output. This is not a production fault-injection option or rollback promise.

No test-only native seam has been implemented in this specification commit.
Until the frontend/build workflow exists, consumer setup blocks all behavioral
assertions. The handoff records that distinction and the exact observed failure.
