# DF-nested-alias-targets-land-in-0-19: a finding a target pinned at its own nested alias alone produces is an error from 0.19.0

**Date:** 2026-10-21

## status: Accepted

## 1. Context

[§DF-nested-alias-rule-targets.2.2](DF-nested-alias-rule-targets.md#22-the-new-failures-take-the-deprecation-path) made a rule target pinned at its own member's nested alias path
match what the bare kind matches, and sent the findings only that corrected match produces down
the deprecation path of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): through 0.18.x each was a warning
([§FS-rules.7.9](../../functional-spec/FS-rules.md#79-a-target-pinned-at-its-own-nested-alias-warns-until-0190)) ending
`; a target pinned at the rule's own nested alias path now matches, and this warning becomes an error in grund 0.19.0`.
That record's notice shipped with 0.18.0, and a notice
already released is not published again ([§FS-distribution.4.6.4](../../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions)), so the release that ends the
warnings needs a notice of its own, and this record carries it.

## 2. Decision

In 0.19.0 (agent-grounds/grund#881) every such finding is the same code's ordinary error on the
`must` channel ([§FS-rules.7.9](../../functional-spec/FS-rules.md#79-a-target-pinned-at-its-own-nested-alias-warns-until-0190)): a `forbidden-citation` at a site only the pin reaches, the
`citation-cardinality` or `uncited-unit` of an `at most N` or `exactly N` count, outbound or
inbound, that those sites push out of bounds, and every per-target `citation-cardinality` for a
target only the pin brings in, a miss included. Its message is the one the warning carried with
the clause gone and none in its place, so in member `group/api`
`Each API must not cite any group/api/AR.` prints
`group/api/docs/api/API-a.md:3: error: API must not cite group/api/AR (RULE-y) — re-point the
citation or downgrade it to a plain Markdown link` and contributes exit `1`. What the pin
matches, the codes, the sites and the recommended level's suggestions do not move, and no
`must` or `must not` rule finding is a warning any longer.

## 3. Rejected alternatives

- **Keep the warning for another release.** 0.18.0 printed 0.19.0 as the release it becomes
  an error in, and the release guard refuses a version that reaches a promise it does not keep
  ([§FS-distribution.4.2](../../functional-spec/FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)).
- **Keep a clause saying it became an error.** The code was an error throughout and only these
  findings warned, so once they are errors nothing is left to tell apart, as with the
  counted-section findings that landed in 0.18.0 ([§FS-rules.7.8](../../functional-spec/FS-rules.md#78-a-newly-counted-section-citation-warns-until-0180)).

## release-note: Release note

- [§DF-nested-alias-targets-land-in-0-19](DF-nested-alias-targets-land-in-0-19.md#df-nested-alias-targets-land-in-0-19-a-finding-a-target-pinned-at-its-own-nested-alias-alone-produces-is-an-error-from-0190), [§FS-rules.7.9](../../functional-spec/FS-rules.md#79-a-target-pinned-at-its-own-nested-alias-warns-until-0190), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **a finding a rule target pinned at its own nested alias path alone produces is now an error.** What grund 0.18.0 announced lands: in a member nested inside another, such as `group/api`, a rule that pins the member's own alias path, such as `Each API must not cite any group/api/AR.`, printed `group/api/docs/api/API-a.md:3: warning: API must not cite group/api/AR (RULE-y) — re-point the citation or downgrade it to a plain Markdown link; a target pinned at the rule's own nested alias path now matches, and this warning becomes an error in grund 0.19.0` and now prints `group/api/docs/api/API-a.md:3: error: API must not cite group/api/AR (RULE-y) — re-point the citation or downgrade it to a plain Markdown link`, failing `grund check` with exit `1`. The same holds for an `at most N` or `exactly N` count the pin pushes out of bounds, outbound or inbound, and for a per-target `cite each` finding on a declaration the pin brings in. **Who this affects:** a workspace whose `grund check` showed that warning in 0.18.0: remove the citation, add the missing one, loosen the rule's count, or pass `--ignore` on the finding's code. Closes [issue #881](https://github.com/agent-grounds/grund/issues/881).
