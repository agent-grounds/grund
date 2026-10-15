# DF-scoped-guidance: a version-2 row's guidance is written above its places, with a root pointer for agents that do not load nested files

**Date:** 2026-10-14

## status: Accepted

## 1. Context

Every kind's citation directions, grounding clause and chapter rules render into one managed block in the root `AGENTS.md` ([§FS-init.2.3.5](../../functional-spec/FS-init.md#235-citation-directions)), so every session reads every kind's rules to find the few that govern the file it is writing ([§GOAL-token-economy](../../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file)), and a declaration's shape is learned only from a later `grund check` ([§FS-fields.requirements.record](../../functional-spec/FS-fields.md#requirementsrecord-the-agent-and-a-schema-view-read-the-shape-the-checks-read--realized)). Version 2 gives each row its places ([§FS-config-v2.schema.places](../../functional-spec/FS-config-v2.md#schemaplaces-rows-and-their-places)), so the directory above them is known. agent-grounds/grund#460 asked for the guidance there. Its proposal, round 1, was approved by the owner on 2026-10-14 (agent-grounds/grund#460, comment 6171720050), with four additions recorded below, and it settles decisions 3, 4 and 5 of [§DISC-core-concerns.10.1](../../discussions/proposals/2026-09-30-core-concerns.md#101-the-five-decisions-the-2026-09-30-proposal-leaves-open).

## 2. Decision

### 2.1 Decision 3: `[presentation] rules = "home" | "root"`, default `home`

A version-2 config renders each row's guidance at the deepest directory holding its places ([§FS-init.2.3.5.11](../../functional-spec/FS-init.md#23511-scoped-guidance-a-rows-guidance-lands-above-its-places)); `rules = "root"` renders everything at the root, byte for byte as today ([§FS-init.2.3.5.11.5](../../functional-spec/FS-init.md#235115-rules--root-and-every-version-1-config-render-at-the-root)); a version-1 config has no key and always renders at the root. `grund config migrate` writes `rules = "root"` into the file it migrates, so a migrated config writes the files its v1 config wrote ([§FS-config-v2.mapping](../../functional-spec/FS-config-v2.md#mapping-what-the-records-keep-apart)). The migrator itself is agent-grounds/grund#461's.

### 2.2 Decision 4: Claude reads a scoped file through a per-directory symlink

