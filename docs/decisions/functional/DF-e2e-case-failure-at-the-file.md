# DF-e2e-case-failure-at-the-file: a failure reading an E2E case is located at the file that failed, and path-scoped exits move with it in both directions

**Date:** 2026-11-07

## status: Accepted

## 1. Context

Every failure of the pass over the E2E cases was published at one location, the literal `e2e/cases` under the project root, whatever failed and wherever the kind's folder was. A path-scoped run decides whether a failure belongs to it by asking whether its path contains the failure's location ([§FS-check.1.3.6.3](../../functional-spec/FS-check.md#1363-a-file-the-wider-walk-could-not-read-is-said-out-loud)), and no path below the cases root contains its root. So `grund check e2e/cases/b` over a `b` holding the bytes `ff fe` in `expected.exit` exited `0` and called the failure outside the report scope, on a run whose own command line named the case. With the kind's folder elsewhere the location was not in the tree at all, so even the whole cases root was told the failure lay outside, and a directory that merely had that name was told the failure lay inside. Reported as [issue #1264](https://github.com/agent-grounds/grund/issues/1264); [§DF-e2e-case-read-failure-fails-alone.2.3](DF-e2e-case-read-failure-fails-alone.md#23-what-does-not-move) recorded the first of these as a defect it left standing.

## 2. Decision

A failure reading an E2E case is located at the file or directory that failed ([§FS-declarations.e2e-case.failure](../../functional-spec/FS-declarations.md#e2e-casefailure-a-case-that-cannot-be-read-fails-alone)), as the tree spells it under the configured folder. Nothing about the partition changes: [§FS-check.1.3.6.3](../../functional-spec/FS-check.md#1363-a-file-the-wider-walk-could-not-read-is-said-out-loud) was right, and the location it was handed was wrong.

### 2.1 Why the file

Each candidate location was measured with the report's reproducer, on a copy of the code changed only there, over its 36 judgements of a run's exit and of the word "outside".

| Location | Judgements failing |
|---|---|
| The file that failed, partition untouched | 0 |
| The case directory | 4: the run naming the file itself, from the root and from inside the case |
| The configured folder | 27, `grund check e2e/cases/b` among them |
| Unchanged, every case failure kept in scope | 4: the two runs naming a healthy case |

The location is also what makes the other three questions answerable at all: a whole-project run names the file to repair, two failing cases read as two different failures, and a line names a path the tree has.

A file that failed does not hide another. A case read stopped at its first failure, which a location at the file would have turned into the same green exit one file down: over a case with an undecodable `command.args` and `expected.exit`, `grund check e2e/cases/b/expected.exit` would have called the first outside and said nothing of the file it was asked about. So every file and directory of a failing case is attempted and each failure reported.

### 2.2 Exits move in both directions

Measured on the report's tree: cases `a` and `c` healthy and `b` holding `ff fe` in `expected.exit`, with `e2e/cases` as the folder unless the row says otherwise. "Before" is `grund 0.21.0`. Each `error:` line below ends `; make the path readable, or keep it out of the scan`, and each caution ends `— outside the report scope, so a citation declared there may be reported as unresolved`.

| Run | Before | Now |
|---|---|---|
| `grund check e2e/cases/b` | `warning: e2e/cases: stream did not contain valid UTF-8 — …`, exit `0` | `error: e2e/cases/b/expected.exit: stream did not contain valid UTF-8; …`, exit `2` |
| `grund check e2e/cases/b/expected.exit`, or from inside the case `grund check .` and `grund check expected.exit` | the same caution, exit `0` | the same `error:` line, exit `2` |
| Workspace member `m`: `grund check m/e2e/cases/b` from the root, `grund check e2e/cases/b` from the member | the same caution, exit `0` | the same `error:` line, exit `2` |
| Folder `tests/e2e`: `grund check tests/e2e/b` | the same caution, exit `0` | `error: tests/e2e/b/expected.exit: …`, exit `2` |
| Folder `tests/e2e`, `a` indexed: `grund check tests/e2e`, `grund check tests` | the same caution, exit `0` | the same `error:` line, exit `2` |
| Folder `tests/e2e`, `a` not indexed: `grund check tests/e2e`, `grund check tests` | the same caution and `a`'s finding, exit `1` | the `error:` line and `a`'s finding, exit `2` |
| Folder `tests/e2e`, a real `e2e/` directory beside it: `grund check e2e` | `error: e2e/cases: stream did not contain valid UTF-8; …`, exit `2` | `warning: tests/e2e/b/expected.exit: … — …`, and the directory's own findings: exit `0`, or `1` where it has one |
| A real `e2e/cases/` directory with the folder at `tests/e2e`: `grund check e2e/cases` | as above, exit `2` | as above, exit `0` or `1` |

In general: a run's exit reaches `2` exactly where its path holds the file that failed, and every run that holds the file exits `2` whatever it printed before. A run that held only the old location, which takes the folder configured elsewhere and a directory of that name, loses the `2` and keeps the caution. A manifest at mode `000`, a non-UTF-8 `command.args` or `spec.refs`, and an `expected.exit` that does not parse move as `ff fe` does. A fixture directory at mode `000` already exited `2` from the walk's own line, and loses only its second line calling the same failure outside ([§FS-check.2.4.2](../../functional-spec/FS-check.md#242-one-failure-one-line)).

### 2.3 What moves in bytes only

A whole-project run still exits `2`, and its line and `check --format json`'s `path` name the file instead of `e2e/cases`. So does a binding's `details.scan_errors` entry, and any other command that reports the scan failure. Two failing cases stop reading as two identical lines. A run naming a healthy case keeps its caution and its exit, and the caution names the file. The editor's report moves as `check`'s does, for a snapshot anchored at a path ([§AR-lsp.5.1.1](../../architecture/AR-lsp.md#511-a-path-anchor-narrows-the-report-not-the-resolution)). A case with two failing files now reports both.

### 2.4 What does not move

- The reason's own text, the `parse <absolute path>/expected.exit: …` of a value that does not parse among them, and the `io` code and remedy it is reported under ([#1272](https://github.com/agent-grounds/grund/issues/1272)).
- A run that physically holds the file through another spelling. With `e2e/cases` a link to `real`, or `e2e/cases/b` a link to `elsewhere/b`, `grund check real/b` and `grund check elsewhere/b` still exit `0` and caution, at the configured spelling, because a failure is one location and the partition compares paths ([§FS-check.2.4.2](../../functional-spec/FS-check.md#242-one-failure-one-line)). `grund check e2e/cases/b` exits `2` there.
- `grund list`, `grund show` and `refs` exit `2` on such a tree as before: they do not partition.
- `grund cover e2e/cases/b/expected.exit` exits `0` and reads no case, as it did. `cover` bounds its scan to its path ([§FS-scan.path.cover](../../functional-spec/FS-scan.md#pathcover-cover-runs-one-narrowed-scan)) and the pass reads nothing for a path naming a file, so that run has no failure to locate. `grund cover e2e/cases/b` exits `2` and names the file.
- Moving the pass onto the one walk and whether `exclude` reaches a case ([#1271](https://github.com/agent-grounds/grund/issues/1271), [#1255](https://github.com/agent-grounds/grund/issues/1255)).

## 3. The correction route

These exits flip a shipped verdict, which [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered) allows only by a named route. [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) is the route, and all five conditions hold.

1. *Prior prohibition.* The old verdicts violated [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips), which says "a file the scan cannot read is reported at its path and the run exits `2`", and that the report "never passes as if it were" complete. The old exits `0` and `1` of a run whose path held the failing file broke both clauses: the file was reported at another path and the run passed, or failed only on a finding, as if it had read the file it was asked about. The old exit `2` of a run that did not hold the file broke the first: the only reason it could exit `2` is that the failure was reported at a path that was not the file's, and its line told its reader that a directory holding nothing wrong had failed. That section has been in every release since `v0.10.1`, and the project-wide read that hands a narrowed run these failures first shipped in `v0.15.0`, so the prohibition already applied when each verdict shipped.
2. *Accepted proof.* This record, which cites that section and proves the conflict above. [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)'s deprecation path does not fit, because there is no old form to carry beside a new one: nothing a user writes selects a location. [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)'s mechanical migration does not fit either, because nothing a user wrote is at fault, so there is no command to run on anyone's behalf.
3. *Named release.* The release note below names each of these exits, who notices and what to do.
4. *Actionable findings.* Where `0` or `1` becomes `2`, what replaces the old verdict is the `error:` line naming the file, the reason it failed and the remedy, which is what the same bytes in an ordinary file print. Where `2` becomes lower, nothing replaces it: the false line goes, the caution names the real file and the run's own findings stand.
5. *No new licence.* This route cannot justify ordinary policy tightening, feature removal, or a prohibition invented by this change, and it is used for none of them: no check, rule or partition changes, and a run reaches `2` only for the file [§FS-check.1.3.6.3](../../functional-spec/FS-check.md#1363-a-file-the-wider-walk-could-not-read-is-said-out-loud) always said it reaches `2` for.

**The weak points.** The old runs did print a warning. For `0` and `1` that is no defence, because the warning said the opposite of the truth, that a failure on the command line lay outside the run.

The direction down rests on less. [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips) forbids mislocating the failure and does not forbid a run exiting `2`; a run that exits `2` is the conservative answer, and the tree really did hold a failure. The exit moves because it has no other ground: [§FS-check.1.3.6.3](../../functional-spec/FS-check.md#1363-a-file-the-wider-walk-could-not-read-is-said-out-loud) grants `2` "only for a file at or under the path", and to keep it would take a rule that a directory named `e2e/cases` or `e2e` stays red for a file under `tests/e2e`. An objector can hold that a false red harms nothing. This record refuses that, on the reading [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms) gives a checker that cries wolf, without claiming that section reaches a line about no citation.

The remedy is a weak point of its own for a value that does not parse: `make the path readable` is not what `not-an-exit` needs, and `keep it out of the scan` is not on offer for a case while `exclude` does not reach one. The reason names the file's own fault, so the location and the action still stand, and the wording is [#1272](https://github.com/agent-grounds/grund/issues/1272)'s and [#1255](https://github.com/agent-grounds/grund/issues/1255)'s to correct.

## release-note: Release note

- [§FS-declarations.e2e-case.failure](../../functional-spec/FS-declarations.md#e2e-casefailure-a-case-that-cannot-be-read-fails-alone), [§DF-e2e-case-failure-at-the-file](DF-e2e-case-failure-at-the-file.md#df-e2e-case-failure-at-the-file-a-failure-reading-an-e2e-case-is-located-at-the-file-that-failed-and-path-scoped-exits-move-with-it-in-both-directions), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips): **a failure reading an E2E case is located at the file that failed, so path-scoped exits move in both directions.** A `command.args`, `expected.exit` or `spec.refs` that cannot be read or decoded, an `expected.exit` that does not parse, and a fixture directory that cannot be listed are reported at that path, under the kind's configured folder, rather than at a fixed `e2e/cases`, and every one a case holds is reported. **Exits move to `2`:** `grund check <path>` exits `2` where the path holds the failing file, whether it names the file, its case, the cases root or a directory above it, where it passed at `0` or stopped at `1`. Make the file readable, or repair its contents. **Exits move down from `2`:** with the kind's folder elsewhere, `grund check e2e` or `grund check e2e/cases` over a directory that happens to have that name now answers about that directory, `0` or `1`, and cautions about the file that failed. **Who notices:** a hook or CI step narrowed to paths in a tree holding a bad case, and a script matching the line or `check --format json`'s `path` on `e2e/cases`, which now holds the file. A run naming a healthy case keeps its caution and its exit, and the fixture directory's second line, which called the walk's own failure outside, goes. The verdict moves by the correction route of [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), because a file the scan cannot read is reported at its path under [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips), and every finding that replaces the old verdict names its location and the action to take. Closes [issue #1264](https://github.com/agent-grounds/grund/issues/1264).
