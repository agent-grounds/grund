# DF-a-near-miss-title-citation-is-a-citation: a citation on a near-miss declaration's heading is a citation, as on any declaration's

**Date:** 2026-10-23

Completes [§DF-a-title-citation-is-a-citation](DF-a-title-citation-is-a-citation.md#df-a-title-citation-is-a-citation-a-citation-on-a-declarations-heading-is-a-citation-unless-it-cites-the-id-that-heading-declares) for the one declaration it left out: a heading [§FS-declarations.checks.declaration-near-miss](../../functional-spec/FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss) flags. Reported as [grund#945](https://github.com/agent-grounds/grund/issues/945).

## status: Accepted

## 1. Context

A near-miss heading is a declaration: it is kept in the catalog under its token as written and resolves through every reader ([§FS-config.3.2.5](../../functional-spec/FS-config.md#325-off-grammar-declarations-stay-readable)), and its error never changes what is recognized ([§FS-declarations.checks.declaration-near-miss.5](../../functional-spec/FS-declarations.md#checksdeclaration-near-miss5-a-warning-before-0160-an-error-in-it)). Its title was not read. Reproduced on `main` at 85471deefd (0.19.1-dev), under the default `{kind}-{number}-{slug}`, on this tree:

```text
docs/functional-spec/FS-001-alpha.md    # FS-001-alpha: Alpha

                                        body

                                        # FS-beta: Beta, per §FS-001-alpha
```

`grund check` reports the near miss at line 5, as it should, and also `FS-001-alpha.md:1: warning: declared but never cited: FS-001-alpha`, and `grund refs FS-001-alpha` lists only the index entry. With `FS-beta` renamed `FS-002-beta`, `refs` lists line 5 and the warning goes. A title citation of an ID nothing declares draws no `dangling` error on the near miss, and does on the conforming twin. The line-prefix doc comments (`;;`, `---`, `-- |`), a Python docstring, a bare line of a `/* … */` block and a `///` line drop it the same way, and the reviewed goldens of `line-prefix-forms-near-miss` and two other cases pinned the drop.

0.18.0 counted a declaration's title citations ([§DF-a-title-citation-is-a-citation](DF-a-title-citation-is-a-citation.md#df-a-title-citation-is-a-citation-a-citation-on-a-declarations-heading-is-a-citation-unless-it-cites-the-id-that-heading-declares)), and [§FS-citations.recognized.title-lines](../../functional-spec/FS-citations.md#recognizedtitle-lines-a-declarations-heading-cites-what-it-names) drops only a citation of the ID the heading declares. The scan reads a near-miss heading on a path of its own, which records the finding and the declaration and then leaves the line before the citation scan, so the correction never reached it.

## 2. Decision

A near-miss heading's line is read for citations exactly as a conforming declaration's is ([§FS-citations.recognized.title-lines](../../functional-spec/FS-citations.md#recognizedtitle-lines-a-declarations-heading-cites-what-it-names)). The ID it declares is its token as written, the spelling the catalog keeps, so `# FS-beta: Beta, see` its own `FS-beta` cites nothing, and every other citation on the line counts. On the tree above, `refs FS-001-alpha` lists `FS-001-alpha.md:5`, the unused warning on `FS-001-alpha` goes, a title citation that resolves to nothing is reported as dangling, and the citation directions judge it from the near-miss declaration it sits on. Nothing else about the near miss moves: its `declaration-near-miss` error, its catalog entry and its body read as before, `fmt` still never rewrites its heading ([§FS-fmt.2.3](../../functional-spec/FS-fmt.md#23-what-is-never-rewritten)), and its title citation is still not an index entry ([§FS-check.3.18.5.1](../../functional-spec/FS-check.md#31851-what-appears-means)).

## 3. Rejected alternative: leave a near miss's title unread

The heading is already an error, so its title could stay unread until it is renamed. That makes the rename the error asks for move a second verdict: renaming `FS-beta` to `FS-002-beta` would make its title citations appear, a dangling one included, and an ID cited only there would stop being unused. The finding would decide what is read, which [§FS-declarations.checks.declaration-near-miss.5](../../functional-spec/FS-declarations.md#checksdeclaration-near-miss5-a-warning-before-0160-an-error-in-it) rules out. Counting it after a warning release is rejected for the reason [§DF-a-title-citation-is-a-citation](DF-a-title-citation-is-a-citation.md#df-a-title-citation-is-a-citation-a-citation-on-a-declarations-heading-is-a-citation-unless-it-cites-the-id-that-heading-declares) gives: it keeps a citation unread that a repository wrote.

## 4. Consequences

The verdict moves both ways, on near-miss heading lines that cite another ID only.

- **Pass to fail.** A near-miss title citation that dangles is now a `dangling` error at its line. One that a `must-not` direction forbids is now a `forbidden-citation` error. One that names a section its target lacks is a `missing-section` error, one under an unknown alias is an `unknown-project` error, and a number-only shorthand is a `shorthand-citation` error. The same holds for each one's `out-of-scope-` form, and a glob there is a `glob-citation` error. The line already carries the `declaration-near-miss` error, so a default run's exit code does not move. A run that leaves that code out, with `--ignore`, can now exit 1.
- **New suggestions.** An escaped title citation that resolves is an `escaped-citation-resolves` suggestion, and one a `should-not` direction discourages is a `discouraged-citation` suggestion.
- **Fail to pass, or quieter.** An ID cited only from a near-miss title is no longer `unused`, and a `must` direction such a citation satisfies is now satisfied.
- **Counts move.** `refs`, `cover` and the inbound counts include the citation, in the CLI, the bindings and the editor alike, which read the one scan.

A near-miss title that cites only its own ID, and one with no citation, answer as before.

## 5. The correction route

[§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) licenses the move in one release, and each of its five conditions holds.

1. *Prior prohibition.* The old verdict violated [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded), which already applied when that verdict shipped: 0.18.0 is the first release whose specification reads every title line but for the declared ID, and both it and 0.19.0 drop every citation on a near-miss heading line. That section makes a skip no section names the bug. Nothing names this one: [§FS-citations.recognized.title-lines](../../functional-spec/FS-citations.md#recognizedtitle-lines-a-declarations-heading-cites-what-it-names) excepts only the declared ID, and [§FS-declarations.checks.declaration-near-miss.5](../../functional-spec/FS-declarations.md#checksdeclaration-near-miss5-a-warning-before-0160-an-error-in-it) says a near miss is recognized as any declaration is.
2. *Accepted proof.* This record. The [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path does not fit, because there is no old form a user chose, only citations the user wrote that the tool did not read, and keeping them unread a release longer keeps a dangling one passing. The [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) mechanical migration has no command to point at, because nothing in the repository needs rewriting: the titles are right as written.
3. *Named release.* The release note below names the verdict move in both directions, and says who sees it.
4. *Actionable findings.* Every finding that replaces the old verdict is one a citation anywhere else already gets, at the title's line and naming the ID and the edit: those the consequences above list, or the absence of `unused`. Each names its location and the action to take.
5. *No new licence.* This is not a licence for anything else. No rule, finding or prohibition is new, `fmt` gains no write on a near-miss heading, and the `declaration-near-miss` error is unchanged.

## release-note: Release note

- [§DF-a-near-miss-title-citation-is-a-citation](DF-a-near-miss-title-citation-is-a-citation.md#df-a-near-miss-title-citation-is-a-citation-a-citation-on-a-near-miss-declarations-heading-is-a-citation-as-on-any-declarations), [§FS-citations.recognized.title-lines](../../functional-spec/FS-citations.md#recognizedtitle-lines-a-declarations-heading-cites-what-it-names), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded): **a citation in a near-miss declaration's title is now a citation.** A heading `check` reports as `declaration-near-miss`, such as `# FS-beta: Beta, per` a cited `FS-001-alpha` under the default `{kind}-{number}-{slug}`, now cites it, as a conforming declaration's title has since 0.18.0: `refs FS-001-alpha` lists it, and `FS-001-alpha` is no longer reported unused. This holds in every form a near miss takes, Markdown headings and doc comments alike, and only a citation of the near miss's own ID, as written, is still dropped. **Who this breaks:** a repository whose near-miss headings cite other IDs in their titles. The verdict moves both ways: a title citation there that dangles, names a missing section or an unknown project alias, or is a number-only shorthand is now an error (`dangling`, `missing-section`, `unknown-project`, `shorthand-citation`), a direction rule now judges it (`forbidden-citation`, `discouraged-citation`), a glob there is a `glob-citation` error, and every finding names its location and the fix, as for a citation anywhere else. Renaming the ID to the configured format, as the `declaration-near-miss` error asks, no longer changes what its title cites.
