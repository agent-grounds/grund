# DF-block-teaches-unused: The managed block teaches `grund list --unused`

**Date:** 2026-10-23

Settles whether the managed block's cheap-grounding list names `grund list --unused`, as [§FS-init.2.3.4.3](../../functional-spec/FS-init.md#2343-cheap-grounding) specifies. Serves [§GOAL-token-economy](../../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file): one call that lists the declarations nothing cites replaces one `grund refs` per ID. Reported as [grund#937](https://github.com/agent-grounds/grund/issues/937), left open by #925 on purpose because it moves the block.

## status: Accepted

## 1. Context

`grund list --unused` lists the declarations no citation points at, the set `check` warns on ([§FS-list.1.3](../../functional-spec/FS-list.md#13---unused)), and the list spec already tells a tool looking for them to pass it rather than threshold on `refs` ([§FS-list.3.2.1](../../functional-spec/FS-list.md#321-refs-counts-citations)). The block, which an agent reads before anything else, taught `grund list --kind` and the `--size` sweep but not this flag. Asked which specs nothing cites, agents ran `grund list` and then `grund refs` once per row ([grund#905](https://github.com/agent-grounds/grund/issues/905)). #925 taught the flag in `grund --help` and left the block alone, because a change to the block moves its version.

## 2. Decision

### 2.1 One bullet after the `--kind` one

The list gains `` - `grund list --unused` — the declarations nothing cites yet, in one call ``, directly after the `grund list --kind` bullet ([§FS-init.2.3.4.3](../../functional-spec/FS-init.md#2343-cheap-grounding)). It is one line with no recipe.

### 2.2 Why it clears the block's bar

[§DF-block-teaches-structural-queries](DF-block-teaches-structural-queries.md#df-block-teaches-structural-queries-the-managed-block-teaches-the-structural-query-inline-and-links-the-query-guide) keeps out of the block what is one flag away from what it already teaches, because every line is read in every session. That decision's own test is an agent falling back to a slower path because nothing it reads first said otherwise, and #905 is that evidence for `--unused`: the flag was one away from `--kind`, and agents still looped `refs`. One line read per session costs less than one `refs` call per declaration in the sessions that ask.

### 2.3 What it costs

The block's fixed text changes, so its version moves. The change shares the unreleased v39–v42 pairs rather than adding one, since no release wrote them before it ([§FS-init.2.3.7.1](../../functional-spec/FS-init.md#2371-the-current-version)).

## 3. Consequences

- An agent that reads only the block finds the uncited declarations in one call.
- `grund --help` and the block now name the same query; the user guide and the `grund-init` skill are #938's.
- The empty answer of `grund refs` does not point at the flag; that is #936's.

## 4. Alternatives considered

- **A clause on the `--kind` bullet**: as cheap to read, but it ties two queries into one line, and a test cannot assert the rung apart from discovery.
- **Leave it to `grund --help`**: agents read the block before `--help`, and #905 is what they did instead.
- **A recipe for finding and pruning dead specs**: what [§DF-block-teaches-structural-queries](DF-block-teaches-structural-queries.md#df-block-teaches-structural-queries-the-managed-block-teaches-the-structural-query-inline-and-links-the-query-guide) rejects, a recipe charged to every session for a question most never ask.

## release-note: Release note

- [§FS-init.2.3.4.3](../../functional-spec/FS-init.md#2343-cheap-grounding): **The managed block teaches `grund list --unused`.** The cheap-grounding list in the `grund init` block gains one bullet, `` - `grund list --unused` — the declarations nothing cites yet, in one call ``, directly after the `grund list --kind` one, so an agent that reads only the block lists the uncited declarations in one call instead of running `grund refs` per ID. **What an upgrader does:** run `grund init` once, which the block versions v39/v40 and v41/v42 of this release already ask for. (agent-grounds/grund#937)
