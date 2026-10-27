# DF-front-matter-field: A kind's front-matter field holds its file's YAML front matter, with the block's top-level keys as its sections

**Date:** 2026-10-27

## status: Accepted

## context: The facts a format keeps in front matter could not be held

[§DF-front-matter](DF-front-matter.md#df-front-matter-a-markdown-files-front-matter-is-a-region-of-its-own-owned-by-the-files-first-declaration-and-never-rewritten) gave a Markdown file's front matter an extent and an owner, and parsed
none of it. Many formats keep a record's defining facts there: a MADR 4.0 record its
`status` and `date`, an Open Knowledge Format concept its `type`, a kramdown-rfc draft
its metadata, a Doorstop item its verification method
([#1042](https://github.com/agent-grounds/grund/issues/1042)). A kind could require
body chapters, at a strength and with a closed list of values ([§FS-fields](../../functional-spec/FS-fields.md#fs-fields-a-v2-kind-states-the-shape-of-its-declarations-and-check-holds-every-declaration-to-it)), but it could
say nothing about the block above its first heading. So a project that wanted grund to
hold those facts had to copy each into a chapter, where the copy and the original could
disagree, and the agent guidance taught the copy
([#1048](https://github.com/agent-grounds/grund/issues/1048)).

## decision: The block is a field of its declaration, read on request

### decision.1: One field, named `front-matter`

On a kind that declares `[schema.kinds.<KIND>.fields.front-matter]`, that field is the
front matter of its declaration's file, and the block's top-level keys are its sections
([§FS-fields.front-matter](../../functional-spec/FS-fields.md#front-matter-front-matter-is-a-field)). The field record, its strengths, its four field codes and the
one section map carry it, so nothing gains a second path: a key is required, held to a
`one_of`, closed, cited, globbed, measured and ruled on as a chapter is.

### decision.2: Read only where a kind asks

A block is parsed only when its owner's kind declares the field
([§FS-fields.front-matter.reading](../../functional-spec/FS-fields.md#front-matterreading-reading-the-block)). A repository that declares none parses no YAML it did
not parse before, and every byte it reports stays as it is.

### decision.3: A value is its decoded text

A key's value is the text the YAML parser decodes, compared with `one_of` as written and
given no type, so `1.10` stays `1.10` and `no` stays `no`
([§FS-fields.front-matter.values](../../functional-spec/FS-fields.md#front-mattervalues-a-keys-value-is-its-decoded-text)). A type would make grund a schema validator for one
format's conventions, and every format collected in #1042 writes its facts as words.

### decision.4: Whole-declaration reads stay the same

The coordinates lie outside the body ([§FS-fields.front-matter.coordinates](../../functional-spec/FS-fields.md#front-mattercoordinates-its-coordinates)). The body
still starts at the heading, as [§DF-front-matter.2.2](DF-front-matter.md#22-its-citations-belong-to-the-files-first-declaration) keeps it, so no lead, fingerprint
or section lock moves, and a whole-declaration read prints and lists what it printed
before.

### decision.5: YAML now, TOML later

The span tree grund already has reads YAML ([§DA-yaml-value-reader](../architectural/DA-yaml-value-reader.md#da-yaml-value-reader-one-yaml-event-parser-builds-one-span-tree-and-value-rules-are-a-pass-over-it)), and every format
collected in #1042 writes YAML. A `+++` block in a kind that declares the field is
reported as unreadable, `front-matter is TOML, which grund does not read yet`
([§FS-fields.checks.unreadable-front-matter](../../functional-spec/FS-fields.md#checksunreadable-front-matter-unreadable-front-matter)). Reading it needs a TOML parser that keeps
spans and a span tree of its own, which is
[#1057](https://github.com/agent-grounds/grund/issues/1057). When that lands, a valid
`+++` block stops being reported, a change from fail to pass.

### decision.6: The tree bounds its nesting

The span tree recursed once per nested collection, so a value source nested 10000 deep
ran it out of stack. Reading every opted-in file's front matter with it would hand that
crash to any Markdown file, so the tree refuses a collection nested more than 128 deep
with an error, for both of its readers ([§FS-values.5.3](../../functional-spec/FS-values.md#53-incomplete-input-and-deterministic-output)).

## rejected: Rejected alternatives

- **A separate `[schema.kinds.<KIND>.front_matter]` table.** Two records of one shape,
  the parallel path [§DF-kind-fields.2.4](DF-kind-fields.md#24-the-value-chapter-moves-into-the-field-list) rules out, and the keys would be kept out of
  citations, globs and rule subjects.
- **Listing the coordinates in a whole-declaration `--toc`.** A table-of-contents line is
  a heading as written, and these coordinates have none. Listing them would change that
  read for every opted-in declaration.
- **TOML now.** `toml_parser` brings two dependencies the tree does not have, and its
  span tree must understand dotted keys, `[table]` headers and quoted keys before it can
  name a top-level key. A hand-written subset is the "two readers of one language"
  [§DA-yaml-value-reader.3.1](../architectural/DA-yaml-value-reader.md#31-a-hand-written-reader-of-the-value-subset) rejects. Deferring costs nothing a collected format needs.
- **Reading the keys a merge key `<<` names.** Most YAML tools treat them as present, but
  merging is a value rule, and the tree applies none ([§DA-yaml-value-reader.2.2](../architectural/DA-yaml-value-reader.md#22-one-span-tree-with-no-value-rules-in-it)). A key
  written in the block is a key; one merged into it is not
  ([§FS-fields.front-matter.keys](../../functional-spec/FS-fields.md#front-matterkeys-its-keys-are-its-sections)).

## compatibility: Use the pre-1.0 licence, not a deprecation window

`[schema.kinds.<KIND>.fields.front-matter]` loads today and means a body chapter,
`## front-matter:`. After this decision it means the file's front matter. That moves
verdicts: a config that sets `title`, `position`, `after`, `content` or `one_of` on the
field, or another field's `after = "front-matter"`, stops loading at that key with exit
`2`, and a config that sets only `presence` now asks for front matter and reports each
`## front-matter:` chapter as `front-matter-heading`.

[§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered) lets a verdict move only by a deprecation window, a loud
migration, or a correction. This decision uses instead the licence of
[§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise): before `1.0`, a surface may change without an alias where
carrying the alias would cost more than the rename. The alias here would be a release in
which `[fields.front-matter]` means both the chapter and the block. Every check, view and
coordinate would answer two ways for one name, and a `## front-matter:` heading would be
a chapter and a refusal at once, for a release. That costs more than it protects: no
config in grund, rhei, ephor, fissile or the workspace that holds them declares the
field, and nothing taught it. Every break is loud and located, and it names its remedy:
rename the chapter. The licence expires at `1.0`, and this decision does not stretch it
past that.

## release-note: Release note

- [§DF-front-matter-field](DF-front-matter-field.md#df-front-matter-field-a-kinds-front-matter-field-holds-its-files-yaml-front-matter-with-the-blocks-top-level-keys-as-its-sections): **A kind's `front-matter` field now holds its file's YAML front
  matter, with the block's top-level keys as its sections.** Declare
  `[schema.kinds.<KIND>.fields.front-matter]` and its
  `[schema.kinds.<KIND>.fields.front-matter.keys.<key>]` tables to require keys and hold
  their values to a `one_of`. **Who this breaks:** a config that used `front-matter` as
  the name of a body chapter. Setting `title`, `position`, `after`, `content` or `one_of`
  on it, or `after = "front-matter"` on another field, now stops the config from loading,
  and a `## front-matter:` chapter of such a kind is now a `front-matter-heading` error.
  Rename the chapter and its field. A `+++` (TOML) block in a kind that declares the
  field is reported as unreadable until
  [#1057](https://github.com/agent-grounds/grund/issues/1057). A repository that declares
  no `front-matter` field sees no change.
