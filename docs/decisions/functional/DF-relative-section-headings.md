# DF-relative-section-headings: a heading may write only the component it adds

**Date:** 2026-10-12

## status: Accepted

## context: Context

Named sections ([issue #178](https://github.com/agent-grounds/grund/issues/178)) made every citable heading carry its complete path. The login example in the README reads:

```markdown
# FS-login: Login
## format: Formatting
### format.1: Username
```

The child repeats `format`, and every deeper level repeats everything above it. That is authoring work, and a second copy of a fact the outline already states, which can drift from it. [Issue #643](https://github.com/agent-grounds/grund/issues/643) asks for the child to write only `.1` while every citation and query still uses `FS-login.format.1`. The owner approved pursuing it and revisiting the full-path requirement, on condition that canonical addresses, the meaning of existing documents and backward compatibility are kept ([§GOAL-friendliness-first](../../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible)).

The design was argued in an agora, whose verdict the owner ruled on in [the ruling on #643](https://github.com/agent-grounds/grund/issues/643): V-D01 chose a generated lock for cited relative addresses under three conditions, and V-D02 kept the form to Markdown declaration bodies in this release.

## decision: Decision

### decision.form: A leading dot and one component

A relative heading writes `.` and one component, then the title syntax the complete-path heading would use: `### .1: Username` under `## format: Formatting` is `format.1` ([§FS-declarations.section.relative.form](../../functional-spec/FS-declarations.md#sectionrelativeform-the-form)). One dot and one component, because a heading that wrote `.a.b` would be a second way to write a path the parent already supplies. The title syntax is the complete form's, rather than a colon after every relative component, so a numeric outline keeps one title syntax.

The form needs no setting and serves every outline. It is unambiguous because no passing repository contains it: on grund 0.17, every dotted heading inside a Markdown declaration body is an `unmarked-heading` error at exit `1`, with named sections on or off and in every depth mode. Giving it a meaning moves no verdict a default `check` reaches, so it needs no warning release.

### decision.parent: The parent is exactly one level up

The parent is the nearest shallower heading, which must be exactly one level up and a usable section, or the declaration heading ([§FS-declarations.section.relative.parent](../../functional-spec/FS-declarations.md#sectionrelativeparent-the-parent)). Anything else is an error naming the parent it needed ([§FS-declarations.checks.relative-section-parent](../../functional-spec/FS-declarations.md#checksrelative-section-parent-a-relative-heading-without-its-parent)) rather than a guess at the nearest heading that would do ([§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution)). Depth is then judged on the canonical path, so a relative heading never carries a depth finding of its own.

### decision.where: Markdown declaration bodies only

The form is recognised only where today's verdict on it is already an error: inside a Markdown declaration body ([§FS-declarations.section.relative.where](../../functional-spec/FS-declarations.md#sectionrelativewhere-markdown-declaration-bodies-and-doc-comments-from-0190)). In a source doc-comment the same dotted line is prose today and passes, so giving it a meaning would move a verdict; that follow-up takes the warning route on its own, tracked under #643.

### decision.shown: Canonical where a coordinate is copied, bytes everywhere else

`--toc` and the title hover print the canonical path in place of the relative one and keep the title ([§FS-declarations.section.relative.shown](../../functional-spec/FS-declarations.md#sectionrelativeshown-how-a-relative-heading-is-shown), [§FS-lsp.1.2.4](../../functional-spec/FS-lsp.md#124-the-title-hovers-text)), because those are where a reader copies a coordinate from. Every read of the text prints the file's bytes, and no command rewrites a heading between the two forms ([§FS-fmt.2.3](../../functional-spec/FS-fmt.md#23-what-is-never-rewritten)).

For one line that choice was replaced: a read of a selected section prints its own heading, the slice's first line, with the canonical path as well ([§DF-show-selected-relative-heading](DF-show-selected-relative-heading.md#df-show-selected-relative-heading-a-read-of-a-selected-section-names-the-section-it-read)). Every other heading of a body read, the map and the hover stand as decided here.

## lock: A cited relative address is locked to its text

A complete-path heading carries its address, so it moves with its text. A relative heading does not: swap two `### .1` blocks under different parents and every citation of the old address reads other text, with no error. That is the silent retarget the ticket rules out. The ruling chose a generated, committed lock over an identity tag on every heading ([§FS-section-lock](../../functional-spec/FS-section-lock.md#fs-section-lock-a-cited-relative-section-keeps-its-text-or-check-says-where-the-text-went)), under three conditions, each met there:

1. It guards only addresses the tree still cites; a stale entry is an `fmt --check` row that `fmt --write` drops, never a `check` error, and a moved address still cited follows the pointer protocol ([§FS-section-lock.write.rules](../../functional-spec/FS-section-lock.md#writerules-add-refresh-drop), [§FS-section-lock.move](../../functional-spec/FS-section-lock.md#move-moving-a-cited-section-on-purpose)).
2. It is sharded per cited declaration, one sorted entry per line, so it never becomes a file every change edits ([§FS-section-lock.file.shard](../../functional-spec/FS-section-lock.md#fileshard-one-file-per-cited-declaration)).
3. The edit-plus-move gap is accepted and documented with the case that shows it ([§FS-section-lock.limits.edit-and-move](../../functional-spec/FS-section-lock.md#limitsedit-and-move-a-section-moved-and-edited-in-one-change)). No second key narrows it, because a title key would report an in-place retitle and edit whenever another section carries the old title.

Copying a cited section and then editing the original is the one false positive, and the finding's message names its way out ([§FS-section-lock.move.copy](../../functional-spec/FS-section-lock.md#movecopy-copy-then-edit-the-original)).

## amends: What it amends in the named-sections design

It amends two items of #178's design record. Item 2, "every citable heading carries its complete effective path", now reads: every citable heading carries its complete path or, in a Markdown declaration body, a relative path whose parent supplies the rest ([§FS-config.3.3.1](../../functional-spec/FS-config.md#331-named-components)). Item 7, a named heading "appears verbatim in `--toc`", now holds for complete-path headings only; a relative heading appears at its canonical path.

What stays in force: one citable address per section; no inferred title slug; no dual addressing, since `.1` is never a coordinate; no rename map, since the lock maps an address to its text and never an old address to a new one; and no automatic migration, since no command converts between the two forms.

## compatibility: Compatibility

No default `check` verdict moves ([§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise)): every heading this gives a meaning to was an error, and nothing outside Markdown declaration bodies changes. Three things do move, and the release note names each:

- A run with `--ignore unmarked-heading` that passes on a body holding dotted headings can now fail through `relative-section-parent` or `duplicate-section`; the lock codes fire only on a citation of a relative address, which dangles today.
- A malformed dotted heading's `unmarked-heading` message gains an appended clause; its prefix and code are unchanged ([§FS-errors.3](../../functional-spec/FS-errors.md#3-message-text)).
- The managed agent block moves to v16, or v17 where a rule kind is enabled, so every repository carrying it reports `agents-init` once until `grund init` runs ([§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)).

## rejected: Rejected alternatives

- **An opt-in setting.** The same bytes would mean two things in two repositories, and the marker already makes the form unambiguous.
- **A colon after every relative component.** It would give a numeric outline a second title syntax.
- **An identity tag on every heading.** It puts metadata on every heading, cited or not, makes a relative heading cost more to write than the complete one it replaces, and fails the ticket's own example as written.
- **Comparing titles or anchors.** It misses a swap of two sections with the same title.
- **Reinterpreting existing numeric headings as relative.** It changes the meaning of existing documents.

## release-note: Release note

- [§FS-declarations.section.relative](../../functional-spec/FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds), [§FS-section-lock](../../functional-spec/FS-section-lock.md#fs-section-lock-a-cited-relative-section-keeps-its-text-or-check-says-where-the-text-went), [§FS-errors.5.5](../../functional-spec/FS-errors.md#55-the-check-code-catalog): **a section heading may write only the component it adds.** Inside a Markdown declaration body, `### .1: Username` under `## format: Formatting` declares `FS-login.format.1`; citations, `refs`, `list`, completion, JSON, diagnostics, `--toc` and the title hover all use the canonical path, and the default read, `--brief` and `--full` print the heading as written. A relative heading needs its parent exactly one level up, or fails with the new `relative-section-parent` error. Citing one needs an entry in the committed section lock, `.grund/sections/<ID>.lock`, or `.grund/sections/<alias>/<ID>.lock` of the citing project for a citation into another workspace member: run `grund fmt --write` after citing a relative section and commit what it writes, or `check` reports the new `unrecorded-section-citation`; `section-address-changed` reports a cited section whose text moved to another address, and `invalid-section-lock` a lock line it cannot read. **Who this breaks:** no default `check` run, because every dotted heading in a declaration body was already an error and nothing outside one changes. A run with `--ignore unmarked-heading` over a body holding dotted headings can now fail through `relative-section-parent` or `duplicate-section`. A malformed dotted heading's `unmarked-heading` message gains an appended `; under <parent> a relative heading is written <heading>` clause, so a consumer matching the whole message sees it change and a prefix or `code` consumer does not. The managed agent block moves to v16, or v17 where a rule kind is enabled, so each repository carrying it reports `agents-init` once until `grund init` runs. Source doc-comments are unchanged in this release. Closes [issue #643](https://github.com/agent-grounds/grund/issues/643). (PR #666)
