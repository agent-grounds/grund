# DF-fetch-section-structure: fetch refuses section structure that check already rejects

## status: Accepted

## 1. The pre-write validator accepted invalid section structure

Issue #1162 demonstrates a green tree becoming red after a successful `fetch`:
the saved declaration carries a section path deeper than its heading permits,
duplicate section paths, or a name-bearing path with an absent prefix. The checker
already rejects these structures, but fetch did not apply its section rules before
replacing the snapshot. The accepted-output contract already requires validation
before any mutation ([§FS-fetch.3](../../functional-spec/FS-fetch.md#3-accepted-declaration)).

## 2. Refuse structural errors before mutation

[§FS-fetch.3.sections](../../functional-spec/FS-fetch.md#3sections-section-structure-is-validated-before-writing)
applies the existing section rules to the snapshot before either writer runs.
The scanner's canonical section and duplicate records and the checker's structural
predicates are shared; fetch selects only these rules, not a whole-project check.
Configuration strengths, named-section enablement, complete-path-set prefix
semantics, and fenced content retain their existing meaning.

## 3. Correct the invalid write without a warning window

This changes fetch's acceptance verdict under
[§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids).
The prior verdict violated [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own):
fetch may replace or insert only its **validated** named declaration. This prohibition
already applied when that verdict shipped: tag `v0.21.0` includes both it and the
pre-write contract. The prior
[§FS-fetch.3](../../functional-spec/FS-fetch.md#3-accepted-declaration) defines whole-output
validation before mutation and refuses invalid subsection depth; the section rules
already make the reported structures errors. Replacing a valid saved snapshot with
one carrying these errors crosses that existing validation boundary. This correction
introduces no new checker prohibition.

A warning window under [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) cannot fit:
it would continue the forbidden replacement while warning about it.
A mechanical migration under [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) cannot repair arbitrary provider output:
fetch preserves accepted bytes verbatim
and cannot choose a missing prefix or a duplicate heading's intended coordinate.
Refuse before writing and name the coordinate, fetched-output line, and repair;
the author can correct the fetcher or fence provider headings as content. Existing
snapshot bytes remain available. No checker verdict or diagnostic text changes.
This is not a licence for policy tightening or a new prohibition: it corrects only
the existing validation boundary. The release note below records the verdict change
and the located repair every refusal must give.

## release-note: Release note

- [§DF-fetch-section-structure](DF-fetch-section-structure.md#df-fetch-section-structure-fetch-refuses-section-structure-that-check-already-rejects), [§FS-fetch.3.sections](../../functional-spec/FS-fetch.md#3sections-section-structure-is-validated-before-writing), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own): **fetch's acceptance verdict changes for section structures that check already rejects.** A path deeper than its heading permits under strict heading-depth validation, duplicate paths, and enabled name-bearing paths missing a prefix are refused with exit 2 before any mutation, preserving the previous snapshot and absent destinations. Every finding names its fetched-output location and the action: correct the fetcher's paths and heading levels, give each path one heading and each name-bearing path its prefixes, or fence provider headings as content. Heading-depth warnings, suggestions, and disabled checks retain their strengths; purely numeric prefix gaps remain allowed by the orphan rule.
