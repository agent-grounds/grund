# DF-one-parse-lands-in-0-19: every command answers from one parse in 0.19.0, and the managed block teaches doc-comment sections

**Date:** 2026-10-15

## status: Accepted

## 1. Context

[§DF-one-parse-answers-from-0-19](DF-one-parse-answers-from-0-19.md#df-one-parse-answers-from-0-19-one-parses-other-answers-ship-in-0190-after-a-release-that-announces-them-and-warns-where-a-verdict-moves) kept 0.17's answers for one release through a compatibility
reading over the one file record, announced in 0.18.0 every answer that reading held back, and
warned `parse-moves` where one moved a `check` verdict. [§DF-doc-comment-relative-headings](DF-doc-comment-relative-headings.md#df-doc-comment-relative-headings-a-relative-heading-in-a-doc-comment-warns-in-0180-and-is-a-section-from-0190) and
[§DF-number-name-sections.decision.doc-comments](DF-number-name-sections.md#decisiondoc-comments-doc-comments-wait-for-639) put two more answers on the same boundary: a
relative heading, and a heading whose path puts a name under a number, in a source doc-comment
declaration body. All three notices shipped with 0.18.0, and a notice already released is not
published again, however it is edited since ([§FS-distribution.4.6.4](../../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions)). So the release that lands
them needs a notice of its own, and this record carries it.

Teaching the doc-comment forms is also a change to the managed block's fixed text, which
[§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) records as a version bump with its remedy.

## 2. Decision

In 0.19.0 the compatibility reading and the `parse-moves` check are removed, and every reader
answers from the record alone (agent-grounds/grund#639):

- each row of the table in [§DF-one-parse-answers-from-0-19.2](DF-one-parse-answers-from-0-19.md#2-decision) gives the record's answer, so rows
  P01, P02, P03, P07, P10 and P12 leave [§REQ-one-answer.differences.sections](../../requirements/REQ-one-answer.md#differencessections-sections);
- a relative or mixed heading in a doc-comment declaration body declares its section under the
  Markdown rules, its parent read from the section map that body has in 0.19.0, so a relative
  child of a `/** … */` line without ` * ` that reads `## 1. Sub` resolves under `.1`
  ([§FS-declarations.section.relative.where](../../functional-spec/FS-declarations.md#sectionrelativewhere-markdown-declaration-bodies-and-doc-comments-from-0190));
- a declaration behind a single `;` or `--` reads its body as every other declaration does, which
  [§DF-line-prefix-forms-declare](DF-line-prefix-forms-declare.md#df-line-prefix-forms-declare-lua-haskell-and-lisp-doc-comment-forms-declare-and-a-lisp-doc-block-is-not-a-note) deferred to this release;
- `parse-moves` is no longer a code, so `--only parse-moves` and `--ignore parse-moves` are
  refused as any unknown code is ([§FS-declarations.checks.parse-moves](../../functional-spec/FS-declarations.md#checksparse-moves-a-verdict-that-moves-in-0190));
- the managed block teaches the relative and mixed forms for doc-comments as well as Markdown, and
  its two current pairs advance, v19/v20 to v23/v24 and v21/v22 to v25/v26 ([§FS-init.2.3.4.5](../../functional-spec/FS-init.md#2345-declaration-forms),
  [§FS-init.2.3.7.1](../../functional-spec/FS-init.md#2371-the-current-version)).

## 3. Rejected alternatives

- **Rewrite the 0.18.0 notice to say it shipped.** The bump would not publish it, so the release
  that changes the answers would say nothing about them.
- **Keep `parse-moves` as a code that never fires, for a release.** It would keep a script's
  `--ignore parse-moves` working, but the code was new in 0.18.0, no released script older than
  that names it, and [§FS-declarations.checks.parse-moves.4](../../functional-spec/FS-declarations.md#checksparse-moves4-removed-in-0190) already announced the refusal. A code
  that selects nothing hides that the selection is stale.
- **Teach the doc-comment forms without a bump.** The block's fixed text is what its version
  versions ([§FS-init.2.3.7](../../functional-spec/FS-init.md#237-the-block-version)); a silent rewrite would leave older blocks teaching the Markdown-only
  rule with no finding to say so.

## release-note: Release note

- [§DF-one-parse-lands-in-0-19](DF-one-parse-lands-in-0-19.md#df-one-parse-lands-in-0-19-every-command-answers-from-one-parse-in-0190-and-the-managed-block-teaches-doc-comment-sections), [§REQ-one-answer.differences.sections](../../requirements/REQ-one-answer.md#differencessections-sections), [§FS-declarations.section.relative.where](../../functional-spec/FS-declarations.md#sectionrelativewhere-markdown-declaration-bodies-and-doc-comments-from-0190), [§FS-init.2.3.7.1](../../functional-spec/FS-init.md#2371-the-current-version), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **every command now reads one parse of each file, the `parse-moves` warning is gone, and relative and mixed doc-comment headings are sections.** What grund 0.18.0 announced lands: `show --full` and `list --size` end a Markdown body at the next same-or-higher heading, plain or declaring, and leave out a `//` note next to a `///` block; a deeper declaration no longer ends its parent's body; `show` ends a section at a plain heading of its level or above; `--toc` stops listing `/// # 1. Sub` and drops a heading's closing hashes from its `title`; a `/** … */` line without ` * ` that reads `## 1. Sub` is section `.1`; and `refs`, `cover --lines`, the chapter rules and the editor end a doc-comment section where `show` does. In a doc-comment declaration body, `/// ### .1: Username` under `/// ## format: Formatting` is section `format.1`, and `/// ### 1.goals: Goals` or `/// ### .goals: Goals` under `/// ## 1. Overview` is `1.goals`, as in Markdown. **Who this affects:** a repository that saw `parse-moves` in 0.18.0 sees the verdict it named; one that did not sees changed output only. `--only parse-moves` and `--ignore parse-moves` are now refused as an unknown code, so drop them from scripts. The managed block teaches the doc-comment forms, so a v19, v20, v21 or v22 block is reported `agents-init` as outdated until one `grund init` re-renders it at v23, v24, v25 or v26. Closes [issue #639](https://github.com/agent-grounds/grund/issues/639).
