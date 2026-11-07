# DF-value-candidate-metadata-failure: a failed value candidate lookup makes the scan incomplete

**Date:** 2026-11-07

## status: Accepted

## 1. Context

A value home at mode `0444` permits listing but denies metadata lookup of its
children. Discovery treated the failed file-status lookup as a confirmed non-file,
silently removing every value source. `list` and uncited `check` returned `0`;
`check` citing a declaration in that home returned `1` for an unknown reference,
without reporting why the declaration was missing
([issue #1289](https://github.com/agent-grounds/grund/issues/1289)).

## 2. Decision and compatibility

[§FS-values.5.3.1](../../functional-spec/FS-values.md#531-a-listed-candidate-whose-file-status-cannot-be-read)
makes failed classification incomplete input through the existing scan-failure
channel, retaining the source or home path, shared contextual I/O reason, and
repair. Confirmed non-files remain silent; readable sources and findings remain
available. The discovery rules, buffer view, alias ownership, ordering, and
selected-folder fetch boundaries stay as specified there.

The prior verdict violated [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips):
an input whose metadata could not be read disappeared without a diagnostic, so a
partial tree passed as complete or reported only a dangling citation.
That prohibition already applied when the old verdict shipped; the existing
[§FS-check.2.4](../../functional-spec/FS-check.md#24-an-incomplete-run) required
continuation and exit `2` for an incomplete scan.

Use [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids)'s correction route.
[§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) cannot fit:
a warning window would continue treating unread input as a complete report.
[§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) does not fit:
no single grund command can decide whether to grant filesystem access or keep
the input out of the scan. The diagnostic names the affected location and tells
the maintainer to make the path readable or keep it out of the scan.
This is not a licence for new source recognition, scan policy, or citation advice.

## release-note: Release note

- In grund 0.22.0, a listed value-source candidate whose file status cannot be
  read produces an incomplete-scan failure: `list` and `check` verdicts change
  from `0` or `1` to `2`, including a value home readable but not searchable.
  Every finding names its location and an action: make the path readable, or
  keep it out of the scan. Confirmed non-files remain silent and readable input
  remains available. This correction uses [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids)
  because the prior verdict violated [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips)
  ([§DF-value-candidate-metadata-failure](DF-value-candidate-metadata-failure.md#df-value-candidate-metadata-failure-a-failed-value-candidate-lookup-makes-the-scan-incomplete)).
