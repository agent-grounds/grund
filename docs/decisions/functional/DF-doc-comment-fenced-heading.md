# DF-doc-comment-fenced-heading: a heading inside a fence in a comment block is not a section

**Date:** 2026-10-13

## status: Accepted

## 1. Context

A doc-comment may draw a fenced example that holds a heading. Reproduced on `main` at
fc0b545f89 ([#641](https://github.com/agent-grounds/grund/issues/641)), with `FS` at the
folder `docs/fs`:

```text
src/lib.rs   /// FS-c: Gamma
             ///
             /// Gamma body.
             ///
             /// ## 1. One
             ///
             /// One body.
             ///
             /// ```text
             /// ## 2. Example
             /// ```
             ///
             /// After the fence cites §FS-c.
             pub struct C;
```

`grund FS-c --toc` lists `## 2. Example` beside `## 1. One`, `grund FS-c.1 --full` stops at
the fence opener, `refs` puts the citation on line 13 in section `2`, and `check` resolves a
citation of `<§>FS-c.2`, a section nobody wrote. A fenced `#### 7.` draws a false heading-level
mismatch, a fenced repeat of `## 1. One` a false duplicate section, and `fmt` links a citation
of `<§>FS-c.2` to the source file. The same body in a Markdown file is read right on every
surface. A `///`, `//!`, `/** … */`, Python docstring and Go `//` block all read it wrong.

The specification disagreed with itself. [§FS-declarations.heading](../../functional-spec/FS-declarations.md#heading-what-a-heading-is) said a line inside a
fenced block is never a heading, with no exception for a comment block, and the scanner's
own plain-heading pass already skipped a plain heading inside a doc-comment fence. But the
last paragraph of [§FS-show.2.5](../../functional-spec/FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example), written in 0.10.1, said a fence is not tracked in a
comment block on either side, and the section scan followed it.

## 2. Decision

Inside a comment block a fence is read through the comment wrapper, and while it is open a
line that would be a section or a plain heading is not a heading
([§FS-languages.fence.comment-block](../../functional-spec/FS-languages.md#fencecomment-block-a-fence-inside-a-comment-block-hides-its-headings)). On the tree above `--toc` lists only `## 1. One`,
`FS-c.1 --full` runs through the fence to line 13, `refs` puts line 13 in section `1`, and
`check` reports `missing section FS-c.2`. Every command reads the one section record the
scan fills, so they move together.

### 2.1 Headings only

The fence hides headings and nothing else. Citations inside it stay live, `grund:fmt off`
inside it still toggles, `show` still flattens a citation wrapper inside it, and a source
declaration line inside it still declares. Each of those is a verdict of its own that the
issue did not report wrong, and moving them would move verdicts this record has not
measured.

### 2.2 A fence closes with its comment block

A fence still open where its comment block ends closes there. An unclosed fence in one
doc-comment must not hide the sections of the next declaration's doc-comment in the same
file; Markdown's "runs to end of file" is about one Markdown body, and a comment block is
the body here.

## 3. Rejected alternatives

- **A warning release first**
  ([§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)). That would keep, for one more release, a citation
  resolving to a section that does not exist and `show` cutting a body open at an example,
  which [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution) forbids. Nothing in a user's repository needs to change:
  the comment is right as written.
- **Track the fence for everything in a comment block**, as Markdown does. That would also
  silence citations and `fmt` directives inside a doc-comment fence, which
  `fmt-directive-in-source-comment` pins live today, and widen the move beyond what was
  measured.
- **Keep 0.17's bytes until 0.19.0** under [§DF-one-parse-answers-from-0-19](DF-one-parse-answers-from-0-19.md#df-one-parse-answers-from-0-19-one-parses-other-answers-ship-in-0190-after-a-release-that-announces-them-and-warns-where-a-verdict-moves). Its rows are
  answers that differ between two readings each defensible on its own; this one is a
  wrong citation verdict, and the compatibility reading would keep it wrong.

## 4. Consequences

The verdict moves both ways, and only on trees with a section-shaped heading line inside a
fence inside a comment block.

- **Pass to fail.** A citation of the coordinate such a line spells is a `missing section`.
- **Fail to pass.** A heading-level mismatch, a duplicate section or a section outside a
  declaration reported on such a line is no longer reported.
- **Answers.** `show`, `--toc`, `list --size`, `refs`' enclosing section, `cover` and the
  editor read the declaration as if the fenced line were prose, and `fmt` no longer links to
  it.
- **Untouched.** Markdown files, citations and `fmt` directives inside a doc-comment fence,
  and declaration lines; no flag, configuration key or output format changes.

The blast radius was measured before this record was settled, with a walk over every
comment block that holds a declaration line, listing each ATX heading inside a fence in it:
grund, rhei, ephor, fissile, akoe and skopos at their `main`, and the workspace's `scripts`
and `docs`. It finds one line, in grund's own `check-parse-moves-fence-info-backtick`
fixture, whose ```` ```a`b ```` is not a fence. No real tree has a fenced heading in a
doc-comment, so no real citation's verdict moves.

## 5. The correction route

[§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) licenses the move in one release, and each of its five
conditions holds.

1. *Prior prohibition.* The old verdict violated [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution), which already applied when it shipped: a section coordinate resolved to a heading the declaration does not have, only an example of one. It has stood since doc-comment sections were recorded, and the sentence that sanctioned it since 0.10.1.
2. *Accepted proof.* This record. The deprecation path of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) does not fit: there is no form a user chose that the tool now refuses, only a fenced example the tool misread, and keeping it misread a release longer keeps the wrong resolution. The mechanical migration of [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) has nothing to rewrite, because the comment is right as written.
3. *Named release.* The release note below names the move in both directions and says who sees it.
4. *Actionable findings.* The one finding the change adds is `missing section`, at the citing line and naming the coordinate, which the author fixes by citing a section that exists.
5. *No new licence.* This correction is not a licence for any other move. No rule, finding or prohibition is new: the change reads one more line as [§FS-declarations.heading](../../functional-spec/FS-declarations.md#heading-what-a-heading-is) already said to, and the measurement in section 4 finds no tree it moves.

## release-note: Release note

- [§DF-doc-comment-fenced-heading](DF-doc-comment-fenced-heading.md#df-doc-comment-fenced-heading-a-heading-inside-a-fence-in-a-comment-block-is-not-a-section), [§FS-languages.fence.comment-block](../../functional-spec/FS-languages.md#fencecomment-block-a-fence-inside-a-comment-block-hides-its-headings),
  [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution): **a heading inside a
  fenced example in a doc-comment or docstring is no longer read as a section.** `--toc`
  stops listing it, `show` no longer cuts the section above it at the fence, `refs` no
  longer names it as an enclosing section, and `fmt` no longer links to it. **Who this
  breaks:** a repository that cites the coordinate such a fenced line spells, which now gets
  `missing section`; the heading-level, duplicate-section and section-outside-declaration
  errors such a line drew go away, so the verdict moves both ways. Every finding the change
  adds names its location, the citing line, and the fix: cite a section that exists. Measured before release over grund, rhei, ephor, fissile
  and their workspace, none of them sees a new finding.
