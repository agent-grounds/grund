# DF-block-names-section-heading: The block's first rung says a section read opens with its heading line

**Date:** 2026-10-26

Settles how the managed block's first grounding rung describes what `grund <ID>` prints, as [§FS-init.2.3.4.3](../../functional-spec/FS-init.md#2343-cheap-grounding) specifies, and why that moves the block's version ([§FS-init.2.3.7.1](../../functional-spec/FS-init.md#2371-the-current-version)). Serves [§GOAL-token-economy](../../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file): an agent that writes its checks from the block should not spend a second round finding out what the command prints. Reported as [grund#1026](https://github.com/agent-grounds/grund/issues/1026).

## status: Accepted

## 1. Context

A text read of a selected section prints that section's own heading line first, in every format ([§FS-show.2.2](../../functional-spec/FS-show.md#22-section)). The block called every default read "heading-less", which is true only of a whole declaration, and `show --help` said only md "keeps the heading" ([§FS-cli.2.3.4](../../functional-spec/FS-cli.md#234-show-says-a-section-read-opens-with-its-heading)). An agent that trusted either wrote a check that failed on the section's heading line.

## 2. Decision

### 2.1 Fix the words, not the output

The rung keeps its one line and names both reads: heading-less for a whole declaration, and the section's heading line first for `grund <ID>.<section>`. The output stays as [§FS-show.2.2](../../functional-spec/FS-show.md#22-section) specifies; making section reads heading-less would undo the canonical heading a text read now names.

### 2.2 What it costs

The block's fixed text changes, and 0.20.0 already wrote v39–v42, so the change cannot share those pairs: both advance four, to v43/v44 and v45/v46 ([§FS-init.2.3.7.1](../../functional-spec/FS-init.md#2371-the-current-version)).

## 3. Consequences

- An agent that reads only the block expects the heading line on a section read and none on a whole declaration.
- Every block 0.20.0 wrote is reported outdated until `grund init` re-renders it.

## 4. Alternatives considered

- **Make text section reads heading-less**: matches the old words, but amends [§FS-show.2.2](../../functional-spec/FS-show.md#22-section) and drops the heading that tells a reader which section it got.
- **Leave the block and fix only `show --help`**: agents read the block before `--help`, so the mismatch the report describes would stay.

## release-note: Release note

- [§FS-init.2.3.4.3](../../functional-spec/FS-init.md#2343-cheap-grounding): **The managed block says a section read opens with its heading line.** The `grund <ID>` rung of the `grund init` block now reads that the lead is heading-less for a whole declaration, while `grund <ID>.<section>` opens with that section's heading line, as `grund show` has always printed it. The block moves from v39/v40 and v41/v42 to **v43/v44** and **v45/v46**, so `grund check` reports a block 0.20.0 wrote as outdated. **What an upgrader does:** run `grund init` once to re-render the block; unmanaged content is kept. (agent-grounds/grund#1026)
