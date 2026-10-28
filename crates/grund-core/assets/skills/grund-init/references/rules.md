<!-- BEGIN chapter-rules -->
### Chapter rules

Write one controlled-English sentence as a rule declaration title and put the
reason in its non-empty body. Kind names and fixed words are case-sensitive;
every sentence ends in one `.`. `must` and `must not` produce errors. `should`
and `should not` produce suggestions, visible with `grund check --suggestions`
and never changing the exit status.

Subjects select local units only:

- `Each FS` selects every FS declaration.
- `FS-login` selects one declaration.
- `The requirements chapter of each FS` selects that named chapter in every FS.
- `FS-login.requirements` selects one exact named chapter.
- `The front-matter.sources chapter of each METRIC` selects one key of a kind's
  [front-matter field](https://github.com/agent-grounds/grund/blob/main/docs/user-facing/fields.md#front-matter-is-a-field).
- `Each SWR whose verification is Test` selects every SWR whose scalar field
  `verification` holds `Test`.
- `The evidence chapter of each SWR whose verification is Test` selects that
  named chapter in those SWR alone.

Named chapter subjects select handles: `The goal chapter of each FS` reaches `## goal: Goal and hypothesis` by `goal` ([chapter subjects](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-rules.md#2-subject-selectors)). Presence rules compare direct chapter display names case-insensitively: `Each FS must have exactly one goal chapter.` counts zero for that heading and one for `## goal: Goal` ([presence matching](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-rules.md#31-chapter-presence)). The finding appends the expected display name and observed pairs such as `"goal: Goal and hypothesis"`, or `none` ([comparison context](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-rules.md#721-display-name-comparison-context)). Presence `NAME` must be one non-empty token: `Goal and hypothesis` is refused with `NAME forbids whitespace anywhere.` appended to the existing guidance; authored display titles may still contain spaces ([whitespace refusal](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-rules.md#351-presence-name-whitespace-refusal)).

Named-chapter subjects require `[id] named_sections = true` in version 1;
version 2 always reads named sections. Phase 1 has no paths, files, folders,
wildcards, subject namespaces, numbered-chapter subjects, exceptions,
definitions, derived terms, settings, or source-code symbols.

<!-- BEGIN chapter-rules-emptiness -->
A quantified chapter subject selects only the chapters that exist, and a
declaration of the kind that has none is reported rather than passed over, so a
chapter-scoped citation rule reaches every declaration of its kind.

`The requirements chapter of each FS must cite at least one REQ.` reports an FS
whose `requirements` chapter cites no REQ, and reports an FS with no
`requirements` chapter at all — the edit the rule could not see before:

```text
docs/fs/FS-login.md:1: error: FS-login has no requirements chapter, so RULE-requirements cannot reach it; add the chapter, or narrow the rule to the declarations that have one; this became an error in grund 0.16.0
```

Add the chapter or narrow the rule: those two are the whole answer. The error
fails the run like every other `must` finding; `--ignore unreached-declaration`
is the opt-out for a repository that wants the absence reported by nothing, and
a `should` rule reports it as a suggestion that never moves the exit status.

A chapter-presence rule beside the citation rule is not a third answer — it
raises its own `chapter-cardinality` and suppresses nothing, so one absent
chapter under both prints both lines. Write the two sentences together to state
the count as well as the citation:

```text
Each FS must have at least one requirements chapter.
The requirements chapter of each FS must cite at least one REQ.
```

The exact spelling reports the same absence as a different kind of finding.
`FS-login.requirements must cite at least one REQ.` names one unit, so deleting
that chapter makes the rule itself an `invalid-rule` finding — `literal subject
FS-login.requirements does not resolve` — located where the sentence is written
rather than at the declaration. Where both spellings stand over the same absent
chapter, both fire.

`grund list --selector FS.requirements` prints one row per declaration that has
the chapter, so the declarations it omits are the ones the rule now reports. A
subject that selects nothing at all prints nothing and exits 0.
<!-- END chapter-rules-emptiness -->

<!-- BEGIN chapter-rules-accepted -->
The complete accepted sentence forms, with representative findings, are:

- `Each FS must have at least one requirements chapter.` → `chapter-cardinality`.
- `Each FS must have at least 2 requirements chapters.` → `chapter-cardinality`.
- `FS-login should have at most 2 goals chapters.` → suggestion `chapter-cardinality`.
- `Each FS must have exactly one requirements chapter.` → `chapter-cardinality`.
- `Each FS should have exactly 2 review chapters.` → suggestion `chapter-cardinality`.
- `Each FS must cite at least one GOAL or REQ.` → zero matches reuse `missing-citation`.
- `The requirements chapter of each FS must cite at least one REQ.` → zero matches reuse `missing-citation`; a declaration with no such chapter, `unreached-declaration`.
- `Each FS should cite at least one GOAL.` → suggestion `suggested-citation`.
- `Each FS must cite at least 2 REQ.` → `citation-cardinality`, including at zero, because the floor is above one.
- `FS-login.requirements should cite at most 2 REQ.` → suggestion `citation-cardinality`.
- `FS-login.requirements must cite exactly one REQ.` → `citation-cardinality`.
- `AR-overview.system-overview must cite each AR at least once.` → one `citation-cardinality` per missed AR.
- `AR-overview.system-overview must cite each AR at least 2 times.` → one `citation-cardinality` per short-counted AR.
- `AR-overview.system-overview must cite each AR at most once.` → one `citation-cardinality` per repeated AR.
- `AR-overview.system-overview should cite each AR at most 2 times.` → suggestion `citation-cardinality`.
- `AR-overview.system-overview must cite each AR exactly once.` → one `citation-cardinality` per off-count AR.
- `AR-overview.system-overview should cite each AR exactly 2 times.` → suggestion `citation-cardinality`.
- `Each FS must be cited by at least one AR.` → `uncited-unit`.
- `Each FS must be cited by at least 2 AR.` → `uncited-unit`.
- `FS-login.requirements should be cited by at most 2 AR or GOAL.` → suggestion `uncited-unit`.
- `Each FS must be cited by exactly one AR.` → `uncited-unit`.
- `Each FS must not cite any AR.` → one site-anchored `forbidden-citation` per citation.
- `FS-login.requirements should not cite any AR or GOAL.` → one site-anchored `discouraged-citation` suggestion per citation.
<!-- END chapter-rules-accepted -->

Counts are positive decimal integers. Spell one as `one` where shown, and as
`once` for every per-target bound; numeric counts above one use plural
`chapters` or `times`. Chapter, outbound and inbound ceilings keep numeric one:
`at most 1 requirements chapter`, `cite at most 1 AR`, `be cited by at most 1 AR`.
Object kinds may be local
(`REQ`), pinned to a workspace member (`api/REQ`), or any member (`*/REQ`), and
`or` forms one normalized target set. A member nested inside another pins
itself with its whole alias path (`group/api/REQ` in member `group/api`), which
judges exactly like the bare kind. A namespaced object kind needs the workspace
in scope: a run without one says so at the rule's heading and still writes the block.

Per-target `at most once` allows zero or one physical citation to each target;
two sites citing the same target produce `citation-cardinality`, with
`requires at most 1` in the finding ([per-target counts](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-rules.md#33-per-target-coverage)).
Replace an existing per-target `at most 1 times` with `at most once`, then rerun
`grund init` to refresh rendered guidance. The numeric spelling is refused and
stops init before it writes ([migration decision](https://github.com/agent-grounds/grund/blob/main/docs/decisions/functional/DF-per-target-once-ceiling.md#compatibility-use-the-pre-10-allowance-not-a-verdict-correction)).

<!-- BEGIN chapter-rules-refused -->
Common refusals are intentional. A sentence is refused for the first stage it fails, and no later stage is read: the final `.`, the fixed word and the phase-1 subject, a conjunction, the modality, the rest of the subject, then the predicate. A configured kind that declares no IDs is refused as that, naming its home, rather than as an unknown kind, as in `kind "e2e" declares no IDs — tests/e2e/ is not a citable home`. Each names what failed and offers your sentence with only the refused part replaced, which is accepted when you paste it back and judges the same declarations and chapters you named; where grund cannot know what belongs in its place, or every sentence it could offer would judge other ones, it offers nothing, and `grund check --rule` lists the known kinds when what is missing is a kind:

- `Each FS may not cite any AR.` → `modality "may not" is not accepted; accepted form: Each FS must not cite any AR.`
- `Each FS must cite no AR.` → `"cite no" is not accepted; accepted form: Each FS must not cite any AR.`
- `Each FS must cite a GOAL.` → `quantifier "a" is ambiguous; accepted forms: "Each FS must cite at least one GOAL." or "Each FS must cite exactly one GOAL."`
- `Each FS must cite at least 1 GOAL.` → `numeric "at least 1" is not canonical; accepted form: Each FS must cite at least one GOAL.`
- `Each FS must cite exactly 1 GOAL.` → `numeric "exactly 1" is not canonical; accepted form: Each FS must cite exactly one GOAL.`
- `AR-overview.system-overview must cite each AR at most 1 times.` → `numeric "at most 1 times" is not canonical; accepted form: AR-overview.system-overview must cite each AR at most once.`
- `Each FS must cite at least one GOAL and must not cite any AR.` → `conjunctions are not accepted; accepted forms: "Each FS must cite at least one GOAL." and "Each FS must not cite any AR."`
- `Each FS must have exactly one  chapter.` → `chapter name must be a non-empty NAME with no surrounding whitespace; NAME forbids whitespace anywhere.`
- `Each FS must cite at least one GOAL` → `rule must end with "."; accepted form: Each FS must cite at least one GOAL.`
- `each FS must cite at least one GOAL.` → `fixed word "Each" is case-sensitive; accepted form: Each FS must cite at least one GOAL.`
- `Every FS must cite at least one GOAL.` → `quantifier "every" is not accepted; accepted form: Each FS must cite at least one GOAL.`
- `The requirements chapter of every FS must cite at least one REQ.` → `quantifier "every" is not accepted; accepted form: The requirements chapter of each FS must cite at least one REQ.`
- `AR-overview.system-overview must cite every AR exactly once.` → `quantifier "every" is not accepted; accepted form: AR-overview.system-overview must cite each AR exactly once.`
- `Each file in vendor/ must cite at least one FS.` → `path subjects are not accepted in phase 1`, then `known kinds: GOAL, REQ, FS, AR, RULE`
- `Each */FS must cite at least one GOAL.` → `subject namespaces must be local in phase 1; accepted form: Each FS must cite at least one GOAL.`
- `FS-login.* must cite at least one REQ.` → `section-component wildcards are not accepted in phase 1` (no rewrite)
- `Each chapter of each FS must cite at least one REQ.` → `chapter-quantified subjects are not accepted in phase 1` (no rewrite)
- `FS-login.2 must cite at least one REQ.` → `numbered chapter subjects can detach when headings move` (no rewrite)
- With named sections off, `FS-login.requirements must cite at least one REQ.` → `named chapter subjects require [id] named_sections = true; accepted form after enabling it: FS-login.requirements must cite at least one REQ.`
- `Each POLICY must cite at least one GOAL.` → `unknown kind "POLICY"`, then `known kinds: GOAL, REQ, FS, AR, RULE`
<!-- END chapter-rules-refused -->

A refused subject's offer keeps its recoverable chapter and declaration scope.
For example, `Each FS requirements chapter must cite at least one REQ.`
can be repaired to `The requirements chapter of each FS must cite at least one REQ.`;
the dotted `Each FS.requirements` and the shorter `Each FS requirements`
name that same scope. `FS-store requirements chapter` can be repaired to
`FS-store.requirements`, preserving the one declaration. If grund cannot
recover that scope unambiguously, it offers no form. An unnamed
`Each chapter of each FS` therefore gets no rewrite; choose the chapter
explicitly. See [scope-preserving recovery](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-rule-refusals.md#sentencesrewriterecovered-only-what-can-be-recovered-is-supplied).

Nor does an offer drop a chapter you typed to reach a sentence grund accepts,
because the declaration or parent chapter left behind would judge other units:
the rule would pass on a citation of any chapter, or count the wrong chapters.
`FS-login.2`, `FS-login.*` and `The requirements.1 chapter of each FS` get no
rewrite, and neither does a chapter subject given a chapter count, as in
`FS-login.requirements must have exactly one should chapter.`. No rule sentence
asks those questions today. To see whether a numbered section is cited, run
`grund refs FS-login.2`; to count the chapters under a chapter, list them with
`grund list --glob 'FS-*.requirements.*'`. `grund list --selector` still
answers `FS.requirements.1` with `FS.requirements`, because a selector judges
nothing. See [offers that judge the same units](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-rule-refusals.md#sentencesrewritesame-units-a-form-judges-the-units-the-typed-sentence-named-or-none-is-offered).

Use `Each` for a declaration subject and `each` after `chapter of` or `cite`.
The unsupported quantifier `every` stays refused; grund offers the `each`
spelling after reparsing it, preserving the other typed parts. `list --selector`
offers a selector for the two subject forms. A chapter actually named `every`,
a nested `every.detail` path, and an ID such as `FS-every` remain valid.
See [the quantifier refusal](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-rule-refusals.md#sentencesevery-unsupported-every-is-refused-as-a-quantifier-with-an-each-repair).

For a configured, scanned non-citable place `e2e` at `tests/e2e/`, both
`Each e2e must cite at least one FS.` and
`Each file in tests/e2e/ must cite at least one FS.` remain refused. Follow
the supplementary hint to express this obligation in a version-2 `grund.toml`
(`[citations.e2e]` in version 1):

```toml
[rules.citations.e2e]
must = ["FS"]
```

Use `[citations.e2e]` in a version-1 config. That entry judges only a file
that already cites something, so a file there that cites nothing still passes.
Where the row grounds nothing, the hint therefore also names the grounding
setting that makes such a file a finding, in a version-2 `grund.toml`:

```toml
[rules.citations.e2e.grounding]
must = "file"
```

Use `require_grounding = true` on the `e2e` `[[kinds]]` row in a version-1
config. Where the row already grounds, the hint names the citation entry alone.
This is configuration guidance; it offers no accepted sentence rewrite. See the
[place-subject guidance](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-rule-refusals.md#sentencesrewriteno-formplace-a-refused-place-subject-names-its-citation-configuration)
for the exact matching boundary, and the [runnable example](https://github.com/agent-grounds/grund/tree/main/examples/rules)
for the refusal and its citation-direction finding.

#### Narrowing a subject by a field's value

A version-2 kind's scalar field, one that sets `one_of`, can narrow a
quantified subject. End `Each KIND` or `The NAME chapter of each KIND` in one
`whose FIELD is VALUE` clause, and the rule reaches only the declarations whose
field `FIELD` holds `VALUE` ([whose clause](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)).
A version-2 `grund.toml` declares such a field as:

```toml
[schema.kinds.SWR.fields.verification]
one_of = ["Test", "Analysis", "Inspection", "Demonstration"]
presence = "must"
```

```text
Each SWR whose verification is Test must be cited by at least one TC.
The evidence chapter of each SWR whose verification is Test must cite at least one TC.
```

`FIELD` is the field's name, the direct section that holds it, and `VALUE` runs
from after ` is ` to the modality and must equal one `one_of` entry byte for
byte: `is test` does not match `Test`. A value may hold spaces, a capitalised
`Must`, or a final `.`, because the subject ends only at a lowercase ` must `,
` should ` or their negations. A declaration whose field is absent or holds
another value is outside the rule: it is neither checked nor unreached, and the
field's own `missing-field` or `invalid-field-value` finding is what reports a
missing or wrong value. A filtered subject is never kind-wide, so it never
folds into a citation direction, `[rules.citations]` in version 2. `grund list --selector` takes the
same clause.

A clause the grammar does not accept is refused for the part that failed,
checked in this order: a version-1 config, the clause's shape, the field,
whether it is a scalar, the value's whitespace, then the value ([whose refusals](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-rule-refusals.md#sentenceswhose-a-whose-clause-is-refused-for-the-part-that-failed)). In a repository whose
`SWR` declares the scalar `verification` and the prose field `rationale`, whose
`AN` declares the scalar `status` with `one_of = ["Draft", "Must fix", "fix and
retest", "fix must wait"]`, and whose `TC` declares no field, the reasons are:

- `Each SWR whose verfication is Test must be cited by at least one TC.` → `"verfication" is not a field of kind SWR; its scalar fields: verification`
- `Each SWR whose rationale is Test must be cited by at least one TC.` → `field "rationale" of kind SWR is not a scalar: it sets no one_of; its scalar fields: verification`
- `Each TC whose verification is Test must cite at least one SWR.` → `"verification" is not a field of kind TC; kind TC declares no scalar field`
- `Each SWR whose verification is Tset must be cited by at least one TC.` → `"Tset" is not a value of SWR.verification; kind SWR admits Test, Analysis, Inspection, Demonstration`
- `Each SWR whose verification is Test or Demonstration must be cited by at least one TC.` → `"Test or Demonstration" is not a value of SWR.verification; kind SWR admits Test, Analysis, Inspection, Demonstration; accepted forms: "Each SWR whose verification is Test must be cited by at least one TC." and "Each SWR whose verification is Demonstration must be cited by at least one TC."`
- `Each AN whose status is fix must wait must cite at least one SWR.` → `"fix" is not a value of AN.status; kind AN admits Draft, Must fix, fix and retest, fix must wait — value "fix must wait" holds the modal word "must", which ends the subject, so a rule cannot name it`
- `Each SWR whose verification must be cited by at least one TC.` → `whose clause "whose verification" has no " is "; write whose FIELD is VALUE`
- `Each SWR whose  is Test must be cited by at least one TC.` (two spaces) → `whose clause "whose  is Test" names no field; write whose FIELD is VALUE`
- `Each SWR whose verification is  must be cited by at least one TC.` (two spaces) → `whose clause "whose verification is " names no value; write whose FIELD is VALUE`
- `Each SWR whose verification is Test  must be cited by at least one TC.` (two spaces) → `value "Test " of SWR.verification has surrounding whitespace, which no one_of value can hold`
- `SWR-brake-latency whose verification is Test must be cited by at least one TC.` → `literal subject "SWR-brake-latency whose verification is Test" does not match the configured ID grammar — a whose clause narrows only Each KIND and The NAME chapter of each KIND`
- `Each AN must cite at least one SWR whose verification is Test.` → `unknown kind "SWR whose verification is Test" — a whose clause narrows only a rule's subject`, then `known kinds: SWR, TC, AN`
- `Each POLICY whose status is Open must cite at least one SWR.` → `unknown kind "POLICY"`, then `known kinds: SWR, TC, AN`
- A version-1 config → `whose clauses require grund_config_version = 2`

Only a value joined by ` or ` is offered a form, one sentence per value; a
selector is offered none. Every rule row reads the same with
`The evidence chapter of each SWR` for `Each SWR`, and `list --selector` gives
the same reasons for the same subjects. Each refusal prints its reason alone,
for the rows above and for a sentence whose clause is accepted but whose
quantifier, object kind or predicate is refused, and `list --selector` follows
one with `known kinds:` only where the kind is not configured. Through grund
0.21.0 the reason carried the one grund printed before clauses existed as a
prefix
([the closed window](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-errors.md#310-the-whose-clause-wording-migration)):

```text
$ grund check --rule 'Each SWR whose verification is Tset must be cited by at least one TC.'
error: "Tset" is not a value of SWR.verification; kind SWR admits Test, Analysis, Inspection, Demonstration
```

The [runnable examples](https://github.com/agent-grounds/grund/tree/main/examples)
include one that walks this rule from every SWR to the tested ones, as a
declared rule and as a selector.

Try a sentence without adding a declaration:

```bash
grund check --rule "Each FS should have exactly one security chapter." --suggestions
```

List the units a subject denotes:

```bash
grund list --selector FS.requirements
grund list --selector FS.requirements --format json
```

Configured syntax failures are located `invalid-rule` findings; `--rule`
syntax/vocabulary failures stop before scanning with exit 2. A syntactically
valid missing literal, such as `FS-missing`, is resolved after scanning and is
an `invalid-rule` finding with exit 1.

Findings sort bytewise by path, line, then message. Two identical declarations
collapse semantically and name sorted authorities, for example
`(RULE-a, RULE-b)`. If an existing `[citations]` (version 1) or `[rules.citations]`
(version 2) entry says the same bare-kind rule, the existing config finding
wins byte-for-byte and no rule tail is added.
<!-- END chapter-rules -->
