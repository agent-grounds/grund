# FS-distribution: grund distribution targets

`grund` is written in Rust; the target distribution is **all three** major language ecosystems — cargo, npm, and PyPI — with idiomatic API bindings on each. The check engine stays a single shared library; only the surfaces differ. Today the implemented release path publishes the Rust crates: `grund-core`, the Cargo CLI package `grund`, and the optional Cargo LSP package `grund-lsp`. The npm and PyPI bindings, including their future `grund-lsp` packages, are tracked in [§RM-distribution](../roadmap.md#rm-distribution-cargo--npm--pypi-from-one-engine). Serves [§GOAL-multi-language](../goals.md#goal-multi-language-same-engine-three-platforms) and [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

## 1. Targets

| Registry | Package name        | Status      | Contents                                                                  |
|----------|---------------------|-------------|---------------------------------------------------------------------------|
| cargo    | `grund-core`          | implemented | Shared engine library used by the CLI, LSP, and future bindings.            |
| cargo    | `grund`               | implemented | CLI crate depending on `grund-core`; installs the `grund` binary.              |
| cargo    | `grund-lsp`           | implemented | Optional LSP server crate ([§FS-lsp](FS-lsp.md#fs-lsp-grund-ships-an-optional-lsp-server)) depending on `grund-core`; installs the `grund-lsp` binary. |
| npm      | `grund-cli`           | planned     | Prebuilt CLI binary + thin Node API surface (via `napi-rs`).              |
| npm      | `grund-lsp`           | planned     | Optional LSP server binary, prebuilt per platform.                        |
| PyPI     | `grund`               | planned     | Prebuilt CLI wheel + Python API surface (via `PyO3` / `maturin`).         |
| PyPI     | `grund-lsp`           | planned     | Optional LSP server, distributed via wheel (`pipx install grund-lsp`).      |

The planned PyPI wheel installs the `grund` command and the import module `grund`, npm's CLI package is `grund-cli` because the unscoped `grund` is externally occupied, and every name is re-verified against the live registries before publish (§1.1). Support packages publish registry README content that links users to the `grund` CLI (§1.2), and the CLI install on each registry does not transitively pull in `grund-lsp` (§1.3).

### 1.1 Package names

On PyPI the planned CLI package is also `grund`, whose wheel installs the `grund` command and the import module `grund` (§3.3). On npm the planned CLI package is `grund-cli`, because the unscoped `grund` package is externally occupied; it still installs the `grund` command. [§DA-pypi-uses-grund-as-the-package-name](../decisions/architectural/DA-pypi-uses-grund-as-the-package-name.md#da-pypi-uses-grund-as-the-package-name-pypi-uses-grund-as-the-package-name) sets the final PyPI name and records why PyPI uses the bare name while npm keeps `grund-cli`; the tool itself was renamed from its pre-release working title `gnd` ([§DA-rename-to-grund](../decisions/architectural/DA-rename-to-grund.md#da-rename-to-grund-rename-gnd-to-grund-before-first-publish)). The release process re-verifies every one of these names — and the remaining unreserved npm/PyPI LSP slots — against the live registries before publish (§4.3).

### 1.2 Support packages point at the CLI

Support packages that are not the primary user-facing install, such as `grund-core`, publish registry README content that links users to the `grund` CLI and names sibling packages such as `grund-lsp` once they exist.

### 1.3 The CLI does not pull in the LSP

The CLI install on each registry does **not** transitively pull in `grund-lsp` — they are independent packages, per [§DA-lsp-optional](../decisions/architectural/DA-lsp-optional.md#da-lsp-optional-lsp-server-ships-as-a-separate-optional-binary). A user who only runs `grund check` in CI installs the CLI alone; a user who wants editor integration installs `grund-lsp` separately and configures their editor to launch it ([§FS-lsp.2](FS-lsp.md#2-installation-and-lifecycle)).

## 2. CLI parity

The `grund` binary behaves identically regardless of how it was installed: the same flags, the same exit codes, the same byte-for-byte report format ([§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)). Users on Linux, macOS, and Windows who run `grund check .` against the same repo get the same answer.

CLI reports use logical paths, relative to the base `relative_paths` selects ([§FS-config.3.6](FS-config.md#36-output--report-format)), with `/` as the separator, even on Windows. This applies to text reports, JSON fields, `sites`, ID-query e2e fixture lists, stub-link targets, and generated cross-reference URLs; native platform paths may appear only in launch-time errors about paths outside the scanned repo, where there is no repo-relative path to print. The CI build/test matrix is the proof for this contract: every normal e2e case must pass on Linux, macOS, and Windows.

## 3. API surfaces

Each binding exposes the same conceptual operations as the CLI subcommands, plus a programmatic check-and-iterate path so the engine can be embedded inside test runners and editor servers.

### 3.0 Language-neutral data shapes

Every binding returns the same data, only spelled idiomatically: a report and its findings (§3.0.1), and the options a `show` takes (§3.0.2). These fields are normative. The byte-for-byte JSON form the CLI emits under `--format=json`, and IDE/agent integrations consume, follows the same shape and is the cross-binding equivalence test for [§GOAL-multi-language](../goals.md#goal-multi-language-same-engine-three-platforms), tracked in [AR-goal-measurement.2](../architecture/AR-goal-measurement.md#2-goal-meters).

#### 3.0.1 Report and Finding

```
Report {
  errors:      [Finding]
  warnings:    [Finding]
  suggestions: [Finding]  // filled only under `check --suggestions` (FS-check.2.3)
}

Finding {
  severity: "error" | "warning"  // absent on a suggestion, whose JSON carries "channel": "suggestion"
                                 // in its place (FS-errors.5)
  code:     // a `check` finding — one code of the sorted supported catalog in FS-errors.5
            // — or, on a failed ID query (FS-show.3, rendered with this same shape on stderr,
            //   path/line null) — "not-found" | "missing-section" | "broken-stub" | "ambiguous"
            //                   | "ambiguous-section" | "invalid-id" | "query-failed"
  path:     string?        // relative to config root (FS-config.3.6); null for a run-level warning in the
                           // report or a failed ID query (FS-errors.5.2, FS-errors.5.2.3)
  line:     u32?           // 1-indexed; null for a file-level finding with no line (e.g. an unreadable file, FS-check.2)
  message:  string         // the human-readable text
  sites:    [{ path, line }]?  // null for a single-site diagnostic; a list naming every site for a multi-site
                               // finding (a duplicate declaration) or an ambiguous-ID / ambiguous-section
                               // query failure that names sites; null for the number-only shorthand's
                               // `ambiguous` refusal, which names candidates instead (FS-errors.5.2.1)
}
```

#### 3.0.2 ShowOpts

```
ShowOpts {
  section: string?    // dotted section path, e.g. "3.1.2"
  mode:    "lead" | "brief" | "toc" | "full"
                      // the same mutually exclusive show ladder as §FS-show.1;
                      // "lead" is the CLI's no-flag default
                      // default: "lead"
  format:  "text" | "md" | "json"
}
```

### 3.1 Rust (`grund-core` crate)

```rust
let report = grund_core::check(&path)?;
let body = grund_core::show("FS-check", ShowOpts::default())?;
```

`Report` and the underlying `Findings` are exposed as plain data structures so callers can iterate, filter, or render their own output. Every function on this surface returns data and writes to no stream, which is what lets one answer be rendered by a terminal, an editor and an embedder alike — the report's warning channel included, so a caution settled before any report exists still reaches a caller as a warning rather than as a line on its stderr ([§FS-check.4.7](FS-check.md#47-a-workspace-member-swallows-the-blocks-own-scan), [§FS-check.4.10](FS-check.md#410-include_root--false-leaves-the-blocks-own-files-unread)). The one exception, `main_entry()`, leaves in 0.15.0 (§3.1.1).

#### 3.1.1 `main_entry()` is the one exception, and it is leaving

`grund_core::main_entry()` is a process entry point — it parses argv, prints, and returns an exit code — kept since 0.4.1 for `grund-core = "0.4"` consumers that built a thin binary around the old core symbol, and deprecated since that release without ever naming the release it stops working in. It now takes the path of [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path), whose new form has shipped beside it since 0.4.1: `check`, `show` and `scan` for an embedder, and the `grund` CLI package for a process entry point. **0.14.0** is the release whose deprecation note and changelog entry name the removal, in the `is removed in <release>` clause of §4.2.2, and **0.15.0** is the release that removes the symbol together with the rest of the engine's renderer. A tree still carrying the note therefore cannot be cut as 0.15.0 or above (§4.2.2), which is the release guard reading the promise rather than a person remembering it. An embedder migrates by replacing that one call with either new form.

### 3.2 Node (`grund-cli` npm package)

```js
import { check, show } from 'grund-cli';

const report = await check('./repo');
const body = await show('FS-check', { mode: 'brief' });
```

The Node binding is built with `napi-rs`. Native binaries are prebuilt for the platforms covered by `napi-rs` (macOS arm64/x64, Linux x64/arm64, Windows x64). Source builds are supported as a fallback.

### 3.3 Python (`grund` PyPI package)

```python
from grund import check, show

report = check("./repo")
body = show("FS-check", mode="brief")
```

The Python binding is built with `PyO3` and packaged with `maturin`. Wheels are built for CPython 3.10+ across the platforms covered by `cibuildwheel`. The distribution package and import module are both named `grund` ([§DA-pypi-uses-grund-as-the-package-name](../decisions/architectural/DA-pypi-uses-grund-as-the-package-name.md#da-pypi-uses-grund-as-the-package-name-pypi-uses-grund-as-the-package-name)).

## 4. Release process

The implemented release workflow publishes the Cargo CLI today and builds downloadable PGO binaries for every supported desktop CI platform. `release.yml` publishes a commit that already carries its version (§4.3), and two helper workflows make that commit on a candidate they validate first (§4.4), with a version bump that covers the manifests, the lockfile, `--version` fixtures, ramp constants and the changelog rotation (§4.5). Pull-request CI requires `## Unreleased` to name the pull request (§4.6), and the changelog supplies the GitHub release notes (§4.7). Linux binaries build in digest-pinned `manylinux2014` containers and every job on one pinned toolchain (§4.8); distributed binaries are PGO-built, with an LTO-only fallback for a platform whose PGO training is broken (§4.9); crates publish to crates.io in dependency order before artifacts upload (§4.10). npm and PyPI join the same shape once their frontends exist (§4.11).

### 4.1 Between releases, main carries a dev version

A release leaves `main` holding the version it just published, so every build from `main` until the next release reports the tag it is already ahead of. Nothing then distinguishes a binary built from `main` from the released one, and a fix that is merged but not installed looks exactly like one that is installed. So the release advances `main` as its last act: after publishing `X.Y.Z` both helpers commit `X.Y.(Z+1)-dev`. The suffix is what makes `grund --version` say which side of the tag a build came from. A `-dev` manifest is never publishable — `release.yml` still verifies that the selected commit carries the exact version being released (§4.3), and the helpers set that clean version on their candidate branch — so the rule that a released version matches its tag is unchanged.

### 4.2 A release may not contradict the releases the tree's own messages name

A ramp is a promise written into a message: a warning names the release it becomes an error in, or names the release in which a scalar status will move ([§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)), and once an error ramp lands the error that replaced it names the release the change was made in. Both halves are claims about a version, and each is false at the wrong one. A pending warning shipped *at* its deadline breaks the promise it makes; a landed change shipped *below* the release its own message names is worse, because it puts a breaking change in a release whose version says there is none.

A unit test can hold only the pending half (§4.2.1), so the release guard reads the release each message names and refuses a version that contradicts one (§4.2.2), in a closed vocabulary (§4.2.3) whose scalar clause is the `refs` warning's (§4.2.4). Its refusal names every line that disagrees and the window of releases left (§4.2.5); it does not yet read `wording changes in <release>` (§4.2.6); and it runs on every publication path (§4.2.7).

#### 4.2.1 A test can hold only the pending half

A unit test can hold the pending half — the bump that reaches a deadline bumps the running version, and a test can read it — but nothing could hold an ordinary landed-message half, because the helpers bump the version on a candidate branch that is neither `main` nor a pull request (§4.4) and no test suite runs between that bump and the publish.

#### 4.2.2 The release guard reads the releases the tree names

The release path asks directly: the release guard, `scripts/check_release_ramps.py <version>`, reads the release each message names out of the tree's own message text — the Rust sources under `crates/`, and the checked `expected.stdout` and `expected.stderr` goldens under `tests/e2e/cases/` that pin the bytes a user sees — and refuses a version that contradicts one:

| clause | what the line claims | the version being cut |
|---|---|---|
| `becomes an error in <release>` | the change has not been made yet | must be **below** that release |
| `became an error in <release>` | the change has been made | must be **at or above** it |
| `was removed in <release>` | the change has been made | must be **at or above** it |
| `is removed in <release>` | a named removal has not been made yet | must be **below** that release |
| `stopped loading in <release>` | the change has been made | must be **at or above** it |
| `unchecked in <release>` | the change has been made | must be **at or above** it |
| `will exit <status> ... in <release>` | a scalar exit-status change has not been made yet | must be **below** that release |
| `wording changes in <release>` | the message wording has not changed yet | must be **below** that release |

#### 4.2.3 The vocabulary is closed

The vocabulary is closed on purpose: it is the wording the warnings and errors already use, so a ramp written in it is seen and a ramp written outside it names no release the release guard can read. It asks the general question rather than naming any one ramp, so a ramp that lands later is covered the day its message is written. `is removed in <release>` is the pending half of `was removed in <release>` — the clause a deprecation names its removal release with, where the two named-error clauses name a verdict's — and it is written in that spelling rather than "will be removed in" so the pending and landed halves of one removal read as the same claim in two tenses.

#### 4.2.4 The scalar clause is the `refs` warning's

The scalar clause matches the exact `refs` warning in [§FS-refs.4](FS-refs.md#4-exit-codes): its replacement diagnostics must remain the ordinary failed-query bytes, so they do not gain a historical release suffix. A version-gated contract test then owns the landed phase; at 0.15.0 it expects exit `1` and the warning's absence.

#### 4.2.5 The refusal names the window left

The refusal names every line that disagrees and the window of releases the tree may still be cut as — which can be empty, when a tree has landed one ramp and still promises another at the same release, and an empty window is itself the answer: nothing may be published until the rest of that release's ramps land.

#### 4.2.6 `wording changes in <release>` is not read yet

The release guard does not yet read `wording changes in <release>`; that clause is specified but not implemented today. The follow-up must add it to `scripts/check_release_ramps.py`'s `CLAUSES` tuple and add a deadline test for each wording constant, the narrowed-alias scope suffix, and the `agents-init` compatibility tail.

#### 4.2.7 Every publication path runs the release guard

`release.yml`'s verify job runs the release guard on the version it is about to publish, which is every publication path — a `vX.Y.Z` tag push, a manual dispatch, and the non-publishing dry run both bump helpers wait on before they touch `main`. `auto-bump.yml` and `release-minor.yml` run it again on the version they compute, beside their existing gates, so a bump that could not be published fails before it pushes a candidate branch rather than after.

### 4.3 `release.yml` publishes a commit that already carries its version

A `vX.Y.Z` tag triggers `.github/workflows/release.yml` directly. The same workflow can also be run manually from the release commit: the operator enters the version, crate publishing is enabled by default, and the workflow creates `vX.Y.Z` if that tag does not already exist. If the requested tag already exists, that tag becomes the release source ref after the workflow verifies the tagged Cargo package versions match the requested version; this lets a failed release be recovered from a newer workflow commit without moving the release tag. In either entry path, the workflow verifies the tag/version matches the versions of all three Cargo packages it publishes (§4.10), runs a fail-fast preflight on the crates.io token when crate publishing is enabled, re-runs `scripts/check-registry-names.sh` so claimed package names must still be available or owned by this project (§1.1), then builds and self-checks profile-guided-optimized binaries (§4.9) on six targets — `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`, and `aarch64-pc-windows-msvc`. `release.yml` does not bump versions: the selected commit already carries the version being released.

### 4.4 Two helper workflows bump the version on a validated candidate

Version bumping lives in separate helper workflows, and neither publishes directly. `.github/workflows/release-minor.yml` is a manual helper that starts from the latest `vX.Y.Z` tag, computes `vX.(Y+1).0`, verifies that `main` has commits since the previous tag and that CI is green on the current `main` tip, commits the workspace version bump (§4.5) to a temporary release-candidate branch, runs `release.yml` as a non-publishing dry run on that candidate, and only then fast-forwards `main` and dispatches `release.yml` for the real publish. `.github/workflows/auto-bump.yml` is the scheduled/manual patch helper with the same shape: when `main` has substantive non-doc/CI changes since the latest `vX.Y.Z` tag and CI is green on that exact tip, it computes `vX.Y.(Z+1)`, validates the version bump on a temporary release-candidate branch, and publishes the bump to `main` only after the dry run succeeds. Both helpers use the release push credential configured for branch-protection bypass.

### 4.5 What the version bump includes

In both helpers, "the version bump" includes:

- the Cargo manifests and the lockfile;
- any checked fixture whose expected output embeds `grund --version`;
- every **ramp constant** whose window is stated as a version in message text — a deprecation deadline is a promise about a release, and a bump is the only moment it can come due ([§DF-index-compatibility-ramp.2.3](../decisions/functional/DF-index-compatibility-ramp.md#23-both-findings-name-their-versions-and-a-test-keeps-the-names-honest) states the rule). Both ramps that rule was written for have since landed; the live ones are the absorbed-scan warning, the unlisted-workspace-block warning, the narrowed-alias scope suffix, and the `agents-init` compatibility tail, and the two workspace ramps each have a unit test that fails the bump which reaches their deadline rather than letting them pass silently (§4.2.1);
- the deterministic changelog rotation performed by `scripts/prepare_changelog_release.py`: the curated `## Unreleased` bullets become the new inline release section, the former inline latest release is archived under `docs/changelog/<version>.md`, and the older-release index gains the archive link.

The helper fails rather than inventing release notes when `## Unreleased` has no bullet entries, so the release candidate is already e2e-clean and changelog-clean before `release.yml` runs.

### 4.6 Pull-request CI keeps the changelog mappable to its pull requests

Pull-request CI protects the curated `## Unreleased` notes before release time: `.github/workflows/ci.yml` runs `scripts/check_changelog_pr_entry.py` on `pull_request` events, and that script requires `docs/changelog.md`'s `## Unreleased` body to mention the current pull request as `PR #N`, `pull request #N`, or a `/pull/N` URL. The local pre-push hook runs the same check once the current branch has a resolvable GitHub PR number. This keeps the release section mappable back to the PRs it contains; push CI and release-candidate branches skip the PR-number gate because they do not have a current pull request context.

### 4.7 The changelog is the source of release notes

`release.yml` treats the changelog as the source of GitHub release notes. Before creating or updating a GitHub release, it extracts the requested `vX.Y.Z` section from `docs/changelog.md` at the selected release ref and passes that body to `gh release create` or `gh release edit`; if the section is missing or empty, the release fails before artifacts are published to the GitHub release.

### 4.8 One old glibc baseline, one pinned toolchain

Both Linux binaries are built on GitHub inside a `manylinux2014_<arch>` container (pinned by digest, not by tag) so the release artifact targets an old glibc baseline instead of inheriting whatever glibc happens to ship on `ubuntu-latest`. Every job — host-runner or in-container — installs the same pinned Rust toolchain so the six binaries are produced by the same compiler version.

### 4.9 Distributed binaries are profile-guided-optimized

The distributed `grund` binaries are profile-guided-optimized: each platform build runs `scripts/pgo-build.sh`, which builds an instrumented binary, runs the [AR-benchmarks](../architecture/AR-benchmarks.md#ar-benchmarks-instruction-counting-benchmarks-for-the-hot-cli-commands) self-repo hot command list (the commands agents and CI invoke most) against `grund`'s own conformant tree to record a profile, then rebuilds against it. The rationale, and why the self-repo benchmark workload is also the PGO training corpus, is [§DA-pgo-release](../decisions/architectural/DA-pgo-release.md#da-pgo-release-distributed-binaries-are-pgo-built-trained-on-the-benchmark-workload). If a hosted runner's PGO training is platform-broken while the ordinary release build still self-checks cleanly, that platform may publish a self-checked LTO-only fallback binary instead of blocking the whole release. PGO is not part of development builds or push/PR CI; it is a release-packaging step, and an explicit benchmarking step when comparing the optimized release artifact. A `cargo install grund` from source is LTO-optimized but not PGO'd — `cargo install` runs no custom build step — and is byte-for-byte behavior-identical to the distributed binary; only its performance differs.

### 4.10 crates.io publishes in dependency order, and artifacts upload last

After every platform binary passes its self-check, the workflow publishes `grund-core`, `grund`, and `grund-lsp` to crates.io when crate publishing is enabled. The dependency order is fixed: `grund-core` publishes first, and any run that still needs to publish `grund` or `grund-lsp` waits up to 30 minutes for Cargo to resolve the matching `grund-core` version before publishing the dependent crate. GitHub release artifacts are uploaded only after the platform PGO builds pass and, when enabled, crates.io publishing succeeds.

### 4.11 The full-ecosystem release

The future full-ecosystem release keeps the same shape but adds the remaining packages after their frontends exist:

1. Build per-platform Node binaries and publish `grund-cli` and `grund-lsp` to npm.
2. Build per-platform Python wheels and publish `grund` and `grund-lsp` to PyPI.

All artifacts must succeed for a full ecosystem release to be considered complete. Versions across the CLI and the LSP move together within a release; each `grund-lsp` package pins or bundles the same `grund-core` version the matching CLI release ships, so a CLI/LSP version mismatch from the official packages is structurally avoided.

## 5. What we do not promise

- 100% identical APIs across languages. Each binding is idiomatic to its host (camelCase for Node, snake_case for Python, `Result<T,E>` for Rust). The *behavior* is identical; the surface fits each ecosystem.
- Stable ABI for the C-level FFI. Bindings link against the Rust core at compile time; we do not ship a separate C library.
