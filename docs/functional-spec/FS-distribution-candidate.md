# FS-distribution-candidate: one candidate is assembled, rehearsed and verified before any registry sees it

The detail of [§FS-distribution.4.13](FS-distribution.md#413-a-candidate-is-rehearsed-before-anything-is-published): what the cross-registry release assembles from one commit, how a credential-free rehearsal installs and proves it on every advertised row, the manifest that records it, and the disabled, gated publisher that may one day upload exactly those bytes. Nothing here publishes. A candidate that passes everything below is ready to publish, not published; the npm and PyPI names stay unpublished until a separately authorized operator turns the publisher on ([§FS-distribution-candidate.8.1](FS-distribution-candidate.md#81-publication-is-disabled-and-gated)). Serves [§GOAL-multi-language](../goals.md#goal-multi-language-same-engine-three-platforms) and [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

## terms: Terms

Leans on [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, verdict).

- **candidate** — Every artifact one release would publish, assembled from one commit at one version, with the `manifest.json` that lists them. Never rebuilt once assembled.
- **registry row** — One of the five platform rows of [§FS-distribution-candidate.1.1](FS-distribution-candidate.md#11-six-rows-five-of-them-registry-rows) that npm and PyPI publish for. The sixth row ships downloadable archives only.
- **payload** — One compiled binary a candidate carries: the `grund` executable, the `grund-lsp` executable, the Node addon or the Python extension, for one Rust target.
- **placement** — Where one payload sits inside one artifact, with the digest it has there.
- **platform package** — An npm package carrying one registry row's payloads, named `@grund-cli/<suffix>` or `@grund-lsp/<suffix>`; the names are provisional until the scopes are held ([§FS-distribution-candidate.8.2](FS-distribution-candidate.md#82-identity-is-not-authority)).
- **launcher** — The `grund` and `grund-lsp` executables the npm packages put on `PATH`, which only find their platform payload and run it.
- **rehearsal** — The manual, credential-free run that assembles a candidate, installs it fresh on every registry row and records what each row proved.
- **receipt** — The record one row's rehearsal writes of the checks it passed against one manifest digest.

## 1. The support matrix

The rows a candidate is built for, the floors a payload runs on, and the targets it does not promise.

### 1.1 Six rows, five of them registry rows

| Row | OS / payload floor | Rust target | npm suffix, both families | Wheel platform | Installed-artifact runner |
|---|---|---|---|---|---|
| `linux-x64-gnu` | Linux x64 / glibc 2.17 | `x86_64-unknown-linux-gnu` | `linux-x64-gnu` | `manylinux2014_x86_64` | `ubuntu-latest`, pinned manylinux container |
| `linux-arm64-gnu` | Linux arm64 / glibc 2.17 | `aarch64-unknown-linux-gnu` | `linux-arm64-gnu` | `manylinux2014_aarch64` | `ubuntu-24.04-arm`, pinned manylinux container |
| `darwin-x64` | macOS x64 / 11.0 | `x86_64-apple-darwin` | `darwin-x64` | `macosx_11_0_x86_64` | `macos-15-intel` |
| `darwin-arm64` | macOS arm64 / 11.0 | `aarch64-apple-darwin` | `darwin-arm64` | `macosx_11_0_arm64` | `macos-15` |
| `win32-x64-msvc` | Windows x64 / Windows 10, MSVC 14 runtime | `x86_64-pc-windows-msvc` | `win32-x64-msvc` | `win_amd64` | `windows-latest` |
| `win32-arm64-msvc` | Windows arm64 / Windows 11, MSVC 14 runtime | `aarch64-pc-windows-msvc` | deferred | deferred | `windows-11-arm`: downloadable binaries only |

The first five are registry rows. The row ids are the npm suffixes the Node binding fixed in [§FS-distribution.3.2.3.2](FS-distribution.md#3232-moduleruntimeload-interface), and the table is the one place a script, a workflow and a test read the matrix from: `scripts/distribution/candidate.py matrix` prints it as JSON, one object per row, and nothing else spells a row by hand. Every row is built with Rust 1.95.0, and both Linux rows in the digest-pinned `manylinux2014` images [§FS-distribution.4.8](FS-distribution.md#48-one-old-glibc-baseline-one-pinned-toolchain) names.

### 1.2 Every registry row proves every runtime it promises

Each registry row is installed and exercised on Node 22.x and 24.x, and on CPython 3.10, 3.11, 3.12, 3.13 and 3.14. Node loads one N-API 8 addon per row. Python loads one `cp310-abi3` extension per row: the binding builds against the stable ABI from CPython 3.10 ([§FS-distribution.3.3.7](FS-distribution.md#337-local-source-and-typing-handoff)), so one wheel serves all five interpreters, and the rehearsal installs that one wheel into each of the five rather than trusting the tag. `grund-lsp` wheels carry no extension and are tagged `py3-none-<platform>`.

### 1.3 Runtime floors are inspected, not assumed

A payload's floor is the table's: glibc 2.17 on Linux, macOS 11.0, Windows 10 or 11 with the MSVC 14 runtime. The host runtimes add their own: Node on Linux needs kernel 4.18 and glibc 2.28, Node 22 on macOS needs 11 and Node 24 needs 13.5. The rehearsal reads each payload's linkage and deployment metadata — the highest glibc symbol version a Linux payload needs, the minimum OS version a macOS payload declares, the runtime DLLs a Windows payload imports — and refuses a payload above its row's floor. Linux payloads also run inside the old container, and npm installs run only on a host its Node supports.

### 1.4 Deferred targets gain no support

Windows arm64 has downloadable `grund` and `grund-lsp` archives and no npm or PyPI package. musl, other architectures, free-threaded CPython and PyPy are not promised. An install on any of them may try the source fallback ([§FS-distribution-candidate.4](FS-distribution-candidate.md#4-source-fallback)) and gains no support by succeeding.

## 2. The inventory

What one candidate holds, and how each registry chooses among it.

### 2.1 Thirty-nine artifacts from one version

A full-release candidate holds exactly:

- three crates: `grund-core`, `grund` and `grund-lsp`;
- npm `grund-cli` and `grund-lsp`, and one platform package per registry row for each, `@grund-cli/<suffix>` and `@grund-lsp/<suffix>` — twelve tarballs;
- PyPI `grund`: five `cp310-abi3-<platform>` wheels and an sdist;
- PyPI `grund-lsp`: five `py3-none-<platform>` wheels and an sdist;
- twelve archives: `grund-<version>-<target>` and `grund-lsp-<version>-<target>` for all six rows, `.tar.gz` except `.zip` on Windows, each beside its `.sha256`.

`scripts/distribution/candidate.py plan --version <version> --sha <sha>` prints that inventory, and the twenty-two payloads it is built from — six `grund`, six `grund-lsp`, five addons and five extensions — before anything is built. Nothing the plan does not name may appear in a candidate, and nothing it names may be missing.

### 2.2 npm selects one platform package by OS, CPU and libc

`grund-cli` and `grund-lsp` each list their five platform packages as `optionalDependencies` at exactly the candidate's version, never a range. Each platform package declares the `os`, `cpu` and, on Linux, `libc: ["glibc"]` of its row, so a package manager installs the one that matches and skips the rest. The umbrella's `grund.platformPackages` maps each Rust target to its platform package, which is what the binding's loader reads ([§FS-distribution.3.2.3.2](FS-distribution.md#3232-moduleruntimeload-interface)). The CLI and addon of one row share a platform package, so the `grund` command and `import "grund-cli"` can never come from two versions.

### 2.3 One assembly recipe consumes the binding's fragment

`crates/grund-node/package-api.json` is the only committed description of `grund-cli`'s API surface, and the assembly consumes it unchanged: every key it carries reaches the assembled `package.json` with the same value, except that `files` gains the launcher and `grund` gains `platformPackages`, and the recipe adds only `name`, `version`, `bin`, `optionalDependencies`, package metadata, and `os`/`cpu`/`libc` on platform packages. No second umbrella manifest is committed. `scripts/distribution/candidate.py npm-trees` writes every npm package's file tree without its compiled payloads, so the shape is checkable without a build.

### 2.4 Every package says what it holds

Each npm and PyPI package carries a README that names what that package installs — the CLI and its API, or the language server alone — which row a platform package serves, and links to the `grund` CLI and its installation guide; a platform package's README says it is installed by its umbrella and not directly. Every package carries the MIT `LICENSE`, and its metadata names this repository. A `grund-lsp` package names no `grund-cli` or `grund` dependency.

## 3. Launchers and payload placement

How an installed package reaches its payload, and what happens when it cannot.

### 3.1 Payloads sit where each registry runs them

- `@grund-cli/<suffix>`: `bin/grund` (`bin/grund.exe` on Windows) and `grund.node`, with an `index.cjs` exporting `addonPath`, `executablePath` and `metadata`;
- `@grund-lsp/<suffix>`: `bin/grund-lsp`, with an `index.cjs` exporting `executablePath` and `metadata`;
- `grund-cli` and `grund-lsp` map the commands `grund` and `grund-lsp` to their launchers;
- the `grund` wheel holds the executable as `grund-<version>.data/scripts/grund` and the extension as `grund/_native.abi3.so` (`.pyd` on Windows); the `grund-lsp` wheel holds `grund_lsp-<version>.data/scripts/grund-lsp`;
- an archive holds one top-level directory, `grund-<version>-<target>/` or `grund-lsp-<version>-<target>/`, with the executable, `LICENSE` and `README.md`.

A wheel installs the executable itself onto `PATH`, so `pip install`, `pipx install` and `uv tool install` run the payload with no Python launcher in front of it, and the binding still adds no console entrypoint ([§FS-distribution.3.3.7](FS-distribution.md#337-local-source-and-typing-handoff)). The placed bytes are the manifest's payload bytes, or a transformation the manifest records ([§FS-distribution-candidate.6.1](FS-distribution-candidate.md#61-one-commit-produces-one-manifest)).

### 3.2 The npm launcher is invisible

The npm `grund` and `grund-lsp` launchers start their payload with the same arguments, byte for byte, including spaces and non-ASCII text; the same working directory and environment; and the caller's standard input, output and error inherited rather than piped. The launcher's exit status is the payload's. A payload killed by a signal kills the launcher with the same signal on POSIX, and an `INT`, `TERM` or `HUP` the launcher receives reaches the payload. A closed output pipe ends the payload the way it ends a directly run `grund`, quietly and with the same status. The launcher prints nothing of its own while the payload runs. So every finding, every verdict and every byte of output is the payload's, whichever way it was installed ([§FS-distribution.2](FS-distribution.md#2-cli-parity)).

### 3.3 A missing payload is an error, never a substitute

When the launcher finds no platform package for the host, finds one whose version is not the umbrella's, or runs on a row the matrix does not list, it exits `2` and writes one line to standard error, beginning `error:`, that names the host platform and what it looked for and ends with the fix: install the matching platform package, or run `npm run build:source` in the installed package ([§FS-distribution-candidate.4.1](FS-distribution-candidate.md#41-npm-builds-from-source-only-when-asked)). It never runs another row's payload, never downloads one and never compiles one. A matching `native/grund` from a source build is preferred over the platform package, the same precedence the addon loader keeps ([§FS-distribution.3.2.3.2](FS-distribution.md#3232-moduleruntimeload-interface)).

### 3.4 The LSP's stdout stays the protocol's

`grund-lsp`, however installed, writes nothing to standard output that is not an LSP message, and its launcher adds nothing to either stream. A launch failure is the one exception, and it goes to standard error before any message is read ([§FS-lsp.2.2](FS-lsp.md#22-lifecycle)).

## 4. Source fallback

What a user on a row without a prebuilt payload can do, and what it needs.

### 4.1 npm builds from source only when asked

`npm run build:source` in an installed `grund-cli` builds the addon with the binding's own source builder ([§FS-distribution.3.2.3.3](FS-distribution.md#3233-source-buildlocal-package-ownership)) and the `grund` executable from the same bundled, locked sources, both into `native/`. In `grund-lsp` it builds only the server. Neither package runs a build on install: there is no `install` or `postinstall` script, so an install never compiles.

### 4.2 Python builds from the sdist only when asked

`pip install --no-binary=:all: grund` builds the extension and the `grund` executable from the sdist and installs both; `grund-lsp` builds only the server. The sdists carry everything the build reads: the frontend and core sources, assets, manifests and `Cargo.lock`, the build hooks, the licence and the metadata.

### 4.3 A source build needs Rust and says so

A source build needs Rust and Cargo, a linker and platform SDK, and the pinned host build tools, and builds with LTO but no profile. A missing prerequisite fails with a line naming it. A source build installs the same commands with the same behavior, slower ([§FS-distribution.4.9](FS-distribution.md#49-distributed-binaries-are-profile-guided-optimized)).

## 5. The rehearsal

The one run that proves a candidate on every row before anyone could publish it.

### 5.1 The rehearsal holds no credential

`.github/workflows/candidate-rehearsal.yml` runs only on `workflow_dispatch`. Its permissions are `contents: read`, no job asks for `id-token`, and nothing in it names a secret, creates a tag or a release, or uploads to a registry. npm installs go through a loopback registry the rehearsal starts and serves the candidate's own tarballs from; Cargo installs come from the candidate's `.crate` files; Python installs come from the candidate's wheel and sdist files with no index.

### 5.2 Every registry row installs fresh and without Rust

On every registry row and every promised runtime, the rehearsal installs into a fresh environment that has no Rust toolchain on `PATH` and no cached package: `grund-cli` with `npm install` and with `npx`, the `grund` wheel with `pip` into a fresh virtual environment and with `pipx`, and `grund-lsp` the same ways. Each install puts its commands on `PATH` and answers `--version` with the candidate's version. Every row installs the CLI without the LSP and the LSP without the CLI, and each works alone ([§FS-distribution.1.3](FS-distribution.md#13-the-cli-does-not-pull-in-the-lsp)). The crates install with `cargo install` from the candidate's `.crate` files.

### 5.3 Every install of the CLI answers alike

The rehearsal replays the read-only e2e cases against every installed `grund` — Cargo, npm, wheel and archive — with the tree, configuration, invocation and environment held fixed, and compares standard output, standard error and exit status byte for byte against the checked goldens. The cases cover a clean tree, findings, JSON, selectors, a failed ID query, both path bases, and a copy of a case under a directory whose name has spaces and non-ASCII text. A row passes only when every install matches every golden ([§FS-distribution.2](FS-distribution.md#2-cli-parity)).

### 5.4 Every installed API answers alike

The rehearsal runs the binding acceptance corpus against the installed packages rather than a checkout build: the Python acceptance modules against each interpreter's installed wheel, and the Node parity corpus against an installed `grund-cli`, both against the same-source oracle ([§FS-distribution.3.0.3](FS-distribution.md#303-complete-data-and-canonical-parity)). That includes nulls, suggestions, cautions, sites and authority.

### 5.5 Every installed language server holds its lifecycle

Each installed `grund-lsp` answers `initialize`, publishes the candidate fixture's `dangling` finding after `didOpen`, answers a hover, answers `shutdown`, and exits `0` on `exit`, with nothing on standard output but protocol messages.

### 5.6 A row runs on its own runner, and a skipped row is named

Every row's installed checks run on that row's runner. A local run of the rehearsal runs the host's row and names every other row it skips, with the runner that row needs; asking for a row the host is not fails rather than passing with nothing run, and a row in which no check ran fails. Each row that passes writes its receipt ([§FS-distribution-candidate.6.5](FS-distribution-candidate.md#65-receipts-record-what-each-runner-proved)).

## 6. The manifest

The record of a candidate that the publisher trusts and nothing else.

### 6.1 One commit produces one manifest

The candidate's `manifest.json` records: its schema version and scope; the package version and engine version; the commit SHA it was built from; the matrix rows; the Rust toolchain and the container image digests; every artifact the plan names, with its registry, package, version, file name and SHA-256; and every payload, with its product, target, build digest, optimization (`pgo` or `lto-exception`), profile and training digests, and each placement with the artifact, the path inside it, the digest there and the transformations between — repair, stripping — that the build applied. The manifest's own SHA-256 is the candidate's identity.

### 6.2 One version across every package

A full release carries one version in every crate, npm package and Python distribution, and one engine version in every payload. Python spells it in PEP 440, so a development version `X.Y.Z-dev` is `X.Y.Z.dev0` there and a release `X.Y.Z` is `X.Y.Z` everywhere. `scripts/distribution/candidate.py versions` reads every version source — the Cargo manifests, `Cargo.lock` and `pyproject.toml` — and fails when they disagree; `candidate.py set-version` writes all of them, and both bump helpers use it ([§FS-distribution.4.5](FS-distribution.md#45-what-the-version-bump-includes)).

### 6.3 A binding-only candidate never claims completeness

A candidate whose scope is `binding-only` records its own package version beside the unchanged engine and CLI version it carries. It never claims ecosystem completeness, and the publisher refuses any scope but `full-release`.

### 6.4 Verification rejects drift and corruption

`scripts/distribution/candidate.py verify <candidate> --release --sha <sha> --tag <tag>` reads the version inside each artifact — npm `package.json`, wheel `METADATA`, sdist `PKG-INFO`, the crate's `Cargo.toml` — and every payload digest at its placement, and fails, naming the artifact, on: a development version, two versions or two engine versions in one candidate, a SHA or tag that disagrees with the manifest, an artifact missing, unexpected or whose digest differs, an archive whose `.sha256` disagrees, a payload whose placed bytes differ from the recorded ones, an LTO exception anywhere but the Windows arm64 row, and a non-full-release scope.

### 6.5 Receipts record what each runner proved

Each row writes `receipts/<row>.json` beside the manifest, naming the manifest digest, the row, the runner and every check it passed. A candidate is ready to publish when every registry row has a receipt for the candidate's manifest digest; a receipt for another digest counts for nothing.

## 7. Profile-guided payloads

How each payload is optimized, and what proves the packaged one is the optimized one.

### 7.1 Each product trains on its own workload

The `grund` executable trains on the benchmark command list ([§FS-distribution.4.9](FS-distribution.md#49-distributed-binaries-are-profile-guided-optimized)); the language server on `initialize`, opening documents, publishing their findings, hover and `shutdown`; the Node addon and the Python extension on native API calls over the same benchmark operations. A product never ships with another product's profile.

### 7.2 Profiles are isolated by what changes code generation

A profile is keyed by compiler, target, source SHA, product, features and, for the extension, the stable ABI it targets. A build uses only the profile of its own key, and a missing or mismatched one fails the build.

### 7.3 The packaged payload is the profile-use build

The manifest records each payload's generate, train, merge and use steps and the digest of the profile-use build. The placed payload's digest is that build's, or that build's after a recorded transformation; an instrumented binary is never packaged.

### 7.4 Only an identified Windows arm64 training failure falls back

When the Windows arm64 row's training run produces no profile — the way hosted-runner PGO is broken there — that row may package a self-checked LTO build, recorded as `lto-exception` with the failure it met. A failure to build, merge or self-check is not that failure, and no failure on any other row or product is an exception: each fails the candidate ([§FS-distribution.4.9](FS-distribution.md#49-distributed-binaries-are-profile-guided-optimized)).

## 8. The publisher

What may one day upload a candidate, and what keeps it from doing so today.

### 8.1 Publication is disabled and gated

`.github/workflows/cross-registry-publish.yml` runs only on `workflow_dispatch`, with the inputs `candidate_sha`, `manifest_sha256` and `rehearsal_run_id`, and every job that could upload is disabled by a literal `if: ${{ false }}`. Its publish jobs use the `cross-registry-publish` environment, which a maintainer must approve. No merge, schedule, tag, workflow completion or rehearsal reaches it. Turning it on is a reviewed change of its own, made by an operator separately authorized to publish.

### 8.2 Identity is not authority

Publishing uses trusted publishing: the job exchanges its workflow identity token for a short-lived npm or PyPI token, and only publisher jobs hold `id-token: write`. A name the registry guard finds free or this project's ([§FS-distribution.1.1](FS-distribution.md#11-package-names)) is not thereby a name this workflow may publish: holding the npm scopes and the PyPI projects, and registering the trusted publishers and the environment, are prerequisites no part of this candidate meets. Every identity is exchanged before any upload, as the authority check, and a refused exchange stops the publisher before any upload, naming the package it was refused for. Each upload then uses a token exchanged immediately before it, from a fresh workflow identity token — one per npm package and one per Python distribution's set — because a token taken before a readiness wait can expire during it. A refusal at one of these later exchanges stops the run, which reports itself partial, naming the artifact it stopped at ([§FS-distribution-candidate.8.5](FS-distribution-candidate.md#85-reruns-resume-and-never-overwrite)).

### 8.3 The publisher uploads only verified artifacts

`scripts/distribution/publish.py` takes the candidate directory, the manifest digest and the SHA, and refuses unless `candidate.py verify --release` passes and every registry row's receipt names that digest. It uploads exactly the candidate's files and never runs a build tool; a candidate that would need one is refused, because a rebuilt artifact is not the one the rehearsal proved.

### 8.4 Order and readiness

The matching crates must already resolve on crates.io before the publisher uploads anything, waiting up to 90 attempts 20 seconds apart, because Cargo stays core-first and is published by the existing release ([§FS-distribution.4.10](FS-distribution.md#410-cratesio-publishes-in-dependency-order-and-artifacts-upload-last)). npm platform packages publish before the umbrellas that depend on them, and each must resolve at its exact version before any umbrella goes up; each Python distribution publishes as a complete set, sdist and every wheel. Every readiness wait is the same 90 × 20 seconds, and one that runs out fails the run, naming what never appeared.

### 8.5 Reruns resume and never overwrite

Before each upload the publisher asks the registry what it already holds at that version. An artifact already there with the manifest's digest is skipped; one there with another digest stops the run without uploading anything else. With `--status-out` a run writes the state of every npm and PyPI artifact — `published`, `skipped`, `pending` or `failed` — with the candidate's version and manifest digest, and a run that stops reports itself partial. A rerun with the same candidate resumes from there. A release is complete only when every artifact the plan names is published: the crates and archives by the existing release, the rest by this publisher ([§FS-distribution.4.11](FS-distribution.md#411-the-full-ecosystem-release)).

### 8.6 Checksums establish integrity, not identity

The manifest's digests and the archives' `.sha256` files say a file is the one the rehearsal proved. They do not say who built it: build provenance and the registries' own attestations are required before publication is turned on, and signing and notarization are deferred.

### 8.7 The Cargo release cannot reach the publisher

`release.yml`, `auto-bump.yml` and `release-minor.yml` keep publishing only the crates and the GitHub release. None of them names the rehearsal or the publisher workflow or runs an npm or PyPI publish, and neither new workflow is triggered by them.
