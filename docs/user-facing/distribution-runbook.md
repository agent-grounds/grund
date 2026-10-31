# Distribution runbook: the cross-registry candidate

How a maintainer builds, rehearses and checks the npm and PyPI candidate of one
commit, and what has to be true before anyone may publish it. The contract is
[§FS-distribution-candidate](../functional-spec/FS-distribution-candidate.md#fs-distribution-candidate-one-candidate-is-assembled-rehearsed-and-verified-before-any-registry-sees-it); this page is the order of operations.

**The cross-registry commands here never upload packages.** The npm and PyPI
packages are assembled and rehearsed, never uploaded: the publisher workflow is disabled, and the accounts
it would publish with do not exist yet. The Cargo crates and the GitHub release
are published by `release.yml` as before, and nothing in this lane reaches it
([§FS-distribution-candidate.8.7](../functional-spec/FS-distribution-candidate.md#87-the-cargo-release-cannot-reach-the-publisher)). For what a user can install today, see
[Installation](installation.md).

## Prepare a Cargo/GitHub release

The owner can deliberately prepare the first stable release
([§FS-distribution.4.4.2](../functional-spec/FS-distribution.md#442-deliberately-select-the-first-stable-release-from-the-dispatch-main-tip)):

```bash
gh workflow run release-minor.yml --ref main \
  -f version=1.0.0 -f expected_main_sha=FULL_DISPATCH_MAIN_SHA
```

Replace `FULL_DISPATCH_MAIN_SHA` with the full 40-hex main tip at dispatch,
which must match the dispatch SHA, checkout HEAD and fetched `origin/main`.
It is not #467's freeze candidate; comparing it with that evidence, kept in
the [1.0 freeze record](../releases/1.0-freeze.md), remains the owner's
responsibility. Omitting both inputs selects `next-minor`;
an optional `expected_main_sha` on that choice is checked too.

First-stable selection refuses another ref, missing or mismatched SHA,
disagreeing source versions, sources outside `0.x`, missing or unreachable
latest stable `0.x` history, an empty commit range, or any existing stable
tag at or above `1.0.0`. Unsupported versions also refuse; prerelease tags
do not count as stable history. The existing release guard still enforces
the shared floor and pending retirement promises. Both choices run exact-tip
CI, both candidates' CI and a non-publishing dry run before advancing main
and starting Cargo/GitHub publication. Dispatch remains the owner's release
authorization; merging this helper capability performs no release.

### While the merge queue lands pull requests

A helper no longer needs a quiet hour. Both `release-minor.yml` and
`auto-bump.yml` hand everything from their candidates on to
`scripts/release_candidates.py`, which works in rounds and follows `main`
([§FS-distribution.4.4.1.1](../functional-spec/FS-distribution.md#4411-a-moved-main-is-followed-not-refused)). Run 39962684811 of
`release-minor.yml` shows why: it was dispatched on `853e88959a`, its three
candidate runs passed an hour later, and it was refused because the queue had
landed #975 (`c5362e3936`) three minutes before they finished. Today that run
gives round 1 up as soon as it sees `c5362e3936`: it cancels the runs still
going, deletes `release-candidate/v0.20.0-39962684811-1-r1` and its `-dev`
twin, waits until `main` has held still for three minutes, rebuilds both
candidates on the new tip, and runs all three runs again on
`release-candidate/v0.20.0-39962684811-1-r2`. Only the commits the last round's
runs passed reach `main`, and the release section lists #975.

The rounds are bounded ([§FS-distribution.4.4.1.2](../functional-spec/FS-distribution.md#4412-six-rounds-within-five-hours)): at most six, in a job of 300
minutes, and a round starts only with 90 minutes left. A helper that runs out
of either publishes nothing and says which ran out; dispatch it again. It also
stops, before pushing anything, when the target tag appeared or a merged pull
request changed the version sources, and the release guard refuses a ramp a
merged pull request landed before any run starts.

A run dispatched with `expected_main_sha` follows no move
([§FS-distribution.4.4.2](../functional-spec/FS-distribution.md#442-deliberately-select-the-first-stable-release-from-the-dispatch-main-tip)): it cancels its runs, deletes its branches and names the new
tip. To release that tip, dispatch again with a fresh pin:

```bash
gh workflow run release-minor.yml --ref main \
  -f version=1.0.0 -f expected_main_sha=THE_TIP_THE_REFUSAL_NAMED
```

Only a failure leaves branches ([§FS-distribution.4.4.1.4](../functional-spec/FS-distribution.md#4414-only-a-failure-leaves-branches)): a candidate run
that did not pass, or a push to `main` git refused while `main` was still the
round's base, keeps that round's two branches for you to read, and `main` is
untouched. A round given up for a move, a helper out of rounds or time and a
pinned refusal delete theirs.

Once the release commit is on `main`, the helper dispatches `release.yml` on
the round's release-candidate branch with `expected_release_sha` set to that
commit ([§FS-distribution.4.4.1.3](../functional-spec/FS-distribution.md#4413-the-push-to-main-reads-main-back)); the publisher refuses any other source before it
builds, and removes the branch once it has published
([§FS-distribution.4.3](../functional-spec/FS-distribution.md#43-releaseyml-publishes-a-commit-that-already-carries-its-version)). If that dispatch fails, the helper says so and does not start
over: dispatch `release.yml` on the branch it names, with the version and the
`expected_release_sha` it prints.

## The candidate

Before merging a ramp change, check that the tree still permits some release:

```bash
python3 scripts/check_release_ramps.py
# To check another tree:
python3 scripts/check_release_ramps.py --root /path/to/checkout
```

The Python pull-request gate runs this version-free check on the actual
repository with every scanned ramp included ([§FS-distribution.4.2.6](../functional-spec/FS-distribution.md#426-every-publication-path-runs-the-release-guard)). It
refuses an empty release window with `no release at all` and the conflicting
landed/shared-floor and pending locations ([§FS-distribution.4.2.5](../functional-spec/FS-distribution.md#425-the-refusal-names-the-window-left)). Land or
revert the conflicting ramps together, or in an order that keeps a release
possible at every merge. A nonempty window passes even if it excludes the
next patch. This check chooses no release and evaluates no version-specific
deadline of the core-concerns plan.

Before preparing a release, check the version against the checked-out tree:

```bash
python3 scripts/check_release_ramps.py <version>
```

This run also reads the core-concerns plan as that release would record it
([§FS-distribution.4.2.2.1](../functional-spec/FS-distribution.md#4221-the-preflight-reads-the-plan-as-the-release-would-record-it)). It refuses the version while a `planned`
notice in the deprecation ledger, or a phase whose lead still opens
`Not shipped`, is due at or below it, and names the plan's file, the notice's
surface or the phase's number, and the release it is planned for. Record that
notice or phase as shipped, or move it to a later release, before cutting.

The guard reads the single shared minimum publishable release from
`scripts/release-floor.txt`, currently `0.19.0`, together with bounds named by
diagnostic messages ([§FS-distribution.release-floor](../functional-spec/FS-distribution.md#release-floor-one-shared-minimum-publishable-release)). Rewriting a development manifest
does not lower that floor. After this transition merges, a `0.18.x` patch can
only be cut from a branch made before it. Independent warning windows that
still promise a change at `0.19.0` can leave no publishable version: the floor
requires at least `0.19.0`, while those promises require less than `0.19.0`.
The pull-request check refuses that conflict before merging; every publication
path also runs the guard against its selected version ([§FS-distribution.4.2.6](../functional-spec/FS-distribution.md#426-every-publication-path-runs-the-release-guard)).

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

A payload that fails to build, train or self-check fails the candidate, naming
the row. The one exception is `win32-arm64-msvc`, where rustc crashes compiling
the instrumented build on hosted runners: that row packages a self-checked LTO
build instead and records the crash in the manifest ([§FS-distribution-candidate.7.4](../functional-spec/FS-distribution-candidate.md#74-only-an-identified-windows-arm64-compiler-crash-falls-back)).

## Rehearse

The rehearsal installs the candidate's own files into fresh environments with
no Rust on `PATH`, and replays the CLI, API and language-server corpora against
every install ([§FS-distribution-candidate.5.2](../functional-spec/FS-distribution-candidate.md#52-every-registry-row-installs-fresh-and-without-rust) through
[§FS-distribution-candidate.5.5](../functional-spec/FS-distribution-candidate.md#55-every-installed-language-server-holds-its-lifecycle)). Locally it runs the host's row and names the
others as skipped.

The API corpora compare every install with the same-source Rust oracle
([§FS-distribution-candidate.5.4](../functional-spec/FS-distribution-candidate.md#54-every-installed-api-answers-alike)), which the rehearsal does not build. Build it
from the commit first, as CI does. The Python corpus reads it from
`target/debug`, the Node one from `GRUND_BINDINGS_ORACLE`:

```bash
GRUND_BINDINGS_SOURCE_SHA="$(git rev-parse HEAD)" cargo build --locked -p grund-core \
  --example grund-binding-oracle --target-dir ~/oracle-target
mkdir -p target/debug
cp ~/oracle-target/debug/examples/grund-binding-oracle target/debug/
export GRUND_BINDINGS_ORACLE=~/oracle-target/debug/examples/grund-binding-oracle
```

Then rehearse:

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
   byte, under a manifest naming the candidate's digest), builds the oracle,
   and runs `run.py` against it, on Node 22 and then on Node 24, writing its
   receipt.
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
  its job's identity token is exchanged for a short-lived one per registry.
  Every exchange is made once before any upload, and a refused one stops the
  run there, naming the package. Each upload then exchanges a fresh token
  immediately before it, because a readiness wait can outlast one; a refusal at
  that point leaves the run partial, and a rerun resumes it.
- **The environment.** A `cross-registry-publish` environment with a required
  reviewer, so every run waits for a maintainer's approval.
- **Attestations.** The publishing job attests every npm and PyPI file with
  `actions/attest-build-provenance` before it uploads. Checksums say a file is
  the rehearsed one. The attestation binds the published files to the
  publishing run, which took them from the rehearsal run `rehearsal_run_id`
  names. It does not record which run built them. Signing and notarization are
  deferred.
- **The registries' own attestations.** npm provenance and PyPI's PEP 740
  attestations must be in place before publication is turned on. The publisher
  produces neither today: its npm upload carries no provenance statement and its
  PyPI upload no `attestations` field.
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
