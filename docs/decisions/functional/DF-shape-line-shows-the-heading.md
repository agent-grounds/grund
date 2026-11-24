# DF-shape-line-shows-the-heading: A field's line shows the heading to write, with its placement and content in check's words

**Date:** 2026-11-16

Settles how `grund schema` and the managed block's `### Declaration shapes` describe a field, as [§FS-schema.4.1](../../functional-spec/FS-schema.md#41-fields-are-lines-inside-their-kinds-node) and [§FS-init.2.3.5](../../functional-spec/FS-init.md#235-citation-directions) specify, and why that shares the block bump to v47–v50 rather than making one of its own ([§FS-init.2.3.7](../../functional-spec/FS-init.md#237-the-block-version)). It supersedes [§DF-field-line-names-its-title](DF-field-line-names-its-title.md#df-field-line-names-its-title-a-field-line-names-the-heading-title-the-field-is-held-to). Serves [§FS-fields.requirements.record](../../functional-spec/FS-fields.md#requirementsrecord-the-agent-and-a-schema-view-read-the-shape-the-checks-read--realized): an agent reads the shape the checks hold before it writes, so a line that leaves out what check enforces tells it the shape only after the fact ([§GOAL-agent-grounding](../../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work)). Reported as [grund#1373](https://github.com/agent-grounds/grund/issues/1373).

## status: Accepted

## 1. Context

