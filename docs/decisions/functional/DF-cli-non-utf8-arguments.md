# DF-cli-non-utf8-arguments: refuse non-UTF-8 argv before dispatch while preserving version precedence

**Date:** 2026-11-06

## status: Accepted

## 1. Context

Collecting `std::env::args()` panics before dispatch when any argument is not
UTF-8, including a path the scanner can read and an extra word beside `--version`.
The controlled reproduction for [issue #1259](https://github.com/agent-grounds/grund/issues/1259)
exits `101` in all five cases. The CLI promises only `0`, `1`, and `2`
([§FS-cli.5](../../functional-spec/FS-cli.md#5-exit-code-mapping-is-fixed)).

## 2. Decision

Take OS arguments without panicking and apply [§FS-cli.5.1](../../functional-spec/FS-cli.md#51-non-utf-8-process-arguments-are-refused-before-dispatch):
literal `--version` or `-V` anywhere wins with the normal version line and exit
`0`; otherwise a non-UTF-8 argument gets one escaped `error:` diagnostic, empty
stdout, and exit `2` before dispatch or writes. This applies before help and
before completion's quiet config/scan failure handling. Valid UTF-8 invocations
keep their behavior. No arbitrary-byte path support is introduced.

The issue permits faithful use or refusal. Refusal keeps the existing string
parsers and gives a truthful answer without widening every command's accepted
paths. Lossy conversion is rejected: a replacement-character filename may name a
different file, including one a writer would change.

## 3. Compatibility route

Use [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) for the changed process status:

- The old exit `101` violated [§REQ-never-crashes.1](../../requirements/REQ-never-crashes.md#1-every-failure-is-a-diagnostic), which requires an in-band diagnostic rather than a stack trace, and [§REQ-never-crashes.2](../../requirements/REQ-never-crashes.md#2-exit-codes-are-the-api), which freezes the exit mapping. These requirements already applied when the old verdict shipped in `0.21.0`. The workspace-member alias deviation concerns another entry path and does not permit an argv panic.
- This accepted record proves that conflict. [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) does not fit: preserving the panic for a deprecation window would continue breaking the prior no-crash requirement. [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) has no mechanical migration to apply: the defect is in argument collection, not project content.
- The release note below names `101` becoming `2`, or `0` for a version request, and the callers affected. The refusal identifies its location as the escaped offending argument and identifies invalid UTF-8 as the problem; the caller can pass a UTF-8 argument, renaming a path or naming its UTF-8 parent directory. A successful version request has no replacement finding.
- This route cannot justify ordinary policy tightening or feature removal. It restores existing no-crash and version promises and changes no invocation whose arguments are valid UTF-8.

## release-note: Release note

- [§DF-cli-non-utf8-arguments](DF-cli-non-utf8-arguments.md#df-cli-non-utf8-arguments-refuse-non-utf-8-argv-before-dispatch-while-preserving-version-precedence), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids): process verdicts change from panic exit `101` to an explicit exit `2` refusal for non-UTF-8 arguments, or to exit `0` with the normal version line when literal `--version` or `-V` occurs anywhere. Scripts passing filesystem names may notice the status change. The old verdict violated [§REQ-never-crashes.1](../../requirements/REQ-never-crashes.md#1-every-failure-is-a-diagnostic) and [§REQ-never-crashes.2](../../requirements/REQ-never-crashes.md#2-exit-codes-are-the-api); every finding replacing it identifies its location as the escaped argument and the action is to pass UTF-8, renaming the path or naming a UTF-8 parent directory. Refusal occurs before any write; valid UTF-8 invocations keep their behavior. Closes [issue #1259](https://github.com/agent-grounds/grund/issues/1259).
