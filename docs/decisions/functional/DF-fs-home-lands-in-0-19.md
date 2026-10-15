# DF-fs-home-lands-in-0-19: the built-in FS home is docs/functional-spec/ from 0.19.0

**Date:** 2026-10-15

## status: Accepted

## 1. Context

[§DF-fs-home-moves-from-0-19](DF-fs-home-moves-from-0-19.md#df-fs-home-moves-from-0-19-the-built-in-fs-home-moves-back-to-docsfunctional-spec-in-0190-and-the-template-declares-req-now) moved the built-in `FS` home from `file =
"requirements.md"` to `folder = "docs/functional-spec"` in two releases: 0.18.0 warned
`fs-home-moves` at every project the move touches, and 0.19.0 moves it. That record's notice
shipped with 0.18.0, and a notice already released is not published again
([§FS-distribution.4.6.4](../../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions)), so the release that lands the move needs a notice of its own, and this
record carries it.

## 2. Decision

In 0.19.0 (agent-grounds/grund#723):

- the built-in `FS` row is `folder = "docs/functional-spec"` ([§FS-config.3.4.4](../../functional-spec/FS-config.md#344-the-default-kinds)), so a project with
  no config and a `grund.toml` that omits `[[kinds]]` get the same rows, and the special case of
  [§FS-config.3.4.4.4](../../functional-spec/FS-config.md#3444-a-config-that-omits-kinds-keeps-the-older-fs-home) is retired;
- the `software-development` template's `FS` row is `folders = ["docs/functional-spec"]`, and
  `--scaffold` writes `docs/functional-spec/README.md` and no `requirements.md`
  ([§FS-init.2.4.10](../../functional-spec/FS-init.md#2410-req-is-declared-in-docsrequirements), [§FS-init.2.1.3](../../functional-spec/FS-init.md#213-the-software-development-starter-files));
- `fs-home-moves` is no longer a code, so `--only fs-home-moves` and `--ignore fs-home-moves` are
  refused as any unknown code is ([§FS-check.checks.fs-home-moves](../../functional-spec/FS-check.md#checksfs-home-moves-the-built-in-fs-home-moves-in-grund-0190));
- the version-1 `[scan] include` default keeps `requirements.md` ([§FS-config.3.5](../../functional-spec/FS-config.md#35-scan--what-gets-scanned)), because
  dropping it would stop reading a file in a project that changed nothing, which no release
  announced.

## 3. Rejected alternatives

- **Rewrite the 0.18.0 notice to say it shipped.** The bump would not publish it, so the release
  that moves the home would say nothing about it.
- **Keep `fs-home-moves` as a code that never fires, for a release.** It would keep a script's
  `--ignore fs-home-moves` working, but a code that selects nothing hides that the selection is
  stale, and [§FS-declarations.checks.parse-moves](../../functional-spec/FS-declarations.md#checksparse-moves-a-verdict-that-moves-in-0190) retired its code the same way.
- **Drop `requirements.md` from the default `include` as well.** No release warned that the file
  stops being read, so it would move verdicts silently ([§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)).

## release-note: Release note

- [§DF-fs-home-lands-in-0-19](DF-fs-home-lands-in-0-19.md#df-fs-home-lands-in-0-19-the-built-in-fs-home-is-docsfunctional-spec-from-0190), [§FS-config.3.4.4](../../functional-spec/FS-config.md#344-the-default-kinds), [§FS-init.2.4.10](../../functional-spec/FS-init.md#2410-req-is-declared-in-docsrequirements), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **the built-in `FS` home is now `docs/functional-spec/`, and the `fs-home-moves` warning is gone.** What grund 0.18.0 announced lands: a project with no `grund.toml` and a `grund.toml` that omits `[[kinds]]` both home `FS` in `docs/functional-spec/`, and a fresh `grund init --template software-development --scaffold` declares `FS` there with `folders = ["docs/functional-spec"]`, writes `docs/functional-spec/README.md`, and no longer writes `requirements.md`. **Who this affects:** only a project with no `grund.toml` whose `FS` declarations live in `requirements.md`, the project 0.18.0 warned with `fs-home-moves`: those declarations are no longer in the `FS` home. To keep them there, run `grund init` and set its `FS` row to `files = ["requirements.md"]`, since the row it writes is `folders = ["docs/functional-spec"]`; or move the declarations into `docs/functional-spec/`. Every config `grund init` has written names its `FS` home and is unaffected. `--only fs-home-moves` and `--ignore fs-home-moves` are now refused as an unknown code, so drop them from scripts. Closes [issue #723](https://github.com/agent-grounds/grund/issues/723).
