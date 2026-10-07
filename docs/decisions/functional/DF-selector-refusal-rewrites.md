# DF-selector-refusal-rewrites: a refused selector is answered with a selector, and its old lines are replaced, not appended to

**Status:** Accepted
**Date:** 2026-10-07

## 1. Context

`grund list --selector` reads a rule subject with the parser rule sentences use
([§FS-rules.2](../../functional-spec/FS-rules.md#2-subject-selectors)), and it printed that parser's refusals unchanged. Each ended in
`accepted form:` and a whole rule sentence taken from a fixed template rather
than from the query or the configuration. In a repository whose only kind is
`FS`, `FS-*.requirements` was answered with
`accepted form: FS-login must cite at least one GOAL.`, which names a kind the
repository does not have and which `--selector` refuses again when pasted back.
Three refusals also named the wrong failure. `FS.requirements.1` and
`The requirements.1 chapter of each FS` were called section-grammar mismatches
although they are grammatical numbered paths, `FS.*` was called the same
although it is a component wildcard, and `Each chapter of each FS` was called an
unknown kind (agent-grounds/grund#505).

Those lines are message text, which tools may match
([§FS-errors.3](../../functional-spec/FS-errors.md#3-message-text), [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered)), so changing them is a decision
rather than a fix.

## 2. Decision

On the `--selector` surface the refusals are replaced now. A reason that was
true keeps its released text, and the three that were not are corrected. The
`accepted form:` tail becomes one `accepted selector:` built from what was typed
and from the configured kinds, or a second `known kinds:` line where no kind can
be recovered ([§FS-rules.8.1](../../functional-spec/FS-rules.md#81-a-refused-selector-is-answered-with-a-selector)). `check --rule` and configured rule declarations keep
every byte, including every row of [§FS-rules.3.5](../../functional-spec/FS-rules.md#35-strict-refusals).

The replacement is argued under the pre-release licence of
[§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise): carrying the old line would cost more than the
change.

- The old tail could never be pasted back into `--selector`, so nothing that
  worked depended on it.
- These refusals carry no `code`. They are bare `error:` lines on stderr at exit
  `2`, even under `--format json`. As [§FS-errors.3.7](../../functional-spec/FS-errors.md#37-the-rule-site-unknown-alias-wording-migration) says of `check --rule`'s
  matching pre-scan refusal, a consumer of them keys on the exit code and on the
  input it passed, and this change moves neither.

One parse still decides what failed for both surfaces, and only the rendering
of that decision differs, so the selector side cannot drift from the rule side
on the reason.

## 3. Consequences

- A consumer matching a refused selector's exact line reads a new line. One
  matching on the reason keeps matching, except on the three corrected reasons
  and on a pasted rule sentence.
- Some refusals gain a second line: `hint:` after a numbered, wildcard or
  chapter-quantified refusal, and `known kinds:` where no kind is recovered.
- Exit codes, stdout, and the set of accepted selectors do not move.
- The rule side keeps two defects of its own. The rule sentence
  `The requirements.1 chapter of each FS must cite at least one REQ.` is still
  called a section-grammar mismatch, and rule rewrites still name kinds a
  repository may not configure. Fixing either changes documented rows of
  [§FS-rules.3.5](../../functional-spec/FS-rules.md#35-strict-refusals), so it is a separate decision.

## 4. Alternatives considered

| Approach | Why rejected |
|---|---|
| Append the selector after the released line, the [§FS-rules.3.5.1](../../functional-spec/FS-rules.md#351-presence-name-whitespace-refusal) route | The false reason and the rule sentence that fails again would still come first, so the reader would get two contradictory suggestions. It does not answer the report. |
| Migrate over three releases, as [§FS-errors.3.7](../../functional-spec/FS-errors.md#37-the-rule-site-unknown-alias-wording-migration) did | A window lets exact-line consumers move to a stable `code`. These refusals have none, so nobody has anything to migrate to, and the misleading text would print for two more releases. |
| Classify selector failures separately from rule subjects | Two parsers each deciding what failed is how the `KIND.NAME` branch came to skip the numbered check. |

## release-note: Release note

- [§FS-rules.8.1](../../functional-spec/FS-rules.md#81-a-refused-selector-is-answered-with-a-selector): **a refused `grund list --selector` is answered with a selector,
  not a rule sentence.** Every refused selector's `accepted form: <rule
  sentence>` tail becomes `accepted selector: <selector>`, built from what was
  typed and the configured kinds, or a second line `known kinds: …` where no
  kind can be recovered. `FS.requirements.1` and
  `The requirements.1 chapter of each FS` are refused for being numbered, `FS.*`
  for its wildcard and `Each chapter of each FS` for its quantifier, and each of
  those adds a `hint: grund show --batch --toc …` line. A pasted rule sentence is
  answered with its subject. **Who this breaks:** a script matching a refused
  selector's exact stderr, or its text after `accepted form:`. Exit codes,
  stdout and the accepted selectors do not move, and `check --rule` prints
  every byte it did. (agent-grounds/grund#505)
