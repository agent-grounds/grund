# DF-scan-failure-code-and-fix: A scan failure carries its own code, and a fix that works at its path

**Date:** 2026-11-09

Gives a file that was read and does not parse the code `unparsable-file`, and `io`'s *Fix* one case per place, as [§FS-check.2.4.3](../../functional-spec/FS-check.md#243-what-failed-and-where-decide-the-code-and-the-fix) lays out, and names a second exception in [§FS-errors.requirements.prefix-windows](../../functional-spec/FS-errors.md#requirementsprefix-windows-a-wording-change-keeps-the-old-message-whole-until-its-window-closes--realized-directional-for-the-next-change). Serves [§GOAL-friendliness-first.1](../../goals.md#1-hard-requirements): a failure line is what a user follows next, so what it tells them has to work where it is printed. Reported as [grund#1272](https://github.com/agent-grounds/grund/issues/1272), with [#1251](https://github.com/agent-grounds/grund/issues/1251) and [#1263](https://github.com/agent-grounds/grund/issues/1263) fused into it.

## status: Accepted

## 1. Context

A scan failure was a path and a string, so everything was published as `io` with `io`'s one remedy, `make the path readable, or keep it out of the scan`. Two cases made that remedy false. A readable `expected.exit` holding `not-an-exit`, or a value source holding `{ not json`, is told to make a readable path readable. And the second half of the sentence cannot work at the paths `exclude` does not reach: a scan root ([§FS-scan.scope.roots](../../functional-spec/FS-scan.md#scoperoots-a-scan-root-outruns-every-rule-about-descent)), a path the command is given, a value source, a section lock. The user who follows it prints the same line again.

## 2. Decision

The record says what failed ([§FS-check.2.4.3](../../functional-spec/FS-check.md#243-what-failed-and-where-decide-the-code-and-the-fix)), and the catalog's *Fix* cell says what can work there ([§FS-errors.5.5](../../functional-spec/FS-errors.md#55-the-check-code-catalog)).

- **One code for a read-and-wrong file.** A file that was read and does not parse is `unparsable-file`, one code with one case per file kind, because a script's question is one question: was this file read and found wrong?
- **Exit `2` stays.** A file that did not parse is a hole in what the run read, which is what `2` means ([§FS-check.2.4](../../functional-spec/FS-check.md#24-an-incomplete-run)); [§FS-values.5.3](../../functional-spec/FS-values.md#53-incomplete-input-and-deterministic-output) already says so for value sources. Whether a tree passes does not move.
- **`io` keeps today's words where they work.** Case `entry` keeps today's bytes exactly. At `root`, `passed` and `value` the remedy keeps today's words whole and appends how, naming why `exclude` does not reach there. Bytes appended after a surviving prefix are no change at all ([§FS-errors.3](../../functional-spec/FS-errors.md#3-message-text)).

Two families of line cannot keep their prefix, and they move in place in 0.23.0, with no window, under the pre-1.0 licence of [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise), as [§DF-stale-section-differs-in-0-20](DF-stale-section-differs-in-0-20.md#df-stale-section-differs-in-0-20-the-stale-block-finding-names-its-section-as-one-subject-corrected-in-place-in-0200) moved the first:

- The parse lines moving to `unparsable-file`. The old remedy tells a user to make a readable file readable, which does nothing, so a window would keep printing it on purpose to exactly the users this decision is about.
- The `readable` lines, losing `, or keep it out of the scan`. A section lock or the config root has no row to remove it from and no walk that reaches it, so those words do not work there.
- 0.23.0 is the last release before 1.0 where this can happen at all: from 1.0 a wording change must keep its prefix through a window.

Consumers are not stranded: `code` is the contract, no consumer acts on `message` ([§FS-errors.requirements.fields](../../functional-spec/FS-errors.md#requirementsfields-what-a-program-needs-from-a-finding-is-a-field--realized-directional-for-the-next-finding)), and a script filtering on the new code can tell the two apart for the first time. The five-condition route of section 5 of `REQ-backwards-compatibility` does not apply, because every one of these runs exits `2` before and after, so no verdict moves.

## 3. Consequences

- A script filtering `code == "io"` no longer sees parse failures; one grepping `make the path readable` no longer matches a parse line.
- `root`, `passed` and `value` lines gain words at the end and lose none.
- `--ignore unparsable-file` is accepted and hides nothing, as `--ignore io` behaves ([§FS-check.1.4](../../functional-spec/FS-check.md#14-selecting-findings-with---only-and---ignore)).
- The caution of [§FS-check.1.3.6.3](../../functional-spec/FS-check.md#1363-a-file-the-wider-walk-could-not-read-is-said-out-loud) wears the code of the failure behind it, so a parse failure outside the report scope is `unparsable-file` there too.
- `ApiScanError` and the Python and Node records gain `code`, one field and no new public type ([§AR-bindings.2](../../architecture/AR-bindings.md#2-grund-core-the-only-place-logic-lives)).

## 4. Alternatives considered

| Approach | Why rejected |
|---|---|
| Keep the tuple and tell failures apart by their text | That is the classification [§AR-bindings.2](../../architecture/AR-bindings.md#2-grund-core-the-only-place-logic-lives) forbids, and [§FS-check.2.4.1](../../functional-spec/FS-check.md#241-one-reason-for-one-scan-io-failure) forbids rewriting a finished message. |
| One code per file kind | The script's question is one question; the *Fix* cases already say what is true for each file kind. |
| Exit `1` for a parse failure | It would call an unread file a finding about the graph, and it would move a verdict for no gain. |
| Replace the remedy at `root`, `passed` and `value` outright | "Keep it out of the scan" is true there once it says how, so the prefix can survive and the change needs no exception. |
| Keep the old line whole for one release at the parse and `readable` lines | It shows a remedy that does not work to the users it misleads, and consumers key on `code`. |

## release-note: Release note

- [§FS-check.2.4.3](../../functional-spec/FS-check.md#243-what-failed-and-where-decide-the-code-and-the-fix): **a scan failure says what failed, and offers a fix that works where it failed.** A file that was read and does not parse is now reported as `unparsable-file` rather than `io`, still exit `2`, with a fix that says what the file should hold, for example ``write the case's expected exit status in the file as one integer``. At a scan root, a path passed on the command line and a value source, `io`'s remedy keeps its words and now says how, for example ``make the path readable, or keep it out of the scan by removing it from the [schema.kinds] row that lists it, because exclude does not prune scan roots``. A section lock reads ``make the path readable``. **Who this affects:** a script filtering on `code == "io"`, or matching the remedy text in a parse or section-lock line. Filter on `code` and treat both `io` and `unparsable-file` as an incomplete run. Exit statuses are unchanged. `ApiScanError` and the Python and Node scan-failure records gain `code`. Closes [issue #1272](https://github.com/agent-grounds/grund/issues/1272).
