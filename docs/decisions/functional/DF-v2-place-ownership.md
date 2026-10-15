# DF-v2-place-ownership: in v2 the row named `code` is the complement, and a file belongs to its listed row or else its deepest folder

**Date:** 2026-10-14

## status: Accepted

## 1. Context

A version-2 row may list several places, and v2 derives the scan from the places rather than from a separate `include` list ([§FS-config-v2.schema.places](../../functional-spec/FS-config-v2.md#schemaplaces-rows-and-their-places)). The owner ruled that direction on 2026-09-30 ([§DISC-core-concerns.6.4](../../discussions/proposals/2026-09-30-core-concerns.md#64-amended-after-the-verdict-v-a08-and-v-a14)) and left two choices open for agent-grounds/grund#457 ([§DISC-core-concerns.10.1](../../discussions/proposals/2026-09-30-core-concerns.md#101-the-five-decisions-the-2026-09-30-proposal-leaves-open)): what marks the complement row once a row may have no place, and which row owns a file that two places contain.

Version 1 answers the second by falling back: a file in two nested homes belongs to neither and goes to the complement ([§FS-config.3.9.2](../../functional-spec/FS-config.md#392-the-homeless-kind)), so [§FS-declarations.checks.misplaced-declaration](../../functional-spec/FS-declarations.md#checksmisplaced-declaration-misplaced-declaration-configured-kind-home) exempts it. On the shape v2's own canonical example writes, `page` at `docs` with `FS` at `docs/spec/core` inside it, that rule leaves the whole spec tree outside `FS`'s citation rules and outside the misplaced-declaration check, with no message.

## 2. Decision

### 2.1 The complement is the row named `code`

The row named `code` is the complement ([§FS-config-v2.schema.places.complement](../../functional-spec/FS-config-v2.md#schemaplacescomplement-code-is-the-complement-by-name)). It is non-citable by definition, holds source declarations, and owns every scanned file no other place claims. A `code` row with no place, or no `code` row, has the config root as its extent; listing places on `code` is the one way to narrow the scan. A row with no place is then unambiguous: a citable one is declared only in doc-comments, and a non-citable one other than `code` is a load error.

### 2.2 A listed file wins, then the deepest folder

A file listed in a row's `files` belongs to that row; otherwise to the row whose folder containing it is deepest, `code`'s folders included; otherwise to `code` ([§FS-config-v2.schema.places.ownership](../../functional-spec/FS-config-v2.md#schemaplacesownership-a-listed-file-then-the-deepest-folder)). The same place written twice is a load error located at the later entry ([§FS-config-v2.schema.places.boundaries](../../functional-spec/FS-config-v2.md#schemaplacesboundaries-what-a-place-may-name)), so no tie remains to break. Every per-file rule, misplaced declarations, indexes, stubs, citation directions and grounding, reads that one owner.

## 3. Rejected alternatives

### 3.1 A key marks the complement

`complement = true` on any row would keep v1's freedom to rename the complement. It is rejected because the one thing renaming bought, a truer label, already has its v2 home in `[presentation.kinds.code] title`; the name `code` is already reserved ([§FS-config.3.4.5.2](../../functional-spec/FS-config.md#3452-code-is-reserved-to-the-homeless-kind)); and a key needs an "exactly one row" rule that a name gives for free.

### 3.2 Keep v1's nesting rule

Letting a file in two nested places fall to the complement, with nesting under `code` as the only exception, keeps v1's rule in v2. It is rejected because it makes nesting mean "neither": the canonical example's spec tree would sit outside `FS`'s rules and outside the misplaced-declaration check, which is the blind spot this format exists to close ([§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded)).

## 4. Consequences

- A version-1 config is untouched: its homes, its nesting rule and `[scan] include` keep their meaning ([§FS-config.5.2](../../functional-spec/FS-config.md#52-every-older-version-keeps-its-meaning)).
- In version 2, a file in a nested place is checked against its owner, and the misplaced-declaration exemption for nested homes has no v2 counterpart.
- A version-2 config whose rows list places and which writes no `code` row reads the whole config root, and the files outside every place are `code`'s. Before this change such a file read only its places.
- `grund config migrate` (agent-grounds/grund#461) must report, rather than rewrite silently, each v1 arrangement this changes: nested homes whose files gain an owner, a renamed complement, `include` entries outside the config root or inside a member, homes that are outward directory links, and two kinds with the same home.

## release-note: Release note

- [§FS-config-v2.schema.places](../../functional-spec/FS-config-v2.md#schemaplaces-rows-and-their-places): **a version-2 row may list several `folders` and `files`, and the scan is derived from them.** The row named `code` is the complement; with no `code` row, or one with no place, a v2 config reads the whole config root, so a v2 file that lists places and no `code` row now reads files outside those places as `code`, where it read only its places. A file belongs to the row that lists it, or else to the row of the deepest folder containing it, so a declaration in a place nested inside another row's place is now checked against that nested row: a declaration of the wrong kind there is a `misplaced-declaration` error where version 1 checks nothing. The same place on two rows, a place outside the config root or inside a workspace member, a folder place that links out of the project, a non-citable row with no place, and `scan = false` or `citable = true` on `code` are load errors at their line. Version-1 configs are unchanged.
