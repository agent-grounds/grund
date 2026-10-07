# DF-canonical-slug-declarations: configured slug punctuation retains its canonical declaration identity

**Status:** Accepted
**Date:** 2026-10-07

Honor the configured slug grammar throughout declaration discovery and exact
reads ([§FS-declarations.line.configured-slug](../../functional-spec/FS-declarations.md#lineconfigured-slug-characters-admitted-by-the-slug-pattern-belong-to-the-canonical-id)).

## 1. Conflict with the existing requirement

[§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms)
already forbids reporting a resolving, canonical citation as broken.
This prohibition is present at the same numbered point in both released
`v0.16.0` and `v0.16.1`; it predates this correction. Under
`format = "{kind}-{slug}"` and `slug_pattern = "[a-z*][a-z0-9*-]*"`,
`FS-*` matches the configured grammar and its declaration is present in
`list`. Nevertheless, grund 0.16.2-dev at `7ee51b29f8` reports its marked
citations as `unknown reference`, its valid Markdown-link index entry as
missing, and its declaration as off-format. These are false alarms caused
by storing a conforming declaration under an off-grammar identity while its
queries and citations use the canonical identity. No citation policy or
configuration change makes those reports true.

## 2. Decision

Restore canonical recognition and consistent body reads in every declaration
position, including a slug consisting only of `*` and a slug ending in `*`.
Keep the complete-token and section-coordinate safeguards. Exact queries use
the literal configured ID. This decision introduces no pattern-query syntax.

This verdict correction follows
[§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids),
with each of its five conditions established below.

**Prior prohibition:** the old verdict violated [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms),
a separately declared hard requirement whose prohibition already applied when the old verdict shipped.
The canonical declaration is present, yet its citations are reported as broken;
the conflict is proved above, and the prohibition is present in both released tags.

**Accepted proof:** this accepted record cites that numbered requirement and proves the conflict.
[§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)'s deprecation path does not fit:
there is no supported authoring behavior to withdraw, and a warning about a valid citation remains a false alarm.
[§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)'s mechanical migration does not fit either:
the declaration, citations, configuration and index links are already valid;
renaming them would require unnecessary edits to accommodate the parser defect.

**Named release:** this record's release-note section names the before and after verdict for the correcting release.

**Actionable findings:** this correction removes the false findings without replacing them.
Unrelated findings retain their locations and the actions a maintainer can take.

**No new licence:** this route cannot justify ordinary policy tightening, feature removal,
or a prohibition invented by this change. The configured grammar and genuine off-grammar findings remain in force.

## release-note: Release note

- [§DF-canonical-slug-declarations](DF-canonical-slug-declarations.md#df-canonical-slug-declarations-configured-slug-punctuation-retains-its-canonical-declaration-identity), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms), [§FS-declarations.line.configured-slug](../../functional-spec/FS-declarations.md#lineconfigured-slug-characters-admitted-by-the-slug-pattern-belong-to-the-canonical-id): **configured slug punctuation retains its canonical declaration identity.**
  Before the correction, a valid repository declaring and citing `FS-*` or `FS-tail*`,
  with Markdown-link index entries, failed `check` at exit `1` with off-format,
  unknown-reference and missing-index-entry errors. After the correction it prints
  `success` at exit `0`; explicit and bare `show` return the actual lead, and
  declaration-only `refs` no longer emits the typo note.
  No repository edit or migration is required. Consumers expecting the erroneous
  findings must accept their removal; genuinely off-grammar tokens and unresolved
  citations retain their findings.
  The verdict changes from failure to success for the valid fixture; every finding
  that remains keeps its location and corrective action.
