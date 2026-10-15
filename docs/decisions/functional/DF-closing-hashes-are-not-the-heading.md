# DF-closing-hashes-are-not-the-heading: a heading's closing hashes are not part of its anchor

**Date:** 2026-10-12

## status: Accepted

## 1. Context

CommonMark lets an ATX heading close with a run of `#`: `## 1. Steps ##` renders as the
heading "1. Steps", and GitHub gives it the anchor `1-steps`. grund read the closing run as
part of the heading's text. Reproduced on `main` at 89aa15b51a, on this tree:

```text
docs/fs.md    # FS-a: Alpha

              Alpha body.

              ## 1. Steps ##

              Step text.
docs/use.md   See §FS-a.1.
```

`grund fmt --cross-refs --write` turns `docs/use.md` into `See [<§>FS-a.1](fs.md#1-steps-).`,
and `grund FS-a --toc --format json` reports the section's `anchor` as `1-steps-`. The file
has no heading whose anchor is `1-steps-`, so the link `fmt` persisted lands at the top of
the page, and a caller that composes a link from `show`'s `anchor` lands there too. The
generic slugger turned the closing run into a trailing `-` that `github-slugger` never
produces for this heading, because the run is not in the heading's rendered text at all.

[§DF-one-answer-rulings-parse](DF-one-answer-rulings-parse.md#df-one-answer-rulings-parse-one-parse-gives-every-command-the-same-extents-comment-blocks-anchors-and-title-citations) ruled the answer (D17e, V-A18): an anchor comes from the
complete rendered text of the authored heading, under its renderer profile, with no generic
trailing-hyphen trim. It named the anchor's fix a candidate for the correction route and the
TOC `title` a separate output that takes the warning release.

## 2. Decision

A heading's closing sequence of `#`, and the whitespace before it, is not part of its
rendered text ([§FS-declarations.heading.rendered-text](../../functional-spec/FS-declarations.md#headingrendered-text-a-headings-rendered-text)). Every anchor is slugged from the
rendered text by [§FS-fmt.6.2.3](../../functional-spec/FS-fmt.md#623-the-github-profile-reproduces-github-slugger), so `fmt --cross-refs` writes `fs.md#1-steps`, `show`'s
JSON `anchor` is `1-steps` ([§FS-show.3.1.3.1](../../functional-spec/FS-show.md#3131-the-heading-anchor)), and the two stay one answer
([§REQ-one-answer.writes.anchors](../../requirements/REQ-one-answer.md#writesanchors-shows-anchor-is-the-anchor-fmt-writes)). A heading with no closing run slugs exactly as before:
`## A — B` is still `a--b`, because `github-slugger` trims no trailing `-` of its own.

The TOC `title` is displayed text, not a write, and it keeps the closing run, `Steps ##`,
until 0.19.0 ([§DF-one-parse-answers-from-0-19](DF-one-parse-answers-from-0-19.md#df-one-parse-answers-from-0-19-one-parses-other-answers-ship-in-0190-after-a-release-that-announces-them-and-warns-where-a-verdict-moves)).

## 3. Rejected alternative: keep the old anchor for one release

`fmt` could keep writing `#1-steps-` for a release, with a warning, and flip in the next.
That would keep a writer persisting a link to nowhere into repositories for another release,
one `fmt --check` would then have to unwrite. The anchor is not a form a user chose; it is a
wrong write, and the correction route below says no release may keep one.

## 4. Consequences

- **`fmt --check` reports a link once.** A repository that already holds a link ending in
  `#1-steps-` sees `fmt --check` report that line, and `grund fmt --cross-refs --write`
  rewrites it to `#1-steps`. No other link moves.
- **`show`'s JSON `anchor` moves with it**, in `--toc` `sections` entries and on a selected
  section, for a heading with closing hashes only.
- **Nothing else moves.** No `check` finding is added or removed, the TOC `title` and every
  printed body keep their bytes, and a heading without a closing run keeps its anchor.

## 5. The correction route

[§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) licenses the move in one release, and each of its five conditions holds.

1. *Prior prohibition.* The old verdict violated [§REQ-no-wrong-citation.3](../../requirements/REQ-no-wrong-citation.md#3-no-wrong-write), read through [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution), which already applied when that verdict shipped: [§REQ-no-wrong-citation.3](../../requirements/REQ-no-wrong-citation.md#3-no-wrong-write) arrived in 78482203dc, first released in 0.13.0, and it is present in 0.17.0, whose `fmt` writes `#1-steps-` for `## 1. Steps ##`. A persisted rewrite is a resolution, and a link to a fragment the file does not carry resolves the citation to no heading, which is the wrong write that section forbids, shown here on the closing-hash case itself (V-A25).
2. *Accepted proof.* This record. The [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path does not fit, because there is no old form a user chose to keep beside a new one, only a link to nowhere, and keeping it a release longer would mean writing more of them. The [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) mechanical migration has no separate command to point at, because the migration is the writer itself: `grund fmt --cross-refs --write` rewrites every stale link on its next run.
3. *Named release.* The release note below names the move and who sees it.
4. *Actionable findings.* The one finding that replaces the old verdict is `fmt --check`'s report of the stale link, at its file and line, whose fix is `grund fmt --cross-refs --write`.
5. *No new licence.* This is not a licence for anything else. No rule or finding is new, and the TOC `title`, which also carries the closing run today, takes the deprecation path ([§DF-one-parse-answers-from-0-19](DF-one-parse-answers-from-0-19.md#df-one-parse-answers-from-0-19-one-parses-other-answers-ship-in-0190-after-a-release-that-announces-them-and-warns-where-a-verdict-moves)).

## release-note: Release note

- [§DF-closing-hashes-are-not-the-heading](DF-closing-hashes-are-not-the-heading.md#df-closing-hashes-are-not-the-heading-a-headings-closing-hashes-are-not-part-of-its-anchor), [§FS-declarations.heading.rendered-text](../../functional-spec/FS-declarations.md#headingrendered-text-a-headings-rendered-text), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), [§REQ-no-wrong-citation.3](../../requirements/REQ-no-wrong-citation.md#3-no-wrong-write): **a heading's closing hashes are not part of its anchor.** `## 1. Steps ##` renders as "1. Steps", whose anchor is `1-steps`, but `grund fmt --cross-refs` wrote `#1-steps-`, a fragment no heading carries, and `show`'s JSON `anchor` said the same. Both now give `1-steps`. **Who this breaks:** a repository holding a cross-reference link to a heading with closing hashes. Its verdict moves once: `fmt --check` reports each such link, and every finding names its file and location and the fix, `grund fmt --cross-refs --write`. Headings without closing hashes keep their anchors, and the `--toc` title keeps its bytes until 0.19.0.
