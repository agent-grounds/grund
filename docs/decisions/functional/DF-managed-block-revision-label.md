# DF-managed-block-revision-label: the managed block heading labels its number as the block's revision

**Date:** 2026-10-20

## status: Accepted

## 1. Context and decision

The managed block's version heading read `## Grounding with grund (v19)`, and a grund 0.18.0 refusal ended `this wording changes in grund 0.19.0`. An agent read the two numbers as one scale, took the block for instructions written for a newer grund, and diagnosed a correct refusal as a binary too old to follow them (#827). Nothing in the heading said that `v19` counts revisions of the block's text and has no relation to a grund release.

The heading now says so: `## Grounding with grund (block vN)` ([§FS-init.2.3.9.3](../../functional-spec/FS-init.md#2393-the-headings-two-spellings)). `vN` keeps its meaning and its mechanism ([§FS-init.2.3.7](../../functional-spec/FS-init.md#237-the-block-version)); only the word that classifies it is new, and a grund release is still read from `grund -V` alone.

## 2. What was weighed

- **The version in the delimiter**, `<!-- BEGIN GRUND MANAGED BLOCK v31 -->`, was rejected. It hides the revision from the rendered Markdown a reader skims, and it changes the boundary every reader of the block finds it by ([§FS-init.2.3.9.1](../../functional-spec/FS-init.md#2391-why-the-conventional-shape)): a delimiter-aware older grund would no longer see a block at all and could append a second one.
- **Requalifying the refusal's wording-window suffix** was dropped. The one-grammar window closes in the next release, which removes the suffix, so the qualifier would never ship; once the heading reads `block vN` the two numbers no longer look alike.
- **Reading both spellings inside the delimiters** is what keeps every existing block readable: a v30 block is reported outdated, not malformed, and one `grund init` relabels it in place. The new spelling opens no legacy block, because no block without delimiters was ever written with it ([§FS-init.2.3.9.2](../../functional-spec/FS-init.md#2392-legacy-blocks)).
- **The spelling moves the version.** It is fixed text, so the change is a bump of both current pairs ([§FS-init.2.3.7.1](../../functional-spec/FS-init.md#2371-the-current-version)) rather than a silent rewrite.

## release-note: Release note

- The managed block's heading now reads `## Grounding with grund (block vN)`, so its number reads as the revision of the block's text rather than a grund release; `grund -V` is where the release is read. Both current block versions move: v27/v28 becomes v31/v32 and v29/v30 becomes v33/v34. v23 to v30 never shipped, so a 0.18 repository goes from v19–v22 straight to v31–v34 with the one `grund init` it already owes: `grund check` and `grund init --check` report the earlier block outdated, and `grund init` re-renders it in place, keeping unmanaged content. This breaks two readers. Anything that matches `## Grounding with grund (vN)` must also accept `(block vN)`. An older grund reports the new block as malformed, so upgrade grund before running `grund init`.
