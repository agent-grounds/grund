# Spec Kit: spec-driven development held to a grund schema

[GitHub Spec Kit](https://github.com/github/spec-kit) (v1.2.2, released 2026-10-19) has a
coding agent write a project constitution, then for each feature a specification, an
implementation plan and a task list, each from a template. A spec numbers its functional
requirements `FR-001` on and its success criteria `SC-001` on, afresh in every spec, as bold
list items. A task is a checklist line, `- [ ] T004 [P] [US1] …`. The plan carries a
Constitution Check, a gate that must pass before research starts. What ties the files
together is checked by `/speckit.analyze`, a prompt the agent runs: it maps tasks to
requirements by "inference by keyword / explicit reference patterns", and only reports.

This example is for a team that works in Spec Kit and wants `grund check` as the gate. It sets
up a conformant repository whose `grund.toml` holds Spec Kit's own files to more than the
templates do, and cites the requirements from tasks, code and tests ([§FS-examples.2](../../docs/functional-spec/FS-examples.md#2-canonical-use-cases)):

- each constitution principle is a declaration, and every plan's Constitution Check must cite
  every one of them;
- each spec and each plan is a declaration whose template chapters are fields, in the
  template's order;
- each requirement, success criterion and user story is a named section of its spec;
- each task list is a place, and every phase of it must cite what it serves;
- every source file and every test cites the requirements it implements.

The fixture is Pixelbin, a small photo library with two features, so the two `FR-001`s are
real: `001-photo-upload` and `002-album-share`. The example pins three runs over the same
fixture ([§FS-examples.5.5](../../docs/functional-spec/FS-examples.md#55-an-example-may-pin-several-runs)): the schema, the check, and the citers of one requirement. None
writes anything.

## The repository

Spec Kit's layout is kept as it is: the constitution in `.specify/memory/`, one directory per
feature under `specs/`.

```text
grund.toml                       the schema
.specify/memory/constitution.md  ART-library-first … ART-privacy-by-default, one per principle
.specify/memory/rules.md         RULE-constitution-gate, the governance grund enforces
specs/001-photo-upload/
  spec.md                        SPEC-001-photo-upload
  plan.md                        PLAN-001-photo-upload
  tasks.md                       declares nothing; every phase cites what it serves
specs/002-album-share/
  spec.md                        SPEC-002-album-share, with a Clarifications session
  plan.md                        PLAN-002-album-share
  tasks.md
src/pixelbin/upload.py           cites the requirements of SPEC-001-photo-upload
src/pixelbin/share.py            cites the requirements of SPEC-002-album-share
tests/test_upload.py             cites the requirement each test proves
tests/test_share.py
```

Open [`grund.toml`](1-schema/repo/grund.toml) first, then
[`specs/001-photo-upload/spec.md`](1-schema/repo/specs/001-photo-upload/spec.md), the
Constitution Check of [`plan.md`](1-schema/repo/specs/001-photo-upload/plan.md) beside it, and
[`tasks.md`](1-schema/repo/specs/001-photo-upload/tasks.md).

## How Spec Kit maps onto grund

| Spec Kit | grund | In this repository |
|---|---|---|
| a principle, `### I. Library-First` | a declaration of kind `ART`, the ID written before the title | `### ART-library-first: I. Library-First` |
| `# Feature Specification: Photo upload` in `specs/001-photo-upload/` | a `SPEC` declaration whose number and slug are the directory's | `SPEC-001-photo-upload` |
| a template chapter, `## Requirements *(mandatory)*` | a field: its handle, then the template's title ([§FS-fields](../../docs/functional-spec/FS-fields.md#fs-fields-a-v2-kind-states-the-shape-of-its-declarations-and-check-holds-every-declaration-to-it)) | `## requirements: Requirements *(mandatory)*` |
| *(mandatory)*, and the template's order | `presence = "must"`, and each field `after` the one before | `grund schema` lists both |
| `- **FR-001**: System MUST …` | a named section whose title is the label, with the statement as its body ([§FS-config.3.3.1](../../docs/functional-spec/FS-config.md#331-named-components)) | `#### requirements.functional.fr-001: FR-001` |
| `FR-001` again in the next spec | another coordinate, because a section belongs to its declaration | `SPEC-002-album-share.requirements.functional.fr-001` |
| `**SC-001**`, `### User Story 1 - … (Priority: P1)` | named sections too | `success-criteria.outcomes.sc-001`, `user-scenarios.us1` |
| `## Clarifications` and `### Session 2026-10-09`, which `/speckit.clarify` adds | an optional field, first | `clarifications.session-2026-10-09` |
| the plan's `**Spec**: [link]` | a citation, which `grund fmt --write` writes as that link ([§FS-fmt.6](../../docs/functional-spec/FS-fmt.md#6-cross-reference-emission)) | `**Spec**: [<§>SPEC-001-photo-upload](spec.md#…)` |
| the plan's Constitution Check gate | a rule: that chapter cites each principle at least once ([§FS-rules.3.3](../../docs/functional-spec/FS-rules.md#33-per-target-coverage)) | `RULE-constitution-gate` |
| `tasks.md` | a place, which declares nothing, so its headings stay as the template writes them | `tasks`, grounded at `h3` |
| `- [X] T006 [US1] Implement …` | unchanged, with the requirement it serves cited at the end | `(<§>SPEC-001-photo-upload.requirements.functional.fr-001)` |
| a row of `/speckit.analyze`'s coverage table | `grund refs` on the requirement ([§FS-refs](../../docs/functional-spec/FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id)) | Step 3 |
| `**Status**: Accepted`, `**Feature Branch**: …` | lead prose; grund reads none of it | |

## Step 1: what the repository is held to

```bash
grund schema examples/spec-kit/1-schema/repo
echo $?    # 0
```

`grund schema` reads the configuration alone ([§FS-schema](../../docs/functional-spec/FS-schema.md#fs-schema-grund-schema-shows-what-a-repository-is-held-to-read-from-its-configuration-alone)):

```text
pixelbin · grund.toml · v2

kinds and places
  ART    kind                         .specify/memory/constitution.md
  RULE   kind                         .specify/memory/rules.md
  SPEC   kind                         specs/001-photo-upload/spec.md, specs/002-album-share/spec.md
           field clarifications       may · first
           field user-scenarios       must · after clarifications
           field requirements         must · after user-scenarios
           field success-criteria     must · after requirements
           field assumptions          may · after success-criteria
           closed must
  PLAN   kind                         specs/001-photo-upload/plan.md, specs/002-album-share/plan.md
           field summary              must · first
           field technical-context    must · after summary
           field constitution-check   must · after technical-context
           field project-structure    must · after constitution-check
           field complexity-tracking  may · after project-structure
           closed must
  tasks  place                        specs/001-photo-upload/tasks.md, specs/002-album-share/tasks.md
  test   place                        tests/
  code   complement                   src/

citations                             grounded
  ART    may       ART (itself)
         may       RULE
         must not  everything else
  RULE   —
  SPEC   may       SPEC (itself)
         may       ART
         must not  everything else
  PLAN   must      SPEC
  tasks  must      SPEC|PLAN          must · h3
  test   must      SPEC               must · file
  code   must      SPEC               must · file

must = check error · warn = check warning · should = suggestion (check --suggestions)
may = allowed · must not / warn not / should not = forbidden at that strength
A|B = one obligation, met by citing either · — = no direction rule names it
```

Read it as Spec Kit:

- **SPEC's fields** are the spec template's chapters, in its order. `/speckit.specify` is told
  to write the spec "preserving section order and headings"; here a missing, misplaced or
  retitled chapter is an error. `closed` admits those chapters and nothing else
  ([§FS-fields.declare.closed](../../docs/functional-spec/FS-fields.md#declareclosed-closed-is-a-strength-on-the-row)), and `/speckit.clarify` may add only `## Clarifications`, which is why
  that is the one optional field placed first.
- **SPEC must not** cite anything but a spec or a principle. A spec is written before its plan
  and names no technology, so it never cites the plan.
- **PLAN must** cite a spec. Its gate is not in this view: it is the rule sentence in
  `.specify/memory/rules.md`, which `grund check` evaluates and `grund schema` does not show,
  because rules are declarations rather than configuration.
- **tasks** is grounded at `h3`: every `##` phase and every `###` group of tests or
  implementation must cite something, and what it cites must include the spec or the plan
  ([§FS-check.3.6](../../docs/functional-spec/FS-check.md#36-ungrounded-unit-opt-in), [§FS-check.3.11.3](../../docs/functional-spec/FS-check.md#3113-the-unit-follows-grounding_level)). Setup tasks come from the plan ("per implementation plan"),
  so `PLAN` is the alternative.
- **test** and **code** must cite a requirement in every file. `code` lists `src`, so the scan
  reads the places above and `src/`, and nothing else ([§FS-config-v2.schema.places.complement](../../docs/functional-spec/FS-config-v2.md#schemaplacescomplement-code-is-the-complement-by-name)).

## Step 2: the repository holds

```bash
grund check examples/spec-kit/2-check/repo
echo $?    # 0
```

```text
success
```

Both plans' Constitution Checks name all four principles. Every phase of both task lists
cites the spec or the plan, every source and test file cites a requirement, and each of the
two `FR-001`s resolves in its own spec. The two specs and two plans carry the template's
chapters in its order, and no others.

## Step 3: who serves a requirement

```bash
grund refs SPEC-001-photo-upload.requirements.functional.fr-003 examples/spec-kit/3-refs/repo
echo $?    # 0
```

```text
specs/001-photo-upload/plan.md:43: §SPEC-001-photo-upload.requirements.functional.fr-003
specs/001-photo-upload/tasks.md:44: §SPEC-001-photo-upload.requirements.functional.fr-003
specs/001-photo-upload/tasks.md:51: §SPEC-001-photo-upload.requirements.functional.fr-003
src/pixelbin/upload.py:27: §SPEC-001-photo-upload.requirements.functional.fr-003
tests/test_upload.py:18: §SPEC-001-photo-upload.requirements.functional.fr-003
```

That is one row of `/speckit.analyze`'s Coverage Summary Table, for the requirement that
strips GPS tags: the plan's gate weighs it under Privacy by Default, tasks T005 and T007 serve
it, `strip_gps` implements it and a test proves it. `/speckit.analyze` has an agent infer that
mapping; `grund refs` reads it from the files, so two machines get the same rows
([§FS-refs](../../docs/functional-spec/FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id)). A glob asks for both `FR-001`s at once, and each site names its own spec:

```console
$ cd examples/spec-kit/3-refs/repo
$ grund refs --glob 'SPEC-*.requirements.functional.fr-001'
specs/001-photo-upload/tasks.md:42: §SPEC-001-photo-upload.requirements.functional.fr-001
specs/001-photo-upload/tasks.md:49: §SPEC-001-photo-upload.requirements.functional.fr-001
specs/002-album-share/tasks.md:25: §SPEC-002-album-share.requirements.functional.fr-001
specs/002-album-share/tasks.md:34: §SPEC-002-album-share.requirements.functional.fr-001
src/pixelbin/share.py:23: §SPEC-002-album-share.requirements.functional.fr-001
src/pixelbin/upload.py:34: §SPEC-001-photo-upload.requirements.functional.fr-001
tests/test_share.py:13: §SPEC-002-album-share.requirements.functional.fr-001
tests/test_upload.py:11: §SPEC-001-photo-upload.requirements.functional.fr-001
```

The same question about album share's `FR-004` shows a gap the check passes over: the plan and
two pending tasks cite it, and no test does yet.

```console
$ grund refs SPEC-002-album-share.requirements.functional.fr-004
specs/002-album-share/plan.md:41: §SPEC-002-album-share.requirements.functional.fr-004
specs/002-album-share/tasks.md:29: §SPEC-002-album-share.requirements.functional.fr-004
specs/002-album-share/tasks.md:38: §SPEC-002-album-share.requirements.functional.fr-004
```

## Writing a feature

Give a new feature its number the way Spec Kit does. `grund id` proposes the next free one
([§FS-id](../../docs/functional-spec/FS-id.md#fs-id-grund-proposes-ids-for-new-declarations)), the number `create-new-feature.sh` gives the directory:

```console
$ grund id SPEC "album export"
SPEC-003-album-export
```

Add the feature's `spec.md`, `plan.md` and `tasks.md` to the `SPEC`, `PLAN` and `tasks` rows
of `grund.toml`, one path each; a place names files and folders, never a pattern
([§FS-config-v2.schema.places](../../docs/functional-spec/FS-config-v2.md#schemaplaces-rows-and-their-places)).

Every heading in `spec.md` and `plan.md` carries its handle, `## requirements: Requirements
*(mandatory)*`, because a heading inside a declaration that is neither numbered nor named is
an error ([§FS-declarations.checks.unmarked-heading](../../docs/functional-spec/FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading)). A requirement is a heading of its own, titled
with its label, with the statement below it:

```console
$ grund SPEC-001-photo-upload.requirements.functional.fr-003
#### requirements.functional.fr-003: FR-003

System MUST remove every GPS tag from a photo before storing it, as §ART-privacy-by-default
requires.
```

Put the handles into the templates the commands copy. Spec Kit looks for a template in
`.specify/templates/overrides/` before anywhere else, so handled copies of `spec-template.md`
and `plan-template.md` there make every new spec and plan start in this form.

Write a citation bare, `<§>SPEC-001-photo-upload`, and run `grund fmt --write` from the project
root. Each citation in Markdown becomes a link to its heading, so the plan's `**Spec**: [link]`
is written for you; source comments keep the bare form. Before the first `grund init`, keep
`[presentation] rules = "root"`: without it, `grund init` writes an `AGENTS.md` into
`.specify/memory/`, `specs/`, `src/` and `tests/` ([§FS-init.2.3.5.11](../../docs/functional-spec/FS-init.md#23511-scoped-guidance-a-rows-guidance-lands-above-its-places)); with it, one at the
root.

## Break it

Copy the fixture somewhere writable and make five edits an agent plausibly makes. Spec Kit
accepts every one, and `grund check` refuses all five:

```bash
demo_root=$(mktemp -d)
cp -R examples/spec-kit/2-check/repo/. "$demo_root/"
cd "$demo_root"
```

1. Amend the constitution: in `.specify/memory/constitution.md`, add a fifth principle after
   the fourth.

   ```markdown
   ### ART-observability: V. Observability

   Every command logs one structured line per request to stderr.
   ```

2. Drop a requirement: in `specs/001-photo-upload/spec.md`, delete the
   `#### requirements.functional.fr-003: FR-003` heading and its statement.
3. Record a clarification in `specs/001-photo-upload/spec.md` the way `/speckit.clarify`
   writes it, just before the user scenarios:

   ```markdown
   ## Clarifications

   ### Session 2026-10-14

   - Q: Are HEIC photos accepted? → A: No; only JPEG and PNG.
   ```

4. Add a phase to `specs/002-album-share/tasks.md`, before its dependencies:

   ```markdown
   ## Phase 4: Polish & Cross-Cutting Concerns

   - [ ] T010 [P] Document `pixelbin share` and `pixelbin revoke` in README.md
   ```

5. Let the album share spec name its storage, in its Assumptions:

   ```markdown
   - Links are kept as §PLAN-002-album-share.technical-context stores them.
   ```

```console
$ grund check
specs/001-photo-upload/plan.md:32: error: PLAN-001-photo-upload.constitution-check cites ART-observability 0 times; RULE-constitution-gate requires at least one
specs/001-photo-upload/plan.md:43: error: section not found: SPEC-001-photo-upload.requirements.functional.fr-003; write <§> before it to show the shape without citing it
specs/001-photo-upload/spec.md:12: error: unmarked heading inside SPEC-001-photo-upload; number it (## 1. Clarifications) as SPEC-001-photo-upload.1, declare an ID, or use a bold label; this became an error in grund 0.16.0
specs/001-photo-upload/spec.md:14: error: unmarked heading inside SPEC-001-photo-upload; number it (### 1.1. Session 2026-10-14) as SPEC-001-photo-upload.1.1, declare an ID, or use a bold label; this became an error in grund 0.16.0
specs/001-photo-upload/tasks.md:44: error: section not found: SPEC-001-photo-upload.requirements.functional.fr-003; write <§> before it to show the shape without citing it
specs/001-photo-upload/tasks.md:51: error: section not found: SPEC-001-photo-upload.requirements.functional.fr-003; write <§> before it to show the shape without citing it
specs/002-album-share/plan.md:32: error: PLAN-002-album-share.constitution-check cites ART-observability 0 times; RULE-constitution-gate requires at least one
specs/002-album-share/spec.md:95: error: SPEC must not cite PLAN (citation direction) — re-point the citation or downgrade it to a plain Markdown link
specs/002-album-share/tasks.md:56: error: ungrounded section `## Phase 4: Polish & Cross-Cutting Concerns` in kind home specs/002-album-share/tasks.md: no § citation to a declared ID
src/pixelbin/upload.py:27: error: section not found: SPEC-001-photo-upload.requirements.functional.fr-003; write <§> before it to show the shape without citing it
tests/test_upload.py:18: error: section not found: SPEC-001-photo-upload.requirements.functional.fr-003; write <§> before it to show the shape without citing it
.specify/memory/constitution.md:27: warning: declared but never cited: ART-observability
```

- **The amendment** fails both existing plans' gates, not only the next plan written. Until
  each plan weighs the new principle, its gate does not pass.
- **The dropped requirement** names every site that served it: the plan's gate, two tasks,
  the code and the test ([§FS-check.3.2](../../docs/functional-spec/FS-check.md#32-missing-section)). The finding offers the escape, but here the repair is to
  delete those tasks, code and test, or point them at the requirement that replaced it.
- **The clarification** needs the field's form, `## clarifications: Clarifications` and
  `### clarifications.session-2026-10-14: Session 2026-10-14`. Numbering the heading, as the
  finding offers, leaves `user-scenarios` out of place instead:
  `SPEC-001-photo-upload.user-scenarios is section 2 of 5; kind SPEC places it first`.
- **The new phase** cites nothing. Every phase must say which requirement it serves.
- **The spec** names its plan, which it is written before ([§FS-check.3.12](../../docs/functional-spec/FS-check.md#312-forbidden-citation)).

## Trade-offs

What the schema buys:

- **The gate is checked.** Every plan weighs every principle, by citation. An amendment
  reaches the existing plans at the next check.
- **The templates' shape holds.** The mandatory chapters are there, in the template's order,
  and nothing else is added but a Clarifications chapter.
- **Traceability is a query, not an inference.** `grund refs` lists who serves a requirement,
  and `grund check` names every citer of a requirement that is dropped or renamed. Two
  machines give the same answer.
- **Per-spec numbering is safe.** `FR-001` is scoped by the spec that declares it, so the two
  never collide and a citation always says whose it is.

What it costs:

- **A requirement is a heading, not a list item.** grund declares a section only from a
  heading ([§FS-declarations.section](../../docs/functional-spec/FS-declarations.md#section-what-a-section-is)), so `- **FR-001**: …` cannot be cited and is rewritten as
  `#### requirements.functional.fr-001: FR-001`. Handles are lower-case, so the coordinate
  says `fr-001` and only the title keeps `FR-001`.
- **Every heading shows its handle.** A spec or plan renders `requirements: Requirements
  (mandatory)`, and a Spec Kit heading without one is an error, as edit 3 shows.
- **Citations are long.** A task in `tasks.md` names the spec of the requirement it serves,
  `SPEC-001-photo-upload.requirements.functional.fr-001`, where Spec Kit writes `FR-001` and
  leaves the feature implicit.
- **Three config lines per feature.** Each feature's three files are listed by hand, because a
  place takes no pattern. `files = ["specs/*/spec.md"]` loads as a path that names no file, so
  no spec is read, and this fixture then reports 61 lines, none of them about the place.
- **A task is not a unit.** Grounding and obligations reach headings, not list items, so the
  `h3` ladder holds each phase: a task that cites nothing passes when its phase cites
  something.
- **No requirement is owed a task or a test.** Principle III says "Every functional
  requirement has a test that cites it". A rule can count inbound citations ([§FS-rules.3.4](../../docs/functional-spec/FS-rules.md#34-inbound-citation-count-and-prohibition)):
  `SPEC-002-album-share.requirements.functional.fr-004 must be cited by at least one PLAN.`
  holds here, one sentence per requirement. Two things keep Principle III out of reach. The
  citer must be a kind, and tasks, tests and code are places. And a subject that ranges over
  every requirement of every spec is one of the rule language's phase-1 absences
  ([§FS-rules.12](../../docs/functional-spec/FS-rules.md#12-deliberate-phase-1-absences)). Album share's `FR-004` has no test, and `grund check` says `success`:

  ```console
  $ grund check --rule 'Each SPEC must be cited by at least one test.' --only-rule
  error: kind "test" declares no IDs — tests/ is not a citable home
  known kinds: ART, RULE, SPEC, PLAN
  $ grund check --rule 'Each chapter of each SPEC must be cited by at least one test.' --only-rule
  error: chapter-quantified subjects are not accepted in phase 1
  known kinds: ART, RULE, SPEC, PLAN
  ```

- **An accepted spec can still be unclear.** grund reads prose as opaque text
  ([§FS-non-goals.2](../../docs/functional-spec/FS-non-goals.md#2-spelling-grammar-prose-quality)), and no rule can read the `**Status**` line. In a fresh copy of the
  fixture, made as under [Break it](#break-it), replace `7 days` in album share's `FR-002` with
  `[NEEDS CLARIFICATION: how long does a link live?]`. The spec still says
  `**Status**: Accepted`, and the check passes:

  ```console
  $ grep -n 'NEEDS' specs/002-album-share/spec.md
  67:System MUST stop serving a share link [NEEDS CLARIFICATION: how long does a link live?] after it was created.
  $ grund check
  success
  ```

- **Metadata is unchecked.** `**Status**`, `**Feature Branch**` and the plan's
  `**Branch**` are lead prose. A `status` field could hold the status to a list of values, but
  only as a heading, `## status: Accepted`, and no rule could make the check depend on it.
- **A plan is not tied to its own spec.** `PLAN-001-photo-upload` must cite a spec, and nothing
  makes it `SPEC-001-photo-upload` but the number they share. A count is no help:
  `Each PLAN must cite exactly one SPEC.` counts citation sites, and each plan here has two.
- **Some files are not read.** The `research.md`, `data-model.md` and `contracts/` that
  `/speckit.plan` writes, and Spec Kit's own `.specify/templates/`, are in no place, so a
  citation there is not checked. List them in a row to check them.
