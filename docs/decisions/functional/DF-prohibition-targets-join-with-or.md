# DF-prohibition-targets-join-with-or: A prohibition's targets join with "or"

**Date:** 2026-10-27

Settles how the `### Citation directions` section joins a prohibition's targets, as [§FS-init.2.3.5.4](../../functional-spec/FS-init.md#2354-targets) specifies. Refines the target rules of [§DF-directions-render.2.2](DF-directions-render.md#22-the-rules-the-render-follows). Serves [§GOAL-agent-grounding](../../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work): the rule an agent reads is the rule `grund check` enforces.

## status: Accepted

## 1. Context

[§DF-directions-render.2.2](DF-directions-render.md#22-the-rules-the-render-follows) joined every list's entries with "and", which is right for an obligation and wrong for a prohibition. `must = ["FS", "AR"]` asks for a citation to each, but `must-not = ["FS", "AR"]` bans each on its own: `grund check` reports a citation site to either one ([§FS-check.3.12](../../functional-spec/FS-check.md#312-forbidden-citation)), so a file that cites only FS fails. The block rendered `The file **README.md** must not cite FS and AR.`, which reads as forbidding the pair, and this repository's own entrypoint carried that line (agent-grounds/grund#151, 2026-10-12).

## 2. Decision

A prohibition clause's targets join as one list of alternatives: every entry's alternatives in written order, "or" between two, the Oxford comma from three, and no parentheses, since nothing is grouped ([§FS-init.2.3.5.4](../../functional-spec/FS-init.md#2354-targets)). `must-not = ["FS", "AR"]` and `must-not = ["FS|AR"]` forbid the same citations, so they render the same `must not cite FS or AR`. An obligation keeps the grouping of [§DF-directions-render.2.2](DF-directions-render.md#22-the-rules-the-render-follows). No `grund check` verdict moves: only the text an agent reads, under the block-version bump it shares ([§FS-init.2.3.7.1](../../functional-spec/FS-init.md#2371-the-current-version)).

## 3. Alternatives considered

| Option | Why rejected |
|---|---|
| `must not cite FS and must not cite AR` | Says the same thing at twice the length, and the clause order of [§FS-init.2.3.5.3](../../functional-spec/FS-init.md#2353-the-clauses) has no slot for a repeated modal. |
| `must not cite FS nor AR` | "nor" after "not" is archaic in English and reads as a double negative to a reader for whom English is a second language. |
| `must not cite any of FS and AR` | Keeps the "and" that misled, and adds a quantifier no other clause has. |

## release-note: Release note

- A prohibition with more than one target now renders its Citation directions targets joined with "or": `must-not = ["FS", "AR"]` reads `must not cite FS or AR`, where it read `must not cite FS and AR`, which said only the pair was forbidden. The managed block's versions advance from v39/v40 to v43/v44 and from v41/v42 to v45/v46, the same pairs the first rung's section-heading wording moves them to. Existing blocks are reported outdated by `grund check` and `grund init --check`; run `grund init` to refresh them, preserving unmanaged text. No verdict over the repository's own files changes.
