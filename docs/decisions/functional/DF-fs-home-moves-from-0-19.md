# DF-fs-home-moves-from-0-19: the built-in FS home moves back to docs/functional-spec/ in 0.19.0, and the template declares REQ now

**Status:** Accepted
**Date:** 2026-10-13

## 1. Context

[Issue #661](https://github.com/agent-grounds/grund/issues/661) asks that the structure
`grund init` writes keep the requirements a project must never break as `REQ` in
`docs/requirements/`, apart from the behaviour `FS` specifies, as grund keeps its own, and that
`FS` leave the root file `requirements.md`, whose name says the opposite once `REQ` exists.

Two answers already exist for where `FS` lives. The built-in default is `file =
"requirements.md"` (§FS-config.3.4.4), and every config `grund init` writes names it explicitly,
because a written key matches its default ([§FS-init.2.4.3](../../functional-spec/FS-init.md#243-every-written-key-differs-from-its-default)). A config that omits `[[kinds]]`
keeps the older `folder = "docs/functional-spec"` ([§FS-config.3.4.4.4](../../functional-spec/FS-config.md#3444-a-config-that-omits-kinds-keeps-the-older-fs-home)). Moving only the
template would add a third answer; moving the default would move the FS home of every project
that relies on it.

## 2. Decision

The author's ruling on #661 settles it in three parts.

### 2.1 The built-in FS home moves back to docs/functional-spec/

The built-in `FS` home becomes `folder = "docs/functional-spec"`, the home a config that omits
`[[kinds]]` already has, so the move unifies two answers and deletes the special case of
[§FS-config.3.4.4.4](../../functional-spec/FS-config.md#3444-a-config-that-omits-kinds-keeps-the-older-fs-home) rather than adding one. The template's `FS` row moves in the same release,
together with the default, so the template never disagrees with the default.

### 2.2 The release split

The only projects the move touches are those with **no config file** whose `FS` declarations
live in `requirements.md`: every config `grund init` has written names its `FS` home, and an
explicit home does not move. So the move takes the warning route of
[§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path):

| Release | What ships |
|---|---|
| 0.18.0 | this record, and the `fs-home-moves` warning ([§FS-check.checks.fs-home-moves](../../functional-spec/FS-check.md#checksfs-home-moves-the-built-in-fs-home-moves-in-grund-0190)), which fires on exactly that case and names 0.19.0 |
| 0.19.0 | the built-in default and the template's `FS` row move together, [§FS-config.3.4.4.4](../../functional-spec/FS-config.md#3444-a-config-that-omits-kinds-keeps-the-older-fs-home) is retired, and the warning goes ([issue #723](https://github.com/agent-grounds/grund/issues/723)) |

### 2.3 REQ goes into the template now, with no built-in default

The `software-development` template declares `REQ` as `folder = "docs/requirements"` in 0.18.0
([§FS-init.2.4.10](../../functional-spec/FS-init.md#2410-req-is-declared-in-docsrequirements)), with a starter index, a row in the project map and the vocabulary, and a
commented `[citations.REQ]` block in its canonical ruleset. `REQ` gets no built-in default: one
would add a row to the managed block of every repository whose config omits `[[kinds]]`, and
`grund init --check` would fail there for a change nobody asked for. [§FS-init.2.4.3](../../functional-spec/FS-init.md#243-every-written-key-differs-from-its-default) therefore
reads "every written key that has a built-in default matches it".

## 3. Rejected alternatives

- **Change only the template.** Existing repositories are untouched, but three `FS` homes stay in
  play and the generated config stops teaching the defaults.
- **Move the default in this release.** A project with no config would lose its FS declarations
  from the FS home in the release that first tells it so, which [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)
  forbids.
- **One warning per declaration.** One edit, `grund init` or a move, clears every declaration, so
  one line is the whole finding.
- **A config-level warning with no location, like `deprecated-config-location`.** Whether the
  move touches a project depends on what the scan finds, so the finding has a site, and an editor
  can put it on the line.

## 4. The cases the warning leaves alone

The warning ([§FS-check.checks.fs-home-moves](../../functional-spec/FS-check.md#checksfs-home-moves-the-built-in-fs-home-moves-in-grund-0190)) fires on the one case the move
breaks, and each case it leaves alone has its reason:

- **A config that omits `[[kinds]]`.** It already homes `FS` at `docs/functional-spec`
  ([§FS-config.3.4.4.4](../../functional-spec/FS-config.md#3444-a-config-that-omits-kinds-keeps-the-older-fs-home)), so the move does not touch it.
- **A config that writes `kinds = []`.** It declares no `FS` at all ([§FS-config.3.4.4.5](../../functional-spec/FS-config.md#3445-kinds---declares-no-kinds)).
- **Any config with a `[[kinds]]` table**, a generated one included. Every config `grund init`
  has written names its `FS` home explicitly, and an explicit home does not move.
- **A declared `FS` row that names no `file` or `folder`.** Only the built-in rows read the
  per-kind default, so such a row has no home today and none after the move.
- **A project with no config whose `requirements.md` holds no `FS` declaration**, or which has no
  `requirements.md`. Nothing in it moves.

A workspace member loaded with no config of its own takes the built-in rows as a root with no
config does, so its home moves too, and it is warned at its own `requirements.md`. One edit, `grund
init` or a move, clears every declaration of a project, so one warning is the whole finding: a
second `FS` declaration in the same file adds no second line.

## release-note: Release note

- [§DF-fs-home-moves-from-0-19](DF-fs-home-moves-from-0-19.md#df-fs-home-moves-from-0-19-the-built-in-fs-home-moves-back-to-docsfunctional-spec-in-0190-and-the-template-declares-req-now), [§FS-check.checks.fs-home-moves](../../functional-spec/FS-check.md#checksfs-home-moves-the-built-in-fs-home-moves-in-grund-0190), [§FS-init.2.4.10](../../functional-spec/FS-init.md#2410-req-is-declared-in-docsrequirements), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **the `software-development` template declares `REQ` in `docs/requirements/`, and from 0.19.0 the built-in `FS` home is `docs/functional-spec/`.** A fresh `grund init --scaffold` declares `REQ` as `folder = "docs/requirements"`, writes `docs/requirements/README.md`, lists `REQ` in the managed block's project map, and carries a commented `[citations.REQ]` block; `FS` stays in `requirements.md` in this release. **Who this affects:** only a project with no `grund.toml` whose `FS` declarations live in `requirements.md`. Its `grund check` now prints the `fs-home-moves` warning, at exit `0`, naming 0.19.0 as the release where the default moves; run `grund init` to write a config that keeps `FS` in `requirements.md`, or move the declarations into `docs/functional-spec/`. Every config `grund init` has written names its `FS` home and is unaffected. The move is [issue #723](https://github.com/agent-grounds/grund/issues/723).
