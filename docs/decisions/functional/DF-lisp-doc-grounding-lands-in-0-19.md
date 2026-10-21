# DF-lisp-doc-grounding-lands-in-0-19: an ungrounded Lisp doc-comment is the ordinary error from 0.19.0

**Date:** 2026-10-21

## status: Accepted

## 1. Context

[§DF-line-prefix-forms-declare.grounding](DF-line-prefix-forms-declare.md#grounding-a-lisp-doc-block-that-cites-nothing-warns-first) made a Lisp doc-comment a grounding unit under
`grounding_level` 2 or higher, and because that reached code no rule had judged, an ungrounded
one took the deprecation path of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): through 0.18.x it was a
`warning:` ending `this warning becomes an error in grund 0.19.0`
([§FS-check.3.6.2.2](../../functional-spec/FS-check.md#3622-in-a-source-file)). That record's notice shipped with 0.18.0, and a notice already released is
not published again ([§FS-distribution.4.6.4](../../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions)), so the release that ends the warning needs a
notice of its own, and this record carries it.

## 2. Decision

In 0.19.0 (agent-grounds/grund#881) an ungrounded Lisp doc-comment is the error every other
ungrounded doc-comment is ([§FS-check.3.6.2.2](../../functional-spec/FS-check.md#3622-in-a-source-file)), at the severity and with the message they have,
with no clause appended. Under a version 1 config that is `src/router.lisp:3: error: ungrounded
doc-comment: no § citation to a declared ID`, contributing exit `1`. Its code, `ungrounded`,
its site and what makes it a unit do not move.

## 3. Rejected alternatives

- **Keep the warning for another release.** 0.18.0 printed 0.19.0 as the release it becomes
  an error in, and the release guard refuses a version that reaches a promise it does not keep
  ([§FS-distribution.4.2](../../functional-spec/FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)).
- **Keep a clause saying it became an error.** Every other ungrounded doc-comment prints the
  bare message, and the error is that one; a clause on the Lisp line alone would keep two
  wordings of one finding, told apart by the file's language.

## release-note: Release note

- [§DF-lisp-doc-grounding-lands-in-0-19](DF-lisp-doc-grounding-lands-in-0-19.md#df-lisp-doc-grounding-lands-in-0-19-an-ungrounded-lisp-doc-comment-is-the-ordinary-error-from-0190), [§FS-check.3.6.2.2](../../functional-spec/FS-check.md#3622-in-a-source-file), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **an ungrounded Lisp doc-comment is now an error.** What grund 0.18.0 announced lands: under `grounding_level` 2 or higher a Lisp doc-comment that cites and declares nothing printed `src/router.lisp:3: warning: ungrounded doc-comment: no § citation to a declared ID; this warning becomes an error in grund 0.19.0` and now prints `src/router.lisp:3: error: ungrounded doc-comment: no § citation to a declared ID`, as every other ungrounded doc-comment does, and fails `grund check` with exit `1` wherever those do. **Who this affects:** a repository with Lisp sources and `grounding_level` 2 or higher whose `grund check` showed that warning in 0.18.0: cite the spec the documented item realizes in its doc-comment, or declare an ID there. Closes [issue #881](https://github.com/agent-grounds/grund/issues/881).
