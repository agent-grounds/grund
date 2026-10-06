# DF-block-teaches-structural-queries: the managed block teaches the structural query inline and links the query guide

**Status:** Accepted
**Date:** 2026-10-07

## 1. Context

`grund list --selector` prints `{id, section}` rows and `grund show --batch` reads exactly those rows, so selecting a set of units and expanding each one's section map is two commands piped together. Nothing an agent reads first said so: the selector was documented as the rule language, `show --batch` only in the README, and the managed block taught one coordinate at a time. Agents rebuilt the structure by parsing heading lines, or by filtering the whole citation graph, both slower and less exact (agent-grounds/grund#491).

## 2. Decision

The block's cheap-read ladder gains one bullet carrying the pipeline inline and the absolute URL of the Querying grund guide ([§FS-init.2.3.4.3.1](../../functional-spec/FS-init.md#23431-structural-queries)). The pipeline is inline because the block is read in sessions that do not follow links; the link is absolute because the block lands in repositories with no `docs/user-facing/` of their own ([§REQ-shipped-surfaces](../../requirements/REQ-shipped-surfaces.md#req-shipped-surfaces-what-grund-ships-or-prints-resolves-where-it-lands)).

## 3. Consequences

- The block's fixed text changes, so its version moves ([§FS-init.2.3.7](../../functional-spec/FS-init.md#237-the-block-version)): v12 to v14, and v13 to v15 where a rule kind is enabled, keeping rule-enabled at base plus one and every number naming one text ([§FS-init.2.3.7.1](../../functional-spec/FS-init.md#2371-the-current-version)).
- The guide and the `grund-init` skill carry the rest of the recipes; the block carries one line, because every line in it is read in every session.

## 4. Alternatives considered

| Approach | Why rejected |
|---|---|
| Link the guide without the pipeline | An agent that does not follow the link learns nothing, and the pipeline is the recipe the report needed. |
| Put every recipe in the block | `refs`, `cover` and bulk bodies are already one flag away from what the block teaches; spelling them out charges every session for questions most never ask. |

## release-note: Release note

- [§FS-init.2.3.4.3.1](../../functional-spec/FS-init.md#23431-structural-queries), [§FS-init.2.3.7.1](../../functional-spec/FS-init.md#2371-the-current-version): **the managed `AGENTS.md` block moves v12 → v14, and v13 → v15 where a rule kind is enabled.** It gains one bullet teaching `grund list --selector "<selector>" --format json | jq -c '{id,section}' | grund show --batch --toc --format json` and linking the new [Querying grund](../../user-facing/querying.md) guide. **Who this breaks:** every repository with a grund block — `grund check` reports it outdated until it is re-rendered. Migration: run `grund init`. (agent-grounds/grund#491)