The field lines named a field's presence, then `first`, `last` or `after <field>`, then `one of A, B` or `values`, then its title. `grund check` holds `after X` as *directly* after X ([§FS-fields.order.after](../../functional-spec/FS-fields.md#orderafter-after-means-directly-after)), a scalar's heading title as its value ([§FS-fields.content.one-of](../../functional-spec/FS-fields.md#contentone-of-a-scalars-value-is-its-display-name)), a value chapter as named roots with numbered parts ([§FS-values.2.5](../../functional-spec/FS-values.md#25-chapter-declared-value-roots)), and a value file as members that are nonempty arrays ([§FS-values.2.2.1](../../functional-spec/FS-values.md#221-source-shape)). None of that showed on the line. Asked for the chapters reason, goals, example, terminology, requirements, an agent kept that order, because terminology does come after reason, and check refused it. In the report's agent runs, with grund unavailable to the agents, today's lines passed the order task in 1 of 12 runs and the values task in 5 of 12. The same section, with each field shown as its own heading and its placement in words, passed 11 of 11 and 11 of 12, for about 39 more input tokens.

## 2. Decision

### 2.1 Each line shows the heading to write

A named field's line is its heading as a code span, `## <name>: <title>`, then its presence and placement in parentheses, then, after a colon, what the chapter holds where it is not prose. A scalar shows `<one of A, B>` in the title's place and says `the title is the value`. A value chapter says `only values`, and shows its root `### <field>.<name>: <title>` and its part `#### .1: <number or word>`. A `front-matter` field is no heading, so its line shows the `---` or `+++` block at the top of the file, its keys nested under it as `` `<key>: …` ``, and its own `closed` as a line of its own. A value kind's line shows the file its values take, in each form its home can hold, and the citation `§<ID>.1` that its first element answers to. A fetched kind's line says `grund fetch` writes the file and shows no form to write by hand. A kind's `closed` reads `no other chapter except numbered ones`.

The block prints the line without the `field <name>` key the view puts before it, because the heading already names the field. `grund schema`'s table, Mermaid and DOT print the same line after that key. One function writes it for every view, over the slots check reads ([§AR-system.principle](../../architecture/README.md#principle-one-fact-one-point-one-implementation)).

### 2.2 The depth objection

[§DF-field-line-names-its-title.3](DF-field-line-names-its-title.md#3-alternatives-considered) rejected the full heading because its depth depends on the declaration. A field is always its declaration's direct section ([§FS-fields.identity.name](../../functional-spec/FS-fields.md#identityname-a-field-is-the-direct-section-of-its-name)), one level below its heading. So once the declaration is written `# <ID>: …`, as Markdown declarations and doc-comment bodies both write theirs, the field's depth is a fact about the field. The one exception, a declaration written at `##` or deeper, takes one sentence in the block's lead and one legend line in `grund schema` ([§FS-schema.8.4](../../functional-spec/FS-schema.md#84-the-legend-says-where-a-fields-heading-sits)) rather than a note on every line. That record's other reason was that the block already teaches `## name: Title` in general. The report's runs answer it: the form that turned 1 of 12 into 11 of 11 is each field's own heading.

### 2.3 The words are check's, and placeholders are not values

- `directly after` is what `misplaced-field`, `grund config explain` and [§FS-fields.order.after](../../functional-spec/FS-fields.md#orderafter-after-means-directly-after) already say. The report's sketch wrote `right after`; one phrase for one fact keeps the views and check alike.
- The strength words stay `must`, `warn`, `should` and `may`. The sketch's `required` would be a second name for one of four strengths, with nothing to match it for the other three, while the citation directions, the chapter rules and the legend all use the four.
- Where the author's text goes, a line shows `<title>`, `<one of …>`, `<name>`, `<value>`, `<ID>` or `<number or word>`, never a plausible value, because agents copy what they see. A value part is `<number or word>` and `a unit is a part of its own`, because a citation binds a number only to a part that is exactly that number ([§FS-values.4](../../functional-spec/FS-values.md#4-exact-equality)).
- A purely numbered part of a Markdown value declaration is written `## 1. <number or word>`. `## 1: …` is read as an unmarked heading, and the declaration is refused ([§FS-values.2.1](../../functional-spec/FS-values.md#21-markdown-declarations)). `#### .1:` stays for the part under a named value root, whose path has a name in it.

### 2.4 The code span carries over

[§DF-field-line-names-its-title.2.2](DF-field-line-names-its-title.md#22-the-title-is-a-code-span) carries over unchanged: fences one backtick longer than the longest run inside, a space inside each fence where the text begins or ends with a backtick, and a control character as its TOML basic-string escape. Its [§DF-field-line-names-its-title.2.1](DF-field-line-names-its-title.md#21-the-title-is-the-last-fact-on-the-line) (the title is the last fact, every earlier prefix kept) and [§DF-field-line-names-its-title.2.3](DF-field-line-names-its-title.md#23-untitled-lines-keep-their-bytes) (untitled lines keep their bytes) are dropped on purpose, because every line changes in this bump anyway.

### 2.5 It shares the v47–v50 bump

Every block that renders a `### Declaration shapes` section renders the scoped pair. In the same release, [§DF-trigger-sentence-names-the-id](DF-trigger-sentence-names-the-id.md#df-trigger-sentence-names-the-id-the-blocks-trigger-sentence-names-the-id-that-follows-the-trigger) had already moved both pairs, v43/v44 to v47/v48 and v45/v46 to v49/v50, and no release had written them ([§FS-init.2.3.7](../../functional-spec/FS-init.md#237-the-block-version)). So this change shares that bump: a config that renders a declaration shape renders v49, or v50 with a rule kind, and a block that renders no shape is byte for byte the block that bump renders. Moving only the scoped pair, v45/v46 to v47/v48 with v43/v44 kept, was the plan only had no other change moved both pairs first.

### 2.6 What does not change

`grund check`'s verdicts and messages, `grund config explain` and `grund config show` stay as they are. The legend's three strength lines stay; it gains only the depth line.

## 3. Alternatives considered

| Option | Why rejected |
|---|---|
| A legend entry for `after`, `first`, `last`, `titled` | It fixes `grund schema` only, because the block has no legend, and the report measured it at about 179 more tokens against about 39. |
| `right after` and nothing else | It fixes the order case only; the scalar, value chapter and value file cases stay. |
| New words in the block only, the table keeping today's line | `after reason` would stay in the view, where the next report of this defect would come from, and the block would describe the shape in words of its own. |
| One bump of all four members, to v47–v50 | It outdates every block whose bytes did not change, for the reason [§DF-scoped-guidance](DF-scoped-guidance.md#df-scoped-guidance-a-version-2-rows-guidance-is-written-above-its-places-with-a-root-pointer-for-agents-that-do-not-load-nested-files) rejected one. |

## release-note: Release note

- [§FS-schema.4.1](../../functional-spec/FS-schema.md#41-fields-are-lines-inside-their-kinds-node), [§FS-init.2.3.5](../../functional-spec/FS-init.md#235-citation-directions), [§FS-init.2.3.7](../../functional-spec/FS-init.md#237-the-block-version): **`grund schema` and the managed block show each field as the heading to write.** A field's line is now its heading, ``` `## terminology: Terminology` (must, directly after reason) ```, with a scalar's `<one of …>` and `the title is the value`, a value chapter's `### values.<name>:` roots and `#### .1:` parts, a front-matter block and its keys, and a value kind's file form and its `§<ID>.1` citation. `grund schema`'s legend gains one line saying a field sits one level below its declaration's heading. A script that matches `grund schema` field lines sees new text; `grund config show` remains the machine-readable reader. A config that renders a declaration shape renders the managed block's v49/v50, along with the v47–v50 bump of [§DF-trigger-sentence-names-the-id.release-note](DF-trigger-sentence-names-the-id.md#release-note-release-note): `grund check` reports an earlier block outdated, and the block `grund init` writes in its place shows the new lines. **What an upgrader does:** run `grund init` once to re-render the block; unmanaged content is kept. No `grund check` verdict changes. (agent-grounds/grund#1373)
