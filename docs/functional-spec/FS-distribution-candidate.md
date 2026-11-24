# FS-distribution-candidate: one candidate is assembled, rehearsed and verified before any registry sees it

The detail of [§FS-distribution.4.13](FS-distribution.md#413-a-candidate-is-rehearsed-before-anything-is-published): what the cross-registry release assembles from one commit, how a credential-free rehearsal installs and proves it on every advertised row, the manifest that records it, and the disabled, gated publisher, which uploads nothing but those bytes. Nothing here publishes: a candidate that passes everything below is ready to publish, not published ([§FS-distribution-candidate.8.1](FS-distribution-candidate.md#81-publication-is-disabled-and-gated)). Serves [§GOAL-multi-language](../goals.md#goal-multi-language-same-engine-three-platforms) and [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

## terms: Terms

Leans on [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, verdict).

- **candidate** — Every artifact one release would publish, assembled from one commit at one version, with the `manifest.json` that lists them. Never rebuilt once assembled.
- **registry row** — One of the five platform rows of [§FS-distribution-candidate.1.1](FS-distribution-candidate.md#11-six-rows-five-of-them-registry-rows) that npm and PyPI publish for.
- **payload** — One compiled binary a candidate carries: the `grund` executable, the `grund-lsp` executable, the Node addon or the Python extension, for one Rust target.
- **placement** — Where one payload sits inside one artifact, with the digest it has there.
- **platform package** — An npm package carrying one registry row's payloads, named `@grund-cli/<suffix>` or `@grund-lsp/<suffix>`, names that stay provisional until held ([§FS-distribution.1.1](FS-distribution.md#11-package-names)).
- **launcher** — The `grund` and `grund-lsp` executables the npm packages put on `PATH`, which only find their platform payload and run it.
- **rehearsal** — The manual, credential-free run that assembles a candidate, installs it fresh on every registry row and records what each row proved.
- **receipt** — The record one row's rehearsal writes of the checks it passed against one manifest digest.

## 1. The support matrix

### 1.1 Six rows, five of them registry rows

| Row | OS / payload floor | Rust target | npm suffix, both families | Wheel platform | Installed-artifact runner |
|---|---|---|---|---|---|
| `linux-x64-gnu` | Linux x64 / glibc 2.17 | `x86_64-unknown-linux-gnu` | `linux-x64-gnu` | `manylinux2014_x86_64` | `ubuntu-latest`, pinned manylinux container |
| `linux-arm64-gnu` | Linux arm64 / glibc 2.17 | `aarch64-unknown-linux-gnu` | `linux-arm64-gnu` | `manylinux2014_aarch64` | `ubuntu-24.04-arm`, pinned manylinux container |
| `darwin-x64` | macOS x64 / 11.0 | `x86_64-apple-darwin` | `darwin-x64` | `macosx_11_0_x86_64` | `macos-15-intel` |
| `darwin-arm64` | macOS arm64 / 11.0 | `aarch64-apple-darwin` | `darwin-arm64` | `macosx_11_0_arm64` | `macos-15` |
| `win32-x64-msvc` | Windows x64 / Windows 10, MSVC 14 runtime | `x86_64-pc-windows-msvc` | `win32-x64-msvc` | `win_amd64` | `windows-latest` |
| `win32-arm64-msvc` | Windows arm64 / Windows 11, MSVC 14 runtime | `aarch64-pc-windows-msvc` | deferred | deferred | `windows-11-arm`: downloadable binaries only |

The row ids are the npm suffixes of [§FS-distribution.3.2.3](FS-distribution.md#323-workers-loading-and-source-builds), and the table is the one place a script, a workflow and a test read the matrix from: `scripts/distribution/candidate.py matrix` prints it as JSON, one object per row, and nothing else spells a row by hand. Every row is built with Rust 1.95.0, and both Linux rows in the digest-pinned `manylinux2014` images of [§FS-distribution.4.8](FS-distribution.md#48-one-old-glibc-baseline-one-pinned-toolchain).

### 1.2 Every registry row proves every runtime it promises

Each registry row is installed and exercised on Node 22.x and 24.x and on CPython 3.10, 3.11, 3.12, 3.13 and 3.14. Node loads one N-API 8 addon per row; Python loads one `cp310-abi3` extension per row ([§FS-distribution.3.3.7](FS-distribution.md#337-interpreters-source-installs-and-typing)), carried by the row's one wheel, which the rehearsal installs into each of the five interpreters rather than trusting its tag.

### 1.3 Runtime floors are inspected, not assumed

A payload's floor is its row's in the table, and the host runtimes add their own: Node on Linux needs kernel 4.18 and glibc 2.28, Node 22 on macOS needs 11 and Node 24 needs 13.5. The rehearsal reads each payload's linkage and deployment metadata and refuses a payload above its row's floor. Linux payloads also run inside the old container, and npm installs run only on a host its Node supports.

### 1.4 Deferred targets gain no support

Windows arm64 has downloadable `grund` and `grund-lsp` archives and no npm or PyPI package. musl, other architectures, free-threaded CPython and PyPy are not promised. An install on any of them may try the source fallback ([§FS-distribution-candidate.4](FS-distribution-candidate.md#4-source-fallback)) and gains no support by succeeding.

## 2. The inventory

### 2.1 Thirty-nine artifacts from one version

A full-release candidate holds exactly:

- three crates: `grund-core`, `grund` and `grund-lsp`;
- npm `grund-cli` and `grund-lsp`, and one platform package per registry row for each, `@grund-cli/<suffix>` and `@grund-lsp/<suffix>` — twelve tarballs;
- PyPI `grund`: five `cp310-abi3-<platform>` wheels and an sdist;
- PyPI `grund-lsp`: five `py3-none-<platform>` wheels and an sdist;
- twelve archives: `grund-<version>-<target>` and `grund-lsp-<version>-<target>` for all six rows, `.tar.gz` except `.zip` on Windows, each beside its `.sha256`.

`scripts/distribution/candidate.py plan --version <version> --sha <sha>` prints that inventory, and the twenty-two payloads it is built from — six `grund`, six `grund-lsp`, five addons and five extensions — before anything is built.

### 2.2 npm selects one platform package by OS, CPU and libc

`grund-cli` and `grund-lsp` each list their five platform packages as `optionalDependencies` at exactly the candidate's version. Each platform package declares the `os`, `cpu` and, on Linux, `libc: ["glibc"]` of its row. The umbrella's `grund.platformPackages` maps each Rust target to its platform package, which is what the binding's loader reads ([§FS-distribution.3.2.3](FS-distribution.md#323-workers-loading-and-source-builds)). The CLI and addon of one row share a platform package, so the `grund` command and `import "grund-cli"` can never come from two versions.

### 2.3 One assembly recipe consumes the binding's fragment

`crates/grund-node/package-api.json` is the only committed description of `grund-cli`'s API surface, and the assembly consumes it unchanged: every key it carries reaches the assembled `package.json` with the same value, except that `files` gains the launcher and `grund` gains `platformPackages`, and the recipe adds only `name`, `version`, `bin`, `optionalDependencies`, package metadata, and `os`/`cpu`/`libc` on platform packages. `scripts/distribution/candidate.py npm-trees` writes every npm package's file tree without its compiled payloads, so the shape is checkable without a build.

### 2.4 Every package says what it holds

Each npm and PyPI package carries a README that names what that package installs — the CLI and its API, or the language server alone — which row a platform package serves, and links to the `grund` CLI and its installation guide; a platform package's README says it is installed by its umbrella and not directly. Every package carries the MIT `LICENSE`, and its metadata names this repository. A `grund-lsp` package names no `grund-cli` or `grund` dependency.

## 3. Launchers and payload placement

### 3.1 Payloads sit where each registry runs them

- `@grund-cli/<suffix>`: `bin/grund` (`bin/grund.exe` on Windows) and `grund.node`, with an `index.cjs` exporting `addonPath`, `executablePath` and `metadata`;
- `@grund-lsp/<suffix>`: `bin/grund-lsp`, with an `index.cjs` exporting `executablePath` and `metadata`;
- `grund-cli` and `grund-lsp` map the commands `grund` and `grund-lsp` to their launchers;
- the `grund` wheel holds the executable as `grund-<version>.data/scripts/grund` and the extension as `grund/_native.abi3.so` (`.pyd` on Windows); the `grund-lsp` wheel holds `grund_lsp-<version>.data/scripts/grund-lsp`;
- an archive holds one top-level directory, `grund-<version>-<target>/` or `grund-lsp-<version>-<target>/`, with the executable, `LICENSE` and `README.md`.

A wheel installs the executable itself onto `PATH`, so `pip install`, `pipx install` and `uv tool install` run the payload with no Python launcher in front of it ([§FS-distribution.3.3.7](FS-distribution.md#337-interpreters-source-installs-and-typing)).

### 3.2 The npm launcher is invisible

The npm `grund` and `grund-lsp` launchers start their payload with the same arguments, byte for byte; the same working directory and environment; and the caller's standard input, output and error inherited rather than piped. The launcher's exit status is the payload's. A payload killed by a signal kills the launcher with the same signal on POSIX, and an `INT`, `TERM` or `HUP` the launcher receives reaches the payload. The launcher prints nothing of its own while the payload runs, so every finding, every verdict and every byte of output is the payload's, whichever way it was installed ([§FS-distribution.2](FS-distribution.md#2-cli-parity)).

### 3.3 A missing payload is an error, never a substitute

When the launcher finds no platform package for the host, finds one whose version is not the umbrella's, or runs on a row the matrix does not list, it exits `2` and writes one line to standard error, beginning `error:`, that names the host platform and what it looked for and ends with the fix: install the matching platform package, or run `npm run build:source` in the installed package ([§FS-distribution-candidate.4.1](FS-distribution-candidate.md#41-npm-builds-from-source-only-when-asked)). It never runs another row's payload, never downloads one and never compiles one. A matching `native/grund` from a source build is preferred over the platform package, the same precedence the addon loader keeps ([§FS-distribution.3.2.3](FS-distribution.md#323-workers-loading-and-source-builds)).

### 3.4 The LSP's stdout stays the protocol's

`grund-lsp`, however installed, writes nothing to standard output that is not an LSP message, and its launcher adds nothing to either stream except a launch failure, which goes to standard error before any message is read ([§FS-lsp.2.2](FS-lsp.md#22-lifecycle)).

## 4. Source fallback

### 4.1 npm builds from source only when asked

`npm run build:source` in an installed `grund-cli` builds the addon with the binding's own source builder ([§FS-distribution.3.2.3](FS-distribution.md#323-workers-loading-and-source-builds)) and the `grund` executable from the same bundled, locked sources, both into `native/`. In `grund-lsp` it builds only the server. Neither package has an `install` or `postinstall` script, so an install never compiles.

### 4.2 Python builds from the sdist only when asked

`pip install --no-binary=:all: grund` builds the extension and the `grund` executable from the sdist and installs both; `grund-lsp` builds only the server. Each sdist carries everything its build reads ([§FS-distribution.3.3.7](FS-distribution.md#337-interpreters-source-installs-and-typing)).

### 4.3 A source build needs Rust and says so

A source build needs Rust and Cargo, a linker and platform SDK, and the pinned host build tools, and builds with LTO but no profile. A missing prerequisite fails with a line naming it. A source build installs the same commands with the same behavior, slower ([§FS-distribution.4.9](FS-distribution.md#49-distributed-binaries-are-profile-guided-optimized)).

## 5. The rehearsal

### 5.1 The rehearsal holds no credential

`.github/workflows/candidate-rehearsal.yml` runs only on `workflow_dispatch`. Its permissions are `contents: read`, no job asks for `id-token`, and nothing in it names a secret, creates a tag or a release, or uploads to a registry. npm installs go through a loopback registry the rehearsal starts and serves the candidate's own tarballs from; Cargo installs come from the candidate's `.crate` files; Python installs come from the candidate's wheel and sdist files with no index.

### 5.2 Every registry row installs fresh and without Rust

On every registry row and every promised runtime, the rehearsal installs into a fresh environment that has no Rust toolchain on `PATH` and no cached package: `grund-cli` with `npm install` and with `npx`, the `grund` wheel with `pip` into a fresh virtual environment and with `pipx`, and `grund-lsp` the same ways. Each install puts its commands on `PATH`, and each answers `--version` with `<command> <version>`, the version being what [§FS-distribution.4.1.1](FS-distribution.md#411-a-development-build-names-its-commit) makes of the manifest's version and source commit for that command. The rehearsal computes that line from the manifest, never from another install's answer, and holds the payloads, the archives and the source builds to it too. Every row installs the CLI without the LSP and the LSP without the CLI, and each works alone ([§FS-distribution.1.3](FS-distribution.md#13-the-cli-does-not-pull-in-the-lsp)).

The fresh `PATH` hides the Rust toolchain. In a directory holding `cargo`, `rustc` or `rustup`, nothing is found under those three names, under `clippy-driver` or `rls`, or under a name beginning `cargo-` or `rust`. A `grund` or `grund-lsp` there is not found either, since `cargo install` puts those commands exactly there and a host copy could otherwise answer for an install that failed to put its command on `PATH`. Every other command, beside them or not, is found as it was and runs as it did, the same one first. A command the rehearsal cannot keep fails the rehearsal, naming the command, its directory and why, because running on without it reads as a broken candidate.

No npm setting exported in the environment the rehearsal was started in reaches an install, so the verdict never depends on the shell of whoever ran it. Every variable whose name begins `npm_config_`, in any mix of case, is dropped, and so are `PREFIX` and `DESTDIR`, so npm reads the global `npmrc` of the installation `node` runs from. The only npm settings the fresh environment holds are the ones the rehearsal sets, each in one spelling and with the value it gave.

### 5.3 Every install of the CLI answers alike

The rehearsal replays the read-only e2e cases against every installed `grund` — Cargo, npm, wheel and archive — with the tree, configuration, invocation and environment held fixed, and compares standard output, standard error and exit status byte for byte against the checked goldens. A row passes only when every install matches every golden ([§FS-distribution.2](FS-distribution.md#2-cli-parity)).

### 5.4 Every installed API answers alike

The rehearsal runs the binding acceptance corpus against the installed packages rather than a checkout build: the Python acceptance modules against each interpreter's installed wheel, and the Node parity corpus against an installed `grund-cli`, both against the same-source oracle ([§FS-distribution.3.0.3](FS-distribution.md#303-complete-data-and-canonical-parity)).

### 5.5 Every installed language server holds its lifecycle

Each installed `grund-lsp` answers `initialize`, publishes the candidate fixture's `dangling` finding after `didOpen`, answers a hover, answers `shutdown`, and exits `0` on `exit`, with nothing on standard output but protocol messages.

### 5.6 A row runs on its own runner, and a skipped row is named

Every row's installed checks run on that row's runner. A local run of the rehearsal runs the host's row and names every other row it skips, with the runner that row needs; asking for a row the host is not fails rather than passing with nothing run, and a row in which no check ran fails. Each row that passes writes its receipt ([§FS-distribution-candidate.6.5](FS-distribution-candidate.md#65-receipts-record-what-each-runner-proved)).

## 6. The manifest

### 6.1 One commit produces one manifest

The candidate's `manifest.json` records: its schema version and scope; the package version and engine version; the commit SHA it was built from; the matrix rows; the Rust toolchain and the container image digests; every artifact the plan names, with its registry, package, version, file name and SHA-256; and every payload, with its product, target, build digest, optimization (`pgo` or `lto-exception`), profile and training digests, and each placement with the artifact, the path inside it, the digest there and the transformations between — repair, stripping — that the build applied. A row's candidate is built from a checkout whose HEAD is that commit and which has no uncommitted change to a tracked file; the builder refuses any other, and hands that commit to every build that cannot read git ([§FS-distribution.4.1.1](FS-distribution.md#411-a-development-build-names-its-commit)). A container build is passed `GRUND_BUILD_COMMIT` exactly when it is set, an empty value included, and runs as the caller's UID:GID on a rootful Docker daemon (a rootless one maps the caller already); its setup works under the macOS suite runner's Bash, 3.2 included, with unset-variable checking on. The manifest's own SHA-256 is the candidate's identity. Every `manifest.json` the tool writes — a row's, the assembled candidate's and each row's share — is UTF-8 with LF line endings on every runner, and its digest is taken over those bytes on disk.

### 6.2 One version across every package

A full release carries one version in every crate, npm package and Python distribution, and one engine version in every payload. Python spells it in PEP 440, so a development version `X.Y.Z-dev` is `X.Y.Z.dev0` there. `scripts/distribution/candidate.py versions` reads every version source — the Cargo manifests, `Cargo.lock` and `crates/grund-py/pyproject.toml` — and fails when they disagree; at a ref without `crates/grund-py/pyproject.toml` it reads the root `pyproject.toml` that ref carries, and a ref with neither has no Python version to check. `candidate.py set-version` writes all of them ([§FS-distribution.4.5](FS-distribution.md#45-what-the-version-bump-includes)).

### 6.3 A binding-only candidate never claims completeness

A candidate whose scope is `binding-only` records its own package version beside the engine and CLI version it carries unchanged. It never claims ecosystem completeness, and the publisher refuses any scope but `full-release`.

### 6.4 Verification rejects drift and corruption

`scripts/distribution/candidate.py verify <candidate> --release --sha <sha> --tag <tag>` reads the version inside each artifact — npm `package.json`, wheel `METADATA`, sdist `PKG-INFO`, the crate's `Cargo.toml` — and every payload digest at its placement, and fails, naming the artifact, on: a development version, two versions or two engine versions in one candidate, a SHA or tag that disagrees with the manifest, an artifact missing, unexpected or whose digest differs, an archive whose `.sha256` disagrees, a payload whose placed bytes differ from the recorded ones, an LTO exception anywhere but the Windows arm64 row, and a non-full-release scope.

### 6.5 Receipts record what each runner proved

Each row writes `receipts/<row>.json` beside the manifest, naming the manifest digest, the row, the runner and every check it passed. A candidate is ready to publish when every registry row has a receipt for the candidate's manifest digest; a receipt for another digest counts for nothing.

## 7. Profile-guided payloads

Decided in [§DA-pgo-release](../decisions/architectural/DA-pgo-release.md#da-pgo-release-distributed-binaries-are-pgo-built-trained-on-the-benchmark-workload).

### 7.1 Each product trains on its own workload

The `grund` executable trains on the benchmark command list ([§FS-distribution.4.9](FS-distribution.md#49-distributed-binaries-are-profile-guided-optimized)); the language server on `initialize`, opening documents, publishing their findings, hover and `shutdown`; the Node addon and the Python extension on native API calls over the same benchmark operations. A product never ships with another product's profile.

### 7.2 Profiles are isolated by what changes code generation

A profile is keyed by compiler, target, source SHA, product, features and, for the extension, the stable ABI it targets. A build uses only the profile of its own key, and a missing or mismatched one fails the build.

### 7.3 The packaged payload is the profile-use build

The manifest records each payload's generate, train, merge and use steps and the digest of the profile-use build. The placed payload's digest is that build's, or that build's after a recorded transformation; an instrumented binary is never packaged.

### 7.4 Only an identified Windows arm64 compiler crash falls back

When rustc crashes compiling the Windows arm64 row's instrumented build of a product — every crate Cargo could not compile is a `-Cprofile-generate` rustc process that exited `0xc0000005, STATUS_ACCESS_VIOLATION` — that row may package a self-checked LTO build of the product, recorded as `lto-exception` with the failure it met. Any other failure fails the candidate: a compile error, a build that fails another way, a training run that writes no profile, a failure to merge, to build with the profile or to self-check, and any failure on another row ([§FS-distribution.4.9](FS-distribution.md#49-distributed-binaries-are-profile-guided-optimized)).

## 8. The publisher

### 8.1 Publication is disabled and gated

`.github/workflows/cross-registry-publish.yml` runs only on `workflow_dispatch`, with the inputs `candidate_sha`, `manifest_sha256` and `rehearsal_run_id`, and every job that could upload is disabled by a literal `if: ${{ false }}`. Its publish jobs use the `cross-registry-publish` environment, which a maintainer must approve. No merge, schedule, tag, workflow completion or rehearsal reaches it. Turning it on is a reviewed change of its own, made by an operator separately authorized to publish.

### 8.2 Identity is not authority

Publishing uses trusted publishing: a publisher job, the only kind that holds `id-token: write`, exchanges its workflow identity token for a short-lived npm or PyPI token. A name the registry guard finds free or this project's ([§FS-distribution.1.1](FS-distribution.md#11-package-names)) is not thereby a name this workflow may publish: holding the npm scopes and the PyPI projects, and registering the trusted publishers and the environment, are prerequisites no part of this candidate meets. Every identity is exchanged before any upload, as the authority check, and a refused exchange stops the publisher there, naming the package it was refused for. Each upload then uses a token exchanged immediately before it from a fresh workflow identity token — one per npm package and one per Python distribution's set — since a token taken before a readiness wait can expire during it. A refusal at one of these later exchanges stops the run, which reports itself partial, naming the artifact it stopped at ([§FS-distribution-candidate.8.5](FS-distribution-candidate.md#85-reruns-resume-and-never-overwrite)).

### 8.3 The publisher uploads only verified artifacts

`scripts/distribution/publish.py` takes the candidate directory, the manifest digest and the SHA, and refuses unless `candidate.py verify --release` passes and every registry row's receipt names that digest. It uploads exactly the candidate's files and never runs a build tool; a candidate that would need one is refused, because a rebuilt artifact is not the one the rehearsal proved.

### 8.4 Order and readiness

The publisher uploads nothing until the matching crates, which the Cargo release publishes, resolve on crates.io ([§FS-distribution.4.10](FS-distribution.md#410-cratesio-publishes-in-dependency-order-and-artifacts-upload-last)). npm platform packages publish before the umbrellas that depend on them, and each must resolve at its exact version before any umbrella goes up; each Python distribution publishes as a complete set, sdist and every wheel. Every readiness wait asks up to 90 times, 20 seconds apart, and one that runs out fails the run, naming what never appeared.

### 8.5 Reruns resume and never overwrite

Before each upload the publisher asks the registry what it already holds at that version. An artifact already there with the manifest's digest is skipped; one there with another digest stops the run without uploading anything else. With `--status-out` a run writes the state of every npm and PyPI artifact — `published`, `skipped`, `pending` or `failed` — with the candidate's version and manifest digest, and a run that stops reports itself partial. A rerun with the same candidate resumes from there until the release is complete ([§FS-distribution.4.11](FS-distribution.md#411-the-full-ecosystem-release)).

A question to npm about a package's document, or to PyPI about a release, that gets no HTTP answer at all — the connection refused, reset or closed before any response — is asked up to three times in all, a moment apart, before the publisher refuses; an HTTP answer of any status, a 5xx included, is taken as given. The refusal never reads the registry as empty, and after its reason it names the last attempt's transport error, so a sighting says whether the connection was refused, reset or dropped. An upload or a token exchange is never repeated: a write that may have landed is settled by the next run's question about what the registry holds, and a readiness wait asks again by its own count ([§FS-distribution-candidate.8.4](FS-distribution-candidate.md#84-order-and-readiness)).

### 8.6 Checksums establish integrity, not identity

The manifest's digests and the archives' `.sha256` files say a file is the one the rehearsal proved, not who built it: build provenance and the registries' own attestations are required before publication is turned on, and no payload carries a publisher's code signature or a notarization.

### 8.7 The Cargo release cannot reach the publisher

`release.yml`, `auto-bump.yml` and `release-minor.yml` publish only the crates and the GitHub release. None of them names the rehearsal or the publisher workflow or runs an npm or PyPI publish, and neither candidate workflow is triggered by them.
