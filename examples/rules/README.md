# Chapter rules

This runnable repository shows the grounded rule declarations of
[§FS-rules](../../docs/functional-spec/FS-rules.md#fs-rules-grounded-declarations-state-and-enforce-chapter-rules).
Run it from the project root:

```bash
grund check examples/rules/repo --suggestions
echo $?    # 1: hard violations remain even though suggestions do not gate
```

`RULE-requirements` demonstrates **must cite at least one**: a citation in the
`goals` chapter does not satisfy the rule for the `requirements` chapter.
`RULE-requirements-floor` demonstrates **must cite at least N**, a floor above
one: the same chapter falls short of two, and reports `citation-cardinality`
rather than `missing-citation` even at zero. `RULE-overview` demonstrates **must
cite each** with an exact count: one AR is cited twice and another zero times.
Their four deterministic findings are the checked `expected.stdout` below.

`FS-unreached` has no `requirements` chapter at all, so the two chapter-scoped
citation rules over `FS` cannot reach it, and each says so at its title line
([§FS-rules.checks.unreached-declaration](../../docs/functional-spec/FS-rules.md#checksunreached-declaration-unreached-declaration)):

```text
docs/fs/FS-unreached.md:1: error: FS-unreached has no requirements chapter, so RULE-inbound, RULE-inbound-copy cannot reach it; add the chapter, or narrow the rule to the declarations that have one; this became an error in grund 0.16.0
docs/fs/FS-unreached.md:1: error: FS-unreached has no requirements chapter, so RULE-requirements cannot reach it; add the chapter, or narrow the rule to the declarations that have one; this became an error in grund 0.16.0
```

Two lines for one absent chapter, because each semantic rule group reports its
own: the first names the `RULE-inbound`, `RULE-inbound-copy` group the way
every grouped authority is named. Each is an error on the ordinary `must`
channel ([§FS-rules.7](../../docs/functional-spec/FS-rules.md#7-findings-and-channels)), and the two actions it names are the only
two: add the chapter, or narrow the rule's subject.

The complete writing guide also demonstrates the remaining released families:
**must have** a named chapter, **be cited by** another kind, and **must not cite
any** prohibited kind. It shows both hard and recommendation findings, why
kinds with a **shared prefix** are matched longest-first, and both semantic
deduplication directions: **config-to-rule** keeps the established citation
direction wording, while **rule-to-rule** combines the sorted rule IDs into one
authority such as `RULE-a, RULE-b`. A malformed configured declaration produces
an **invalid-rule** finding; a malformed `check --rule` sentence is refused
before scanning.

See the [chapter-rules guide](../../docs/user-facing/rules.md) for every accepted
sentence, every common refusal and its exact rewrite, and the finding produced
by each family.

## Complete sentence inventory

These are the accepted spellings exercised by the parser contract; substitute
configured kinds, full IDs, and named chapter handles without changing the
fixed words:

```text
Each FS must have at least one requirements chapter.
Each FS must have at least 2 requirements chapters.
FS-demo should have at most 2 requirements chapters.
Each FS must have exactly one requirements chapter.
Each FS should have exactly 2 requirements chapters.
Each FS must cite at least one GOAL or REQ.
Each FS should cite at least one GOAL.
FS-demo.requirements must cite at least 2 REQ.
The requirements chapter of each FS should cite at most 2 REQ.
FS-demo.requirements must cite exactly one REQ.
AR-overview.system-overview must cite each AR at least once.
AR-overview.system-overview must cite each AR at least 2 times.
AR-overview.system-overview should cite each AR at most 2 times.
AR-overview.system-overview must cite each AR exactly once.
AR-overview.system-overview should cite each AR exactly 2 times.
Each FS must be cited by at least one AR.
Each FS must be cited by at least 2 AR.
FS-demo.requirements should be cited by at most 2 AR or GOAL.
Each FS must be cited by exactly one AR.
Each FS must not cite any AR.
FS-demo.requirements should not cite any AR or GOAL.
```

The deliberate violations in this repository produce, verbatim, the findings
in [`expected.stdout`](expected.stdout): `chapter-cardinality`, rule and config
forms of `missing-citation`, aggregate and per-target `citation-cardinality`,
`uncited-unit`, `forbidden-citation`, `invalid-rule`, `suggested-citation`,
`discouraged-citation`, and `unreached-declaration`. `FS-good` and
`RULE-chapter-pass` supply passing instances; the two `RULE-inbound*` and two `RULE-positive*` declarations show
rule-to-rule deduplication, while `RULE-config-duplicate` shows config-to-rule
precedence.

## Refusal inventory

Each left-hand sentence is refused; the right-hand text is the exact accepted
rewrite grund reports:

- `Each FS may not cite any AR.` → `Each FS must not cite any AR.`
- `Each FS must cite no AR.` → `Each FS must not cite any AR.`
- `Each FS must cite a GOAL.` → `Each FS must cite at least one GOAL.` or `Each FS must cite exactly one GOAL.`
- `Each FS must cite at least 1 GOAL.` → spell one as the word: `Each FS must cite at least one GOAL.`
- `Each FS must cite exactly 1 GOAL.` → spell one as the word: `Each FS must cite exactly one GOAL.`
- `Each FS must cite at least one GOAL and must not cite any AR.` → split it into `Each FS must cite at least one GOAL.` and `Each FS must not cite any AR.`
- `Each FS must cite at least one GOAL` → add the terminal `.`.
- `each FS must cite at least one GOAL.` → `Each FS must cite at least one GOAL.`
- `Each file in vendor/ must cite at least one FS.` → `Each FS must cite at least one GOAL.`
- `Each */FS must cite at least one GOAL.` → `Each FS must cite at least one GOAL.`
- `FS-demo.* must cite at least one REQ.` → `FS-demo.requirements must cite at least one REQ.`
- `Each chapter of each FS must cite at least one REQ.` → `The requirements chapter of each FS must cite at least one REQ.`
- `FS-demo.2 must cite at least one REQ.` → `FS-demo.requirements must cite at least one REQ.`
- With named sections disabled, enable them before writing `FS-demo.requirements must cite at least one REQ.`
- `Each POLICY must cite at least one GOAL.` → use a configured kind, as in `Each FS must cite at least one GOAL.`
