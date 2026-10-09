# DF-rule-refusal-reasons: a chapter path is refused for the component that failed, corrected in place

**Status:** Accepted
**Date:** 2026-10-09

## 1. Context

A rule's chapter subject can be refused for its chapter path, and two of those
refusals named a failure the sentence did not have (agent-grounds/grund#507).
In a repository whose kinds are `FS` and `REQ`, with named sections on:

- `The requirements.1 chapter of each FS must cite at least one REQ.` was
  refused with `named chapter subject "The requirements.1 chapter of each FS"
  does not match the configured section grammar`. `requirements.1` is
  grammatical. What is wrong is that it is numbered, and the literal spelling of
  the same chapter, `FS-login.requirements.1`, already said so. The wildcard
  `The requirements.* chapter of each FS` was called a grammar mismatch the same
  way.
- `FS-login.requirements.` and `FS-login..requirements` were refused with
  `numbered chapter subjects can detach when headings move`. Neither names a
  section number. A trailing or doubled separator leaves an empty component,
  and the literal spelling counted an empty component as all digits.

The parser had already told the first case apart, and the rule side threw
that away. `list --selector` reads the same parse and was corrected for the
same subject by [§DF-selector-refusal-rewrites](DF-selector-refusal-rewrites.md#df-selector-refusal-rewrites-a-refused-selector-is-answered-with-a-selector-and-its-old-lines-are-replaced-not-appended-to). [§FS-rules.8.1](../../functional-spec/FS-rules.md#81-a-refused-selector-is-answered-with-a-selector) says a
rule and a selector never disagree about what failed, but here they did. That
record left the rule side byte for byte, and its consequences named this
defect as a separate decision ([§DF-selector-refusal-rewrites.3](DF-selector-refusal-rewrites.md#3-consequences)). This is
that decision. The second defect it named there, accepted forms that name kinds
a repository may not configure, is agent-grounds/grund#513's and is not decided
here.

The reasons are message text on both rule surfaces, `check --rule` and a
configured rule declaration's `invalid-rule` finding, and tools may match it
([§FS-errors.3](../../functional-spec/FS-errors.md#3-message-text), [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered)). So changing it is a decision
rather than a fix.

## 2. Decision

A chapter subject refused for its chapter path opens with the reason for the
component that failed, whichever spelling reached it ([§FS-rules.3.5.3](../../functional-spec/FS-rules.md#353-a-chapter-path-is-refused-for-the-component-that-failed)):

- an all-digit component gets the numbered reason;
- a component holding `*` gets the wildcard reason;
- an empty component gets the section-grammar reason.

Only the reason moves. Every byte from `; ` on is what the same sentence printed
before. The new text replaces the old in place, in one release, under the
pre-release licence of [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise).

- Every reason that changes named a failure its sentence did not have. Those
  sentences were refused before and stay refused, so nothing that worked can
  have depended on the old reason.
- A consumer of the `invalid-rule` finding keeps its code, its path, line,
  severity and authority, the exit code, selection by `--only invalid-rule`, and
  every byte after the reason.
- `check --rule` has no code. A consumer of it keys on the exit code and on the
  sentence it passed, and this change moves neither.
- No verdict moves: the accepted and refused sentences, the exit codes and the
  finding code stay as they are. So this is not a verdict correction, and it
  needs none of that route's proof.

[§DF-selector-refusal-rewrites.3](DF-selector-refusal-rewrites.md#3-consequences) expected this fix to change documented rows of
[§FS-rules.3.5](../../functional-spec/FS-rules.md#35-strict-refusals). It changes none. No documented row of [§FS-rules.3.5](../../functional-spec/FS-rules.md#35-strict-refusals) or
[§FS-rules.3.5.2](../../functional-spec/FS-rules.md#352-a-subject-that-needs-named-sections-is-answered-with-one-they-make-valid) spells a numbered or wildcard component through
`The PATH chapter of each KIND` with named sections on, and none spells an
empty component. This narrows again what [§DF-selector-refusal-rewrites.2](DF-selector-refusal-rewrites.md#2-decision)
said of the rule side, as [§DF-rule-after-enabling-rewrites.2](DF-rule-after-enabling-rewrites.md#2-decision) did. Neither
record is edited.

## 3. Consequences

- A script that matches the reason or the exact line of a refused rule reads a
  new reason, for a `The PATH chapter of each KIND` subject with a numbered or
  wildcard component, and for a literal subject with an empty component. One
  that matches after `; ` keeps matching.
- `list --selector` reads the same parse. A selector with an empty component,
  such as `FS-login.requirements.`, is refused for the section grammar rather
  than as numbered, and loses the `hint:` line that only the numbered, wildcard
  and quantified reasons earn. Its accepted selector does not move.
- With named sections off, a literal subject with an empty component gets the
  section-grammar reason rather than the numbered one. That spelling judges its
  path before it asks about named sections, as it always did.
- Exit codes, stdout, finding codes, locations and the set of accepted sentences
  do not move. No repository that passed starts failing.
- Two neighbouring defects are left alone. A literal path whose component is
  off-grammar in some other way, such as `FS-login.Requirements`, is still
  refused for its ID grammar. The accepted forms after these reasons still name
  `FS-login` and `REQ` whatever the repository configures (agent-grounds/grund#513).

## 4. Alternatives considered

| Approach | Why rejected |
|---|---|
| Append the true reason after the released line, the [§FS-errors.3](../../functional-spec/FS-errors.md#3-message-text) route | The false reason would still come first, which is what agent-grounds/grund#507 reports. [§DF-selector-refusal-rewrites.4](DF-selector-refusal-rewrites.md#4-alternatives-considered) and [§DF-rule-after-enabling-rewrites.4](DF-rule-after-enabling-rewrites.md#4-alternatives-considered) rejected the same route for the same reason. |
| Migrate over three releases, as [§FS-errors.3.7](../../functional-spec/FS-errors.md#37-the-rule-site-unknown-alias-wording-migration) did | A window lets exact-line consumers move to a stable code. The finding's code already exists and does not move, and `check --rule` has none, so a window gives them nothing new. It would also print the false reason for two more releases. |
| Correct `check --rule` only and keep the finding's bytes | One function renders both surfaces. Keeping the finding would need a second renderer whose only job is to print a false reason, on the surface a repository's `grund check` fails on. |
| Give an empty component a reason of its own | That is a new production name on three surfaces. The section-grammar reason is already true of it, and it is what `FS.requirements.` already gets as a selector. |
| Refuse a literal empty component for its ID grammar, as an uppercase one is | The path is what is wrong, not the ID. The literal spelling would then disagree with `The PATH chapter of each KIND` and `KIND.PATH` about the same path. |
| Correct the accepted forms in the same change | Those are agent-grounds/grund#513's change, approved on its own. Coupling them would hold a reason fix behind a redesign of every tail. |

## release-note: Release note

- [§FS-rules.3.5.3](../../functional-spec/FS-rules.md#353-a-chapter-path-is-refused-for-the-component-that-failed): **a refused rule's chapter subject names the component of its
  path that failed.** `grund check --rule` and a configured rule declaration's
  `invalid-rule` finding now refuse `The requirements.1 chapter of each FS …`
  with `numbered chapter subjects can detach when headings move`, as they
  already refused `FS-login.requirements.1 …`. They refuse
  `The requirements.* chapter of each FS …` for its wildcard. Neither is called
  a section-grammar mismatch any more. A path with an empty component, such as
  `FS-login.requirements.` or `FS-login..requirements`, is refused as not
  matching the configured section grammar rather than as numbered. That holds
  on the rule surfaces and on `grund list --selector`, which also drops its
  `hint:` line for such a path. **Who this breaks:** a script matching, for
  such a subject, the exact stderr of `check --rule`, the exact `message` of an
  `invalid-rule` finding, the exact failure message of the Python
  `check(rule=…)` or Node `check(root, { rule })`, or a refused selector's exact
  stderr. Everything after the reason, exit codes, stdout, the finding code
  and location, and every documented refusal row do not move. Closes
  [issue #507](https://github.com/agent-grounds/grund/issues/507).
