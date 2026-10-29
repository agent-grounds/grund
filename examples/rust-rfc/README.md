# Rust RFCs: an RFC repository held to a grund schema

The [Rust RFC process](https://github.com/rust-lang/rfcs) keeps one accepted
design per file, `text/NNNN-feature-name.md`, each copied from
`0000-template.md`. The template opens with a four-item metadata list
(`- Feature Name:`, `- Start Date:`, `- RFC PR:`, `- Rust Issue:`) and then
nine `##` sections, from Summary to Future possibilities. The number is the
pull request's number. An author writes `0000-my-feature.md`, opens the pull
request, and only then renames the file to its number. Nothing in the process
checks the shape: accepted RFCs differ in their sections (those written before
RFC 2333 added Prior art have none), retitle them (RFC 3137 writes
"Reference-level explanations"), and code refers to them by number, in
comments no tool checks.

This example is for a project that adopts the Rust RFC template, such as a crate,
a toolchain or a team, and wants `grund check` as the gate. It sets up a
conformant repository ([§FS-examples.2](../../docs/functional-spec/FS-examples.md#2-canonical-use-cases)) in which:

- each RFC is a declaration of one closed kind;
- the template's nine sections are an ordered chain of fields;
- the metadata list is the declaration's lead;
- the crate and its tests are places that must cite the RFC sections they
  implement.

The fixture is `skerry`, a fictional crate with a retry policy. RFC 0007
specifies capped exponential backoff. RFC 0012 builds on it with jitter,
answers one of its unresolved questions, and is recorded as its amendment.
The example pins three runs over the same fixture ([§FS-examples.5.5](../../docs/functional-spec/FS-examples.md#55-an-example-may-pin-several-runs)):
the schema, the check, and the agent guidance `grund init` writes. Kubernetes
KEPs and Python PEPs are compared [at the end](#the-other-formats-keps-and-peps).

## The repository

```text
grund.toml                        the schema
Cargo.toml                        the crate; not a scanned language
rfcs/
  README.md                       the process, in three steps
  0000-template.md                RFC-0000-my-feature         the template, held to the same fields
  text/0007-retry-policy.md       RFC-0007-retry-policy       capped exponential backoff
  text/0012-jittered-backoff.md   RFC-0012-jittered-backoff   full jitter; amends 0007
src/lib.rs                        cites the reference-level sections it implements
tests/backoff.rs                  cites the reference-level sections each test proves
```

Open [`grund.toml`](1-schema/repo/grund.toml) first, then
[`rfcs/text/0012-jittered-backoff.md`](1-schema/repo/rfcs/text/0012-jittered-backoff.md):
it carries every section, cites the RFC it builds on, and keeps an open
question. Then [`src/lib.rs`](1-schema/repo/src/lib.rs), whose comments cite
the sections each line implements.

## How the RFC template maps onto grund

| Rust RFC | grund | In this repository |
|---|---|---|
| an accepted RFC, one file in `text/` | a declaration of the closed kind `RFC`, whose place is `rfcs/text/` | `rfcs/text/0007-retry-policy.md` |
| the number and the file's slug | the `{kind}-{number}-{slug}` ID scheme ([§FS-config.3.2](../../docs/functional-spec/FS-config.md#32-id--id-grammar)) | `RFC-0007-retry-policy` |
| no title line; the file name names the RFC | a declaration heading, added above the metadata | `# RFC-0007-retry-policy: Retry with capped exponential backoff` |
| the metadata list | the lead, the text before the first section; grund reads none of it | `grund RFC-0007-retry-policy` prints it |
| the nine sections, `## Summary` | a chain of fields, `## summary: Summary` ([§FS-fields.order](../../docs/functional-spec/FS-fields.md#order-order)) | `summary` first, each next one directly after the one before |
| a section's self-anchor, `[summary]: #summary` | kept, pointed at the anchor the field heading gets | `[summary]: #summary-summary` |
| a subsection, `### Desugaring example` | a named section with its complete path ([§FS-declarations.section](../../docs/functional-spec/FS-declarations.md#section-what-a-section-is)) | `### reference-level-explanation.schedule: The delay schedule` |
| an `Amendments` note on an amended RFC | an optional field after the last of the nine | `## amendments: Amendments` in RFC 0007 |
| "RFC 7" in prose, or a link to its pull request | a citation, which `grund fmt --write` links ([§FS-fmt.6](../../docs/functional-spec/FS-fmt.md#6-cross-reference-emission)) | the motivation of RFC 0012 cites the drawbacks of RFC 0007 |
| `0000-template.md` | a declaration outside the kind's place, held to the same fields | `RFC-0000-my-feature` |
| an implementation that says "see RFC 7" | a place that must cite an RFC, in every file ([§FS-check.3.6](../../docs/functional-spec/FS-check.md#36-ungrounded-unit-opt-in)) | `src/` and `tests/` |

The template already names every section the way a field is named. Its
self-anchors are `#summary`, `#guide-level-explanation` and
`#rationale-and-alternatives`, and those are the field handles here. What
changes is the heading: grund finds a field by the section name its heading
writes, never by its title ([§FS-fields.identity](../../docs/functional-spec/FS-fields.md#identity-identity)), so a bare `## Summary` is
no field.

## Step 1: what an RFC is held to

```bash
grund schema examples/rust-rfc/1-schema/repo
echo $?    # 0
```

`grund schema` reads the configuration alone ([§FS-schema](../../docs/functional-spec/FS-schema.md#fs-schema-grund-schema-shows-what-a-repository-is-held-to-read-from-its-configuration-alone)):

```text
skerry · grund.toml · v2

kinds and places
  RFC    kind                                 rfcs/text/
           field summary                      must · first
           field motivation                   must · after summary
           field guide-level-explanation      must · after motivation
           field reference-level-explanation  must · after guide-level-explanation
           field drawbacks                    must · after reference-level-explanation
           field rationale-and-alternatives   must · after drawbacks
           field prior-art                    must · after rationale-and-alternatives
           field unresolved-questions         must · after prior-art
           field future-possibilities         must · after unresolved-questions
           field amendments                   may · after future-possibilities
           closed must
  crate  place                                src/
  test   place                                tests/
  code   complement                           every scanned file no other row claims

citations                                     grounded
  RFC    —
  crate  must  RFC                            must · file
  test   must  RFC                            must · file
  code   —

must = check error · warn = check warning · should = suggestion (check --suggestions)
may = allowed · must not / warn not / should not = forbidden at that strength
A|B = one obligation, met by citing either · — = no direction rule names it
```

Read it as the template:

- **The nine fields are one chain.** `summary` is first and every other field
  comes directly after the one before it, so the template's order is the only
  order. An absent field is skipped, and the next one takes its place, so
  dropping a section is one finding, not one for every section after it.
- **`closed must`** admits the ten fields and nothing else. A named chapter
  the template does not have, such as the older template's
  `## detailed-design: Detailed design`, is an error. Each field's `title`, not
  shown here but written in [`grund.toml`](1-schema/repo/grund.toml), is the
  template's heading, byte for byte.
- **`amendments`** is `may`. Only an amended RFC carries it, after Future
  possibilities, as accepted RFCs append their amendment notes.
- **`crate` and `test` must cite RFC, in every file.** Each file in `src/` and
  `tests/` cites the RFC it implements or proves. A file that cites nothing is
  an error too, because both rows are grounded.
- `code` is the rest: `rfcs/README.md` and the template.

## Step 2: the RFCs hold

```bash
grund check examples/rust-rfc/2-check/repo
echo $?    # 0
```

```text
success
```

So each RFC, and the template, carries all nine sections in the template's
order and with its titles. Every citation in the RFCs, the crate and the tests
resolves, down to the section. Every RFC is cited somewhere, so no
`declared but never cited` warning remains.

The lead keeps the metadata where the template puts it, under the title:

```console
$ grund RFC-0007-retry-policy
- Feature Name: `retry_policy`
- Start Date: 2026-03-02
- RFC PR: [skerry/rfcs#7](https://forge.example/skerry/rfcs/pull/7)
- Skerry Issue: [skerry/skerry#41](https://forge.example/skerry/skerry/issues/41)
```

And who implements a reference-level explanation is one query
([§FS-refs](../../docs/functional-spec/FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id)):

```console
$ grund refs RFC-0007-retry-policy.reference-level-explanation --descendants --summary
rfcs/text/0007-retry-policy.md: 1 (line 107)
rfcs/text/0012-jittered-backoff.md: 1 (line 44)
src/lib.rs: 3 (lines 16, 36, 40)
tests/backoff.rs: 3 (lines 10, 19, 27)
```

Without `--descendants`, `refs` counts only citations of the chapter itself,
and the code cites its sections.

## Step 3: what an agent drafting an RFC is told

```bash
demo_root=$(mktemp -d)
cp -R examples/rust-rfc/3-init/repo/. "$demo_root/"
grund init "$demo_root"
echo $?    # 0
```

The run writes one file, `AGENTS.md` beside `grund.toml`, which
[`3-init/expected.repo/`](3-init/expected.repo/AGENTS.md) records. Its report
is on stderr:

```text
wrote AGENTS.md
exists grund.toml

next:
  1. run `grund check`
  2. allocate an ID:  ID=$(grund id RFC "…")  then add it under rfcs/text
see AGENTS.md for the full workflow.
```

The `next:` steps name the repository's own kind and place, `RFC` in
`rfcs/text`, because it has no `FS` row ([§FS-init.2.2.2.3](../../docs/functional-spec/FS-init.md#2223-the-kind-the-guidance-names)). The managed block's declaration shapes
([§FS-init.2.3.5.12.2](../../docs/functional-spec/FS-init.md#235122-declaration-shapes)) are the RFC template as the schema states it:

```markdown
- **RFC**
  - field summary must · first
  - field motivation must · after summary
  - field guide-level-explanation must · after motivation
  - field reference-level-explanation must · after guide-level-explanation
  - field drawbacks must · after reference-level-explanation
  - field rationale-and-alternatives must · after drawbacks
  - field prior-art must · after rationale-and-alternatives
  - field unresolved-questions must · after prior-art
  - field future-possibilities must · after unresolved-questions
  - field amendments may · after future-possibilities
  - closed must
```

An agent learns the order and the handles before it writes. It does not learn
the titles, so it takes the heading itself, `## summary: Summary`, from the
template it copies.

`[presentation] rules = "root"` keeps that guidance in the root `AGENTS.md`
([§FS-init.2.3.5.11.5](../../docs/functional-spec/FS-init.md#235115-rules--root-and-every-version-1-config-render-at-the-root)). The default writes it into `rfcs/text/AGENTS.md`,
where a tool that reads every file in `text/` as an RFC would pick it up.

## Writing an RFC

Copy the template into `text/` and run the check before writing anything:

```console
$ cp rfcs/0000-template.md rfcs/text/0000-retry-budget.md
$ grund check
rfcs/0000-template.md:1: error: duplicate declaration of RFC-0000-my-feature (also declared at rfcs/text/0000-retry-budget.md:1)
```

The template is a declaration, so the copy must take a slug of its own, as the
process asks of the file name. Write the heading as
`# RFC-0000-retry-budget: A retry budget per dependency`, fill in the
sections, and give each rule of the reference-level explanation a named
section, here `### reference-level-explanation.ratio: The budget ratio`. Cite a
section as you write it, bare, in the same form the code uses:

```markdown
Stop retrying a dependency once retries make up too large a share of its
calls, the ratio §RFC-0000-retry-budget.reference-level-explanation.ratio fixes.
```

The check then passes. Leave the number at `0000` until the pull request
exists. `grund id` is the wrong tool here ([§FS-id](../../docs/functional-spec/FS-id.md#fs-id-grund-proposes-ids-for-new-declarations)): it proposes the next
number after the highest, `RFC-0013-retry-budget` with `--width 4`, while the
number this process wants is the pull request's.

Say the pull request is number 19. Rename the file to
`rfcs/text/0019-retry-budget.md` and the heading's ID to
`RFC-0019-retry-budget`. `grund fmt` never rewrites an ID you wrote, so
`grund refs RFC-0000-retry-budget`, run before the rename, lists what else
must change:

```console
$ grund refs RFC-0000-retry-budget
rfcs/text/0000-retry-budget.md:12: §RFC-0000-retry-budget.reference-level-explanation.ratio
```

Rename the heading alone and the check names the citation it left behind:

```console
$ grund check
rfcs/text/0019-retry-budget.md:12: error: unknown reference RFC-0000-retry-budget; did you mean RFC-0019-retry-budget?
rfcs/text/0019-retry-budget.md:1: warning: declared but never cited: RFC-0019-retry-budget
```

Replace the old ID at each site `refs` listed, then run `grund fmt --write`,
which links every new citation to its section's anchor. `grund check` then
prints `success`.

## What the schema cannot say: unresolved questions

A natural house rule is that an RFC must not leave its Unresolved questions
open once its feature is stable. Write each open question as a named section,
as RFC 0007 and RFC 0012 do, and grund can list and count them
([§FS-coordinate-globs](../../docs/functional-spec/FS-coordinate-globs.md#fs-coordinate-globs-a-coordinate-glob-selects-catalog-coordinates-by-pattern)):

```console
$ grund list examples/rust-rfc/2-check/repo --glob 'RFC-*.unresolved-questions.*'
RFC-0007-retry-policy.unresolved-questions.jitter  rfcs/text/0007-retry-policy.md:89  Randomized delays
RFC-0012-jittered-backoff.unresolved-questions.seeding  rfcs/text/0012-jittered-backoff.md:76  A seedable default source
```

No rule can forbid them, though. A chapter subject cannot count its own
chapters, and a count must be positive ([§FS-rules.2](../../docs/functional-spec/FS-rules.md#2-subject-selectors)):

```console
$ grund check examples/rust-rfc/2-check/repo --rule 'The unresolved-questions chapter of each RFC must have at most 1 open chapter.' --only-rule
error: chapter subjects cannot have chapters
$ grund check examples/rust-rfc/2-check/repo --rule 'Each RFC must have at most 0 unresolved-questions chapters.' --only-rule
error: count must be a canonical positive base-10 integer
```

The first refusal offers no accepted form, because every rule sentence grund
could build would count other chapters than the ones under
`unresolved-questions` ([§FS-rule-refusals.sentences.rewrite.same-units](../../docs/functional-spec/FS-rule-refusals.md#sentencesrewritesame-units-a-form-judges-the-units-the-typed-sentence-named-or-none-is-offered)). The
question it asks is the one `grund list --glob 'RFC-*.unresolved-questions.*'`
above answers. Two more pieces are missing as well. Which RFCs are
stable is not in the file: it lives on the tracking issue, and nothing in a
rule can select "each RFC whose status is stable". And a chapter-presence rule
names a chapter by its title, in one word, so it cannot name
`Unresolved questions` at all ([§FS-rules.3.5.1](../../docs/functional-spec/FS-rules.md#351-presence-name-whitespace-refusal)). The count above is
therefore for a CI step to compare, not something `grund check` judges.

## Break it

Copy the fixture somewhere writable and make four edits that the RFC process
itself accepts:

```bash
demo_root=$(mktemp -d)
cp -R examples/rust-rfc/2-check/repo/. "$demo_root/"
cd "$demo_root"
```

1. In `rfcs/text/0012-jittered-backoff.md`, paste the upstream template's
   heading in place of the field: `## Drawbacks` and `[drawbacks]: #drawbacks`.
2. In `rfcs/text/0007-retry-policy.md`, title the section the way RFC 3137
   does: `## reference-level-explanation: Reference-level explanations`.
3. Append a chapter the template does not have to RFC 0012:

   ```markdown
   ## security: Security considerations

   A source an attacker can predict tells them when each client retries.
   ```

4. Refine a section name in RFC 0007: `### reference-level-explanation.limit:`
   becomes `### reference-level-explanation.max-retries:`.

```console
$ grund check
rfcs/text/0007-retry-policy.md:46: error: RFC-0007-retry-policy.reference-level-explanation is titled "Reference-level explanations"; kind RFC titles it "Reference-level explanation"
rfcs/text/0012-jittered-backoff.md:1: error: RFC-0012-jittered-backoff has no drawbacks field; kind RFC declares it
rfcs/text/0012-jittered-backoff.md:53: error: unmarked heading inside RFC-0012-jittered-backoff; number it (## 1. Drawbacks) as RFC-0012-jittered-backoff.1, declare an ID, or use a bold label; this became an error in grund 0.16.0
rfcs/text/0012-jittered-backoff.md:86: error: RFC-0012-jittered-backoff.security is not a field of kind RFC, which is closed; its fields: summary, motivation, guide-level-explanation, reference-level-explanation, drawbacks, rationale-and-alternatives, prior-art, unresolved-questions, future-possibilities, amendments
src/lib.rs:36: error: section not found: RFC-0007-retry-policy.reference-level-explanation.limit; write <§> before it to show the shape without citing it
tests/backoff.rs:19: error: section not found: RFC-0007-retry-policy.reference-level-explanation.limit; write <§> before it to show the shape without citing it
```

The repair for the pasted heading is the field's form, `## drawbacks: Drawbacks`,
with its anchor `#drawbacks-drawbacks`. Numbering it as the second finding
suggests would leave the field missing. The renamed section is the edit grund
exists for: the code and the test still cite the old name, and the check
fails at the lines that implement and prove it
([§FS-check.3.2](../../docs/functional-spec/FS-check.md#32-missing-section)). Rename the citations, rather than escaping them.

## The other formats: KEPs and PEPs

| | Rust RFC | Kubernetes KEP | Python PEP |
|---|---|---|---|
| a proposal is | `text/NNNN-slug.md` | `keps/<sig>/NNNN-slug/README.md` beside `kep.yaml` | `peps/pep-NNNN.rst` |
| its number comes from | the pull request, after writing | the enhancement issue, before writing | the next free number, chosen when writing |
| its title line | none | `# KEP-NNNN: Title` | a `Title:` header |
| its metadata | a Markdown list | `kep.yaml`: status, stage, milestones | an RFC 822-style header: `PEP:`, `Status:`, `Type:` |
| its sections | nine `##` headings | `##` to `######`, the review questions at `######` | reStructuredText underlined titles |

- **KEP.** Once numbered, the title `# KEP-1234: Title` is already a grund
  declaration under `{kind}-{number}`. Its sections are bare headings, as an
  RFC's are, so they need the field form. Its review questionnaire puts each
  question at `######` under a `###` heading, two levels lower than a section
  path allows; `[schema] heading_depth = "may"` admits that, and a bold label
  avoids it. Its status lives in `kep.yaml`, which grund does not read, so the
  state a rule would want to select on is out of reach.
- **PEP.** A PEP is reStructuredText, whose headings are a title line
  underlined by punctuation. grund declares only from a Markdown heading or a
  source comment, so a PEP declares nothing; the
  [markup-format discussion](../../docs/discussions/proposals/2026-05-25-markup-format-declarations.md)
  records why that heading form resists a line-oriented reader. Of the three,
  PEP numbering fits `grund id`, because the author picks the next free number.

## Trade-offs

What the schema buys:

- **The template, checked.** Nine sections, in the template's order and with
  its titles, in every RFC and in the template itself. A copied template
  cannot drift from the schema, and a missing Prior art is an error rather
  than a reviewer's catch.
- **Code that cites the section it implements.** A comment that once said "see
  RFC 7" now names the reference-level section, and renaming that section
  fails the check at the code.
- **Navigation for free.** `grund RFC-0007-retry-policy --toc` is the RFC's
  outline, `grund refs` lists its implementers, and `grund list --glob` counts
  its open questions.

What it costs:

- **Every heading shows its handle.** `## Summary` becomes `## summary: Summary`,
  so the anchor becomes `#summary-summary`, and a section pasted from the
  upstream template fails until it is rewritten. The template's self-anchors
  must be repointed by hand: `grund fmt` keeps citation links current, but not
  those.
- **A title line the template does not have.** grund declares from a heading,
  so each RFC gains one above its metadata.
- **The metadata is unchecked.** Nothing checks that the `RFC PR` number is the
  ID's number, that `Feature Name` is set, or that the tracking issue exists.
- **Renumbering is by hand.** The number arrives after writing, and every
  citation of the draft's ID is renamed at the sites `grund refs` lists.
- **One repository.** rust-lang keeps its RFCs in their own repository, while
  grund resolves citations only within one checkout. Citing another
  repository's declarations is specified ([§FS-remote-projects](../../docs/functional-spec/FS-remote-projects.md#fs-remote-projects-a-project-cites-another-repositorys-declarations-from-a-committed-pinned-projection)) but not yet
  shipped, so this crate keeps its RFCs in `rfcs/`.
- **Stricter slugs than the file names.** 33 of the 643 RFC files in
  rust-lang/rfcs' `text/` have an underscore, a capital or a dot in the slug.
  `[schema] slug_pattern = "[a-z][a-z0-9_-]*"` admits underscores, and a
  capital can be admitted the same way. A dot cannot be, because it separates
  an ID from its section.
