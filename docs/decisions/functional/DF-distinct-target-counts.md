# DF-distinct-target-counts: an outbound count may count distinct cited declarations, spelled `distinct` after the count

**Date:** 2026-11-16

## status: Accepted

## 1. Context

A project keeps one record per country and wants each country's truck-mass
figures to rest on three different sources. It writes
`The truck-mass chapter of each COUNTRY must cite at least 3 SRC.`, and a
chapter that cites one source three times passes it (agent-grounds/grund#1367).
That is the rule as specified: an outbound count counts every physical citation
site ([§FS-rules.3.2](../../functional-spec/FS-rules.md#32-outbound-citation-count), [§FS-rules.5.1](../../functional-spec/FS-rules.md#51-facts-and-identity)), so three citations of one `SRC` are three.
The sentence is rendered verbatim into the managed agent instructions
([§FS-rules.9](../../functional-spec/FS-rules.md#9-managed-guidance-and-editor-parity)), where it reads as three sources, while `grund check` accepts
one.

No accepted sentence says "three different sources". `distinct` is read as part
of a kind name, so `… must cite at least 3 distinct SRC.` is refused with
`unknown kind "distinct SRC"`. Per-target coverage ([§FS-rules.3.3](../../functional-spec/FS-rules.md#33-per-target-coverage)) looks at
targets one at a time but requires every `SRC` in the project. Counting sites
was a default and never a ruling against counting targets: distinct counting is
not among the deliberate phase-1 absences ([§FS-rules.12](../../functional-spec/FS-rules.md#12-deliberate-phase-1-absences)), and no record weighs
it. The language grows by addition ([§FS-rules.requirements.addition](../../functional-spec/FS-rules.md#requirementsaddition-the-language-grows-by-addition--directional)), and
[§DF-field-value-subjects](DF-field-value-subjects.md#df-field-value-subjects-a-rule-subject-may-select-on-a-scalar-fields-value-with-one-whose-clause) is the precedent for a refusal of this shape becoming an
accepted clause through one record.

## 2. Decision

### 2.1 A fourth spelling of the outbound count

```text
<subject> must|should cite at least|at most|exactly N distinct <KIND> [or <KIND> ...].
```

`distinct` is written once, right after the count, and covers the whole `or`
list. It counts the different declarations that own the targets of the sites the
plain count reads, so `§SRC-x`, `§SRC-x.2` and `§SRC-x.outcome` count once, and
a subject in its own target set counts itself once ([§FS-rules.3.2.1](../../functional-spec/FS-rules.md#321-counting-distinct-cited-declarations),
[§FS-rules.5.2](../../functional-spec/FS-rules.md#52-family-clauses)). A failure is `citation-cardinality` with a row of its own
([§FS-rules.7.3](../../functional-spec/FS-rules.md#73-outbound-citation-cardinality)), and from its first release that row names every site it read
([§FS-rules.7.3.1](../../functional-spec/FS-rules.md#731-a-count-above-zero-names-the-citations-it-counted)). No code is added, so the rule-produced codes stay nine
([§FS-rules.7.6](../../functional-spec/FS-rules.md#76-selection-json-ordering-and-exits)). The managed block renders each authored sentence verbatim, so
its version does not move.

Every sentence accepted today keeps its meaning, which is what
[§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered) requires: `… at least 3 SRC.` still counts sites
and gives the same verdict and bytes, and a project with both rules gets two
findings, because the counting mode is part of the deduplication key
([§FS-rules.6](../../functional-spec/FS-rules.md#6-semantic-deduplication)).

### 2.2 Counted by declaration, not by unit

A named chapter citation such as `§SRC-x.outcome` targets a different rule unit
from `§SRC-x` ([§FS-rules.5.1](../../functional-spec/FS-rules.md#51-facts-and-identity)). Counted by unit, one source cited at two of its
chapters would count as two sources, which is the mistake this form exists to
catch. The count is therefore by owning declaration, the projection per-target
coverage already makes.

### 2.3 The keyword is decided by the rule's scope

`distinct` is a legal kind name in a version-2 config, so in a project with such
a kind `… at least 3 distinct.` may already be an accepted sentence. Where
`distinct` is not a citable kind of the rule's own project, it is the keyword
whenever an object follows the count; where it is one, it is the keyword only
when what follows it is accepted as the object kinds, citable or unverifiable in
the scope ([§FS-rules.4.1](../../functional-spec/FS-rules.md#41-a-rule-this-scope-cannot-verify)). The decision reads the same kind table the object
kinds are judged in, never the scan, so a member run and the workspace root read
a sentence the same way, and `distinct mem/X` gets what `mem/X` gets in the same
scope.

### 2.4 Redundant and meaningless uses are refused, with a form

`at least one distinct X` means what `at least one X` means; `distinct` has no
meaning in per-target coverage or a prohibition; and a `distinct` before a later
object kind restates or contradicts the one after the count. Each is refused
with a form that drops it, which keeps one canonical spelling per meaning
([§DF-chapter-rules.2](DF-chapter-rules.md#2-decision), [§FS-rule-refusals.sentences](../../functional-spec/FS-rule-refusals.md#sentences-strict-refusals)). A `distinct` before a later
object kind that does not also follow the count, and a `distinct` in an inbound
count, are offered no form: either repair counts something the sentence did not
say ([§FS-rule-refusals.sentences.rewrite.same-units](../../functional-spec/FS-rule-refusals.md#sentencesrewritesame-units-a-form-judges-the-units-the-typed-sentence-named-or-none-is-offered)).

### 2.5 The inbound form is left refused

`GOAL-x must be cited by at least 2 distinct TEST.` is the same idea in the other
direction. It is not added: it would double what has to be specified and taught,
and it opens a question this record does not settle, whether a citer's chapters
count as separate citers. Adding it later is purely additive.

### 2.6 Refusals that move are corrected in place

Every sentence that holds `distinct` after a count was refused before this
change, mostly as `unknown kind "distinct X"`, a kind that does not exist. Those
refusals now name the real failure. The new reasons replace the old ones in
place, in 0.23.0, with no window, under the pre-1.0 licence of
[§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise), as a named exception of
[§FS-errors.requirements.prefix-windows](../../functional-spec/FS-errors.md#requirementsprefix-windows-a-wording-change-keeps-the-old-message-whole-until-its-window-closes--realized-directional-for-the-next-change). [§DF-one-answer-rulings-grammar](DF-one-answer-rulings-grammar.md#df-one-answer-rulings-grammar-one-grammar-keeps-the-documented-refusal-bytes-and-order-and-one-kind-universe-keeps-each-commands-vocabulary) D10 gives a
refusal's wording a warning release; this change departs from it by the same
licence [§DF-presence-or-refusal](DF-presence-or-refusal.md#df-presence-or-refusal-a-presence-name-holding-or-between-two-words-is-refused-for-the-or-corrected-in-place) used, for the same reason.

- A window would print `unknown kind "distinct X"` on purpose, naming as
  unknown a kind nobody configured and hiding the failure the sentence has.
- Every sentence whose refusal moves was refused before, so nothing that worked
  depended on the old text. A configured rule written as `… distinct X` that
  failed as `invalid-rule` is now evaluated, which is the point of the change.
- A consumer of the `invalid-rule` finding keeps its code, path, line, severity,
  authority, the exit code and selection by `--only invalid-rule`; only the text
  after `is not a valid rule: ` moves. `check --rule` keeps its exit `2`.

If this misses 0.23.0, the change takes a window instead: each moved refusal
keeps `unknown kind "distinct X"` as a verbatim prefix for one release.

## 3. Rejected alternatives

| Approach | Why rejected |
|---|---|
| A project or per-kind switch that makes `at least 3 SRC` count distinct targets | It moves the verdict of an accepted sentence ([§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered)), and one sentence would mean different things in different repositories. |
| Per-target coverage, `must cite each SRC at least once` | It requires every `SRC` in the project, so it fails as soon as a second country cites other sources. It does not answer the question. |
| Checking distinctness outside grund | Each consumer repeats the check, and the rule sentence the agent instructions render still says something the check does not. |
| Counting distinct target units | One source cited at two of its chapters would count as two sources (2.2). |
| Deciding the keyword by whether a kind named `distinct` exists locally | A member run and the workspace root could then read one sentence two ways (2.3). |
| A window for the moved refusals | It would print a false reason on purpose for a release (2.6). |

## 4. Consequences

- A project can state "three independent sources" or "two different tests" as a
  rule `grund check` enforces and agents read verbatim.
- `at least 3 SRC` and `at least 3 distinct SRC` are two constraints; written
  together, they give two findings. A `[citations]` direction counts sites and
  never absorbs a distinct rule.
- Exit codes, finding codes and every accepted sentence's verdict do not move.
  The refusal text of the sentences below moves in place.

## release-note: Release note

- [§FS-rules.3.2.1](../../functional-spec/FS-rules.md#321-counting-distinct-cited-declarations): **an outbound count may count distinct cited
  declarations, and refusals of a sentence with `distinct` after its count
  name the real failure.** `<subject> must cite at least 3 distinct SRC.`
  counts the different `SRC` declarations the subject cites, so citing one
  source three times, or at its sections and chapters, counts once. A failure
  is `citation-cardinality`:
  `COUNTRY-001-rs.truck-mass cites 1 distinct SRC; RULE-001-country-truck-mass requires at least 3; add or remove those citations until the count fits; counted at …`.
  `… at least 3 SRC.` still counts citation sites, with the same verdict and
  bytes. In any sentence with `distinct` after the count, a refusal now names
  what the same sentence without `distinct` is refused for, rather than
  `unknown kind "distinct X"` ([§FS-rule-refusals.sentences](../../functional-spec/FS-rule-refusals.md#sentences-strict-refusals)): a configured
  `RULE-001-country-truck-mass: The truck-mass chapter of each COUNTRY must cite at least 3 distinct SRC or DOCX.`
  is reported as
  `RULE-001-country-truck-mass is not a valid rule: unknown kind "DOCX"`,
  no longer as
  `RULE-001-country-truck-mass is not a valid rule: unknown kind "distinct SRC"`,
  and `check --rule` prints `error: unknown kind "DOCX"` where it printed
  `error: unknown kind "distinct SRC"`, each followed by its `known kinds:`
  line; a non-citable kind or an unknown alias moves the same way. With kinds
  `FS`, `GOAL` and `REQ`, these refusals move on both surfaces, each from what
  it printed before to what it prints now:
  `Each FS must cite at least 3 distinct GOAL.` from `unknown kind "distinct GOAL"` to accepted;
  `Each FS must cite at least 3 distinct mem/GOAL.` from `` citation target `distinct mem/GOAL`: invalid namespace qualifier segment `distinct mem` in `distinct mem` before kind `GOAL` (expected [a-z][a-z0-9-]*, one segment per workspace level) `` to what `mem/GOAL` gets in the same scope;
  `Each FS must cite at least one distinct GOAL.` from `unknown kind "distinct GOAL"` to `"distinct" is redundant after "at least one"; accepted form: Each FS must cite at least one GOAL.`;
  `Each FS must cite each distinct GOAL at least once.` from `unknown kind "distinct GOAL"` to `"distinct" has no meaning per target; accepted form: Each FS must cite each GOAL at least once.`;
  `Each FS must not cite any distinct GOAL.` from `unknown kind "distinct GOAL"` to `"distinct" has no meaning in a prohibition; accepted form: Each FS must not cite any GOAL.`;
  `Each FS must cite at least 3 distinct GOAL or distinct REQ.` from `unknown kind "distinct GOAL"` to `"distinct" must follow the count; accepted form: Each FS must cite at least 3 distinct GOAL or REQ.`;
  `Each FS must cite at least 3 GOAL or distinct REQ.` from `unknown kind "distinct REQ"` to `"distinct" must follow the count`, with no form and no `known kinds:`;
  `Each FS must be cited by at least 2 distinct GOAL.` from `unknown kind "distinct GOAL"` to `"distinct" counts only outbound citations`, with no form and no `known kinds:`;
  `Each FS must cite at least 3 distinct POLICY.` from `unknown kind "distinct POLICY"` to `unknown kind "POLICY"`;
  `Each FS must cite at least 1 distinct GOAL.` from `numeric "at least 1" is not canonical`, with no form, to `numeric "at least 1" is not canonical; accepted form: Each FS must cite at least one GOAL.`;
  `Each FS must cite exactly 1 distinct GOAL.` from `numeric "exactly 1" is not canonical`, with no form, to `numeric "exactly 1" is not canonical; accepted form: Each FS must cite exactly one distinct GOAL.`;
  and `Each FS must not cite at least 2 distinct GOAL.` from `a prohibition must use "cite any"`, with no form, to `a prohibition must use "cite any"; accepted form: Each FS must not cite any GOAL.`.
  Each of them printed `known kinds:` after it on `check --rule` before; one
  that offers a form now prints nothing after it. **Who this breaks:** a
  script matching, for such a sentence, the exact stderr of `check --rule`,
  the exact `message` of an `invalid-rule` finding, or the exact failure
  message of the Python `check(rule=…)` or Node `check(root, { rule })`; and a
  configured rule spelled `… distinct X` that failed as `invalid-rule`, which
  is now evaluated. Exit codes of refused sentences, the finding code and
  location, and every accepted sentence's verdict do not move. Closes
  [issue #1367](https://github.com/agent-grounds/grund/issues/1367).
