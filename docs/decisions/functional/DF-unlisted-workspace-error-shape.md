# DF-unlisted-workspace-error-shape: `check` and the editor take the located shape, the five walking surfaces keep the CLI-level line

**Status:** Accepted
**Date:** 2026-09-24

## 1. Context

The unlisted-`[workspace]` finding shipped in `0.13.0` as a warning whose own text named the release it would become an error in, and `0.15.0` is that release ([§FS-check.3.29.14](../../functional-spec/FS-check.md#32914-an-error-because-the-deadline-the-warning-named-has-arrived)). The ramp decided the *verdict*: exit `0` becomes exit `1`, and [§DF-unlisted-workspace-block.2.1](DF-unlisted-workspace-block.md#21-a-warning-that-names-the-release-it-becomes-an-error-in) argued that two releases ago. It did not decide the *shape*, because a warning had only one to take.

An error has two, and they are not interchangeable. [§FS-check.2.1](../../functional-spec/FS-check.md#21-report-format) makes the bare `<path>:<line>:` prefix mandatory on a per-site finding, and [§FS-errors.2.2.2](../../functional-spec/FS-errors.md#222-exit-codes) makes a leading `error:` on stderr mean exit `2` everywhere but `grund config validate`. So a finding that exits `1` and prints `error:` on stderr would be a second named exception to the one rule that keeps the two streams legible. But the shape the finding has today is exactly the CLI-level line [§DF-unlisted-workspace-block.2.4](DF-unlisted-workspace-block.md#24-one-shape-on-every-surface) chose *on purpose*, over a located finding, so that one grep would match across all six commands that report the fact.

Two further things came due with the flip, and both are shape questions rather than verdict ones. [§DA-engine-renders-nothing.4](../architectural/DA-engine-renders-nothing.md#4-consequences) left the JSON object's `path` and `line` `null` although the anchor existed structurally, saying the fields were "a change of its own, for a release that names it". And [§FS-lsp.4](../../functional-spec/FS-lsp.md#4-determinism-and-parity-with-the-cli) requires the editor and the terminal to say the same thing byte for byte, so whichever channel `check` uses is the channel the editor has to read.

## 2. Decision

### 2.1 In `check` and in the editor, the located shape

In `check` the finding is one of the report's errors, located at the block's `[workspace]` line: on **stdout** behind the `<path>:<line>: error:` prefix, and under `--format json` one diagnostic on stdout whose `path` is the block's own config, whose `line` is that line, and whose `sites` is `null` ([§FS-check.3.29.13](../../functional-spec/FS-check.md#32913-in-check-one-of-the-reports-errors)). The location leaves the message text, because the prefix now carries it ([§FS-check.3.29.7](../../functional-spec/FS-check.md#3297-the-location-sits-in-the-prefix-where-the-finding-is-located-and-inside-the-text-where-it-is-not)). The editor publishes the same finding, from `check`'s report rather than from the run's warning channel, as an error at the same anchor ([§FS-lsp.1.1.3](../../functional-spec/FS-lsp.md#113-workspace-warnings-on-the-runs-warning-channel)).

Three reasons, and the first is the one that decides it. **An error that does not look like an error is worse than an inconsistency between commands.** [§GOAL-friendliness-first.1](../../goals.md#1-hard-requirements) opens "Errors point at `path:line`"; every other finding in [§FS-check.3](../../functional-spec/FS-check.md#3-errors-detected) wears the prefix, and a reader who has learned that a located line on stdout is a thing to fix would meet one exception. **It needs no new exception to the stream rule.** The CLI-level alternative below buys its one grep with a second exit-`1` `error:` on stderr, and [§FS-errors.2.2.2](../../functional-spec/FS-errors.md#222-exit-codes)'s value is that it has almost none. **And it reuses what is already built**: `check` routes a diagnostic by whether it carries a line, so the prefix, the stream and the JSON object all follow from the finding having one, with no new printer, flag or JSON path.

### 2.2 The five walking surfaces keep the CLI-level line

`list`, `refs`, `cover`, `fmt` and the ID read of [§FS-show](../../functional-spec/FS-show.md#fs-show-grund-reads-a-single-declaration-body-by-id) keep the `warning:` line on stderr they print today, with the location inside its text and one word of tense changed ([§FS-check.3.29.6](../../functional-spec/FS-check.md#3296-the-message)).

They have no error channel for a fact about the run, and [§FS-cli.5](../../functional-spec/FS-cli.md#5-exit-code-mapping-is-fixed) freezes their exit codes. Flipping them too would mean giving five read commands a new way to fail, which is a much larger change than the one the ramp promised — and the guarantee this finding protects is gated in `check`, which is the run CI does ([§FS-check.3.29.9](../../functional-spec/FS-check.md#3299-every-command-that-walks-reports-it-not-check-alone)).

### 2.3 [§DF-unlisted-workspace-block.2.4](DF-unlisted-workspace-block.md#24-one-shape-on-every-surface) is superseded for those two surfaces and stands for the other five

Not rewritten in place. What 2.4 argued is that *one shape was right for a warning*, and that argument is still right for the five commands that still print one: their text is unchanged, and a consumer grepping any of them keeps what it had. What changed underneath it is that one of the six surfaces stopped carrying a warning at all.

Rewriting 2.4 would lose the reason, which is worth more than the verdict: a reader who finds the two shapes and wants to know why the divergence was once refused should be able to read the refusal.

The cost 2.4 named is real and is now paid. The text a consumer greps for depends on which command produced it: `check` prints `b/grund.toml:7: error: this [workspace] is …` on stdout, the five print `warning: b/grund.toml:7: this [workspace] is …` on stderr. The finding's `code` is the same on both, and `code` is the stable identifier a machine should be matching ([§FS-errors.5.5](../../functional-spec/FS-errors.md#55-the-check-code-catalog)); the divergence bites a text grep and not a JSON consumer.

### 2.4 The JSON move is argued under [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise), not carried by the ramp

The ramp covers the verdict half. It does not cover the rest, and the rest is a covered JSON surface ([§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered)) moving with no alias available: the object changes stream (stderr → stdout) and `severity` (warning → error), `path` and `line` go from `null` to populated, and the message loses its leading `<path>:<line>: `.

There is no alias to carry. A JSON diagnostic has no second spelling, and emitting the object twice — once in each shape — would tell a consumer the finding fired twice. So this takes the pre-`1.0` licence of [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise), argued rather than assumed, which is what that section requires of it:

- **What breaks.** A consumer that grepped **stderr** for this warning stops seeing it under `check` — the one seam that bites silently, since nothing appears on the stream it was reading. One that parses the location out of the message text gets it structurally instead, which is the improvement [§DA-engine-renders-nothing.4](../architectural/DA-engine-renders-nothing.md#4-consequences) deferred. One that matches the leading text prefix stops matching.
- **What it would cost to keep.** Keeping `path` and `line` `null` on an error means keeping the location in the text, which means keeping the CLI-level shape, which is [§DF-unlisted-workspace-error-shape.2.1](DF-unlisted-workspace-error-shape.md#21-in-check-and-in-the-editor-the-located-shape)'s rejected alternative. The nulls are not separable from the shape.
- **Why it is proportionate.** `grund` is pre-`1.0`, the finding fires only on a repository that has an unlisted block, and such a repository is having its exit code flipped in the same release by the ramp it was told about twice. A JSON consumer reading a run that has just started failing is re-reading it anyway.

## 3. Consequences

`check`'s report gains a finding class it did not carry, and no type changes: `CheckOutput.report.errors` holds it where `report.warnings` did, and the code is unchanged, so exhaustive matching over codes is unaffected. `--ignore unlisted-workspace-block` still suppresses the finding, and now suppresses the exit code with it ([§FS-check.1.4](../../functional-spec/FS-check.md#14-selecting-diagnostics-with---only-and---ignore)).

The editor takes it from the report, which means the parity sweep compares it as an ordinary located finding on both surfaces rather than holding it against a stderr golden ([§FS-lsp.4.1](../../functional-spec/FS-lsp.md#41-how-parity-is-held)). That is the assertion that would catch the two surfaces drifting, and it is worth naming because this is the one finding whose location has two places it could come from.

Nothing moves for a repository whose tree is clean, for the `success` marker (a warning already displaced it), or for `grund config validate`, which is still deliberately silent about this block ([§DF-unlisted-workspace-block.3](DF-unlisted-workspace-block.md#3-consequences)).

## 4. Alternatives considered

| Option | Why rejected |
|---|---|
| **Keep the CLI-level `error:` line on stderr, and only fill in `path`/`line` in JSON.** One shape across all six commands, as [§DF-unlisted-workspace-block.2.4](DF-unlisted-workspace-block.md#24-one-shape-on-every-surface) decided, and the ticket's original question answered on its own. | It needs a second named exit-`1` exception to [§FS-errors.2.2.2](../../functional-spec/FS-errors.md#222-exit-codes)'s `error:`-means-`2` rule, and it produces an error that looks like no other [§FS-check.3](../../functional-spec/FS-check.md#3-errors-detected) error. It also does not actually keep one shape: the JSON object would carry a location the text still repeats. |
| **Flip all six surfaces to errors.** No divergence at all. | Five read commands gain a way to fail, [§FS-cli.5](../../functional-spec/FS-cli.md#5-exit-code-mapping-is-fixed) freezes their exit codes, and the ramp promised `check`'s verdict rather than theirs. A much larger change than the one announced. |
| **Keep `path` and `line` `null` on the error.** No JSON schema move, so no [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise) argument owed. | Not separable from the shape: a lineless diagnostic is routed to stderr without a prefix, so the nulls *are* the CLI-level shape. It also keeps the location only in prose, on the one surface where the anchor is what a reader needs — the defect the ticket opened on. |
