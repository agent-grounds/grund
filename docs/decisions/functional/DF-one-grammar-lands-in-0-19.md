# DF-one-grammar-lands-in-0-19: the one-grammar window closes in 0.19.0, and a pinned target the run cannot accept fails the load

**Date:** 2026-10-21

## status: Accepted

## 1. Context

[§DF-one-kind-verdict.decision.routes](DF-one-kind-verdict.md#decisionroutes-which-route-each-move-takes) shipped two moves with a window in 0.18.0. A non-citable
kind on a selector or in a rule, and a section-component wildcard subject joined by `and`, kept
their legacy reason and appended the new one, ending `; this wording changes in grund 0.19.0`
([§FS-errors.3.8](../../functional-spec/FS-errors.md#38-the-one-grammar-wording-migration)). A pinned or `*/` `[citations]` target the run cannot accept loaded with a
warning ending `; this becomes an error in grund 0.19.0` ([§FS-config.3.9.5.2](../../functional-spec/FS-config.md#3952-a-pinned-target-is-judged-against-its-projects-kinds)). The homeless kind
`code` left the window first, under [§DF-homeless-code-declares-no-ids](DF-homeless-code-declares-no-ids.md#df-homeless-code-declares-no-ids-code-with-no-row-is-refused-as-declaring-no-ids-alone-from-0190). [§RM-one-grammar-flip](../../roadmap.md#rm-one-grammar-flip-end-the-one-grammar-window-in-0190)
asked that the rest be its own change, with its own notice: the 0.18.0 notice shipped, and a
notice already released is not published again ([§FS-distribution.4.6.4](../../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions)).

## 2. Decision

In 0.19.0 (agent-grounds/grund#881):

- the legacy prefix and the suffix go, so a non-citable kind is refused with
  `kind "<KIND>" declares no IDs — <home>/ is not a citable home` alone, and a wildcard subject
  joined by `and` with `section-component wildcards are not accepted in phase 1` alone; the line
  that followed, `known kinds:` or none, still follows, and the exit code, the `invalid-rule`
  code and its severity do not move ([§FS-errors.3.8](../../functional-spec/FS-errors.md#38-the-one-grammar-wording-migration), [§FS-errors.3.8.1](../../functional-spec/FS-errors.md#381-the-final-templates));
- a pinned or `*/` `[citations]` target the run cannot accept fails the load: the first such
  target prints its line as an `error:`, ending `; this became an error in grund 0.19.0`, and
  the run writes nothing to stdout and exits `2` ([§FS-config.3.9.5.2](../../functional-spec/FS-config.md#3952-a-pinned-target-is-judged-against-its-projects-kinds));
- the canonical config of [§DF-directions-render.3](DF-directions-render.md#3-the-canonical-config), which pinned `api/AR` in a project that
  declared no `[workspace]`, gains a `[workspace]` whose member `api` configures `AR`, so it
  loads and the section it renders is unchanged.

## 3. Rejected alternatives

- **Keep the window for another release.** 0.18.0 printed 0.19.0 as the release it closes in,
  and the release guard refuses a version that reaches a promise it does not keep
  ([§FS-distribution.4.2](../../functional-spec/FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)).
- **Print every target the run cannot accept, not the first.** A config refuses its first
  error and exits, as an overlap between two targets already does ([§FS-config.3.9.5](../../functional-spec/FS-config.md#395-validation)), and the
  order the targets are judged in is fixed, so fixing one shows the next.
- **Drop `api/AR` from the canonical config.** It is the config's one pinned-alias target, the
  branch that renders an alias as spelled, so dropping it would leave that branch unpinned. A
  workspace keeps the branch and the rendered bytes.

## release-note: Release note

- [§DF-one-grammar-lands-in-0-19](DF-one-grammar-lands-in-0-19.md#df-one-grammar-lands-in-0-19-the-one-grammar-window-closes-in-0190-and-a-pinned-target-the-run-cannot-accept-fails-the-load), [§FS-errors.3.8.1](../../functional-spec/FS-errors.md#381-the-final-templates), [§FS-config.3.9.5.2](../../functional-spec/FS-config.md#3952-a-pinned-target-is-judged-against-its-projects-kinds), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **a non-citable kind and a wildcard subject are refused with their reason alone, and a `[citations]` target the run cannot accept is an error.** What grund 0.18.0 announced lands. `check --rule 'Each e2e must have exactly one terms chapter.'`, where `e2e` is a configured non-citable kind homed at `tests/e2e/`, printed `error: unknown kind "e2e" — kind "e2e" declares no IDs — tests/e2e/ is not a citable home; this wording changes in grund 0.19.0` and now prints `error: kind "e2e" declares no IDs — tests/e2e/ is not a citable home`, on a rule's subject and object, a configured rule and `list --selector` alike; `list --selector e2e.terms` no longer opens with `literal subject "e2e.terms" does not match the configured ID grammar`; and `check --rule 'FS-x.* must cite at least one FS and must not cite any FS.'` printed the conjunction refusal first and now prints `error: section-component wildcards are not accepted in phase 1`. A pinned or `*/` target the run cannot accept, such as `[citations.FS] must-not = ["api/FS"]` where member `api` configures no `FS`, printed a warning ending `; this becomes an error in grund 0.19.0` and loaded; it now prints the same line as an `error:` ending `; this became an error in grund 0.19.0`, and every command exits `2`. **Who this affects:** a script that matches the legacy prefix of one of those refusals, which should match the reason after the ` — ` it printed in 0.18.0; and a config that saw the `[citations]` warning in 0.18.0, which now fails to load until the target names a kind its project configures as citable, or, for a run with no workspace, until the config declares the `[workspace]` the alias names. Closes [issue #881](https://github.com/agent-grounds/grund/issues/881).
