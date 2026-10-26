# Kind fields

A version-2 kind can say what a declaration of it looks like: which named
chapters it carries, in what order, what a one-word chapter may say, and whether
anything else may appear. `grund check` then holds every declaration of the kind
to that shape, on the channel each field's strength names. The contract is
[§FS-fields](../functional-spec/FS-fields.md#fs-fields-a-v2-kind-states-the-shape-of-its-declarations-and-check-holds-every-declaration-to-it); this guide shows the shapes people
actually write, and the runnable [`examples/fields/`](../../examples/fields/)
repository checks three of them.

Fields are version 2 only. A version-1 `[[kinds]]` row keeps meaning what it
meant, and `fields` on it is an unknown key; see
[writing the config in version 2](repository-options.md#write-the-config-in-version-2).

## A field is a table under its kind

```toml
[schema.kinds.FS.fields.terms]
title = "Terms"
presence = "must"
position = "first"
```

| Key | What it says | Default |
|---|---|---|
| `title` | the display name the chapter must show | any |
| `presence` | `must`, `warn`, `should` or `may` | `may` |
| `position` | `first` or `last` | none |
| `after` | the field this one comes *directly* after | none |
| `content` | `prose` or `values` | `prose` |
| `one_of` | the display names a scalar chapter may show | none |

The field is the declaration's **direct** chapter whose section name is the
table's name — `terms` in `## terms: Terms` — never a chapter that only shows
the same title, and never one nested deeper. So a citation of
`FS-login.terms` resolves whatever the chapter is titled. `title` and `one_of`
compare with the display name byte for byte; there is no case folding
([§FS-fields.identity](../functional-spec/FS-fields.md#identity-identity)).

## Repair a bare field heading

With the required `terms` field above, a bare `## Terms` produces both
`unmarked-heading` and `missing-field`. The heading error appends the field
form: `kind FS declares the field terms, written ## terms: Terms`
([§FS-declarations.checks.unmarked-heading.2.2](../functional-spec/FS-declarations.md#checksunmarked-heading22-a-direct-field-title-or-name-is-also-shown-the-field-form)).
Change the heading to `## terms: Terms` to clear both errors. Following the
generic numbered suggestion, `## 1. Terms`, clears the heading error but
leaves the required field missing: a field needs its section name.

## The FS opens with its Terms

```toml
[schema.kinds.FS]
folders = ["docs/fs"]

[schema.kinds.FS.fields.terms]
title = "Terms"
presence = "must"
position = "first"
```

`position = "first"` counts every direct section, numbered or named; the lead
does not count. An FS that writes `## 1. Behaviour` before its Terms, one with
no `terms` chapter, and one that titles it `Glossary` get:

```text
docs/fs/FS-login.md:9: error: FS-login.terms is section 2 of 2; kind FS places it first; move the section there
docs/fs/FS-logout.md:1: error: FS-logout has no terms field; kind FS declares it; add its terms section
docs/fs/FS-reset.md:5: error: FS-reset.terms is titled "Glossary"; kind FS titles it "Terms"; retitle the heading to match
```

A missing field is reported at the declaration's title line; every other
finding at the chapter's heading.

## The AR places itself, then its Terms

```toml
[schema.kinds.AR.fields.placement]
presence = "must"
position = "first"

[schema.kinds.AR.fields.terms]
title = "Terms"
presence = "must"
after = "placement"
```

`after` means **directly** after: the `terms` chapter is the section right
after `placement`, not anywhere below it. Fields linked by `after` form a chain,
and a chain whose head is `first` fills the opening sections in chain order
([§FS-fields.order](../functional-spec/FS-fields.md#order-order)). Three more rules keep one mistake to
one finding:

- An absent field is skipped and the next takes its place. An AR with no
  `placement` must open with `terms`, and gets only `missing-field` for
  `placement`.
- Each field is judged against its nearest *present* predecessor, where that
  predecessor actually stands. An AR that writes `1. Design`, `placement`,
  `terms` gets one finding, for `placement`; `terms` is still directly after it.
- A chain whose head has no `position` fixes only the relative order; if its
  head is absent, the next field is unconstrained.

```text
docs/ar/AR-late.md:9: error: AR-late.placement is section 2 of 3; kind AR places it first; move the section there
docs/ar/AR-mis.md:13: error: AR-mis.terms is section 3 of 3; kind AR places it directly after placement; move the section there
docs/ar/AR-noplace.md:1: error: AR-noplace has no placement field; kind AR declares it; add its placement section
```

## A decision has a status, and nothing it does not declare

```toml
[schema.kinds.DF]
folders = ["docs/df"]
closed = "must"

[schema.kinds.DF.fields.status]
one_of = ["Proposed", "Accepted", "Superseded"]
presence = "must"
position = "first"
```

A field with `one_of` is a scalar: its display name *is* its value, so a DF
opens with `## status: Accepted`, and scripts read it from
`grund show --format json` knowing it is one of three spellings. The body below
the heading is ordinary prose. `closed` makes the kind admit its fields and
numbered sections and nothing else, on the channel its strength names. The same
shape fits a DA.

```text
docs/df/DF-session.md:3: error: DF-session.status is "Maybe"; kind DF admits Proposed, Accepted, Superseded; retitle the heading to one of them
docs/df/DF-session.md:7: error: DF-session.surprise is not a field of kind DF, which is closed; its fields: status; rename it to one of them, number it, or add it to the kind's fields
```

A scalar takes no `title`, because its title is its value, and no `content`,
because its content is its title; the loader refuses either beside `one_of`.

## A values chapter

```toml
[schema.kinds.AR.fields.values]
content = "values"
```

A `content = "values"` field is the version-2 spelling of v1's
`value_chapter`: every named child of that chapter is a value root, with the
same coordinates, the same `invalid-value-declaration` findings and the same
prerequisites ([§FS-fields.content.values](../functional-spec/FS-fields.md#contentvalues-a-values-field-is-the-value-chapter)). What the field adds is optional: a
`presence`, a `position` or a `title`. A kind has at most one values field. The
[values guide](values.md) covers writing the values themselves.

## Rule and value kinds

```toml
[schema.kinds.CONST]
folders = ["values"]
form = "value"

[schema.kinds.RULE]
folders = ["docs/rules"]
form = "rule"
```

`form = "value"` is v1's `values = true` and `form = "rule"` is v1's
`rules = true`: the same declarations, findings and prerequisites, refused with
the same messages at the `form` key ([§FS-fields.forms](../functional-spec/FS-fields.md#forms-form-lowers-to-the-version-1-contracts)). A value kind's
children are numbered components, so it takes no fields and cannot be closed;
neither can a kind that is not citable or one whose declarations are fetched.
A rule kind takes fields like any prose kind, including a values field.

## Strengths and channels

The first four codes take the field's `presence`; `undeclared-chapter` takes the
kind's `closed` ([§FS-fields.strengths.table](../functional-spec/FS-fields.md#strengthstable-which-strength-each-finding-takes)).

| What is wrong | Code | `must` | `warn` | `should` | `may` |
|---|---|---|---|---|---|
| the field is absent | `missing-field` | error | warning | suggestion | not reported |
| it is out of place | `misplaced-field` | error | warning | suggestion | error |
| it shows another title | `mistitled-field` | error | warning | suggestion | error |
| a scalar is not in `one_of` | `invalid-field-value` | error | warning | suggestion | error |
| a named chapter no field declares | `undeclared-chapter` | error | warning | suggestion | open |

`may` waives absence and nothing else: an optional field that is present is
still held to its shape, as an error. Write `presence = "warn"` when the shape
itself should be soft. An error exits 1, a warning stands and exits 0, and a
suggestion appears only with `--suggestions`. All five codes work with `--only`
and `--ignore`, and reach the language server unchanged.

A field written twice is the existing `duplicate-section` error, and the field
checks read its first occurrence.

## What the loader refuses

A malformed shape is a config error at the key that makes it so, and the run
exits 2 before anything is checked
([§FS-fields.order.refusals](../functional-spec/FS-fields.md#orderrefusals-what-the-loader-refuses)):

| Written | Refused with |
|---|---|
| `position` and `after` on one field | `field AR.terms sets both position and after; a field takes one placement` |
| two fields `first`, or two `last` | `fields AR.placement and AR.scope are both first; only one field can be` |
| two fields `after` one field | `fields AR.terms and AR.scope both come directly after AR.placement; chain one after the other` |
| `after` naming no field | `after = "plcement" names no field of AR; its fields: placement, terms, values` |
| `after` naming the `last` field | `AR.why is last; nothing can come after it` |
| a cycle | `AR.a and AR.b each come after the other; one of them needs position or no after` |
| `one_of` beside `title` | `field DF.status sets one_of and title; a scalar's title is its value` |
| an empty or repeated `one_of` | `field DF.status has an empty one_of; a scalar needs at least one value` |
| a field on a value kind | `CONST is form = "value": its children are numbered components, so it takes no fields` |

## The agent is told the shape

`grund init` writes each kind's fields into the managed block as a
`### Declaration shapes` section, in the words of `grund schema`'s field lines,
so an agent reads the shape before it writes a declaration rather than learning
it from the finding ([§FS-init.2.3.5.12.2](../functional-spec/FS-init.md#235122-declaration-shapes)):

```markdown
### Declaration shapes

`grund check` holds each declaration of these kinds to these fields, as `grund schema` lists them:

- **DF**
  - field status must · first · one of Proposed, Accepted, Superseded
  - closed must
```

The section sits in the `AGENTS.md` of the directory above the kind's places,
or in the root block under `[presentation] rules = "root"`
([scoped guidance](init.md#scoped-guidance-in-a-version-2-config)). A declared
field moves the block to v21, or v22 where a rule kind is enabled, so the first
`grund check` after adding one reports the block outdated until `grund init`
runs.

## Fields beside chapter rules

A chapter rule such as `Each FS must have exactly one Terms chapter.` stays
valid beside a `terms` field, and where both see the same absence the field's
finding is the one reported. The [chapter-rules guide](rules.md#fields-and-chapter-rules)
says when to reach for which.
