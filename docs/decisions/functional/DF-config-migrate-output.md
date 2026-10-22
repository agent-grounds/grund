# DF-config-migrate-output: successful migration writes report the outcome; the candidate and mapping report are opt-in

## status: Accepted

## 1. Context

After reviewing a migration preview, a caller applying it needs the outcome and follow-up work. Issue #907's clean 16-line config produces a 166-line candidate and 30 stderr lines on preview; successful `--write` repeats the candidate and 29 report lines. The qualified `run grund init` reminder ends at line 34. Reading or trimming that repetition costs tokens and hides the next action ([§GOAL-token-economy](../../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file)).

## 2. Decision

Only an actual successful write defaults to the compact report of [§FS-config-migrate.6.5](../../functional-spec/FS-config-migrate.md#65-what-it-says-after-the-write) and empty stdout ([§FS-config-migrate.2.1](../../functional-spec/FS-config-migrate.md#21-stdout-is-the-candidate)). Migration-only `--full` restores the candidate and complete report ([§FS-config-migrate.1.3](../../functional-spec/FS-config-migrate.md#13-the-other-flags), [§FS-config-migrate.2.2](../../functional-spec/FS-config-migrate.md#22-stderr-is-the-report)). Previews, refusals, failures and no-ops retain their behavior. Presentation never changes candidate bytes, gates, comment authorization or config-only atomic writing. No automatic init or managed-block drift detection is added.

## 3. Compatibility and alternatives

The previous successful-write stdout and report are covered user-visible promises under [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered). This deliberately changes those bytes: a script capturing `grund config migrate --to 2 --write` must add `--full` to keep its prior output. Preview captures remain unchanged.

Apply the argued pre-1.0 licence of [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise), following [§DF-show-default-token-cheap.4](DF-show-default-token-cheap.md#4-consequences): that decision made the cheap read the default and kept `--full` for the former output. Here carrying the old default beside a compact flag and a release-long warning would prolong the measured replay on every ordinary write and bury the actionable advice. The mechanical replacement costs affected callers one flag, preserves the complete legacy output, and changes no verdict or migration policy. That cost is smaller than retaining a legacy default through a release window; this argument applies before 1.0 and grants no general exemption.

The ordinary deprecation route of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) was considered and rejected for that specific cost. Moving the reminder alone leaves the duplicated candidate and mapping report. Suppressing stdout alone leaves repeated stderr detail. Removing the complete output would deny callers a supported replacement. The issue's tokens label alone authorizes none of these compatibility choices; the author approved this concrete design on 2026-10-22.

## release-note: Release note

- A successful `grund config migrate --to 2 --write` no longer prints the candidate or mapping report; `--full` brings both back.
