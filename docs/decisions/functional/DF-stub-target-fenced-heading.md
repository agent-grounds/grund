# DF-stub-target-fenced-heading: a heading inside a fence of a stub's target does not declare its ID

**Status:** Accepted
**Date:** 2026-10-08

## 1. Context

A stub `# <ID>: [<text>](<path>)` is healthy only when its target declares the ID ([§FS-declarations.checks.broken-stub](../../functional-spec/FS-declarations.md#checksbroken-stub-broken-inline-spec-stub)), and `check` and `show` take that answer from one test, which re-reads the target for a declaration heading of the ID. The scan reads a Markdown file with its fence state first: a fence delimiter line and every line while a fence is open bypass declaration detection ([§AR-scanner.2.3.3](../../architecture/AR-scanner.md#233-fence-state-is-decided-first)), so a heading inside a fence is an example and declares nothing ([§FS-show.2.5](../../functional-spec/FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example)). The stub's test kept no fence state. A target whose only `# <ID>:` heading was a fenced example passed it, while the scan recorded no declaration there:

~~~markdown
# Notes

An example:

```markdown
# FS-second: Second

Fenced lead.
```
~~~

With a stub `# FS-second: [../target.md](../target.md)` pointing at that file, `grund check` printed `success`, so every citation of the ID passed the gate with no body behind it. `grund show FS-second` got past its broken-stub refusal, found no record for the target, and answered `ID not found` for an ID `grund list` names, which sent the reader after a missing declaration when the stub was what was wrong. Both happened whether or not the target was inside `[scan] include`, because the scan skips the fenced heading either way. A document that shows the declaration shape as a fenced example is common, and this repository's own specifications do it throughout, so renaming or deleting the real declaration beside such an example left its stub green. Reported as [issue #536](https://github.com/agent-grounds/grund/issues/536).

## 2. Decision

### 2.1 The target is read as the scan reads it

[§FS-declarations.checks.broken-stub.2](../../functional-spec/FS-declarations.md#checksbroken-stub2-a-heading-inside-a-fence-of-the-target-declares-nothing): in a Markdown target, a fence delimiter line and every line while a fence is open declare nothing, by the fence rules of [§FS-check.1.1.5](../../functional-spec/FS-check.md#115-contexts-read-as-neither-prose-nor-code). A target whose only heading of the ID is fenced makes its stub broken: `check` reports it at the stub's line, and `show` refuses with the second line of [§FS-show.2.3.4](../../functional-spec/FS-show.md#234-broken-stub). A fenced example beside the real declaration changes nothing: the stub is healthy, and `show` reads the real heading.

### 2.2 One fence reader

The fix is in the stub's test, through the fence reader the scan and the body reader already share, not through a second notion of a fence. A test with its own reading would drift from the scan again at the next corner — a tilde fence, a longer closer, a fence never closed — which is how this one drifted. Fences are tracked in a Markdown target only, as the scan tracks them ([§FS-show.2.5](../../functional-spec/FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example)).

## 3. Alternatives considered

**Fix `show` alone.** `show` could refuse a stub whose target has no record without asking the test. Rejected: `check` would still pass a citation with no body behind it, which is the defect, and the two commands would disagree about one stub.

**Report the fenced heading itself.** A finding at the example, saying it looks like the declaration. Rejected: a heading inside a fence is an example by design, and the near-miss rule never reads one ([§FS-declarations.checks.declaration-near-miss.3](../../functional-spec/FS-declarations.md#checksdeclaration-near-miss3-never-in-inline-code-prose-or-a-fenced-block)). What is wrong is the stub, whose target lacks the declaration, so the finding stays at the stub.

## 4. What this costs, and why it may be taken

One verdict moves, from passing to failing: `grund check` exits `1` with `stub link target lacks <ID>: <path>` at a stub whose Markdown target declares the ID only inside a fenced block, where it printed `success`. On the same stub, `grund show <ID>` moves from one refusal to another, `ID not found` to the `broken stub:` line, and its exit code stays `1`. No tree moves from failing to passing.

The route is [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), and this record is the accepted proof it requires. Its five conditions:

1. **Prior prohibition.** The old verdict violated [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution) by accepting a citation of the ID on the strength of a fenced example the specification says is not a declaration ([§FS-show.2.5](../../functional-spec/FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example)): the citation resolved to no declaration at all, where it must resolve to exactly the declaration its ID names or be reported. The prohibition already applied in the release when that verdict shipped.
2. **Accepted proof.** This record, at [§DF-stub-target-fenced-heading.4](DF-stub-target-fenced-heading.md#4-what-this-costs-and-why-it-may-be-taken).
3. **Named release.** The release notes name the verdict change and the two ways to clear it.
4. **Actionable findings.** The one finding that replaces the old verdict is located and clearable: `stub link target lacks <ID>: <path>` names the stub's file and line and the target it points at, and `show` names the same stub and target in its refusal. The action is to point the stub at the file that holds the real declaration, or to move the declaration out of the fence.
5. **No new licence.** This route cannot justify anything broader: nothing is tightened, nothing is removed, and no prohibition is invented by the correcting change, because a fenced heading was never a declaration to the scan ([§AR-scanner.2.3.3](../../architecture/AR-scanner.md#233-fence-state-is-decided-first)) and only the stub's test is brought into line with it. [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) does not fit, because there is no old spelling to keep working: the fenced heading never declared the ID anywhere else. [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) does not fit either, because there is no single command for the move: which file holds the real declaration, if any, is the maintainer's to say.

## release-note: Release note

- [§FS-declarations.checks.broken-stub.2](../../functional-spec/FS-declarations.md#checksbroken-stub2-a-heading-inside-a-fence-of-the-target-declares-nothing), [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), [§DF-stub-target-fenced-heading](DF-stub-target-fenced-heading.md#df-stub-target-fenced-heading-a-heading-inside-a-fence-of-a-stubs-target-does-not-declare-its-id): a heading inside a fenced code block of a stub's Markdown target no longer counts as the target's declaration of the stub's ID, as the scan never counted it. The broken-stub test read every line of the target, fenced or not, so a stub whose target declared its ID only in a fenced example passed `grund check` with no body behind it, and `grund show <ID>` answered `ID not found` for an ID `grund list` names. The verdict changes from passing to failing on such a tree, whether or not the target is inside `[scan] include`: `check` reports `stub link target lacks <ID>: <path>` at the stub's line, and `show` refuses with the `broken stub:` line instead of `ID not found`. A target that holds a fenced example beside the real declaration stays healthy. Every finding that replaces the old verdict names its location and the action to take: point the stub at the file that holds the real declaration, or move the declaration out of the fence. Closes [issue #536](https://github.com/agent-grounds/grund/issues/536). (PR #537)
