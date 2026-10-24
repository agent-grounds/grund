# Kiro specs held to a grund schema

[Kiro](https://kiro.dev/docs/specs/), AWS's agentic IDE, writes a spec per
feature into `.kiro/specs/<feature>/`. `requirements.md` holds user stories,
each with numbered acceptance criteria in EARS form (`WHEN … THE <system> SHALL
…`). `design.md` holds the architecture and the correctness properties to test.
`tasks.md` is a checkbox plan whose tasks name the criteria they serve, as
`_Requirements: 1.1, 2.3_`. Project-wide context lives in
[steering files](https://kiro.dev/docs/steering/) under `.kiro/steering/`.
Nothing checks those references. `1.1` is a position in one file: Kiro
renumbers it when it regenerates the requirements, and a task can name a
criterion that no longer exists.

This example is for a team that builds with Kiro, or has agents keep Kiro specs,
and wants `grund check` as the gate on that traceability. It sets up a Kiro
project held to a grund schema ([§FS-examples.2](../../docs/functional-spec/FS-examples.md#2-canonical-use-cases)):

- each spec file is a kind or a place, and the feature folder is the slug;
- each requirement and each acceptance criterion is a numbered section, so
  criterion `1.1` of the coupon spec is `REQ-coupon-codes.1.1`;
- each `_Requirements:` reference, and each test's `Validates:` tag, is a
  citation;
- design, tasks, code and tests must each cite the requirements.

The fixture is the checkout of Quillmarsh Pantry, a fictional shop. Its
coupon-codes spec is mostly built, with its code and tests. Its free-shipping
spec is planned but not started. Only the file shapes are Kiro's; Kiro is
proprietary, and every word of content was written for this example. The
example pins three runs ([§FS-examples.5.5](../../docs/functional-spec/FS-examples.md#55-an-example-may-pin-several-runs)): the schema, the check, and one
criterion traced through everything that cites it. None writes anything.

## The repository

```text
grund.toml                                 the schema
.grund/sections/*.lock                     the section lock: what each cited criterion says
.kiro/specs/coupon-codes/requirements.md   REQ-coupon-codes     two requirements, five criteria
.kiro/specs/coupon-codes/design.md         DESIGN-coupon-codes  Kiro's chapters, two properties
.kiro/specs/coupon-codes/tasks.md          tasks                the plan; tasks 3.2 and 4 are open
.kiro/specs/free-shipping/                 REQ-, DESIGN-free-shipping, and a plan not started
.kiro/steering/product.md                  STEER-product        no front matter: always included
.kiro/steering/tech.md                     STEER-tech           inclusion: always
.kiro/steering/structure.md                STEER-structure      pulls in grund.toml with #[[file:…]]
.kiro/steering/testing.md                  STEER-testing        inclusion: fileMatch, test/**/*.test.ts
src/coupons.ts                             code                 cites what each function builds
test/coupons.test.ts                       test                 each doc comment cites what it proves
```

Open [`grund.toml`](1-schema/repo/grund.toml) first. Then open the coupon spec's
[`requirements.md`](1-schema/repo/.kiro/specs/coupon-codes/requirements.md) and
[`tasks.md`](1-schema/repo/.kiro/specs/coupon-codes/tasks.md) side by side: every
`_Requirements:` line in the plan resolves to a criterion heading in the
requirements.

## How Kiro maps onto grund

| Kiro | grund | In this repository |
|---|---|---|
| a spec folder, `.kiro/specs/<feature>/` | the slug of that spec's IDs | `coupon-codes` gives `REQ-coupon-codes` and `DESIGN-coupon-codes` |
| `requirements.md` | kind `REQ`, its files listed one per spec ([§FS-config-v2.schema.places](../../docs/functional-spec/FS-config-v2.md#schemaplaces-rows-and-their-places)) | closed: numbered requirements and nothing else |
| `## Requirements` | the declaration heading; `# Requirements Document`, `## Introduction` and `## Glossary` stay above it as Kiro writes them | `## REQ-coupon-codes: Requirements` |
| `### Requirement 1: Apply a coupon code` | a numbered section ([§FS-declarations.section](../../docs/functional-spec/FS-declarations.md#section-what-a-section-is)) | `### 1. Apply a coupon code` is `REQ-coupon-codes.1` |
| criterion `1.` under `#### Acceptance Criteria` | a relative numbered heading, with the EARS sentence below it ([§FS-declarations.section.relative](../../docs/functional-spec/FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds)) | `#### .1. Active code` is `REQ-coupon-codes.1.1` |
| `_Requirements: 1.1, 2.1_` | a citation per number, which `grund fmt --write` writes as a link ([§FS-fmt.6](../../docs/functional-spec/FS-fmt.md#6-cross-reference-emission)) | `tasks.md` names `REQ-coupon-codes.1.1` and `REQ-coupon-codes.2.1` |
| `design.md` and its chapters | kind `DESIGN`, each Kiro chapter a field, in Kiro's order ([§FS-fields](../../docs/functional-spec/FS-fields.md#fs-fields-a-v2-kind-states-the-shape-of-its-declarations-and-check-holds-every-declaration-to-it)) | `## overview: Overview` comes first and `## testing: Testing Strategy` last |
| `### Property 1: …` with `**Validates: Requirements 1.1**` | a relative section of the `properties` field, citing its criteria | `### .1:` is `DESIGN-coupon-codes.properties.1` |
| `tasks.md` | the place `tasks`, which is the folder `.kiro/specs/`; a listed file outranks the folder that holds it ([§FS-config-v2.schema.places.ownership](../../docs/functional-spec/FS-config-v2.md#schemaplacesownership-a-listed-file-then-the-deepest-folder)) | every `tasks.md`, unchanged but for its references |
| a steering file | kind `STEER`, one declaration per file, below its front matter | `STEER-tech.money` |
| front matter, and `#[[file:…]]` references | text grund does not read | `testing.md`, `structure.md` |
| a test tagged `// Feature: …, Property 1` and `// **Validates: Requirements 1.1**` | a doc comment citing the property and its criteria ([§FS-check.3.6](../../docs/functional-spec/FS-check.md#36-ungrounded-unit-opt-in)) | `test/coupons.test.ts` |
| `AGENTS.md`, which Kiro loads as steering | the guidance `grund init` writes ([§FS-init.2.3.5.11](../../docs/functional-spec/FS-init.md#23511-scoped-guidance-a-rows-guidance-lands-above-its-places)) | written on demand; see [Writing a spec](#writing-a-spec) |

A task line keeps Kiro's shape with each number replaced by its citation, so
`_Requirements: 1.1, 2.1_` becomes:

```markdown
    - _Requirements: [§REQ-coupon-codes.1.1](requirements.md#1-active-code), [§REQ-coupon-codes.2.1](requirements.md#1-unknown-code)_
```

The citation spells Kiro's number inside it, so the plan still reads `1.1`.
Writing the bare number beside the citation as well would state one fact twice,
and grund checks only one copy.

## Step 1: what the specs are held to

```bash
grund schema examples/kiro/1-schema/repo
echo $?    # 0
```

`grund schema` reads the configuration alone ([§FS-schema](../../docs/functional-spec/FS-schema.md#fs-schema-grund-schema-shows-what-a-repository-is-held-to-read-from-its-configuration-alone)):

```text
kiro-checkout · grund.toml · v2

kinds and places
  REQ     kind                  .kiro/specs/coupon-codes/requirements.md, .kiro/specs/free-shipping/requirements.md
            closed must
  DESIGN  kind                  .kiro/specs/coupon-codes/design.md, .kiro/specs/free-shipping/design.md
            field overview      must · first
            field architecture  may · after overview
            field components    may · after architecture
            field data-models   may · after components
            field properties    may · after data-models
            field errors        may · after properties
            field testing       must · after errors
  tasks   place                 .kiro/specs/
  STEER   kind                  .kiro/steering/
  test    place                 test/
  code    complement            src/

citations                       grounded
  REQ     may       REQ (itself)
          may       STEER
          must not  everything else
  DESIGN  must      REQ
          may       DESIGN (itself)
          may       STEER
          must not  everything else
  tasks   must      REQ         must · file
          may       DESIGN
          must not  everything else
  STEER   may       STEER (itself)
          must not  everything else
  test    must      REQ         must · h3
  code    must      REQ|DESIGN  must · h2

must = check error · warn = check warning · should = suggestion (check --suggestions)
may = allowed · must not / warn not / should not = forbidden at that strength
A|B = one obligation, met by citing either · — = no direction rule names it
```

Read it as Kiro:

- **REQ** is the contract, so everything else cites it. It cites only another
  spec's requirements, as free-shipping's introduction cites the coupon
  discount, or the product steering. A prohibition reaches the whole file, the
  introduction above the declaration included. `closed must` keeps a
  requirements block to numbered requirements.
- **DESIGN must REQ**: a design says which requirements it meets. Its fields are
  Kiro's chapters, from Overview to Testing Strategy. The five between them are
  optional, because Kiro writes Correctness Properties only in the IDE.
- **tasks must REQ, grounded per file**: every plan names criteria, and may name
  the properties its property tests check.
- **STEER must not** cite anything but steering. Steering outlives every spec.
- **test must REQ · h3** holds every doc comment in a test file to cite a
  criterion, nested ones included, because the obligation's unit follows the
  grounding unit ([§FS-check.3.11.3](../../docs/functional-spec/FS-check.md#3113-the-unit-follows-grounding_level)). **code must REQ|DESIGN · h2** holds every
  top-level doc comment in `src/` to cite a criterion or a design point.

## Step 2: the specs hold

```bash
grund check examples/kiro/2-check/repo
echo $?    # 0
```

```text
success
```

The free-shipping spec has no code and an unstarted plan, and it passes: a
direction constrains what is written, not what is missing. Criterion 2.2, the
expired code, also passes. Its task is still open, so it has a design paragraph
and a task but no code and no test. [Is every criterion covered?](#is-every-criterion-covered)
shows how to find that.

## Step 3: one criterion, traced

```bash
grund refs REQ-coupon-codes.1.2 examples/kiro/3-refs/repo
echo $?    # 0
```

```text
.kiro/specs/coupon-codes/design.md:27: §REQ-coupon-codes.1.2
.kiro/specs/coupon-codes/design.md:45: §REQ-coupon-codes.1.2
.kiro/specs/coupon-codes/tasks.md:21: §REQ-coupon-codes.1.2
.kiro/specs/coupon-codes/tasks.md:24: §REQ-coupon-codes.1.2
src/coupons.ts:24: §REQ-coupon-codes.1.2
test/coupons.test.ts:15: §REQ-coupon-codes.1.2
test/coupons.test.ts:31: §REQ-coupon-codes.1.2
```

This is the traceability row for the floor-at-zero criterion ([§FS-refs](../../docs/functional-spec/FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id)). It
lists the design paragraph and the property that meet the criterion, the task
that built it, the task that tested it, the function, and the two tests that
prove it. Kiro's IDE shows a property's requirement on hover; `grund refs` gives
the whole row for any criterion, from any terminal. Run it before you change a
criterion's wording.

## Writing a spec

### Acceptance criteria are headings

grund addresses sections, and a section is a heading ([§FS-declarations.section](../../docs/functional-spec/FS-declarations.md#section-what-a-section-is)),
never a list item. So each criterion becomes a heading. It is written relative,
`#### .1. Active code`, so its number reads like Kiro's list number, and its
full address is `REQ-coupon-codes.1.1`. A cited relative section is recorded in
the section lock, `.grund/sections/REQ-coupon-codes.lock`, with a fingerprint of
its text ([§FS-section-lock](../../docs/functional-spec/FS-section-lock.md#fs-section-lock-a-cited-relative-section-keeps-its-text-or-check-says-where-the-text-went)). The lock is what catches Kiro renumbering criteria
(edit 1 under [Break it](#break-it)).

The EARS sentence goes below a short name, not into the heading, because the
fingerprint is the text below the heading line ([§FS-section-lock.holds.fingerprint](../../docs/functional-spec/FS-section-lock.md#holdsfingerprint-the-fingerprint)).
Put the whole criterion into the heading instead, as a one-line list item would
be, and every criterion's text is empty. They all match, so the lock records
none of them ([§FS-section-lock.holds.repeated](../../docs/functional-spec/FS-section-lock.md#holdsrepeated-a-text-the-declaration-repeats)):

```console
$ grund fmt --write
$ cat .grund/sections/REQ-coupon-codes.lock
REQ-coupon-codes.1.1 -
REQ-coupon-codes.1.2 -
REQ-coupon-codes.1.3 -
REQ-coupon-codes.2.1 -
REQ-coupon-codes.2.2 -
```

Then insert a criterion above `.1.` and renumber, and `grund check` still prints
`success`.

### Citing a criterion

Write the citation bare and let `grund fmt --write` finish it. Add a third
requirement in the criterion form, and the task that serves it:

```markdown
### 3. One coupon per order

**User Story:** As the shop's operator, I want one coupon per order, so that
discounts never stack.

#### .1. Second code

WHILE an order carries a coupon, THE Checkout SHALL refuse a second code and
keep the first.
```

```markdown
  - [ ] 3.3 Refuse a second code on an order
    - _Requirements: §REQ-coupon-codes.3.1_
```

```console
$ grund check
.kiro/specs/coupon-codes/tasks.md:32: error: REQ-coupon-codes.3.1 is cited but has no entry in .grund/sections/REQ-coupon-codes.lock; run grund fmt --write with no path at the workspace root, or at the project root outside a workspace
$ grund fmt --write
rewrote 2 lines:
  .grund/sections/REQ-coupon-codes.lock (1)
  .kiro/specs/coupon-codes/tasks.md (1)
$ grund check
success
```

`fmt` recorded the new criterion in the lock
([§FS-section-lock.checks.unrecorded-section-citation](../../docs/functional-spec/FS-section-lock.md#checksunrecorded-section-citation-a-locked-address-the-lock-does-not-hold)) and turned the citation
into a link to the criterion's heading, which Kiro's Markdown preview follows:
`_Requirements: [<§>REQ-coupon-codes.3.1](requirements.md#1-second-code)_`.
Source comments keep the bare form. Run `fmt` at the project root, because it
writes the lock only there.

### Kiro reads grund's guidance

`grund init` writes the schema's rules as agent guidance. One `AGENTS.md` goes
at the root, and one beside each row's places: `.kiro/specs/AGENTS.md`,
`.kiro/steering/AGENTS.md`, `src/AGENTS.md` and `test/AGENTS.md`
([§FS-init.2.3.5.11](../../docs/functional-spec/FS-init.md#23511-scoped-guidance-a-rows-guidance-lands-above-its-places)). Kiro loads all five as always-included steering. It reads
an `AGENTS.md` at the root or in any subdirectory, and every Markdown file in
`.kiro/steering/`. So Kiro's agent reads the citation rules before it writes a
task. `structure.md` also pulls `grund.toml` itself into Kiro's context, with
`#[[file:grund.toml]]`. The fixture leaves the generated files out.

## Is every criterion covered?

Kiro promises traceability: every criterion has a task, and tests prove it.
grund holds the outbound half: every plan, every test doc comment and every
top-level code comment cites a criterion. It cannot yet hold the inbound half,
that every criterion is cited by a task and by a test.

An inbound count is a rule sentence, `<subject> must be cited by at least one
<KIND>.` (§FS-rules.3.4). Its subject is a declaration or a named chapter
([§FS-rules.2](../../docs/functional-spec/FS-rules.md#2-subject-selectors)). So a rule can require each REQ to be cited by a design, which
passes here. It cannot require that of each criterion. From
`examples/kiro/2-check/repo`:

```console
$ grund check --rule 'Each REQ must be cited by at least one DESIGN.' --only-rule
success
$ grund check --rule 'REQ-coupon-codes.2.2 must be cited by at least one DESIGN.' --only-rule
error: numbered chapter subjects can detach when headings move; accepted form: REQ-coupon-codes must be cited by at least one DESIGN.
```

The accepted form counts a citation of any criterion of the spec, so it passes
while 2.2 has no citation at all. And an inbound count counts only citations
made from a declaration. Tests, plans and code declare nothing, so none of them
can be the source:

```console
$ grund check --rule 'Each REQ must be cited by at least one test.' --only-rule
error: kind "test" declares no IDs — test/ is not a citable home
known kinds: REQ, DESIGN, STEER
```

The unused-declaration warning does not reach a criterion either. It is about
declarations ([§FS-check.4.1](../../docs/functional-spec/FS-check.md#41-unused-declaration)), so an uncited criterion of a cited spec earns
none.

The closest grund comes is one question per criterion. `grund list --glob`
names every criterion ([§FS-coordinate-globs](../../docs/functional-spec/FS-coordinate-globs.md#fs-coordinate-globs-a-coordinate-glob-selects-catalog-coordinates-by-pattern)), and `grund refs` with a path
counts only the citations in that tree ([§FS-scan.path.refs](../../docs/functional-spec/FS-scan.md#pathrefs-refs-scans-the-paths-tree)):

```console
$ for c in $(grund list --glob 'REQ-*.*.*' | cut -d' ' -f1); do
>   printf '%s: ' "$c"; grund refs "$c" test --total 2>/dev/null
> done
REQ-coupon-codes.1.1: cited at 2 sites across 1 file
REQ-coupon-codes.1.2: cited at 2 sites across 1 file
REQ-coupon-codes.1.3: cited at 1 site across 1 file
REQ-coupon-codes.2.1: cited at 1 site across 1 file
REQ-coupon-codes.2.2: not cited
REQ-free-shipping.1.1: not cited
REQ-free-shipping.1.2: not cited
```

The `2>/dev/null` drops a note `refs` writes for each uncited criterion. Here,
the three criteria no test cites are the ones whose tasks are still open, but
`grund check` passes either way. To require coverage today, every criterion must
be a declaration of its own, such as `#### AC-coupon-codes-1-1: Active code`. A
test then declares an ID in its doc comment, from a folder that may declare, so
that `Each AC must be cited by at least one TEST.` has a source to count. That
gives up Kiro's numbering, and every test ID earns `declared but never cited`.

## Break it

Copy the fixture somewhere writable and make four edits that Kiro's own
workflow makes. Kiro accepts all four, and `grund check` refuses all four:

```bash
demo_root=$(mktemp -d)
cp -R examples/kiro/2-check/repo/. "$demo_root/"
cd "$demo_root"
```

1. Kiro regenerates `.kiro/specs/coupon-codes/requirements.md` with a new first
   criterion. Insert above `#### .1. Active code`:

   ```markdown
   #### .1. Code field

   THE Checkout SHALL show one field for a coupon code beside the subtotal.

   ```

   Then renumber the three criteria below it `.2.`, `.3.` and `.4.`, as Kiro
   renumbers its list.
2. Append a third requirement to the same file in Kiro's own shape:

   ```markdown
   ### 3. One coupon per order

   **User Story:** As the shop's operator, I want one coupon per order, so that
   discounts never stack.

   #### Acceptance Criteria

   1. WHILE an order carries a coupon, THE Checkout SHALL refuse a second code and
      keep the first.
   ```

3. Add the task that Kiro's Sync Files adds for it, under task 3.2 of
   `tasks.md`:

   ```markdown
     - [ ] 3.3 Refuse a second code on an order
       - _Requirements: §REQ-coupon-codes.3.1_
   ```

4. Start a gift-cards spec without touching `grund.toml`. Create
   `.kiro/specs/gift-cards/requirements.md`:

   ```markdown
   # Requirements Document

   ## Introduction

   Shoppers can pay for part of an order with a gift card.

   ## REQ-gift-cards: Requirements

   ### 1. Redeem a gift card

   **User Story:** As a shopper, I want to pay with a gift card, so that I can
   spend a gift I was given.

   #### .1. Balance

   WHEN a shopper redeems a gift card, THE Checkout SHALL take its balance off the
   subtotal, never below zero.
   ```

```console
$ grund check
.kiro/specs/coupon-codes/requirements.md:23: error: REQ-coupon-codes.1.1 is cited, and what .grund/sections/REQ-coupon-codes.lock recorded for it now sits at REQ-coupon-codes.1.2; move it back, cite REQ-coupon-codes.1.2 instead, or leave a pointer to it at REQ-coupon-codes.1.1; if REQ-coupon-codes.1.2 is a copy, edit the copy or cite it from REQ-coupon-codes.1.1
.kiro/specs/coupon-codes/requirements.md:27: error: REQ-coupon-codes.1.2 is cited, and what .grund/sections/REQ-coupon-codes.lock recorded for it now sits at REQ-coupon-codes.1.3; move it back, cite REQ-coupon-codes.1.3 instead, or leave a pointer to it at REQ-coupon-codes.1.2; if REQ-coupon-codes.1.3 is a copy, edit the copy or cite it from REQ-coupon-codes.1.2
.kiro/specs/coupon-codes/requirements.md:32: error: REQ-coupon-codes.1.3 is cited, and what .grund/sections/REQ-coupon-codes.lock recorded for it now sits at REQ-coupon-codes.1.4; move it back, cite REQ-coupon-codes.1.4 instead, or leave a pointer to it at REQ-coupon-codes.1.3; if REQ-coupon-codes.1.4 is a copy, edit the copy or cite it from REQ-coupon-codes.1.3
.kiro/specs/coupon-codes/requirements.md:63: error: unmarked heading inside REQ-coupon-codes; number it (#### 3.1. Acceptance Criteria) as REQ-coupon-codes.3.1, declare an ID, or use a bold label; this became an error in grund 0.16.0
.kiro/specs/coupon-codes/tasks.md:32: error: missing section REQ-coupon-codes.3.1
.kiro/specs/gift-cards/requirements.md:1: error: ungrounded file in kind home .kiro/specs/: no § citation to a declared ID
.kiro/specs/gift-cards/requirements.md:7: error: REQ-gift-cards must be declared in .kiro/specs/coupon-codes/requirements.md, .kiro/specs/free-shipping/requirements.md (single-file kind)
.kiro/specs/gift-cards/requirements.md:7: warning: declared but never cited: REQ-gift-cards
```

- **Edit 1** is the first three lines. Every task, test and design line that
  meant the active-code criterion now names the code field, and Kiro reads it
  that way without a word ([§FS-section-lock.checks.section-address-changed](../../docs/functional-spec/FS-section-lock.md#checkssection-address-changed-a-cited-sections-text-sits-at-another-address)).
  The repair is to cite the new addresses: change 1.3 to 1.4 first, then 1.2 to
  1.3, then 1.1 to 1.2, and run `grund fmt --write`.
- **Edits 2 and 3** are the next two lines. Kiro's `#### Acceptance Criteria`
  heading is not a section, so 3.1 does not exist and the task's citation
  fails. The repair is the criterion's form, `#### .1. Second code` with the
  sentence below it. Do not number the heading as the finding offers: with
  `#### 3.1. Acceptance Criteria` the task's citation resolves to the heading
  over the whole list, and the check passes.
- **Edit 4** is the last three lines. A version-2 place is a path, not a
  pattern, so the new spec's file is not REQ's until `grund.toml` lists it.
  Until then it falls to the `tasks` place, which must cite a REQ. The repair is
  one more entry in REQ's `files`, and one in DESIGN's for its design. The
  misplaced-declaration finding names the other specs' files instead, which is
  not where this declaration belongs ([§FS-declarations.checks.misplaced-declaration](../../docs/functional-spec/FS-declarations.md#checksmisplaced-declaration-misplaced-declaration-configured-kind-home)).

## Trade-offs

What the schema buys:

- **Criterion references are checked.** A task, design paragraph, function or
  test that names a criterion names one that exists, in the right spec. Kiro's
  `1.1` means something different in every spec; `REQ-coupon-codes.1.1` names
  one criterion, and fails once it is gone.
- **A renumbering is caught.** The lock reports a cited criterion whose text has
  moved to another number. That is the edit Kiro makes most often and checks
  least.
- **Every link is queryable.** `grund refs` gives any criterion's traceability
  row, and `grund schema` prints the rules. `grund init` writes those rules
  where Kiro loads steering.
- **Steering is citable.** A design names the convention it follows, such as
  `STEER-tech.money`, and steering never comes to depend on one spec.

What it costs:

- **Criteria are headings.** Kiro writes them as a numbered list under
  `#### Acceptance Criteria`. Here each is a `#### .N. <name>` heading with the
  EARS sentence below it, and the name is one Kiro does not write. When Kiro
  regenerates a requirements file it writes its list back, and grund refuses it
  (edit 2). Edit requirements by hand, or reshape what Kiro regenerates.
- **Kiro's labels go.** `### Requirement 1: Apply a coupon code` becomes
  `### 1. Apply a coupon code`, and `### Property 1: …` becomes `### .1: …`.
  The number is the address, so it is written once.
- **The schema lists every spec.** A place is a path, so each new spec adds its
  `requirements.md` and `design.md` to `grund.toml`. The `tasks` place over
  `.kiro/specs/` turns a forgotten one into an error rather than a silent gap.
  For the same reason the tests sit in `test/` rather than beside the code as
  `*.test.ts`.
- **Coverage runs one way.** grund holds every plan, test and function to cite
  a criterion, but not every criterion to be cited.
- **A bare number is prose.** A task that Kiro writes as `_Requirements: 2.4_`
  cites nothing. The file is still grounded by its other citations, so
  `grund check` passes it. Review a Sync Files diff for bare numbers.
- **A test is held only through its doc comment.** grund parses no code
  ([§FS-non-goals.3](../../docs/functional-spec/FS-non-goals.md#3-code-ast-parsing)), so an `it(...)` with no doc comment is no unit. Kiro's own
  `//` tags are line comments, which grund does not read as documentation, so
  only a `/** … */` block is held to cite.
- **Front matter and file references are invisible.** grund reads no
  `inclusion` or `fileMatchPattern`, so a pattern that matches no file goes
  unnoticed. `#[[file:grund.toml]]` is plain text, so a moved file breaks Kiro's
  reference and grund says nothing.
- **Hidden directories are read only as places.** grund never descends into a
  hidden directory on its own ([§FS-scan.scope.hidden](../../docs/functional-spec/FS-scan.md#scopehidden-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent)). `.kiro/` is read here
  because the rows name places inside it ([§FS-scan.scope.roots](../../docs/functional-spec/FS-scan.md#scoperoots-a-scan-root-outruns-every-rule-about-descent)). Keep every
  Kiro file grund should see inside a place.
- **A requirements declaration has no lead.** Kiro writes nothing between
  `## Requirements` and the first requirement, so `grund REQ-coupon-codes`
  prints nothing. Read a spec with `--toc`, and a criterion by its address.
