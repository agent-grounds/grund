# DF-bare-grund-lands-on-an-error: bare `grund` lands on a CLI-level error, not the top-level help page

**Status:** Accepted
**Date:** 2026-09-27

## 1. Context

[§FS-cli.1](../../functional-spec/FS-cli.md#1-the-default-subcommand)'s no-argument fallback ran `grund check .` behind a warning, through the named window [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)'s deprecation path owes. Removing it leaves one question the window never answered: what bare `grund` does once it no longer checks.

Two landings were already shapes this CLI has, so neither is an invention. The top-level help page on stdout with exit `0` ([§FS-cli.2.2](../../functional-spec/FS-cli.md#22-the-top-level-help-page)) is one branch over in the same dispatch, and it is what the project *published*: `docs/changelog/0.3.0.md` says, in its Changed section, "`grund` with no arguments prints the top-level help instead of running a check on `.`", and its Removed section says "bare `grund` no longer runs `grund check .`". Both sentences have been false since they were written — the window kept the old behavior for eleven releases — and the archived section is not corrected, because a released changelog moves verbatim. A CLI-level error on stderr with exit `2` ([§FS-cli.4](../../functional-spec/FS-cli.md#4-errors-with-no-source-location)) is the other, and is the shape several goldens already pin.

## 2. Decision

Bare `grund` is a CLI-level error: `error:` and two `hint:` lines on stderr, empty stdout, exit `2`, in the exact bytes [§FS-cli.4](../../functional-spec/FS-cli.md#4-errors-with-no-source-location) gives. The help page is not the landing.

The reason is [§FS-cli.1.1](../../functional-spec/FS-cli.md#11-why-these-defaults)'s own, the only sentence in the specification that states a *purpose* for the fallback: bare `grund` kept checking "so that old CI scripts do not turn into a successful no-op". A CI step still spelled `grund` that prints help and exits `0` is precisely that no-op — and silently, because exit `0` is what it got before. Under the error it goes red on the upgrade and the message names `grund check`. That is the outcome [§GOAL-no-silent-breakage](../../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path) calls a release blocker, so the cheaper landing is cheaper in specification surface, not in risk.

## 3. Consequences

- The behavior is specified at [§FS-cli.4](../../functional-spec/FS-cli.md#4-errors-with-no-source-location) rather than at a new point under [§FS-cli.1](../../functional-spec/FS-cli.md#1-the-default-subcommand): it goes through the existing CLI-level error mechanism ([§FS-errors.2.2](../../functional-spec/FS-errors.md#22-cli-level-message)), so no new printer, flag or exit path exists, and [§FS-cli.1](../../functional-spec/FS-cli.md#1-the-default-subcommand)'s first bullet is one line pointing there.
- [§FS-cli.5](../../functional-spec/FS-cli.md#5-exit-code-mapping-is-fixed) does not move. It freezes the *mapping* from a report to an exit code; bare `grund` moves between two codes that mapping already defines.
- `grund .` — a path with no subcommand — stays the failed ID query at exit `1` of [§FS-cli.1.2](../../functional-spec/FS-cli.md#12-a-first-word-that-is-not-an-id). This landing claims only a genuinely empty argument list.
- `--help`, `-h`, `help`, `--version` and `-V` short-circuit before the no-argument case and are unchanged ([§FS-cli.2](../../functional-spec/FS-cli.md#2-global-flags)).
- The error names the release the removal was made in, which is the landed tense of the clause the warning wrote in the pending one ([§FS-distribution.4.2.3](../../functional-spec/FS-distribution.md#423-the-vocabulary-is-closed)), so both halves of the removal read as one claim.
- What `docs/changelog/0.3.0.md` announced is now true of the removal and false of the landing: bare `grund` no longer runs `grund check .`, and it does not print the help page either. This record is where a reader who finds that archive first learns which half survived.

## 4. Alternatives considered

| Approach | Why rejected |
|---|---|
| The top-level help page, exit `0` — what `docs/changelog/0.3.0.md` published | A CI step spelled `grund` goes green having validated nothing, which is the one outcome [§GOAL-no-silent-breakage](../../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path) treats as a release blocker, and is the no-op [§FS-cli.1.1](../../functional-spec/FS-cli.md#11-why-these-defaults) says the window was bought to prevent. Cheaper in specification surface — it extends a point seven sites lean on and needs no new case — and it would make an eleven-release-old changelog true; neither buys back a silent green pipeline. |
| Keep the fallback and drop only the warning | Leaves the bare form as an undocumented alias for `grund check .` forever, so the deprecation path named a release and then did not use it. |
| Exit `1` rather than `2` | `1` is a finding or a failed query ([§FS-cli.5](../../functional-spec/FS-cli.md#5-exit-code-mapping-is-fixed)); nothing was scanned and no query was made. A CLI launch failure is `2`. |

## release-note: Release note

- [§FS-cli.1](../../functional-spec/FS-cli.md#1-the-default-subcommand), [§FS-cli.4](../../functional-spec/FS-cli.md#4-errors-with-no-source-location), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **bare `grund` no longer runs `grund check .`.** The deprecation window the warning named closes on the release it named, so the path's worked example is now a path performed end to end: `grund` with no arguments is a CLI-level error — empty stdout, exit `2`, and on stderr ``error: no command given; the bare `grund` fallback that ran `grund check .` was removed in grund 0.16.0`` followed by two `hint:` lines naming `grund check` and `grund --help`. **Who this breaks:** a CI step still spelled `grund`, which was green having validated the repository and is now red having validated nothing; the line to write instead is `grund check`, which is what the error says. `grund check`, `grund <ID>`, `grund --help`, `grund --version` and `grund .` are untouched, and [§FS-cli.5](../../functional-spec/FS-cli.md#5-exit-code-mapping-is-fixed)'s report-to-exit mapping does not move — bare `grund` moves between two codes that mapping already defines. Why the landing is an error rather than the top-level help page, which would be stdout and exit `0` and so the successful no-op the window was bought to prevent, is [§DF-bare-grund-lands-on-an-error](DF-bare-grund-lands-on-an-error.md#df-bare-grund-lands-on-an-error-bare-grund-lands-on-a-cli-level-error-not-the-top-level-help-page). What `docs/changelog/0.3.0.md` announced eleven releases ago is now true of the removal and false of the landing — bare `grund` no longer runs `grund check .`, and it does not print the top-level help either; that archived section is left as it stands. Closes [issue #301](https://github.com/agent-grounds/grund/issues/301). (PR #331)
