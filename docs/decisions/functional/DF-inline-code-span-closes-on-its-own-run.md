# DF-inline-code-span-closes-on-its-own-run: an inline code span closes on a run of its own length

**Status:** Accepted
**Date:** 2026-10-04

## 1. Context

[§FS-fmt.2.3](../../functional-spec/FS-fmt.md#23-what-is-never-rewritten) has always kept a citation inside a Markdown inline code span out of every rewrite, and [§FS-check.3.13.1](../../functional-spec/FS-check.md#3131-where-the-text-forbids-the-rewrite), [§FS-check.3.1.2](../../functional-spec/FS-check.md#312-an-illustration-in-inline-code) and [§FS-check.3.17.4](../../functional-spec/FS-check.md#3174-only-a-citation-fmt-would-wrap-reaches-this-rule) read the same spans from the checker's side. None of them said where a span ends, and the one predicate they share answered by parity: every unescaped backtick flipped the line between prose and code. CommonMark does not read a span that way. A span opened by a run of two backticks closes only at the next run of exactly two, which is how Markdown writes a literal backtick in code:

```markdown
Escaped `` \ ` * `` then [§GOAL-a](goals.md#goal-a-stale) here.
```

Five backticks leave the parity reading in code after that span, so every pass read the citation after it as code. `grund fmt --write` left its stale anchor in place and reported the file clean, `grund fmt --check` passed, and the broken link reached a commit; it happened in this repository's own specification, and only a link checker caught it. The same parity error ran the other way inside a span: the text between a double-backtick opener and the next backtick read as prose, so `fmt` rewrote a citation that sat inside code. Both halves come from one missing fact, the length of the run that closes a span.

## 2. Decision

### 2.1 A span closes on a run of exactly its opener's length

[§FS-fmt.2.3.5](../../functional-spec/FS-fmt.md#235-an-inline-code-span-closes-on-a-run-of-its-own-length) states the CommonMark reading one line at a time: a run of *n* unescaped backticks opens a span, the next run of exactly *n* closes it, and a backslash inside the span is literal. Text after the closing run is prose again, and every run inside the span, shorter or longer, is content.

### 2.2 An opener that does not close on its line keeps the rest of the line code

CommonMark reads a run that never closes as literal backticks. The predicate is asked about one line, and a code span may continue onto the next line of its paragraph, so a run with no closer on its line may be the start of a real span. Reading the rest of the line as prose would let `fmt` write into code it cannot see the end of, the persisted guess [§REQ-no-wrong-citation.3](../../requirements/REQ-no-wrong-citation.md#3-no-wrong-write) forbids. Reading it as code is what the parity reading already did for a single stray backtick, so this keeps that answer rather than changing it, and `check` withholds its demand for a rewrite at the same site, so nothing is left that a repository cannot clear.

### 2.3 One predicate, read by every caller

The fix is in the predicate, not in the pass the report named. Patching only the cross-reference wrap would have left the shorthand pass, the canonical-form exemption, the dangling-citation hint and the kind-index entry form on the old reading, so `fmt` and `check` would disagree about the same bytes, which is the opposite of what the report asked for. One predicate keeps one verdict per site, as [§FS-fmt.2.3.5](../../functional-spec/FS-fmt.md#235-an-inline-code-span-closes-on-a-run-of-its-own-length) requires.

## 3. Alternatives considered

**Read an unclosed opener as literal text, as CommonMark does.** Rejected for [§DF-inline-code-span-closes-on-its-own-run.2.2](DF-inline-code-span-closes-on-its-own-run.md#22-an-opener-that-does-not-close-on-its-line-keeps-the-rest-of-the-line-code)'s reason: the reader sees one line, and the first line of a wrapped code span looks exactly like a stray backtick. Recognizing spans across line breaks is a larger change of its own, and nothing in the report needs it.

**Fix only the cross-reference pass.** The smaller change, and the one the report's symptom pointed at. Rejected by [§DF-inline-code-span-closes-on-its-own-run.2.3](DF-inline-code-span-closes-on-its-own-run.md#23-one-predicate-read-by-every-caller).

## 4. What this costs, and why it may be taken

Correcting where a span ends moves text from code to prose after such a span, and from prose to code inside it, so verdicts move both ways. A tree moves from failing to passing where `fmt --check` asked for a rewrite inside a span, or where `check` asked for a canonical form there. Four verdicts move from passing to failing, each on a line that holds a span the parity reading misread:

- `grund fmt --check` exits `1` where a citation after the span needs the rewrite it was always owed, such as the stale anchor in [§DF-inline-code-span-closes-on-its-own-run.1](DF-inline-code-span-closes-on-its-own-run.md#1-context). That rewrite is the fix this decision exists for.
- `grund check` raises the canonical-form error of [§FS-check.3.13](../../functional-spec/FS-check.md#313-number-only-shorthand-citation) for a number-only shorthand after the span under the default `canonical` policy, because the exemption of [§FS-check.3.13.1](../../functional-spec/FS-check.md#3131-where-the-text-forbids-the-rewrite) no longer reaches prose.
- A wrapped index entry inside a span stops being an entry, so `grund check` may raise [§FS-check.3.18](../../functional-spec/FS-check.md#318-declaration-missing-from-its-kinds-index) for the declaration it named.
- In a source file of a workspace, a marked qualified citation inside a span is now skipped as [§FS-check.1.1](../../functional-spec/FS-check.md#11-recognized-citations) says it is, so a declaration it was the only citation of may go uncited and its file ungrounded, an error under `[reference] require_grounding` ([§FS-check.3.6.3](../../functional-spec/FS-check.md#363-findings)), or a citation rule it satisfied may fail.

The route is [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), and this record is the accepted proof it requires. Its five conditions:

1. **Prior prohibition.** The old reading violated [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded) by skipping prose after such a span, a blind spot no section named, so a citation there kept a broken link and a non-canonical form with no finding. The old reading also violated [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own) by rewriting bytes inside a span, which [§FS-fmt.2.3](../../functional-spec/FS-fmt.md#23-what-is-never-rewritten) excludes from what `fmt` owns. And the old reading violated [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution) by counting as a citation, and as an index entry, text the specification says is code. Each prohibition already applied in the release when that verdict shipped.
2. **Accepted proof.** This record, at [§DF-inline-code-span-closes-on-its-own-run.4](DF-inline-code-span-closes-on-its-own-run.md#4-what-this-costs-and-why-it-may-be-taken).
3. **Named release.** The release notes name the verdict change and the four ways a passing tree can fail.
4. **Actionable findings.** Each finding that replaces the old verdict is located and clearable: `grund fmt --check` names the file and line it would rewrite and `grund fmt --write` makes the rewrite; the canonical-form error names its line and the form to write, and `grund fmt --write` writes it; the missing index entry names the declaration and the index, and the action is to write the entry outside the span; the uncited declaration and the ungrounded file are named where they are, and the action is to write the citation outside the span.
5. **No new licence.** This route cannot justify anything broader: nothing is tightened, nothing is removed, and no prohibition is invented by the correcting change. [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) does not fit, because there is no old spelling to keep working: the bytes stay legal and only where a span ends is corrected. [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) does not fit either, because there is no single command for every move: the index entry and the qualified citation in source have to be moved out of the span by hand.
