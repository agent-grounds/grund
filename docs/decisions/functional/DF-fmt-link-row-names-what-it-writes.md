# DF-fmt-link-row-names-what-it-writes: a `markdown link` row of fmt's preview names each link before and after

**Date:** 2026-11-16

## status: Accepted

## 1. Context

`grund fmt` with neither `--check` nor `--write` "previews every change --write would apply", and until this decision a row that wrote a link said only `path:line: markdown link`. [§FS-fmt.3.6](../../functional-spec/FS-fmt.md#36-an-expansion-or-a-link-names-the-text-it-writes) justified that: a link moves markup around an ID token it leaves byte-identical, so a wrong one is something `grund check` names. That holds for the ID and not for what the link adds. A link writes a destination and an anchor the source never held, and `check` deliberately reads neither ([§FS-non-goals.1](../../functional-spec/FS-non-goals.md#1-markdown-link-validation)): it passed a link with no anchor and one with a stale anchor alike. So the text fmt invents for a link was reviewed nowhere — not in the preview, which did not show it, and not after the write.

Issue #1378 met that in a real file. Twenty new rows of a sources table needed anchors, the file held other pending rewrites so `--write` on all of it was not an option, and the preview said `markdown link` twenty times. The reporter worked out the heading-slug rule from two existing entries and wrote the anchors by script.

## 2. Decision

### 2.1 The row names the whole link, before and after

A `markdown link` row is followed by `: ` and one item per link the line writes, in source order: the text the write replaces, `→`, and the text it writes ([§FS-fmt.3.6](../../functional-spec/FS-fmt.md#36-an-expansion-or-a-link-names-the-text-it-writes)). A re-pointed link reads `[<§>ID](old) → [<§>ID](new)` and a fresh one `<§>ID → [<§>ID](new)`. A link already right writes nothing and is not listed. Where the repeated-anchor note fires it comes first, unchanged, then `; ` and the items ([§DF-repeated-heading-anchor.decision.row](DF-repeated-heading-anchor.md#decisionrow-the-changed-link-names-the-versions)).

Each left side is one span the link pass replaces, read on the line as the row's earlier kinds leave it, so replacing each in turn gives the line `--write` leaves ([§FS-fmt.3.6](../../functional-spec/FS-fmt.md#36-an-expansion-or-a-link-names-the-text-it-writes)), and each side ends at the `)` that closes its `(`, so a title holding `, ` stays inside its item ([§FS-fmt.3.6](../../functional-spec/FS-fmt.md#36-an-expansion-or-a-link-names-the-text-it-writes)). A caller can apply one line by hand, and an anchor gets the one review it can get before it lands.

### 2.2 The items are a reading of what the write writes

The link pass keeps one record per link it writes — the span it read and the text it puts there — and the `--write` count, the repeated-anchor note and the items are three readings of it, for the ordinary pass and the index carve-out alike. The preview is the write with the write withheld ([§FS-fmt.7.3](../../functional-spec/FS-fmt.md#73-preview-equivalence)), so an item is the bytes the write splices and not a second derivation of them.

## 3. Alternatives considered

**The destination alone** (`markdown link: SRC-501-alpha.md → SRC-501-alpha.md#…`). Shorter, and it cannot be applied where another link on the line has the same destination or one that starts with it — the repeated-anchor case, where an already-right `FS-a.1` link and a stale `FS-b.1` link carry the same old destination. The citation in the item says which link it is, and the whole destination shows a moved file part or a dropped title, not only a moved anchor.

**A unified diff, or the line before and after.** A second output shape beside the one-row-per-changed-line report tools parse ([§FS-errors.2.1](../../functional-spec/FS-errors.md#21-located-finding)), and one that repeats the whole line to show a few bytes.

## 4. Consequences

Rows grow by one item per link written, so a table line with many links prints many items; that is the price of showing what is written, and it is bounded by the line. `--write` output, its totals, the exit codes and `counts` do not change. Every row an earlier release printed is a prefix of its row now ([§FS-errors.3](../../functional-spec/FS-errors.md#3-message-text)), so a prefix match, a row count and the exit code keep working.

## release-note: Release note

- [§FS-fmt.3.6](../../functional-spec/FS-fmt.md#36-an-expansion-or-a-link-names-the-text-it-writes), [§FS-distribution.3.2.2](../../functional-spec/FS-distribution.md#322-typed-readonly-host-records): **a `markdown link` row of `grund fmt`'s preview and `--check` now names each link the line writes, before and after.** `docs/sources/README.md:5: markdown link` becomes `docs/sources/README.md:5: markdown link: [<§>SRC-501-alpha](SRC-501-alpha.md) → [<§>SRC-501-alpha](SRC-501-alpha.md#src-501-alpha-alpha-source-report)`, and a fresh link reads `<§>ID → [<§>ID](destination)`; after the repeated-anchor note the items follow `; `. The library's `FmtData.changes[].label` carries the same text in preview and write results. **What does not move:** what `--write` writes, its summary, the exit codes, `counts` and `lockFile`; every earlier row is a verbatim prefix of its new row. **Who this breaks:** a consumer that matches a whole row exactly (`grep -x 'path:N: markdown link'`) or compares `label` with `==`; read `counts` instead, or match the prefix. Closes [issue #1378](https://github.com/agent-grounds/grund/issues/1378).
