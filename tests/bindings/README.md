# Shared binding protocol, and each binding's acceptance

Required by [§FS-distribution.3.0.3](../../docs/functional-spec/FS-distribution.md#303-complete-data-and-canonical-parity) and [§FS-distribution.3.3](../../docs/functional-spec/FS-distribution.md#33-python-grund-pypi-package). This directory holds
what both bindings share: the oracle side in `support.py`, with the build target
each checkout's acceptances build into, and the operation/fixture matrix in
`corpus.py`. Each binding's own side is a directory
of its own, as its source is one directory under `crates/`
([§AR-bindings.1](../../docs/architecture/AR-bindings.md#1-target-workspace-layout)): Python's in [`python/`](python/), Node's in [`node/`](node/).

The Python gate runs Python's on every push and pull request, through
`tests/integration/test_python_binding_acceptance.py`
([§AR-bindings.6.2](../../docs/architecture/AR-bindings.md#62-the-python-acceptance-runs-in-the-gate)), as it runs Node's. To run it by hand from the checkout,
install it with `python -m pip install crates/grund-py` first:

```sh
python tests/bindings/python/run.py
```

The implementer supplies a core-only test driver at
`target/debug/grund-binding-oracle` (with `.exe` on Windows). It reads one JSON
request from stdin with `operation`, native `root`, positional `args`, keyword
`options`, and optional isolated `home`. It uses supported core data-returning APIs
and the approved additive adapters. It must not call a CLI or a Python converter.
Build it with an explicit checkout target directory before running acceptance.
`GRUND_BINDINGS_ORACLE`, when set, names another build of it, as the Python gate's.

It returns one JSON response with `data` (the complete public envelope) and
`canonical` (base64 of independently encoded canonical UTF-8 bytes). For checks
it also returns `cli_stdout`/`cli_stderr`, the frozen finding projection; tests
compare these to existing authoritative goldens. Raw caution text is kept separate.
For writer requests it executes against only the supplied isolated root/home,
mapping Python write defaults to the corresponding engine preview or opt-in.
Host TypeError/ValueError validation is tested directly, outside engine parity.

The Python adapter, `python/adapter.py`, recursively enumerates every dataclass
field and tuple; it does not whitelist report fields. Deep equality and
canonical byte equality are both mandatory. The canonical comparator is a test-only encoder, not a shipped
conversion implementation. The operation/fixture matrix lives in corpus.py.
Node's adapter exists in [`node/`](node/): `node/adapter.mjs` speaks the same
request/response contract to the same Rust oracle, and `node/run.py` runs it
apart from Python's `python/run.py`. Neither binding's parity claims the other's
coverage.

Local build tests copy only git-tracked source to scratch under `~/ag/tmp`, install
in fresh environments, build an sdist and install its unpacked sources separately.
No upload, release command, registry credential or CLI console script is required.
