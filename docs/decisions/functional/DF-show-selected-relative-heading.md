# DF-show-selected-relative-heading: A read of a selected section names the section it read

**Date:** 2026-10-25

Settles how a read of a selected section prints that section's own heading when the author wrote it relative, as [§FS-declarations.section.relative.shown.selected](../../functional-spec/FS-declarations.md#sectionrelativeshownselected-a-selected-sections-own-heading-names-its-path) specifies. Serves [§GOAL-token-economy](../../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file): a citation is answered in one `grund <ID>` read, and that read has to say which section it answered. Reported as [grund#1009](https://github.com/agent-grounds/grund/issues/1009) and approved by its author as proposed.

## status: Accepted

## context: Context

#643 let a heading write only the component it adds: `### .2: Ceiling` under `## retry: Retry schedule` is `retry.2`. Its answer 5 split the surfaces in two ([§DF-relative-section-headings.decision.shown](DF-relative-section-headings.md#decisionshown-canonical-where-a-coordinate-is-copied-bytes-everywhere-else)). The section map and the title hover print the canonical path, because a reader copies a coordinate from them, and every read of the text prints the file's bytes.

The selected section's own heading falls between the two. It is the first line of every read of a section, and [§FS-show.2.2](../../functional-spec/FS-show.md#22-section) calls it the slice's self-label. A relative heading labels nothing on its own. `grund FS-billing.retry.2 --full` opened with `### .2: Ceiling`, the same first line `FS-payouts.retry.2` printed, and nothing in the text or `md` output named `retry.2`. Only the JSON fields and the parent's `--toc` did. The agent in #1009 spent three extra reads finding out which address its one read had answered. `--toc` of the same section already rewrote that very line, so the four modes of one read disagreed about it.

## decision: Decision

### decision.line: The selected section's own first line, and nothing else

A read that selects a section prints its own heading with the canonical path in place of the relative one, at its written depth and with its punctuation and title kept. That holds in every mode, in `text`, `md` and the JSON `body`, for a direct coordinate and `--section`, and for Markdown and doc-comment bodies ([§FS-declarations.section.relative.shown.selected](../../functional-spec/FS-declarations.md#sectionrelativeshownselected-a-selected-sections-own-heading-names-its-path)). The line is expanded the way the section map already expands it, from the recorded parent, so the read and the map agree and the expansion stays deterministic ([§FS-non-goals.14](../../functional-spec/FS-non-goals.md#14-generated-summaries-token-saving-inside-check)).

Every other heading keeps #643's answer. A heading inside the slice below the first line, every heading of a whole-declaration read, and a heading already written in complete form print as written. The anchor stays the anchor of the heading as written, because `fmt` writes links to that anchor ([§REQ-one-answer.writes.anchors](../../requirements/REQ-one-answer.md#writesanchors-shows-anchor-is-the-anchor-fmt-writes)). The map and the hover are unchanged.

### decision.compatibility: A pre-1.0 change of output, argued here

No verdict moves. `check` reports what it reported, and no file, lock or link is written. What changes is output a caller may have read. The JSON `body` of a selected relative section's read now opens with the canonical heading. `list --size` then measures a few more bytes, and an opt-in byte budget for leads can newly warn. Carrying the old line behind a switch would mean two display rules and two measurement rules for one heading. Before `1.0` that costs more than the change, which is the licence [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise) grants and this record argues.

## alternatives: Alternatives considered

- **A header line before the body that names the ID, path and line.** It would leave every body byte alone, but it adds a second output shape to `text`, whose definition is "the body only" ([§FS-show.3.1.1](../../functional-spec/FS-show.md#311-text)). It also repeats metadata the JSON form already carries, and it costs bytes on every read, relative heading or not.
- **Relabel every relative heading in the slice.** The section map already does that for a reader who wants coordinates. A body read that rewrote interior headings would stop being the author's text, which is what #643 kept on purpose.
- **Leave it, and point readers at `--toc` or JSON.** That is the workaround #1009 measured: a second call on a task that needed one.

## release-note: Release note

- [§FS-declarations.section.relative.shown.selected](../../functional-spec/FS-declarations.md#sectionrelativeshownselected-a-selected-sections-own-heading-names-its-path): **A read of a selected section names the section it read.** When a section's heading is written relative, as `### .2: Ceiling` under `## retry: Retry schedule`, `grund FS-billing.retry.2` and `grund FS-billing --section retry.2` now open with `### retry.2: Ceiling` in every mode and format, as `--toc` already did. Interior headings, whole-declaration reads, anchors and `--toc` are unchanged. **What changes for a caller:** the JSON `body` of such a read now starts with the canonical heading, while `anchor`, `section`, `path` and `line` stay as they were. `grund list --size` counts the extra bytes, so a version-1 `lead_size_warning` byte budget can newly warn on a lead that sat at its limit. Word budgets and pass/fail status do not move. (agent-grounds/grund#1009)
