# Writing chapter rules

Chapter rules turn repository conventions into checked, agent-readable
declarations. They are deliberately a small language: if a sentence parses, a
reader and `grund check` give it the same meaning. See [§FS-rules.1](../functional-spec/FS-rules.md#1-rule-declarations-and-opt-in) and
[§FS-rules.10](../functional-spec/FS-rules.md#10-documentation-and-executable-examples).

Opt in one citable Markdown kind, then put one sentence and a non-empty
rationale in every declaration:

```toml
[[kinds]]
kind = "RULE"
folder = "docs/rules"
index = false
rules = true

[citations.RULE]
must = ["GOAL"]
```

```markdown
# RULE-requirements: The requirements chapter of each FS must cite at least one REQ.

This keeps requirements tied to authority, because §GOAL-grounded-rules.
```

The title is a grammar island: formatting never inserts markers, links, or
expanded shorthand into it. The rationale remains ordinary grounded Markdown.

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

Named-chapter subjects require `[id] named_sections = true`. Phase 1 has no
paths, files, folders, wildcards, subject namespaces, numbered-chapter subjects,
exceptions, definitions, derived terms, settings, or source-code symbols.

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

Counts are positive decimal integers. Spell one as `one` where shown; numeric
counts other than one use plural `chapters` or `times`. Object kinds may be local
(`REQ`), pinned to a workspace member (`api/REQ`), or any member (`*/REQ`), and
`or` forms one normalized target set. A namespaced object kind needs the workspace
in scope: a run without one says so at the rule's heading and still writes the block.

<!-- BEGIN chapter-rules-refused -->
Common refusals are intentional and name the exact accepted rewrite:

- `Each FS may not cite any AR.` → `modality "may not" is not accepted; accepted form: Each FS must not cite any AR.`
- `Each FS must cite no AR.` → `"cite no" is not accepted; accepted form: Each FS must not cite any AR.`
- `Each FS must cite a GOAL.` → `quantifier "a" is ambiguous; accepted forms: "Each FS must cite at least one GOAL." or "Each FS must cite exactly one GOAL."`
- `Each FS must cite at least 1 GOAL.` → `numeric "at least 1" is not canonical; accepted form: Each FS must cite at least one GOAL.`
- `Each FS must cite exactly 1 GOAL.` → `numeric "exactly 1" is not canonical; accepted form: Each FS must cite exactly one GOAL.`
- `Each FS must cite at least one GOAL and must not cite any AR.` → `conjunctions are not accepted; accepted forms: "Each FS must cite at least one GOAL." and "Each FS must not cite any AR."`
- `Each FS must have exactly one  chapter.` → `chapter name must be a non-empty NAME with no surrounding whitespace; accepted form: Each FS must have exactly one requirements chapter.`
- `Each FS must cite at least one GOAL` → `rule must end with "."; accepted form: Each FS must cite at least one GOAL.`
- `each FS must cite at least one GOAL.` → `fixed word "Each" is case-sensitive; accepted form: Each FS must cite at least one GOAL.`
- `Each file in vendor/ must cite at least one FS.` → `path subjects are not accepted in phase 1; accepted form: Each FS must cite at least one GOAL.`
- `Each */FS must cite at least one GOAL.` → `subject namespaces must be local in phase 1; accepted form: Each FS must cite at least one GOAL.`
- `FS-login.* must cite at least one REQ.` → `section-component wildcards are not accepted in phase 1; accepted form: FS-login.requirements must cite at least one REQ.`
- `Each chapter of each FS must cite at least one REQ.` → `chapter-quantified subjects are not accepted in phase 1; accepted form: The requirements chapter of each FS must cite at least one REQ.`
- `FS-login.2 must cite at least one REQ.` → `numbered chapter subjects can detach when headings move; accepted form: FS-login.requirements must cite at least one REQ.`
- With named sections off, `FS-login.requirements must cite at least one REQ.` → `named chapter subjects require [id] named_sections = true; accepted form after enabling it: FS-login.requirements must cite at least one REQ.`
- `Each POLICY must cite at least one GOAL.` → `unknown kind "POLICY"; accepted form: Each FS must cite at least one GOAL.`
<!-- END chapter-rules-refused -->

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
`(RULE-a, RULE-b)`. If an existing `[citations]` entry says the same bare-kind
rule, the existing config finding wins byte-for-byte and no rule tail is added.
<!-- END chapter-rules -->

A namespaced object kind — `api/REQ` or `*/REQ` — is resolved against the
workspace the run's own `grund.toml` declares, so it resolves at the workspace
root and in any run that loaded that workspace. A run rooted inside a member
declares no `[workspace]` of its own and holds no namespace at all, so it cannot
judge the alias either way: `grund check` and `grund init` report it at the
rule's own heading as `invalid-rule`, the run exits nonzero, and `grund init`
writes the managed block anyway with the rule's bullet rendered exactly as
authored. That last part is what lets a member complete a `grund` upgrade: the
block it writes is byte-for-byte what a run holding the whole workspace would
write for it. One genuinely invalid rule beside it and nothing is written at
all, which is the ordinary refusal.

The runnable [`examples/rules/`](../../examples/rules/) repository includes a
passing and violated instance of all five families, shared-prefix coverage
targets, both deduplication directions, both channels, and the strict refusal
inventory. `grund init` repeats valid configured sentences under a v11
`### Chapter rules` managed section; without a rule kind, v10 bytes are
unchanged. [§FS-rules.6](../functional-spec/FS-rules.md#6-semantic-deduplication) [§FS-rules.7.6](../functional-spec/FS-rules.md#76-selection-json-ordering-and-exits) [§FS-rules.9](../functional-spec/FS-rules.md#9-managed-guidance-and-editor-parity)

### Asking only what one sentence found

`--rule` adds the sentence to every configured rule rather than replacing them,
so without help its findings arrive sorted into the whole tree's report and only
the authority in the message tail tells them apart. `--only-rule` narrows the
report to what the sentence authored, which is what makes trying a sentence cost
its own findings:

```bash
grund check --rule "FS-login.requirements must cite exactly 3 REQ." --only-rule
grund check --rule "Each FS should have exactly one security chapter." --only-rule --suggestions
```

It requires `--rule` — alone it is `error: --only-rule requires --rule` and exit
2, rather than a report scoped to nothing — and it narrows by authority where
`--only` narrows by code, so the two intersect and `--ignore` wins over both. A
`should`-level sentence lands in the suggestions channel the default run
withholds, so the second form needs `--suggestions` as well or the scoped run
prints `success`. A finding the sentence authored jointly with a declared rule
that means the same is in a scoped report, with its tail unchanged; a sentence
duplicating a `[citations]` direction authors nothing, so its scoped report is
empty and the run says `success` while the tree's own findings still stand.

Under `--format json` every finding record carries the same authority as its
last field — `null` where no rule authored it, `["--rule"]` for the trial
sentence, `["--rule","RULE-security"]` where both reached the same meaning — so
a caller can ask which rule said this without matching message text.
[§FS-rules.8](../functional-spec/FS-rules.md#8-command-surfaces) [§FS-check.1.4](../functional-spec/FS-check.md#14-selecting-findings-with---only-and---ignore) [§FS-errors.5.1](../functional-spec/FS-errors.md#51-on-stdout--the-commands-output)
