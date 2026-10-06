# grund Python frontend

The local Python API embeds `grund-core` through PyO3, as specified by
[§FS-distribution.3.3](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-distribution.md#33-python-grund-pypi-package).
PyPI publication is pending. Build/install from the repository root:

```sh
python -m pip install .
```

See the [Python API guide](https://github.com/agent-grounds/grund/blob/main/docs/user-facing/python-api.md)
for runnable examples and signatures. Calls are synchronous, silent and return
immutable typed results. Completed checks return normally even with errors;
operational refusals carry structured exceptions.

## Source and ABI handoff

The root `pyproject.toml` uses maturin's mixed package layout. `python/grund`
contains public functions, frozen dataclasses and `py.typed`; the private native
library is `grund._native`. Distribution and import names are both `grund`.
The extension uses `abi3-py310` and requires CPython 3.10+ with the GIL. Its build
script refuses PyPy, GraalPy and free-threaded interpreters. Normal Cargo builds
default to the CLI and have no Python dependency.

The `extension-module` Cargo feature is enabled by maturin rather than ordinary
Cargo workspace builds, so Cargo can also link its normal targets. The sdist
includes workspace manifests/lockfile, required Rust source/assets, integration
workspace metadata/source, Python modules/types and the MIT licence. Build with
`python -m build --sdist`; install the unpacked archive with `python -m pip install .`.
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
Node (#469) adds its adapter to the same corpus; current parity infrastructure
targets Rust/Python and does not claim Node has been implemented or checked.

## Checking locally

After installing the checkout into the checking interpreter:

```sh
cargo build -p grund-core --example grund-binding-oracle --target-dir target
cp target/debug/examples/grund-binding-oracle target/debug/grund-binding-oracle
python tests/bindings/run.py
```

On Windows copy the corresponding `.exe`. The core-only oracle has independent
canonical encoding and frozen CLI projection. The checking step also owns
clean-checkout/sdist installs and the complete repository gate.
