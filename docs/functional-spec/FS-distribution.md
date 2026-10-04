# FS-distribution: grund distribution targets

`grund` is written in Rust; the target distribution is **all three** major language ecosystems — cargo, npm, and PyPI — with idiomatic API bindings on each. The check engine stays a single shared library; only the surfaces differ. Today the implemented release path publishes the Rust crates: `grund-core`, the Cargo CLI package `grund`, and the optional Cargo LSP package `grund-lsp`. The npm and PyPI bindings, including their future `grund-lsp` packages, are tracked in [§RM-distribution](../roadmap.md#rm-distribution-cargo--npm--pypi-from-one-engine). Serves [§GOAL-multi-language](../goals.md#goal-multi-language-same-engine-three-platforms) and [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, body, section, lead), [§FS-terms.terms.2](FS-terms.md#terms2-citations)
(shorthand), [§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (stub), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope, config root, workspace,
alias), and [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, severity, suggestion, caution, verdict).

- **compatibility notice** — The one line a release publishes about a change that a pull
  request's title cannot carry: a verdict the release moves, in the words of the decision
  record that moves it, written as that record's `release-note` section.

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

The planned PyPI wheel installs the `grund` command and the import module `grund`, npm's CLI package is `grund-cli` because the unscoped `grund` is externally occupied, and every name is re-verified against the live registries before publish ([§FS-distribution.1.1](FS-distribution.md#11-package-names)). Support packages publish registry README content that links users to the `grund` CLI ([§FS-distribution.1.2](FS-distribution.md#12-support-packages-point-at-the-cli)), and the CLI install on each registry does not transitively pull in `grund-lsp` ([§FS-distribution.1.3](FS-distribution.md#13-the-cli-does-not-pull-in-the-lsp)).

### 1.1 Package names

On PyPI the planned CLI package is also `grund`, whose wheel installs the `grund` command and the import module `grund` ([§FS-distribution.3.3](FS-distribution.md#33-python-grund-pypi-package)). On npm the planned CLI package is `grund-cli`, because the unscoped `grund` package is externally occupied; it still installs the `grund` command. [§DA-pypi-uses-grund-as-the-package-name](../decisions/architectural/DA-pypi-uses-grund-as-the-package-name.md#da-pypi-uses-grund-as-the-package-name-pypi-uses-grund-as-the-package-name) sets the final PyPI name and records why PyPI uses the bare name while npm keeps `grund-cli`; the tool itself was renamed from its pre-release working title `gnd` ([§DA-rename-to-grund](../decisions/architectural/DA-rename-to-grund.md#da-rename-to-grund-rename-gnd-to-grund-before-first-publish)). The release process re-verifies every one of these names — and the remaining unreserved npm/PyPI LSP slots — against the live registries before publish ([§FS-distribution.4.3](FS-distribution.md#43-releaseyml-publishes-a-commit-that-already-carries-its-version)). A name that no registry answers for is free, and the package's own endpoint is the only thing that can say so: a `404` there passes, while a failure to query that endpoint is a query failure and never an availability. Once it has answered that the package exists, nothing later turns the name back into a free one. What makes an existing package *this project's* is then registry-specific. On crates.io ownership is read from the registry's own owner record for the name ([§FS-distribution.1.1.1](FS-distribution.md#111-cratesio-ownership-is-the-registrys-owner-record)), never from metadata the package declares about itself, and ownership that cannot be established stops the release rather than being worked around ([§FS-distribution.1.1.2](FS-distribution.md#112-unproven-cratesio-ownership-stops-the-release)). On npm and PyPI a claimed name is this project's when the registry's own metadata for it names this repository; because that metadata is written by the last publish, it names the repository the package was published *from*, and a repository move reaches those registries only with the next release — the one this guard stands in front of — so there the former repository is accepted alongside the current one, or the move would lock the project out of its own names.

#### 1.1.1 crates.io ownership is the registry's owner record

For the three crates.io names — `grund-core`, `grund`, and `grund-lsp` — an existing package is this project's only when the registry's owner endpoint for that name, `/api/v1/crates/<name>/owners`, returns a `users` entry whose `kind` is exactly `user` and whose `login` is exactly `vjovanov`. That login is this project's crates.io identity, and it is carried as a named constant rather than spelled into a pattern, because it is a credential and not a shape. Nothing else in the response establishes ownership: the mutable display `name`, the URL text, the opaque numeric id and the `github_username_matches` flag are publisher-supplied or meaningless out of context; a bare organization string such as `agent-grounds` is neither a user record nor Cargo's `github:org:team` team form; and team records establish nothing here, because no team is part of the identity this project declares — admitting one is an ownership-policy change of its own, not a widening that happens by accident. The package's own `repository` metadata never participates in the decision. It is written by the last publish, so after a repository move it can only be corrected *through* the publish this guard stands in front of; and it is publisher-supplied, so any package at all may copy this repository's URL into it.

#### 1.1.2 Unproven crates.io ownership stops the release

A crates.io name that exists and is not proven this project's fails the release, and the message says which of four things went wrong, because the operator's next move differs in each. An owner record that is well-formed but carries no trusted user — an empty owner set, or only untrusted ones — reports `error: crates.io/<name> is already taken without trusted owner vjovanov` and names the owner endpoint. Owner data that is missing, malformed or the wrong shape reports that the owner evidence could not be read. An owner endpoint that answers `404`, or any other non-`200`, reports that ownership could not be determined and names the status. A request that never completes reports the transport failure rather than falling through as an unexplained non-zero exit. Each of the four exits non-zero, each is distinguishable from the other three, and each names the owner endpoint it asked. None of them falls back to the package's declared metadata: that fallback is what both the move deadlock and the copied-URL acceptance are made of, and ownership decided from unauthoritative data is not decided at all.

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

Every binding returns the same data, only spelled idiomatically: a report and its findings ([§FS-distribution.3.0.1](FS-distribution.md#301-report-and-finding)), and the options a `show` takes ([§FS-distribution.3.0.2](FS-distribution.md#302-showopts)). These fields are normative. The byte-for-byte JSON form the CLI emits under `--format=json`, and IDE/agent integrations consume, follows the same shape and is the cross-binding equivalence test for [§GOAL-multi-language](../goals.md#goal-multi-language-same-engine-three-platforms), tracked in [AR-goal-measurement.2](../architecture/AR-goal-measurement.md#2-goal-meters).

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
  sites:    [{ path, line }]?  // null for a single-site finding; a list naming every site for a multi-site
                               // finding (a duplicate declaration) or an ambiguous-ID / ambiguous-section
                               // query failure that names sites; null for the number-only shorthand's
                               // `ambiguous` refusal, which names candidates instead (FS-errors.5.2.1)
  authority: [string]?         // last key; the bytewise-sorted rule origins that authored this finding
                               // (FS-rules.7.6) — a declared rule's ID, or "--rule" for a `check --rule`
                               // trial sentence, or both where they reached the same meaning (FS-rules.6);
                               // null for a finding no rule authored, and always null on a failed ID query
                               // and a run-level warning (FS-errors.5.2, FS-errors.5.2.3)
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

`Report` and the underlying `Findings` are exposed as plain data structures so callers can iterate, filter, or render their own output. Every function on this surface returns data and writes to no stream, which is what lets one answer be rendered by a terminal, an editor and an embedder alike — the report's warning channel included, so a caution settled before any report exists still reaches a caller as a warning rather than as a line on its stderr ([§FS-check.4.7](FS-check.md#47-a-workspace-member-swallows-the-blocks-own-scan), [§FS-check.4.10](FS-check.md#410-include_root--false-leaves-the-blocks-own-files-unread)). Process callers use the `grund` CLI package; embedders call the data-returning `check`, `show`, `scan`, and related APIs ([§FS-distribution.3.1.1](FS-distribution.md#311-main_entry-is-absent-from-the-embedding-api)).

#### 3.1.1 `main_entry()` is absent from the embedding API

`grund_core::main_entry()` is not exported in 0.15.0 or later. It was a process entry point — it parsed argv, printed, and returned an exit code — kept for `grund-core = "0.4"` consumers until 0.14.0 shipped a deprecation note naming its removal in 0.15.0, the sequence [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) requires. A former caller uses `check`, `show`, `scan`, or another data-returning engine API when embedding grund, and invokes the `grund` CLI package when it needs a process entry point. Removing the engine adapter changes none of the shipped CLI's commands, flags, rendered bytes, or exit decisions, and changes no LSP behavior.

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

The implemented release workflow publishes the Cargo CLI today and builds downloadable PGO binaries for every supported desktop CI platform. `release.yml` publishes a commit that already carries its version ([§FS-distribution.4.3](FS-distribution.md#43-releaseyml-publishes-a-commit-that-already-carries-its-version)), and two helper workflows make that commit on a candidate they validate first ([§FS-distribution.4.4](FS-distribution.md#44-two-helper-workflows-bump-the-version-on-a-validated-candidate)), with a version bump that covers the manifests, the lockfile, `--version` fixtures, ramp constants and the changelog rotation ([§FS-distribution.4.5](FS-distribution.md#45-what-the-version-bump-includes)). The release writes its own changelog section, listing the pull requests merged since the previous tag and the compatibility notices the decision records added since it ([§FS-distribution.4.6](FS-distribution.md#46-the-release-lists-the-pull-requests-merged-since-the-previous-tag)), so no change waits in the tree for it ([§FS-distribution.4.12](FS-distribution.md#412-pending-changelog-entries-are-one-file-each)); the bump writes that section inline ([§FS-distribution.4.5](FS-distribution.md#45-what-the-version-bump-includes)), and the changelog supplies the GitHub release notes ([§FS-distribution.4.7](FS-distribution.md#47-the-changelog-is-the-source-of-release-notes)). Linux binaries build in digest-pinned `manylinux2014` containers and every job on one pinned toolchain ([§FS-distribution.4.8](FS-distribution.md#48-one-old-glibc-baseline-one-pinned-toolchain)); distributed binaries are PGO-built, with an LTO-only fallback for a platform whose PGO training is broken ([§FS-distribution.4.9](FS-distribution.md#49-distributed-binaries-are-profile-guided-optimized)); crates publish to crates.io in dependency order before artifacts upload ([§FS-distribution.4.10](FS-distribution.md#410-cratesio-publishes-in-dependency-order-and-artifacts-upload-last)). npm and PyPI join the same shape once their frontends exist ([§FS-distribution.4.11](FS-distribution.md#411-the-full-ecosystem-release)).

### 4.1 Between releases, main carries a dev version

A release leaves `main` holding the version it just published, so every build from `main` until the next release reports the tag it is already ahead of. Nothing then distinguishes a binary built from `main` from the released one, and a fix that is merged but not installed looks exactly like one that is installed. So the release advances `main` as its last act: after publishing `X.Y.Z` both helpers commit `X.Y.(Z+1)-dev`. The suffix is what makes `grund --version` say which side of the tag a build came from. A `-dev` manifest is never publishable — `release.yml` still verifies that the selected commit carries the exact version being released ([§FS-distribution.4.3](FS-distribution.md#43-releaseyml-publishes-a-commit-that-already-carries-its-version)), and the helpers set that clean version on their candidate branch — so the rule that a released version matches its tag is unchanged.

### 4.2 A release may not contradict the releases the tree's own messages name

A ramp is a promise written into a message: a warning names the release it becomes an error in, or names the release in which a scalar status will move ([§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)), and once an error ramp lands the error that replaced it names the release the change was made in. Both halves are claims about a version, and each is false at the wrong one. A pending warning shipped *at* its deadline breaks the promise it makes; a landed change shipped *below* the release its own message names is worse, because it puts a breaking change in a release whose version says there is none.

A unit test can hold only the pending half ([§FS-distribution.4.2.1](FS-distribution.md#421-a-test-can-hold-only-the-pending-half)), so the release guard reads the release each message names and refuses a version that contradicts one ([§FS-distribution.4.2.2](FS-distribution.md#422-the-release-guard-reads-the-releases-the-tree-names)), in a closed vocabulary ([§FS-distribution.4.2.3](FS-distribution.md#423-the-vocabulary-is-closed)) whose scalar clause is the `refs` warning's ([§FS-distribution.4.2.4](FS-distribution.md#424-the-scalar-clause-is-the-refs-warnings)). Its refusal names every line that disagrees and the window of releases left ([§FS-distribution.4.2.5](FS-distribution.md#425-the-refusal-names-the-window-left)), and it runs on every publication path ([§FS-distribution.4.2.6](FS-distribution.md#426-every-publication-path-runs-the-release-guard)).

#### 4.2.1 A test can hold only the pending half

A unit test can hold the pending half — the bump that reaches a deadline bumps the running version, and a test can read it — but nothing could hold an ordinary landed-message half, because the helpers bump the version on a candidate branch that is neither `main` nor a pull request ([§FS-distribution.4.4](FS-distribution.md#44-two-helper-workflows-bump-the-version-on-a-validated-candidate)) and no test suite runs between that bump and the publish.

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
| `an error in <release>` | the change has been made | must be **at or above** it |
| `will exit <status> ... in <release>` | a scalar exit-status change has not been made yet | must be **below** that release |
| `wording changes in <release>` | the message wording has not changed yet | must be **below** that release |

#### 4.2.3 The vocabulary is closed

The vocabulary is closed on purpose: it is the wording the warnings and errors already use, so a ramp written in it is seen and a ramp written outside it names no release the release guard can read. It asks the general question rather than naming any one ramp, so a ramp that lands later is covered the day its message is written. `is removed in <release>` is the pending half of `was removed in <release>` — the clause a deprecation names its removal release with, where the two named-error clauses name a verdict's — and it is written in that spelling rather than "will be removed in" so the pending and landed halves of one removal read as the same claim in two tenses.

##### 4.2.3.1 The landed half of an attribution pair is read where it opens a clause

A verdict that moved names both of its releases in one message: the rules that carry an attribution pair print `unchecked in grund <prior>, an error in <release>`, so the same line says when the finding went unchecked and when it became an error. Both halves are claims, and the landed one is the bare `an error in <release>` — which is why the vocabulary carries that clause and not only the two it is a substring of.

Being a substring is what gives it its left boundary. `becomes an error in <release>` and `became an error in <release>` both contain it, so the bare clause is read only where it **opens a clause** — at the start of a line, or after a comma, semicolon or colon — and never where a word stands in front of it. A tense therefore keeps the direction it spells rather than also yielding a bare landed claim, and a tense the vocabulary does not carry, such as the infinitive `become an error in <release>` of a promise not yet made, is not read as landed. The boundary enumerates no tenses, so a tense added to a message later cannot silently change what the guard reads.

#### 4.2.4 The scalar clause is the `refs` warning's

The scalar clause matches the exact `refs` warning in [§FS-refs.4](FS-refs.md#4-exit-codes): its replacement findings must remain the ordinary failed-query bytes, so they do not gain a historical release suffix. A version-gated contract test then owns the landed phase; at 0.16.0 it expects exit `1` and the warning's absence.

#### 4.2.5 The refusal names the window left

The refusal names every line that disagrees and the window of releases the tree may still be cut as — which can be empty, when a tree has landed one ramp and still promises another at the same release, and an empty window is itself the answer: nothing may be published until the rest of that release's ramps land.

#### 4.2.6 Every publication path runs the release guard

`release.yml`'s verify job runs the release guard on the version it is about to publish, which is every publication path — a `vX.Y.Z` tag push, a manual dispatch, and the non-publishing dry run both bump helpers wait on before they touch `main`. `auto-bump.yml` and `release-minor.yml` run it again on the version they compute, beside their existing gates, so a bump that could not be published fails before it pushes a candidate branch rather than after.

### 4.3 `release.yml` publishes a commit that already carries its version

A `vX.Y.Z` tag triggers `.github/workflows/release.yml` directly. The same workflow can also be run manually from the release commit: the operator enters the version, crate publishing is enabled by default, and the workflow creates `vX.Y.Z` if that tag does not already exist. If the requested tag already exists, that tag becomes the release source ref after the workflow verifies the tagged Cargo package versions match the requested version; this lets a failed release be recovered from a newer workflow commit without moving the release tag. In either entry path, the workflow verifies the tag/version matches the versions of all three Cargo packages it publishes ([§FS-distribution.4.10](FS-distribution.md#410-cratesio-publishes-in-dependency-order-and-artifacts-upload-last)), runs a fail-fast preflight on the crates.io token when crate publishing is enabled, re-runs `scripts/check-registry-names.sh` so claimed package names must still be available or owned by this project ([§FS-distribution.1.1](FS-distribution.md#11-package-names)), then builds and self-checks profile-guided-optimized binaries ([§FS-distribution.4.9](FS-distribution.md#49-distributed-binaries-are-profile-guided-optimized)) on six targets — `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`, and `aarch64-pc-windows-msvc`. `release.yml` does not bump versions: the selected commit already carries the version being released.

### 4.4 Two helper workflows bump the version on a validated candidate

Version bumping lives in separate helper workflows, and neither publishes directly. `.github/workflows/release-minor.yml` is a manual helper that starts from the latest `vX.Y.Z` tag, computes `vX.(Y+1).0`, verifies that `main` has commits since the previous tag and that CI is green on the current `main` tip, commits the workspace version bump ([§FS-distribution.4.5](FS-distribution.md#45-what-the-version-bump-includes)) to a temporary release-candidate branch, runs `release.yml` as a non-publishing dry run on that candidate, and only then fast-forwards `main` and dispatches `release.yml` for the real publish. `.github/workflows/auto-bump.yml` is the scheduled/manual patch helper with the same shape: when `main` has substantive non-doc/CI changes since the latest `vX.Y.Z` tag and CI is green on that exact tip, it computes `vX.Y.(Z+1)`, validates the version bump on a temporary release-candidate branch, and publishes the bump to `main` only after the dry run succeeds. Neither helper waits for anything to be written first: the section a release publishes is built by the bump from what the repository already holds ([§FS-distribution.4.6](FS-distribution.md#46-the-release-lists-the-pull-requests-merged-since-the-previous-tag)), so the scheduled helper's filter decides only *whether* a patch release happens and never what it lists, and either helper can run at any moment. `release-minor.yml`, dispatched by a person, refuses a range in which no pull request was merged, through the bump's own refusal ([§FS-distribution.4.6.3](FS-distribution.md#463-a-refused-release-leaves-the-tree-as-it-was)). In both, the advance to the next `-dev` version ([§FS-distribution.4.1](FS-distribution.md#41-between-releases-main-carries-a-dev-version)) runs only after a release, so a run that releases nothing opens no `-dev` version. Both helpers use the release push credential configured for branch-protection bypass.

### 4.5 What the version bump includes

In both helpers, "the version bump" includes:

- the Cargo manifests and the lockfile;
- any checked fixture whose expected output embeds `grund --version`;
- every **ramp constant** whose window is stated as a version in message text — a deprecation deadline is a promise about a release, and a bump is the only moment it can come due ([§DF-index-compatibility-ramp.2.3](../decisions/functional/DF-index-compatibility-ramp.md#23-both-findings-name-their-versions-and-a-test-keeps-the-names-honest) states the rule). Both ramps that rule was written for have since landed; the live ones are the absorbed-scan warning, the narrowed-alias scope suffix, and the `agents-init` compatibility tail, and the absorbed-scan ramp has a unit test that fails the bump which reaches its deadline rather than letting it pass silently ([§FS-distribution.4.2.1](FS-distribution.md#421-a-test-can-hold-only-the-pending-half));
- the deterministic changelog rotation performed by `scripts/prepare_changelog_release.py prepare`: the section [§FS-distribution.4.6](FS-distribution.md#46-the-release-lists-the-pull-requests-merged-since-the-previous-tag) builds — the list of pull requests and, after it, its compatibility notices — becomes the new inline release section, written where the former inline latest release stood; the former is archived under `docs/changelog/<version>.md`, and the older-release section gains its archive link, `- [<version>](changelog/<version>.md) — <date>: N pull requests, M compatibility notices.`, where M counts the archived section's bullets under `### Compatibility notices` and N every other bullet it holds, so a release archived in the earlier Keep-a-Changelog shape counts every bullet as a pull request; a count of one is written in the singular. `scripts/prepare_changelog_release.py preview` prints the body `prepare` would write under the new release heading, and writes nothing.

The helper refuses rather than publish an incomplete or invented section in every case [§FS-distribution.4.6.3](FS-distribution.md#463-a-refused-release-leaves-the-tree-as-it-was) names, before it writes anything, so the release candidate is already e2e-clean and changelog-clean before `release.yml` runs.

### 4.6 The release lists the pull requests merged since the previous tag

No change is gated on the changelog, and nothing is written for it before a release either: no hook, no CI job and no release step asks a change, or a person, for an entry. The bump ([§FS-distribution.4.5](FS-distribution.md#45-what-the-version-bump-includes)) builds the release section from two things the repository already holds — the pull requests merged since the previous tag, each by its title ([§FS-distribution.4.6.1](FS-distribution.md#461-the-range-is-every-commit-since-the-previous-tag), [§FS-distribution.4.6.2](FS-distribution.md#462-one-line-per-pull-request-its-title-linked)), and the compatibility notices the decision records added since that tag ([§FS-distribution.4.6.4](FS-distribution.md#464-compatibility-notices-come-from-the-decisions)) — or refuses and writes nothing ([§FS-distribution.4.6.3](FS-distribution.md#463-a-refused-release-leaves-the-tree-as-it-was)). So a pull request's title is its release line, and the one thing a title cannot carry, what a verdict change breaks and for whom, is written where the change is decided.

The list is every merged pull request, with no docs or CI filter. `docs/` holds the specification, so a pull request that changes only docs can change what the tool promises, and a filter would hide exactly those. The scheduled helper's own filter ([§FS-distribution.4.4](FS-distribution.md#44-two-helper-workflows-bump-the-version-on-a-validated-candidate)) decides whether a patch release happens at all, not what a release lists. A list built from the merged pull requests cannot leave one out, which a hand-written record could only ask its writer to avoid; that is the release half of [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path).

#### 4.6.1 The range is every commit since the previous tag

`HEAD` is read once, first, and everything after reads that commit. The previous release is the highest `vX.Y.Z` tag reachable from it — a higher tag on a branch `HEAD` does not contain is not a previous release — and that tag must name the release `docs/changelog.md` keeps inline. For every commit in `<tag>..HEAD` the bump asks the forge which pull requests the commit belongs to, and keeps a pull request only when it is merged, its base is `main`, and its merge commit is in the range; a pull request is listed once however many of its commits the range holds.

Merges are rebase-only, so every commit of a merged pull request lands on `main`, and a list that has asked about every commit in the range has seen every pull request merged into it. A commit that belongs to no pull request contributes no line. The commit that opens the next development version, `Open X.Y.Z-dev for development` ([§FS-distribution.4.1](FS-distribution.md#41-between-releases-main-carries-a-dev-version)), is expected to be one; any other is named in one warning on standard error, by its short hash and subject, and the release goes on.

The list is newest first, ordered by where each pull request's last commit sits on `main`. The order is read from git, never from the forge's timestamps.

#### 4.6.2 One line per pull request, its title linked

Each pull request is one line, `- [<title>](<url>) (PR #N)`. The title is the pull request's own, with every run of whitespace collapsed to one space and the ends trimmed; each of `` \ ` * _ [ ] < > & `` in it is escaped with a backslash, and `§` is written `&sect;`, because `docs/changelog.md` is scanned ([§FS-check](FS-check.md#fs-check-grund-validates-every-citation-in-a-repo)) and a title must not become a live citation the release is then held to. The URL is the pull request's address on this repository, `<repository>/pull/N` for the same `N`, where the repository is the one the forge answers for; a pull request whose URL is anything else is refused ([§FS-distribution.4.6.3](FS-distribution.md#463-a-refused-release-leaves-the-tree-as-it-was)), because a line that links elsewhere than it says is worse than none.

#### 4.6.3 A refused release leaves the tree as it was

The bump reads everything it needs — the range, every commit's pull requests, and every notice it would publish — before it writes anything. It refuses, exits non-zero, names which case it met, and leaves every file byte for byte as it found it, when:

- `gh` is missing, any call to the forge fails, or any commit in the range goes unanswered;
- the checkout is shallow, or the previous tag does not name the release `docs/changelog.md` keeps inline;
- a compatibility notice it would publish is malformed ([§FS-distribution.4.6.4](FS-distribution.md#464-compatibility-notices-come-from-the-decisions)), or a pull request's URL is not this repository's ([§FS-distribution.4.6.2](FS-distribution.md#462-one-line-per-pull-request-its-title-linked));
- the range holds no merged pull request at all.

A partial list is never written: a commit the forge did not answer for may be exactly the pull request a reader needed to see.

#### 4.6.4 Compatibility notices come from the decisions

A decision record — a `DF` or a `DA` declaration — may carry the named section `## release-note: Release note`, holding exactly one bullet (`- ` and its text, continuation lines indented two spaces) and nothing else but blank lines, with its links written relative to the record's own file. A verdict correction must carry one: it is the release record [§REQ-backwards-compatibility.5](../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) condition 3 and [§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) ask for, and the meter `tests/integration/test_backwards_compatibility_routes.py` reads it there, offline, on the pull request that makes the correction.

The bump publishes, under `### Compatibility notices` after the list, the bullet of every decision whose `release-note` section exists at `HEAD` and did not exist at the previous tag, ordered by decision ID. The two trees are compared by decision ID, so a record moved or renamed since the tag is the same record. Each bullet is published as written, with every relative link rebased from the record's directory to `docs/`, and a link to an anchor of the record's own file gaining that file's path. A notice already present at the tag is not published again, however it was edited since, because it shipped with the release that added it. A release that adds no notice has no `### Compatibility notices` subsection. A `release-note` section it would publish that is not one well-formed bullet is refused ([§FS-distribution.4.6.3](FS-distribution.md#463-a-refused-release-leaves-the-tree-as-it-was)).

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

### 4.12 Pending changelog entries are one file each

Retired. No change waits in the tree for a release: the release lists the pull requests merged since the previous tag itself ([§FS-distribution.4.6](FS-distribution.md#46-the-release-lists-the-pull-requests-merged-since-the-previous-tag)), and the one record a change still writes for a release, a compatibility notice, lives on the decision record that makes the change ([§FS-distribution.4.6.4](FS-distribution.md#464-compatibility-notices-come-from-the-decisions)). `docs/changelog/unreleased/`, the store of pending entries this section specified, is gone with it. The number and its heading stay so that neither is reused, the way [§FS-config.2](FS-config.md#2-precedence) outlived its withdrawal.

## 5. What we do not promise

- 100% identical APIs across languages. Each binding is idiomatic to its host (camelCase for Node, snake_case for Python, `Result<T,E>` for Rust). The *behavior* is identical; the surface fits each ecosystem.
- Stable ABI for the C-level FFI. Bindings link against the Rust core at compile time; we do not ship a separate C library.
