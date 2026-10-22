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
Common refusals are intentional. A sentence is refused for the first stage it fails, and no later stage is read: the final `.`, the fixed word and the phase-1 subject, a conjunction, the modality, the rest of the subject, then the predicate. A configured kind that declares no IDs is refused as that, naming its home, rather than as an unknown kind, as in `kind "e2e" declares no IDs — tests/e2e/ is not a citable home`. Each names what failed and offers your sentence with only the refused part replaced, which is accepted when you paste it back; where grund cannot know what belongs in its place, it offers nothing, and `grund check --rule` lists the known kinds when what is missing is a kind:

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
- `FS-login.* must cite at least one REQ.` → `section-component wildcards are not accepted in phase 1; accepted form: FS-login must cite at least one REQ.`
- `Each chapter of each FS must cite at least one REQ.` → `chapter-quantified subjects are not accepted in phase 1; accepted form: Each FS must cite at least one REQ.`
- `FS-login.2 must cite at least one REQ.` → `numbered chapter subjects can detach when headings move; accepted form: FS-login must cite at least one REQ.`
- With named sections off, `FS-login.requirements must cite at least one REQ.` → `named chapter subjects require [id] named_sections = true; accepted form after enabling it: FS-login.requirements must cite at least one REQ.`
- `Each POLICY must cite at least one GOAL.` → `unknown kind "POLICY"`, then `known kinds: GOAL, REQ, FS, AR, RULE`
<!-- END chapter-rules-refused -->

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
setting that makes such a file a finding:

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
