# DF-section-lock-unnamed-alias: fmt drops a lock shard of an alias nothing names any more

**Date:** 2026-11-18

## status: Accepted

## 1. Context

A project records its citations into another project in `.grund/sections/<alias>/`
([§FS-section-lock.file.members](../../functional-spec/FS-section-lock.md#filemembers-citations-into-another-project)). A run that does not load the alias left that
directory alone, because it cannot judge the entries there. Once a member was renamed or
dropped, no run loaded its alias again, so the directory stayed committed for good, though
[§FS-section-lock.write.rules](../../functional-spec/FS-section-lock.md#writerules-add-refresh-drop) drops an entry nothing cites and
[§FS-section-lock.file.shard](../../functional-spec/FS-section-lock.md#fileshard-one-file-per-cited-declaration) says a file with no entry does not exist. Reproduced on
0.23.0: after `git mv api backend` and the citations moved to `§backend/…`, the root's
`fmt --write` created the `backend/` lock, and `.grund/sections/api/FS-orders.lock` stayed
while `check` and `fmt --check` passed.

## 2. Decision

A run that does not load an alias still leaves its directory alone while a citation in the
project names the alias, so a member's own run and a run with a missing import keep what
they cannot judge. Once no citation names the alias, nothing cites its entries, and
`fmt --write` drops them as it drops any uncited entry, deleting the files. `check` does
not change: a stale entry is never a finding.

The rule does not ask whether the run is a member run, so one test covers a renamed member,
a removed workspace and a member that stopped citing a sibling. The cost is the baseline of
a citation removed and later restored, which is lost as it is for an unaliased entry
([§FS-section-lock.limits.baseline](../../functional-spec/FS-section-lock.md#limitsbaseline-a-baseline-the-lock-never-saw)).

## release-note: Release note

- [§DF-section-lock-unnamed-alias](DF-section-lock-unnamed-alias.md#df-section-lock-unnamed-alias-fmt-drops-a-lock-shard-of-an-alias-nothing-names-any-more), [§FS-section-lock.file.members](../../functional-spec/FS-section-lock.md#filemembers-citations-into-another-project): **`fmt` drops the lock files of an alias nothing cites any more.** A `.grund/sections/<alias>/` left behind by a renamed or dropped workspace member, or by an import no citation uses, is listed as `section lock: drop …; deletes the file` by `fmt --check`, which now exits `1` on that tree, and deleted by `fmt --write`. **Who this breaks:** a repository that gates on `fmt --check` and still carries such a directory; one `grund fmt --write` at the workspace root, or at the project root outside a workspace, clears it. `check` is unchanged.
