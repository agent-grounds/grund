# FS-distribution: grund distribution targets

`grund` is written in Rust; the target distribution is **all three** major language ecosystems — cargo, npm, and PyPI — with idiomatic API bindings on each. The check engine stays a single shared library; only the surfaces differ. Today the implemented release path publishes the Rust crates: `grund-core`, the Cargo CLI package `grund`, and the optional Cargo LSP package `grund-lsp`. The npm and PyPI bindings, including their future `grund-lsp` packages, are tracked in [§RM-distribution](../roadmap.md#rm-distribution-cargo--npm--pypi-from-one-engine). Serves [§GOAL-multi-language](../goals.md#goal-multi-language-same-engine-three-platforms) and [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, body, section, lead), [§FS-terms.terms.2](FS-terms.md#terms2-citations)
(shorthand), [§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (stub), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope, config root, workspace,
alias), and [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, severity, suggestion, caution, verdict).

- **entry** — One change recorded for the changelog and not yet released: a file of its own
  under `docs/changelog/unreleased/`, holding the one bullet the release publishes for it.

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

The implemented release workflow publishes the Cargo CLI today and builds downloadable PGO binaries for every supported desktop CI platform. `release.yml` publishes a commit that already carries its version ([§FS-distribution.4.3](FS-distribution.md#43-releaseyml-publishes-a-commit-that-already-carries-its-version)), and two helper workflows make that commit on a candidate they validate first ([§FS-distribution.4.4](FS-distribution.md#44-two-helper-workflows-bump-the-version-on-a-validated-candidate)), with a version bump that covers the manifests, the lockfile, `--version` fixtures, ramp constants and the changelog rotation ([§FS-distribution.4.5](FS-distribution.md#45-what-the-version-bump-includes)). The changelog gate requires an entry file ([§FS-distribution.4.12](FS-distribution.md#412-pending-changelog-entries-are-one-file-each)), before the push and again in pull-request CI, and the release stamps each entry's number ([§FS-distribution.4.6](FS-distribution.md#46-the-changelog-gate-keeps-the-release-section-mappable-to-its-pull-requests)), and the changelog supplies the GitHub release notes ([§FS-distribution.4.7](FS-distribution.md#47-the-changelog-is-the-source-of-release-notes)). Linux binaries build in digest-pinned `manylinux2014` containers and every job on one pinned toolchain ([§FS-distribution.4.8](FS-distribution.md#48-one-old-glibc-baseline-one-pinned-toolchain)); distributed binaries are PGO-built, with an LTO-only fallback for a platform whose PGO training is broken ([§FS-distribution.4.9](FS-distribution.md#49-distributed-binaries-are-profile-guided-optimized)); crates publish to crates.io in dependency order before artifacts upload ([§FS-distribution.4.10](FS-distribution.md#410-cratesio-publishes-in-dependency-order-and-artifacts-upload-last)). npm and PyPI join the same shape once their frontends exist ([§FS-distribution.4.11](FS-distribution.md#411-the-full-ecosystem-release)).

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

Version bumping lives in separate helper workflows, and neither publishes directly. `.github/workflows/release-minor.yml` is a manual helper that starts from the latest `vX.Y.Z` tag, computes `vX.(Y+1).0`, verifies that `main` has commits since the previous tag and that CI is green on the current `main` tip, commits the workspace version bump ([§FS-distribution.4.5](FS-distribution.md#45-what-the-version-bump-includes)) to a temporary release-candidate branch, runs `release.yml` as a non-publishing dry run on that candidate, and only then fast-forwards `main` and dispatches `release.yml` for the real publish. `.github/workflows/auto-bump.yml` is the scheduled/manual patch helper with the same shape: when `main` has substantive non-doc/CI changes since the latest `vX.Y.Z` tag and CI is green on that exact tip, it computes `vX.Y.(Z+1)`, validates the version bump on a temporary release-candidate branch, and publishes the bump to `main` only after the dry run succeeds. Both helpers use the release push credential configured for branch-protection bypass.

### 4.5 What the version bump includes

In both helpers, "the version bump" includes:

- the Cargo manifests and the lockfile;
- any checked fixture whose expected output embeds `grund --version`;
- every **ramp constant** whose window is stated as a version in message text — a deprecation deadline is a promise about a release, and a bump is the only moment it can come due ([§DF-index-compatibility-ramp.2.3](../decisions/functional/DF-index-compatibility-ramp.md#23-both-findings-name-their-versions-and-a-test-keeps-the-names-honest) states the rule). Both ramps that rule was written for have since landed; the live ones are the absorbed-scan warning, the narrowed-alias scope suffix, and the `agents-init` compatibility tail, and the absorbed-scan ramp has a unit test that fails the bump which reaches its deadline rather than letting it pass silently ([§FS-distribution.4.2.1](FS-distribution.md#421-a-test-can-hold-only-the-pending-half));
- the pull request numbers `scripts/prepare_changelog_release.py stamp` can resolve: it runs immediately before the rotation below and, for each entry ([§FS-distribution.4.12](FS-distribution.md#412-pending-changelog-entries-are-one-file-each)) that does not already end in `(PR #N)`, finds the commit that added the entry — the commit at which its slug last appeared in `docs/changelog/unreleased/`, so neither a later change of category, even one made in the same commit as an edit, nor a later edit moves it, and a slug used again after an earlier release is not taken for the entry that used it first — and resolves that commit to its pull request, so an entry keeps the number of the pull request that wrote it whoever edited it since. Where exactly one resolves, the number is written at the end of the entry and nowhere else: a trailing `(PR #TBD)` is replaced, otherwise ` (PR #N)` is appended to the entry's last line, and a `PR #TBD` written in the entry's prose is left as it stands. An entry whose commit resolves to no pull request or to more than one, or which no commit has added yet, is warned about once and left as it stands; `stamp` never fails the release, never edits `docs/changelog.md`, and in a release in which nothing resolves writes nothing at all;
- the deterministic changelog rotation performed by `scripts/prepare_changelog_release.py prepare`: the entries become the new inline release section, written directly under `## Unreleased`'s pointer — grouped into one section per category in the order [§FS-distribution.4.12](FS-distribution.md#412-pending-changelog-entries-are-one-file-each) gives, the oldest-landed entry first, and every relative link rebased from `docs/changelog/unreleased/` to `docs/` — and the entry files are deleted, leaving the directory's `README.md` and the pointer as they were; the former inline latest release is archived under `docs/changelog/<version>.md`, and the older-release section gains the archive link. `scripts/prepare_changelog_release.py preview` prints the body `prepare` would write under the new release heading, and writes nothing.

The helper fails rather than inventing release notes when `docs/changelog/unreleased/` holds no entry, and fails rather than drop one when a file there is not a well-formed entry or a bullet sits under `## Unreleased`'s pointer, so the release candidate is already e2e-clean and changelog-clean before `release.yml` runs.

### 4.6 The changelog gate keeps the release section mappable to its pull requests

`scripts/check_changelog_pr_entry.py` makes every pull request record its change, and both halves of it ask one question of local git alone: the head must hold an entry ([§FS-distribution.4.12](FS-distribution.md#412-pending-changelog-entries-are-one-file-each)) whose slug `docs/changelog/unreleased/` does not hold at `git merge-base <base> <head>`. The `pre-push` hook asks it against the remote's `main` before any pull request exists; `.github/workflows/ci.yml` asks it on `pull_request` events against the base commit the event names. Both sides of that one comparison are listed from commits — the head the check is handed and the merge base — and never from the working tree, so neither the merge commit `actions/checkout` leaves in the tree on a `pull_request` event nor an uncommitted file decides what this branch added.

**Only an added slug counts.** Editing an entry, changing its category or otherwise renaming it under the same slug, editing the directory's `README.md`, and writing a bullet under `docs/changelog.md`'s `## Unreleased` add no entry, and a branch that does nothing more is refused. An entry whose bullet already sat under the merge base's `## Unreleased` was **moved**, not written, and counts no more than an edit does; two bullets are the same once whitespace collapses, a trailing `PR #N` or `PR #TBD` is dropped, and a link's destination is disregarded, so a bullet whose links were rebased into the entry's directory and whose number was written on the way is still the bullet it was.

For every file the branch adds or changes in the directory, the gate refuses a file that is not an entry — the `README.md` aside — an entry with no category or with one outside the six [§FS-distribution.4.12](FS-distribution.md#412-pending-changelog-entries-are-one-file-each) names, an entry that is not exactly one bullet, and a slug two files share; an entry the branch left untouched is not examined. **No pull request number is required**: an entry may end in `(PR #TBD)` or in nothing. Where a number *is* known, a pull request number the branch writes into an entry must be the pull request's own — `PR #TBD` is not a number, a number the entry already carried at the merge base was not written by this branch, and a moved entry's number is never examined. This keeps the release section mappable back to the pull requests it contains, with the number written at release time by [§FS-distribution.4.5](FS-distribution.md#45-what-the-version-bump-includes)'s `stamp` rather than by the author who could not have known it.

A branch that adds no entry is refused with the directory and the file to add, `docs/changelog/unreleased/<slug>.<category>.md`, followed by the six categories and the note that a number is optional. The suggested slug is the branch name — the pushed local ref with `refs/heads/` dropped before the push, the pull request's head branch in CI — lowercased, with every run of characters other than `a`–`z` and `0`–`9` made one `-` and the hyphens at either end trimmed, so `fix/issue-379` suggests `fix-issue-379`. Three conditions skip the check and no others: a push that deletes a ref, which adds no entry to require; a head the base already contains; and a base no merge base can be taken against — because no ref resolves it, or because it shares no history with the head, as a shallow clone's grafted base does — where it falls back to requiring at least one entry at the head and says on stderr which of the two it met. Push CI does not run the gate, and a release-candidate branch opens no pull request, so neither needs an exemption of its own. `SKIP=changelog-pr-entry git push` is the documented escape for a branch that is not becoming a pull request, and the pre-push refusal names it.

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

Every change this repository has merged and not yet released is an entry, one file under `docs/changelog/unreleased/`, in the format that directory's `README.md` states. The README's first part is the format itself and holds no rule of this repository's, so another repository can take it unchanged; its second part is this repository's use of it. So two pull requests that each record a change add two files and edit no line in common, and they merge or rebase in either order without a conflict in any changelog file. `docs/changelog.md` holds no pending change: its `## Unreleased` is a fixed one-line pointer to that README, which no ordinary change edits.

This repository's use of the format: an entry's file name is `<slug>.<category>.md`, and the category is required and is one of `added`, `changed`, `deprecated`, `removed`, `fixed` and `security`, released in that order as the Keep-a-Changelog sections of the same names capitalized. Within a section, the entry that landed first comes first, and entries that landed in one commit are ordered by file name. The entry's body is the one bullet the release publishes — `- ` and its text, continuation lines indented two spaces — with its links written relative to the entry's own file, as `grund fmt --write` writes them. A `**Schema:**` callout is a `changed` entry whose text begins with `**Schema:**`. The gate that requires a pull request to add an entry is [§FS-distribution.4.6](FS-distribution.md#46-the-changelog-gate-keeps-the-release-section-mappable-to-its-pull-requests); the release that numbers, collects and deletes the entries is [§FS-distribution.4.5](FS-distribution.md#45-what-the-version-bump-includes), and the section it writes is still the one [§FS-distribution.4.7](FS-distribution.md#47-the-changelog-is-the-source-of-release-notes) reads the release notes from.

## 5. What we do not promise

- 100% identical APIs across languages. Each binding is idiomatic to its host (camelCase for Node, snake_case for Python, `Result<T,E>` for Rust). The *behavior* is identical; the surface fits each ecosystem.
- Stable ABI for the C-level FFI. Bindings link against the Rust core at compile time; we do not ship a separate C library.
