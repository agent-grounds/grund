# Installation

`grund` is two commands that install independently: the `grund` CLI and the
`grund-lsp` language server. Install the one you use; neither needs the other
([§FS-distribution.1.3](../functional-spec/FS-distribution.md#13-the-cli-does-not-pull-in-the-lsp), [§FS-lsp.2.1](../functional-spec/FS-lsp.md#21-install)).

## What is published

The CLI and the language server are published as Cargo crates, and build from
source with a Rust toolchain:

```bash
cargo install grund
cargo install grund-lsp
```

Each GitHub release also carries prebuilt, profile-guided-optimized archives for
the six platforms below, each beside its `.sha256` ([§FS-distribution.4.9](../functional-spec/FS-distribution.md#49-distributed-binaries-are-profile-guided-optimized));
`grund-lsp` archives sit beside the `grund` ones from the first release built
with this layout ([§FS-distribution.4.8](../functional-spec/FS-distribution.md#48-one-old-glibc-baseline-one-pinned-toolchain)).

**Nothing is published to npm or PyPI.** The npm packages `grund-cli` and
`grund-lsp` with their `@grund-cli/*` and `@grund-lsp/*` platform packages, and
the PyPI distributions `grund` and `grund-lsp`, are built and rehearsed as a
local candidate only ([§FS-distribution.4.13](../functional-spec/FS-distribution.md#413-a-candidate-is-rehearsed-before-anything-is-published)). Their names are provisional, and
`npm install grund-cli` or `pip install grund` does not install this project.
What follows shows how to build that candidate and install from its own files.

## Supported platforms

| Row | OS / payload floor | Rust target | npm and wheel packages |
|---|---|---|---|
| `linux-x64-gnu` | Linux x64, glibc 2.17 | `x86_64-unknown-linux-gnu` | yes |
| `linux-arm64-gnu` | Linux arm64, glibc 2.17 | `aarch64-unknown-linux-gnu` | yes |
| `darwin-x64` | macOS x64, 11.0 | `x86_64-apple-darwin` | yes |
| `darwin-arm64` | macOS arm64, 11.0 | `aarch64-apple-darwin` | yes |
| `win32-x64-msvc` | Windows x64, Windows 10 with the MSVC 14 runtime | `x86_64-pc-windows-msvc` | yes |
| `win32-arm64-msvc` | Windows arm64, Windows 11 with the MSVC 14 runtime | `aarch64-pc-windows-msvc` | archives only |

The table is [§FS-distribution-candidate.1.1](../functional-spec/FS-distribution-candidate.md#11-six-rows-five-of-them-registry-rows)'s;
`python3 scripts/distribution/candidate.py matrix` prints it as JSON. On every
row with packages, the npm packages support Node 22 and 24, and the `grund`
wheel supports CPython 3.10 through 3.14 from one stable-ABI wheel
([§FS-distribution-candidate.1.2](../functional-spec/FS-distribution-candidate.md#12-every-registry-row-proves-every-runtime-it-promises)). Node itself needs more than the payload: on
Linux, kernel 4.18 and glibc 2.28; on macOS, 11 for Node 22 and 13.5 for Node 24
([§FS-distribution-candidate.1.3](../functional-spec/FS-distribution-candidate.md#13-runtime-floors-are-inspected-not-assumed)).

## From a local candidate

Build the candidate for your platform's row from a clean checkout. This needs
Rust 1.95.0 with the `llvm-tools-preview` component, Node, a Python of 3.10 or
newer, and on Linux Docker, because Linux payloads are built in the pinned
`manylinux2014` image ([the runbook](distribution-runbook.md#build-one-row) has
the details):

```bash
CANDIDATE=~/candidate
python3 scripts/distribution/candidate.py build --row linux-x64-gnu \
  --sha "$(git rev-parse HEAD)" --out "$CANDIDATE" --target-dir ~/candidate-target
```

The examples below are for `linux-x64-gnu`; on another row, use its row id in the
platform package's file name. Each installs from the candidate's own files and
reaches no registry.

### The CLI and its API

From the wheel, into a virtual environment with no index. The wheel puts the
`grund` executable itself in the environment's `bin/` and makes `import grund`
work:

```bash
python3 -m venv .venv
.venv/bin/pip install --no-index "$CANDIDATE"/pypi/grund-[0-9]*.whl
.venv/bin/grund --version
```

From npm tarballs, the package and its row's platform package together.
`--offline` keeps npm from asking a registry for the other rows' platform
packages, which it skips:

```bash
npm install --offline "$CANDIDATE"/npm/grund-cli-[0-9]*.tgz "$CANDIDATE"/npm/grund-cli-linux-x64-gnu-*.tgz
npx grund --version
```

That also makes `require("grund-cli")` and `import "grund-cli"` work; see the
[Node Promise API](node-api.md).

### The language server alone

The `grund-lsp` packages install the server and nothing else — no CLI, no Rust
toolchain — and put `grund-lsp` on `PATH` for an editor to launch
([editor setup](lsp.md)):

```bash
pipx install "$CANDIDATE"/pypi/grund_lsp-*.whl
grund-lsp --version
```

```bash
npm install --offline "$CANDIDATE"/npm/grund-lsp-[0-9]*.tgz "$CANDIDATE"/npm/grund-lsp-linux-x64-gnu-*.tgz
npx grund-lsp --version
```

### An archive

Each archive holds one directory with the executable, `LICENSE` and `README.md`,
beside a `.sha256` to check it against:

```bash
cd "$CANDIDATE"/archives
sha256sum -c grund-lsp-*-x86_64-unknown-linux-gnu.tar.gz.sha256
tar -xzf grund-lsp-*-x86_64-unknown-linux-gnu.tar.gz
```

## Building from source

Every package can build its commands from source instead of using a prebuilt
payload, but only when asked: nothing compiles on install
([§FS-distribution-candidate.4.1](../functional-spec/FS-distribution-candidate.md#41-npm-builds-from-source-only-when-asked), [§FS-distribution-candidate.4.2](../functional-spec/FS-distribution-candidate.md#42-python-builds-from-the-sdist-only-when-asked)).

- **npm.** In an installed `grund-cli`, `npm run build:source` builds the
  `grund` command and the Node addon into its `native/` directory from the
  bundled, locked sources; in `grund-lsp` it builds only the server.
- **PyPI.** `pip install --no-binary=:all:` with an sdist builds the extension
  and the `grund` command, or only the server for `grund-lsp`. pip fetches the
  `maturin` build backend from its index.
- **Cargo.** `cargo install grund` and `cargo install grund-lsp` always build
  from source.

A source build needs Rust and Cargo (the npm packages pin Rust 1.95.0), a linker
and the platform SDK — Xcode's command-line tools on macOS, the MSVC build tools
on Windows. It builds with LTO but no profile, so it behaves the same and runs
slower than a prebuilt payload. A missing prerequisite fails with a line naming
it ([§FS-distribution-candidate.4.3](../functional-spec/FS-distribution-candidate.md#43-a-source-build-needs-rust-and-says-so)):

```console
$ npm run build:source
error: cargo was not found: a source build of grund-cli needs Rust 1.95.0 and Cargo (https://rustup.rs), a linker and the platform SDK
```
