# Distribution runbook: the cross-registry candidate

How a maintainer builds, rehearses and checks the npm and PyPI candidate of one
commit, and what has to be true before anyone may publish it. The contract is
[§FS-distribution-candidate](../functional-spec/FS-distribution-candidate.md#fs-distribution-candidate-one-candidate-is-assembled-rehearsed-and-verified-before-any-registry-sees-it); this page is the order of operations.

**Nothing here publishes.** The npm and PyPI packages are assembled and
rehearsed, never uploaded: the publisher workflow is disabled, and the accounts
it would publish with do not exist yet. The Cargo crates and the GitHub release
are published by `release.yml` as before, and nothing in this lane reaches it
([§FS-distribution-candidate.8.7](../functional-spec/FS-distribution-candidate.md#87-the-cargo-release-cannot-reach-the-publisher)). For what a user can install today, see
[Installation](installation.md).

## The candidate

A candidate is a directory built from one commit at one version
([§FS-distribution-candidate.6.1](../functional-spec/FS-distribution-candidate.md#61-one-commit-produces-one-manifest)):

```text
candidate/
  manifest.json         the commit, toolchain, images, every artifact and payload with its digest
  cargo/                grund-core, grund, grund-lsp .crate files
  npm/                  grund-cli, grund-lsp and their platform packages
  pypi/                 grund and grund-lsp wheels and sdists
  archives/             grund and grund-lsp archives, each beside its .sha256
  receipts/<row>.json   what each row's rehearsal proved
```

The SHA-256 of `manifest.json` is the candidate's identity: every receipt names
it, and the publisher is given it ([§FS-distribution-candidate.6.5](../functional-spec/FS-distribution-candidate.md#65-receipts-record-what-each-runner-proved)). What a full
candidate holds is fixed before anything is built:

```bash
python3 scripts/distribution/candidate.py matrix                          # the six rows, as JSON
python3 scripts/distribution/candidate.py plan --version "$(python3 scripts/distribution/candidate.py versions)" \
  --sha "$(git rev-parse HEAD)"
```

## Build one row

Each row is built on its own runner ([§FS-distribution-candidate.5.6](../functional-spec/FS-distribution-candidate.md#56-a-row-runs-on-its-own-runner-and-a-skipped-row-is-named)). On a
host that is a matrix row, from a clean checkout of the commit:

```bash
python3 scripts/distribution/candidate.py build --row linux-x64-gnu \
  --sha "$(git rev-parse HEAD)" --out ~/candidate --target-dir ~/candidate-target
python3 scripts/distribution/candidate.py verify ~/candidate --sha "$(git rev-parse HEAD)"
```

`build` refuses a `--sha` that is not the checkout's `HEAD`, version sources
that disagree ([§FS-distribution-candidate.6.2](../functional-spec/FS-distribution-candidate.md#62-one-version-across-every-package)), a host that is not the row, and
an output directory that is not empty. Every payload is built by
`scripts/pgo-build.sh --product …` and trained on its own workload
([§FS-distribution-candidate.7.1](../functional-spec/FS-distribution-candidate.md#71-each-product-trains-on-its-own-workload)); the Linux rows build inside the pinned
`manylinux2014` image, so Docker must be available. Rust 1.95.0 with the
`llvm-tools-preview` component, Node 22 or 24, and a Python of 3.10 or newer
for training must be on `PATH`.

Training that writes no profile fails the build, naming the row, on every row
but `win32-arm64-msvc`, which packages a self-checked LTO build instead and
records the exception in the manifest ([§FS-distribution-candidate.7.4](../functional-spec/FS-distribution-candidate.md#74-only-an-identified-windows-arm64-training-failure-falls-back)).

## Rehearse

The rehearsal installs the candidate's own files into fresh environments with
no Rust on `PATH`, and replays the CLI, API and language-server corpora against
every install ([§FS-distribution-candidate.5.2](../functional-spec/FS-distribution-candidate.md#52-every-registry-row-installs-fresh-and-without-rust) through
[§FS-distribution-candidate.5.5](../functional-spec/FS-distribution-candidate.md#55-every-installed-language-server-holds-its-lifecycle)). Locally it runs the host's row and names the
others as skipped:

```bash
GRUND_REHEARSAL_PYTHONS=/path/to/python3.10:/path/to/python3.11:/path/to/python3.12:/path/to/python3.13:/path/to/python3.14 \
  python3 tests/integration/rehearsal/run.py --row linux-x64-gnu --candidate ~/candidate \
  --receipt ~/candidate/receipts/linux-x64-gnu.json
```

It needs Node and npm, `pipx`, Docker on Linux, Rust for the Cargo and
source-build checks, and every one of CPython 3.10 through 3.14
([§FS-distribution-candidate.1.2](../functional-spec/FS-distribution-candidate.md#12-every-registry-row-proves-every-runtime-it-promises)); a missing prerequisite fails with its name.
Without `--candidate` it builds one first. A receipt is written only when every
check ran and passed.

### On every row: `candidate-rehearsal.yml`

`.github/workflows/candidate-rehearsal.yml` runs the same thing on every row's
own runner. Only a person starts it, from the Actions tab or with
`gh workflow run candidate-rehearsal.yml --ref <branch>`; it can read the
repository and nothing else ([§FS-distribution-candidate.5.1](../functional-spec/FS-distribution-candidate.md#51-the-rehearsal-holds-no-credential)). Its jobs:

1. `build` — one row's candidate on each of the six runners, verified.
2. `assemble` — `candidate.py assemble` merges the six into one candidate; with
   every row present it is a `full-release`.
3. `rehearse` — each row takes its share of that candidate with
   `candidate.py share` (its own artifacts and the row-independent ones, byte for
   byte, under a manifest naming the candidate's digest) and runs `run.py`
   against it, on Node 22 and then on Node 24, writing its receipt.
   `win32-arm64-msvc` has no registry packages, so it rehearses only its
   inventory and provenance and writes no receipt.
4. `receipts` — `candidate.py receipts` accepts a row's receipt only when that
   row's share is exactly the one `share` derives from this candidate, and then
   records it against the candidate's own digest. The result is uploaded as the
   `rehearsed-candidate` artifact.

A row's receipt is the proof that its runner installed exactly these bytes. A
receipt for another digest counts for nothing ([§FS-distribution-candidate.6.5](../functional-spec/FS-distribution-candidate.md#65-receipts-record-what-each-runner-proved)).

## Verify

`verify` holds a candidate to its plan and its manifest
([§FS-distribution-candidate.6.4](../functional-spec/FS-distribution-candidate.md#64-verification-rejects-drift-and-corruption)): every planned file present and nothing else,
every digest matching, every version read from inside the artifact, every
placement holding the payload's own bytes, and every payload's profile key and
steps recorded.

```bash
python3 scripts/distribution/candidate.py verify <candidate> --sha <sha>                    # a share or a rehearsal
python3 scripts/distribution/candidate.py verify <candidate> --release --sha <sha> --tag v<version>
```

`--release` additionally requires all six rows, `full-release` scope and a
release version, not a `-dev` one.

## The publisher is disabled

`.github/workflows/cross-registry-publish.yml` is the only path to npm and PyPI,
and its publishing job carries a literal `if: ${{ false }}`
([§FS-distribution-candidate.8.1](../functional-spec/FS-distribution-candidate.md#81-publication-is-disabled-and-gated)). Its `check` job downloads the
`rehearsed-candidate` artifact of the named rehearsal run and verifies it as a
release; nothing else runs. Turning publication on is a reviewed change of its
own, made by an operator separately authorized to publish.

### Prerequisites nothing here meets yet

Before that change may be made ([§FS-distribution-candidate.8.2](../functional-spec/FS-distribution-candidate.md#82-identity-is-not-authority),
[§FS-distribution-candidate.8.6](../functional-spec/FS-distribution-candidate.md#86-checksums-establish-integrity-not-identity)):

- **Names and accounts.** The npm scopes `@grund-cli` and `@grund-lsp`, the npm
  packages `grund-cli` and `grund-lsp`, and the PyPI projects `grund` and
  `grund-lsp`, held by this project. Every one of these names is provisional
  until held. `scripts/check-registry-names.sh` says whether each is free or
  already this project's; a name that passes is still not one this workflow
  may publish to.
- **Trusted publishers.** On npm and on PyPI, a trusted publisher for each
  package naming this repository, `cross-registry-publish.yml` and the
  `cross-registry-publish` environment. The workflow holds no stored token;
  its job's identity token is exchanged for a short-lived one per registry, and
  a refused exchange stops the run before any upload, naming the package.
- **The environment.** A `cross-registry-publish` environment with a required
  reviewer, so every run waits for a maintainer's approval.
- **Attestations.** The publishing job attests the build provenance of every
  npm and PyPI file with `actions/attest-build-provenance` before it uploads.
  Checksums say a file is the rehearsed one; the attestation says which
  workflow run built it. Signing and notarization are deferred.
- **The crates.** The matching `grund-core`, `grund` and `grund-lsp` must
  already resolve on crates.io, published by `release.yml`; the publisher waits
  for them and publishes no crate ([§FS-distribution-candidate.8.4](../functional-spec/FS-distribution-candidate.md#84-order-and-readiness)).

### Starting a publication, once enabled

The operator names the commit, the manifest digest and the rehearsal run:

```bash
gh workflow run cross-registry-publish.yml \
  -f candidate_sha=<sha> -f manifest_sha256=<manifest digest> -f rehearsal_run_id=<run id>
```

`scripts/distribution/publish.py` then refuses unless `verify --release` passes
and every registry row's receipt names that digest
([§FS-distribution-candidate.8.3](../functional-spec/FS-distribution-candidate.md#83-the-publisher-uploads-only-verified-artifacts)). It publishes the npm platform packages first,
waits for each to resolve at its exact version, then the umbrellas; each PyPI
distribution goes up as a complete set, sdist and every wheel. Every readiness
wait is 90 attempts 20 seconds apart.

### Recovering from a partial upload

A run that stops part way leaves the registries holding some of the candidate.
`--status-out` writes every npm and PyPI artifact's state — `published`,
`skipped`, `failed` or `pending` — to `publish-status.json`, which the workflow
uploads even when the run fails. Rerun the workflow with the **same** three
inputs ([§FS-distribution-candidate.8.5](../functional-spec/FS-distribution-candidate.md#85-reruns-resume-and-never-overwrite)): before uploading anything, the
publisher asks each registry what it already holds at that version, skips every
file it holds with the same digest, and stops on any file it holds with other
bytes, because nothing is ever overwritten. A file published with the wrong
bytes cannot be fixed by a rerun; it needs a new version.
