# DF-escape-position-is-not-a-citation: an escape position is not a citation, in either strict mode

**Status:** Accepted
**Date:** 2026-09-28

## 1. Context

The escape `<§>ID` exists so that documentation can show the *shape* of a citation without making one. [§FS-check.2.3.1](../../functional-spec/FS-check.md#231-escaped-citation-resolves) states what it means flatly: the marker is not immediately followed by the ID, so **no pass** treats it as a citation. That is true of every marker-gated pass, because each of them looks for the marker and the escape hides it behind `>`.

It was not true of the one pass that does not read the marker. Under `[reference] strict = false` a bare ID-shaped token is a citation ([§FS-check.1.1](../../functional-spec/FS-check.md#11-recognized-citations)), and the bare pass never knew about the escape, so `<§>FS-042-user-login` was recognized, failed to resolve, and produced the dangling-reference error of [§FS-check.3.1](../../functional-spec/FS-check.md#31-dangling-citation) — whose inline-code hint ([§FS-check.3.1.2](../../functional-spec/FS-check.md#312-an-illustration-in-inline-code)) names the one edit the file already contained. Following the hint changed nothing and printed the same hint again, so the finding was one no edit could clear.

This was not a corner. `grund init` writes a managed block that teaches the escape by using it ([§FS-init.2.3.8.2](../../functional-spec/FS-init.md#2382-the-worked-example-is-escaped)), so the first command a repository runs produced a tree its second command rejected — in the mode [§FS-check.1.1](../../functional-spec/FS-check.md#11-recognized-citations) recommends to a migrating repository, and only in that mode. The same block points the reader at `grund fmt --marker` as the way out of compatibility mode, and that writer had the mirror-image defect: its bare-to-marker pass spliced a marker *inside* the escape brackets, turning an inert illustration into a live dangling citation in both modes.

Both halves come from the same missing fact: the escape was a property of the marker passes rather than a property of the position.

## 2. Decision

### 2.1 The exclusion is total, not a withheld error

[§FS-check.1.1.9](../../functional-spec/FS-check.md#119-an-id-in-an-escape-position) removes the token from the recognized-citation set, not the finding from the report. Such a token is not a citation for `refs`, for unused-declaration counting ([§FS-check.4.1](../../functional-spec/FS-check.md#41-unused-declaration)), or for grounding ([§FS-check.3.6](../../functional-spec/FS-check.md#36-ungrounded-unit-opt-in)) either. This is the reach of the link-destination carve-out ([§FS-check.1.1.4](../../functional-spec/FS-check.md#114-markdown-link-destinations)), copied deliberately and for the stated reason: a repository cannot clear a finding whose only named fix is an edit `grund fmt` refuses to perform ([§FS-fmt.2.3](../../functional-spec/FS-fmt.md#23-what-is-never-rewritten)).

The narrower reading — suppress the error, keep the citation — was available and has precedent: [§FS-check.4.1.2](../../functional-spec/FS-check.md#412-an-index-entry-does-not-count) excludes a kind's own index entry from the unused count while leaving `grund refs` listing it. That precedent is deliberately not what this copies. An index row *is* a citation that happens to prove nothing about use; an escaped illustration is not a citation at all, and a tool that lists it under `grund refs` while refusing to call it one is saying two things about the same bytes. Suppressing the error alone would also have put the rule in a second place from the one `grund fmt` reads, which is how the two disagreed in the first place.

### 2.2 Both escape forms are exempt by this rule

The qualified form `<§>alias/ID` already produced no error, but not for this reason: an unmarked `alias/ID` is never a citation anywhere ([§FS-workspace.1.3](../../functional-spec/FS-workspace.md#13-the-shape-outside-a-workspace)), because the shape collides with a path. So the two spellings of one escape were exempt by two unrelated rules, one of which did not mention escapes. [§FS-check.1.1.9](../../functional-spec/FS-check.md#119-an-id-in-an-escape-position) covers both, and [§FS-workspace.1.3](../../functional-spec/FS-workspace.md#13-the-shape-outside-a-workspace) is left doing its own job for a genuinely unmarked `alias/ID` in prose.

### 2.3 The escape is the configured marker, not the character `§`

`[reference] marker` is configurable, so the escape is that marker wrapped in `<` and `>`. In a project whose marker is `@`, `<@>ID` is the escape and `<§>ID` is an ordinary bare token. Anything else would make one repository's illustration another's dangling reference.

### 2.4 One rule, read by the reader and the writer

[§FS-fmt.2.3](../../functional-spec/FS-fmt.md#23-what-is-never-rewritten) gains the escape position in its never-rewritten list, and the predicate is shared rather than duplicated ([§AR-scanner.2.3.1](../../architecture/AR-scanner.md#231-the-carve-outs-share-one-predicate)). `check` never demands an edit in a zone `fmt` will not touch; `fmt` never writes into a zone `check` reads as inert. The suggestion of [§FS-check.2.3.1](../../functional-spec/FS-check.md#231-escaped-citation-resolves) is unaffected in both modes, and stays the only thing that reports such a site — an escape whose ID resolves to a real declaration is worth naming, because it may be a live citation bracketed by accident.

## 3. Alternatives considered

**Withdraw the hint under `strict = false`.** The smaller change, and the one the report offered as its second option: stop naming the escape where the escape does not work, and name a fenced code block or a non-ID-shaped rewording instead. It was rejected because it leaves a non-strict repository with no way to illustrate a citation's shape in inline code — the one place a bare token deliberately stays live off strict mode — and because it would force [§FS-check.2.3.1](../../functional-spec/FS-check.md#231-escaped-citation-resolves) to be narrowed to a strict-mode facility. Shrinking a contract to work around a missing carve-out is the wrong direction; [§FS-check.2.3.1](../../functional-spec/FS-check.md#231-escaped-citation-resolves) already promised the behavior this change delivers.

**Suppress the dangling finding in the checker.** Filter findings whose site coincides with a recorded escape. See [§DF-escape-position-is-not-a-citation.2.1](DF-escape-position-is-not-a-citation.md#21-the-exclusion-is-total-not-a-withheld-error): it clears the error and leaves `grund refs`, the unused count and `fmt --marker` all still treating the token as a citation.

**Leave `fmt --marker` alone.** The corruption only bites a repository migrating out of compatibility mode, which is a minority of a minority. But the block `grund init` writes tells that repository to run exactly this command, and a writer that turns an illustration into a claim about a real declaration is the persisted guess [§REQ-no-wrong-citation.3](../../requirements/REQ-no-wrong-citation.md#3-no-wrong-write) forbids.

## 4. What this costs, and why it may be taken

The exclusion is total, so it takes an inbound citation away: a non-strict repository whose only mention of a declaration is an escape of that declaration's real ID stops having it cited. `grund refs` no longer lists the site and the declaration may earn `declared but never cited`; both are warnings, so they stand in place of the `success` marker ([§FS-check.2.1.3](../../functional-spec/FS-check.md#213-the-success-line)) without moving the exit code. Under `[reference] require_grounding` the file holding the escape may become ungrounded, and that finding is an **error** ([§FS-check.3.6.3](../../functional-spec/FS-check.md#363-findings)), so a tree with grounding required goes from `0` to `1`. Either way the reported verdict moves, which [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered) governs.

The route is [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), and this record is the accepted proof it requires. Its five conditions:

1. **Prior prohibition.** The old verdict violated [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution) by counting as a citation text the specification says is not one, and it violated [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms) by reporting an error whose only named fix the tool itself refuses to perform; both prohibitions already applied in the release when that verdict shipped.
2. **Accepted proof.** This record, at [§DF-escape-position-is-not-a-citation.4](DF-escape-position-is-not-a-citation.md#4-what-this-costs-and-why-it-may-be-taken).
3. **Named release.** The release notes name the verdict change and what a non-strict tree loses by it.
4. **Actionable findings.** Each finding that replaces the old verdict is located and clearable: `declared but never cited` at the declaration's heading line, the grounding finding at the file that needs a citation. The action in both cases is to write the citation the escape was standing in for — or to accept that the declaration is unused, which is what the warning is for.
5. **No new licence.** This route cannot justify anything broader: nothing here is tightened, nothing is removed, and no prohibition is invented by the correcting change. The deprecation path of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) does not fit, because there is no old spelling to keep working: the bytes stay legal and only their meaning is corrected. The mechanical migration of [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) does not fit either, because there is no edit for a repository to make — the escape is already what its author meant.

A repository that ran `grund fmt --marker --write` before this change has mangled escapes in its tree, spelled `<§>§` or `<§>[§`. This change stops new ones and repairs none; the release notes say what to grep for.

The strained requirement is carried rather than contradicted, and the change serves [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms), [§REQ-no-wrong-citation.3](../../requirements/REQ-no-wrong-citation.md#3-no-wrong-write) and [§REQ-shipped-surfaces](../../requirements/REQ-shipped-surfaces.md#req-shipped-surfaces-what-grund-ships-or-prints-resolves-where-it-lands) — the scaffold `grund init` writes now lands clean in both modes, which is the whole of the report this decision answers.
