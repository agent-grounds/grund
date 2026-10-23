# AR-ci-terminal-inputs: The terminal watch acceptance selects this checkout's CLI

The terminal harness's binary selection realizes [§FS-repository-maintenance.3](../functional-spec/FS-repository-maintenance.md#3-checkout-owned-terminal-acceptance)
within the Python gate's input convention ([§AR-ci.3.4](AR-ci.md#34-the-python-gates-inputs)).

## placement: What setup selects

```text
gate inputs, checkout root ─► [ terminal harness setup ] ─► CLI executable ─► PTY tests
```

`tests/integration/test_watch_terminal.py` selects its executable once in class
setup, before the PTY tests exercise [§FS-check.6.2.2](../functional-spec/FS-check.md#622-owned-terminal-screen). This is acceptance
infrastructure: a meter outside the shipped CLI and core ([§AR-system.5](README.md#5-what-holds-the-shape)).

## terms: Terms

Leans on [§FS-terms.terms.8](../functional-spec/FS-terms.md#terms8-the-architectures-own-words) (meter),
and [§FS-repository-maintenance.terms](../functional-spec/FS-repository-maintenance.md#terms-terms) (checkout root).

- **Supplied input**: an executable or Cargo target selected by the caller
  through the existing Python gate environment convention.

## 1. Executable and build selection

When `GRUND_BUILT` is supplied, setup uses that executable without building
again, following the caller-owned input convention of [§AR-ci.3.4](AR-ci.md#34-the-python-gates-inputs). Otherwise
it builds `grund` with `--locked` from the harness's checkout root into
`CARGO_TARGET_DIR` when supplied, or `target` under that root by default,
and runs that target's `debug/grund` executable (`grund.exe` on Windows).
A relative explicit target is resolved from the checkout root, the build's
working directory, even when the harness is invoked from elsewhere. The build
and the executable selection use the same target.

This default never selects a machine-wide target. Cargo's path-package
freshness can reuse another checkout's `grund-core` even after sequential
builds, so cleaning a shared target is not a remedy. Explicit artifacts and
targets remain the caller's responsibility; setup does not validate their
source provenance. Build failures remain errors under [§AR-ci.3.4](AR-ci.md#34-the-python-gates-inputs).

## 2. Setup regression

`tests/integration/test_watch_terminal_inputs.py` loads the actual terminal
harness under two simulated checkout roots and one home, intercepting Cargo
to record its effective target and setup's executable. It pins distinct
checkout-local defaults, absolute and relative explicit targets, and supplied
gate-binary reuse without a build. The six PTY behavior tests remain the proof
of [§FS-check.6.2.2](../functional-spec/FS-check.md#622-owned-terminal-screen); the setup regression adds no terminal semantics.
