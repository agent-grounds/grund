# DF-one-site-order: a duplicate's sites have one order, check's, whichever command lists them

**Date:** 2026-11-09

Decides the key behind [§FS-errors.4.1.1](../../functional-spec/FS-errors.md#411-the-order-of-a-duplicates-sites). Reported as [grund#1219](https://github.com/agent-grounds/grund/issues/1219).

## status: Accepted

## 1. Context

`check` sorted a duplicate's sites by the path as scanned and the line as a number. `show` and `refs` sorted the same sites by the rendered `path:line` string. The two keys part on a line number of a different digit count (`:10` before `:9`), on a path that prefixes another (`.` sorts before `:`), and on a run from a subdirectory (a rendered `../` sorts first). Nothing was wrong about the sites or the exit codes. But a program comparing the two `sites` arrays saw a difference that meant nothing, and whether it saw one depended on a file's name.

[§FS-show.2.2.1](../../functional-spec/FS-show.md#221-ambiguous-id) and [§FS-errors.2.3.1](../../functional-spec/FS-errors.md#231-ambiguity-and-hint-lines) wrote the string order down, so for most trees `show` followed its own sentence. For a stub's target it did not: [§FS-declarations.stubs.pairing.3](../../functional-spec/FS-declarations.md#stubspairing3-a-declaration-reached-through-stubs-is-named-at-its-target) already says an ambiguity refusal's sites sort per [§FS-check.2.1](../../functional-spec/FS-check.md#21-report-format). The four e2e cases that recorded `source.rs:11, source.rs:6` (`show-stub-unscanned-target-ambiguous-id`, `refs-stub-unscanned-target-ambiguous-id` and each one's `-json` twin) pinned a breach of that point, not an order the specification chose.

## 2. Decision

`check`'s key wins, and [§FS-errors.4.1.1](../../functional-spec/FS-errors.md#411-the-order-of-a-duplicates-sites) is the one place that says so. It orders a duplicate's sites by the name the site is reported by, relative to its project root and taken before it is rendered against the report base, then by the line as a number. `check` still sorts after it settles each site on its first name ([§FS-scan.identity.findings.content](../../functional-spec/FS-scan.md#identityfindingscontent-a-content-finding-is-one-line-naming-every-name)), so a run handed the second name of a file keeps the finding where a whole run puts it. `show` and `refs` move to that order, and a host's query failure carries the same `sites` ([§FS-distribution.3.3.2](../../functional-spec/FS-distribution.md#332-operational-failures-and-invalid-arguments)), so Node and Python move with them.

The point covers a duplicate's sites only. A content finding's `(also at …)` names follow spelling order and a value mismatch is left alone, and how a path is spelled is a separate matter.

## 3. Rejected alternative: check moves to the rendered string

It would keep the two `show` sentences, but `:10` would come before `:9` everywhere, `check` would locate findings at different sites, and that location would start following the directory a run starts from.

The cost of the choice made: with `--path-base=invocation` a printed list may not look sorted, because `a.md` comes before `../z.md`. It is ordered by where the files are in the project, not by how they are spelled from where the run stands.

## 4. Compatibility

This reorders the sites inside an existing message, so [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered) applies to it, and no verdict moves, so <§>REQ-backwards-compatibility.5 is not engaged. The change ships under the pre-1.0 licence of [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise), without an old form kept beside the new. There is no form to keep a user on: a sort order has no alias, and a flag that restored `:10` before `:9` would preserve the defect. Who is affected is narrow, a consumer matching `show`'s or `refs`' exact site order for an ID whose order changes, and `check`'s output does not move.

It is not a wording change under [§FS-errors.requirements.prefix-windows](../../functional-spec/FS-errors.md#requirementsprefix-windows-a-wording-change-keeps-the-old-message-whole-until-its-window-closes--realized-directional-for-the-next-change), and that requirement keeps its one named exception. The text of the message is unchanged: the `ambiguous ID: <ID> (declared at ` prefix survives byte for byte, and only the values filled in after it change places, as they already did with a file's name or a run's directory. The code, the severity and the exit do not move. No window applies, and a window that kept `:10` before `:9` would help nobody.

## release-note: Release note

- [§DF-one-site-order](DF-one-site-order.md#df-one-site-order-a-duplicates-sites-have-one-order-checks-whichever-command-lists-them), [§FS-errors.4.1.1](../../functional-spec/FS-errors.md#411-the-order-of-a-duplicates-sites): `show` and `refs` now list an ambiguous ID's sites in `check`'s order.
