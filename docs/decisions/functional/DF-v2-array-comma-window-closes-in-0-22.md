# DF-v2-array-comma-window-closes-in-0-22: The version-2 trailing-comma window closes in 0.22.0

**Date:** 2026-10-29

## status: Accepted

## 1. Context

[§DF-v2-array-one-trailing-comma](DF-v2-array-one-trailing-comma.md#df-v2-array-one-trailing-comma-a-version-2-array-takes-one-trailing-comma-after-an-element-warned-for-in-0210-and-refused-from-0220) held a version-2 array to the one trailing comma TOML
allows, after an element, and took the deprecation path of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) to
get there. grund 0.21.0 still read an array whose run of commas before its `]` held a comma
with no element before it, `exclude = ["ab",,]` as `["ab"]` and `exclude = [,]` as the empty
list, and warned with code `config-not-toml`, ending `this warning becomes an error in grund
0.22.0`. That record's notice ships with 0.21.0, and a notice already released is not
published again ([§FS-distribution.4.6.4](../../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions)), so the release that refuses these arrays needs a
notice of its own, and this record carries it.

## 2. Decision

In 0.22.0 (agent-grounds/grund#1098):

- a comma before a version-2 array's `]` comes off only where it is the one trailing comma
  after an element; any other stays for the element reader, which refuses the empty element
  it leaves as `expected string` at the key's line ([§FS-config-v2.reader.5.commas.refusal](../../functional-spec/FS-config-v2.md#reader5commasrefusal-from-grund-0220-a-comma-with-no-element-before-it-is-expected-string)), so
  `check` and `config show` exit `2` and `config validate` exits `1`;
- where the array's text loaded through the window, the refusal ends `; this became an error
  in grund 0.22.0`. The text decides it, not the key: the suffix is there exactly where taking
  off the whole run of adjacent commas before the `]`, as 0.21.x did, leaves no empty
  element, so `languages = ["nope",,]`, which 0.21.x refused as `unknown language`, carries
  it too;
- a shape refused before, such as `["ab", ,]` or `[,"a"]`, keeps the bare `expected string`,
  and `["ab",]`, `[` / `"ab"` / `,` / `]` and the empty arrays load as before;
- the warning goes, and with it the `commas` remedy case of `config-not-toml` in
  [§FS-errors.5.5](../../functional-spec/FS-errors.md#55-the-check-code-catalog); the code stays for the constructs whose windows are still open
  ([§FS-config-v2.checks.config-not-toml.1](../../functional-spec/FS-config-v2.md#checksconfig-not-toml1-a-warning-in-021x-the-unclosed-array-or-expected-string-refusal-from-0220));
- the ancestor climb reads such a `[workspace] members` value as not a list, as it reads any
  value the full load refuses ([§FS-workspace.6.1.7.5](../../functional-spec/FS-workspace.md#6175-an-unobtainable-members-value-leaves-the-claim-undecidable));
- version 1, whose list reader already refuses every comma before a `]`, does not move.

## 3. Rejected alternatives

- **Keep the window for another release.** 0.21.0 printed 0.22.0 as the release these arrays
  are refused in, and the release guard refuses a version that reaches a promise it does not
  keep ([§FS-distribution.4.2](../../functional-spec/FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)).
- **Add the suffix only where the key would then have loaded.** The suffix tells a reader what
  changed about the array's text, which is the same under whatever key holds it, and deciding
  the other reading needs a trial read of the key with the run taken off, which writes into the
  project being loaded for a verdict the refusal then throws away.

## release-note: Release note

- [§DF-v2-array-comma-window-closes-in-0-22](DF-v2-array-comma-window-closes-in-0-22.md#df-v2-array-comma-window-closes-in-0-22-the-version-2-trailing-comma-window-closes-in-0220), [§FS-config-v2.reader.5.commas.refusal](../../functional-spec/FS-config-v2.md#reader5commasrefusal-from-grund-0220-a-comma-with-no-element-before-it-is-expected-string), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path):
  **a comma before a version-2 array's `]` with no element before it is now refused.** What
  grund 0.21.0 announced lands. `exclude = ["ab",,]`, `["ab" ,, ]`, `["ab",,,]`, an array whose
  last line is `"ab",,`, and `[,]`, `[ , ]`, `[,,]` and an array of one comma written across
  lines, which 0.21.0 read with every comma before the `]` taken off and warned for, are
  refused as `error: grund.toml:9: expected string; this became an error in grund 0.22.0`;
  `check` and `config show` exit `2`, `config validate` exits `1`, and the `config-not-toml`
  warning for them is gone. One trailing comma after the last element, `["ab",]`, and the
  empty arrays load as before, and a shape already refused, such as `["ab", ,]`, keeps its
  bare `expected string`. Version-1 configs load exactly as before. **Who this affects:** a
  repository whose version-2 `grund.toml` drew the `delete each comma on line … that has no
  element before it` warning in 0.21.0, which should delete each such comma. Closes
  [issue #1098](https://github.com/agent-grounds/grund/issues/1098).
