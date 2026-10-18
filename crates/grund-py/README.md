# grund Python frontend

The local Python API embeds `grund-core` through PyO3, as specified by
[§FS-distribution.3.3](../../docs/functional-spec/FS-distribution.md#33-python-grund-pypi-package).
PyPI publication is pending. This directory is the whole binding; build and
install it from the repository root:

```sh
python -m pip install crates/grund-py
```

See the [Python API guide](https://github.com/agent-grounds/grund/blob/main/docs/user-facing/python-api.md)
for runnable examples and signatures. Calls are synchronous, silent and return
immutable typed results. Completed checks return normally even with errors;
operational refusals carry structured exceptions.
`list_ids`, `list_sizes`, `show_batch`, `refs` and `cover` take keyword-only
`globs=()`, the coordinate patterns `grund list --glob` takes, and their results carry
a nullable `note`; `refs` also takes `from_=None`, the source `grund refs --from` reads,
with no ID ([§FS-coordinate-globs.bindings](../../docs/functional-spec/FS-coordinate-globs.md#bindings-node-and-python)).

## Source and ABI handoff

This crate's `pyproject.toml` uses maturin's mixed package layout. `python/grund`
beside it contains public functions, frozen dataclasses and `py.typed`; the
private native library is `grund._native`. Distribution and import names are both `grund`.
The extension uses `abi3-py310` and requires CPython 3.10+ with the GIL. Its build
script refuses PyPy, GraalPy and free-threaded interpreters. Normal Cargo builds
default to the CLI and have no Python dependency.

The `extension-module` Cargo feature is enabled by maturin rather than ordinary
Cargo workspace builds, so Cargo can also link its normal targets. The sdist
includes the workspace manifest and lockfile, the Rust source and assets of this
crate and `grund-core`, the Python modules and types, and the MIT licence, which
`license-file` places at `crates/grund-py/LICENSE` inside it. Build with
`python -m build --sdist crates/grund-py`; install the unpacked archive with
`python -m pip install .` from inside it.
No source path outside that archive is required.

Issue #471 owns the release wheel matrix, prebuilt CLI payload and publication.
This frontend adds no console script. A future `grund` executable can coexist
with the public package directory and private `_native` library. `grund-lsp`
remains a separate installation. No registry credentials, tags or release
dispatch are needed for this local handoff.

## Engine transition

Carry **Adapt Python marshalling to #466/#453/#454**: replace internal Rust
adapters when that core transition lands, preserve the approved Python schema,
and rerun the complete parity corpus. The additive `EmbeddingRequest`/envelope
surface uses schema fields instead of exposing Config or Findings layouts.
This package's acceptance lives in `tests/bindings/python/`, as Node's lives in
`tests/bindings/node/`; each runs against the shared Rust oracle in
`tests/bindings/`, and this package's parity claims no Node coverage.

## Checking locally

After installing the checkout into the checking interpreter:

```sh
cargo build -p grund-core --example grund-binding-oracle --target-dir target
cp target/debug/examples/grund-binding-oracle target/debug/grund-binding-oracle
python tests/bindings/python/run.py
```

On Windows copy the corresponding `.exe`. The core-only oracle has independent
canonical encoding and frozen CLI projection. The checking step also owns
clean-checkout/sdist installs and the complete repository gate.
