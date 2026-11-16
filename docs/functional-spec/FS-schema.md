# FS-schema: grund schema shows what a repository is held to, read from its configuration alone

`grund schema` prints one view of a repository's schema and rules: which kinds
exist and where they live, which folders are places rather than ID namespaces,
what shape each kind's declarations must have, which units must be grounded, and
who must cite whom and how strongly. It reads only the effective configuration,
so it answers on a tree whose citations are broken, and it needs no browser and
no renderer: the default is a terminal table, and the two export formats are
source text on stdout. It serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible) (what a repository asks
of an author fits one screen) and [§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work) (an agent learns the rules
before it writes, rather than from the findings after). `grund config show` stays
the display of the effective TOML ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)); why the view is its own
command is [§DF-schema-view](../decisions/functional/DF-schema-view.md#df-schema-view-the-schema-view-is-its-own-command-prints-no-json-yet-shows-a-workspaces-root-and-draws-an-any-of-entry-through-one-junction).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (kind, home, citable, field, closed), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure)
(scan, config root, place, complement place, workspace, member), [§FS-terms.terms.6](FS-terms.md#terms6-rules-and-directions)
(direction, unit, grounded), [§FS-terms.terms.9](FS-terms.md#terms9-the-configurations-concerns) (schema, rules) and
[§FS-config-v2.terms](FS-config-v2.md#terms-terms) (strength, ladder). Defines:

- **Schema view**: what `grund schema` prints for one project: its rows as nodes,
  its direction rules as edges, its grounding as badges, and a legend.
- **Node**: one row of the configuration as the view shows it, carrying its
  class, its places and, for a kind, its fields.
- **Junction**: the one node an any-of entry is drawn through in the graph
  formats, so that one obligation is one line from its subject.
- **Badge**: a fact printed on a node rather than drawn as an edge, because it
  names no target.

## 1. The command

```
grund schema [PATH] [--format table|mermaid|dot] [--path-base project|invocation]
```

`--format` picks the printer, `table` by default ([§FS-schema.8](FS-schema.md#8-the-table), [§FS-schema.9](FS-schema.md#9-the-mermaid-source),
[§FS-schema.10](FS-schema.md#10-the-dot-source)). Every format prints the same facts. `[output] format` does not
apply to this command, and `--format` takes no `text` or `json` value
([§FS-schema.13](FS-schema.md#13-what-is-absent)). `--path-base` is accepted as on every page that renders a
config-load error ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)); the paths inside the view are always spelled from
the config root as written, so the flag moves only the path in a config error.

The command's help page ([§FS-cli.2.3](FS-cli.md#23-a-subcommands-help-page)) carries these examples, one per flag:

```
grund schema                         # the quick terminal view
grund schema --format mermaid        # Mermaid source, e.g. for a GitHub comment
grund schema --format dot | dot -Tsvg > schema.svg
grund schema docs/ --path-base=invocation
```

### 1.1 The word `schema` is a subcommand

`schema` is on the subcommand list ([§FS-cli.1](FS-cli.md#1-the-default-subcommand)), so it wins over the default ID
query exactly as `list` and `refs` do: this is the precedence
[§FS-cli.requirements.as-written](FS-cli.md#requirementsas-written-an-invocation-runs-as-written-or-is-refused--realized-directional-for-the-next-subcommand) already writes down, not a new one. Under the
shipped ID formats `schema` was never a valid ID. An ID that an unusual format
spells `schema` stays reachable as `grund show schema`.

## 2. Selection

The configuration is discovered by walking up from PATH, `.` by default, exactly
as `grund config show` discovers it ([§FS-config.1](FS-config.md#1-file-location-and-discovery), [§FS-config.4.2](FS-config.md#42-grund-config-show-path)). A path inside a
workspace member shows that member.

### 2.1 A workspace root shows the root, and names its members

At a workspace root the view is of the root project only. The table ends with one
more line, after a blank line, naming every entry of the root's `[workspace]
members` as written and in written order:

```
members: api, web — run grund schema <member>
```

The Mermaid and DOT sources carry the same line as their last comment line. Every
member is not rendered in turn: a graph consumer expects one graph, and a quick view
does not load every member's configuration ([§DF-schema-view.2.3](../decisions/functional/DF-schema-view.md#23-a-workspace-root-shows-the-root-and-names-its-members)).

## 3. What the view reads

The view is derived from the effective configuration and nothing else. It reads no
declaration body, runs no check, fetches no external fact and builds no
declaration-ID graph, so a tree whose citations are broken, whose index files are
missing or whose declarations are malformed shows the same view as a clean one.

A version-1 and a version-2 file are both read through the project's one set of
records ([§FS-config.concerns](FS-config.md#concerns-every-key-belongs-to-exactly-one-concern)), so one configuration spelled in either version
shows the same view, its header aside. A v1 `require_grounding` with its
`grounding_level` shows as a `must` rung; a v1 value chapter shows as the field it
lowers to; v1's `avoid` and `never` show as `should not` and `must not`.

Only what is configured or effective is shown. An implicit row the project does
have, such as the complement place `code`, is shown; a field or a rule nobody
configured is never invented. The ID grammar, note budgets, lead sizes, sources
and the presentation keys stay with `grund config show`: the view says what a
repository is held to, not every value its configuration evaluates to.

## 4. Nodes

Every row of the configuration is one node, and its class is always written as a
word, never shown by colour or line alone:

| class | the row | table word | Mermaid | DOT |
|---|---|---|---|---|
| citable kind | declares IDs and has a home | `kind` | box | `shape=box` |
| homeless kind | declares IDs and has no home | `kind` with home `no home` | box | `shape=box` |
| place | non-citable, scanned | `place` | box, class `place` (dashed) | `style=dashed` |
| unscanned row | `scan = false` | `place, not scanned` | box, class `unscanned` (dotted) | `style=dotted` |
| complement | every file outside every other place | `complement` | box | `shape=box` |

A folder home is printed with a trailing `/`, a file home as written, and the
complement as `every scanned file no other row claims`. A row with more than one
place lists every one, comma-separated, in the row's place order, files then folders,
each in written order (`docs/a/, docs/b/`; [§FS-config-v2.schema.places](FS-config-v2.md#schemaplaces-rows-and-their-places)).
In a version-2 config a `code` row that lists places is the complement node, and
its home column lists those places in the same way (`crates/, scripts/`) in place
of `every scanned file no other row claims`; no second, implicit complement is
drawn beside it ([§FS-config-v2.schema.places.complement](FS-config-v2.md#schemaplacescomplement-code-is-the-complement-by-name)).

### 4.1 Fields are lines inside their kind's node

A kind's fields follow its node, in header order, one line each: `field <name>`,
then the field's **shape line**, which shows what an author writes and says what
`grund check` then holds it to. A named field's shape line has three parts:

1. the heading to write, as a code span: `## <name>: <title>`, with the field's
   `title` where it sets one, `<title>` where it sets none, and
   `<one of A, B>` for a scalar field;
2. in parentheses, its presence, then its placement where one is set, joined by
   `, `: `first`, `last`, or `directly after <field>` ([§FS-fields.order.after](FS-fields.md#orderafter-after-means-directly-after));
3. after a colon, what the chapter holds where it is not prose:
   `the title is the value` for a scalar ([§FS-fields.content.one-of](FS-fields.md#contentone-of-a-scalars-value-is-its-display-name)), and for a value
   chapter ``only values, each `### <field>.<name>: <title>` with parts `#### .1: <number or word>`; a unit is a part of its own``,
   `<field>` being the field's own name ([§FS-fields.content.values](FS-fields.md#contentvalues-a-values-field-is-the-value-chapter), [§FS-values.2.5](FS-values.md#25-chapter-declared-value-roots)).

A kind whose `closed` strength is not `may` adds the line `closed` after its fields,
whose text is `no other chapter except numbered ones (<strength>)` ([§FS-fields.declare.closed](FS-fields.md#declareclosed-closed-is-a-strength-on-the-row)).
These are the slots the checks read ([§FS-fields.record](FS-fields.md#record-the-record)), so the view and
`grund check` cannot describe two shapes, and the words are check's own:
`directly after` is what `misplaced-field` says. The strength words are the
legend's ([§FS-schema.8](FS-schema.md#8-the-table)).

#### 4.1.1 A heading is shown under a declaration written `# <ID>: …`

A field is always its declaration's direct section ([§FS-fields.identity.name](FS-fields.md#identityname-a-field-is-the-direct-section-of-its-name)), one
level below the declaration's heading, so its depth is a fact about the field
once the declaration's depth is fixed. Every heading a shape line shows is
written under a declaration written `# <ID>: …`, which is also how a doc-comment
declaration body writes its sections. A declaration written `## SWR-001: …`
writes the same field at `###`, and the table's legend says so once
([§FS-schema.8.4](FS-schema.md#84-the-legend-says-where-a-fields-heading-sits)) rather than every line.

#### 4.1.2 The code span and the placeholders

The heading is the exact text `grund check` holds the field's heading to, its
title byte for byte ([§FS-fields.identity.match](FS-fields.md#identitymatch-titles-and-values-match-byte-for-byte)), so the view shows it before the
author writes rather than after `check` refuses the heading
([§FS-fields.requirements.record](FS-fields.md#requirementsrecord-the-agent-and-a-schema-view-read-the-shape-the-checks-read--realized)). Each code span on a shape line is written as a
Markdown code span: its bytes between fences of backticks one longer than the
longest run of backticks inside it, with one space inside each fence where it
begins or ends with a backtick. A control character in a title is written as its
TOML basic-string escape (`\n`, `\t`, `\r`, otherwise `\uXXXX`), so the line stays
one line.

Where the author's own text goes, a line shows a placeholder in angle brackets,
never a plausible value an author could copy: `<title>`, `<one of A, B>`,
`<name>`, `<value>`, `<ID>` and `<number or word>`. A value part is shown as
`<number or word>`, which is narrower than the one-line text check accepts,
because a citation binds a number only to a part that is exactly that number
([§FS-values.4](FS-values.md#4-exact-equality)); `a unit is a part of its own` says where the unit goes.

#### 4.1.3 Front matter is no heading

A `front-matter` field is the block at the top of its declaration's file and not
a section ([§FS-fields.identity.name](FS-fields.md#identityname-a-field-is-the-direct-section-of-its-name)), so its line never shows a heading. Its text is
`` `---` YAML or `+++` TOML front matter at the top of the file (<presence>) ``.
Each of its keys follows as `key <name>`, two columns deeper, with the text
`` `<name>: <one of A, B>` (<presence>) ``, or `` `<name>: <value>` (<presence>) `` for a key
with no values. Where the field's own `closed` is not `may`, a last line `closed`,
two columns deeper too, has the text `no other key (<strength>)`
([§FS-fields.front-matter.record](FS-fields.md#front-matterrecord-the-record-and-what-renders-it)).

#### 4.1.4 A value kind's line names the file its values take

A value kind's numbered children are one line, `field <number>`. It carries no
parentheses, because its presence is always `may` and it takes no placement.
Its text names the form its home holds, by the rule the scan reads a value home
by ([§FS-values.2.2](FS-values.md#22-declarations-from-value-sources), [§FS-config-v2.schema.sources.values](FS-config-v2.md#schemasourcesvalues-a-value-home-reads-a-file-by-its-language)), so it cannot show a form the
scan would not read:

- a file the scan reads with the JSON reader: `` `{"<ID>": [<number or word>, …], …}` in `<file>` ``;
- a file it reads with the YAML reader: `` `<ID>: [<number or word>, …]` in `<file>` ``;
- any other file: `` `# <ID>: <title>` with only `## 1. <number or word>`, `## 2. …` under it in `<file>` `` ([§FS-values.2.1](FS-values.md#21-markdown-declarations));
- a folder: each of those forms the folder can hold, Markdown first, then JSON,
  then YAML, each ending `in a Markdown file in`, `in a JSON file in` or
  `in a YAML file in` and the folder as a code span, joined by `, ` and by
  `, or ` before the last.

The forms are followed by ``: part 1 is `§<ID>.1`; a unit is a part of its own``:
element *i* of a member declares part *i*+1 ([§FS-values.2.2.1](FS-values.md#221-source-shape)), which is the one
thing a reader cannot infer from the file. A fetched value kind's file is written
by its integration ([§FS-fetch](FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot)), so its line teaches no form to write by hand:
it is `` written by `grund fetch` into `<home>`: part 1 is `§<ID>.1` ``.

#### 4.1.5 The issue's RFC kind

For the issue's RFC kind, with an untitled `amendments`:

```text
  RFC   kind                             rfcs/text/
          field summary                  `## summary: Summary` (must, first)
          field motivation               `## motivation: Motivation` (must, directly after summary)
          field guide-level-explanation  `## guide-level-explanation: Guide-level explanation` (must, directly after motivation)
          field amendments               `## amendments: <title>` (may, last)
          closed                         no other chapter except numbered ones (must)
```

The form, and the forms it was chosen over, are [§DF-shape-line-shows-the-heading](../decisions/functional/DF-shape-line-shows-the-heading.md#df-shape-line-shows-the-heading-a-fields-line-shows-the-heading-to-write-with-its-placement-and-content-in-checks-words),
which supersedes [§DF-field-line-names-its-title](../decisions/functional/DF-field-line-names-its-title.md#df-field-line-names-its-title-a-field-line-names-the-heading-title-the-field-is-held-to).

### 4.2 A node no direction rule names is set apart

In the graph formats, a node that is neither the subject nor a target of any
direction rule is drawn inside one box titled `no direction rule names these`, so
a disconnected node reads as unruled rather than lost. In the table such a row
shows `—` in its citations line.

## 5. Directions

Each entry of a subject's `[citations.<KIND>]` (v1) or `[rules.citations.<KIND>]`
(v2) table is one direction: its subject, its strength, and its target or any-of
targets. A subject's `default` other than `may` is shown as the direction
`everything else: <strength>`, after the subject's entries; a project-wide
`default` other than `may` is shown once, as the line
`everything else, for every subject: <strength>` directly under the table's
`citations` heading and as a comment line in the graph formats.

### 5.1 An any-of entry is one obligation

An entry such as `GOAL|FS` is one obligation, met by citing either target, and the
view never shows it as two. In the table it is one cell, written as configured. In
Mermaid and DOT it is one junction: one line from the subject to the junction,
labelled with the strength and with no arrow head, then one unlabelled arrow in the
same line style from the junction to each alternative, in written order. Subjects whose entries are
identical at the same strength share one junction. A junction's label is `any of`.

### 5.2 Strength is a word and a line

Every edge carries its strength as text, and as a line style besides:

| strength | label | DOT | Mermaid |
|---|---|---|---|
| `must` | `must` | `style=bold` | thick, `==` |
| `warn` | `warn` | `style=solid` | solid, `--` |
| `should` | `should` | `style=dashed` | dotted, `-.` |
| `may` | `may` | `style=dotted` | dotted, `-.` |
| `must-not` | `must not` | `style=bold, arrowhead=tee` | thick, `x` head |
| `warn-not` | `warn not` | `style=solid, arrowhead=tee` | solid, `x` head |
| `should-not` | `should not` | `style=dashed, arrowhead=tee` | dotted, `x` head |

Mermaid has three line styles, so `should` and `may` share one there and the word
tells them apart.

### 5.3 Self-edges and cycles are drawn as they are

A kind that cites itself is one edge from its node to its node, and the table
writes the target as `<KIND> (itself)`. A cycle needs nothing special: the view
computes no ranking and no topological order, so no configuration is refused or
reordered for having one.

## 6. Grounding is a badge

A row's effective grounding is printed on its own node as `grounded: ` followed by
its ladder, each rung `<strength> · <unit>` and rungs joined by `, `
(`grounded: must · file`). It is never an edge, because grounding names no target
kind. A row the project's grounding applies to carries the project's ladder, a
row with its own ladder carries that one, and a row that need not be grounded
carries no badge. A `may` rung checks nothing, so it is no rung of the badge: the
rungs are the ones `grund check` enforces. In the table the badge is the `grounded` column of the row's
first citations line.

## 7. Order

The same configuration prints the same bytes ([§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)). Nodes
follow row order, the complement after every written row unless a row names it.
Edges follow their subject's node order, then strength in the order of the table
in [§FS-schema.5.2](FS-schema.md#52-strength-is-a-word-and-a-line), then the order the entries were written. Junctions are numbered
in the order their first edge is printed. Nothing is sorted alphabetically.

## 8. The table

The default format. A header line `<project> · <config path> · v<version>`, the
config path spelled from the config root, or `defaults, no grund.toml` where no
config file was found; then the `kinds and places` section,
one line per node with its name, class and places, and its field lines under it;
then the `citations` section, headed `citations` with `grounded` over the badge
column, one line per direction of each scanned row in node order, the first line of
a row carrying its badge; then one line per unscanned row,
`<name>  not scanned: cites nothing and is never checked`; then the legend:

```
must = check error · warn = check warning · should = suggestion (check --suggestions)
may = allowed · must not / warn not / should not = forbidden at that strength
A|B = one obligation, met by citing either · — = no direction rule names it
```

then the line [§FS-schema.8.3](FS-schema.md#83-the-legend-says-an-obligation-skips-an-uncited-file) asks for where it asks for one, and last the line
[§FS-schema.8.4](FS-schema.md#84-the-legend-says-where-a-fields-heading-sits) asks for where some kind prints a field line.

Sections are separated by one blank line. Columns are padded with spaces, two
between columns: the name column is as wide as the longest row name, the strength
column as the longest strength word, and the last column starts at one offset in
both sections, two past the widest cell before it on a line that has a last cell.
A field line is indented two past the class column. A table in which no row carries a badge
heads its citations section `citations` alone and has no badge column.

### 8.1 The table, for one small project

```toml
grund_config_version = 2
project_name = "fixture"
[schema.sources]
languages = ["markdown"]
[schema.kinds.GOAL]
files = ["docs/goals.md"]
[schema.kinds.FS]
folders = ["docs/fs"]
[schema.kinds.FS.fields.status]
one_of = ["Draft", "Accepted"]
presence = "should"
position = "first"
[schema.kinds.AR]
folders = ["docs/ar"]
[schema.kinds.test]
citable = false
folders = ["tests"]
[schema.kinds.vendor]
citable = false
scan = false
folders = ["vendor"]
[rules.citations.FS]
should = ["GOAL|FS"]
may = ["AR"]
[rules.citations.AR]
must = ["FS"]
should = ["AR"]
[rules.citations.test]
must = ["FS"]
[rules.citations.grounding]
must = "file"
[rules.citations.test.grounding]
warn = "h2"
```

```text
fixture · grund.toml · v2

kinds and places
  GOAL    kind                docs/goals.md
  FS      kind                docs/fs/
            field status      `## status: <one of Draft, Accepted>` (should, first): the title is the value
  AR      kind                docs/ar/
  test    place               tests/
  vendor  place, not scanned  vendor/
  code    complement          every scanned file no other row claims

citations                     grounded
  GOAL    —                   must · file
  FS      should  GOAL|FS     must · file
          may     AR
  AR      must    FS          must · file
          should  AR (itself)
  test    must    FS          warn · h2
  code    —                   must · file
  vendor  not scanned: cites nothing and is never checked

must = check error · warn = check warning · should = suggestion (check --suggestions)
may = allowed · must not / warn not / should not = forbidden at that strength
A|B = one obligation, met by citing either · — = no direction rule names it
## = under a declaration written # <ID>: … · a field always sits one level below its declaration's heading
```

### 8.2 With no grund.toml, a note names what `grund init --scaffold` writes

Where no config file was found, the line directly after the header, before the
blank line, is one note:

```text
note: these are the built-in defaults a tree with no grund.toml is held to; grund init --scaffold writes a version-<n> grund.toml that also declares <KIND> in <places>
```

`<n>` is the config version of the template `grund init --scaffold` writes, and
each `<KIND> in <places>` is a kind that template declares and the defaults do not,
its places spelled as the `kinds and places` section spells them, in the
template's order and joined by `, `. Today that is `REQ in docs/requirements/`,
which has no built-in default ([§DF-fs-home-moves-from-0-19.2.3](../decisions/functional/DF-fs-home-moves-from-0-19.md#23-req-goes-into-the-template-now-with-no-built-in-default)). Both are read from
the template init writes, never written down a second time, so the note follows
the template when it changes; where the template declares no kind the defaults
lack, the note ends at `grund.toml` and drops the `that also declares` clause.

The note is not part of the view: no node, row or direction comes from the
template, so what the view shows is still the effective configuration alone
([§FS-schema.3](FS-schema.md#3-what-the-view-reads)). A tree whose config file was read prints no note. The Mermaid
and DOT sources carry the header comment and no note: they are source for a
renderer, and stay byte for byte what they were.

### 8.3 The legend says an obligation skips an uncited file

A place's or the complement's `must`, `warn` and `should` judge only its files that cite something ([§FS-config-v2.rules.citations.uncited](FS-config-v2.md#rulescitationsuncited-an-obligation-never-judges-a-file-that-cites-nothing), [§FS-config.3.9.1.4](FS-config.md#3914-a-file-that-cites-nothing-is-not-judged)), so a `must` line beside no badge would read as a promise every file cites. Where some scanned place or the complement carries an obligation and no `grounded` badge, the legend gains a fourth line, in version 2:

```text
must/warn/should skip an uncited file of a place or the complement · [rules.citations.<ROW>.grounding] makes it a finding
```

and in version 1 the same line ending `· require_grounding makes it a finding`. Where every such row is grounded, or none carries an obligation, the legend has no such line, as in [§FS-schema.8.1](FS-schema.md#81-the-table-for-one-small-project). The Mermaid and DOT comments carry the legend's first two lines only ([§FS-schema.9](FS-schema.md#9-the-mermaid-source)), so they do not change. The `place` row of the config of agent-grounds/grund#151 —

```toml
grund_config_version = 2
project_name = "demo"

[schema.kinds.FS]
files = ["requirements.md"]

[schema.kinds.e2e]
folders = ["tests/e2e"]
citable = false

[rules.citations.e2e]
must = ["FS"]
```

— prints:

```text
demo · grund.toml · v2

kinds and places
  FS    kind        requirements.md
  e2e   place       tests/e2e/
  code  complement  every scanned file no other row claims

citations
  FS    —
  e2e   must  FS
  code  —

must = check error · warn = check warning · should = suggestion (check --suggestions)
may = allowed · must not / warn not / should not = forbidden at that strength
A|B = one obligation, met by citing either · — = no direction rule names it
must/warn/should skip an uncited file of a place or the complement · [rules.citations.<ROW>.grounding] makes it a finding
```

### 8.4 The legend says where a field's heading sits

A shape line shows each heading under a declaration written `# <ID>: …`
([§FS-schema.4.1.1](FS-schema.md#411-a-heading-is-shown-under-a-declaration-written--id-)), so a table that printed `## verification: …` and nothing more
would give the wrong depth for a declaration written `## SWR-001: …`, as
`examples/traceability` declares one. Where some kind prints a field line, the
legend gains a last line:

```text
## = under a declaration written # <ID>: … · a field always sits one level below its declaration's heading
```

after the line of [§FS-schema.8.3](FS-schema.md#83-the-legend-says-an-obligation-skips-an-uncited-file) where both apply. Where no kind prints a field
line, the legend has no such line, as in [§FS-schema.8.3](FS-schema.md#83-the-legend-says-an-obligation-skips-an-uncited-file)'s example. The Mermaid
and DOT comments carry the legend's first two lines only, so they do not change.

## 9. The Mermaid source

`--format mermaid` prints a `flowchart LR`, ready to paste into a fenced
`mermaid` block. After the first line come the comments: the header, the legend's
first two lines, `any of = one obligation, met by citing any one of its arrows`,
and the workspace members line where [§FS-schema.2.1](FS-schema.md#21-a-workspace-root-shows-the-root-and-names-its-members) asks for it. Then, in order:
the two class definitions `classDef place stroke-dasharray:5 5` and
`classDef unscanned stroke-dasharray:2 2`; every ruled node, `n<i>["…"]`, its label
lines joined by `<br/>`; every junction, `j<i>(("any of"))`; every target outside
the project (`alias/KIND`, `*/KIND`), `x<i>["…"]` labelled as written; the
`no direction rule names these` subgraph holding the unruled nodes; one
`class <ids> place` and one `class <ids> unscanned` line where such nodes exist;
then the edges. A node's label lines are its name, its class word, its places, its
field lines as `field <name>: <shape line>`, its `closed` line as `closed: <text>`, and its
badge ([§FS-schema.4.1](FS-schema.md#41-fields-are-lines-inside-their-kinds-node)). A `front-matter` field's keys, and its own `closed`, follow its
field line as `key <name>: <text>` and `closed: <text>` ([§FS-fields.front-matter.record](FS-fields.md#front-matterrecord-the-record-and-what-renders-it)).
The scalar of [§FS-schema.8.1](FS-schema.md#81-the-table-for-one-small-project) is the label line
``field status: `## status: #lt;one of Draft, Accepted#gt;` (should, first): the title is the value``,
its placeholder's brackets escaped as [§FS-schema.11](FS-schema.md#11-escaping) says. Node identifiers are
generated, `n` with the row's position, `j` with the junction's, and `x` with the
order a target outside the project is first named, and never come from user text.

## 10. The DOT source

`--format dot` prints one `digraph schema` with `rankdir=LR` and
`node [shape=box]`, ready for `dot -Tsvg`. The comments come first inside it, as
`//` lines, the same as Mermaid's. A node's label lines are Mermaid's, a `front-matter`
field's key lines included, each escaped for DOT rather than for Mermaid, so the same
scalar is ``field status: `## status: <one of Draft, Accepted>` (should, first): the title is the value``
and a JSON value home's `{"<ID>": …}` writes each `"` as `\"`. Then every ruled node, `n<i> [label="…"]` with
its label lines joined by `\n` and its style where its class has one; every
junction, `j<i> [label="any of", shape=circle]`; every target outside the project
(`alias/KIND`, `*/KIND`), `x<i> [label="…"]`, numbered as in Mermaid; the unruled nodes inside
`subgraph cluster_unruled` labelled `no direction rule names these`; then the
edges, each with its `label` and `style`, `arrowhead=none` from a subject to a
junction. grund runs neither Mermaid nor Graphviz.

## 11. Escaping

Every title, shape line and path is escaped for the printer it reaches, so no configuration
can break the source or the table ([§REQ-never-crashes](../requirements/REQ-never-crashes.md#req-never-crashes-garbage-in-diagnostic-out)). DOT labels use DOT string
escapes (`\"`, `\\`, and a newline as `\n`). Mermaid labels are quoted, with `"`
written `#quot;`, `<` as `#lt;`, `>` as `#gt;` and `&` as `#amp;`. The table
writes a control character as its TOML basic-string escape: `\n`, `\t`, `\r`,
otherwise `\uXXXX`. A shape line's own placeholders and code spans
([§FS-schema.4.1.2](FS-schema.md#412-the-code-span-and-the-placeholders)) are escaped like any other text: a Mermaid label writes
`<title>` as `#lt;title#gt;`, and the table and the managed block write them as they are.

## 12. Exit codes and streams

| case | stdout | stderr | exit |
|---|---|---|---|
| the view is printed | the view | empty | `0` |
| the configuration cannot be read | empty | the located `error: <path>:<line>: …` that `grund config show` prints | `2` |
| a usage error | empty | one `error:` line, and for an unknown format a second line `known formats: table, mermaid, dot` | `2` |

A usage error is an unknown flag, an unknown `--format` value
(``error: unsupported schema format `json` ``), `--format` with no value
(`error: --format requires a value`), or a second path
(`error: schema takes at most one path argument`, [§FS-cli.3.2](FS-cli.md#32-at-most-one-path)). It is answered
before any configuration is read ([§FS-cli.requirements.before-load](FS-cli.md#requirementsbefore-load-a-usage-mistake-is-answered-before-the-tree-is-read--realized-directional-for-the-next-flag)). There is no
exit `1`: the command makes no query that can fail ([§FS-cli.5](FS-cli.md#5-exit-code-mapping-is-fixed)).

## 13. What is absent

- **No `--format json`.** A JSON shape would be a stable contract
  ([§REQ-backwards-compatibility.1](../requirements/REQ-backwards-compatibility.md#1-what-is-covered)), and the record it would freeze still moves.
  A caller that needs machine-readable facts reads `grund config show`, whose TOML
  loads back to the same values ([§DF-schema-view.2.2](../decisions/functional/DF-schema-view.md#22-no---format-json-yet)).
- **No ID graph.** The view draws the configuration, never declarations, their
  citations or their counts ([§FS-non-goals.6](FS-non-goals.md#6-decision-database-audit-log-history-tracking)); there is no instance overlay, no
  `--toc`, and no browser hosting.
- **No change to `grund config show`** ([§FS-fields.record](FS-fields.md#record-the-record)). The managed block prints
  the view's shape lines ([§FS-init.2.3.5.12.2](FS-init.md#235122-declaration-shapes)), so a change to their wording, such as
  showing a field's heading ([§FS-schema.4.1](FS-schema.md#41-fields-are-lines-inside-their-kinds-node)), is a block-version change
  ([§FS-init.2.3.7](FS-init.md#237-the-block-version)).

## 14. Where an agent learns it

The command is taught in the three places an agent learns grund from, and each is
held to what the command prints:

- **The runnable example** `examples/schema/`: a small tree whose `grund.toml` has
  an any-of entry, grounding, a field, a non-citable place and an unscanned row.
  Its README shows the command, its table output verbatim, and its Mermaid source in
  a fenced `mermaid` block; the table is the example's `expected.stdout`, so the
  README cannot drift from the command ([§FS-examples.5](FS-examples.md#5-e2e-reuse-without-duplication)). `examples/README.md` and the
  README's schema-and-links sentence link it.
- **The shipped skill** `skills/grund-init/`, byte-identical to its embedded
  copy ([§FS-init.5.3](FS-init.md#53-one-directory-two-surfaces)), tells an agent in its `schema` section
  ([§FS-init.5.5](FS-init.md#55-the-core-and-its-sections)) to run `grund schema` to see what a repository
  is held to before it writes declarations or citations, gives a short example of
  the table, and says that `grund config show` is for the effective TOML.
- **The user guide** `docs/user-facing/schema.md`: what each format shows, the
  legend, and a rendered Mermaid example.
