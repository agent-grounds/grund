# DF-config-explain-dotted-answered: grund config explain answers a dotted path with exactly one reading, as its bracketed spelling is

**Date:** 2026-10-25

Moves [§FS-config-explain.1.1](../../functional-spec/FS-config-explain.md#11-targets) and [§FS-config-explain.5](../../functional-spec/FS-config-explain.md#5-a-target-the-version-does-not-know). Serves [§REQ-discoverable-offline](../../requirements/REQ-discoverable-offline.md#req-discoverable-offline-the-installed-grund-answers-for-every-setting-and-every-check-code): the binary answers what it knows under the spelling people reach for first. Reported as [grund#1005](https://github.com/agent-grounds/grund/issues/1005).

## status: Accepted

## 1. Context

[grund#904](https://github.com/agent-grounds/grund/issues/904) reported that `grund config explain schema.sources` fell through to "knows no table, key or check code". It offered two remedies: answer the dotted path, or, short of that, refuse it with a line that names the bracketed spelling that answers. [grund#926](https://github.com/agent-grounds/grund/pull/926) took the second. Answering widens what the command accepts, so the same input moves from exit `1` to exit `0`, and #926 called that a contract move that belongs in a planned ticket of its own. This is that ticket.

The refusal still cost one call per dotted target, and it broke a chain. In the runs #1005 reports, `grund config explain schema.kinds.ADR && grund config validate .` skipped `validate`, which had to be run again. The same report found two failures that named the wrong thing. A bare key that two tables hold, `exclude`, got the line for a name nothing knows. `schema.kinds.id_format` got a hint for a kind named `id_format` instead of the key. The author approved the plan on #1005 on 2026-10-25 ([comment 6391028993](https://github.com/agent-grounds/grund/issues/1005#issuecomment-6391028993)): "Answer a dotted path that has exactly one reading. Keep exit 1 for a bare name and for a path with two readings, listing every reading."

## 2. Decision

- A path with exactly one reading against the version's own tables is answered as its bracketed spelling is: the same stdout, its `Next:` line included, and exit `0`. One segment is a path when it spells a table from the top.
- A path with more than one reading stays exit `1` and names every reading. Where a segment fills a placeholder, the path also reads with the name left out, and that reading is named first, so `schema.kinds.id_format` names the key before the kind.
- A bare name stays exit `1` even where one table holds it, and names every table that holds it. A dotted path is a full address, so its answer does not move when the vocabulary grows elsewhere. A bare name is not: answering `sources` because one table ends in it today would change that answer the day a second table does.
- The exit code moves in place, under the pre-1.0 licence of [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise). It moves from "found nothing" to an answer, on a query, and no grund surface reads that exit `1`. No verdict moves, because `explain` judges no tree, so this is not a verdict correction and the route of section 5 of `REQ-backwards-compatibility` does not apply.

[§DF-config-explain](DF-config-explain.md#df-config-explain-the-installed-grund-explains-its-own-settings-and-codes-under-grund-config) stays as it is: the subcommand, its rungs, and what it reads of the file do not move.

## 3. Consequences

- A script that ran `grund config explain` on a dotted path to learn whether the spelling is unknown now gets exit `0` for one that names a table or a key. Bracketed targets, codes and top-level keys answer as before.
- The line #926 added for a path with one reading is no longer printed, because that path is answered.
- With no config file both versions answer, so a dotted path that one version answers is exit `0`, as a bracketed target already is ([§FS-config-explain.2](../../functional-spec/FS-config-explain.md#2-which-version-answers)).
- A failure that used to read "knows no table, key or check code" for a name some table holds now names those tables. A script that matched that line for `exclude` or `sources` stops matching. These are query failures, not check findings, so no finding's stable phrasing moves.

## 4. Alternatives considered

| Approach | Why rejected |
|---|---|
| Keep the refusal and its hint, as #926 shipped it | It costs one call per target and breaks `&&` chains, for a spelling grund has already resolved. |
| Answer a bare name that exactly one table holds | That answer moves the day another table gains a key of the same name. |
| Answer the longest-run reading of a path with two readings | `schema.kinds.id_format` would answer the row of a kind named `id_format`, which the project may not declare. `explain` reads only the version, so it cannot tell. |
| Let the key reading win outright | A kind that really is named `title` could no longer be reached by its dotted path. |

## release-note: Release note

- [§FS-config-explain.1.1](../../functional-spec/FS-config-explain.md#11-targets): **`grund config explain` answers a dotted path that names exactly one table or key.** `grund config explain schema.sources` was refused with exit 1 and a line naming `'[schema.sources]'`. It now prints what `grund config explain '[schema.sources]'` prints, with exit 0. **Who this affects:** a script that read exit 1 from a dotted target as "unknown". A path that reads two ways, such as `schema.kinds.id_format`, and a bare name, such as `exclude`, still exit 1, and they now name every spelling that answers. Closes [issue #1005](https://github.com/agent-grounds/grund/issues/1005).
