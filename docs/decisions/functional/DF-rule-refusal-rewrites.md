# DF-rule-refusal-rewrites: a refused rule's accepted form is the typed sentence with the failed part replaced, in place

**Status:** Accepted
**Date:** 2026-10-09

## 1. Context

A refused rule sentence ends in `accepted form:` and a sentence to paste back.
That sentence was a fixed string for nearly every refusal, chosen for the
example repository of [§FS-rules.3.5](../../functional-spec/FS-rules.md#35-strict-refusals): `Each FS must cite at least one GOAL.`,
`FS-login.requirements must cite at least one REQ.`,
`Each FS must have exactly one requirements chapter.`,
`AR-overview.system-overview must cite each AR exactly 2 times.` and a few
more. It was not built from the sentence the author typed, nor from the kinds
the repository configures (agent-grounds/grund#513).

In a repository that configures only `FS`, every one of those suggestions is
refused when it is pasted back. For an unknown object kind or a non-canonical
count, the suggestion is refused with the same suggestion, so an agent that
follows it loops until it gives up. The suggestion also drops the kinds the
author typed, and where it names a chapter it names `requirements` whether or
not the repository has one. `FS.* must cite at least one REQ.` was answered
with `FS.requirements must cite at least one REQ.`, which no repository
accepts, because `KIND.NAME` is a selector spelling and not a rule subject.

[§DF-selector-refusal-rewrites.3](DF-selector-refusal-rewrites.md#3-consequences) reserved this change: rule rewrites still
name kinds a repository may not configure, and fixing them changes documented
rows of [§FS-rules.3.5](../../functional-spec/FS-rules.md#35-strict-refusals), so it is a separate decision.
[§DF-rule-after-enabling-rewrites.3](DF-rule-after-enabling-rewrites.md#3-consequences) left the fixed predicate of its own
suggestions open for the same reason. These lines are message text on both
rule surfaces, `check --rule` and a configured rule declaration's
`invalid-rule` finding, which tools may match ([§FS-errors.3](../../functional-spec/FS-errors.md#3-message-text),
[§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered)). So changing them is a decision rather than a
fix.

## 2. Decision

Both rule surfaces build every accepted form from the sentence the author
typed. They replace only the production that failed, supply only what can be
recovered, parse the form again before offering it, and offer no form where
nothing can be supplied ([§FS-rules.3.5.4](../../functional-spec/FS-rules.md#354-an-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced)). Where no form is offered,
`check --rule` lists the known kinds when what is missing is a kind, and the
finding ends at the reason. The new text replaces the old in place, in one
release, under the pre-release licence of
[§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise).

- Every byte that changes belongs to a suggestion that was refused when
  followed in a repository that does not configure what it names, or that
  named a chapter, a declaration or a kind the author did not write. Where the
  fixed text already was the rewrite of the typed sentence, it is kept, so ten
  documented rows of [§FS-rules.3.5](../../functional-spec/FS-rules.md#35-strict-refusals) print what they printed.
- Five documented rows move. The path subject and `Each POLICY` offer no form,
  because the sentence does not say which kind belongs there. `FS-login.*`,
  `FS-login.2` and `Each chapter of each FS` are answered with the subject
  `list --selector` already gives them, `FS-login` or `Each FS`, rather than
  with a `requirements` chapter they never named. The
  [§FS-rules.3.5.1](../../functional-spec/FS-rules.md#351-presence-name-whitespace-refusal) refusal keeps its reason as a verbatim prefix and its
  explanation as the end of the message, and the example between them becomes
  the typed sentence with the `NAME` trimmed, or nothing.
- Three rows of [§FS-rules.3.5.3](../../functional-spec/FS-rules.md#353-a-chapter-path-is-refused-for-the-component-that-failed)'s table move with them, for the same
  reason. `The requirements.1 chapter of each FS` and
  `The requirements.* chapter of each FS` are answered with
  `The requirements chapter of each FS` rather than with `FS-login`, a
  declaration they never named. `FS-login..requirements` is answered with
  `FS-login`, the selector `list --selector` gives it, rather than with
  `FS-login.requirements`: an empty component keeps what was typed before it,
  and a name after it is not one the parser read as a chapter.
- A consumer of the `invalid-rule` finding keeps its code, its path, line,
  severity and authority, the exit code, selection by `--only invalid-rule`,
  and the text through the reason. Only what follows the reason changes.
- `check --rule` has no code. A consumer of it keys on the exit code and on the
  sentence it passed, and this change moves neither.

This is the separate decision [§DF-selector-refusal-rewrites.3](DF-selector-refusal-rewrites.md#3-consequences) reserved.
Neither that record nor [§DF-rule-after-enabling-rewrites](DF-rule-after-enabling-rewrites.md#df-rule-after-enabling-rewrites-a-rule-subject-that-needs-named-sections-is-answered-with-one-they-make-valid-in-place) is edited.

## 3. Consequences

- A consumer matching a refused rule's exact line reads a new line wherever the
  old suggestion named something the typed sentence did not. One matching
  through the reason keeps matching.
- A tool that expects every rule refusal to contain `accepted form` finds
  refusals without one: the path subject, an unknown or malformed object kind,
  a subject no configured kind is recovered from, no modality, an unknown verb,
  and a count that is not accepted or is not a numeral. Where the missing piece
  is a kind, `check --rule` prints a second line, `known kinds: …`.
- The Python `check(rule=…)` and Node `check(root, { rule })` failure message is
  the stderr of `check --rule`, so it changes with it.
- Exit codes, stdout, finding codes, locations and the set of accepted
  sentences do not move, and no repository that passed starts failing, because
  every line this touches is already a failure.
- The reasons, the text before the `;`, do not move. Those that named the
  wrong failure were corrected separately, by
  [§DF-rule-refusal-reasons](DF-rule-refusal-reasons.md#df-rule-refusal-reasons-a-chapter-path-is-refused-for-the-component-that-failed-corrected-in-place).

## 4. Alternatives considered

| Approach | Why rejected |
|---|---|
| Supply a chapter name, such as the tables' `requirements` | Presence rules compare display names, so in a repository whose `requirements` chapter has another title, `Each FS must have exactly one requirements chapter.` is accepted and then reports zero such chapters. A suggestion that fails at the scan is the same defect one step later. |
| Fall back to the first configured kind when no subject can be recovered | It is a guess that names a kind the author did not mean; [§DF-rule-after-enabling-rewrites.4](DF-rule-after-enabling-rewrites.md#4-alternatives-considered) rejected the same fallback, and `list --selector 'Each POLICY'` answers with `known kinds:` instead. |
| Fix only the first failure, and offer the result even when it is refused | The `accepted form` label would still promise a sentence that is refused, which is the defect. |
| Append the rebuilt sentence after the released line, the [§FS-errors.3](../../functional-spec/FS-errors.md#3-message-text) route | The sentence that fails when followed would still come first; [§DF-selector-refusal-rewrites.4](DF-selector-refusal-rewrites.md#4-alternatives-considered) rejected the same route for the same reason. |
| Migrate over three releases, as [§FS-errors.3.7](../../functional-spec/FS-errors.md#37-the-rule-site-unknown-alias-wording-migration) did | A window lets exact-line consumers move to a stable code. The finding's code already exists and does not move, so a window gives them nothing new, and `check --rule` has no code at all. |
| Change `check --rule` only and keep the finding's bytes | One function renders both surfaces. Keeping the finding would need a second renderer whose only job is to print the sentence that fails, on the surface where following it costs a commit. |

## release-note: Release note

- [§FS-rules.3.5.4](../../functional-spec/FS-rules.md#354-an-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced): **a refused rule's `accepted form:` is the sentence you typed
  with the refused part replaced, and it is accepted when you paste it back.**
  `grund check --rule` and a configured rule declaration's `invalid-rule`
  finding no longer suggest a fixed sentence about `FS`, `GOAL`, `REQ` or `AR`
  whatever the repository configures. `FS-*.requirements must cite at least one
  FS.` is answered with `The requirements chapter of each FS must cite at least
  one FS.`, and `Each FS should cite no FS.` with `Each FS should not cite any
  FS.`. Where grund cannot know what belongs in the gap, such as a kind the
  repository does not configure or a path subject, it offers no form:
  `check --rule` prints the reason and then `known kinds: …`, and the finding
  ends at the reason. Five documented refusals change: the path subject and
  `Each POLICY` now offer no form, and `FS-login.*`, `FS-login.2` and
  `Each chapter of each FS` are answered with `FS-login` or `Each FS`, and a
  chapter path with a numbered, wildcard or empty component keeps the chapter
  names typed before it. The
  whitespace refusal for a presence `NAME` offers the sentence with the `NAME`
  trimmed, or nothing. **Who this breaks:** a script matching the exact stderr
  of `check --rule`, the exact `message` of an `invalid-rule` finding, or the
  exact failure message of the Python `check(rule=…)` or Node
  `check(root, { rule })`, for a refusal whose form changes or goes away, and a
  tool that expects every rule refusal to contain `accepted form`. Exit codes,
  stdout, the finding code and location, the reasons and the set of accepted
  sentences do not move. Closes [issue #513](https://github.com/agent-grounds/grund/issues/513).
