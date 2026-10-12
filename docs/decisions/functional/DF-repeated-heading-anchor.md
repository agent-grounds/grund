# DF-repeated-heading-anchor: a repeated heading's anchor is numbered as the renderer numbers it

**Status:** Accepted
**Date:** 2026-10-12

## context: Context

A renderer gives two headings with the same text in one file two anchors: `github-slugger` keeps a count of each slug it has handed out in the document and suffixes a repeat `-1`, then `-2`. grund 0.17 does not count. Where `docs/architecture/AR-login.md` holds `# AR-login: Login`, `## 1. Overview`, `# AR-logout: Logout` and `## 1. Overview`, `grund fmt --cross-refs` links `<§>AR-logout.1` to `AR-login.md#1-overview`, which GitHub renders on AR-login's heading, and `show` reports the same anchor for both sections. The link resolves, so `lychee --include-fragments` passes it, and lands on the wrong section. That is a wrong answer from the profile that claims byte-for-byte fidelity ([§FS-fmt.6.2.3](../../functional-spec/FS-fmt.md#623-the-github-profile-reproduces-github-slugger), [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution)).

Relative section headings make it common: `### .1: Overview` under every parent of a declaration renders as the same text each time ([§DF-relative-section-headings](DF-relative-section-headings.md#df-relative-section-headings-a-heading-may-write-only-the-component-it-adds)).

## decision: Decision

### decision.count: Every heading of the file is counted, in document order

The anchor of a heading is its slug ([§FS-fmt.6.2.2](../../functional-spec/FS-fmt.md#622-the-anchor-slugifies-the-headings-rendered-text)) numbered against every heading the renderer renders earlier in the same file: declarations, sections and plain headings alike, and none inside a fenced code block. Under `github`, `gitlab` and `pandoc` a slug already handed out takes `-1`, then `-2`, and so on, and a numbered slug that is itself taken moves on to the next number, as `github-slugger` does. Under `mkdocs` the suffix is `_1`, `_2`, as Python-Markdown's TOC extension writes it. `none` has no anchor to number.

So in the example the second `## 1. Overview` anchors at `#1-overview-1`, for the link `fmt` writes and for the `anchor` `show` reports ([§REQ-one-answer.writes.anchors](../../requirements/REQ-one-answer.md#writesanchors-shows-anchor-is-the-anchor-fmt-writes)), and the first keeps `#1-overview`.

### decision.row: The changed link names the versions

A link an older grund wrote to the unnumbered anchor is now stale, and `fmt --check` fails on it. That is a verdict flip, taken on the route of a loud, mechanical migration ([§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)). Where the only change on a line is a link whose old anchor is the unnumbered slug of the heading it now numbers, its dry-run row ([§FS-fmt.3.5](../../functional-spec/FS-fmt.md#35-the-dry-run-report)) reads `markdown link: repeated heading anchor #1-overview → #1-overview-1, numbered since grund 0.18`, rather than the bare `markdown link`. The fix is the one command `grund fmt --cross-refs --write`, and the release note names both.

## rejected: Rejected alternatives

- **Number only declarations and sections.** The renderer counts every heading, so a plain heading between two sections would put grund's count behind GitHub's.
- **Number per declaration body.** The renderer's count is per file, not per declaration.
- **Leave it, and tell authors to vary titles.** A relative heading's title is the author's to choose, and the anchor is grund's to get right.

## release-note: Release note

- [§FS-fmt.6.2.3](../../functional-spec/FS-fmt.md#623-the-github-profile-reproduces-github-slugger): **a repeated heading's anchor is numbered.** Two headings with the same text in one file now anchor as GitHub renders them, `#1-overview` and then `#1-overview-1` (`_1` under `mkdocs`), in the links `fmt --cross-refs` writes and in the `anchor` `show` reports; before, every repeat linked to the first. **Who this breaks:** a repository whose committed links cite the second or a later of two identically titled headings in one file: `grund fmt --check` now fails on each, with a row naming the anchor it moves to and that it is numbered since grund 0.18. Run `grund fmt --cross-refs --write` and commit the result.