Where the root `CLAUDE.md` or `GEMINI.md` is a symlink to `AGENTS.md`, each destination gets the same symlink ([§FS-init.2.3.5.14.3](../../functional-spec/FS-init.md#235143-companions-in-a-scoped-directory)). The scoped bytes do not differ by agent, and [§REQ-agents-md.1](../../requirements/REQ-agents-md.md#1-one-source-symlinked-companions) already delivers the root block to Claude through a symlink, so one mechanism serves both.

### 2.3 Decision 5: fields render in the same block as the kind's rules

A kind's declaration shape renders in the block of its destination, beside its citation directions and chapter rules, in the words of `grund schema` ([§FS-init.2.3.5.12.2](../../functional-spec/FS-init.md#235122-declaration-shapes)), so a directory's block says both what a declaration there looks like and what it cites.

### 2.4 The block version

A config that renders a scoped file or a shape selects the v21/v22 pair; every other config keeps v19/v20 byte for byte ([§FS-init.2.3.7.1](../../functional-spec/FS-init.md#2371-the-current-version)). This follows the precedent that the rule-enabled block is the base plus one, a version the config selects.

### 2.5 The owner's additions

The approval added four things to the same change: a runnable `examples/scoped-guidance/` pinning the files `init` writes for the proposal's config; both copies of the `grund-init` skill saying where scoped guidance lands, telling the agent to read the scoped `AGENTS.md` of the directory it writes in when its tool has not loaded it, and naming `rules = "root"` as the opt-out; an `init` report that names each scoped file and companion symlink it creates, updates or removes ([§FS-init.2.3.5.14.4](../../functional-spec/FS-init.md#235144-the-report-names-every-scoped-file-and-link-it-touched)); and the loading table below, where each row either cites the tool's own documentation with the date it was read or says "not verified".

## loading: Which agents load a scoped file by themselves

A session started at the repository root, editing `docs/spec/core/x.md`, with `docs/spec/AGENTS.md` and its `CLAUDE.md` and `GEMINI.md` symlinks in place. Each row was read from the tool's own documentation on 2026-10-14; a row that could not be confirmed there says so. No row is assumed.

| Agent | What its documentation says | Documentation, read 2026-10-14 |
|---|---|---|
| Claude Code | Loads a subdirectory's `CLAUDE.md` once it reads, writes or edits a file in that subdirectory; automatic through the symlink. | <https://code.claude.com/docs/en/memory> |
| Codex | Builds its instruction chain once at startup, from the project root down to the working directory, and none below it. Not automatic: it follows the root pointer, or is started in the directory (`--cd docs/spec`). | <https://learn.chatgpt.com/docs/agent-configuration/agents-md> |
| Gemini CLI | When a tool accesses a file, scans that file's directory and its ancestors for `GEMINI.md`; automatic through the symlink. `context.fileName` may name `AGENTS.md` instead. | <https://geminicli.com/docs/cli/gemini-md/> |
| Cursor | Applies an `AGENTS.md` in a subdirectory when working with files in that directory or its children; automatic. | <https://cursor.com/docs/context/rules> |
| Windsurf | Applies a subdirectory's `AGENTS.md` only when Cascade reads or edits files inside that directory; automatic. | <https://docs.devin.ai/desktop/cascade/agents-md> |
| Pi | Loads context files from the agent directory, the working directory and its parents at startup; nothing below the working directory. Not automatic: it follows the root pointer. | <https://github.com/badlogic/pi-mono/blob/main/packages/coding-agent/docs/configuration.md> |
| GitHub Copilot | Not verified. The repository-instructions page says the nearest `AGENTS.md` takes precedence but not when a nested one is read, and VS Code's nested `AGENTS.md` support is off by default. It follows the root pointer. | <https://docs.github.com/en/copilot/how-tos/configure-custom-instructions/add-repository-instructions>, <https://code.visualstudio.com/docs/copilot/customization/custom-instructions> |
| Zed | Not verified. Its instructions page describes one project-level file and nothing in subdirectories. It follows the root pointer. | <https://zed.dev/docs/ai/instructions> |

An agent that does not load the nested file by itself is served by the root block's `### Scoped guidance` list and the instruction beside it to read the scoped file before writing there ([§FS-init.2.3.5.13](../../functional-spec/FS-init.md#23513-the--scoped-guidance-pointer)). That is the fallback the owner approved. A team that needs every agent to see every rule without that read sets `rules = "root"`.

## 3. Rejected alternatives

- **`init` creates the destination directory.** Bare `init` promises no directory under `docs/`, `tests/` or a kind home ([§FS-init.3.6](../../functional-spec/FS-init.md#36-bare-init-writes-the-config-and-the-entrypoints-and-nothing-in-a-home)). A missing directory renders at the root instead, and `check` reports the drift once it exists ([§FS-init.2.3.5.11.4](../../functional-spec/FS-init.md#235114-a-destination-directory-that-does-not-exist-renders-at-the-root)).
- **Walking the tree for managed blocks to find orphans.** It costs a full walk on every `check` and could touch a nested project's own `AGENTS.md`. The root block's list on disk is the record instead ([§FS-init.2.3.5.13.1](../../functional-spec/FS-init.md#235131-the-list-on-disk-records-what-init-owns)).
- **`.claude/rules/` path-scoped files for Claude.** A second delivery mechanism for bytes that do not differ by agent; the symlink is the one [§REQ-agents-md.1](../../requirements/REQ-agents-md.md#1-one-source-symlinked-companions) already uses.
- **One universal block bump.** It would change every version-1 repository's bytes when version-1 content does not change.
- **One layout for everyone.** It would put new files into every repository on its next `init`; `rules = "root"` and version 1 keep the root layout.

## release-note: Release note

- [§FS-init.2.3.5.11](../../functional-spec/FS-init.md#23511-scoped-guidance-a-rows-guidance-lands-above-its-places), [§FS-config-v2.presentation](../../functional-spec/FS-config-v2.md#presentation-the-presentation-concern): **a version-2 repository's guidance moves into the `AGENTS.md` above each kind's places, and its managed block moves v19 → v21, or v20 → v22 where a rule kind is enabled.** `[presentation] rules` is now executed, default `home`. **Who this breaks:** every version-2 repository with a row whose citation rule, grounding clause, field or chapter rule lands outside the root, and every one that declares a field. `grund check` reports `agents-init` until `grund init` runs; that run writes `AGENTS.md` files (and their `CLAUDE.md`/`GEMINI.md` symlinks) into the destination directories, and those bullets leave the root block, which points to them under `### Scoped guidance`. Migration: run `grund init`; to keep everything in the root file, set `[presentation] rules = "root"`. Version-1 repositories are unchanged. (agent-grounds/grund#460)
