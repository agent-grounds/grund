# ADRs and arc42: decision records held to a grund schema

An architecture decision record (ADR) is Michael Nygard's 2011 proposal in
[*Documenting Architecture Decisions*](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions):
one short file per decision, with a title, Context, Decision, Status and
Consequences, numbered "sequentially and monotonically". A reversed decision
is kept and marked superseded, "with a reference to its replacement".
[adr-tools](https://github.com/npryce/adr-tools) writes such files as
`0001-record-architecture-decisions.md`, opens each with `# 1. Record
architecture decisions`, and records a supersession as a `Superceded by` link in
the old record's `## Status` section. [MADR](https://github.com/adr/madr) 4.0.0
moves the status, date and decision-makers into YAML front matter.
[arc42](https://arc42.org) is a twelve-chapter template for architecture
documentation, and its chapter 9, Architecture Decisions, is where the
decisions are listed. Its multi-page Markdown edition keeps one chapter per
file, from `01_introduction_and_goals.md` to `12_glossary.md`.

None of these formats checks itself. A record can lose its status, a
superseded record can point at a successor that was renumbered, chapter 9 can
miss a record, and a quality scenario can lose the goal it was written for.

This example is for a team that keeps ADRs beside an arc42 document and wants
`grund check` as the gate. It sets up a conformant repository
([§FS-examples.2](../../docs/functional-spec/FS-examples.md#2-canonical-use-cases)) whose `grund.toml` holds both formats to rules of their own:

- an ADR is a closed kind: a status from Nygard's four, then Context, Decision
  and Consequences, in adr-tools' order;
- arc42 chapter 9 is the ADR kind's index, so it lists every record;
- the quality goals of arc42 section 1.2 and the scenarios of section 10.2 are
  kinds, and each scenario must cite the goal it makes measurable;
- every source file must cite the decision it implements.

The fixture is Carrel, a seat-booking service for the reading rooms of a
fictional university library. Its pilot kept bookings in SQLite (ADR-0002); a
second instance needed a shared database, so ADR-0004 moved them to PostgreSQL
and superseded it. The example pins three runs over the same fixture
([§FS-examples.5.5](../../docs/functional-spec/FS-examples.md#55-an-example-may-pin-several-runs)): the schema, the check, and the decision log. None writes
anything.

## The repository

```text
grund.toml                                 the schema
docs/adr/                                  one record per file, named as adr-tools names them
  0001-record-architecture-decisions.md    ADR-0001  Accepted
  0002-store-bookings-in-sqlite.md         ADR-0002  Superseded, by ADR-0004
  0003-release-unclaimed-seats.md          ADR-0003  Accepted
  0004-store-bookings-in-postgresql.md     ADR-0004  Accepted
  0005-claim-seats-from-the-phone.md       ADR-0005  Proposed
docs/arc42/                                arc42's multi-page Markdown edition, all twelve chapters
  01_introduction_and_goals.md             QG-1 to QG-3, under Quality Goals
  02_architecture_constraints.md … 08_concepts.md
  09_architecture_decisions.md             the ADR kind's index
  10_quality_requirements.md               QS-1 to QS-4, under Quality Scenarios
  11_technical_risks.md, 12_glossary.md
src/carrel/                                bookings.py, release.py, migrate_sqlite.py
```

Open [`grund.toml`](1-schema/repo/grund.toml) first, then the superseded
record, [`0002-store-bookings-in-sqlite.md`](1-schema/repo/docs/adr/0002-store-bookings-in-sqlite.md),
and [chapter 9](1-schema/repo/docs/arc42/09_architecture_decisions.md).

## How the formats map onto grund

| Format | grund | In this repository |
|---|---|---|
| an ADR file, `NNNN-slug.md` | a declaration of kind `ADR`, with `id_format = "{kind}-{number}"`; the file name is free | `0002-store-bookings-in-sqlite.md` declares `ADR-0002` |
| the record's title line | the declaration heading; the authored digits are kept ([§FS-config.3.2.8](../../docs/functional-spec/FS-config.md#328-numeric-spelling-survives-reading-and-reporting)) | `# ADR-0002: Store bookings in SQLite` |
| the `Date:` line | lead prose, which grund does not read | `Date: 2026-01-19` |
| the Status section | a scalar field, whose heading is its value ([§FS-fields.content.one-of](../../docs/functional-spec/FS-fields.md#contentone-of-a-scalars-value-is-its-display-name)) | `## status: Superseded` |
| Context, Decision, Consequences | fields chained with `after`, in a closed kind ([§FS-fields](../../docs/functional-spec/FS-fields.md#fs-fields-a-v2-kind-states-the-shape-of-its-declarations-and-check-holds-every-declaration-to-it)) | `## context: Context` |
| `Superceded by [4. …](0004-….md)` | a citation in the status chapter, which `grund fmt --cross-refs --write` links ([§FS-fmt.6](../../docs/functional-spec/FS-fmt.md#6-cross-reference-emission)) | `Superseded by` and a link to `ADR-0004` |
| the arc42 chapters | a place, `arc42`, whose files cite but declare nothing | `docs/arc42/` |
| arc42 chapter 9 | the ADR kind's index ([§FS-check.3.18](../../docs/functional-spec/FS-check.md#318-declaration-missing-from-its-kinds-index)) | a table whose first column links each record |
| a quality goal, arc42 1.2 | a declaration of kind `QG` in chapter 1, a file the `QG` row lists ([§FS-config-v2.schema.places.ownership](../../docs/functional-spec/FS-config-v2.md#schemaplacesownership-a-listed-file-then-the-deepest-folder)) | `### QG-1: A seat is never booked twice` |
| a quality scenario, arc42 10.2 | a declaration of kind `QS` in chapter 10, which must cite a `QG` | `### QS-1: Two students book one seat at once` |
| code that implements a decision | the complement, `code`, which must cite an `ADR` and is grounded per file | `bookings.py` cites `ADR-0004` |
| MADR's front matter | text before the declaration; grund reads none of it | [MADR](#madr-the-status-in-front-matter) |

## Step 1: what the records are held to

```bash
grund schema examples/adr-arc42/1-schema/repo
echo $?    # 0
```

`grund schema` reads the configuration alone ([§FS-schema](../../docs/functional-spec/FS-schema.md#fs-schema-grund-schema-shows-what-a-repository-is-held-to-read-from-its-configuration-alone)):

```text
carrel · grund.toml · v2

kinds and places
  ADR    kind                  docs/
           field status        must · first · one of Proposed, Accepted, Deprecated, Superseded
           field context       must · after status
           field decision      must · after context
           field consequences  must · after decision
           closed must
  arc42  place                 docs/arc42/
  QG     kind                  docs/arc42/01_introduction_and_goals.md
  QS     kind                  docs/arc42/10_quality_requirements.md
  code   complement            src/

citations                      grounded
  ADR    —
  arc42  —
  QG     must not  everything else
  QS     must      QG
  code   must      ADR         must · file

must = check error · warn = check warning · should = suggestion (check --suggestions)
may = allowed · must not / warn not / should not = forbidden at that strength
A|B = one obligation, met by citing either · — = no direction rule names it
```

Read it as the two formats:

- **ADR** is a Nygard record in adr-tools' order: the status first, then
  Context, Decision and Consequences, each *directly* after the one before.
  Nygard's post lists Status after Decision; `after` can state either order.
  `closed must` refuses any other named chapter, such as an `Alternatives`
  chapter, unless the kind declares it.
- **ADR at `docs/`** is not a typo. A kind's index must be a file inside the
  kind's place ([§FS-config.3.4.2.3](../../docs/functional-spec/FS-config.md#3423-a-named-index-is-a-markdown-file-inside-folder)), so the row's place is `docs/` and its index is
  `arc42/09_architecture_decisions.md`. The deeper `arc42` place and the two
  listed chapter files take everything else, which leaves `docs/adr/` to the
  records. The view does not print the index.
- **QG must not** cite anything. A quality goal is where the reasoning starts:
  decisions and scenarios cite it, and it cites none of them back
  ([§FS-config-v2.rules.citations.default](../../docs/functional-spec/FS-config-v2.md#rulescitationsdefault-a-default-may-forbid-never-oblige)).
- **QS must cite QG** follows arc42's own advice: the most important quality
  requirements are the goals of section 1.2, and section 10 refers to them
  rather than restating them. A scenario that names no goal refines nothing.
- **code must cite ADR, grounded per file**: every source file names a
  decision, and a file that cites nothing is a finding too
  ([§FS-config-v2.rules.grounding](../../docs/functional-spec/FS-config-v2.md#rulesgrounding-grounding-is-a-ladder-replaced-whole)).

## Step 2: the tree holds

```bash
grund check examples/adr-arc42/2-check/repo
echo $?    # 0
```

```text
success
```

The records are still the records adr-tools and Nygard describe: one numbered
file each, a `Date:` line, a status, and the three sections in order. The
arc42 chapters keep the template's file names and headings, from
`# Introduction and Goals` to `# Glossary`; the only headings that carry grund
syntax are the goals' and the scenarios'. Every record is listed in chapter 9,
every scenario names its goal, and each of the three source files cites the
decision it implements. `migrate_sqlite.py` cites ADR-0002 as well as ADR-0004, because
it moves the pilot's data out of the store the old decision chose.

## Step 3: the decision log

```bash
grund list --selector ADR.status examples/adr-arc42/3-log/repo
echo $?    # 0
```

```text
ADR-0001.status  docs/adr/0001-record-architecture-decisions.md:5  Accepted
ADR-0002.status  docs/adr/0002-store-bookings-in-sqlite.md:5  Superseded
ADR-0003.status  docs/adr/0003-release-unclaimed-seats.md:5  Accepted
ADR-0004.status  docs/adr/0004-store-bookings-in-postgresql.md:5  Accepted
ADR-0005.status  docs/adr/0005-claim-seats-from-the-phone.md:5  Proposed
```

This is adr-tools' `adr list` with the status beside each record. Because the
status is a field, the third column is one of four spellings, and
`--format json` gives it to a script as each row's `title`. That is also why
chapter 9 has no status column: a second copy there is one that nothing
checks, while this query reads the copy `grund check` holds to the closed set.

## Writing a record

Allocate the next number with the width adr-tools uses. `grund id` pads to
three digits unless told otherwise ([§FS-id.1.2](../../docs/functional-spec/FS-id.md#12-number-width)):

```console
$ grund id ADR "Close a room from the desk"
ADR-006
$ grund id ADR "Close a room from the desk" --width 4 --explain
ADR-0006
next: write the declaration at docs/ADR-0006.md  (H1: `# ADR-0006: <one-line statement>`), then cite it as §ADR-0006
```

Ignore the suggested path. It names the row's place, `docs/`, and a file named
after the ID. Name the file as adr-tools would,
`docs/adr/0006-close-a-room-from-the-desk.md`; `--format json` prints the slug.
Write the record in the kind's shape, add its row to chapter 9, write each
citation bare, and run `grund fmt --cross-refs --write` from the project root to
turn the citations into links:

```markdown
# ADR-0006: Close a room from the desk

Date: 2026-10-20

## status: Accepted

## context: Context

## decision: Decision

## consequences: Consequences
```

To supersede a record, first ask what leans on it. Here every citation of
ADR-0002 is one that should stay:

```console
$ grund refs ADR-0002
docs/adr/0004-store-bookings-in-postgresql.md:10: §ADR-0002
docs/arc42/09_architecture_decisions.md:10: §ADR-0002 (index entry)
src/carrel/migrate_sqlite.py:3: §ADR-0002
```

Then set the old record's status to `Superseded` and name the successor on the
first line under it, as adr-tools and MADR both do in the old record:

```markdown
## status: Superseded

Superseded by §ADR-0004.
```

Only that direction is written. `grund refs ADR-0004` lists that line, so it
answers "what did ADR-0004 replace?" without adr-tools' second `Supercedes`
link in the new record, which nothing would hold in agreement with the first.

grund checks that the link resolves, so a renumbered or deleted successor is a
`dangling` error. It cannot check that the link exists. An obligation applies
to every declaration of a kind, or to every chapter of a name, and never only
to the records whose status says `Superseded` ([§FS-config.3.9.1.1](../../docs/functional-spec/FS-config.md#3911-obligations-and-prohibitions)). The
nearest rule, tried on the clean tree, reports the records that need no link:

```console
$ grund check --rule 'The status chapter of each ADR must cite at least one ADR.' --only-rule
docs/adr/0001-record-architecture-decisions.md:5: error: ADR-0001.status must cite ADR (--rule)
docs/adr/0003-release-unclaimed-seats.md:5: error: ADR-0003.status must cite ADR (--rule)
docs/adr/0004-store-bookings-in-postgresql.md:5: error: ADR-0004.status must cite ADR (--rule)
docs/adr/0005-claim-seats-from-the-phone.md:5: error: ADR-0005.status must cite ADR (--rule)
```

A separate optional `superseded-by` chapter does no better: a rule over a
chapter reports every record that lacks it as unreached
([§FS-rules.checks.unreached-declaration](../../docs/functional-spec/FS-rules.md#checksunreached-declaration-unreached-declaration)). The successor-cites-predecessor direction
has the same problem, so neither is checked, and the old-to-new link is the one
written, because it is the one the formats agree on and the one a reader of the
old record needs.

## MADR: the status in front matter

MADR 4.0.0 opens each record with metadata it marks optional:

```yaml
---
status: "{proposed | rejected | accepted | deprecated | … | superseded by ADR-0123}"
date: {YYYY-MM-DD when the decision was last updated}
decision-makers: {list everyone involved in the decision}
---
```

grund reads nothing before a declaration's heading. Give ADR-0005 that front
matter in place of its `Date:` line and status chapter, and the record has no
status at all, with nothing said about the `status:` key:

```text
docs/adr/0005-claim-seats-from-the-phone.md:7: error: ADR-0005 has no status field; kind ADR declares it
```

A MADR repository under grund therefore writes its status twice: in the front
matter for MADR's tooling, and as `## status: proposed` for grund, which checks
only the second copy. It also puts the ID in the title, against MADR's own
decision to keep numbers out of headings, and declares MADR's sections as
fields. This shape, with `one_of` in MADR's lower case, checks clean against a
record written in it, and MADR's two closing chapters, Pros and Cons of the
Options and More Information, follow `outcome` the same way as optional fields:

```toml
[schema.kinds.ADR.fields.status]
one_of = ["proposed", "rejected", "accepted", "deprecated", "superseded"]
presence = "must"
position = "first"

[schema.kinds.ADR.fields.problem]
title = "Context and Problem Statement"
presence = "must"
after = "status"

[schema.kinds.ADR.fields.drivers]
title = "Decision Drivers"
after = "problem"

[schema.kinds.ADR.fields.options]
title = "Considered Options"
presence = "must"
after = "drivers"

[schema.kinds.ADR.fields.outcome]
title = "Decision Outcome"
presence = "must"
after = "options"
```

`superseded by ADR-0123` is a value only if every successor is listed, because
`one_of` is a closed list matched byte for byte; write `## status: superseded`
and the link under it, as above. MADR's `### Consequences` sits under
`## Decision Outcome`, so it can be written `### outcome.consequences:
Consequences` but never required: a field is a direct chapter
([§FS-fields.identity.name](../../docs/functional-spec/FS-fields.md#identityname-a-field-is-the-direct-section-of-its-name)). Written bare, it gets the same advice to number it as the
`## Status` of edit 1 below.

## Break it

Copy the fixture somewhere writable and make six edits that the formats
themselves allow:

```bash
demo_root=$(mktemp -d)
cp -R examples/adr-arc42/2-check/repo/. "$demo_root/"
cd "$demo_root"
```

1. In ADR-0003, write the status the way adr-tools' template does: replace
   `## status: Accepted` with a `## Status` heading and `Accepted` on its own
   line below it.
2. In ADR-0002, spell the status as adr-tools' `Superceded by` link does:
   `## status: Superceded`.
3. In ADR-0004, move `## status: Accepted` down to just above
   `## consequences:`, the order Nygard's post lists the parts in.
4. In ADR-0005, replace the `Date:` line and the status chapter with the MADR
   front matter above, `status: proposed` among it.
5. Add `docs/adr/0006-close-a-room-from-the-desk.md`, the record from
   [Writing a record](#writing-a-record) with one sentence under each chapter,
   and leave chapter 9 alone.
6. In `docs/arc42/10_quality_requirements.md`, delete the line that says QS-4
   makes QG-3 measurable.

```console
$ grund check
docs/adr/0002-store-bookings-in-sqlite.md:5: error: ADR-0002.status is "Superceded"; kind ADR admits Proposed, Accepted, Deprecated, Superseded
docs/adr/0003-release-unclaimed-seats.md:1: error: ADR-0003 has no status field; kind ADR declares it
docs/adr/0003-release-unclaimed-seats.md:5: error: unmarked heading inside ADR-0003; number it (## 1. Status) as ADR-0003.1, declare an ID, or use a bold label; this became an error in grund 0.16.0
docs/adr/0004-store-bookings-in-postgresql.md:5: error: ADR-0004.context is section 1 of 4; kind ADR places it directly after status
docs/adr/0004-store-bookings-in-postgresql.md:17: error: ADR-0004.status is section 3 of 4; kind ADR places it first
docs/adr/0004-store-bookings-in-postgresql.md:19: error: ADR-0004.consequences is section 4 of 4; kind ADR places it directly after decision
docs/adr/0005-claim-seats-from-the-phone.md:7: error: ADR-0005 has no status field; kind ADR declares it
docs/adr/0006-close-a-room-from-the-desk.md:1: error: ADR-0006 is not listed in docs/arc42/09_architecture_decisions.md: an entry is a `§`-marked Markdown link to the declaration; write `[§ADR-0006](../adr/0006-close-a-room-from-the-desk.md#adr-0006-close-a-room-from-the-desk)` in docs/arc42/09_architecture_decisions.md — became an error in grund 0.13.0
docs/arc42/10_quality_requirements.md:45: error: QS-4 must cite QG (citation direction)
docs/adr/0006-close-a-room-from-the-desk.md:1: warning: declared but never cited: ADR-0006
```

Two of these say more than the edit deserves:

- For edit 1, the repair is `## status: Accepted`. Numbering the heading, as
  the advice says, leaves the status missing, because a bare `## Status` is
  never the field ([§FS-declarations.checks.unmarked-heading](../../docs/functional-spec/FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading)).
- Edit 3 moved one chapter and drew three findings, because each chapter is
  judged against where its predecessor now stands. The repair is to move the
  status back; nothing else needs to change.

The index finding writes out the row chapter 9 needs, and the warning goes once
something cites ADR-0006.

## What still passes

These three edits, made on a fresh copy, leave a tree `grund check` accepts:

1. Add record 6 with adr-tools' default template: `# 6. Close a room from the
   desk`, then `## Status`, `Accepted`, `## Context` and the rest.
2. In ADR-0002, delete the `Superseded by` line.
3. In `src/carrel/bookings.py`, cite ADR-0002 where it cites ADR-0004.

```console
$ grund check
success
```

The first file declares nothing, so neither the closed kind, nor its fields,
nor chapter 9 asks anything of it: a record `adr new` writes is invisible until
someone renames its heading. The second is the supersession rule grund cannot
state. The third is code that follows a decision the records have replaced;
`grund refs ADR-0002` finds it, and nothing reports it.

## Trade-offs

What the schema buys:

- **A record keeps its shape.** Each ADR has a status from the closed set and
  its three sections in order, and a status spelled any other way, such as
  adr-tools' own `Superceded`, is an error.
- **Chapter 9 is complete.** A record that is not listed there is an error,
  and the finding writes out the missing row.
- **Quality scenarios stay attached.** A scenario must name a quality goal,
  and a goal cites nothing back, so the arc42 chain from goal to scenario to
  decision runs one way.
- **Decisions reach the code.** Every source file names a decision, and
  `grund refs` lists what still leans on one before it is superseded.

What it costs:

- **The headings show grund's syntax.** The ID sits in the title, where
  adr-tools writes `# 2.` and MADR writes no number, and every section heading
  carries its name, `## context: Context`. adr-tools' scripts find the status
  by a bare `## Status` heading and a line under it, so `adr new -s` and
  `adr link` no longer update these records. Write supersessions by hand.
- **The status moves into the heading.** adr-tools keeps it on a line below
  `## Status`, and MADR in front matter, but a checked status is the heading of
  its chapter. A MADR record writes it twice.
- **Supersession is half checked.** A successor link must resolve, but a
  superseded record need not have one, and code may cite a superseded record.
- **Chapter 9 sets the ADR row's place.** Because the index must sit inside it,
  the row's place is `docs/`, which `grund schema` and `grund id --explain`
  both show. A record declared in `docs/` outside `docs/adr/` is not
  misplaced; only chapter 9's index finding catches it.
- **The arc42 chapters themselves are not checked.** Nothing says which files a
  place must hold, and even a listed place that does not exist is quiet
  ([§FS-config-v2.schema.places.boundaries.4](../../docs/functional-spec/FS-config-v2.md#schemaplacesboundaries4-a-missing-place-is-quiet)), so deleting `12_glossary.md`
  still passes. Their headings stay exactly arc42's because no rule reads them.
- **arc42's quality goal table becomes headings.** arc42 suggests a table of
  goals, but a declaration is a heading. Chapter 1 is also wholly the `QG`
  row's file, so its Requirements Overview and Stakeholders are held to the
  goals' rule and cite nothing either.
- **Four digits by hand.** `grund id` needs `--width 4` on every call, and its
  suggested path is not adr-tools' name.
- **Markdown only.** arc42's reference edition is AsciiDoc. This example uses
  its Markdown edition, because grund declares nothing in AsciiDoc.
