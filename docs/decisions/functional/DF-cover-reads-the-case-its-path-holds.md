# DF-cover-reads-the-case-its-path-holds: a narrowed `grund cover` reads the E2E case its path holds, so a case file named on the command line, or a case linked out of the cases folder, fails as `check` does

**Date:** 2026-11-10

## status: Accepted

## 1. Context

`grund cover <path>` is one scan narrowed to its path ([§FS-scan.path.cover](../../functional-spec/FS-scan.md#pathcover-cover-runs-one-narrowed-scan)), and the pass over the E2E cases read a case on such a scan only when the path was a directory whose canonical form lay at or above the canonical cases folder, or below it. Two paths that hold a case's file as the tree spells it met neither test, and the run read no case and exited `0` over a manifest `grund check` on the same path, and `grund cover` over the whole tree, report with exit `2` ([issue #1327](https://github.com/agent-grounds/grund/issues/1327), reported against `grund 0.22.2-dev+0cc6275878`):

- **A path naming a case's file.** `grund cover e2e/cases/b/expected.exit`, with that file holding the bytes `ff fe`, exited `0` whether or not `b` is a link: the pass read nothing for a path naming a file.
- **A case directory that is a link out of the cases folder.** With `e2e/cases/b` a link to `../../elsewhere/b`, `grund cover e2e/cases/b` exited `0`: the path's canonical form, `elsewhere/b`, is neither below nor above the cases folder.

[§DF-e2e-case-failure-at-the-file.2.4](DF-e2e-case-failure-at-the-file.md#24-what-does-not-move) recorded both exits as left where they were by [issue #1264](https://github.com/agent-grounds/grund/issues/1264), which moved where `check` places a case failure and did not touch `cover`; the fix for #1264 deferred them to an issue of their own, and this is that record.

## 2. Decision

### 2.1 What the path holds decides, as for `check`

A narrowed `cover` reads the case whose file its path holds as the tree spells it under the configured folder, and holds a failure there exactly as [§FS-declarations.e2e-case.failure](../../functional-spec/FS-declarations.md#e2e-casefailure-a-case-that-cannot-be-read-fails-alone) says for any run narrowed to a path ([§FS-cover.1.2](../../functional-spec/FS-cover.md#12-a-path-that-holds-an-e2e-cases-file-reads-that-case)). That is the file itself, the case directory and any directory above it, and a link among them is followed, not compared by where it resolves. `cover` and `check` already answer alike on the case directory of a plain case and on the cases folder; this makes them answer alike on every path that holds the file.

### 2.2 Exits that move

| Tree | Run | Before | Now |
|---|---|---|---|
| `e2e/cases/b/expected.exit` cannot be decoded | `grund cover e2e/cases/b/expected.exit` | exit `0`, nothing printed | `error: e2e/cases/b/expected.exit: …`, exit `2` |
| the same, `b` a link to `../../elsewhere/b` | `grund cover e2e/cases/b/expected.exit` | exit `0`, nothing printed | `error: e2e/cases/b/expected.exit: …`, exit `2` |
| the same, `b` a link to `../../elsewhere/b` | `grund cover e2e/cases/b` | exit `0`, nothing printed | `error: e2e/cases/b/expected.exit: …`, exit `2` |

`--format json` moves with the text: the `io` entry names the same path.

### 2.3 What does not move

- A path that holds none of the case's failing files: another, readable file of the case such as its `command.args`, and the link's target `elsewhere/b`, which does not resolve to the configured spelling ([§DF-e2e-case-failure-at-the-file.2.4](DF-e2e-case-failure-at-the-file.md#24-what-does-not-move)). Each exits `0` and prints what it printed.
- A narrowed `cover` whose path holds no failing case file prints what it printed; reading the case its path holds adds no record to its output.
- `grund cover` over the whole tree, over the cases folder and over a plain case directory, each of which already exited `2`.
- `fmt --check` on a narrowed path, whose counterpart is reported on its own.

## 3. The correction route

These exits flip a shipped verdict from `0` to `2`, which [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered) allows only by a named route. [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) is the route, and all five conditions hold.

1. *Prior prohibition.* The old verdicts violated [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips): a file the scan cannot read is reported at its path and the run exits `2`, and the report never passes as if it were complete. A `cover` run asked about `e2e/cases/b/expected.exit`, or about a case directory, passed as if it had read the manifest it was asked about. [§FS-check.2.4](../../functional-spec/FS-check.md#24-an-incomplete-run) makes the same failure a per-file `2` for every run that meets it. Both already applied when `cover`'s narrowed scan first shipped.
2. *Accepted proof.* This record. [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)'s deprecation path does not fit, because nothing a user writes selects whether a case is read; [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)'s mechanical migration does not fit, because nothing a user wrote is at fault.
3. *Named release.* The release note below.
4. *Actionable findings.* What replaces the old `0` is the `error:` line naming the file, the reason it failed and the remedy, the line `check` and whole-tree `cover` print for the same file.
5. *No new licence.* This route cannot justify ordinary policy tightening, feature removal, or a prohibition invented by this change, and it is used for none of them. No check, rule or partition changes: a narrowed `cover` reaches `2` only for the file [§FS-declarations.e2e-case.failure](../../functional-spec/FS-declarations.md#e2e-casefailure-a-case-that-cannot-be-read-fails-alone) already says a run narrowed to that path holds.

**The weak point.** [§DF-e2e-case-failure-at-the-file.2.4](DF-e2e-case-failure-at-the-file.md#24-what-does-not-move) recorded both exits as staying, and a reader may take that as a decision. It argued nothing for them; it named what #1264 left alone, and its own fix deferred them here.

## release-note: Release note

- [§FS-cover.1.2](../../functional-spec/FS-cover.md#12-a-path-that-holds-an-e2e-cases-file-reads-that-case), [§DF-cover-reads-the-case-its-path-holds](DF-cover-reads-the-case-its-path-holds.md#df-cover-reads-the-case-its-path-holds-a-narrowed-grund-cover-reads-the-e2e-case-its-path-holds-so-a-case-file-named-on-the-command-line-or-a-case-linked-out-of-the-cases-folder-fails-as-check-does), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips): **`grund cover` narrowed to an E2E case's file, or to a case directory linked out of the cases folder, reads that case.** **Exits move to `2`:** `grund cover e2e/cases/b/expected.exit`, and `grund cover e2e/cases/b` where `b` is a link to a directory outside the cases folder, exit `2` and name a case file that cannot be read or decoded, where they exited `0` without reading it, as `grund check` on the same path already did. Make the file readable, or repair its contents. **Who notices:** an editor or a review step that asks `cover` about one case or one case file in a tree holding a bad case. A path that holds no failing file of the case, the link's target among them, prints and exits as before. The verdict moves by the correction route of [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), because a file the scan cannot read is reported at its path under [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips), and every finding that replaces the old verdict names its location and the action to take. Closes [issue #1327](https://github.com/agent-grounds/grund/issues/1327).
