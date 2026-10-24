# OpenSpec: living specs and change deltas held to a grund schema

[OpenSpec](https://github.com/Fission-AI/OpenSpec) (v1.14.3) is a spec-driven
workflow for coding agents. A project keeps one living spec per capability in
`openspec/specs/<capability>/spec.md`: a `## Purpose`, then
`## Requirements`, where each `### Requirement:` states one behavior with SHALL
or MUST and each `#### Scenario:` under it gives one case of it. A change is
proposed in a folder of its own, `openspec/changes/<change>/`. The folder holds
a `proposal.md` (why, what changes, impact), a `tasks.md` checklist, an
optional `design.md`, and one spec delta per capability the change touches.
A delta's `ADDED`, `MODIFIED`, `REMOVED` and `RENAMED Requirements` sections
say how the living spec will differ. `openspec archive` applies the deltas and
moves the folder to `openspec/changes/archive/<date>-<change>/`.

`openspec validate` holds each file to that structure. What it does not hold
is the thread between them. A requirement is known only by its name. A task
list or a test names it in prose, if at all, and nothing notices when a renamed
requirement leaves a test pointing at a name that is gone.

This example is for a team that writes OpenSpec changes, or has an agent write
them, and wants `grund check` as the gate on that thread
([§FS-examples.2](../../docs/functional-spec/FS-examples.md#2-canonical-use-cases)). Its `grund.toml` says:

- each living spec is a declaration, and each requirement and scenario is a
  named section of it, so a test cites the scenario it proves;
- each delta is a declaration of its own, whose four operations are fields;
- every file of a change folder, every source file and every test must cite
  the requirement or delta it delivers;
- a living spec cites only other living specs, never work in flight.

The fixture is the circulation desk of a fictional library branch, Larkspur
Lending. It has two capabilities, `loans` and `notices`, and one change in
flight, `add-loan-holds`, which adds holds, changes renewals, and renames a
requirement. The example pins four runs ([§FS-examples.5.5](../../docs/functional-spec/FS-examples.md#55-an-example-may-pin-several-runs)): the schema, the
check, the check over four edits an agent plausibly makes, and the check after
the change is archived. None of them writes anything.

## The project

```text
grund.toml                          the schema
openspec/config.yaml                OpenSpec's project context; grund reads none of it
openspec/specs/loans/spec.md        SPEC-loans                  a living spec
openspec/specs/notices/spec.md      SPEC-notices                a living spec
openspec/changes/add-loan-holds/
  .openspec.yaml                    the change's metadata; grund reads none of it
  proposal.md                       cites what the change touches
  tasks.md                          each task cites what it delivers
  specs/loans/spec.md               DELTA-add-loan-holds-loans  the delta for loans
src/larkspur/loans.py, notices.py   cite the requirements they implement
tests/test_loans.py                 each test cites the scenario it proves
.grund/sections/*.lock              the section lock, which grund fmt --write keeps
```

Open [`grund.toml`](2-check/repo/grund.toml) first, then the delta,
[`specs/loans/spec.md`](2-check/repo/openspec/changes/add-loan-holds/specs/loans/spec.md).
Its sections are OpenSpec's operations, and its MODIFIED requirement cites the
living requirement it replaces.

### Where it departs from OpenSpec

`proposal.md` and `tasks.md` keep OpenSpec's headings exactly. They sit in a
place, and grund holds a place's files only to what they cite. The spec files
cannot stay as OpenSpec writes them. Each opens with a declaration heading,
`# SPEC-loans: Loans Specification`, and each heading under it carries a
handle: `## purpose: Purpose`, `### .renew-a-loan: Requirement: Renew a Loan`.

That costs OpenSpec's own tools. The following was read from OpenSpec's parser
at v1.14.3, not run. Its
[`findSection`](https://github.com/Fission-AI/OpenSpec/blob/v1.14.3/src/core/parsers/markdown-parser.ts#L140)
compares a heading's whole text with `Purpose`, so the parser that
`openspec validate` runs throws `Spec must have a Purpose section` on these
files. A requirement must match
[`/^###\s*Requirement:\s*(.+)\s*$/i`](https://github.com/Fission-AI/OpenSpec/blob/v1.14.3/src/core/parsers/requirement-blocks.ts#L38),
and a delta section
[`/^##\s+(ADDED|MODIFIED|REMOVED|RENAMED)\s+Requirements\s*$/i`](https://github.com/Fission-AI/OpenSpec/blob/v1.14.3/src/core/parsers/spec-structure.ts#L6).
So the spec files here hold OpenSpec's structure in grund's syntax, and
archiving is done by hand, as step 4 does it.

## How OpenSpec maps onto grund

| OpenSpec | grund | In this project |
|---|---|---|
| a living spec, `openspec/specs/<capability>/spec.md` | a declaration of kind `SPEC`, named after the capability | `openspec/specs/loans/spec.md` declares `SPEC-loans` |
| `## Purpose` and `## Requirements`, which validate requires | two `must` fields of a closed kind ([§FS-fields](../../docs/functional-spec/FS-fields.md#fs-fields-a-v2-kind-states-the-shape-of-its-declarations-and-check-holds-every-declaration-to-it)) | `## purpose: Purpose`, `## requirements: Requirements` |
| `### Requirement: <Name>` | a named section under `requirements`, written relative to it ([§FS-declarations.section.relative](../../docs/functional-spec/FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds)) | `### .renew-a-loan: Requirement: Renew a Loan` is `SPEC-loans.requirements.renew-a-loan` |
| `#### Scenario: <Name>` | a named section under its requirement | `SPEC-loans.requirements.renew-a-loan.third-renewal` |
| a change's `proposal.md`, `tasks.md`, `design.md` | files of a place, `citable = false`, that must cite | `tasks.md` cites the requirement each task delivers |
| a spec delta, `changes/<change>/specs/<capability>/spec.md` | a declaration of kind `DELTA`, named after the change and the capability | `DELTA-add-loan-holds-loans` |
| the four operation sections | four `may` fields of the closed `DELTA` kind | `## modified: MODIFIED Requirements` |
| a MODIFIED requirement, restated whole | a section of the delta that cites the living requirement it replaces | `DELTA-add-loan-holds-loans.modified.renew-a-loan` |
| RENAMED's `FROM:` and `TO:` lines | kept as written, in backticks, and the sentence under them cites the old requirement | `.borrow-a-book` becomes `.check-out-a-book` |
| `changes/archive/` | a place with `scan = false` | a citation into it resolves nowhere |
| `config.yaml`, `.openspec.yaml` | not read | |

## Step 1: what the project is held to

```bash
grund schema examples/openspec/1-schema/repo
echo $?    # 0
```

`grund schema` reads the configuration alone ([§FS-schema](../../docs/functional-spec/FS-schema.md#fs-schema-grund-schema-shows-what-a-repository-is-held-to-read-from-its-configuration-alone)):

```text
larkspur-lending · grund.toml · v2

kinds and places
  SPEC     kind                  openspec/specs/
             field purpose       must · first
             field requirements  must · after purpose
             closed must
  change   place                 openspec/changes/
  DELTA    kind                  openspec/changes/add-loan-holds/specs/
             field purpose       may · first
             field added         may
             field modified      may
             field removed       may
             field renamed       may
             closed must
  archive  place, not scanned    openspec/changes/archive/
  test     place                 tests/
  code     complement            every scanned file no other row claims

citations                        grounded
  SPEC     may       SPEC (itself)
           must not  everything else
  change   must      SPEC|DELTA  must · file
  DELTA    may       SPEC
           may       DELTA (itself)
           must not  everything else
  test     must      SPEC|DELTA  must · file
  code     must      SPEC|DELTA  must · file
  archive  not scanned: cites nothing and is never checked

must = check error · warn = check warning · should = suggestion (check --suggestions)
may = allowed · must not / warn not / should not = forbidden at that strength
A|B = one obligation, met by citing either · — = no direction rule names it
```

Read it as OpenSpec:

- **SPEC** holds what `openspec validate` requires, a Purpose and then the
  Requirements, and `closed` lets nothing else sit beside them.
- **DELTA** may carry any of the four operations. Its `purpose` comes first
  when present, because OpenSpec's instructions give a delta that creates a
  capability a `## Purpose` as its first section.
- **The DELTA place is one literal path.** A place is a path from the config
  root ([§FS-config-v2.schema.places](../../docs/functional-spec/FS-config-v2.md#schemaplaces-rows-and-their-places)), so each change in flight adds its
  `specs/` folder to `grund.toml`, and archiving it takes the line out again.
- **change, test and code** must each cite a requirement or a delta, and
  `grounded must · file` makes that every file rather than the place as a
  whole. A task list that names no requirement is an error.
- **SPEC must not** cite anything but a living spec. A living spec is the
  source of truth, so the work in flight cites it and never the other way
  round.
- **archive** is history. It is not scanned, so what an archived change cites
  is never checked again.

## Step 2: the change holds

```bash
grund check examples/openspec/2-check/repo
echo $?    # 0
```

```text
success
```

Before the change is archived, `grund refs` lists everything that leans on the
requirement it renames:

```bash
cd examples/openspec/2-check/repo
grund refs SPEC-loans.requirements.borrow-a-book --descendants --summary
```

```text
openspec/changes/add-loan-holds/proposal.md: 1 (line 16)
openspec/changes/add-loan-holds/specs/loans/spec.md: 1 (line 55)
openspec/changes/add-loan-holds/tasks.md: 1 (line 20)
openspec/specs/notices/spec.md: 1 (line 13)
src/larkspur/loans.py: 1 (line 31)
tests/test_loans.py: 2 (lines 11, 19)
```

`--descendants` counts the two tests that cite the requirement's scenarios
rather than the requirement. That is the blast radius of the rename, which the
proposal states and task 3.1 carries out.

## Step 3: four edits, three refused

`3-break/repo` is `2-check/repo` with four edits, and
`diff -ru examples/openspec/2-check/repo examples/openspec/3-break/repo` shows
them:

1. The Purpose of `openspec/specs/loans/spec.md` mentions holds and cites the
   delta's new requirement, `DELTA-add-loan-holds-loans.added.place-a-hold`.
2. In `tests/test_loans.py`, the test of a held copy cites the scenario at the
   living requirement, `SPEC-loans.requirements.renew-a-loan.held-copy`. The
   scenario exists only in the delta until the change is archived.
3. A second change is started: `openspec/changes/add-return-desk/tasks.md`,
   with one task and no citation.
4. A new capability, `openspec/specs/holds/spec.md`, is written in OpenSpec's
   own form:

   ```markdown
   # holds Specification

   ## Purpose

   Who gets a returned copy when more than one member is waiting for it.

   ## Requirements

   ### Requirement: Hold Queue Order
   ```

```bash
grund check examples/openspec/3-break/repo
echo $?    # 1
```

```text
openspec/changes/add-return-desk/tasks.md:1: error: ungrounded file in kind home openspec/changes/: no § citation to a declared ID
openspec/specs/loans/spec.md:7: error: SPEC must not cite DELTA (citation direction) — re-point the citation or downgrade it to a plain Markdown link
tests/test_loans.py:32: error: section not found: SPEC-loans.requirements.renew-a-loan.held-copy; write <§> before it to show the shape without citing it
```

The first three are refused. The fourth produces no line at all. A file whose
first heading declares no ID declares nothing, and nothing requires each file
of the SPEC place to declare. So the new capability sits outside the graph and
the check passes over it. Give it its declaration heading,
`# SPEC-holds: Holds Specification`, and the schema reaches it:

```text
openspec/specs/holds/spec.md:1: error: SPEC-holds has no purpose field; kind SPEC declares it
openspec/specs/holds/spec.md:1: error: SPEC-holds has no requirements field; kind SPEC declares it
openspec/specs/holds/spec.md:3: error: unmarked heading inside SPEC-holds; number it (## 1. Purpose) as SPEC-holds.1, declare an ID, or use a bold label; this became an error in grund 0.16.0
openspec/specs/holds/spec.md:7: error: unmarked heading inside SPEC-holds; number it (## 2. Requirements) as SPEC-holds.2, declare an ID, or use a bold label; this became an error in grund 0.16.0
openspec/specs/holds/spec.md:9: error: unmarked heading inside SPEC-holds; number it (### 2.1. Requirement: Hold Queue Order) as SPEC-holds.2.1, declare an ID, or use a bold label; this became an error in grund 0.16.0
openspec/specs/holds/spec.md:13: error: unmarked heading inside SPEC-holds; number it (#### 2.1.1. Scenario: Two holds) as SPEC-holds.2.1.1, declare an ID, or use a bold label; this became an error in grund 0.16.0
openspec/specs/holds/spec.md:1: warning: declared but never cited: SPEC-holds
```

Numbering the headings, as the unmarked-heading findings suggest
([§FS-declarations.checks.unmarked-heading](../../docs/functional-spec/FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading)), would leave both fields
missing. The repair is the field form, `## purpose: Purpose` and
`## requirements: Requirements`, with a handle on each requirement and
scenario, `### .hold-queue-order: Requirement: Hold Queue Order`.

## Step 4: the change is archived

`4-archived/repo` is `2-check/repo` after the archive that OpenSpec's CLI
cannot run here. The change folder moved, unchanged, to
`openspec/changes/archive/2026-10-22-add-loan-holds/`. The delta was merged
into `openspec/specs/loans/spec.md` in OpenSpec's order, renames first
([conventions spec](https://github.com/Fission-AI/OpenSpec/blob/v1.14.3/openspec/specs/openspec-conventions/spec.md#L178)):
`.borrow-a-book` became `.check-out-a-book`, the renewal requirement took
the delta's text and its held-copy scenario, and `.place-a-hold` was appended.
`grund.toml`, the code, the tests and the locks are as they were, so
`grund.toml` still names the delta folder that moved. That is not a finding.

```bash
grund check examples/openspec/4-archived/repo
echo $?    # 1
```

```text
openspec/specs/notices/spec.md:13: error: section not found: SPEC-loans.requirements.borrow-a-book; write <§> before it to show the shape without citing it
src/larkspur/loans.py:31: error: section not found: SPEC-loans.requirements.borrow-a-book; write <§> before it to show the shape without citing it
src/larkspur/loans.py:44: error: unknown reference DELTA-add-loan-holds-loans
src/larkspur/loans.py:54: error: unknown reference DELTA-add-loan-holds-loans
tests/test_loans.py:11: error: section not found: SPEC-loans.requirements.borrow-a-book.copy-on-shelf; write <§> before it to show the shape without citing it
tests/test_loans.py:19: error: section not found: SPEC-loans.requirements.borrow-a-book.copy-on-loan; write <§> before it to show the shape without citing it
tests/test_loans.py:25: error: unknown reference DELTA-add-loan-holds-loans
tests/test_loans.py:32: error: unknown reference DELTA-add-loan-holds-loans
```

Every site that leaned on the change is reported, in two ways:

- **The rename is "section not found".** The lock recorded the text of
  `.borrow-a-book`, and that text, byte for byte, is now `.check-out-a-book`.
  The lock's own move check reports a cited address whose text went elsewhere
  only while the old address still exists and holds other text
  ([§FS-section-lock.checks.section-address-changed](../../docs/functional-spec/FS-section-lock.md#checkssection-address-changed-a-cited-sections-text-sits-at-another-address)). This address is gone, so the
  finding does not say where the text went.
- **The delta is an unknown reference.** It now lives in the archive, which
  is not scanned. What it said is in the living spec, at the addresses the
  delta's sections would take there.

The archived `proposal.md` and `tasks.md` still cite `.borrow-a-book`. They
are history and nothing checks them.

The repair re-points the citations, then lets `grund fmt --write` record the
new addresses in the lock. On a copy:

```bash
cp -r examples/openspec/4-archived/repo /tmp/larkspur && cd /tmp/larkspur
sed -i -e 's/§SPEC-loans\.requirements\.borrow-a-book/§SPEC-loans.requirements.check-out-a-book/g' \
       -e 's/§DELTA-add-loan-holds-loans\.\(added\|modified\)\./§SPEC-loans.requirements./g' \
       src/larkspur/loans.py tests/test_loans.py openspec/specs/notices/spec.md
grund check .        # 7 errors: each new address is cited but not in the lock
grund fmt --write .
grund check .
```

`check` first reports each re-pointed citation as an address the lock does not
hold yet ([§FS-section-lock.checks.unrecorded-section-citation](../../docs/functional-spec/FS-section-lock.md#checksunrecorded-section-citation-a-locked-address-the-lock-does-not-hold)), for example:

```text
tests/test_loans.py:32: error: SPEC-loans.requirements.renew-a-loan.held-copy is cited but has no entry in .grund/sections/SPEC-loans.lock; run grund fmt --write with no path at the workspace root, or at the project root outside a workspace
```

`fmt --write` writes the locks and the one Markdown link whose anchor moved:

```text
rewrote 17 lines:
  .grund/sections/DELTA-add-loan-holds-loans.lock (5)
  .grund/sections/SPEC-loans.lock (10)
  .grund/sections/SPEC-notices.lock (1)
  openspec/specs/notices/spec.md (1)
```

The second `check` prints `success`. The new lock records
`SPEC-loans.requirements.check-out-a-book 601fa37ebac02171`, the fingerprint
the old lock held for `.borrow-a-book`: the text that was cited is the text
now cited.

## Writing a requirement

A requirement is a heading under `## requirements: Requirements` that writes
only its own handle, and a scenario is one under its requirement
([§FS-declarations.section.relative](../../docs/functional-spec/FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds)):

```markdown
### .renew-a-loan: Requirement: Renew a Loan

#### .third-renewal: Scenario: Third renewal refused
```

The handle is the coordinate and the title declares nothing
([§FS-config.3.3.1](../../docs/functional-spec/FS-config.md#331-named-components)). So retitling `Borrow a Book` to `Check Out a Book` while
keeping `.borrow-a-book` breaks no citation. On a copy of `2-check/repo`,
`grund check` still prints `success`, and `grund fmt --check` lists the four
Markdown links whose anchor follows the title:

```text
openspec/changes/add-loan-holds/proposal.md:16: markdown link
openspec/changes/add-loan-holds/specs/loans/spec.md:55: markdown link
openspec/changes/add-loan-holds/tasks.md:20: markdown link
openspec/specs/notices/spec.md:13: markdown link
```

`grund fmt --write` rewrites those four lines. Renaming the handle is a rename
of the address, and that is the RENAMED operation of step 4.

Write a citation bare, in a docstring or in Markdown:

```python
def renew(loan: Loan) -> None:
    """Move the due date, per §DELTA-add-loan-holds-loans.modified.renew-a-loan."""
```

Then run `grund fmt --write` from the project root. It writes each citation
in Markdown as a link to the heading ([§FS-fmt.6](../../docs/functional-spec/FS-fmt.md#6-cross-reference-emission)), and records every cited
relative section in `.grund/sections/` ([§FS-section-lock](../../docs/functional-spec/FS-section-lock.md#fs-section-lock-a-cited-relative-section-keeps-its-text-or-check-says-where-the-text-went)). Commit the lock with
the change.

A requirement reads on its own. From `2-check/repo`,
`grund SPEC-loans.requirements.renew-a-loan` prints:

```text
### .renew-a-loan: Requirement: Renew a Loan

The system SHALL move a loan's due date 21 days later when the member renews
it, and SHALL refuse a third renewal of the same loan.
```

`grund list --glob 'SPEC-*.requirements.*'` lists every living requirement:

```text
SPEC-loans.requirements.borrow-a-book     openspec/specs/loans/spec.md:10  Requirement: Borrow a Book
SPEC-loans.requirements.loan-limit        openspec/specs/loans/spec.md:28  Requirement: Loan Limit
SPEC-loans.requirements.renew-a-loan      openspec/specs/loans/spec.md:39  Requirement: Renew a Loan
SPEC-notices.requirements.due-reminder    openspec/specs/notices/spec.md:10  Requirement: Due Reminder
SPEC-notices.requirements.overdue-notice  openspec/specs/notices/spec.md:22  Requirement: Overdue Notice
```

A spec's lead is empty, because the declaration heading is followed at once by
`## purpose: Purpose`. So `grund SPEC-loans` prints nothing, and
`grund SPEC-loans --toc` is the read that shows the requirements and their
scenarios.

Before the first `grund init`, keep `[presentation] rules = "root"`
([§FS-config-v2.presentation](../../docs/functional-spec/FS-config-v2.md#presentation-the-presentation-concern)). Without it, `grund init --dry-run` here plans an
`AGENTS.md` in `openspec/specs/`, `openspec/changes/`, `tests/` and
`openspec/changes/add-loan-holds/specs/`, and the last would be archived with
the change. With `"root"`, it plans one `AGENTS.md`, beside `grund.toml`.

## Trade-offs

What the schema buys:

- **A requirement is an address.** A test cites the scenario it proves and a
  task the requirement it delivers, so `grund refs` gives the blast radius of a
  rename before the change is archived.
- **No file of a change goes unexplained.** A task list, a proposal, a source
  file or a test that cites no requirement is an error.
- **The living specs stay the source of truth.** A living spec that cites work
  in flight is refused, so nothing in `openspec/specs/` depends on a change
  that may never land.
- **Archiving cannot strand a citation quietly.** Every site that cited a
  renamed requirement or an archived delta is reported, and the lock carries
  the cited text across the rename.

What it costs:

- **The spec files are no longer OpenSpec's.** Declaration headings and
  handles put `openspec validate`, `openspec show` and `openspec archive` out
  of reach of the spec files, so the merge of step 4 is done by hand.
- **One line of `grund.toml` per change.** The DELTA place lists each change's
  `specs/` folder, and archiving a change leaves a line that names a folder
  that is gone.
- **A spec in OpenSpec's form is invisible.** Edit 4 of step 3 passes: a file
  with no declaration heading is not checked at all.
- **A rename reads as a missing section.** The check of step 4 does not say
  that the text of `.borrow-a-book` is now `.check-out-a-book`, though the lock
  holds what it needs to. Each citation is re-pointed by hand.
- **Coverage runs one way.** Every test must cite a requirement, but nothing
  says every scenario is proved by a test. A rule can count the citations a
  kind receives ([§FS-rules.3.4](../../docs/functional-spec/FS-rules.md#34-inbound-citation-count-and-prohibition)), but not per requirement, since a component
  wildcard is not yet accepted ([§FS-rules.2](../../docs/functional-spec/FS-rules.md#2-subject-selectors)), and not from a place:
  `grund check --rule 'The requirements chapter of each SPEC must be cited by at least one test.'`
  stops with `error: kind "test" declares no IDs — tests/ is not a citable home`
  and exit 2.
- **The YAML files are unread.** `config.yaml` carries the project's rules
  for agents, and `.openspec.yaml` the change's schema and date. grund checks
  neither.
