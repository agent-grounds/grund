# DF-configured-title-metadata: kind titles are separate target metadata

**Status:** Accepted by issue 243's checkpoint-10 supervisory ruling
**Date:** 2026-09-19

## 1. Context

[§FS-config.3.4.3](../../functional-spec/FS-config.md#343-title) already promises
configured-kind metadata on show JSON, refs JSON and hover. Those surfaces
omitted it, and [§FS-lsp.1.2](../../functional-spec/FS-lsp.md#12-hover-preview)
also promised whole-hover preview parity and a one-line declaration hover.
The approved coverage work exposed the omission; a list-summary assertion
cannot prove these surfaces. The supervisor settled the missing wire details
and hover conflict at checkpoint 10. This is that bounded ruling, not a new
author response or a claim that the original unchanged-output prediction held.

## 2. Decision

Append an optional `kind_title` string after the existing fields of each
successful show JSON object, including E2E manifests, JSON values, sections,
all read modes and batch results. Append it to each detailed refs record, but
not file summaries. Use the resolved target kind's effective configuration,
including defaults; omit it for `None` and preserve a present empty string.
Caller and citer configuration cannot supply the title of a workspace target.
Refs still accepts a grammatically valid undeclared ID.

Successful hover appends two newlines, `Kind: ` and a CommonMark code span of
the title. Preserve the existing preview or title-and-usage content, including
its trailing whitespace. Only preview content is linkified; the metadata is
literal even when it contains backticks or citation-shaped text. With no
effective title, hover stays byte-identical. Preview parity now applies before
this paragraph and editor linkification; CLI Markdown never gains metadata.
Ranges, counts, navigation and diagnostic suppression retain their meanings.

## 3. Compatibility and alternatives

This uses the narrow pre-1.0 choice in
[§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise).
The configured-title requirement was promised; its absence was a defect, not
undefined behavior. The changed JSON bytes and full-hover body are disclosed
in the changelog. Old JSON fields, summary shapes and Markdown content remain
available, and existing callable signatures and exhaustively constructible
Rust records remain unchanged. Additive wrappers can carry target context
without mandatory new fields on `RefHit`, `RefsOutput`, `RefsOutcome` or editor
records; `lsp_title_hover_body(title, usage)` retains its original output.

A second output mode solely to preserve the missing metadata would duplicate
single, batch, alternate-show, refs and hover rendering, with a permanent
selection burden larger than this optional field and paragraph. A null field
would change untitled records without conveying metadata. Repurposing `body`,
section `title` or citation `text` would destroy authored data. Giving refs
summaries metadata would alter an aggregate surface whose citing-file meaning
is already complete. These alternatives are rejected, not hidden behind an
exception to the coverage gate.

## 4. Consequences

JSON consumers may read the final field or continue reading existing fields.
Full hover gains separate metadata while its preview remains CLI-parallel.
Target selection is shared with the query or snapshot, with no fetch, network
execution or extra per-hover filesystem scan. Exact JSON/API and editor-handler
assertions pin both configured and absent titles, alternate renderers,
workspace ownership, escaping, and preserved fields and ranges.
