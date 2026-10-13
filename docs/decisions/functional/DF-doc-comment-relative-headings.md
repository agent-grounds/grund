# DF-doc-comment-relative-headings: a relative heading in a doc-comment warns in 0.18.0 and is a section from 0.19.0

**Status:** Accepted
**Date:** 2026-10-13

## context: Context

[§DF-relative-section-headings.decision.where](DF-relative-section-headings.md#decisionwhere-markdown-declaration-bodies-only) gave the relative heading, `### .1: Username` under `## format: Formatting`, its meaning only inside Markdown declaration bodies, because there every dotted heading was already an `unmarked-heading` error and no verdict moved. In a source doc-comment the same line is prose and passes `check`, so giving it a meaning moves a verdict a default `check` reaches. [Issue #665](https://github.com/agent-grounds/grund/issues/665) asks for the doc-comment half on the route [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) prescribes: a warning release, then the meaning ([§FS-declarations.section.relative.where](../../functional-spec/FS-declarations.md#sectionrelativewhere-markdown-declaration-bodies-and-doc-comments-from-0190)).

## decision: Decision

In 0.18.0 such a line keeps its meaning and warns at its line, naming the release in which it becomes a section, the complete-path heading that is that section in both releases, and how to keep it prose: a bold label or a code span. From 0.19.0 it declares its section under the form, parent, lock and display rules of Markdown. The warning is the fourth shape of `parse-moves` ([§FS-declarations.checks.parse-moves.2](../../functional-spec/FS-declarations.md#checksparse-moves2-the-four-shapes)), and the meaning lands with the one-parse change ([§RM-doc-comment-relative-headings](../../roadmap.md#rm-doc-comment-relative-headings-give-a-relative-heading-in-a-doc-comment-its-meaning-in-0190)).

### decision.code: It rides `parse-moves`

`parse-moves` is the 0.18.0 warning for a doc-comment line whose `check` verdict moves in 0.19.0, and its second shape, a `/** … */` line without ` * ` that becomes a section, is this one's near twin. Riding it gives one story one code, one message form, one LSP path and one removal ([§FS-declarations.checks.parse-moves.4](../../functional-spec/FS-declarations.md#checksparse-moves4-removed-in-0190)). The meaning is read off the same per-file record the one-parse change rewrites ([§AR-scanner.2.8](../../architecture/AR-scanner.md#28-the-file-record)), so landing it apart would change that code twice.

### decision.selector: Why one selector is safe

The shape cannot be selected on its own: `--ignore parse-moves` hides it with the other three. That is harmless because `parse-moves` is new in 0.18.0 and absent from v0.17.0, so no released script passes `--ignore parse-moves` yet, and no script can have opted out of the first three shapes and so missed the fourth.

### decision.coupling: Coupled to #639

The meaning ships with [#639](https://github.com/agent-grounds/grund/issues/639). If that slips past 0.19.0, this does too, and every `parse-moves` message names the later release; the window only lengthens, which is the safe direction, as [§DF-absorbed-scan-warning.2.3](DF-absorbed-scan-warning.md#23-the-release-is-named-in-the-message-not-only-in-the-changelog) reads it. This holds only while 0.19.0 is one release away: if 0.18.0 is tagged before this warning ships, the shape takes a code of its own whose meaning lands in 0.20.0.

### decision.moves: One finding per line

From 0.19.0 a citation below such a line moves into the new section, and a citation of its coordinate stops dangling. The line's single warning is the notice for those moves: one finding per line, none per citation.

## rejected: Rejected alternatives

- **A code of its own.** It would let the meaning land apart from #639, at the cost of a second code, a second catalog row and a second deadline for one story, "this doc-comment line moves in 0.19.0".
- **The meaning at once.** It moves a verdict that passes today without a release of notice, which [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) forbids.
- **An `fmt` rewrite between the forms.** The ruling on #643 keeps no automatic migration, and the warning names the edit instead.

## release-note: Release note

- [§FS-declarations.checks.parse-moves.2](../../functional-spec/FS-declarations.md#checksparse-moves2-the-four-shapes), [§FS-declarations.section.relative.where](../../functional-spec/FS-declarations.md#sectionrelativewhere-markdown-declaration-bodies-and-doc-comments-from-0190), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **a relative section heading in a source doc-comment warns, and becomes a section in 0.19.0.** In a doc-comment declaration body, a line such as `/// ### .1: Username` under `/// ## format: Formatting` is prose in this release, as before, and `check` now warns `parse-moves` at it: from grund 0.19.0 it is the heading of `<ID>.format.1`, as it is in Markdown. Write it `### format.1: Username` to make it that section in both releases, or make it a bold label or a code span to keep it prose in both. **Who this affects:** only a repository with such a line in a doc-comment declaration body. Its `check` still exits `0`, but prints the warning where it printed `success`, and under `--format json` a finding object appears where stdout was empty; the editor shows the same warning. `--ignore parse-moves` hides this shape together with the other three. Closes [issue #665](https://github.com/agent-grounds/grund/issues/665).
