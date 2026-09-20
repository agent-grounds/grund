# Writing chapter rules

Chapter rules turn repository conventions into checked, agent-readable
declarations. They are deliberately a small language: if a sentence parses, a
reader and `grund check` give it the same meaning. See §FS-rules.1 and
§FS-rules.10.

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

<!-- BEGIN chapter-rules-accepted -->
The complete accepted sentence forms, with representative findings, are:

- `Each FS must have at least one requirements chapter.` → `chapter-cardinality`.
- `FS-login should have at most 2 goals chapters.` → suggestion `chapter-cardinality`.
- `Each FS must have exactly one requirements chapter.` → `chapter-cardinality`.
- `Each FS should have exactly 2 review chapters.` → suggestion `chapter-cardinality`.
- `Each FS must cite at least one GOAL or REQ.` → zero matches reuse `missing-citation`.
- `Each FS should cite at least one GOAL.` → suggestion `suggested-citation`.
- `FS-login.requirements should cite at most 2 REQ.` → suggestion `citation-cardinality`.
- `FS-login.requirements must cite exactly one REQ.` → `citation-cardinality`.
- `AR-overview.system-overview must cite each AR at least once.` → one `citation-cardinality` per missed AR.
- `AR-overview.system-overview should cite each AR at most 2 times.` → suggestion `citation-cardinality`.
- `AR-overview.system-overview must cite each AR exactly once.` → one `citation-cardinality` per off-count AR.
- `AR-overview.system-overview should cite each AR exactly 2 times.` → suggestion `citation-cardinality`.
- `Each FS must be cited by at least one AR.` → `uncited-unit`.
- `FS-login.requirements should be cited by at most 2 AR or GOAL.` → suggestion `uncited-unit`.
- `Each FS must be cited by exactly one AR.` → `uncited-unit`.
- `Each FS must not cite any AR.` → one site-anchored `forbidden-citation` per citation.
- `FS-login.requirements should not cite any AR or GOAL.` → one site-anchored `discouraged-citation` suggestion per citation.
<!-- END chapter-rules-accepted -->

Counts are positive decimal integers. Spell one as `one` where shown; numeric
counts other than one use plural `chapters` or `times`. Object kinds may be local
(`REQ`), pinned to a workspace member (`api/REQ`), or any member (`*/REQ`), and
`or` forms one normalized target set.

<!-- BEGIN chapter-rules-refused -->
Common refusals are intentional and name the exact accepted rewrite:

- `Each FS may not cite any AR.` → `modality "may not" is not accepted; accepted form: Each FS must not cite any AR.`
- `Each FS must cite no AR.` → `"cite no" is not accepted; accepted form: Each FS must not cite any AR.`
- `Each FS must cite a GOAL.` → `quantifier "a" is ambiguous; accepted forms: "Each FS must cite at least one GOAL." or "Each FS must cite exactly one GOAL."`
- `Each FS must cite at least one GOAL and must not cite any AR.` → `conjunctions are not accepted; accepted forms: "Each FS must cite at least one GOAL." and "Each FS must not cite any AR."`
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

The runnable [`examples/rules/`](../../examples/rules/) repository includes a
passing and violated instance of all five families, shared-prefix coverage
targets, both deduplication directions, both channels, and the strict refusal
inventory. `grund init` repeats valid configured sentences under a v11
`### Chapter rules` managed section; without a rule kind, v10 bytes are
unchanged. §FS-rules.6 §FS-rules.7.6 §FS-rules.9
