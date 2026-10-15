# DF-mixed-path-citation-errors: Mixed citation findings use ordinary errors in 0.19.0

**Status:** Accepted
**Date:** 2026-10-15

## context: Context

The warning release of [§DF-number-name-sections.decision.citations](DF-number-name-sections.md#decisioncitations-a-citation-that-was-skipped-is-now-checked) shipped in
0.18.0: #704 merged before its tag, so the conditional delay was not needed.
Issue [#710](https://github.com/agent-grounds/grund/issues/710) completes that
announced route under [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) and serves
[§GOAL-no-silent-breakage](../../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path).

## decision: Decision

From 0.19.0, unresolved mixed citations use ordinary reference errors
([§FS-citations.checks.mixed-path](../../functional-spec/FS-citations.md#checksmixed-path-a-citation-of-a-name-under-a-number-is-checked-and-warns-until-0190)), and required rule and configured citation
findings lose only mixed-path relief ([§FS-rules.7.12](../../functional-spec/FS-rules.md#712-a-finding-only-a-citation-of-a-name-under-a-number-raises-warns-until-0190)). Their codes, sites and
ordinary messages remain; the temporary warning clause disappears. Independently
qualifying warning windows and explicit warning strengths keep their meanings.
Recommended findings remain opt-in suggestions.

An unrecorded mixed address still errors immediately and names `grund fmt --write`;
only its historical version clause disappears
([§FS-section-lock.checks.unrecorded-section-citation](../../functional-spec/FS-section-lock.md#checksunrecorded-section-citation-a-locked-address-the-lock-does-not-hold)). Resolution, enrollment,
fingerprints and move protection do not change.

The shared floor of [§FS-distribution.release-floor](../../functional-spec/FS-distribution.md#release-floor-one-shared-minimum-publishable-release) is set to `0.19.0`, once for
all parallel release transitions. Development may remain `0.18.1-dev` while
the release guard prevents publishing this tree below the floor, regardless of
manifest rewrites. Independent pending promises still constrain publication.
After this merges, a 0.18.x patch can only be cut from a branch made before it.
The original decision's shipped notice remains historical; this record supplies
a new notice under [§FS-distribution.4.6.4](../../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions). Doc-comment meaning and the `parse-moves`
retirement were separate work, landed by #639.

## release-note: Release note

- [§FS-citations.checks.mixed-path](../../functional-spec/FS-citations.md#checksmixed-path-a-citation-of-a-name-under-a-number-is-checked-and-warns-until-0190), [§FS-rules.7.12](../../functional-spec/FS-rules.md#712-a-finding-only-a-citation-of-a-name-under-a-number-raises-warns-until-0190),
  [§FS-section-lock.checks.unrecorded-section-citation](../../functional-spec/FS-section-lock.md#checksunrecorded-section-citation-a-locked-address-the-lock-does-not-hold): **Mixed citation warnings
  become ordinary errors in 0.19.0 after the 0.18.0 warning release.** Repositories
  relying on unresolved citations of a name under a number, or required rule and
  configured citation findings only those sites cause, now fail `check` and CI.
  Codes and sites stay the same; messages lose the mixed warning suffix, JSON
  severity becomes `error`, and such errors cause exit 1. Fix the target or
  citation, adjust an intended constraint, or explicitly filter its code.
  Independent warning windows, explicit warning strengths and opt-in suggestions
  keep their behavior. An unrecorded mixed address still errors until
  `grund fmt --write` records it, with no historical version suffix. The shared
  release floor prevents a 0.18.x publication from this tree; patches require a
  branch made before this transition. Closes [#710](https://github.com/agent-grounds/grund/issues/710).
