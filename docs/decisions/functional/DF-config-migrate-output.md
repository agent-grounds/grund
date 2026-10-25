# DF-config-migrate-output: successful migration writes report the outcome; the candidate and mapping report are opt-in

## status: Accepted

## 1. Context

After reviewing a migration preview, a caller applying it needs the outcome and follow-up work. Issue #907's clean 16-line config produces a 166-line candidate and 30 stderr lines on preview; successful `--write` repeats the candidate and 29 report lines. The qualified `run grund init` reminder ends at line 34. Reading or trimming that repetition costs tokens and hides the next action ([§GOAL-token-economy](../../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file)).

## 2. Decision

Only an actual successful write defaults to the compact report of [§FS-config-migrate.6.5](../../functional-spec/FS-config-migrate.md#65-what-it-says-after-the-write) and empty stdout ([§FS-config-migrate.2.1](../../functional-spec/FS-config-migrate.md#21-stdout-is-the-candidate)). Migration-only `--full` restores the candidate and complete report ([§FS-config-migrate.1.3](../../functional-spec/FS-config-migrate.md#13-the-other-flags), [§FS-config-migrate.2.2](../../functional-spec/FS-config-migrate.md#22-stderr-is-the-report)). That successful-write change left previews, refusals, failures and no-ops alone; the later refusal wording adjustment is defined below. Presentation never changes candidate bytes, gates, comment authorization or config-only atomic writing. No automatic init or managed-block drift detection is added.

## 3. Compatibility and alternatives

The previous successful-write stdout and report are covered user-visible promises under [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered). This deliberately changes those bytes: a script capturing `grund config migrate --to 2 --write` must add `--full` to keep its prior output. Preview captures remain unchanged.

Apply the argued pre-1.0 licence of [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise), following [§DF-show-default-token-cheap.4](DF-show-default-token-cheap.md#4-consequences): that decision made the cheap read the default and kept `--full` for the former output. Here carrying the old default beside a compact flag and a release-long warning would prolong the measured replay on every ordinary write and bury the actionable advice. The mechanical replacement costs affected callers one flag, preserves the complete legacy output, and changes no verdict or migration policy. That cost is smaller than retaining a legacy default through a release window; this argument applies before 1.0 and grants no general exemption.

The ordinary deprecation route of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) was considered and rejected for that specific cost. Moving the reminder alone leaves the duplicated candidate and mapping report. Suppressing stdout alone leaves repeated stderr detail. Removing the complete output would deny callers a supported replacement. The issue's tokens label alone authorizes none of these compatibility choices; the author approved this concrete design on 2026-10-22.

## 4. Refused writes close with their causes and remedies

Issue #939 shows a comment-only refusal ending with defaults described as `written:`, next-init advice, and a bare unchanged-file footer. Extend [§FS-config-migrate.2.2](../../functional-spec/FS-config-migrate.md#22-stderr-is-the-report) to close a refused requested write with all blocking causes and their remedies, retaining `not written:` and the unchanged-config assurance. Omit completed-write defaults and the next-init Project map note on that path, with or without `--full`; keep useful diagnostics and real comparison differences. Successful-write output stays as decided above; a refused preview closes the same way, as [§DF-config-migrate-refused-preview](DF-config-migrate-refused-preview.md#df-config-migrate-refused-preview-a-refused-migration-preview-closes-with-its-causes-and-keeps-its-candidate-behind---full) decides.

This changes human-readable diagnostic bytes covered by [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered), but moves no verdict: the same inputs refuse, with the same exit, stdout eligibility and unchanged config. The verdict-correction route, section 5 of [§REQ-backwards-compatibility](../../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly), does not apply. Apply the pre-1.0 licence of [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise) narrowly: carrying the old footer beside the actionable one, or retaining misleading completed-write lines through a deprecation window, would preserve precisely the confusing closing report being repaired. Callers matching the whole old footer must accept its added causes; callers matching `not written:` keep that prefix. No flag restores the misleading refused-write wording. That small diagnostic adjustment costs less than an alias and warning that perpetuate the ambiguity; it licenses no policy change or redesign of successful output.

## release-note: Release note

- A successful `grund config migrate --to 2 --write` no longer prints the candidate or mapping report; `--full` brings both back.
  A refused one ends with its causes and remedies, without completed-write defaults or next-init advice; its refusal verdict is unchanged.
