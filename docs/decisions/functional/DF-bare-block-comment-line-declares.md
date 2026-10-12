# DF-bare-block-comment-line-declares: a line inside a block comment declares with or without its gutter, behind any of its openers

**Status:** Accepted
**Date:** 2026-10-12

## 1. Context

A block comment may be written without the ` * ` gutter. Reproduced on `main` at
8b9d62e610 ([#640](https://github.com/agent-grounds/grund/issues/640)), with `FS` at the
folder `docs/fs`:

```text
src/lib.rs   /**
             FS-b: Beta

             Beta body.
             */
             pub struct B;
docs/use.md  See §FS-b.
```

`grund check` reports `docs/use.md:1: error: unknown reference FS-b`, `grund FS-b` answers
`ID not found: FS-b`, `grund list` lists nothing, and the editor underlines the citation as
dangling. The same tree indented as a class member answers alike, and so do two forms the
default prefixes miss for another reason: `/** FS-b: Beta` with the ID on the opener line,
and the one-line Javadoc `/** FS-b: Beta */`. The configured prefix `/*` must be followed
by whitespace, and the second `*` of `/**` is not.

The specification disagreed with itself. [§FS-declarations.line](../../functional-spec/FS-declarations.md#line-what-a-line-in-declaration-position-declares) put declaration position at
a Markdown heading, a comment-prefixed line and a bare line in a Python docstring, and
[§FS-show.2.3](../../functional-spec/FS-show.md#23-source-declarations-in-code-and-doc-comments) repeated it, so a bare block line declared nothing. But [§AR-scanner.4.4](../../architecture/AR-scanner.md#44-comment-lines-are-normalized-before-detection) read
the ` * ` of a continuation line as optional and named `/**` as an opener,
[§FS-languages.comment-wrapper](../../functional-spec/FS-languages.md#comment-wrapper-the-comment-wrapper) kept a bare block line as comment text, and the scanner's
own `comment_blocks` already read the whole `/* … */` span as one block for the body.

## 2. Decision

A line inside a `/* … */` block comment is in declaration position whether or not it
carries ` * `, and `/*`, `/**` and `/*!` all open a block
([§FS-declarations.line.block-comment](../../functional-spec/FS-declarations.md#lineblock-comment-a-line-inside-a-block-comment)). On the tree above `check` passes, `grund FS-b`
prints `Beta body.`, `list` gives `FS-b  src/lib.rs:2  Beta`, `refs FS-b` names the
declaration, and the editor goes to it. Every command reads it from the one catalog the
scanner fills, so they move together.

### 2.1 A bare line declares only in a block that closes

A block runs from its opener to the first `*/`. A `/*` with no `*/` after it is read to the
end of the file for the body of a declaration found some other way, and that stays. But a
bare `ID:` line after such a stray line-initial `/*` (a glob in a script, a string a
scanned file carries) would become a declaration, and a near-miss candidate, under this
change. So a bare line is in declaration position only in a block whose `*/` is found. The
opener and starred forms keep today's reading in an unclosed block. Every gate reads this
from the same block spans, closed or not, so there is still one answer.

### 2.2 The body of a declaration this change finds

Its body, sections and lead are read by the file record's own rules
([§FS-languages.comment-block](../../functional-spec/FS-languages.md#comment-block-the-comment-block), [§FS-languages.comment-wrapper](../../functional-spec/FS-languages.md#comment-wrapper-the-comment-wrapper)): opener to closer, bare
lines kept as written, so a bare `## 1. Sub` under it is its section `.1` at once.
[§DF-one-parse-answers-from-0-19](DF-one-parse-answers-from-0-19.md#df-one-parse-answers-from-0-19-one-parses-other-answers-ship-in-0190-after-a-release-that-announces-them-and-warns-where-a-verdict-moves) keeps 0.17's bytes for one more release only where 0.17
printed some, and 0.17 printed nothing for these declarations, so there is nothing to keep
and no `parse-moves` warning on them. A declaration found today is not touched: a
` * FS-b` or `/* FS-b` declaration whose body has bare lines keeps today's reading until
0.19.0.

### 2.3 The opener moves the section-heading regex too

The configured `/*` is matched as `/\*[*!]?`, the way `//` is matched as `//[/!]?`. The
same prefix alternation feeds the section-heading regex, so `/** ## 1. Sub` on an opener
line becomes a section heading, exactly as `/* ## 1. Sub` is one today
([§FS-show.2.3.3](../../functional-spec/FS-show.md#233-section-selection-inside-a-doc-comment)). That is the same answer the declaration gate gives on what an opener is.
Where such a line sits inside a declaration's block, it is now that declaration's section,
and a citation of it resolves; where it sits outside any declaration, it is now the
`numbered section outside any declaration` error the `/*` form already draws.

## 3. Rejected alternatives

- **A warning release first**, the route row P02 of [§DF-one-parse-answers-from-0-19](DF-one-parse-answers-from-0-19.md#df-one-parse-answers-from-0-19-one-parses-other-answers-ship-in-0190-after-a-release-that-announces-them-and-warns-where-a-verdict-moves) took
  for a bare-line section. That would keep a false `unknown reference` for one more
  release, which [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms) forbids, and nothing in a user's repository needs
  to change: the source is right as written.
- **Declare only in `/**` doc blocks.** That would make a bare line narrower than the
  starred line beside it, which declares in any block comment today, and it would pre-empt
  the declaration-host gate planned in
  [#456](https://github.com/agent-grounds/grund/issues/456), which will apply to both forms
  alike.
- **A separate block-comment reader in the declaration gate.** That would be a second copy
  of what `comment_blocks` already answers, and two answers to one question.

## 4. Consequences

The verdict moves both ways, and only on trees with a line inside a `/* … */` block that
starts, without ` * `, with an ID-shaped token of a configured kind, or that declares
behind a `/**` or `/*!` opener.

- **Fail to pass.** A citation of such a declaration is no longer an `unknown reference`;
  it resolves, `show` prints its body, `list`, `refs` and `cover` count it, a `must`
  direction it satisfies is satisfied, and a file that declares only there now satisfies
  the "every source file must cite or declare" rule.
- **Pass to fail.** A newly found declaration is judged like any other: a duplicate of an
  ID declared elsewhere is reported at both sites, a declaration outside its kind's home
  is `misplaced-declaration`, a missing index entry is reported (and `grund fmt --write`
  adds it), and the chapter and direction rules apply. An uncited one draws the ordinary
  `declared but never cited` warning. The likeliest case is a repository that worked
  around this bug by declaring the ID a second time in `docs/`, which now gets a duplicate
  pointing at both copies.
- **The near miss.** A bare block line such as `FS-Bad: x` becomes a
  `declaration-near-miss`, the same error, with the same message, the starred
  ` * FS-Bad: x` gets today. This rides on the argument of condition 5 below: the rule
  exists already, and only the position it reads is corrected. That argument holds only if
  the before-and-after measurement over the repositories this toolchain gates finds every
  new near miss on an attempted declaration. Should it find one on prose, the near miss on
  a bare block line takes the [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) route instead: a warning in
  this release, and the error from the next one through a follow-up under
  [#576](https://github.com/agent-grounds/grund/issues/576). That measurement belongs to the
  pull request that ships this record, and its outcome is written into this section and
  the release note before the record ships.
- **Sections.** `/** ## 1. Sub` on an opener line is a section heading (2.3).
- **Untouched.** A repository that removed `/*` from `[scan] comment_prefixes` sees no
  change, because the block state follows that prefix as `comment_blocks` does. Python
  docstrings, `//` blocks and Markdown read as before; `fmt` writes nothing new; no flag,
  configuration key or output format changes.

## 5. The correction route

[§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) licenses the move in one release, and each of its five
conditions holds.

1. *Prior prohibition.* The old verdict violated [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms), which already applied when that verdict shipped: a citation of a declaration its author wrote in a block comment was reported as an `unknown reference`. The verdict has stood since grund 0.1.0, whose default `comment_prefixes` already carried `/*` and whose declaration regex already needed a prefix on the line itself, and it is present in 0.17.0.
2. *Accepted proof.* This record. The [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path does not fit the declarations, because there is no old form a user chose, only declarations the user wrote that the tool did not read, and keeping them unread a release longer keeps a false `unknown reference` on every citation of them. The [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) mechanical migration has no command to point at, because nothing in the repository needs rewriting: the comments are right as written.
3. *Named release.* The release note below names the verdict move in both directions, and says who sees it.
4. *Actionable findings.* Every finding that replaces the old verdict is one a starred ` * ` declaration already gets, at the declaring line and naming the ID and the edit: rename, remove, put the token behind prose, or add the index entry.
5. *No new licence.* This is not a licence for anything else. No rule, finding or prohibition is new: every finding the change adds is one the starred form already gets, read at a position [§FS-declarations.line](../../functional-spec/FS-declarations.md#line-what-a-line-in-declaration-position-declares) now names. The near miss is held to this condition by the measurement in section 4.

## release-note: Release note

- [§DF-bare-block-comment-line-declares](DF-bare-block-comment-line-declares.md#df-bare-block-comment-line-declares-a-line-inside-a-block-comment-declares-with-or-without-its-gutter-behind-any-of-its-openers), [§FS-declarations.line.block-comment](../../functional-spec/FS-declarations.md#lineblock-comment-a-line-inside-a-block-comment),
  [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms): **a declaration in a block
  comment is found with or without the ` * ` gutter, and behind a `/**` or `/*!`
  opener.** A bare `FS-b: Beta` line inside `/** … */`, `/** FS-b: Beta` on the opener
  line and the one-line `/** FS-b: Beta */` now declare `FS-b`, so the false
  `unknown reference` on every citation of them goes, and `show`, `list`, `refs`, `cover`
  and the editor find them. A bare line declares only in a block whose `*/` is found.
  `/** ## 1. Sub` on an opener line is now a section heading, as `/* ## 1. Sub` is.
  The verdict moves both ways. **Who this breaks:** a repository with a line inside a block comment that starts, without
  ` * `, with an ID-shaped token of a configured kind. A declaration found there is judged
  like any other: a duplicate of an ID declared elsewhere, a home, index, chapter or
  direction finding, or an unused-declaration warning, and every finding names its
  location and the fix. A bare block line that a configured kind's grammar rejects, followed by a colon, is
  a `declaration-near-miss` error, as its starred twin is.
