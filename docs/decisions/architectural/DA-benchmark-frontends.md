# DA-benchmark-frontends: the benchmarks measure the language server and the Python extension, not the core library or the Node addon

**Status:** Accepted
**Date:** 2026-10-13

## 1. Context

[§DA-benchmark-instruction-counting.3.3](DA-benchmark-instruction-counting.md#33-benchmark-the-binary-because-that-is-what-gets-invoked) put the instruction benchmarks on the `grund` binary, because a process is what the agent loop invokes, and left the library unmeasured. Since then the engine has three more hosts ([§AR-bindings](../../architecture/AR-bindings.md#ar-bindings-target-shape-for-exposing-the-rust-engine-on-three-platforms)): the `grund-lsp` server an editor runs on every save, the Python extension, and the Node addon. The performance-surface inventory names each as a designated mode ([§AR-benchmarks.1.6.1](../../architecture/AR-benchmarks.md#161-the-designated-performance-sensitive-modes)), and until #699 each was waived. #580's approved proposal settled which of them a scenario measures, and this record extends .3.3 with that answer. It does not move .3.3's verdict.

## 2. Decision

The scenarios of [§AR-benchmarks.1.7](../../architecture/AR-benchmarks.md#17-frontends) follow four rules:

- **No core-library scenario.** Every frontend calls the core, and a CLI scenario already counts it together with the start-up cost an agent actually pays. That is .3.3's reason, and it still holds.
- **The language server: yes,** as the `grund-lsp` binary driven over stdio by a scripted session (`initialize`, `didOpen`, `hover`, `shutdown`), the same session its PGO training uses ([§DA-pgo-release](DA-pgo-release.md#da-pgo-release-distributed-binaries-are-pgo-built-trained-on-the-benchmark-workload)). It runs in the pull-request tier.
- **Python: yes,** with Callgrind's collection toggled on at the extension's entry point, so interpreter start-up does not drown the signal. It runs in the nightly tier.
- **Node: waived until shown repeatable,** because V8's JIT and garbage-collector threads make instruction counts move between runs. The inventory records the waiver and this reason.

## 3. Why this shape

### 3.1 A frontend is measured as the process its host runs

The language server is a binary, so .3.3's argument applies to it unchanged: the editor pays for the server's start-up, its framing and its rescans, and a scenario that runs the binary counts all of them. Driving it with the session PGO already trains on keeps one definition of "what an editor does", as [§AR-benchmarks.1.5](../../architecture/AR-benchmarks.md#15-the-same-command-list-trains-pgo) keeps one for the CLI. The session is fed on stdin, so Callgrind counts the server and not the client that wrote it.

### 3.2 Python is measured from its entry point, not from the interpreter's start

A Python process spends far more instructions starting the interpreter than the extension spends on a small fixture, and the interpreter's start-up is not grund's code. Toggling collection on at the extension's entry point counts what grund adds to a Python host. The interpreter and the extension's build are pinned like every other input to the count ([§AR-ci.5](../../architecture/AR-ci.md#5-benchmark-job)). Building the extension costs more than the pull-request tier's wall clock can carry, so the scenario runs nightly.

### 3.3 A count that moves on one binary cannot gate anything

The gate is implementable only because a binary's count does not move between runs ([§DA-benchmark-instruction-counting.3.1](DA-benchmark-instruction-counting.md#31-determinism-is-what-makes-fail-on-regression-implementable)). V8 compiles and collects on threads of its own, on a schedule the count sees, so two runs of one Node build differ by more than the same-binary bound ([§AR-benchmarks.7.3](../../architecture/AR-benchmarks.md#73-the-same-binary-bound)). A scenario whose jitter exceeds its limit would fail unchanged pull requests or pass regressions. The waiver lasts until a measurement shows the counts repeatable.

## 4. Consequences

- The pull-request tier gains a `grund-lsp` row, and the release's language-server training evidence keeps its bytes.
- The nightly tier gains a Python row and the extension build it needs.
- The `node` mode stays waived, with a reason that holds rather than an issue to wait for.

## 5. Alternatives considered

- **A `grund-core` benchmark beside the binary's.** Rejected for .3.3's reason: it would count less than any host pays, and the binary rows already count the core.
- **A Python scenario counting the whole process.** Rejected: interpreter start-up would dominate the count, and a regression in the extension would hide under it.
- **A Node scenario with a wider limit.** Rejected: a limit wide enough for V8's jitter would admit the regressions the gate exists to stop.
