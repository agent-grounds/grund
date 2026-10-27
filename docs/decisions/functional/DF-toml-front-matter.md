# DF-toml-front-matter: A kind's front-matter field reads a TOML block as it reads a YAML one

**Date:** 2026-10-27

## status: Accepted

## context: A `+++` page could not be held to its kind

[§DF-front-matter-field](DF-front-matter-field.md#df-front-matter-field-a-kinds-front-matter-field-holds-its-files-yaml-front-matter-with-the-blocks-top-level-keys-as-its-sections) made a declaration's front matter the
`front-matter` field of a kind that declares one, and read only YAML. Its decision.5
reported a `+++` block in such a kind as unreadable, because reading TOML needed a
parser that keeps spans and a span tree of its own. So a Hugo or Zola page, or any
format that writes `+++`, could not be held to its kind without rewriting its front
matter as YAML ([#1057](https://github.com/agent-grounds/grund/issues/1057)).

## decision: A `+++` block is read as the field

### decision.1: The same keys, coordinates and findings

In a kind that declares the field, a `+++` block's content is read as one TOML 1.1
document ([§FS-fields.front-matter.reading](../../functional-spec/FS-fields.md#front-matterreading-reading-the-block)), and its top-level keys are the field's
sections, with the same coordinates, surfaces and findings a `---` block's keys get. A
valid block in such a kind stops being reported. A block TOML cannot parse is
`unreadable-front-matter`, naming what the TOML parser found and the line it is at
([§FS-fields.checks.unreadable-front-matter](../../functional-spec/FS-fields.md#checksunreadable-front-matter-unreadable-front-matter)). A kind that does not declare the field
parses no block, so its `+++` block declares nothing, as [§FS-check.1.1.5.2](../../functional-spec/FS-check.md#1152-front-matter-holds-no-heading) says.

### decision.2: A key is the first component of what the document writes

TOML reaches a key in more ways than YAML does: a dotted key, a `[table]` header and an
`[[array]]` header each write the top-level key their first component names
([§FS-fields.front-matter.keys](../../functional-spec/FS-fields.md#front-matterkeys-its-keys-are-its-sections)). Every contribution TOML allows is one key written once;
every one TOML refuses because of what the key already is counts as the key written
twice, `duplicate-section`, as a YAML key written twice is. The reading never admits,
at the level grund judges, a page TOML refuses, and its findings stay those of the same
keys between `---` lines.

### decision.3: A value keeps its spelling

A string's value is its decoded text, and every other scalar's its raw spelling, with
no TOML type applied ([§FS-fields.front-matter.values](../../functional-spec/FS-fields.md#front-mattervalues-a-keys-value-is-its-decoded-text)), as a YAML value is given no
YAML type ([§DF-front-matter-field.decision.3](DF-front-matter-field.md#decision3-a-value-is-its-decoded-text)).

## rejected: Rejected alternatives

- **Keeping `+++` unreadable.** It holds the formats that write TOML to nothing, and the
  only cost it saved was a reader, which
  [§DA-toml-front-matter-reader](../architectural/DA-toml-front-matter-reader.md#da-toml-front-matter-reader-one-toml-event-parser-builds-the-span-tree-a-toml-front-matter-block-is-read-through) pays once.
- **Reporting a redefined key as a TOML parse error.** TOML calls it one, but the same
  repetition between `---` lines is `duplicate-section`, so one page would get two
  answers for one defect depending on its delimiter.
- **Applying TOML's types.** `1_000` would become `1000` and a date a date, so a
  `one_of` entry written as the page writes it would not match.

## compatibility: Only a refused page starts to pass, and the old message never shipped

[§REQ-backwards-compatibility](../../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly) holds that an upgrade never changes a verdict
quietly, and never turns a passing repository into a failing one behind its maintainer's
back. Nothing here fails a tree that passes: a valid `+++` block in a kind that declares
the field moves from fail to pass, which the release note states, a block that does not
parse stays an error, and a repository that declares no `front-matter` field sees no
change.

The text of that error changes, and the text of an existing finding changes only through
[§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path). The text it replaces, `is TOML, which grund does not read yet`, is
not one: no release carried it. The latest release, 0.20.0 of 2026-10-23, came before
[§DF-front-matter-field](DF-front-matter-field.md#df-front-matter-field-a-kinds-front-matter-field-holds-its-files-yaml-front-matter-with-the-blocks-top-level-keys-as-its-sections) merged, and this decision ships in the release that first carries
the field, so the release note states the new input rather than a changed message, and
[§DF-front-matter-field](DF-front-matter-field.md#df-front-matter-field-a-kinds-front-matter-field-holds-its-files-yaml-front-matter-with-the-blocks-top-level-keys-as-its-sections)'s own note no longer says a `+++` block is unreadable.

## release-note: Release note

- [§DF-toml-front-matter](DF-toml-front-matter.md#df-toml-front-matter-a-kinds-front-matter-field-reads-a-toml-block-as-it-reads-a-yaml-one): **A kind's `front-matter` field also reads a `+++` (TOML)
  block**, with the same keys, coordinates and findings as a `---` (YAML) block. A dotted
  key or a `[table]` header writes the top-level key its first component names, and a key
  TOML refuses to define again is a `duplicate-section`. A block that does not parse is
  `unreadable-front-matter`, naming the TOML parser's message and line.
