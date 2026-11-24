# DF-trigger-sentence-names-the-id: The block's trigger sentence names the ID that follows the trigger

**Date:** 2026-11-16

Settles how the managed block teaches the `$$`-trigger, as [§FS-init.2.3.8](../../functional-spec/FS-init.md#238-substituted-content) specifies, and why that moves the block's version ([§FS-init.2.3.7](../../functional-spec/FS-init.md#237-the-block-version)). Serves [§GOAL-token-economy](../../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file): a reader who follows the block should not spend time checking an install that works. Reported as [grund#1370](https://github.com/agent-grounds/grund/issues/1370).

## status: Accepted

## 1. Context

A grund-aware editor turns the trigger into the marker once a token with an ID's shape follows it, and leaves a bare trigger alone so money, shell variables and LaTeX stay as typed ([§FS-lsp.1.4](../../functional-spec/FS-lsp.md#14-live-trigger-transform)). The block said "Type `$$` in a grund-aware editor and it becomes `§`." A reader who typed `$$` saw nothing happen and took a working server for a broken one.

## 2. Decision

### 2.1 Fix the words, not the editor

The sentence names the ID that follows, with an example built from the configured trigger and the guided kind's effective format, and says the trigger converts once the ID has its kind's shape. It never says the ID must be recognized: an ID no declaration carries converts too. The editor keeps its behaviour. Under a version-1 config with `[reference] strict = false`, where a bare ID-shaped token is a citation, the sentence carries no example, so the fresh block stays clean in its host repository.

### 2.2 What it costs

The block's fixed text changes, and 0.21.0 already wrote v43–v46, so the change cannot share those pairs: both advance four, to v47/v48 and v49/v50 ([§FS-init.2.3.7](../../functional-spec/FS-init.md#237-the-block-version)).

## 3. Consequences

- A reader who follows the block types an ID after the trigger and sees it convert.
- Every block 0.21.0 and 0.22.x wrote is reported outdated until `grund init` re-renders it.

## 4. Alternatives considered

- **Convert a bare trigger**: makes the old sentence true, but rewrites money and prose that [§FS-lsp.1.4](../../functional-spec/FS-lsp.md#14-live-trigger-transform) leaves alone on purpose.
- **A fixed `FS` example**: shorter to render, but in a project without `FS` the example would not convert as the sentence says.

## release-note: Release note

- [§FS-init.2.3.8](../../functional-spec/FS-init.md#238-substituted-content): **The managed block says the trigger converts once an ID follows it.** The `grund init` block now reads, for the default trigger and a project whose kind is `FS`, "Type `$$` followed by an ID, e.g. `$$FS-user-login`, in a grund-aware editor; the trigger becomes `§` as soon as the ID has its kind's shape." The example follows the project's own trigger, kind and ID format. A version-1 config with `[reference] strict = false` gets the sentence without the example. The block moves from v43/v44 and v45/v46 to **v47/v48** and **v49/v50**, so `grund check` reports a block an earlier release wrote as outdated. **What an upgrader does:** run `grund init` once to re-render the block; unmanaged content is kept. (agent-grounds/grund#1370)
