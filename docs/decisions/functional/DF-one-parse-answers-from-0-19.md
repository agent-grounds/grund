# DF-one-parse-answers-from-0-19: one parse's other answers ship in 0.19.0, after a release that announces them and warns where a verdict moves

**Status:** Accepted
**Date:** 2026-10-12

## 1. Context

From 0.18.0 the scan reads each file once into one record, kept in the `Catalog`
([§AR-scanner.2.8](../../architecture/AR-scanner.md#28-the-file-record)), and every command answers where a body, a section, a lead and a comment
block end from it. On most trees that record gives the answer every command gives today.
On the trees of rows P01, P02, P03, P07, P10 and P12 of [§REQ-one-answer.differences.sections](../../requirements/REQ-one-answer.md#differencessections-sections),
and on the rulings D17a to D17d of [§DF-one-answer-rulings-parse](DF-one-answer-rulings-parse.md#df-one-answer-rulings-parse-one-parse-gives-every-command-the-same-extents-comment-blocks-anchors-and-title-citations), it gives the answer one
command gives and another does not, or an answer no command gives yet.

That ruling routes all of them through a warning release: no requirement forbade today's
extents (V-A17), and a command agreeing with another is not on its own a reason to flip a
verdict (V-A25). The TOC `title` of a heading with closing hashes takes the same route
(V-A18), while its anchor is corrected at once ([§DF-closing-hashes-are-not-the-heading](DF-closing-hashes-are-not-the-heading.md#df-closing-hashes-are-not-the-heading-a-headings-closing-hashes-are-not-part-of-its-anchor)), and
so is a citation in a declaration's title ([§DF-a-title-citation-is-a-citation](DF-a-title-citation-is-a-citation.md#df-a-title-citation-is-a-citation-a-citation-on-a-declarations-heading-is-a-citation-unless-it-cites-the-id-that-heading-declares)).

## 2. Decision

In 0.18.0 every answer below is today's, byte for byte, given from the same record by a
compatibility reading that names this record. In 0.19.0 each is the record's, and the
compatibility reading goes ([#639](https://github.com/agent-grounds/grund/issues/639)).

| Row | What moves in 0.19.0 | Who prints it |
|---|---|---|
| P01 | a Markdown body ends at the next same-or-higher heading, declaring or plain ([§FS-declarations.body](../../functional-spec/FS-declarations.md#body-where-a-declarations-body-ends)) | `show --full`, `list --size` |
| P02 | a `/** … */` line without ` * ` that reads `## 1. Sub` is section `.1`, printed as written | the scanner, a direct query, `refs`, `show` |
| P03 | `/// # 1. Sub` is not a section of a `///` declaration, so `--toc` stops listing it ([§FS-show.2.3.3](../../functional-spec/FS-show.md#233-section-selection-inside-a-doc-comment)) | `--toc`, every lead it cut |
| P07 | a section ends at the next heading of its level or above, numbered or plain (D17b) | `show` |
| P10 | a `//` note next to a `///` block is a block of its own, outside the body (D17c) | `show`, `list --size` |
| P12 | a doc-comment section ends at a duplicate, plain or shallower heading for ownership too (D17b) | `refs`, `cover --lines`, the chapter rules, the editor |
| D17a | a deeper declaration does not end its parent's body | `show --full`, `list --size` |
| D17d | a trailing `*/` is envelope in a C-family comment only, and stays in a Markdown or docstring label | `show` |
| title | a TOC `title` drops a heading's closing hashes: `Steps ##` becomes `Steps` | `--toc` in every format |

D17b also covers a plain heading inside a doc-comment section: today `show` and `refs` both
run past it, and from 0.19.0 both end the section there.

## 3. The sides P02 and P03 take

D17 rules neither row, so this record does.

- **P02 takes the scanner across.** [§FS-languages.comment-wrapper.headings](../../functional-spec/FS-languages.md#comment-wrapperheadings-the-wrapper-of-a-heading-line) keeps a
  block-comment line without ` * ` as written, so `## 1. Sub` is a heading and `--toc` is
  right. Making `--toc` drop it would rewrite the specification to fit the scanner. The new
  section can own citations and enter the chapter rules, so a verdict moves, and the line
  warns in 0.18.0. The edit it names, ` * ## 1. Sub`, means the same in both releases.
  Today's verdict is a false `missing section`, so the correction route was open to it; it
  was not taken, because correcting it now would also move ownership and the chapter rules
  in the same release, which a warning release lets an author see first.
- **P03 takes the query's side.** [§FS-show.2.3.3](../../functional-spec/FS-show.md#233-section-selection-inside-a-doc-comment) makes `## 1.` the depth-1 section in a
  doc-comment, so `/// # 1. Sub` is a same-level heading, not a section. Recording it as one
  would give doc-comments a section depth Markdown does not have.

## 4. A warning where a verdict moves, a notice everywhere else

Where 0.19.0 moves a `check` verdict, 0.18.0 warns at the line that moves it
([§FS-declarations.checks.parse-moves](../../functional-spec/FS-declarations.md#checksparse-moves-a-verdict-that-moves-in-0190)): a heading that will end a doc-comment section above a
citation (P12, and D17b's plain heading), a block-comment line without ` * ` that will become
a section (P02), and a heading that will stop cutting a lead where the lead then passes a
configured budget (P03). Every other answer above changes printed bytes or sizes and no
verdict, so it is announced by the release note below and warns nowhere: no saved
repository has anything to migrate. P07's tree already fails `check` with
`unmarked-heading`, so no passing Markdown repository holds it.

A fourth shape rides the same code: a relative heading in a doc-comment declaration body,
prose today and a section from 0.19.0
([§DF-doc-comment-relative-headings](DF-doc-comment-relative-headings.md#df-doc-comment-relative-headings-a-relative-heading-in-a-doc-comment-warns-in-0180-and-is-a-section-from-0190)).

## 5. Rejected alternatives

- **A caution on every `show` and `refs` whose answer will change.** It would add a
  comparison to every query for a change of bytes, and say nothing a release note does not.
- **Flip the rows now.** Without a requirement the old answers broke, a verdict moves only
  after a warning release, as the ticket's author answered on #576; <§>REQ-one-answer is
  never the requirement a flip rests on.
- **Keep two parsers in step for a release.** Teaching the old slicer the record's rules
  keeps two implementations of one fact ([§AR-system.principle](../../architecture/README.md#principle-one-fact-one-point-one-implementation)). Today's answers are instead
  read from the one record by a reading that exists for one release.

## release-note: Release note

- [§DF-one-parse-answers-from-0-19](DF-one-parse-answers-from-0-19.md#df-one-parse-answers-from-0-19-one-parses-other-answers-ship-in-0190-after-a-release-that-announces-them-and-warns-where-a-verdict-moves), [§AR-scanner.2.8](../../architecture/AR-scanner.md#28-the-file-record), [§FS-declarations.checks.parse-moves](../../functional-spec/FS-declarations.md#checksparse-moves-a-verdict-that-moves-in-0190), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **from 0.19.0 every command reads one parse of each file, and some printed bodies, sizes and section owners change.** grund 0.18.0 already reads each file once into one record and answers from it, keeping today's output byte for byte. From 0.19.0: `show --full` and `list --size` end a Markdown body at the next same-or-higher heading, plain or declaring, and leave out a `//` note next to a `///` block and a deeper declaration's end; `show` ends a section at a plain heading of its level or above; `--toc` stops listing `/// # 1. Sub` and drops a heading's closing hashes from its `title`; a `/** … */` line without ` * ` that reads `## 1. Sub` becomes section `.1`; and `refs`, `cover --lines` and the chapter rules end a doc-comment section where `show` does. **Who this affects:** where one of these moves a `check` verdict, 0.18.0 warns at the line with `parse-moves`, naming the edit that keeps today's meaning in both releases; everything else changes output only, with nothing to migrate. Tracked in [issue #639](https://github.com/agent-grounds/grund/issues/639).
