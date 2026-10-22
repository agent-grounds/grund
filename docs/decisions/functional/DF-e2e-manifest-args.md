# DF-e2e-manifest-args: an E2E case with no `command.args` reports `"args":[]`, and only its text form says `grund check`

**Date:** 2026-10-23

## status: Accepted

## 1. Context

[§FS-show.2.4.2](../../functional-spec/FS-show.md#242-the-manifest-as-json) says the `args` of an E2E case's JSON manifest is the parsed `command.args`,
empty when there is none, and the example in [§FS-output-shapes.4.2](../../functional-spec/FS-output-shapes.md#42-e2e-cases) shows `"args":[]` for such a
case. Every structured surface gave `["check"]` instead: `show --format json`, the bindings'
show `manifest`, and the scan snapshot's `e2eCase` ([§FS-distribution.3.2.2.3](../../functional-spec/FS-distribution.md#3223-scan-snapshot)). The scanner
stored the text form's implicit invocation in the case's record, so one field held two facts —
what the manifest says, and what the first line of [§FS-show.2.4](../../functional-spec/FS-show.md#24-e2e-cases) prints — and every writer
serialized the second. A consumer could not tell a case that names `check` from a case with no
manifest (agent-grounds/grund#947).

## 2. Decision

The record carries what the manifest says ([§AR-scanner.6.2](../../architecture/AR-scanner.md#62-what-a-case-declaration-records)): the parsed words of
`command.args`, none when the file is absent, and whether the file is there. The text form
derives the implicit `grund check` from that absence, on its first line only, and prints the
bare `grund` for a `command.args` that holds no words ([§FS-show.2.4](../../functional-spec/FS-show.md#24-e2e-cases)). Every structured surface
reports the parsed words, so a case with no `command.args` reports `"args":[]`.

No `check` verdict moves: which cases run, and with what arguments, is unchanged. Only the bytes
of a JSON field that contradicted its specification change.

## 3. Rejected alternatives

- **Move [§FS-show.2.4.2](../../functional-spec/FS-show.md#242-the-manifest-as-json) and [§FS-output-shapes.4.2](../../functional-spec/FS-output-shapes.md#42-e2e-cases) to `["check"]`.** It would make the wire
  form describe the invocation rather than the manifest, and erase the one difference a
  consumer needs: whether the case names its arguments or takes the default.
- **Announce it a release ahead.** The deprecation path is for a verdict that moves
  ([§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)); this changes no verdict, and holding a defect for a release
  keeps the documented contract false for that release.

## release-note: Release note

- [§DF-e2e-manifest-args](DF-e2e-manifest-args.md#df-e2e-manifest-args-an-e2e-case-with-no-commandargs-reports-args-and-only-its-text-form-says-grund-check), [§FS-show.2.4.2](../../functional-spec/FS-show.md#242-the-manifest-as-json), [§FS-distribution.3.2.2.3](../../functional-spec/FS-distribution.md#3223-scan-snapshot): **an E2E case with no `command.args` now reports `"args":[]`.** `grund show E2E-<name> --format json`, the Python and Node show `manifest.args`, and the scan snapshot's `e2eCase.args` gave `["check"]` for a case whose directory has no `command.args`, against the specification, which says `[]`. They now give `[]`; a case that has `command.args` reports its words as before. The text form is unchanged, and still opens with `grund check` for such a case. **Who this affects:** a consumer that read `["check"]` as the default invocation: read an empty `args` as "no manifest, runs `grund check`". Closes [issue #947](https://github.com/agent-grounds/grund/issues/947).
