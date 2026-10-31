# DF-dev-build-names-its-commit: a development build's `--version` names its commit, and what a development build prints between releases was never a promise

**Date:** 2026-10-31

## status: Accepted

## 1. Context

Between releases `main` carries a `-dev` version ([§FS-distribution.4.1](../../functional-spec/FS-distribution.md#41-between-releases-main-carries-a-dev-version)), so `grund --version` says which side of the tag a build came from, and stops there: every build of `main` until the next release prints one line. The 0.18.1-dev window covered 391 commits. A log, a CI record or a benchmark result that captured `grund 0.20.1-dev` cannot say which of them it measured, and a `cargo install --path` records no commit anywhere else (agent-grounds/grund#1043).

Naming the commit changes the bytes of a development build's version line, and [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered) covers output bytes, which change only through the deprecation path of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path). [§FS-non-goals.13](../../functional-spec/FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree) rules out two installs of the same version disagreeing on any output byte. So the change has to say why neither is crossed, and [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise) asks for that argument in a decision record rather than assumed.

## 2. Decision

A development build reports its version with the commit it was built from as SemVer build metadata, `0.20.1-dev+41a08f39c2`, and `.dirty` after it when a tracked file differed from that commit ([§FS-distribution.4.1.1](../../functional-spec/FS-distribution.md#411-a-development-build-names-its-commit)). A release build is unchanged, byte for byte, and [§FS-cli.2.1](../../functional-spec/FS-cli.md#21---version) still prints `grund <semver>`.

### 2.1 What a development build prints was never covered

The guarantee is about upgrading: it opens with "Upgrading `grund`" and runs from release `N` to `N+1`, where release `N` ships the new form beside the old ([§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)). No such path runs through a development build. Its bytes already change with every merge to `main` and at every version bump, which rewrites `grund 0.20.1-dev` to `grund 0.21.1-dev` with no warning and no window. So what a development build prints between releases sits outside the guarantee, as the third item of [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise), and that does not expire at `1.0`. Every release's own bytes stay covered, and no release byte moves here.

The alternative argument is the pre-release licence of the same section: before `1.0` a surface may change without an alias where carrying the alias would cost more than the rename. A one-line `--version` cannot ship the old form beside the new at all. This record rests on the first argument, because the licence expires at `1.0` and a development build's line would then need a new argument for the same change.

### 2.2 The same version, for a development build

For [§FS-non-goals.13](../../functional-spec/FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree), two development builds are the same version when they were built from the same source contents and build inputs, and the label is not a fingerprint of either ([§FS-distribution.4.1.1](../../functional-spec/FS-distribution.md#411-a-development-build-names-its-commit)). Two dev builds of different commits already printed different bytes for the same input while reporting one version; the commit makes that visible rather than creating it. Two builds of one dirty tree with different patches carry one label and are still different versions.

### 2.3 How the commit is derived

- **Untracked files never make a build dirty.** The sources a build compiles are tracked files, and the label never claimed to cover anything else. Counting untracked files would mark every `cargo install --git` build dirty, through the `.cargo-ok` Cargo leaves in each checkout it builds, and would make every new file anywhere in the tree rerun the build script. `git describe --dirty` makes the same choice.
- **`GRUND_BUILD_COMMIT` is a fallback, never an override.** Qualifying git wins over it. A malformed value fails a development build even when git wins, so a typo fails on the machine where it was written rather than only on the gitless route it was meant for.
- **No spelling suppresses the commit.** A suppressed development build prints exactly the ambiguous line this record exists to remove, and a golden kept exact by suppression would pass only for a builder that set it.
- **A release reads nothing.** The `-dev` suffix is the only switch: a release version runs no git and reads no variable, so `release.yml`, `cargo install grund` and every PGO release build print `grund X.Y.Z` whatever the builder's environment says.

## 3. Consequences

- [§FS-distribution.4.1.1](../../functional-spec/FS-distribution.md#411-a-development-build-names-its-commit) defines the label once, and [§FS-cli.2.1](../../functional-spec/FS-cli.md#21---version), [§FS-distribution.requirements.versions](../../functional-spec/FS-distribution.md#requirementsversions-what-a-distributed-binary-says-about-versions-is-true-of-it--realized), [§FS-distribution.4.5](../../functional-spec/FS-distribution.md#45-what-the-version-bump-includes) and [§FS-distribution-candidate.5.2](../../functional-spec/FS-distribution-candidate.md#52-every-registry-row-installs-fresh-and-without-rust) cite it rather than restating it.
- The version-dependent goldens keep the bare version, and a development build's test expects that line followed by this build's recorded metadata ([§FS-distribution.4.5](../../functional-spec/FS-distribution.md#45-what-the-version-bump-includes)). `candidate.py set-version` stays their only writer.
- A dev candidate of the rehearsal lane is built from a clean checkout of its commit, and every route that cannot read git is handed that commit ([§FS-distribution-candidate.6.1](../../functional-spec/FS-distribution-candidate.md#61-one-commit-produces-one-manifest)).
- `grund-lsp --version`, the LSP's `serverInfo.version` and the Node binding's `engineVersion` and `packageVersion` keep the bare version.
- No release byte moves, so this record writes no compatibility notice.

## 4. Alternatives considered

| Approach | Why rejected |
|---|---|
| `grund 0.20.1-dev (41a08f39c2)` | Prints something beside `<semver>`, so [§FS-cli.2.1](../../functional-spec/FS-cli.md#21---version) would need a format allowance. Build metadata keeps the line one SemVer version, which a reader splitting on the first space still reads, and SemVer ignores it when ordering versions. |
| `git rev-parse --short` | Its length depends on the clone's object count and `core.abbrev`, so two clones of one commit could print different lines. The first ten digits of the full SHA do not. |
| A separate `grund version --build-info` | Leaves `--version` ambiguous, and every tool that already records it would have to learn a second command. |
| `GRUND_BUILD_COMMIT` as an override of git | A stale variable in a builder's environment would put one commit's name on another commit's code. |
| Rest on the pre-1.0 licence alone | It expires at `1.0`, while the reason a development build's line was never a promise does not. |
