# DF-homeless-code-declares-no-ids: `code` with no row is refused as declaring no IDs alone from 0.19.0

**Date:** 2026-10-15

## status: Accepted

## context: Context

[§DF-one-answer-rulings-grammar](DF-one-answer-rulings-grammar.md#df-one-answer-rulings-grammar-one-grammar-keeps-the-documented-refusal-bytes-and-order-and-one-kind-universe-keeps-each-commands-vocabulary) ruled on G09: in a project whose `[[kinds]]` declares no homeless kind, `code` is not citable ([§FS-config.3.9.2](../../functional-spec/FS-config.md#392-the-homeless-kind)), so a rule's subject and object, `list --kind` and `list --selector` owe it [§FS-errors.2.5.1](../../functional-spec/FS-errors.md#251-a-kind-that-declares-no-ids)'s reason rather than `unknown kind`. The route is the row of [§DF-one-kind-verdict.decision.routes](DF-one-kind-verdict.md#decisionroutes-which-route-each-move-takes) for a non-citable kind on a selector or in a rule: the legacy reason kept and the new one appended for a release, then the new reason alone in `0.19.0`. [Issue #671](https://github.com/agent-grounds/grund/issues/671) opened that window in `0.18.0`. This record does not decide either again; it holds the flip, [issue #680](https://github.com/agent-grounds/grund/issues/680), and the notice it publishes, as [§RM-one-grammar-flip.1](../../roadmap.md#1-what) asks of each part of the flip.

## decision: Decision

From `0.19.0` the five refusals of `code` with no row print the form of [§FS-errors.3.8.1](../../functional-spec/FS-errors.md#381-the-final-templates) with no home alone: `kind "code" declares no IDs` on a rule's subject, a rule's object and `list --selector`, spelled `Each code` or bare, and ``kind `code` declares no IDs`` on `list --kind` ([§FS-errors.3.8](../../functional-spec/FS-errors.md#38-the-one-grammar-wording-migration), [§FS-list.1.1](../../functional-spec/FS-list.md#11---kind)). The `known kinds:` line still follows, and the exit code, the `invalid-rule` code and its severity, and what is refused do not move. G09 leaves [§REQ-one-answer.differences.identity](../../requirements/REQ-one-answer.md#differencesidentity-identity).

### decision.scope: What stays windowed

The flip is `code`'s alone. A configured non-citable kind such as `e2e`, and the section-component wildcard joined by `and`, kept the window of [§FS-errors.3.8](../../functional-spec/FS-errors.md#38-the-one-grammar-wording-migration) until the rest of [§RM-one-grammar-flip](../../roadmap.md#rm-one-grammar-flip-end-the-one-grammar-window-in-0190) landed, in the same release, under [§DF-one-grammar-lands-in-0-19](DF-one-grammar-lands-in-0-19.md#df-one-grammar-lands-in-0-19-the-one-grammar-window-closes-in-0190-and-a-pinned-target-the-run-cannot-accept-fails-the-load). A kind no project configures keeps `unknown kind "POLICY"` (D10).

## consequences: Consequences

A consumer that migrated during the window to the reason after the ` — ` reads the same bytes; one that matched the legacy prefix, `unknown kind "code"`, ``unknown kind `code` `` or `literal subject "code" does not match the configured ID grammar`, no longer finds it.

## release-note: Release note

- [§FS-errors.3.8](../../functional-spec/FS-errors.md#38-the-one-grammar-wording-migration), [§FS-errors.3.8.1](../../functional-spec/FS-errors.md#381-the-final-templates), [§FS-list.1.1](../../functional-spec/FS-list.md#11---kind), [§DF-homeless-code-declares-no-ids](DF-homeless-code-declares-no-ids.md#df-homeless-code-declares-no-ids-code-with-no-row-is-refused-as-declaring-no-ids-alone-from-0190): **`code`, in a project that declares no homeless kind, is refused as declaring no IDs, without `unknown kind`.** The window 0.18.0 announced closes for `code`: `grund list --kind code` printed ``error: unknown kind `code` — kind `code` declares no IDs; this wording changes in grund 0.19.0`` and now prints ``error: kind `code` declares no IDs``; `list --selector 'Each code'`, `check --rule 'Each code must have exactly one terms chapter.'` and `check --rule 'Each FS must cite at least one code.'` printed `error: unknown kind "code" — kind "code" declares no IDs; this wording changes in grund 0.19.0` and now print `error: kind "code" declares no IDs`; and `list --selector code` printed `error: literal subject "code" does not match the configured ID grammar — kind "code" declares no IDs; this wording changes in grund 0.19.0` and now prints `error: kind "code" declares no IDs`. **What does not move:** each is still followed by its `known kinds:` line and exits `2`, the `invalid-rule` code and its severity, and `unknown kind "POLICY"` for a kind no project configures. **Who this breaks:** a script that matches the legacy prefix of one of the five refusals above. Closes [issue #680](https://github.com/agent-grounds/grund/issues/680).
