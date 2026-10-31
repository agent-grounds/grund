# DF-root-value-mismatch-wording-lands-in-0-22: The root-aimed value-mismatch wording window closes in 0.22.0

**Date:** 2026-10-31

## status: Accepted

## 1. Context

[§DF-root-value-mismatch-wording](DF-root-value-mismatch-wording.md#df-root-value-mismatch-wording-a-root-aimed-value-mismatch-names-the-cited-root-and-the-literal-as-written-after-one-release-that-appends-it) made a root-aimed value mismatch whose literal splits into as
many parts as its root has components name the cited root and the literal as written, and staged
the change through the warning release of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): grund 0.21.0 kept the
message earlier releases printed whole in front, followed by ` — whole binding: `, the new
message and `; this wording changes in grund 0.22.0` ([§FS-errors.3.9](../../functional-spec/FS-errors.md#39-the-root-aimed-value-mismatch-wording-migration)). That record's notice
shipped with 0.21.0, and a notice already released is not published again
([§FS-distribution.4.6.4](../../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions)), so the release that drops the prefix needs a notice of its own, and
this record carries it. The suffix named 0.22.0 as the release the bytes change in, and the
release guard refuses a version that reaches a promise it does not keep ([§FS-distribution.4.2](../../functional-spec/FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)).

## 2. Decision

In `0.22.0` (agent-grounds/grund#1118):

- the legacy message, the ` — whole binding: ` and the suffix go, so a root-aimed mismatch whose
  part count equals the component count prints the template of [§FS-errors.3.9.1](../../functional-spec/FS-errors.md#391-the-final-templates) alone, on the
  text line, in the `--format json` `message` and in the editor's diagnostic alike;
- where that one pair differs in kind, the root form of the clause of [§FS-values.5.2.1](../../functional-spec/FS-values.md#521-a-mixed-kind-mismatch-names-each-sides-kind) stays,
  and the component-aimed form the legacy message carried goes with it;
- the two messages that never took the window keep their bytes: a component-aimed mismatch, and
  a root-aimed one whose part count differs from the component count ([§FS-values.5.2.2](../../functional-spec/FS-values.md#522-a-root-aimed-mismatch-names-the-first-unequal-component));
- the code `value-mismatch`, the error severity, the primary location, the one `sites` entry,
  the selectors and the exit verdict do not move.

## 3. Rejected alternatives

- **Keep the window for another release.** 0.21.0 printed 0.22.0 as the release the bytes
  change in, and [§FS-distribution.4.2](../../functional-spec/FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name) refuses a release that reaches a promise it does not keep.
- **Keep the component-aimed kind clause beside the root form.** The root form already names the
  component its kinds belong to, so the line would say one fact twice, and the clause of the
  component-aimed form describes a binding the site did not write.

## release-note: Release note

- [§DF-root-value-mismatch-wording-lands-in-0-22](DF-root-value-mismatch-wording-lands-in-0-22.md#df-root-value-mismatch-wording-lands-in-0-22-the-root-aimed-value-mismatch-wording-window-closes-in-0220), [§FS-errors.3.9.1](../../functional-spec/FS-errors.md#391-the-final-templates), [§FS-values.5.2.2](../../functional-spec/FS-values.md#522-a-root-aimed-mismatch-names-the-first-unequal-component):
  **a value mismatch at a binding aimed at a whole value names the root and the literal alone.**
  What grund 0.21.0 announced lands. For `` `30 minutes` `` bound to the root `FS-session.1`,
  `grund check` printed ``value mismatch for FS-session.1.1: bound `30`, declared `45` at
  requirements.md:11 — whole binding: value mismatch for FS-session.1 (component 1.1): bound
  `30 minutes`, declared `45 minutes` at requirements.md:11; this wording changes in grund
  0.22.0`` before its remedy, and now prints ``value mismatch for FS-session.1 (component 1.1):
  bound `30 minutes`, declared `45 minutes` at requirements.md:11`` before it, in the
  `--format json` `message` and the editor's diagnostic alike. A component-aimed mismatch, and a
  root-aimed one whose part count differs from the component count, print what they printed.
  **Who this affects:** a script that matches the legacy `value mismatch for <component>` prefix
  of a root-aimed mismatch, or the ` — whole binding: ` after it, which should match
  `code == "value-mismatch"`. Closes [issue #1118](https://github.com/agent-grounds/grund/issues/1118).
