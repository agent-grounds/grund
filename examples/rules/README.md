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

`RULE-overview-ceiling` says `AR-overview.system-overview must cite each AR at most once.`
The overview cites itself once and `AR-one-more` zero times, which both pass;
its two physical citations to `AR-one` violate the ceiling. The finding keeps
`requires at most 1`. `RULE-invalid-ceiling` demonstrates the refused numeric
spelling: replace per-target `at most 1 times` with `at most once`, then rerun
`grund init` to update guidance. An invalid numeric title stops init before writes.

See the [chapter-rules guide](../../docs/user-facing/rules.md) for every accepted
sentence, every common refusal and any rewrite offered, and the finding
produced by each family.

## Configured place obligations

The separate [`place-obligations/repo`](place-obligations/repo) fixture configures
a scanned non-citable `e2e` row at `tests/e2e/`.
Both subject spellings remain refused, with supplementary configuration
guidance ([§FS-rule-refusals.sentences.rewrite.no-form.place](../../docs/functional-spec/FS-rule-refusals.md#sentencesrewriteno-formplace-a-refused-place-subject-names-its-citation-configuration)):

```bash
grund check examples/rules/place-obligations/repo --rule 'Each e2e must cite at least one FS.' --only-rule
grund check examples/rules/place-obligations/repo --rule 'Each file in tests/e2e/ must cite at least one FS.' --only-rule
# Both exit 2. Their existing reasons end with:
# ; for configured place "e2e", express this obligation in grund.toml: [rules.citations.e2e] with must = ["FS"], and [rules.citations.e2e.grounding] must = "file", without which a file there that cites nothing is not checked
```

This fixture uses version 2. In version 1, the hint instead names
`[citations.e2e]` and `require_grounding = true` on its `[[kinds]]` row. The
row grounds nothing here, so the hint names the grounding setting too; once the
row grounds, it names the citation entry alone. The suggested obligation is
already configured here:

```toml
[rules.citations.e2e]
must = ["FS"]
```

To see its enforcement independently of the base fixture's chapter-rule
findings, run from the project root:

```bash
grund check examples/rules/place-obligations/repo/tests/e2e/
# tests/e2e/missing-fs.md:1: error: tests/e2e/ must cite FS (citation direction); add such a citation within it
# Exit 1: the file cites AR-one but lacks FS evidence.
```

Adding a citation to `FS-good` in that file satisfies this direction rule.
This command temporarily adds the evidence and restores the fixture afterward:

```bash
python - <<'PY'
from pathlib import Path
import subprocess

repo = Path("examples/rules/place-obligations/repo")
path = repo / "tests/e2e/missing-fs.md"
original = path.read_bytes()
try:
    path.write_bytes(original + "Required evidence: §FS-good.\n".encode())
    subprocess.run(["grund", "check", "tests/e2e/"], cwd=repo, check=True)
finally:
    path.write_bytes(original)
PY
# Exit 0; the original missing-FS fixture remains in place.
```

Files with no citations remain exempt until the grounding setting the hint
names requires evidence in every file ([§FS-config.3.9.2.4](../../docs/functional-spec/FS-config.md#3924-obligations-apply-per-source-file)).
The hint names the supported configuration route and offers no accepted
sentence rewrite. The unrelated `vendor/` refusal below still has no hint.

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
AR-overview.system-overview must cite each AR at most once.
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

Offers keep recoverable chapter and literal scope
([§FS-rule-refusals.sentences.rewrite.recovered](../../docs/functional-spec/FS-rule-refusals.md#sentencesrewriterecovered-only-what-can-be-recovered-is-supplied)).
`Each FS requirements chapter must cite at least one REQ.` can be repaired to
`The requirements chapter of each FS must cite at least one REQ.`, and
`FS-good requirements chapter` to `FS-good.requirements`. When the scope cannot
be reconstructed, grund offers no form; an unnamed chapter quantifier below
therefore has no rewrite.

Each left-hand sentence is refused; the right-hand text describes the refusal
and any accepted rewrite grund reports. Some refusals offer no rewrite
([§FS-rules.10.1](../../docs/functional-spec/FS-rules.md#101-example-refusal-inventory)):

- `Each FS may not cite any AR.` → `Each FS must not cite any AR.`
- `Each FS must cite no AR.` → `Each FS must not cite any AR.`
- `Each FS must cite a GOAL.` → `Each FS must cite at least one GOAL.` or `Each FS must cite exactly one GOAL.`
- `Each FS must cite at least 1 GOAL.` → spell one as the word: `Each FS must cite at least one GOAL.`
- `Each FS must cite exactly 1 GOAL.` → spell one as the word: `Each FS must cite exactly one GOAL.`
- `AR-overview.system-overview must cite each AR at most 1 times.` → `numeric "at most 1 times" is not canonical; accepted form: AR-overview.system-overview must cite each AR at most once.`
- `AR-overview.system-overview must cite each AR at most 1.` → `per-target counts must end in "times"; accepted form: AR-overview.system-overview must cite each AR at most once.`
- `AR-overview.system-overview should cite each AR at most 1 time.` → `per-target counts must end in "times"; accepted form: AR-overview.system-overview should cite each AR at most once.`
- `AR-overview.system-overview must cite each AR at most 01 times.` → `count must be a canonical positive base-10 integer; accepted form: AR-overview.system-overview must cite each AR at most once.`
- `Each FS must cite at least one GOAL and must not cite any AR.` → split it into `Each FS must cite at least one GOAL.` and `Each FS must not cite any AR.`
- `Each FS must cite at least one GOAL` → add the terminal `.`.
- `each FS must cite at least one GOAL.` → `Each FS must cite at least one GOAL.`
- `Every FS must cite at least one GOAL.` → `quantifier "every" is not accepted; accepted form: Each FS must cite at least one GOAL.`
- `The requirements chapter of every FS must cite at least one REQ.` → `quantifier "every" is not accepted; accepted form: The requirements chapter of each FS must cite at least one REQ.`
- `AR-overview.system-overview must cite every AR exactly once.` → `quantifier "every" is not accepted; accepted form: AR-overview.system-overview must cite each AR exactly once.`
- `Each file in vendor/ must cite at least one FS.` → `path subjects are not accepted in phase 1`,
  then `known kinds: GOAL, REQ, FS, AR, RULE`; no rewrite is offered.
- `Each */FS must cite at least one GOAL.` → `Each FS must cite at least one GOAL.`
- `FS-demo.* must cite at least one REQ.` → `FS-demo.requirements must cite at least one REQ.`
- `Each chapter of each FS must cite at least one REQ.` → `chapter-quantified subjects are not accepted in phase 1` (no rewrite)
- `FS-demo.2 must cite at least one REQ.` → `FS-demo.requirements must cite at least one REQ.`
- With named sections disabled, enable them before writing `FS-demo.requirements must cite at least one REQ.`
- `Each POLICY must cite at least one GOAL.` → use a configured kind, as in `Each FS must cite at least one GOAL.`

The three `RULE-every-*` declarations exercise these unsupported quantifiers
in the runnable repository; their `invalid-rule` findings and complete `each`
offers are pinned in `expected.stdout`
([§FS-rule-refusals.sentences.every](../../docs/functional-spec/FS-rule-refusals.md#sentencesevery-unsupported-every-is-refused-as-a-quantifier-with-an-each-repair)).
`list --selector 'Every FS'` likewise refuses the quantifier and offers
`Each FS`. The spelling of a chapter named `every` or an ID such as `FS-every`
is unaffected.
