# DF-format-aware-kind-collisions: Admit prefix-related kinds when their formats distinguish them

**Date:** 2026-11-16

## status: Accepted

The author approved issue #1357's proposal on 2026-11-16. Tracker prefixes should
coexist with the standard kinds in one project, under the effective-format
contract of [§FS-config.3.2](../../functional-spec/FS-config.md#32-id--id-grammar).

## 1. Admission

Replace bare-name prefix-freedom with the ordered name-boundary and head checks
in [§FS-config.3.4.5.1](../../functional-spec/FS-config.md#3451-citable-names-and-formats-must-distinguish-their-tokens).
Heads are defined once ([§FS-config.3.2.9](../../functional-spec/FS-config.md#329-kind-head)).
Heads alone would admit `GR` beside `GR-X` with distinct separators and silently
lose malformed citations that match both name boundaries. Comparing arbitrary
compiled patterns would add a separate overlap analysis without serving this ask.
Duplicate names and non-citable exemptions retain their meanings.

## 2. Compatibility and selection

This expands configuration acceptance in both readers; older binaries still
refuse the newly accepted prefix pairs. No previously accepted config is newly
refused, including non-prefix pairs with non-leading formats. No migration,
config-version bump, goal change or requirement change is needed.

Collision errors retain every existing byte as a prefix, appending a reason
that distinguishes a name-boundary collision requiring renaming from unknown
or overlapping heads requiring unambiguous formats. Merely inserting an arbitrary
separator does not guarantee that overlapping heads become disjoint.

Canonical format selection uses the parser's kind, including separator-free
formats and shorthand. This also corrects a reachable existing behavior:
an alias-qualified `$$` shorthand with a per-kind literal different from the
repository default could use the default's literal boundary and consume a
trigger before an off-format token. Both formatter and LSP use the shared
`id_token_end_at` path. They must now leave that trigger untouched, just as
for its unqualified spelling. The regression uses an already-loadable config
with unrelated kind names; this correction is not limited to newly admitted pairs.

## release-note: Release note

- Prefix-related citable kinds such as `GR` and `GRUND` can now coexist when their ASCII name boundaries and effective format heads distinguish tokens; older grund versions refuse these configs. Collision errors preserve their previous text as a prefix and append the applicable remedy. Alias-qualified `$$` shorthand triggers and LSP transforms now use the matched kind's own format literals, so an off-format continuation no longer consumes its trigger through the repository default's boundary rule.
