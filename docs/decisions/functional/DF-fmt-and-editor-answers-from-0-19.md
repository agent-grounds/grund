# DF-fmt-and-editor-answers-from-0-19: `fmt`'s link wrapper and the editor's title span change in 0.19.0, after a release that announces them

**Date:** 2026-10-12

## status: Accepted

## 1. Context

[§DF-one-answer-rulings-fmt-editor](DF-one-answer-rulings-fmt-editor.md#df-one-answer-rulings-fmt-editor-fmt-writes-only-into-a-complete-link-wrapper-never-into-a-link-destination) sends the wrapper part of D17g to the warning release of
[§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): which `[§<ID>](…` counts as a wrap, and where a wrap ends, is a
policy no requirement fixes. Row P16 of [§REQ-one-answer.differences.sections](../../requirements/REQ-one-answer.md#differencessections-sections) is ruling D17d on
one more surface, and [§DF-one-answer-rulings-parse](DF-one-answer-rulings-parse.md#df-one-answer-rulings-parse-one-parse-gives-every-command-the-same-extents-comment-blocks-anchors-and-title-citations) sends D17d to the same route. The author's
answer on [#576](https://github.com/agent-grounds/grund/issues/576) sends every ruling that only
makes commands agree through a warning release first.

The one part of D17g an existing requirement forbade, writing into a destination `fmt` does not
own, is corrected in the release that ships this record, under
[§DF-fmt-never-writes-into-a-link-destination](DF-fmt-never-writes-into-a-link-destination.md#df-fmt-never-writes-into-a-link-destination-fmt-never-writes-into-a-link-destination-it-does-not-maintain). This record holds the rest, and names the release
it moves in.

## 2. Decision

### 2.1 What moves in 0.19.0

Each answer moves to the one the specification now names for it, in the release after the
one that announces it:

| Where | Until 0.19.0 | From 0.19.0 |
|---|---|---|
| `fmt --cross-refs`, a `[§<ID>](` that never closes before a later `)` | left as written | wrapped fresh, nothing deleted |
| `fmt --cross-refs`, a wrap with balanced parentheses in its URL | re-derived up to the first `)`, the rest kept | its whole destination re-derived |
| `show`'s flatten ([§FS-show.3.2.1](../../functional-spec/FS-show.md#321-the-wrap-shape-it-collapses)) | collapses up to the first `)` | collapses a closed wrap whole, and prints an unclosed one as written |
| `check`'s kind-index entry form ([§FS-check.3.17.4](../../functional-spec/FS-check.md#3174-only-a-citation-fmt-would-wrap-reaches-this-rule)) | reads a wrap up to the first `)` | reads a closed wrap |
| the editor's span for a declaration or section title ([§FS-lsp.1.3.4](../../functional-spec/FS-lsp.md#134-origin-span-and-result-shape)) | runs to the end of the line, a C-family `*/` included | ends where the `--toc` title ends |

The wrapper rows are [§FS-citations.wrap.recognized](../../functional-spec/FS-citations.md#wraprecognized-an-existing-wrap-is-recognized-by-its-brackets). The editor row is P16: the span is taken
from the heading the scan records, whose text leaves out a closing `*/` and a Markdown heading's
closing hashes, so the two answers cannot drift apart again. P16 stays in
[§REQ-one-answer.differences.sections](../../requirements/REQ-one-answer.md#differencessections-sections), with this record as its ruling, until the release that
moves it strikes it.

### 2.2 What 0.18.0 warns about

0.18.0 warns only where a verdict moves. The one place that is so is the first row: `fmt --check`
passes on such a line in 0.18.0 and fails on it again in 0.19.0 unless it was fixed, so `check`
warns there as `unclosed-wrapper` ([§FS-citations.checks.unclosed-wrapper](../../functional-spec/FS-citations.md#checksunclosed-wrapper-unclosed-wrapper-read-differently-from-grund-0190)), naming the `)` that
keeps today's answer. Every other row changes output only, with nothing for a repository to
migrate, so the release note announces it and no finding does.

### 2.3 The follow-up

[Issue #655](https://github.com/agent-grounds/grund/issues/655) makes these moves. It is blocked
by #596, whose record holds the heading the editor's span will be read from, and by #598, which
ships this record, and it waits for the release #598 lands in.

## 3. Rejected alternatives

- **Every row at once, in the release that ships this record.** No requirement forbids today's
  answer on these rows, and the author's answer on #576 sends a change that only makes commands
  agree through a warning release first.
- **A zone field recorded on each citation, for `fmt` to read.** `fmt`'s link pass runs on a line
  earlier passes of the same run have already rewritten, `show` flattens a body string rather
  than a file, and `check`'s index form reads the entry line on purpose. None could read the
  field, and it would be a second copy beside the scanner's own function, which
  [§AR-system.principle](../../architecture/README.md#principle-one-fact-one-point-one-implementation) forbids.
- **Warn at every wrap that 0.19.0 reads differently.** A wrap with balanced parentheses moves
  only the bytes `fmt --write` writes, never a verdict: `fmt --check` fails on it in neither
  release once it has been written.

## 4. The 0.19.0 half landed

Every row of [§DF-fmt-and-editor-answers-from-0-19.2.1](DF-fmt-and-editor-answers-from-0-19.md#21-what-moves-in-0190) moves in grund 0.19.0, on the schedule it
named (agent-grounds/grund#655). A wrap is `[§<ID>](…)` closed by the `)` that balances its `(`,
and `fmt --cross-refs`, `show`'s flatten and `check`'s kind-index entry form read it through the
one wrapper test ([§FS-citations.wrap.recognized](../../functional-spec/FS-citations.md#wraprecognized-an-existing-wrap-is-recognized-by-its-brackets), [§FS-show.3.2.1](../../functional-spec/FS-show.md#321-the-wrap-shape-it-collapses), [§FS-check.3.17.4](../../functional-spec/FS-check.md#3174-only-a-citation-fmt-would-wrap-reaches-this-rule)).
The `unclosed-wrapper` warning of 2.2 is retired with its code and catalog row, so a selector naming
it is refused as an unknown code ([§FS-citations.checks.unclosed-wrapper](../../functional-spec/FS-citations.md#checksunclosed-wrapper-unclosed-wrapper-read-differently-from-grund-0190)). The editor's span for a
declaration or section title is the `--toc` title's ([§FS-lsp.1.3.4](../../functional-spec/FS-lsp.md#134-origin-span-and-result-shape)), and row P16 leaves
[§REQ-one-answer.differences.sections](../../requirements/REQ-one-answer.md#differencessections-sections). The notice below is not rewritten: it shipped in 0.18.0 naming
this release, and a notice already released is not published again
([§FS-distribution.4.6.4](../../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions)).

## release-note: Release note

- [§DF-fmt-and-editor-answers-from-0-19](DF-fmt-and-editor-answers-from-0-19.md#df-fmt-and-editor-answers-from-0-19-fmts-link-wrapper-and-the-editors-title-span-change-in-0190-after-a-release-that-announces-them), [§FS-citations.wrap.recognized](../../functional-spec/FS-citations.md#wraprecognized-an-existing-wrap-is-recognized-by-its-brackets), [§FS-citations.checks.unclosed-wrapper](../../functional-spec/FS-citations.md#checksunclosed-wrapper-unclosed-wrapper-read-differently-from-grund-0190), [§FS-lsp.1.3.4](../../functional-spec/FS-lsp.md#134-origin-span-and-result-shape), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **from 0.19.0, `fmt` reads a link around a citation as complete, and the editor's title span matches the `--toc` title.** A wrap is `[§<ID>](…)` closed by the `)` that balances its `(`. `fmt --cross-refs` re-derives its whole URL, so `[§<ID>](old(1).md)` no longer keeps `.md)`, and a `[§<ID>](` that never closes is wrapped fresh. `show` and the kind-index check read wraps the same way. The editor's span for a declaration or section title ends where the `--toc` title ends, so a C-family `*/` is no longer part of it. **Who this affects:** wherever a verdict moves, 0.18.0 warns with `unclosed-wrapper`, naming the `)` that keeps today's reading; everything else changes output only, with nothing to migrate. The change is [issue #655](https://github.com/agent-grounds/grund/issues/655).
