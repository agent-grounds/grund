# DF-fmt-maintenance-guidance: teach the existing post-rename formatter maintenance

**Date:** 2026-10-20

## status: Accepted

## 1. Decision

Issue #824 reports successful formatter maintenance that installed help and managed
agent guidance do not teach. The top-level summary names links and section locks
([§FS-cli.2.2](../../functional-spec/FS-cli.md#22-the-top-level-help-page)); the full
page explains the post-rename remedy and gives an example
([§FS-cli.2.3.3](../../functional-spec/FS-cli.md#233-fmt-teaches-declaration-maintenance)).
Managed guidance connects refs-before-rename to authored declaration and citation-ID
edits, root-scoped formatting, verification, and committing lock changes
([§FS-init.2.3.4.9](../../functional-spec/FS-init.md#2349-declaration-blast-radius)).

This teaches existing link and lock maintenance rather than changing their scope
or approving protected section moves. Because this changes the managed block's
fixed text, use the existing block-version migration mechanism
([§FS-init.2.3.7](../../functional-spec/FS-init.md#237-the-block-version)), with both
current pairs advanced together ([§FS-init.2.3.7.1](../../functional-spec/FS-init.md#2371-the-current-version)).

## release-note: Release note

The managed agent block now teaches `grund fmt --write` after authored declaration
renames or file moves. Its versions advance from v31/v32 to v35/v36 and from
v33/v34 to v37/v38. Existing blocks are reported outdated by `grund check` and
`grund init --check`; run `grund init` to refresh them, preserving unmanaged text.
Formatter link, lock, and preview/write behavior is unchanged.
