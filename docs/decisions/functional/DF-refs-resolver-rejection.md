# DF-refs-resolver-rejection: an ID rejected by a selected grammar is a failed query

**Status:** Accepted
**Date:** 2026-09-09

## 1. Context

`show` and `refs` sent the same two resolver outcomes to different public exit
codes: an ID rejected by the configured grammar and an ambiguous number-only
shorthand exited `1` from `show`, but `2` from `refs`
([§FS-refs.4](../../functional-spec/FS-refs.md#4-exit-codes)). Both readings had
a coherent local rule. `show` treated them as queries with no result; `refs`
treated every failure outside its possibly-empty list as a run error. The split
made `2` mean either “the scan is untrustworthy” or “repair this operand”
depending on which query command a script invoked.

The status is a frozen user-visible scalar
([§FS-cli.5](../../functional-spec/FS-cli.md#5-exit-code-mapping-is-fixed)), so
choosing a common meaning also has to apply the deprecation path in
[§GOAL-no-silent-breakage](../../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path).

## 2. Decision

### 2.1 Grammar selection is the boundary

Exit `2` means the run could not establish or complete the query context, or
could not complete a trustworthy scan. Once context has selected a project's ID
grammar, rejection of the operand by that resolver is a failed query at exit
`1` ([§FS-errors.2.3](../../functional-spec/FS-errors.md#23-bare-query-failure)).
An unknown alias therefore remains `2`: without its project the ID tail has no
selected grammar. A resolved target with no citations remains the successful
empty answer at `0`.

The rule covers both configured-format rejection and ambiguous number-only
shorthand, including after `--summary` or `--section` and after a known workspace
alias selects its target grammar. It does not change `show`, full-ID ambiguity
across two homes, or configuration-validation failures.

### 2.2 A scalar gets a hold-and-warn release

Grund 0.14.0 retains `refs`' exit `2` and existing diagnostic and hint bytes,
then appends one exact warning naming the 0.16.0 exit-`1` change. Grund 0.16.0
removes the warning and uses the shared failed-query text and JSON shapes. This
is the two-release path required by
[§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path), applied to a scalar for which no command can migrate a caller.

The warning itself holds the pending release promise. At the flip, the ordinary
failed-query diagnostic remains free of a historical suffix; a version-gated
wire contract activates the new mapping and warning retirement, while the
release gate reads the warning-phase scalar clause
([§FS-distribution.4.2](../../functional-spec/FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)).

The flip landed in grund 0.16.0, the release the warning named: both rejections
now exit `1` with the bytes `show` prints, the warning and the `error:` prefix
are gone, and the compatibility notice is [§DF-refs-resolver-rejection.release-note](DF-refs-resolver-rejection.md#release-note-release-note).
What this section decided — a hold-and-warn release for a scalar no command
can migrate — stays true once the window has closed.

## 3. Alternatives considered

Making every malformed operand a CLI error would have moved `show` to `2` and
made a selected grammar's answer indistinguishable from setup or scan failure.
Keeping `refs` permanently at `2` would have preserved its list-only model but
left scripts without one cross-command meaning for the same resolver outcome.
A configurable mapping was rejected because exit mappings are not policy knobs
([§FS-non-goals.9](../../functional-spec/FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization)).

## 4. Consequences

Exact-stderr consumers see an appended warning throughout 0.14.0 and 0.15.0. At 0.16.0,
callers repair these operands on exit `1` and reserve `2` for context, run, I/O,
and incomplete-scan failures. JSON callers receive one failed-query diagnostic
object instead of raw CLI text; successful and empty citation lists do not move.

## release-note: Release note

- [§FS-refs.4](../../functional-spec/FS-refs.md#4-exit-codes), [§FS-output-shapes.6.1.2](../../functional-spec/FS-output-shapes.md#612-from-0160), [§FS-errors.2.3](../../functional-spec/FS-errors.md#23-bare-query-failure), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **`grund refs` on an invalid ID or an ambiguous number-only shorthand now exits `1`, a failed query.** The window every `0.14.x` and `0.15.x` binary announced in its own output closes on the release it named: ``error: invalid ID `FS-bar` ``, the configured-format `hint:`, and ``warning: `grund refs` invalid IDs and ambiguous number-only shorthands currently exit 2; they will exit 1 (failed query) in grund 0.16.0`` at exit `2` become exactly what `grund show` prints — ``invalid ID `FS-bar` `` and the hint, or `ambiguous ID: FS-042 (matches FS-042-user-login, FS-042-user-logout)` — at exit `1`. Under `--format json` the raw text becomes the shared failed-query object, ``{"severity":"error","path":null,"line":null,"code":"invalid-id","message":"invalid ID `FS-bar`","sites":null,"authority":null}`` (code `ambiguous` for the shorthand), with no hint. `--summary`, `--section`, `--descendants` and `--total` follow the same path. Five e2e cases move from exit `2` to exit `1`: `refs-invalid-id-format`, `refs-invalid-id-format-json`, `refs-total-invalid-id`, `refs-ambiguous-shorthand` and `refs-ambiguous-shorthand-json`. **What does not move:** `grund show`'s bytes, an unknown alias and a scan or I/O failure at exit `2`, and a resolved ID with no citations at exit `0`. **Who this breaks:** a script that read exit `2` from `grund refs` as "fix the ID", or matched the `error:` prefix or the warning line — it now reads exit `1` and the bare message. Closes [issue #443](https://github.com/agent-grounds/grund/issues/443).
