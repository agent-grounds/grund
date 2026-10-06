# DF-word-character-citation-markers: recognize accepted word-character markers

**Status:** Accepted
**Date:** 2026-10-06

## 1. Context and decision

[§FS-config.3.1](../../functional-spec/FS-config.md#31-reference--citation-form) accepts `_`
as a citation marker. The existing marker-followed-by-ID recognition promise of
[§FS-check.1.1](../../functional-spec/FS-check.md#11-recognized-citations) gives `_FS_login`
defined meaning under `{kind}_{slug}`, but the ID's leading word boundary hides it.
[§FS-check.1.1.10](../../functional-spec/FS-check.md#1110-the-configured-marker-establishes-the-citation-start)
makes that promise explicit: the configured marker establishes the citation start while
bare tokens retain their boundary. Rejecting an already accepted marker would remove a
configuration contract rather than restore recognition, so it is not the selected remedy.

## 2. Verdict correction and compatibility

Recovering a missing edge removes false unused warnings and can satisfy grounding. It can
also turn a passing check into a failing one when the recovered citation dangles, names a
missing section, or violates an existing direction or chapter rule. This is a verdict change
governed by [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered),
even though the citation spelling and configuration remain unchanged.

The applicable route is
[§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids):

1. **Prior prohibition.** The old passing verdict violated [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded) by silently excluding configured word-character markers without a declared scanner blind spot.
   That prohibition already applied when the old verdict shipped; there was no marker exclusion.
   [§REQ-no-missed-citation.3](../../requirements/REQ-no-missed-citation.md#3-proven-per-host-language)
   additionally requires a dangling citation in every supported doc-comment form to fail check.
   The Rustdoc spelling accepted by the configured-marker contract was silently missed.
2. **Accepted proof.** This record documents that conflict.
   [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) does not fit: there is no old spelling being withdrawn or replacement syntax to migrate to.
   [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) does not fit either: no single rewrite can decide whether an authored missing target needs
   a declaration, a corrected ID, or an escaped illustration.
3. **Named release.** The correcting release must include the compatibility notice below.
4. **Actionable findings.** Recovered edges use the existing located findings and hints.
   A missing target can be declared, corrected, or escaped when it is only an illustration;
   section and policy findings keep their existing actions. No existing finding text changes.
5. **No new licence.** This route cannot justify ordinary policy tightening or a new prohibition.
   This restores an existing recognition promise and invents no marker
   restriction or hard requirement. The no-defined-meaning exception does not apply: marked
   IDs already had defined meaning.

## release-note: Release note

Configured markers ending in word characters, such as `_`, now recognize unqualified full-ID
citations. `refs`, `cover`, and editor definition navigation recover the missing edges, and
`check` stops calling their targets unused. Repositories using such markers may now fail
existing dangling, missing-section, or citation-policy checks for citations previously missed.
Follow the located finding to correct or declare the target, or escape an illustration with
the configured marker wrapped in angle brackets, such as `<_>FS_login`.

This verdict change is the correction recorded in [§DF-word-character-citation-markers.2](DF-word-character-citation-markers.md#2-verdict-correction-and-compatibility), under [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), for the prior violation of [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded).
Every finding names its location and an action the maintainer can take; the existing remedies
apply to recovered citations.
