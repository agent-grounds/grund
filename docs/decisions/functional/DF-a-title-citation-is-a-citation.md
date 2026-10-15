# DF-a-title-citation-is-a-citation: a citation on a declaration's heading is a citation, unless it cites the ID that heading declares

**Date:** 2026-10-12

## status: Accepted

## 1. Context

A declaration's title may name what it builds on. Reproduced on `main` at 89aa15b51a, on
this tree:

```text
docs/fs.md    # FS-a: Alpha, built on §FS-b

              Alpha body.

              # FS-b: Beta

              Beta body.
```

`grund check` warns `docs/fs.md:5: declared but never cited: FS-b`, and `grund refs FS-b`
lists nothing. With the title citing `<§>FS-z` instead, an ID nothing declares, `check` passes and
`grund refs FS-z` says `FS-z` is neither declared nor cited. The scan dropped every
citation on a declaration's heading line, not only the one of the ID that line declares.
No section of the specification names that blind spot: [§AR-scanner.2.3](../../architecture/AR-scanner.md#23-citation-detection) says only that a
declaration's own heading line is never counted as a citation *of the ID it declares*, and
[§FS-citations.recognized.contexts](../../functional-spec/FS-citations.md#recognizedcontexts-contexts-read-as-neither-prose-nor-code) lists every context read as neither prose nor code
without a heading among them.

[§DF-one-answer-rulings-parse](DF-one-answer-rulings-parse.md#df-one-answer-rulings-parse-one-parse-gives-every-command-the-same-extents-comment-blocks-anchors-and-title-citations) ruled the answer (D17f, V-A19): on a declaration's heading,
only a citation of the declared ID is dropped. It named the fix a candidate for the
correction route.

## 2. Decision

A citation on a declaration's heading line is a citation like any other, except a citation
of the ID that heading declares ([§FS-citations.recognized.title-lines](../../functional-spec/FS-citations.md#recognizedtitle-lines-a-declarations-heading-cites-what-it-names)). On the tree above,
`refs FS-b` lists `docs/fs.md:1`, the unused-declaration warning on `FS-b` goes, a title
citation that resolves to nothing is reported as dangling, and the citation-direction rules
judge it from the declaration it sits on. `fmt` still never rewrites a declaration's heading
line ([§FS-fmt.2.3](../../functional-spec/FS-fmt.md#23-what-is-never-rewritten)), and a title citation is still not an index entry ([§FS-check.3.18](../../functional-spec/FS-check.md#318-declaration-missing-from-its-kinds-index)).

## 3. Rejected alternative: count it after a warning release

The citation could stay invisible for a release while a warning announced it. That would
keep, for one more release, a citation a repository wrote and `check` cannot see: a
dangling one passes, and the declaration it names is reported unused. A blind spot nobody
declared is exactly what the correction route below says no release may keep.

## 4. Consequences

The verdict moves both ways, on title lines that cite another ID only.

- **Pass to fail.** A title citation that dangles is now a `dangling` error at its line, one
  that a `must-not` direction forbids is now a `forbidden-citation` error, one that names a
  section its target does not have is a `missing-section` error, one under an alias no
  workspace member carries is an `unknown-project` error, and a number-only shorthand is a
  `shorthand-citation` error, as is each one's `out-of-scope-` form in an out-of-scope file.
  `fmt` never rewrites a title ([§FS-fmt.2.3](../../functional-spec/FS-fmt.md#23-what-is-never-rewritten)), so a shorthand in one is fixed by hand: the
  message names the full ID to write.
- **New warnings and suggestions.** A glob in a title is a `glob-citation` warning, an
  escaped title citation that resolves is an `escaped-citation-resolves` suggestion, and one
  a `should-not` direction discourages is a `discouraged-citation` suggestion. None of them
  changes the exit code.
- **Fail to pass, or quieter.** A declaration cited only from another declaration's title is
  no longer `unused`, and a `must` direction a title citation satisfies is now satisfied.
- **Counts move.** `refs`, `cover` and the inbound counts include the title citation.

A title that cites only its own ID, and a title with no citation, answer as before.

## 5. The correction route

[§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) licenses the move in one release, and each of its five conditions holds.

1. *Prior prohibition.* The old verdict violated [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded), which already applied when that verdict shipped: it arrived in b12e985d5f, first released in 0.10.1, and it is present in 0.17.0, whose scan drops every citation on a declaration heading line. That section makes a skip no section names the bug, and the title line was such a skip: [§AR-scanner.2.3](../../architecture/AR-scanner.md#23-citation-detection) drops only the declared ID and [§FS-citations.recognized.contexts](../../functional-spec/FS-citations.md#recognizedcontexts-contexts-read-as-neither-prose-nor-code) names no heading, so a citation there was neither checked nor declared unread.
2. *Accepted proof.* This record. The [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path does not fit, because there is no old form a user chose, only citations the user wrote that the tool did not read, and keeping them unread a release longer keeps a dangling citation passing. The [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) mechanical migration has no command to point at, because nothing in the repository needs rewriting: the titles are right as written.
3. *Named release.* The release note below names the verdict move in both directions, and says who sees it.
4. *Actionable findings.* Every finding that replaces the old verdict is one a citation anywhere else already gets, at the title's line and naming the ID and the edit: those the consequences above list, or the absence of `unused`. Each names its location and the action to take.
5. *No new licence.* This is not a licence for anything else. No rule, finding or prohibition is new, and `fmt` gains no write on a declaration's heading.

## release-note: Release note

- [§DF-a-title-citation-is-a-citation](DF-a-title-citation-is-a-citation.md#df-a-title-citation-is-a-citation-a-citation-on-a-declarations-heading-is-a-citation-unless-it-cites-the-id-that-heading-declares), [§FS-citations.recognized.title-lines](../../functional-spec/FS-citations.md#recognizedtitle-lines-a-declarations-heading-cites-what-it-names), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded): **a citation in a declaration's title is now a citation.** A title such as `# FS-a: Alpha, built on` a cited `FS-b` now cites it: `refs FS-b` lists it, and `FS-b` is no longer reported unused. Only a citation of the ID the heading declares is still dropped. **Who this breaks:** a repository whose titles cite other IDs. The verdict moves both ways: a title citation that dangles, names a missing section or an unknown project alias, or is a number-only shorthand is now an error (`dangling`, `missing-section`, `unknown-project`, `shorthand-citation`), a direction rule now judges it (`forbidden-citation`, `discouraged-citation`), a glob in a title is a `glob-citation` warning, and every finding names its location and the fix, as for a citation anywhere else. `fmt` still never rewrites a title, so a shorthand there is written out by hand, to the full ID its message names.
