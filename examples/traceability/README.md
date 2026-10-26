# Requirements traceability: a V-model trace held to a grund schema

Regulated software is built down a V and verified back up it. Stakeholder needs
become system requirements, system requirements become software requirements,
those become a design, and the design becomes code. Tests and analyses then
verify every level. The evidence an audit under DO-178C (airborne software),
ISO 26262 (road vehicles) or IEC 62304 (medical-device software) asks for is the
trace between those levels, in both directions. Each requirement must be
realized and verified, and nothing in the code or the tests may lack a
requirement that asked for it.

Teams write each requirement in a controlled form, such as the five EARS
patterns (Mavin et al., RE'09). It carries the attributes ISO/IEC/IEEE 29148
lists: an identifier, a rationale, and a verification method (Inspection,
Analysis, Demonstration or Test). The trace usually lives in a requirements
tool or a spreadsheet, a step away from the code it describes, and it drifts.

This example is for a team that keeps its requirements, design, code and tests in
one repository. It wants `grund check` to hold the trace and `grund refs` to
produce the matrix ([§FS-examples.2](../../docs/functional-spec/FS-examples.md#2-canonical-use-cases)). It produces trace evidence. It does not make a
project compliant with any standard, and what an assessor accepts is the
assessor's call.

The fixture is LC-1, a fictional single-cell lithium-ion charge controller. It
holds two stakeholder needs and five system requirements, one per EARS pattern.
Below them are seven software requirements, three parameters, three design
elements and a little C. On the verification side are six test cases, one
analysis and one test record. The example pins three runs
([§FS-examples.5.5](../../docs/functional-spec/FS-examples.md#55-an-example-may-pin-several-runs)): the schema, the check, and the trace matrix. None writes anything.

## The repository

```text
grund.toml                            the schema
requirements/stakeholder-needs.md     STK-001..002   stakeholder needs
requirements/system.md                SYS-001..005   system requirements, EARS
requirements/software.md              SWR-001..007   software requirements, EARS
requirements/parameters.md            PAR-001..003   each shared limit, declared once
design/software-design.md             DES-001..003   the low-level design
src/charger.h                         the code: cites the design, declares nothing
src/charge_control.c
src/temp_monitor.c
tests/test_charge_control.c           TC-001..005    test cases, on the test functions
tests/test_temp_monitor.c             TC-006
verification/analyses.md              ANA-001        verification by analysis
verification/results/build-1.4.0.md   the test record: cites each test case it ran
plans/verification-plan.md            RULE-001..004  the trace criteria
```

Open [`grund.toml`](1-schema/repo/grund.toml) first. Then read
[`requirements/software.md`](1-schema/repo/requirements/software.md) for a
requirement's shape, and
[`tests/test_charge_control.c`](1-schema/repo/tests/test_charge_control.c) for a
test case declared where it runs. The tests compile with a C99 compiler and pass,
but `grund` only reads them.

A requirement is one heading, its EARS statement, and three attributes:

```markdown
## SWR-001: Open the charge switch above the temperature limit

When a valid temperature reading exceeds `45 °C` (§PAR-001), the charge-control
task shall open the charge switch.

### verification: Test

### rationale: Rationale

The charge switch is the only path for charge current, so opening it stops
charging.

### trace: Traces to

[§SYS-002](system.md#sys-002-stop-charging-above-the-temperature-limit)
```

A test case is declared on the function that runs it, and cites what it verifies:

```c
/**
 * TC-001: Charging stops one degree above the temperature limit
 *
 * Verifies §SWR-001 at its boundary: charging continues at the limit,
 * `45` (§PAR-001.1), and stops at 46.
 */
static void test_over_temperature_opens_switch(void)
```

## How the practice maps onto grund

| Practice | grund | In this repository |
|---|---|---|
| a level of the V, kept as one document | a kind whose home is one file, one declaration per H2 | `SYS` in `requirements/system.md` |
| a requirement identifier such as SYS-012 | the `{kind}-{number}` ID scheme ([§FS-config.3.2](../../docs/functional-spec/FS-config.md#32-id--id-grammar)), allocated by `grund id` ([§FS-id](../../docs/functional-spec/FS-id.md#fs-id-grund-proposes-ids-for-new-declarations)) | `SYS-002`, `SWR-001` |
| the requirement statement, in EARS wording | the declaration's lead, which `grund SWR-001` returns | "When …, the charge-control task shall …" |
| the verification method (29148) | a scalar field whose `one_of` is the four methods ([§FS-fields](../../docs/functional-spec/FS-fields.md#fs-fields-a-v2-kind-states-the-shape-of-its-declarations-and-check-holds-every-declaration-to-it)) | `### verification: Test` |
| the rationale | a required field | `### rationale: Rationale` |
| the parent link, "derived from" | a required `trace` field holding the citation, and a `must` citation direction up the V | `### trace: Traces to` |
| a limit the requirements, code and tests share | a value kind, and a binding at each use ([§FS-values](../../docs/functional-spec/FS-values.md#fs-values-opted-in-kinds-bind-authored-components-to-one-declared-value)) | `PAR-001` is `45 °C` |
| a design constant | a marked value inside the design element ([§FS-values.2.4](../../docs/functional-spec/FS-values.md#24-embedded-section-value-roots)) | `DES-001.1`, the valid ADC range |
| source code | a place: it cites and declares nothing, and every doc comment must cite ([§FS-check.3.6.2.2](../../docs/functional-spec/FS-check.md#3622-in-a-source-file)) | `src/` |
| a test case and its identifier | a kind declared in the test function's doc comment ([§FS-declarations.line.block-comment](../../docs/functional-spec/FS-declarations.md#lineblock-comment-a-line-inside-a-block-comment)) | `TC-001` |
| verification by analysis, and its verdict | a kind with a `Pass`/`Fail` scalar | `ANA-001` |
| a test record | a place whose files must cite the test cases they ran | `verification/results/` |
| "every requirement is realized and verified" | inbound rule sentences ([§FS-rules.3.4](../../docs/functional-spec/FS-rules.md#34-inbound-citation-count-and-prohibition)) | `RULE-004` |
| the traceability matrix | `grund refs --glob` as JSON ([§FS-refs.globs](../../docs/functional-spec/FS-refs.md#globs---glob-lists-who-cites-a-set), [§FS-refs.3.2](../../docs/functional-spec/FS-refs.md#32---format-json)) | run `3-matrix` |
| a hand-kept "Verified by" column | refused: a requirement must not cite down the V ([§FS-config-v2.rules.citations.default](../../docs/functional-spec/FS-config-v2.md#rulescitationsdefault-a-default-may-forbid-never-oblige)) | `default = "must-not"` on `SYS`, `SWR` |

C headers are read as C through one suffix mapping
([§FS-config-v2.schema.sources.extensions](../../docs/functional-spec/FS-config-v2.md#schemasourcesextensions-schemasourcesextensions-maps-a-suffix-to-a-language)).

## Step 1: what the trace is held to

```bash
grund schema examples/traceability/1-schema/repo
echo $?    # 0
```

`grund schema` reads the configuration alone ([§FS-schema](../../docs/functional-spec/FS-schema.md#fs-schema-grund-schema-shows-what-a-repository-is-held-to-read-from-its-configuration-alone)):

```text
charger-trace · grund.toml · v2

kinds and places
  STK      kind                  requirements/stakeholder-needs.md
  SYS      kind                  requirements/system.md
             field verification  must · first · one of Inspection, Analysis, Demonstration, Test
             field rationale     must · after verification
             field trace         must · after rationale
             closed must
  SWR      kind                  requirements/software.md
             field verification  must · first · one of Inspection, Analysis, Demonstration, Test
             field rationale     must · after verification
             field trace         must · after rationale
             closed must
  PAR      kind                  requirements/parameters.md
             field <number>      may · values
  DES      kind                  design/software-design.md
             field trace         must · last
  src      place                 src/
  TC       kind                  tests/
  ANA      kind                  verification/analyses.md
             field verdict       must · first · one of Pass, Fail
             field trace         must · after verdict
             closed must
  results  place                 verification/results/
  RULE     kind                  plans/verification-plan.md
  code     complement            every scanned file no other row claims

citations                        grounded
  STK      —
  SYS      must      STK
           may       SYS (itself)
           may       PAR
           must not  everything else
  SWR      must      SYS
           may       SWR (itself)
           may       PAR
           must not  everything else
  PAR      —
  DES      must      SWR
  src      must      DES         must · h2
  TC       must      SWR
  ANA      must      SWR
  results  must      TC          must · file
  RULE     —
  code     —

must = check error · warn = check warning · should = suggestion (check --suggestions)
may = allowed · must not / warn not / should not = forbidden at that strength
A|B = one obligation, met by citing either · — = no direction rule names it
```

Read it as the V:

- **The left side climbs.** A system requirement must cite a need, a software
  requirement a system requirement, a design element a software requirement,
  and the code the design. Each requirement names its parent in its `trace`
  field.
- **A requirement never cites down.** `SYS` and `SWR` must not cite anything
  but their parents, peers and parameters. The downward half of the matrix,
  "realized by" and "verified by", is never written by hand. `grund refs` derives
  it from the upward citations, so it cannot drift from them.
- **The right side verifies.** A test case or an analysis must cite the
  software requirement it verifies. A test record must cite the test cases it
  ran, and grounding `must · file` makes a record that cites nothing an error
  too: the direction alone judges only a file that cites something.
- **`src must DES`, grounded `must · h2`.** Every unindented doc comment in
  `src/` must cite something ([§FS-config-v2.rules.grounding](../../docs/functional-spec/FS-config-v2.md#rulesgrounding-grounding-is-a-ladder-replaced-whole)), and the file
  must cite the design. A documented function whose comment cites nothing is an
  error.
- **`closed must`** admits the three attributes and numbered sections, and no
  other named chapter. A requirement gains no attribute the schema does not
  list.

The table does not show the other direction. "Each SWR must be cited by at least
one TC or ANA" is a rule declaration in `plans/verification-plan.md`
([§FS-rules.1](../../docs/functional-spec/FS-rules.md#1-rule-declarations-and-opt-in)). It is written in Markdown, not in `grund.toml`, so `grund schema`
does not show it. The trace criteria live in two places.

## Step 2: the trace holds

```bash
grund check examples/traceability/2-check/repo
echo $?    # 0
```

```text
success
```

On this tree, `success` says the following:

- Every requirement has a verification method from the four, a rationale and a
  parent, in that order.
- Each level cites the one above it, and no requirement cites below itself.
- Each need is answered by a system requirement, and each system requirement is
  refined by a software requirement. Each software requirement is realized by a
  design element, and verified by a test case or an analysis. These are
  `RULE-001` to `RULE-004`.
- Every documented function in `src/` cites something, and every file there
  cites the design.
- Every bound limit equals its parameter. The temperature is checked in both
  requirements, the header and the test that crosses it.
- The test record names only test cases that exist.

## Step 3: the trace matrix

```bash
grund refs --glob 'SWR-*' examples/traceability/3-matrix/repo --format json
echo $?    # 0
```

The run prints 15 lines, one per citation of a software requirement. These are
the first two:

```text
{"path":"design/software-design.md","line":21,"column":2,"id":"SWR-003","section":null,"marker":true,"text":"§SWR-003","enclosing_declaration":"DES-001","enclosing_section":"trace"}
{"path":"design/software-design.md","line":50,"column":2,"id":"SWR-001","section":null,"marker":true,"text":"§SWR-001","enclosing_declaration":"DES-002","enclosing_section":"trace"}
```

Each line is one edge of the trace with both its ends: `id` is the requirement,
and `enclosing_declaration` is the design element, test case or analysis that
cites it. Pivoted by requirement, from inside the fixture, it is the downward
matrix:

```console
$ cd examples/traceability/3-matrix/repo
$ grund refs --glob 'SWR-*' --format json \
    | jq -rs 'group_by(.id)[] | "\(.[0].id)  \(map(.enclosing_declaration) | unique | join(" "))"'
SWR-001  DES-002 TC-001
SWR-002  ANA-001 DES-002
SWR-003  DES-001 TC-006
SWR-004  DES-002 TC-003
SWR-005  DES-002 TC-004
SWR-006  DES-002 TC-002 TC-003
SWR-007  DES-003 TC-005
```

The verification-method column is a selector over one field
([§FS-rules.8](../../docs/functional-spec/FS-rules.md#8-command-surfaces)):

```console
$ grund list --selector SWR.verification
SWR-001.verification  requirements/software.md:11  Test
SWR-002.verification  requirements/software.md:26  Analysis
SWR-003.verification  requirements/software.md:43  Test
SWR-004.verification  requirements/software.md:59  Test
SWR-005.verification  requirements/software.md:74  Test
SWR-006.verification  requirements/software.md:89  Test
SWR-007.verification  requirements/software.md:104  Test
```

Following one system requirement down to the code takes one `refs` per level:

```console
$ grund refs SYS-002
requirements/software.md:20: §SYS-002
requirements/software.md:36: §SYS-002
requirements/system.md:62: §SYS-002
$ grund refs SWR-001
design/software-design.md:50: §SWR-001
tests/test_charge_control.c:27: §SWR-001
$ grund refs DES-002 --summary
src/charge_control.c: 5 (lines 5, 11, 19, 33)
src/charger.h: 1 (line 2)
```

What an auditor still assembles by hand:

- **There is no matrix format.** The pivot above is `jq`, and splitting it into
  a design column and a verification column means matching ID prefixes. `grund`
  renders no report ([§FS-non-goals.5](../../docs/functional-spec/FS-non-goals.md#5-documentation-generation)). The JSON is the input to one.
- **A requirement nothing cites has no row.** `refs` lists citations, so an
  uncovered requirement is simply absent from the pivot. `grund check` reports
  it under `RULE-003` or `RULE-004`, but the matrix has to be joined with
  `grund list --kind SWR` to show the gap.
- **One hop per call, and the text form does not say who cites.**
  `requirements/system.md:62` is a mention of SYS-002 in SYS-004's rationale,
  not a refinement. Only the JSON's `enclosing_declaration` and
  `enclosing_section` tell the two apart.
- **No coverage counts per level.** `grund list --format json` carries `refs`,
  the number of citations of every kind together.
- **Code has no units.** A citation in `src/` belongs to no declaration, so the
  matrix names a file and a line, never a function
  ([§FS-non-goals.3](../../docs/functional-spec/FS-non-goals.md#3-code-ast-parsing)).

## Writing a requirement

Ask for the next number, and where it goes:

```console
$ grund id SWR "Stop charging below the minimum temperature" --explain
SWR-008
next: add the declaration to requirements/software.md  (H2: `## SWR-008: <one-line statement>`), then cite it as §SWR-008
```

Write the statement in one EARS pattern, then the three attributes in order:
`verification` with one of the four methods, `rationale`, and `trace` citing
the parent. Quote each limit as a binding, never as a bare number: a backticked
literal, one space, and the parenthesized citation of the parameter. The root
form compares the amount and the unit together ([§FS-values.3.1.2](../../docs/functional-spec/FS-values.md#312-a-binding-aimed-at-the-root)):

```markdown
When the cell temperature rises above `45 °C` (§PAR-001), the charger shall stop
the charge current within `500 ms` (§PAR-002).
```

In code and tests, a binding must sit on a comment line of its own. Put it
above the constant, not after it on the same line. Only a binding wholly inside
a comment line is compared ([§FS-values.3.2](../../docs/functional-spec/FS-values.md#32-recognized-text-contexts)):

```c
/* Over-temperature limit, whole degrees Celsius: `45` (§PAR-001.1) */
#define CHARGE_TEMP_MAX_C 45
```

Declare each test case on its test function, as `TC-001` above. List it in the
next test record. Then run `grund fmt --cross-refs --write` from the fixture
root, so each Markdown citation becomes a link ([§FS-fmt.6](../../docs/functional-spec/FS-fmt.md#6-cross-reference-emission)). Bindings keep
their bare form.

## Break it

Copy the fixture somewhere writable and make five edits that a document-based
workflow lets through. `grund check` refuses all five:

```bash
demo_root=$(mktemp -d)
cp -R examples/traceability/2-check/repo/. "$demo_root/"
cd "$demo_root"
```

1. The cell supplier revises its datasheet. In `requirements/parameters.md`,
   change PAR-001's `### 1. 45` to `### 1. 43`, and touch nothing else.
2. Keep the "Verified by" column a requirements template asks for, by
   appending one sentence to SWR-001's rationale:

   ```markdown
   Verified by §TC-001.
   ```

3. Give SWR-005 the verification method `### verification: Unit test`, a test
   level rather than a method.
4. A refactor of `tests/test_temp_monitor.c` replaces the TC-006 doc comment with
   a plain `/* … */` comment, so the test case disappears.
5. Add a bench-calibration helper to `src/charge_control.c` that no design
   element asks for:

   ```c
   /** Close the charge switch for bench calibration. */
   void charge_force_on(void)
   {
       state = CHARGING;
   }
   ```

```console
$ grund check
requirements/software.md:8: error: value mismatch for PAR-001.1: bound `45`, declared `43` at requirements/parameters.md:8
requirements/software.md:16: error: SWR must not cite TC (citation direction) — re-point the citation or downgrade it to a plain Markdown link
requirements/software.md:38: error: SWR-003 is cited by ANA or TC 0 times; RULE-004 requires at least one
requirements/software.md:74: error: SWR-005.verification is "Unit test"; kind SWR admits Inspection, Analysis, Demonstration, Test
requirements/system.md:23: error: value mismatch for PAR-001.1: bound `45`, declared `43` at requirements/parameters.md:8
src/charge_control.c:39: error: ungrounded doc-comment in kind home src/: no § citation to a declared ID
src/charger.h:11: error: value mismatch for PAR-001.1: bound `45`, declared `43` at requirements/parameters.md:8
tests/test_charge_control.c:28: error: value mismatch for PAR-001.1: bound `45`, declared `43` at requirements/parameters.md:8
verification/results/build-1.4.0.md:16: error: unknown reference TC-006; did you mean TC-001?
```

The new limit is reported at each of the four places that still say 45: both
requirements, the header, and the test that checks the boundary. The test's
`charge_control_step(46)` is not compared, only the comment above it. The
check says the test is stale, and the author updates its values by hand.

Take the advice to downgrade the back-link to a plain Markdown link and the
check passes, but the column is back, and unchecked. The repair here is to
delete it, because `grund refs SWR-001` already answers "verified by".

### What it does not catch

The downward half reaches as far as declarations, and no further. Drop the LED
variant's code, the `#ifdef CONFIG_STATUS_LED` block at the end of
`src/charge_control.c`, and the design element DES-003 has no code left:

```console
$ grund check
success
```

`src/charger.h` still mentions DES-003 in its doc comment, and any citation
counts. Remove that mention as well, and the gap is a warning that leaves the
exit code at 0 ([§FS-check.4.1](../../docs/functional-spec/FS-check.md#41-unused-declaration)):

```console
$ grund check
design/software-design.md:56: warning: declared but never cited: DES-003
```

A rule cannot close this. `Each DES must be cited by at least one src.` is
refused, because `src` is a place and declares no IDs, and an inbound count
counts citations from declarations only.

## Trade-offs

What the schema buys:

- **A trace that cannot go one-sided.** Each level's parent link is a checked
  citation, and the downward half is derived from those links. It is never kept
  as a second copy.
- **Attributes an author cannot drop or misspell.** Every requirement keeps a
  verification method from the four, a rationale and a parent, in a fixed order.
- **One number, checked everywhere it is quoted.** A changed limit is reported
  at each requirement, header and test still quoting the old one.
- **Gaps are failures.** An unrefined system requirement, a software requirement
  with no design or no test, a documented function that cites nothing, and a
  test record naming a deleted test each fail the check.

What it costs, and where `grund` forces a departure from common practice:

- **Attributes are headings.** A requirements tool shows them as columns, and a
  document template as a table row. Here each is an H3 heading under the
  requirement, and the scalar's value is the heading's title.
- **EARS wording is not checked.** The statement is free prose in the lead.
  `grund` cannot tell a WHEN clause from a WHILE clause, or catch a missing
  "shall".
- **The method is not tied to the evidence.** `RULE-004` accepts a test case or
  an analysis for any software requirement. A requirement marked `Test` that
  only an analysis cites still passes, because a rule cannot select the
  requirements whose `verification` is `Test`.
- **The downward rules are documents.** They live in `plans/`, not in
  `grund.toml`, so `grund schema` omits them. Each rule must also be cited once,
  which the plan's own list does, or it warns as never cited.
- **Code is a place.** Nothing requires every design element to have code, and a
  function with no doc comment is invisible to grounding.
- **Bindings sit beside the code, not in it.** The comment is compared, the
  `#define` is not, and a binding written after the code on the same line is not
  compared at all.
- **Citations are untyped.** "Derived from" and "verified by" differ only by
  where the citation sits: in a `trace` field, or in a test case's doc comment.
- **No suspect links.** When SYS-002's text changes, nothing marks SWR-001 for
  review. The check reads the tree as it stands, with no history
  ([§FS-non-goals.6](../../docs/functional-spec/FS-non-goals.md#6-decision-database-audit-log-history-tracking)).
- **Variants are invisible.** TC-005 runs only in a build that defines
  `CONFIG_STATUS_LED`, yet it counts for SWR-007 in every build.
