# DF-unparsed-ignore-file-at-any-depth: an ignore file the walk cannot parse fails the run at any depth, unless the run does not apply it

**Date:** 2026-11-08

## status: Accepted

## 1. Context

The `ignore` crate reports a line it cannot parse by one of two routes. A walk root below the file meets it through the walk's parents and is handed an error. Any directory the walk enters keeps the failure on that directory's own entry and is handed a success. grund read only the first route, so a malformed ignore file was reported where a kind home happened to lie below it and was dropped everywhere else: a project walked only from its config root exited `0` and never named the file ([issue #1305](https://github.com/agent-grounds/grund/issues/1305), tool report grund.275). The rules of the file went with it, so files the user meant to exclude were read, or files they meant to include were skipped, under a clean report.

## 2. Decision

### 2.1 A file the run applies fails it at any depth

The walk asks every entry for its failure as well, and both routes end in the one walk-error report, so a malformed ignore file is one line at the file and exit `2` wherever it lies and whichever walk met it ([§FS-scan.scope.ignore-files.unparsed](../../functional-spec/FS-scan.md#scopeignore-filesunparsed-an-ignore-file-the-walk-cannot-parse-fails-the-run-unless-the-run-does-not-apply-it), [§FS-check.2.4.2](../../functional-spec/FS-check.md#242-one-failure-one-line)). The depth was never a line in the specification: [§FS-check.2.4](../../functional-spec/FS-check.md#24-an-incomplete-run) already says a run that could not see the whole tree exits `2`.

One malformed line git accepts and globset refuses, an unclosed `{` in a `.gitignore` for instance, is reported like any other. grund reads ignore files through globset, so the rule on that line is not applied, whatever git makes of it. Such a line already exited `2` above a kind home.

### 2.2 A file the run does not apply is no failure

Reporting every malformed file the walker reads would have added false alarms ([§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms)), because the walker parses some files it then does not use. Two such cases are carved out, each decided once, in the report both routes share, so whether a kind home lies below the file cannot change the answer:

- **`exclude` keeps the file out.** A run's `exclude` is the statement of what the project reads, and it already silences an unreadable file or directory below the same pattern. The test is the one the walk asks on the way down to any file ([§FS-config-v2.schema.sources.exclude.roots](../../functional-spec/FS-config-v2.md#schemasourcesexcluderoots-how-exclude-composes-with-the-other-filters)), and a file above the walk root is asked it from the config root. So `exclude = ["vendor/**"]` keeps `vendor/.ignore` from failing the run, and the printed remedy, `keep it out of the scan`, is now true of the file by name: `exclude = [".ignore"]` clears the line.
- **A `.gitignore` the walk does not apply.** The walker applies a `.gitignore` only inside a repository, except under a mounted root ([§FS-remote-projects.ignore](../../functional-spec/FS-remote-projects.md#ignore-ignore-isolation)), and it parses the file either way. Outside a repository no rule of it was going to apply, so nothing about the tree was lost, and exit `2`'s meaning of "do not trust this report as complete" would be false. A source tarball or `git archive` output is the ordinary way to meet this. This also removes an existing exit `2`, for a `.gitignore` above a kind home in such a tree. A `.ignore` is applied with or without a repository, so it still fails the run.

### 2.3 Exits and bytes that move

| Tree | Run | Before | Now |
|---|---|---|---|
| A malformed `.ignore` or a repository's malformed `.gitignore`, in a directory the walk enters, with no kind home below it | `check`, `list`, `fmt --check` | exit `0`, no line | one `error:` line at the file, exit `2` |
| The same | `refs`, `status` | exit `1` | exit `2` |
| A `.gitignore` outside any repository, above a kind home | every command that scans | `error:` line, exit `2` | no line; the run exits on the rest of the tree |
| An ignore file `exclude` keeps out, above a kind home | every command that scans | `error:` line, exit `2` | no line; the run exits on the rest of the tree |
| A malformed `.ignore` at a version 1 cases root linked out of the project | `check` | exit `0`, no line | one `error:` line at the file, spelled under the configured folder, exit `2` |
| A malformed ignore file already reported above a kind home | every command that scans | one line, exit `2` | unchanged |

An ignore file with several bad lines is one line at the file, as [§DF-e2e-case-on-the-one-walk](DF-e2e-case-on-the-one-walk.md#df-e2e-case-on-the-one-walk-an-e2e-case-is-read-by-the-one-walk-so-exclude-and-ignore-files-keep-it-out-and-a-scan-failure-is-one-line-at-the-path-that-failed) settled.

## 3. The correction route

The change flips shipped verdicts, which [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered) allows only by a named route. [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) is the route.

1. *Prior prohibition.* The prior verdict of exit `0` over a malformed ignore file the run applies violated [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips): the run could not see part of its tree and passed as if it could, which [§FS-check.2.4](../../functional-spec/FS-check.md#24-an-incomplete-run) already forbade. The prior verdict of exit `2` over a file the run does not apply violated [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms): a report said to be incomplete was complete. Both sections already applied when each verdict shipped.
2. *Accepted proof.* This record. [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) does not fit, because no old form sits beside a new one: the tree is the same tree and only the answer was wrong. [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) does not fit, because there is nothing to rewrite: a person, not a tool, has to choose which pattern was meant.
3. *Named release.* The release note below.
4. *Actionable findings.* Where `0` or `1` becomes `2`, the line names the ignore file, the line number and the glob, and the maintainer can correct the line or keep the file out with `exclude`, which now clears it. Where `2` becomes lower, nothing replaces the old line: the file plays no part in the run.
5. *No new licence.* The route cannot justify ordinary tightening, feature removal or a prohibition invented here, and it is used for none of them: no rule, check or partition changes, and the walk reads what it always read.

## release-note: Release note

- [§FS-scan.scope.ignore-files.unparsed](../../functional-spec/FS-scan.md#scopeignore-filesunparsed-an-ignore-file-the-walk-cannot-parse-fails-the-run-unless-the-run-does-not-apply-it), [§FS-check.2.4.2](../../functional-spec/FS-check.md#242-one-failure-one-line), [§DF-unparsed-ignore-file-at-any-depth](DF-unparsed-ignore-file-at-any-depth.md#df-unparsed-ignore-file-at-any-depth-an-ignore-file-the-walk-cannot-parse-fails-the-run-at-any-depth-unless-the-run-does-not-apply-it), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips), [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms): **an ignore file the walk cannot parse now fails the run wherever it lies, unless the run does not apply it.** **Exits move up:** a malformed `.ignore`, or a malformed `.gitignore` inside a repository, in a directory the walk enters now exits `2` from `check`, `list` and `fmt --check` where it exited `0`, and from `refs` and `status` where it exited `1`; a line git accepts and globset refuses counts. **Exits move down from `2`:** a `.gitignore` outside any repository, and an ignore file `exclude` keeps out, no longer fail a run, including where a kind home below them used to. **Who notices:** a repository whose ignore file has always had a line that does not parse, and a script counting the walk's failures in a source tarball. Correct the line, or keep the file out with `exclude`. The verdict moves by the correction route of [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), because the old verdicts were a silent skip or a false alarm, and every finding that replaces an old verdict names its location and the action to take. Closes [issue #1305](https://github.com/agent-grounds/grund/issues/1305).
