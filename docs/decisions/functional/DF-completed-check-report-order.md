# DF-completed-check-report-order: completed check reports share one order

**Date:** 2026-10-23

## status: Accepted

## 1. Context

The single-project run appended cautions after located warnings, while the workspace
run sorted all three report channels. Rust, Python and Node callers therefore saw
different warning arrays for the same findings merely by adding an empty workspace.
The frozen CLI projection ([§FS-distribution.3.0.3](../../functional-spec/FS-distribution.md#303-complete-data-and-canonical-parity)) sorted findings for comparison;
it did not promise an order for the raw arrays, which bindings preserve
([§FS-distribution.3.0.1](../../functional-spec/FS-distribution.md#301-report-and-finding)). CLI rendering already has its own order.

## 2. Decision

Complete engine reports under [§FS-errors.4.1](../../functional-spec/FS-errors.md#41-ordering) through one shared completion step,
used by single-project, workspace and editor checks. Keep caution insertion and
silence evaluation as they are; selection advice may still follow the findings.
No findings, messages, schemas, CLI output or verdicts change.

Raw-array consumers can observe this change ([§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered)).
Before 1.0, carrying a legacy-order option and two report contracts costs more
than adapting order-sensitive expectations ([§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise)).
Ship the correction with this notice; callers should identify findings by code
and update expectations that depend on array positions.

## 3. Rejected alternative

Keeping the old order would preserve dispatch-dependent answers and force callers
to sort reports themselves. Sorting in each binding would leave Rust and editor
callers inconsistent and break the bindings' preservation of core arrays.

## release-note: Release note

- Rust, Python and Node check report arrays now follow [§FS-errors.4.1](../../functional-spec/FS-errors.md#41-ordering) consistently across single-project and workspace runs; update order-sensitive expectations. Findings, CLI output and verdicts are unchanged. Closes [issue #952](https://github.com/agent-grounds/grund/issues/952).
