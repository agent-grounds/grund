# DF-home-lookup-failure-fails-at-the-home: a root whose lookup fails for a reason other than absence fails at the root, and exits move with it

**Date:** 2026-11-09

## status: Accepted

## 1. Context

The walk asked whether each root exists and took any failure to look it up as no. A fetched kind's home at `data/values` under a `chmod 000 data` therefore scanned as an absent home: nothing was recorded at it, `check` said `no snapshot` for a file that was on disk and that it had never read, and on a run with no wider walk root meeting `data` it exited `0` with no error at all. Reported as [issue #1318](https://github.com/agent-grounds/grund/issues/1318).

## 2. Decision

A root is absent only when its lookup says so. Any other failure is a scan failure at the root ([§FS-scan.scope.homes.lookup](../../functional-spec/FS-scan.md#scopehomeslookup-a-home-that-cannot-be-looked-up-is-a-scan-failure-not-an-absent-home), [§FS-check.2.4](../../functional-spec/FS-check.md#24-an-incomplete-run)), after the root's boundaries have been judged on where it goes as far as the disk answers ([§FS-scan.scope.homes.lookup.outward](../../functional-spec/FS-scan.md#scopehomeslookupoutward-a-root-link-leaving-the-project-answers-to-its-gate-first)).

### 2.1 Exits that move

Measured on one fixture: a kind `CONST` whose only home is `data/values`, holding `CONST-room-limit`, a document citing it, and `chmod 000 data`, as a non-root user on Linux. "Before" is this branch without the fix; "Now" is the final build.

| Config | Run | Before | Now |
|---|---|---|---|
| Version 1, `resolve = "should"` | `grund check`, `grund check --format json` | a `no snapshot` warning, exit `0` | adds `error: data/values: permission denied; make the path readable, or keep it out of the scan`, exit `2` |
| Same | `grund list` | nothing, exit `0` | the same line, exit `2` |
| Same | `grund refs CONST-room-limit` | the citation, exit `0` | the citation and the same line, exit `2` |
| Same | `grund fmt --check` | nothing, exit `0` | `error: nothing was rewritten: data/values: ...`, exit `2` |
| Same | `grund show CONST-room-limit`, `grund CONST-room-limit` | `ID not found`, exit `1` | the same line, exit `2` |
| Same, a second kind with a readable `FS-a` | `grund show FS-a` | the body, exit `0` | the same line and no body, exit `2` |
| Version 2, `data` gitignored, so no wider walk meets it | `grund check` | `unknown reference`, exit `1` | adds the same line, exit `2` |
| Version 1, `data/values` a link to itself | `grund check`, `grund list` | exit `0` | `error: data/values: filesystem loop or indirection limit ...`, exit `2` |
| Version 2, `data/values` a link to itself | `grund check` | `unknown reference`, exit `1` | adds that line, exit `2` |
| Version 1, `data/values` a link inside the project behind a denied directory | `grund check` | a `no snapshot` warning, exit `0` | adds the same line, exit `2` |

An `include` entry and a default root such as `src` move the same way, since the rule is the walk's for every root. `grund show` takes the strictness it already has on any failed scan.

### 2.2 What moves in bytes only

Where a wider walk root meets the denied parent, version 2 exited `2` before and exits `2` now, with one more `error:` line at the home, since the parent and the home are two failures ([§FS-check.2.4.2](../../functional-spec/FS-check.md#242-one-failure-one-line)). A run narrowed to a path outside the home, as `grund check docs/offer.md`, keeps its exit and gains the [§FS-check.1.3.6.3](../../functional-spec/FS-check.md#1363-a-file-the-wider-walk-could-not-read-is-said-out-loud) caution naming the home.

### 2.3 What does not move

- A home that does not exist, one below a regular file, and a dangling link scan as nothing, exit unchanged.
- A root link going outside the project with a denied directory on the way is pruned, as with its target open: exit `0` before and now ([§FS-scan.scope.homes.lookup.outward](../../functional-spec/FS-scan.md#scopehomeslookupoutward-a-root-link-leaving-the-project-answers-to-its-gate-first)). On version 2 such a link as a kind's home exits `0` when denied, while with the target open config load refuses it with exit `2`, and did before this change: that gate is config load's, which cannot see through the denied directory.
- A root the workspace boundary gives to another project stays that project's.
- The `no snapshot` line beside the failure. Withholding it is a separate rule, tracked in [issue #1215](https://github.com/agent-grounds/grund/issues/1215), and this change only records the failure it needs.

## 3. The correction route

These exits flip a shipped verdict, which [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered) allows only by a named route. [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) is the route, and all five conditions hold.

1. *Prior prohibition.* The old exit `0`, and the old exit `1` that read as a complete report, violated [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips), a separately declared hard requirement: a file the scan cannot read is reported at its path, the run exits `2`, and an incomplete report "never passes as if it were". That requirement already applied when the old verdict shipped. The old runs passed over a home holding a file they never read, where a denied subdirectory of the same home exited `2` on the same build, as [§FS-check.2.4](../../functional-spec/FS-check.md#24-an-incomplete-run) already said of a failure during the scan.
2. *Accepted proof.* This record, which cites that section and proves the conflict above. [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)'s deprecation path does not fit, because there is no old form to carry beside a new one: only a wrong answer, which nothing a user writes selects. [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)'s mechanical migration does not fit either, because no input is at fault for the move and no command can rewrite a directory's mode on anyone's behalf.
3. *Named release.* The release note below names these exits, who notices and what to do.
4. *Actionable findings.* What replaces the old verdict is one failure located at the root, whose text names the action: `data/values: permission denied; make the path readable, or keep it out of the scan`.
5. *No new licence.* This route cannot justify ordinary policy tightening, feature removal or a prohibition invented by this change, and it is used for none of them: the rule that a scan failure exits `2` was already [§FS-check.2.4](../../functional-spec/FS-check.md#24-an-incomplete-run)'s, and this change adds no check.

**The weak point.** [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips) opens on a *file* the scan cannot read, and what failed here is the lookup of a directory, behind which there may be no file at all, as with a home that is a link to itself, which held nothing to miss. The answer is that the run cannot tell: a lookup that fails for a reason other than absence is exactly the case where grund does not know what the home holds, and the clause that an incomplete report never passes as complete does not depend on what turned out to be there. An objector can still hold the looping link to be a harmless empty home; it is reported on descent already ([§FS-config.3.5.5](../../functional-spec/FS-config.md#355-a-link-the-scan-cannot-resolve-is-reported-and-not-scanned-into)), so a root treated otherwise would be the odd one.

## release-note: Release note

- [§FS-scan.scope.homes.lookup](../../functional-spec/FS-scan.md#scopehomeslookup-a-home-that-cannot-be-looked-up-is-a-scan-failure-not-an-absent-home), [§DF-home-lookup-failure-fails-at-the-home](DF-home-lookup-failure-fails-at-the-home.md#df-home-lookup-failure-fails-at-the-home-a-root-whose-lookup-fails-for-a-reason-other-than-absence-fails-at-the-root-and-exits-move-with-it), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids): **a root that cannot be looked up is a scan failure at the root, not an absent root.** A kind's home, a default root or an `include` entry whose lookup fails for a reason other than absence, such as a parent directory without search permission or a link to itself, now prints `error: <root>: <reason>`. **Exits move to `2`:** from `0` for `grund check`, `check --format json`, `list`, `refs` and `fmt --check`, and for `grund show` of an ID elsewhere, where no wider walk met the parent, as on most version 1 configs; from `1` for `grund show` of an ID in that home and for `grund check` on a version 2 config whose parent the walk does not read. Each of those runs had passed over a home it never read. **Who notices:** a gate on a tree holding such a directory. Make the path readable, or keep it out of the scan. A root link going outside the project is still pruned. The verdict moves by the correction route, because the old exits broke [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips). Every finding that replaces the old verdict names its location and the action to take. Closes [issue #1318](https://github.com/agent-grounds/grund/issues/1318).
