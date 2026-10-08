# DF-rule-after-enabling-rewrites: a rule subject that needs named sections is answered with one they make valid, in place

**Status:** Accepted
**Date:** 2026-10-08

## 1. Context

With `[id] named_sections = false`, a rule sentence whose subject names a
chapter is refused with `named chapter subjects require [id] named_sections =
true`, and the refusal ended in `accepted form after enabling it:` followed by
the subject exactly as typed. For many subjects enabling named sections does not
make that sentence valid. In a repository whose kinds are `FS`, `REQ` and
`GOAL`, `FS-*.requirements must cite at least one REQ.` was answered with
itself, and once the switch was flipped the same sentence was refused again, for
its ID grammar, with a different subject and predicate
(agent-grounds/grund#520). `POLICY.requirements` names no configured kind at
all and was offered back the same way.

The suggestion tells the reader to edit `grund.toml` and paste the sentence
back, so the round trip changes a project's configuration and then gives up the
rule its author meant to write. The selector side already answers the same
subjects with a selector its own parse accepts once named sections are on
([§FS-rules.8.1](../../functional-spec/FS-rules.md#81-a-refused-selector-is-answered-with-a-selector)). The rule side was left byte-for-byte as it was by
[§DF-selector-refusal-rewrites.2](DF-selector-refusal-rewrites.md#2-decision), so the two surfaces gave different answers about the same
subject.

These lines are message text on both rule surfaces, `check --rule` and a
configured rule declaration's `invalid-rule` finding, which tools may match
([§FS-errors.3](../../functional-spec/FS-errors.md#3-message-text), [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered)). The finding carries a code. So
changing them is a decision rather than a fix.

## 2. Decision

Both rule surfaces answer a subject refused for needing named sections with a
subject that turning them on makes valid, built by the four steps the selector
side uses and written as a rule subject, labelled `after enabling it` only where
the suggestion itself needs the switch, and with nothing guessed where no kind
is recovered ([§FS-rules.3.5.2](../../functional-spec/FS-rules.md#352-a-subject-that-needs-named-sections-is-answered-with-one-they-make-valid)). The new text replaces the old in place, in one
release, under the pre-release licence of
[§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise).

- Every byte that changes belongs to a suggestion that is refused again when it
  is followed. Where the as-typed suggestion worked, it is kept, so the one
  documented row, `FS-login.requirements`, and every other row of
  [§FS-rules.3.5](../../functional-spec/FS-rules.md#35-strict-refusals) print what they printed. Nothing that worked can have
  depended on the old text.
- A consumer of the `invalid-rule` finding keeps its code, its path, line,
  severity and authority, the exit code, selection by `--only invalid-rule`,
  and the text through the reason. Only what follows the reason changes: the
  subject after the label, the label itself on some rows, and the whole tail
  where no kind is recovered.
- `check --rule` has no code. A consumer of it keys on the exit code and on the
  sentence it passed, and this change moves neither.

This narrows what [§DF-selector-refusal-rewrites.2](DF-selector-refusal-rewrites.md#2-decision) said of the rule side,
that `check --rule` and configured rule declarations keep every byte, to every
byte but these. That record is not edited, and this is not the separate
decision its consequences reserve for changing documented rows of
[§FS-rules.3.5](../../functional-spec/FS-rules.md#35-strict-refusals): no documented row changes.

## 3. Consequences

- A consumer matching a refused rule's exact line, for a named-chapter subject
  whose as-typed suggestion was refused once named sections were on, reads a
  new line. One matching through the reason keeps matching.
- Where no kind is recovered, `check --rule` gains a `known kinds:` line after
  the reason, and the finding ends at the reason.
- Exit codes, stdout, finding codes, locations and the set of accepted
  sentences do not move, and no repository that passed starts failing.
- The predicate of every such suggestion stays the fixed
  `must cite at least one REQ.`, which names a kind a repository may not
  configure (agent-grounds/grund#513). That is the predicate's own defect and
  is not decided here.
- Named-sections-off subjects refused for their own production first, such as
  `FS-login.*`, keep their documented rows.

## 4. Alternatives considered

| Approach | Why rejected |
|---|---|
| Append the rebuilt sentence after the released line, the [§FS-errors.3](../../functional-spec/FS-errors.md#3-message-text) route | The sentence that fails when followed would still come first, which is what agent-grounds/grund#520 reports; [§DF-selector-refusal-rewrites.4](DF-selector-refusal-rewrites.md#4-alternatives-considered) rejected the same route for the same reason. |
| Migrate over three releases, as [§FS-errors.3.7](../../functional-spec/FS-errors.md#37-the-rule-site-unknown-alias-wording-migration) did | A window lets exact-line consumers move to a stable code. The finding's code already exists and does not move, so a window gives them nothing new, and it would print the failing sentence for two more releases. `check --rule` has no code at all. |
| Change `check --rule` only and keep the finding's bytes | One function renders both surfaces. Keeping the finding would need a second renderer whose only job is to print the sentence that fails, on the surface where following it costs a commit. |
| Suggest what the rule side suggests once named sections are on, [§FS-rules.8.1](../../functional-spec/FS-rules.md#81-a-refused-selector-is-answered-with-a-selector) taken literally | That suggestion is the fixed template `FS-login must cite at least one GOAL.`, which is agent-grounds/grund#513's defect. |
| Fall back to `Each FS` where no kind is recovered | It is a guess that names a kind the repository may not have. |

## release-note: Release note

- [§FS-rules.3.5.2](../../functional-spec/FS-rules.md#352-a-subject-that-needs-named-sections-is-answered-with-one-they-make-valid): **with named sections off, a rule subject that needs them
  is answered with one they make valid.** `grund check --rule` and a configured
  rule declaration's `invalid-rule` finding no longer suggest the subject as
  typed after `accepted form after enabling it:` when enabling named sections
  would refuse it again. `FS-*.requirements` and `FS.requirements` are answered
  with `The requirements chapter of each FS`, and `FS-login.Requirements` with
  `FS-login` under a plain `accepted form:`, because the repository accepts that
  as configured. Where no configured kind can be recovered, as with
  `POLICY.requirements`, nothing is suggested: `check --rule` prints the reason
  and then `known kinds: …`, and the finding ends at the reason. **Who this
  breaks:** a script matching the exact stderr of `check --rule`, or the exact
  `message` of an `invalid-rule` finding, for such a subject. Exit codes,
  stdout, the finding code and location, and every documented refusal row do
  not move. Closes [issue #520](https://github.com/agent-grounds/grund/issues/520).
