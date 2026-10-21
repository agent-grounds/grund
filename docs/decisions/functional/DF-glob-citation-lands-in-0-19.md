# DF-glob-citation-lands-in-0-19: a glob citation is an error from 0.19.0

**Date:** 2026-10-21

## status: Accepted

## 1. Context

[§DF-glob-citation](DF-glob-citation.md#df-glob-citation-a-glob-operator-ends-no-citation-and-the-prefix-it-left-behind-is-removed) made a marked citation written as a pattern stop shortening to the prefix
before its operator, and reported it as the `glob-citation` warning, ending `this warning
becomes an error in grund 0.19.0`, so a repository had one release to cite the point or escape
the illustration ([§FS-check.checks.glob-citation.1](../../functional-spec/FS-check.md#checksglob-citation1-a-warning-before-0190-an-error-in-it)). That record's notice shipped with 0.18.0,
and a notice already released is not published again ([§FS-distribution.4.6.4](../../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions)), so the release
that makes the finding an error needs a notice of its own, and this record carries it.

[§DF-coordinate-globs](DF-coordinate-globs.md#df-coordinate-globs-coordinates-are-selected-by-a-glob-of-their-own-and-a-brace-before-a-separator-opens-a-pattern), unreleased, adds one more pattern in the same release: a brace group
that opens before the section separator, such as `<§>FS-x{.1,.2}`, is no longer recorded as a
citation of `FS-x`. That is a verdict correction under <§>REQ-backwards-compatibility.5, not a
window of its own, and the finding it reports is this one.

## 2. Decision

In 0.19.0 (agent-grounds/grund#881):

- `glob-citation` is an error: text output reads `<path>:<line>: error: <message>`, the finding
  sorts with the other errors, contributes exit `1`, and JSON carries `"severity":"error"`
  ([§FS-check.checks.glob-citation](../../functional-spec/FS-check.md#checksglob-citation-glob-citation));
- the closing clause is `this became an error in grund 0.19.0`; the site, the code, the
  `--only` and `--ignore` selection, and every other byte of the message stay;
- the catalog row of [§FS-errors.5.5](../../functional-spec/FS-errors.md#55-the-check-code-catalog) reads severity `error` with no ramp, and so does what
  `grund config explain` prints for the code;
- the brace form before a separator is reported by this finding, so it is an error from the
  release that first reads it as a pattern.

## 3. Rejected alternatives

- **Keep the warning for another release.** 0.18.0 printed 0.19.0 as the release it becomes
  an error in, and the release guard refuses a version that reaches a promise it does not keep
  ([§FS-distribution.4.2](../../functional-spec/FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)).
- **Give the brace form a warning release of its own.** One code would then carry two
  severities, chosen by which operator the pattern holds. The correction that makes the brace
  form a pattern asks for a located, actionable finding, which this one is, and not for a
  window.

## release-note: Release note

- [§DF-glob-citation-lands-in-0-19](DF-glob-citation-lands-in-0-19.md#df-glob-citation-lands-in-0-19-a-glob-citation-is-an-error-from-0190), [§FS-check.checks.glob-citation](../../functional-spec/FS-check.md#checksglob-citation-glob-citation), [§FS-errors.5.5](../../functional-spec/FS-errors.md#55-the-check-code-catalog), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **a glob citation is now an error.** What grund 0.18.0 announced lands: a marked citation written as a pattern, such as `<§>FS-login.*`, `<§>FS-log*in.requirements` or `<§>FS-[a,b]`, and from this release the brace form `<§>FS-x{.1,.2}` too, is reported as `<path>:<line>: error: glob citation …` and fails `grund check` with exit `1`. Its message now ends `this became an error in grund 0.19.0`, and JSON carries `"severity":"error"`; the code, the location and the rest of the message do not change. **Who this affects:** a repository whose `grund check` showed a `glob-citation` warning in 0.18.0, and one that writes the brace form before a section separator: cite the exact point, or escape the illustration with the configured marker wrapped in angle brackets. Closes [issue #881](https://github.com/agent-grounds/grund/issues/881).
