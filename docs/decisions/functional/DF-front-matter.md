# DF-front-matter: A Markdown file's front matter is a region of its own, owned by the file's first declaration and never rewritten

**Date:** 2026-10-27

## status: Accepted

## 1. Context

grund had no notion of Markdown front matter, so the `---` or `+++` block at the top of a
page, which Jekyll, Hugo, Zola, Pandoc, Docusaurus and the Open Knowledge Format read as
YAML or TOML, was read as ordinary Markdown
([#1046](https://github.com/agent-grounds/grund/issues/1046), reproduced on `main` at
ea84bfaff8). Three things followed:

- A YAML or TOML comment at the start of a line, `# METRIC-gross: …`, is an ATX heading by
  shape, so it declared `METRIC-gross`. One that repeated the file's own ID made the real
  heading a `duplicate declaration`. That contradicts
  [§FS-declarations.requirements.mentions](../../functional-spec/FS-declarations.md#requirementsmentions-writing-about-an-id-never-declares-it--realized-one-case-deferred): a line that names an ID without introducing it
  never declares it.
- A `§` citation in the block sat above every declaration body, so it had no enclosing
  declaration. `refs --from` did not list it, and a rule such as
  `Each METRIC must cite at least one POL.` failed on a metric that cites a policy.
- `grund fmt --write` turned a front-matter citation into a Markdown link. In YAML a value
  that starts with `[` opens a flow sequence, so the block stopped parsing, and every tool of
  the format lost every key in it.

## 2. Decision

### 2.1 The extent, the heading exemption, the live citations

Front matter is a fourth Markdown context beside the fence, the raw-text HTML block and
prose ([§FS-check.1.1.5.2](../../functional-spec/FS-check.md#1152-front-matter-holds-no-heading)). It opens on line 1 with exactly `---` or `+++` and closes at the
next line that is exactly the same delimiter, or `...` for `---`; an unclosed block is not
front matter. No line of it is a heading, and its citations stay live. The rule is decided
once, in the one reading of Markdown block structure every surface already shares, so the
scanner, `fmt`, the editor and the link reader cannot disagree about it.

### 2.2 Its citations belong to the file's first declaration

A front-matter citation, and every line of the block, is owned by the file's first
declaration, under no section ([§FS-check.1.1.5.2.1](../../functional-spec/FS-check.md#11521-its-citations-belong-to-the-files-first-declaration)). Every format the issue met keeps one
document per file, so the first declaration is the one the front matter sits directly above.
The declaration's body does not grow to line 1: `show`, `--toc`, the section locks and the
body extents stay byte-identical, and only the one lookup that answers "who owns this line"
learns the block.

### 2.3 `fmt` never writes inside it

Front matter joins the zones no `fmt` pass rewrites ([§FS-citations.zones](../../functional-spec/FS-citations.md#zones-where-citation-text-is-never-rewritten)), because a Markdown
link is not a valid YAML or TOML value. `check` follows: it withholds the
`; run grund fmt --write` offer there ([§FS-check.3.24.1](../../functional-spec/FS-check.md#3241-the-release-attribution-and-where-the-command-clause-is-withheld)) and does not demand an index entry
there as a link ([§FS-check.3.17.4](../../functional-spec/FS-check.md#3174-only-a-citation-fmt-would-wrap-reaches-this-rule)), so no finding names a command that would rewrite nothing.
The shorthand error itself still fires, because the canonical form is valid YAML and TOML and
the author can always write it by hand; front matter is not added to
[§FS-citations.rewrite-exemption](../../functional-spec/FS-citations.md#rewrite-exemption-where-the-text-forbids-the-rewrite).

## 3. Rejected alternatives

- **Starting the first declaration's body at line 1.** Ownership would follow for free, but
  `show` would print the front matter, the lead's fingerprint would move, and every section
  lock of such a file would go stale.
- **Parsing the YAML or TOML.** Reading the values, holding them to the kind, and making the
  block a field of its declaration are a larger question, kept in
  [#1042](https://github.com/agent-grounds/grund/issues/1042). The extent needs none of it.
  That question was decided later, for YAML and on a kind that asks for it, in
  [§DF-front-matter-field](DF-front-matter-field.md#df-front-matter-field-a-kinds-front-matter-field-holds-its-files-yaml-front-matter-with-the-blocks-top-level-keys-as-its-sections).
- **Front matter only for YAML.** `+++` is Hugo's and Zola's, and its `#` comments raise the
  same false declaration.
- **Treating front matter as a fence.** Its citations would go dead, and the relationship
  the block was written to carry would be the one thing grund could not see.

## 4. Consequences

- **Fail to pass.** A false `duplicate declaration` from a comment goes. A rule or obligation
  that a front-matter citation satisfies passes, and `refs --from` lists it. `fmt --check`
  stops reporting front-matter citations, and `fmt --write` stops breaking the block.
- **Pass to fail.** Three shapes can turn a run red. A citation of an ID only a front-matter
  comment declared now dangles as an `unknown reference`. A file whose line 1 is a `---`
  thematic break and which has another `---` line later loses the headings between the two,
  so their IDs are no longer declared. A front-matter citation that a `never` direction of
  its owner's kind, or a declaration-scoped rule, forbids is now judged against that owner
  and reported.
- **Untouched.** A file with no front matter, every non-Markdown file, an unclosed block,
  and a block with no heading-shaped line and no citation read exactly as before. That
  includes every shipped `SKILL.md`, whose front matter has neither. Line and column of
  every finding are unchanged.

## 5. The correction route

The verdicts move under [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids). The conditions are argued per change
that can turn a run red, because condition 1 asks for a prohibition at a numbered section for
each, and for one of them there is none.

1. *Prior prohibition.*
   - *A phantom declaration now dangles.* The old verdict resolved a citation to a YAML or
     TOML comment, which [§FS-declarations.requirements.mentions](../../functional-spec/FS-declarations.md#requirementsmentions-writing-about-an-id-never-declares-it--realized-one-case-deferred) says is no declaration.
     That old verdict is a wrong resolution, forbidden by [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution).
     The prohibition already applied when the Markdown reading of front matter shipped, since before front matter reached any repository grund reads.
   - *A `---` thematic break on line 1 hides the headings after it.* The old verdict declared
     IDs at headings that Jekyll, Hugo and GitHub's renderer all read as front matter, so no
     reader of the rendered page could see them: a citation of one resolved to a declaration
     the page does not show. That is [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution) again, the argument
     [§FS-check.1.1.5.1](../../functional-spec/FS-check.md#1151-a-raw-html-block-holds-no-heading) made for the raw-text HTML block. No file in grund, rhei, ephor or
     fissile has this shape.
   - *A front-matter citation is judged against its owner.* No section forbade the old
     verdict here. The old reading saw the citation, resolved it and counted it as a use;
     it only attributed it to no declaration, and no requirement says a citation must have
     an owner. What the old verdict did violate is the opposite direction:
     [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms), a false alarm, where a `must cite` rule or obligation was
     reported unmet on a declaration that cites the target in its front matter. The red
     shape is the same correction seen from the other side, not a separate rule: a citation
     cannot be counted for its declaration's obligations and kept from its declaration's
     prohibitions. This record says so plainly rather than claim a prohibition that is not
     there, and it takes the red shape as a consequence of that correction, as
     [§DF-line-prefix-forms-declare](DF-line-prefix-forms-declare.md#df-line-prefix-forms-declare-lua-haskell-and-lisp-doc-comment-forms-declare-and-a-lisp-doc-block-is-not-a-note) took the duplicates and near misses its newly found
     declarations raised.
2. *Accepted proof.* This record.
   The [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path does not fit:
   a release that warned first would keep the false declarations, the false
   duplicates and the false `must cite` failures a release longer, which
   [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution) and [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms) forbid.
   The [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) mechanical migration does not fit either:
   there is nothing to migrate, because front matter is right as written.
3. *Named release.* The release note below.
4. *Actionable findings.* Every finding that replaces an old verdict is one grund already
   reports elsewhere, at the citation's own line and column, naming its fix: an
   `unknown reference`, a direction violation or a rule finding.
5. *No new licence.* No rule, finding or prohibition is new. The owner is an existing
   reading, the enclosing declaration of [§AR-scanner.2.4.3](../../architecture/AR-scanner.md#243-the-enclosing-declaration), given an answer where it had
   none; every rule that now sees the citation is one the repository wrote. That `fmt`
   writes nothing in front matter serves [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own) and moves no verdict toward red.
   This record is not a licence for any other verdict change.

## release-note: Release note

- [§DF-front-matter](DF-front-matter.md#df-front-matter-a-markdown-files-front-matter-is-a-region-of-its-own-owned-by-the-files-first-declaration-and-never-rewritten), [§FS-check.1.1.5.2](../../functional-spec/FS-check.md#1152-front-matter-holds-no-heading), [§FS-check.1.1.5.2.1](../../functional-spec/FS-check.md#11521-its-citations-belong-to-the-files-first-declaration), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids),
  [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution), [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms): **A Markdown file's front matter is
  read as front matter.** A `---` or `+++` block on line 1, closed by the same delimiter or by
  `...` for `---`, holds no heading, so a `# ID: …` YAML or TOML comment declares nothing.
  Its `§` citations still resolve and now belong to the file's first declaration: `refs`
  and `cover` name it as the enclosing declaration, `refs --from`, rules and obligations
  count them, and citation directions judge them by its kind. `grund fmt` rewrites nothing
  inside the block, so a front-matter citation is no longer turned into a Markdown link that
  breaks the YAML. The verdict moves both ways. **Who this breaks:** a repository that cites
  an ID only a front-matter comment declared (now an `unknown reference`), one whose file
  opens with a `---` thematic break followed by another `---` line (the headings between
  them no longer declare), and one whose front-matter citation a `never` direction or a
  declaration-scoped rule forbids (now reported); every finding names its location and the
  fix. Closes [#1046](https://github.com/agent-grounds/grund/issues/1046).
