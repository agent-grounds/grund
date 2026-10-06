# DISC-core-concerns: Three concerns over two trees — how the configuration, the core spec and the engine are organized

## 1. Status

Open, and a design discussion rather than a change. It is proposed under `agent-grounds/grund#347`, the plan from here to `1.0`, and it is the document that plan rests on: three **concerns** — schema (what exists), rules (how nodes relate), presentation (the bytes `grund` writes) — over two trees, a v2 `grund.toml` organized by them, and the releases that would reach it.

Nothing decided here changes a key, a finding, a byte of output or a Rust symbol. The question it asks of `grund.toml`, of [§FS-config](../../functional-spec/FS-config.md#fs-config-grund-reads-a-toml-config-file-found-by-walking-up) and of `grund-core` is one question: are what exists, how things must relate, and how `grund` writes them separated? None of the three separates them, and [§DISC-core-concerns.2](2026-09-30-core-concerns.md#2-context--what-is-not-separated-today) is the evidence.

### 1.1 What accepting it accepts

Accepting this discussion accepts the model of [§DISC-core-concerns.3](2026-09-30-core-concerns.md#3-the-model), the format of [§DISC-core-concerns.7](2026-09-30-core-concerns.md#7-the-v2-format) and the release plan of [§DISC-core-concerns.9](2026-09-30-core-concerns.md#9-the-releases). Each step it names is then specified and shipped on its own, extends the specification points it would newly contradict, and owes [§REQ-backwards-compatibility](../../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly) on its own terms — the shape [§DISC-grund-core-public-surface](2026-09-22-grund-core-public-surface.md#disc-grund-core-public-surface-what-grund-cores-public-root-surface-is-and-what-it-should-be) takes for the embedding surface, and for the same reason: a discussion that could cut would be proposing under a licence it does not have.

Two of its chapters are not proposals and read differently. [§DISC-core-concerns.6](2026-09-30-core-concerns.md#6-what-the-agora-settled) records what has already been settled about the draft this replaces, and [§DISC-core-concerns.8](2026-09-30-core-concerns.md#8-what-v2-contradicts-in-what-is-specified-today) records which shipped specification points v2 would contradict — which is owed whether or not v2 ships, because a proposal that does not name what it crosses cannot be refused on the merits.

### 1.2 How a ruling made off the record is evidenced

This document is the revision of a draft that was argued over in a local cross-family agora — four rounds, three participants, one moderator — which produced 13 certainties, 24 agreed entries and 4 disputes, and whose disputes the repository owner then ruled. The agora ran on one machine, its artifacts are not published, and the decision is that they stay that way.

So the evidence here is what is public and dated rather than what is at hand:

- The **outcome of the agora** is evidenced by the *What the agora settled* and *What was ruled* chapters of `agent-grounds/grund#347` — the owner's own restatement, in their words, of the entries and the four rulings. [§DISC-core-concerns.6](2026-09-30-core-concerns.md#6-what-the-agora-settled) below is written against that restatement and keeps the verdict's ids, so an entry can be argued about by name.
- The **four points ruled in conversation on 2026-09-30**, and the one amendment to an agreed entry, are evidenced by the owner's comment of [2026-09-30T09:05:23Z](https://github.com/agent-grounds/grund/issues/347). A comment on the issue is dated, attributed and immutable, which is every property a ruling file has.

**Rejected: a sibling evidence file.** This repository already has the shape — `2026-09-22-grund-core-public-surface-inventory.md` is carried beside its proposal, declares no ID and states no status, and `docs/discussions/README.md` describes the arrangement. It is the better form on the merits, and it is rejected here for one reason that is not about form: it would publish the agora. What is lost is that a reader cannot re-read the argument behind an entry, only its outcome; what is kept is that every claim below has a public address.

### 1.3 Where each part of the plan stands

Reconciled on 2026-10-07 under `agent-grounds/grund#451`, against the releases that actually shipped ([§DISC-core-concerns.9.8](2026-09-30-core-concerns.md#98-what-shipped)). Every claim this document makes is in one of four standings, and a reader picking up one of the tickets of `agent-grounds/grund#347` should read it here before reading it below:

| Standing | What | Evidence |
|---|---|---|
| Ruled | `folders` and `files` are lists on every row, and both may appear on one row | the owner's comment of [2026-09-30T09:05:23Z](https://github.com/agent-grounds/grund/issues/347); [§DISC-core-concerns.6.4](2026-09-30-core-concerns.md#64-amended-after-the-verdict-v-a08-and-v-a14) |
| Ruled | v2 has no `include`: the scan set is the union of every row's places | the same comment; [§DISC-core-concerns.6.4](2026-09-30-core-concerns.md#64-amended-after-the-verdict-v-a08-and-v-a14) |
| Ruled | a kind's rules render at the deepest directory containing all of its places | the same comment; [§DISC-core-concerns.6.4](2026-09-30-core-concerns.md#64-amended-after-the-verdict-v-a08-and-v-a14) |
| Ruled | the four disputes V-D01 to V-D04 | [§DISC-core-concerns.6.3](2026-09-30-core-concerns.md#63-ruled) |
| Ruled | the declaration form is spelled `form`, never `body` | the vocabulary merged in `agent-grounds/grund#359`; [§DISC-core-concerns.3.7](2026-09-30-core-concerns.md#37-fields-and-form-the-type-of-a-kind) |
| Ruled | the envelope — version, identity, workspace — sits outside the three concerns | [§FS-config.concerns](../../functional-spec/FS-config.md#concerns-every-key-belongs-to-exactly-one-concern); [§DISC-core-concerns.6.2](2026-09-30-core-concerns.md#62-corrected) |
| Permanent | every v1 config keeps its reader and its meaning, whatever v2 becomes | [§FS-config.5.2](../../functional-spec/FS-config.md#52-every-older-version-keeps-its-meaning) |
| Permanent | the intermediate v1 spellings the draft discarded (`grounding`, `values = "<chapter>"`, `links`) are not added as bridge aliases: they never shipped, so retiring them owes nothing, and any spelling admitted to v1 would be permanent | [§FS-config.5.2](../../functional-spec/FS-config.md#52-every-older-version-keeps-its-meaning) |
| Shipped | Phase 0, and Terms enforcement (`agent-grounds/grund#290`), in `0.15.0`; the bare-`grund` removal (`agent-grounds/grund#301`) in `0.16.0` | [§DISC-core-concerns.9.8](2026-09-30-core-concerns.md#98-what-shipped) |
| Open | the nine choices of [§DISC-core-concerns.10.1](2026-09-30-core-concerns.md#101-the-five-decisions-the-2026-09-30-proposal-leaves-open) and [§DISC-core-concerns.10.2](2026-09-30-core-concerns.md#102-the-five-decisions-the-plan-makes-one-way) that are not settled, each with the ticket that rules on it | [§DISC-core-concerns.10](2026-09-30-core-concerns.md#10-what-this-discussion-still-has-to-settle) |

What is open stays open here until its owner ticket rules, and where that ruling and this document disagree the ruling wins and this document is corrected with it. Nothing in this reconciliation releases a behaviour ticket: each of `agent-grounds/grund#453` to `agent-grounds/grund#468` keeps its own discussion gate, and the schema view proposed as `grund schema` (`agent-grounds/grund#463`) is a separate command rather than a `config show` renderer, specified nowhere here.

## 2. Context — what is not separated today

### 2.1 The spec states one axis and not the other

[§FS-config.principle](../../functional-spec/FS-config.md#principle-a-setting-written-at-a-narrower-scope-wins) states the **scope** axis once and well: a setting is written at the project or the kind scope, the narrower wins, and [§FS-config.principle.inventory](../../functional-spec/FS-config.md#principleinventory-the-settings-admitted-at-both-the-project-and-the-kind-scope) lists, derived rather than asserted, the settings admitted at both. There was no second axis. Nothing said which settings describe the corpus, which relate its parts, and which only shape output, so every key section is organized by the TOML table that happens to hold it — and the word the first concern needs was spent as well, on the title of the key list itself.

### 2.2 One table holds five concerns

`[reference]` is documented as *citation form* ([§FS-config.3.1](../../functional-spec/FS-config.md#31-reference--citation-form)). It holds:

| Keys | What they are |
|---|---|
| `marker`, `strict`, `shorthand` | the lexical grammar of a citation |
| `trigger` | an authoring input |
| `conversation` | how agents render citations in conversation |
| `require_grounding`, `grounding_level` | an obligation on files to cite |
| `inline_style`, the three note budgets, `warn_on_suggested`, `inline_note_layout`, `inline_note_layout_check`, `lead_size_warning` | the shape of notes and of leads |

A `[[kinds]]` row is mixed the same way: `kind`, the home, `citable`, `format`, `values`, `value_chapter` and `rules` describe what exists; `require_grounding`, `grounding_level` and `resolve` constrain citations; `title` feeds presentation.

### 2.3 Constraints live in three places and speak five vocabularies

A constraint can be written as a `[citations]` entry ([§FS-config.3.9](../../functional-spec/FS-config.md#39-citations--citation-direction-rules)), as a key (`require_grounding`, `resolve`, the note budgets, `section_heading_levels`), or as a chapter rule in the docs ([§FS-rules](../../functional-spec/FS-rules.md#fs-rules-grounded-declarations-state-and-enforce-chapter-rules)). [§FS-terms.terms.6](../../functional-spec/FS-terms.md#terms6-rules-and-directions) already defines *rule* as covering all three; the layout does not follow the vocabulary. Their strengths are spelled five ways: RFC-2119 levels (`[citations]`, `resolve`, chapter rules), booleans (`require_grounding`, `warn_on_suggested`), `strict | warn | loose` (`section_heading_levels`), `off | warn | error` (`inline_note_layout_check`), and a fixed severity with no field at all (`lead_size_warning`). [§FS-rules.6](../../functional-spec/FS-rules.md#6-semantic-deduplication) exists to deduplicate two of the three places.

### 2.4 One name, two trees

Seven of this repository's sixteen `[[kinds]]` rows are `citable = false`, and with the implicit `code` that is eight rows declaring no ID namespace. [§FS-config.3.4.1](../../functional-spec/FS-config.md#341-citable--kinds-that-declare-no-ids) says what they are in prose: a non-citable kind is *a place and nothing more*. [§FS-init.2.3.4.4](../../functional-spec/FS-init.md#2344-project-map) renders them by place, never by name.

The distinction is load-bearing and unstated. A `[citations.<name>]` obligation is checked **per declaration** when the name is citable ([§FS-config.3.9.1.1](../../functional-spec/FS-config.md#3911-obligations-and-prohibitions)) and **per file, or per heading unit,** when it is not ([§FS-config.3.9.2.4](../../functional-spec/FS-config.md#3924-obligations-apply-per-source-file)). One table spelling reaches two different subjects, and [§FS-rules.6](../../functional-spec/FS-rules.md#6-semantic-deduplication)'s config-to-rule deduplication is as narrow as it is because a rule about a declaration and a rule about a file only look alike.

### 2.5 A kind's language is implied, and `format` names something else

What a declaration is spelled in follows from the file, never from the kind: Markdown in a Markdown file, Markdown inside a doc-comment in a source file, and — for an opted-in value kind — an object member in a `.json` file of its home ([§FS-values.2.2.1](../../functional-spec/FS-values.md#221-source-shape)). [§DISC-markup-format-declarations](2026-05-25-markup-format-declarations.md#disc-markup-format-declarations-declarations-in-asciidoc-restructuredtext-latex-and-similar-markup-document-formats) asks how AsciiDoc, reStructuredText and LaTeX would join, and finds the scanner's *markdown vs. source* split must become a three-way classification first. The row key a reader would reach for, `format`, is taken: it is the kind-scope ID template ([§FS-config.3.4.10.1](../../functional-spec/FS-config.md#34101-format)).

### 2.6 The shape a kind requires is written as a rule

[§FS-terms.terms](../../functional-spec/FS-terms.md#terms-terms) requires every functional-spec declaration to carry exactly one `## terms: Terms` chapter after its lead, and says in prose that no check holds it. The planned fix is a rule declaration in the chapter-presence family of [§FS-rules.3.1](../../functional-spec/FS-rules.md#31-chapter-presence). The value chapter is the same idea spelled a third way: `value_chapter = "values"` on the row ([§FS-config.3.4.13](../../functional-spec/FS-config.md#3413-value_chapter--the-chapter-whose-named-children-are-values)). So what a declaration of a kind must contain — the part of a kind a reader would call its type — is spread between a row key, a sentence family and prose.

### 2.7 The engine already reads by concern

Every non-test read of a `Config` field in `grund-core` shows the concerns separated in practice: the fields that describe the corpus are first read by the scanner and the resolver, the citation constraints by the checker and the rules engine, the presentation fields by the writers, templates and api. The templates reading constraint fields is the checked-guidance loop of [§GOAL-agent-grounding](../../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work): the entrypoint renders what the checker enforces.

Three reads cross a concern:

| Read | What it is |
|---|---|
| `scanner/units.rs` reads `grounding_level` | the scan records heading structure only where a grounding rule will ask for it — a rule deciding what the parse records, for speed ([§AR-scanner.2.7](../../architecture/AR-scanner.md#27-grounding-units-per-file)) |
| `resolver/link_targets.rs` reads `cross_ref_anchor_format` | turning a coordinate into a link is presentation; it sits in the resolver because `fmt` and the index-entry check both need it |
| `scanner/agent_entrypoints.rs` reads `conversation` | which entrypoints a canonical render can speak for — presentation read at scan time |

None is a bug. Each is a place where the dependency on a concern is implicit, because every component receives the whole `Config`: a flat record of about sixty fields that also carries the compiled `Grammar` and per-run state.

### 2.8 Defaults are frozen where `init` wrote them

`grund init` writes every key explicitly so the file teaches. That also pins each repository to the defaults of the day it ran: all four grounded repositories of this organization carry the inert `color = "auto"`, three carry verbatim copies of the default `number_pattern` and `slug_pattern`, and two keep the numbered default `{kind}-{number}-{slug}` while two override it to slug-only. `named_sections = false` is the teaching default ([§FS-config.3.2.7](../../functional-spec/FS-config.md#327-named_sections--the-gate-for-explicit-section-names)), though every declaration here uses named sections and `value_chapter` is a config error without them. The compatibility carve-outs — the older implicit `FS` home ([§FS-config.3.4.4.4](../../functional-spec/FS-config.md#3444-a-config-that-omits-kinds-keeps-the-older-fs-home)) and the name-keyed `E2E` index default ([§FS-config.3.4.2.4](../../functional-spec/FS-config.md#3424-the-default-is-per-kind-name)) — are defaults of an earlier epoch kept alive without a version.

## 3. The model

### 3.1 The cut: one node, at least two, bytes

Separate by what a setting is about, not by the table it sits in:

- **Schema** — the nodes and their shape. Which declarations, sections and citations exist, what they resolve to, and what a well-formed one looks like. Every finding the schema produces is about **one node on its own**: a malformed heading, a missing required chapter, an oversized lead, a note that breaks its budget, a scalar field outside its allowed values.
- **Rules** — relations between nodes. Every finding a rule produces is about an **edge, or a missing one**: a forbidden or missing citation, an ungrounded file, a count out of range, a citation that does not resolve, a declaration absent from its kind's index.
- **Presentation** — the bytes `grund` writes and shows. It produces no finding of its own, and reaches one only where a written byte has drifted from what the config now renders — which is a rule over the generated region rather than a finding of the setting.

The cut is mechanical, which is what makes it checkable: a schema finding is computed from one record of the catalog, a rule finding needs the resolved graph, and a presentation setting reaches a finding only through the region a renderer produced and a rule then compared. So the key → concern inventory is derived from the read sites rather than written by hand, the way [§DF-config-scope-override.2.2](../../decisions/functional/DF-config-scope-override.md#22-the-inventory-is-derived-in-two-stages) derives the scope inventory. That derivation, and the classification of every key of the format in force, is [§FS-config.concerns](../../functional-spec/FS-config.md#concerns-every-key-belongs-to-exactly-one-concern)'s and the record it names; this chapter is only the cut it applies.

It settles the borderline cases without taste:

- `[scan]` is schema: `include`, `extensions` and `comment_prefixes` decide what exists.
- `marker`, `strict` and `shorthand` are schema; `trigger` is presentation.
- The note budgets are schema: a note is the written form of one citation site, and its shape is a property of that one site. That they also render into the entrypoint is presentation *of* schema, not a second setting.
- The lead budget and `section_heading_levels` are schema: both judge one declaration.
- Chapter presence ([§FS-rules.3.1](../../functional-spec/FS-rules.md#31-chapter-presence)) is schema — a fact about one declaration. Outbound and inbound citation counts are rules.
- A drift check — the managed block, an index entry — is a rule whose subject is a generated region: `compare(present(…), disk)`.

The rules concern that remains is exactly the citation rules: directions, grounding, counts, resolution — and the drift comparisons over what `grund` itself wrote.

### 3.2 Four corollaries

1. **Guidance is presentation, never a setting of its own.** Everything the entrypoint tells an agent to do is a schema constraint or a rule the checker holds, rendered.
2. **Whatever `grund` writes, it must read back.** The persisted form is schema; the decision to write it is presentation. So `marker` is schema and `trigger` is not.
3. **Scope is an axis inside each concern.** Project and kind scope stay as [§FS-config.principle](../../functional-spec/FS-config.md#principle-a-setting-written-at-a-narrower-scope-wins) states them, applied within schema, within rules and within presentation. A cascade never crosses a concern.
4. **One strength vocabulary for every constraint.** A schema constraint and a rule take their strength from the same closed set ([§DISC-core-concerns.4.3](2026-09-30-core-concerns.md#43-one-strength-vocabulary)), so a new required chapter can ramp through `warn` exactly as a new citation rule can.

### 3.3 Two trees and two joins

The catalog is two trees, joined twice, with citations as the edges between them:

```text
physical                                   logical
workspace                                  workspace
 └ project  (config root, alias)  ═══════   └ project  (namespace)
    └ place (folders | files | complement)     └ kind         (ID grammar, form, fields)
       └ file   (a language)                      └ declaration
          └ unit (heading depth)                     └ section   (a field, or numbered)

join  home       kind → place         a kind may claim several; the complement place homes none
join  location   declaration → file   a source declaration sits in code, its stub in the home
edge  citation   site (path:line:col) → coordinate (alias/ID.section)
```

`model::Citation` already has this shape — a located site pointing at an ID and a section — and `grammar::Grammar` is already the compiled schema. What was missing is the word for the physical tree's middle level, and [§FS-terms.terms.4](../../functional-spec/FS-terms.md#terms4-scanning-and-project-structure) now carries it: **place**.

A place is a region of the tree with a name. A citable kind with a home has a place of the same name; a non-citable row *is* a place and nothing more; the homeless kind is the **complement place**, the one place whose extent is *every scanned file no other place claims*. This is [§DF-non-citable-kinds.2.1](../../decisions/functional/DF-non-citable-kinds.md#21-citable--false-on-the-existing-kinds-table)'s two-by-two read as types: *has a home* is `Option<Place>`, *declares IDs* is `Option<Kind>`.

### 3.4 What attaches where, and the unit rule

| Concern | Subject | Object |
|---|---|---|
| Schema | declares both trees, both joins, how every file is read, and the shape of every node | — |
| Rules | a node of **either** tree: a place, a file, a unit, a kind, a declaration, a section — or a region `grund` generated | always **logical**: a kind, a coordinate, or the expected bytes |
| Presentation | either tree | — |

The asymmetry in the rules row is already the spec's: the citing side of `[citations]` may be any configured name, the cited side must be citable ([§FS-config.3.9](../../functional-spec/FS-config.md#39-citations--citation-direction-rules)). Stated as the model, the per-file unit of [§FS-config.3.9.2.4](../../functional-spec/FS-config.md#3924-obligations-apply-per-source-file) is not an exception to the per-declaration unit of [§FS-config.3.9.1.1](../../functional-spec/FS-config.md#3911-obligations-and-prohibitions): **a rule's unit is its subject's node** — a declaration when the subject is a kind, a file or a unit when it is a place. That sentence is now the specification's, stated once in [§FS-config.concerns](../../functional-spec/FS-config.md#concerns-every-key-belongs-to-exactly-one-concern) with those two points as its instances, which is the one piece of this discussion that has already shipped.

The third object — a region `grund` generated — is the agora's correction and not the draft's: a drift finding is an edge between what a renderer produced and what is on disk, so the checker compares an expectation it is handed and never reads a presentation setting itself.

### 3.5 Relation to the non-citable-kinds decision

[§DF-non-citable-kinds.2.2](../../decisions/functional/DF-non-citable-kinds.md#22-rejected-a-second-areas-table) rejected a second `[[areas]]` table because two tables differing by one boolean invite *why two tables*, and because a feature that must remember to consult both vectors fails silently where it forgets one. Both reasons hold, and this proposal keeps **one table in the file**. What it adds is below the file: one `Vec` of rows, each carrying an optional place and an optional kind, and two iterators over it. A feature that asks *every place* or *every kind* asks one iterator over one vector, so there is no union to forget.

[§DF-non-citable-kinds.2.5](../../decisions/functional/DF-non-citable-kinds.md#25-obligations-get-a-per-file-unit-and-grounding-follows-the-home) chose to let *cite something* (`require_grounding`) and *cite an `FS`* (`must`) compose as two keys, because a unit built from citations cannot see a file with none. That holds in v2 too, and [§DISC-core-concerns.7](2026-09-30-core-concerns.md#7-the-v2-format) keeps `grounding` a key of its own — which is the agora's answer to the draft's own first open question, closed rather than carried.

### 3.6 Language: one model under every spelling

A file's **language** decides how anything in it is spelled, and every language falls in one of three classes:

| Class | A declaration is | Today | Later |
|---|---|---|---|
| document | a heading, `# ID: title`, fields as `## name: Title` | Markdown | AsciiDoc, Org, Typst |
| data | an object member keyed by the ID, fields as keys | JSON, for value kinds | JSON for any kind, YAML |
| source | Markdown inside a doc-comment, without the `#` | every source language | — |

The languages are one built-in table from extension to class and comment syntax, which replaces `extensions`, the global `comment_prefixes` list and `docstring_python`: a comment prefix belongs to its language, so `--` stops opening a comment in Rust. A config names the languages it reads, maps extra extensions onto known ones, and — for a language the built-in table lacks — defines one in a table of its own rather than losing the capability the removal would have taken with it.

The language comes from the file and never from the row: a `.json` home is read as data because it is `.json`. To free the word, the ID template is renamed `id_format` at both scopes, one key meaning one thing at every scope as [§FS-config.requirements.3](../../functional-spec/FS-config.md#requirements3-one-key-means-one-thing-at-every-scope--directional) asks, never `format`.

Every language is a surface over one model — a declaration is an ID, a title, named fields, numbered sections and value components — so a feature is specified once against the model and each language is a reader into it. The JSON value shape of [§FS-values.2.2.1](../../functional-spec/FS-values.md#221-source-shape) is the data spelling of a declaration whose form is a value.

### 3.7 Fields and form: the type of a kind

What a declaration of a kind must contain belongs to the kind. A row may declare **fields** — the named chapters its declarations carry — and a **form**:

- `form` says what the declaration itself is: `prose` (the default), `value` (its numbered children are the components; v1's `values = true`), or `rule` (its title is an executable sentence and its body the rationale; v1's `rules = true`). One key with three values replaces two booleans that could not be combined.
- A **field** is a named chapter, keyed by its section name. It has an optional `title` the heading must read exactly, a `presence` strength (default `may`), an optional `position` (`first` or `last`) or `after = "<field>"`, and a `content`: prose by default, `values` to make its named children value roots ([§FS-values.2.5](../../functional-spec/FS-values.md#25-chapter-declared-value-roots)), or `one_of = [...]` for a scalar whose heading title is the value, `## status: Accepted`.
- `closed` takes a strength and admits only the declared fields and numbered sections.

A missing required field, a field out of position, a scalar outside its values and an undeclared chapter in a closed kind are schema findings, each about one declaration. The chapter-presence family of [§FS-rules.3.1](../../functional-spec/FS-rules.md#31-chapter-presence) stays for one exact declaration; for a whole kind, the field is the spelling.

**`form`, not `body`.** The draft spelled this key `body`. [§FS-terms.terms.1](../../functional-spec/FS-terms.md#terms1-declarations-and-coordinates) defines *body* as the declaration heading through the heading that closes it, and that definition reads that way throughout the functional spec and the architecture. One word, one meaning: the term keeps its sense and the key takes another. `content` is the field key, so `form` is the free word and the right one.

## 4. The core spec

### 4.1 Vocabulary

[§FS-terms](../../functional-spec/FS-terms.md#fs-terms-the-shared-vocabulary-of-the-functional-spec-and-the-architecture) gains **place** and **complement place** beside *kind* and *home*, together with **language**, **field** and **unit**, and a group of its own for the three concern words and the **envelope**, each concern defined by the test of [§DISC-core-concerns.3.1](2026-09-30-core-concerns.md#31-the-cut-one-node-at-least-two-bytes) rather than by a list of keys, so the definition survives the format changing. *Presentation* rather than *rendering*, because *rendering layer* is the integrations' ([§FS-integrations](../../functional-spec/FS-integrations.md#fs-integrations-grund-prints-and-installs-its-rendering-layer-integrations)).

Three of the words were already taken, and settling them is the substance of that step rather than a detail of it: *body* keeps its one meaning and the v2 key becomes `form` ([§DISC-core-concerns.3.7](2026-09-30-core-concerns.md#37-fields-and-form-the-type-of-a-kind)); *field* is defined for the declaration sense and names the JSON output sense as the one it does not displace; *place* moves up out of `FS-config`'s own Terms chapter, which keeps *homeless kind* as v1's spelling of the complement place. This half has shipped, in [§FS-terms.terms.9](../../functional-spec/FS-terms.md#terms9-the-configurations-concerns) and the rows beside it.

### 4.2 `FS-config`

- A `concerns` chapter beside `principle`, stating the cut and the unit rule, with a derived **key → concern inventory** recorded with the decision that argues it. Shipped.
- [§FS-config.3](../../functional-spec/FS-config.md#3-keys) retitled *Schema* → *Keys*, so the word names the concern. Shipped.
- Every key section states its concern in its first sentence. The sections are not reordered while v1 is the file's only shape; [§DISC-core-concerns.9.3](2026-09-30-core-concerns.md#93-phase-2-the-v2-format) reorders them with the v2 layout.
- The language table, fields and `form`, as [§DISC-core-concerns.3.6](2026-09-30-core-concerns.md#36-language-one-model-under-every-spelling) and [§DISC-core-concerns.3.7](2026-09-30-core-concerns.md#37-fields-and-form-the-type-of-a-kind) describe them, when Phase 2 is taken.

### 4.3 One strength vocabulary

Every constraint, in the schema and in the rules alike, takes its strength from one closed set: **must** (an error), **warn** (a warning), **should** (a suggestion) and **may** (unchecked). Prohibitions keep `must-not`, `warn-not` and `should-not`. The mapping to channels stays fixed, as [§FS-config.3.9.1.3](../../functional-spec/FS-config.md#3913-the-levelsurface-mapping-is-fixed) fixes it today.

`warn` is a level of its own and not a synonym for `should`: the two reach different channels, and v1 already uses both — the `section_heading_levels` and `inline_note_layout_check` ramps are warnings, and `[citations] should` is a suggestion. Where a constraint has a measure, the **measure names the table and the strength is the key**: `[schema.notes.lines] must = 3`. The draft had it the other way round, and the other way round collides — `[rules.citations.code.must]` would redeclare the `must = [...]` key that table already has.

## 5. The architecture

### 5.1 The core records

```rust
/// One project's configuration, whichever version spelled it. What `config show` prints.
pub struct Project {
    pub name: Option<String>,        // the envelope: identity …
    pub version: u32,                // … and the version that spelled the file
    pub workspace: Members,
    pub schema: Schema,
    pub rules: Rules,
    pub presentation: Presentation,
}

pub struct Schema {
    pub citation: CitationSyntax,        // marker, shorthand
    pub ids: IdGrammar,                  // id_format, separator, patterns, heading depth
    pub sources: Sources,                // exclude, languages, definitions, ignore files
    pub notes: Constraints<NoteMeasure>, // lines, columns, text, layout — per measure
    pub leads: Constraints<LeadMeasure>, // lines, words, bytes — per measure
    pub rows: Vec<Row>,                  // one per kind row, in header order
}

/// §DF-non-citable-kinds.2.1's two-by-two, as types.
pub struct Row {
    pub name: String,
    pub place: Option<Place>,
    pub kind: Option<Kind>,
}
pub struct Place { pub extent: Extent /* Folders | Files | Complement */, pub scanned: bool }
pub struct Kind {
    pub id_format: Option<String>,
    pub form: Form,                 // Prose | Value | Rule
    pub fields: Vec<Field>,
    pub closed: Strength,
    pub index: KindIndex,           // a citable kind with folders keeps one
    pub origin: Origin,             // Local | External { fetch }
}
pub struct Field {
    pub name: String,
    pub title: Option<String>,
    pub presence: Strength,
    pub placement: Option<Placement>, // First | Last | After(String)
    pub content: Content,             // Prose | Values | OneOf(Vec<String>)
}

impl Schema {
    pub fn places(&self) -> impl Iterator<Item = (&str, &Place)> { /* rows with a place */ }
    pub fn kinds(&self)  -> impl Iterator<Item = (&str, &Kind)>  { /* rows with a kind  */ }
}

/// Relations only: every rule reads more than one node.
pub struct Rules {
    pub citations: CitationRules,  // keyed by subject: Subject::Place(name) | Subject::Kind(name)
    pub grounding: Grounding,      // per scope: a strength ladder over file and heading units
    pub resolution: Resolution,    // per cited kind: must | warn
}

pub struct Presentation {
    pub fmt: FmtPresentation,          // exclude, anchors, links
    pub kinds: BTreeMap<String, KindPresentation>,  // title
    pub description: Option<String>,
    pub conversation: Option<Conversation>,
    pub trigger: String,
}

/// Per run, never read from a file.
pub struct Run { pub root: PathBuf, pub cli_base: PathBuf, pub config_file: Option<PathBuf>, pub scope: Scope, pub warnings: Vec<RunWarning> }

/// Derived once from a Project by the config component.
pub struct Compiled { pub grammar: Grammar, pub demand: ScanDemand }
```

Three things in that sketch are the agora's rather than the draft's, and each closes something the draft left open. The **envelope** — identity, version and the workspace members — sits on `Project` outside the three concerns, because it is read before any of them, to decide which file governs and which projects there are, and it constrains no node. `index` lives on the **kind**, not on every `Place`: an index is the document that lists a kind's declarations, so only a citable kind with folders can have one, and a record that put it on every place would represent a state v1 already refuses. Where a key is *spelled* and which concern it *belongs to* are separate questions: the index document is named on the row, and the finding the key reaches — a declaration absent from its kind's index — relates two nodes, so the inventory of [§FS-config.concerns](../../functional-spec/FS-config.md#concerns-every-key-belongs-to-exactly-one-concern) classifies it under rules. And `ScanDemand` is the first leak of [§DISC-core-concerns.2.7](2026-09-30-core-concerns.md#27-the-engine-already-reads-by-concern) closed without moving work: config, which sits below the scanner ([§AR-system.4](../../architecture/README.md#4-dependency-direction)), lowers the grounding rules into *record heading structure to depth d in place p*, so the scanner keeps its speed and reads no rule.

`Findings` becomes `Catalog`, the name [§FS-terms.terms.1](../../functional-spec/FS-terms.md#terms1-declarations-and-coordinates) already gives the scan's shared set; a *finding* is a report item, and the central type should not wear another word's name.

### 5.2 Stages take one concern

```rust
fn compile(project: &Project) -> Result<Compiled, ConfigError>;                    // config
fn scan(schema: &Schema, compiled: &Compiled, run: &Run) -> Catalog;               // scanner, resolver
fn conform(schema: &Schema, catalog: &Catalog) -> Report;                          // checker: one node at a time
fn judge(rules: &Rules, catalog: &Catalog, expected: &Expected) -> Report;         // rules, checker: relations
fn present(p: &Presentation, schema: &Schema, rules: &Rules, catalog: &Catalog) -> Output; // templates, writers
```

A stage that does not receive a concern cannot read it, so the cut becomes a property of signatures: the compiler holds it, as it holds component privacy under [§AR-core-module-layout.1.1](../../architecture/AR-core-module-layout.md#11-modrs-is-the-components-whole-boundary). `conform` is the checker's node-local half — the declaration checks of [§FS-declarations](../../functional-spec/FS-declarations.md#fs-declarations-a-declaration-is-addressable-once-from-one-allowed-place-and-holds-nothing-that-is-neither-a-coordinate-nor-a-finding) and the field, note and lead checks — and `judge` its relational half.

`expected` is what keeps `judge` free of presentation. The renderer computes the managed regions and the canonical index-entry link targets and hands them over; `judge` compares them with what is on disk and never interprets a presentation key. So `markdown_link_target` keeps its place in the resolver and takes `&Presentation`, the renderer calls it, and `CanonicalSurfaceReach` takes `&Presentation` too. `fmt --check` is a run mode rather than a finding of its own.

### 5.3 Versions and languages are readers

[§FS-config.5.2](../../functional-spec/FS-config.md#52-every-older-version-keeps-its-meaning) promises that a binary reads every version it ever supported under that version's meaning. With one `Project`, that promise costs one **reader** per version: `config/v1.rs` parses today's file, with today's defaults and carve-outs, and lowers it into a `Project`; a v2 reader does the same for the format of [§DISC-core-concerns.7](2026-09-30-core-concerns.md#7-the-v2-format). No component above config learns that a second version exists. Validation about meaning rather than spelling — prefix-freedom, home uniqueness, value prerequisites — runs on the `Project`, once, for both readers.

The same shape holds one level down: each language of [§DISC-core-concerns.3.6](2026-09-30-core-concerns.md#36-language-one-model-under-every-spelling) is a scanner reader that lowers a file into the one declaration model, so a check written against the model holds in Markdown, JSON and every later language without being written again.

### 5.4 The public surface

`Config`, `KindConfig`, `CitationRules` and `Findings` are public root names, each kept by [§DISC-grund-core-public-surface.6.1](2026-09-22-grund-core-public-surface.md#61-the-target-disposition-of-every-row)'s inventory, and `Findings` is named by [§FS-distribution.3.1](../../functional-spec/FS-distribution.md#31-rust-grund-core-crate). The new records ship beside them, with the old names as deprecated aliases named in the release notes and removal no earlier than the following minor ([§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)). Which release those notes may name is [§DISC-core-concerns.9.7](2026-09-30-core-concerns.md#97-what-a-deprecation-note-may-name), and it is not the one the draft assumed. That inventory's own check makes a missed row loud.

## 6. What the agora settled

Each entry below is a correction to the draft this document replaces, or a confirmation of it. The ids are the verdict's, and the restatement they are read from is the *What the agora settled* chapter of `agent-grounds/grund#347` ([§DISC-core-concerns.1.2](2026-09-30-core-concerns.md#12-how-a-ruling-made-off-the-record-is-evidenced)).

### 6.1 Confirmed

Three concerns, no `style` concern and none merged (V-A01). The strength vocabulary `must | warn | should | may`, with `warn` distinct from `should` (V-C02). `section_separator` stays (V-A12). Scalar fields as `## status: Accepted`, and no bold-label syntax in v2 (V-A07). `citable` and `scan` are not derivable, and one `form` key replaces two booleans (V-A08, as amended in [§DISC-core-concerns.6.4](2026-09-30-core-concerns.md#64-amended-after-the-verdict-v-a08-and-v-a14)). Kind-scope budgets deferred (V-A16). `[output]` to run flags (V-A22). `grounding` stays a key, and grounding counts an inline declaration (V-C11, V-A19).

### 6.2 Corrected

- **A presentation setting does reach a finding today.** Managed-block drift renders its expected text from `conversation` (V-C01). Drift over any generated region is a rule, and the checker is `judge(rules, catalog, expected)` where `expected` carries the rendered regions and the canonical index-entry link targets; the checker holds no presentation settings (V-A20, V-C12).
- **The envelope.** `grund_config_version`, `project_name` and `[workspace]` are outside the concerns; `project_description` becomes `[presentation] description`; the internal `Project` carries identity and version, which the draft's record omitted (V-A03, V-A04).
- **Keyed rows and fields, not arrays.** An array-row field silently re-attaches when a block moves, and today's reader silently merges a repeated `[citations.FS]` (V-C08, V-A05). A field's `name` becomes its table key; every row and field needs its own explicit header, a repeated one is a load error, and order is header position. `after = "<field>"` is added, exclusive with `position` (V-A06).
- **Measure-as-table, not strength-as-table** (V-C09, V-A16), for the collision [§DISC-core-concerns.4.3](2026-09-30-core-concerns.md#43-one-strength-vocabulary) names.
- **The row keys.** `syntax` is dropped, and `Kind.syntax` with it — a declaration is spelled in the language of the file it sits in (V-A09). `title` moves to `[presentation.kinds.<name>]` (V-A23). `index` stays spelled on the row and lives on the citable kind's home in the records (V-A10). `closed` takes a strength (V-A11).
- **The v1 mappings the draft got wrong.** `resolve = "should"` was a standing warning in v1, so resolution is `must | warn` (V-C03, V-A17). v1's silent soft note cap maps to `may`, and `warn_on_suggested = true` to `warn` (V-C04). `loose` returns as `heading_depth = "may"`, because v1's modes change what is reported and never what is recognized (V-C05, V-A13). `links` splits into `anchors` and `links = index | all`, because one value cannot preserve both halves of v1's independent choice (V-C06, V-A21). The draft's reason for keeping `trigger` in presentation — *never persisted* — was false; the placement is right and the reason is not (V-C07).
- **Citation rules.** `[rules.citations.<name>]` gains `warn` and `warn-not`; `default` admits only `may | should-not | warn-not | must-not`, and a v1 `default = "must"` migrates to `may` (V-A18). Grounding takes a strength ladder, and a narrower scope replaces the wider table whole (V-A19).
- **Languages.** A language-definition table restores what the `comment_prefixes` removal would have lost (V-A15); `languages` defaults to a set fixed at the v2 epoch rather than to every language the binary knows (V-A14).
- **What a migration proves.** The engine must represent every v1 meaning losslessly, and a clean before/after report on one tree is evidence, not proof, that a migration preserved meaning (V-A24). `strict = false` is the standing example: a tree whose citations are all marked reports identically under both, and a bare citation added tomorrow would not.

### 6.3 Ruled

Four entries were disputed and the repository owner ruled them on 2026-09-30.

| | Ruling |
|---|---|
| V-D01 | `fetch` lives on the kind row. There is no fourth *acquisition* concern: the concerns cover the read-and-check pipeline, and a command that runs outside `check` does not make one. |
| V-D02 | The report path base is a run flag, `--path-base=project\|invocation`, default `project`. No `[presentation.report]` table in `2.0`; a key can be added in a later v2 minor, while removing a shipped one would need a new reader version. `migrate` reports a v1 `relative_paths = false` as moved to the flag. |
| V-D03 | An omitted scan set is the config root, bounded by `respect_gitignore`, `exclude` and the admitted languages. Over-inclusion fails loudly and one `exclude` line fixes it; a bounded list fails silently on a source root nobody named, which is the blind spot [§REQ-no-missed-citation](../../requirements/REQ-no-missed-citation.md#req-no-missed-citation-every-citation-the-run-reads-is-checked) exists to close. |
| V-D04 | The language table is `[schema.sources.definitions.<name>]`. |

### 6.4 Amended after the verdict: V-A08 and V-A14

V-A08 reads *the kind row keeps `file`, `folder` (two keys, mutually exclusive)*. That clause is **amended**, not confirmed. On [2026-09-30T09:05:23Z](https://github.com/agent-grounds/grund/issues/347) the repository owner ruled that any kind, citable or not, takes several places, and that the v2 keys are therefore `folders` and `files` — lists, either or both admitted on one row. One home is a one-element list.

The rest of V-A08 stands: `citable` and `scan` are not derivable, `id_format` is the row's ID template, and one `form` key replaces two booleans. **v1 keeps `file` and `folder` as the singular, mutually exclusive keys under their own meaning**, which [§FS-config.5.2](../../functional-spec/FS-config.md#52-every-older-version-keeps-its-meaning) makes permanent — so this is a v2 spelling replacing a v1 spelling, not a v1 key changing sense.

**V-A14's `include` clause is amended too.** V-A14 reads *`[schema.sources]` keeps `include`, `exclude`, `respect_gitignore`, `languages` and the `extensions` map*, and argues in plain words that `include` cannot be derived. The same comment rules that **`include` has no v2 spelling**: the scan set is the union of every row's places, which is what [§DISC-core-concerns.7.2](2026-09-30-core-concerns.md#72-places-on-the-rows-and-what-the-lists-make-necessary), [§DISC-core-concerns.7.4](2026-09-30-core-concerns.md#74-what-the-format-leaves-out) and [§DISC-core-concerns.8.4](2026-09-30-core-concerns.md#84-dropping-include-widens-no-scan) act on. That clause is **amended**, not confirmed; the rest of V-A14 stands, including the `languages` default fixed at the v2 epoch that [§DISC-core-concerns.6.2](2026-09-30-core-concerns.md#62-corrected) records. V-D03 of [§DISC-core-concerns.6.3](2026-09-30-core-concerns.md#63-ruled) is read under the amendment: it is an omitted scan set that defaults to the config root, because there is no longer a key to omit.

Three further points were ruled in the same comment and are recorded here beside the four of [§DISC-core-concerns.6.3](2026-09-30-core-concerns.md#63-ruled), because they carry the same authority and the same date:

1. **Lists on every row.** A kind may name several places: documentation in two trees, tests in two folders.
2. **The root `AGENTS.md` is not scanned by construction.** It is listed like any other file, because a repository may give its entrypoints a row of their own with citations of their own.
3. **A kind's rules render in the `AGENTS.md` of the deepest directory that contains all of its places** — the multi-folder case of [§DISC-core-concerns.7.3](2026-09-30-core-concerns.md#73-a-kinds-rules-render-where-its-places-are).

## 7. The v2 format

### 7.1 The canonical example

Every case the format admits, in one file. The comments are part of the example: they say which case each row is.

The example spells what is ruled ([§DISC-core-concerns.1.3](2026-09-30-core-concerns.md#13-where-each-part-of-the-plan-stands)) and what is still proposed alike. Two lines of it are proposals awaiting their owner tickets: how the `code` row is marked, and the default of `[presentation] rules`, whose comments name the decision and its owner.

```toml
grund_config_version = 2
project_name = "example"                      # envelope: identity and version live outside the concerns

[workspace]                                   # envelope: a member is a boundary, not a scope
members = ["packages/*"]
optional_members = ["vendored/upstream"]
include_root = true

# ── schema: what exists, and what a well-formed one looks like ─────────────
[schema]
marker = "§"
shorthand = "canonical"
id_format = "{kind}-{slug}"
slug_pattern = "[a-z][a-z0-9-]*"
number_pattern = "\\d+"
section_separator = "."
heading_depth = "must"                        # must | warn | should | may   (may = v1's loose)

[schema.sources]                              # how a file is read; where to read is the rows'
exclude = ["tests/e2e/cases/*/repo"]          # gitignore-style globs
respect_gitignore = true
languages = ["markdown", "json", "rust", "python", "shell"]   # default fixed at the v2 epoch

[schema.sources.extensions]
mdx = "markdown"

[schema.sources.definitions.toy]              # a language the built-in table lacks
class = "source"
comment_prefixes = ["%%"]

[schema.notes.lines]                          # measure-as-table: the measure names the table,
must = 3                                      # the strength is the key
should = 1
[schema.notes.columns]
must = 100
[schema.notes.text]                           # must = false is v1's citation-only
[schema.notes.layout]
warn = "citation-first-colon"
[schema.leads.words]
warn = 600

[schema.kinds.FS]                             # keyed rows: a field always names its owner
folders = ["docs/spec/core", "docs/spec/plugins"]
index = "README.md"                           # a rules key, left on the schema row (V-A10)
[schema.kinds.FS.fields.terms]
title = "Terms"
presence = "must"
position = "first"

[schema.kinds.AR]
folders = ["docs/architecture"]
[schema.kinds.AR.fields.placement]
presence = "must"
position = "first"
[schema.kinds.AR.fields.terms]
presence = "must"
after = "placement"
[schema.kinds.AR.fields.values]
content = "values"

[schema.kinds.GOAL]                           # a single-file kind: a one-element list
files = ["docs/goals.md"]

[schema.kinds.DF]
folders = ["docs/decisions/functional"]
closed = "warn"
[schema.kinds.DF.fields.status]
one_of = ["Proposed", "Accepted", "Superseded"]
presence = "warn"
position = "first"

[schema.kinds.RULE]
folders = ["docs/rules"]
form = "rule"                                 # prose | value | rule

[schema.kinds.CONST]
files = ["config/constants.json"]             # the language comes from the file, never from the row
form = "value"

[schema.kinds.TICKET]
files = ["docs/tickets.md"]
id_format = "{kind}-{number}"
fetch = "scripts/fetch-ticket"                # ruled: on the row (V-D01)

[schema.kinds.API]                            # citable, no place: declared in doc-comments only

[schema.kinds.test]                           # a place: several folders, no ID namespace
citable = false
folders = ["tests/e2e", "tests/integration"]
[schema.kinds.templates]
citable = false
folders = ["templates"]
scan = false
[schema.kinds.page]                           # prose no kind declares in, with rules of its own
citable = false
folders = ["docs"]
files = ["README.md", "AGENTS.md"]
[schema.kinds.code]                           # the complement place; with no list, the config root
                                              # — marked by its name pending #457 (§10.1 decision 1)
citable = false
folders = ["crates"]

# ── rules: relations between nodes ─────────────────────────────────────────
[rules.citations]
default = "may"                               # may | should-not | warn-not | must-not
[rules.citations.grounding]
may = "file"                                  # project default; an explicit opt-out
[rules.citations.FS]
should = ["GOAL|FS"]
must-not = ["AR"]
[rules.citations.test]
must = ["FS"]
warn-not = ["AR"]
[rules.citations.test.grounding]
must = "file"
[rules.citations.code]
should = ["FS|AR|API"]
[rules.citations.code.grounding]
must = "file"
warn = "h2"                                   # a ladder: a stronger level never uses a finer unit

[rules.resolution]
TICKET = "warn"                               # must | warn; a load error without fetch

# ── presentation: the bytes grund writes and shows ─────────────────────────
[presentation]
description = "One line beside this project in workspace member lists"
trigger = "$$"
conversation = "link"
rules = "home"                                # home | root — open, §10.1 decision 3, owner #460
[presentation.kinds.FS]
title = "What: behavior, requirements, and constraints"
[presentation.fmt]
anchors = "github"                            # github | gitlab | mkdocs | pandoc | none
links = "index"                               # index | all
exclude = ["docs/architecture/AR-topology.md"]
```

### 7.2 Places on the rows, and what the lists make necessary

Homes are scanned by construction ([§FS-config.3.5.8](../../functional-spec/FS-config.md#358-every-configured-kind-home-is-scanned)), so v1's `[scan] include` already names only the roots of the complement place — the extent of `code`, written in another table. v2 writes it on the row, and `include` therefore has no v2 spelling: **the scan set is the union of every row's places**. `[schema.sources]` keeps what says how a file is read.

A row with no places, or the absence of a `code` row altogether, means the config root — which is the v2 default `include` had under V-D03. `--full` is unchanged ([§FS-config.3.5.11](../../functional-spec/FS-config.md#3511-include-is-a-scan-scope-not-a-fence)): it scans the whole config root past the lists, and a file no place claims is `code`.

| Question | Proposed answer |
|---|---|
| A place inside another row's place (`docs/spec/core` under `page`'s `docs`) | Pending `agent-grounds/grund#457` ([§DISC-core-concerns.10.1](2026-09-30-core-concerns.md#101-the-five-decisions-the-2026-09-30-proposal-leaves-open), decision 2). The deepest place wins, and a `files` entry is deeper than any folder. The same path on two rows is a load error. Today nesting sends the file to the homeless kind ([§FS-config.3.9.2](../../functional-spec/FS-config.md#392-the-homeless-kind)); v1 keeps that. |
| Misplaced declarations | A declaration of kind `K` lives in one of `K`'s places, or in `code`. |
| The index of a citable kind | One per folder, listing that folder's declarations. |
| The stub for a source declaration | A stub in any one of the kind's folders. |
| Project map | One row per kind, linking every place. `code` gains a row — under the name `agent-grounds/grund#457` settles ([§DISC-core-concerns.10.1](2026-09-30-core-concerns.md#101-the-five-decisions-the-2026-09-30-proposal-leaves-open), decision 1). |
| Subjects in the managed block | `Each file in **tests/e2e/** or **tests/integration/**`; `Each source file in **crates/**` replaces *outside the Project map*. |
| `grund config migrate` | Sorts each v1 `include` entry into `folders` or `files` by what is on disk, drops entries that are homes, and reports entries that do not exist. |

### 7.3 A kind's rules render where its places are

Today every citation-directions and chapter-rule bullet is in the root managed block ([§FS-init.2.3.5](../../functional-spec/FS-init.md#235-citation-directions)), so every session reads the rules of every kind. With places on the rows, each rule has a directory: **a kind's bullets render in the `AGENTS.md` of the deepest directory that contains all of its places**, a folder counting as itself and a file as its parent. An agent working in any of the kind's places has that directory on its path.

| Row | Its bullets render in |
|---|---|
| `FS`, `folders = ["docs/spec/core", "docs/spec/plugins"]` | `docs/spec/AGENTS.md` |
| `GOAL`, `files = ["docs/goals.md"]` | `docs/AGENTS.md` |
| `test`, `folders = ["tests/e2e", "tests/integration"]` | `tests/AGENTS.md` |
| `code`, `folders = ["crates"]` | `crates/AGENTS.md` |
| `page`, `folders = ["docs"]`, `files = ["README.md", "AGENTS.md"]` | the root block |
| a citable kind with no place | the root block |

Each such file carries a managed block with the same delimiters and version as the root's, holding the legend, the bullets of the rows that land there in row order, their grounding clause, and the closing line for the global default. The root block keeps what lands at the root plus one sentence: that what a place must cite is in the `AGENTS.md` above it, to be read before writing there.

What it needs, and what it costs:

- **`init`** writes one block per directory, appends to an `AGENTS.md` a directory already has, and removes a block whose directory no longer receives a rule — deleting a file only when the block was all of it.
- **`check`** extends `agents-init` ([§FS-check.3.5](../../functional-spec/FS-check.md#35-invalid-agent-entrypoint-init-block)) to every such directory: a missing, stale or orphaned block is a finding. The render stays a function of the config, so the byte comparison of [§FS-init.2.3.5.8](../../functional-spec/FS-init.md#2358-the-drift-check) holds per file.
- **A generated entrypoint is not a unit of the place it sits in**, unless a row claims it by `files`. Today an `AGENTS.md` without a citation in a grounded non-citable home fails with `ungrounded file in kind home`; its citations are still resolved.
- **Claude Code reads `CLAUDE.md`**, so where the root has that companion each directory gets the symlink arrangement of [§REQ-agents-md.1](../../requirements/REQ-agents-md.md#1-one-source-symlinked-companions). The sunk bytes do not vary by agent, so a symlink is always enough.
- **Chapter rules sink by the same rule**, every subject belonging to a citable kind ([§FS-rules.2](../../functional-spec/FS-rules.md#2-subject-selectors)).
- **The limit.** An agent that does not load nested instruction files on its own gets the rule only through the root's pointer sentence. Which of the supported agents ([§FS-init.2.3.12](../../functional-spec/FS-init.md#2312-supported-agents-and-their-entrypoints)) load them is to be established in the decision record for this change; it is not verified here.

This is one block version and not two: the managed block is frozen surface from `1.0` on ([§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered)), and the `v13` block Phase 2 already writes re-renders exactly the sections this moves.

### 7.4 What the format leaves out

- The `[reference]` table. Its keys went to `[schema]`, `[schema.notes.*]`, `[rules.citations]` and `[presentation]`.
- `strict`, `named_sections`, `section_heading_levels = "loose"` — the last returning as `heading_depth = "may"`.
- The seven inline-note keys and the `lead_size_warning` inline table, with the parser's one inline-table exception.
- `values`, `rules` and `value_chapter`, now `form` and fields; row `format`, now `id_format`; row `syntax`; row `title`, now presentation; `resolve`, now `[rules.resolution]`.
- `[fmt.cross_refs]`, now `anchors` and `links`.
- `extensions`, `comment_prefixes` and `docstring_python`, now `languages` and `definitions`.
- `[output] format` and `color`, now a run flag and nothing; `relative_paths`, now `--path-base`.
- `[scan] include`, now the union of the rows' places ([§DISC-core-concerns.7.2](2026-09-30-core-concerns.md#72-places-on-the-rows-and-what-the-lists-make-necessary)).

With the v2 defaults — slug-only IDs, `[a-z]`-led slugs, named sections always on, the language table, `respect_gitignore` covering build output — a real repository writes little of the above. `init` writes only what differs from a default, and `config show` does the teaching.

### 7.5 What has no v2 spelling, and how `migrate` reports it

The v1 reader is permanent, so nothing here stops loading. What `grund config migrate --to 2` cannot carry across, it says out loud:

| v1 spelling | What migrate does |
|---|---|
| `strict = false` | Refuses the config-only rewrite, and offers `grund fmt --write` to insert the markers as an explicit policy change. v2 recognizes marked citations only. |
| `[output] format = "json"` | Drops the key and reports it as moved to `--format`, because a CI command that parsed JSON will otherwise silently start reading text. |
| `[output] relative_paths = false` | Drops the key and reports it as moved to `--path-base` (V-D02). |
| `[output] color` | Drops it. It is inert today ([§FS-config.3.6](../../functional-spec/FS-config.md#36-output--report-format)). |
| `[citations] default = "must"` | Writes `may`, the only admissible v2 default in that position (V-A18). |
| `[scan] include` | Sorts each entry into a row's `folders` or `files`, drops entries that are homes, and reports entries that do not exist. |
| `[scan] exclude` | Translates rather than copies — see [§DISC-core-concerns.8.3](2026-09-30-core-concerns.md#83-the-two-exclude-keys-mean-opposite-things). |

Every v1 default that differs from its v2 default is written out explicitly, so a migrated config means what it meant; and the before/after report comparison is evidence rather than proof (V-A24), with the reader tests holding the lossless mapping key by key.

## 8. What v2 contradicts in what is specified today

Named here so that a later change extends these points deliberately rather than discovering them. Each stays true of v1, which is read forever.

### 8.1 The homeless kind: three points, not one

A `code` row carrying `folders` and a Project map row crosses three specified points, not the one the proposal's own open decision names:

- [§FS-config.3.9.2](../../functional-spec/FS-config.md#392-the-homeless-kind) — the homeless kind *is not a place*, which is why it has neither `folder` nor `file`.
- [§FS-config.3.9.2.1](../../functional-spec/FS-config.md#3921-declaring-it) — that shape **is** the declaration of the homeless kind, rather than a key on it.
- [§FS-config.3.9.2.5](../../functional-spec/FS-config.md#3925-no-project-map-row-and-the-last-directions-row) — it gets no Project map row.

All three describe v1 exactly and stay true of it. In v2 the row is declared by whatever marks it — its reserved name, if `agent-grounds/grund#457` adopts [§DISC-core-concerns.8.2](2026-09-30-core-concerns.md#82-fs-config3452-is-strengthened-not-crossed) — carries places like any other, and appears in the map like any other — which is what makes what a place must cite renderable beside it.

### 8.2 [§FS-config.3.4.5.2](../../functional-spec/FS-config.md#3452-code-is-reserved-to-the-homeless-kind) is strengthened, not crossed

[§FS-config.3.4.5.2](../../functional-spec/FS-config.md#3452-code-is-reserved-to-the-homeless-kind) reserves the name `code` to the homeless kind. What marks the complement row in v2 is open, and its owner is `agent-grounds/grund#457` ([§DISC-core-concerns.10.1](2026-09-30-core-concerns.md#101-the-five-decisions-the-2026-09-30-proposal-leaves-open), decision 1). **If** it adopts the reserved name, that reservation becomes the row's identity: `code` is the row that holds source declarations and catches what no place claims, and v1's renaming of the homeless kind has no v2 spelling — `[presentation.kinds.code] title` says what it covers instead. The must-cite finding does not name the kind, so a renamed v1 complement migrates with its text intact, and this is the same point read as a definition rather than as a prohibition, which is why it is listed apart from the three above. If it chooses a key on the row instead, the point is neither strengthened nor crossed, and this section is rewritten with that ruling.

Either way the v1 complement is untouched: it is recognized by its shape, non-citable with no home, and may be renamed, under its own meaning, forever ([§FS-config.5.2](../../functional-spec/FS-config.md#52-every-older-version-keeps-its-meaning)).

### 8.3 The two `exclude` keys mean opposite things

`[scan] exclude` is a list of directory **names**, matched per path component ([§FS-config.3.5](../../functional-spec/FS-config.md#35-scan--what-gets-scanned)). `[fmt] exclude` is a list of gitignore-style **globs** against the config root ([§FS-config.3.10.1](../../functional-spec/FS-config.md#3101-entries-are-gitignore-style-globs)). One file, one word, two grammars — and v2 keeps only the glob.

So `migrate` must translate rather than copy. A bare v1 name becomes a glob that matches at any depth, which is what [§FS-config.3.5](../../functional-spec/FS-config.md#35-scan--what-gets-scanned) already means by it. A path-shaped v1 entry — one containing `/` — matched nothing under the component matcher and becomes, in v2, the glob it looks like; `migrate` reports each one, because that is the single case where the rewrite changes what is scanned.

That same path-shaped entry is a v1 defect of its own and is fixed separately, as an immediate located config error: an entry that can never match is a mistake the config should fail loudly on rather than accept in silence. The fix is [§FS-config.3.5.16](../../functional-spec/FS-config.md#3516-an-exclude-entry-containing--is-refused), and it puts one requirement on `migrate`: a historical v1 file that still carries such an entry must be readable by the migrator without first passing the corrected validation, and its report must say how each one's reach changes ([§DF-scan-exclude-component-names.2.4](../../decisions/functional/DF-scan-exclude-component-names.md#24-what-this-leaves-to-the-v2-migration)).

### 8.4 Dropping `include` widens no scan

An `include` entry is a walk root the filter never prunes ([§AR-scanner.1.7](../../architecture/AR-scanner.md#17---full-walks-the-configured-roots-too)) exactly as a home is ([§FS-config.3.5.9](../../functional-spec/FS-config.md#359-a-scan-root-outruns-every-rule-about-descent)), so turning one into a row's `folders` changes what is pruned in no case. The one sentence left without a subject is [§FS-config.3.4.7](../../functional-spec/FS-config.md#347-scan--a-place-that-is-listed-not-scanned)'s, about an `include` entry naming a `scan = false` home: in v2 the home and the list are the same list, so the sentence is rewritten rather than carried.

### 8.5 One correction to the plan's own citation

The `[scan] exclude` fix of [§DISC-core-concerns.8.3](2026-09-30-core-concerns.md#83-the-two-exclude-keys-mean-opposite-things) is described in `agent-grounds/grund#347` as realizing [§FS-config.requirements.5](../../functional-spec/FS-config.md#requirements5-a-mistake-in-the-config-fails-loudly--realized-one-case-deferred). That point's **deferred** case is a different one — a key written at a scope that does not admit it. A silently accepted path-shaped entry is a counterexample to the point's **realized** half, so the requirement's own wording is owed an edit by the change that fixes the bug. Recorded here so the successor issue inherits the right citation rather than the plan's.

## 9. The releases

Every phase lands as one or more pull requests, spec first. The phases below were first written against `0.15.0`, `0.16.0` and `0.17.0`; `0.15.0`, `0.16.0` and `0.16.1` have since shipped, and what they carried is [§DISC-core-concerns.9.8](2026-09-30-core-concerns.md#98-what-shipped), not the phase titles. The phases are therefore named by what they carry rather than by a release, keeping their numbers so that every citation of them still resolves, and the releases each is now planned for are the ledger of [§DISC-core-concerns.9.9](2026-09-30-core-concerns.md#99-the-deprecation-ledger). A planned release is a proposal, not a deadline: no phase after Phase 0 is released for implementation by this document, and each lands only when its tickets are.

### 9.1 Phase 0: the spec, no behavior change

Shipped in `0.15.0`, without the `[scan] exclude` refusal ([§DISC-core-concerns.9.8](2026-09-30-core-concerns.md#98-what-shipped)).


The vocabulary of [§DISC-core-concerns.4.1](2026-09-30-core-concerns.md#41-vocabulary), the `concerns` chapter with its derived key → concern inventory, the [§FS-config.3](../../functional-spec/FS-config.md#3-keys) retitle, the unit rule, this document, and a decision record for the concerns and one for the engine records. No key, finding or byte moves; the gate is `grund check --full` green. The `[scan] exclude` refusal of [§DISC-core-concerns.8.3](2026-09-30-core-concerns.md#83-the-two-exclude-keys-mean-opposite-things) ships separately as a bug fix, because it is the one item of the phase that changes a verdict.

### 9.2 Phase 1: the engine, no user-visible change

Not shipped. Planned for `0.17.0`, the notice release of [§DISC-core-concerns.9.9](2026-09-30-core-concerns.md#99-the-deprecation-ledger), as `agent-grounds/grund#453` and `agent-grounds/grund#454`.


`Project`, `Run` and `Compiled` split out of `Config`, with the v1 reader as `config/v1.rs` lowering into them; the checker split into `conform` and `judge`; the three leaks of [§DISC-core-concerns.2.7](2026-09-30-core-concerns.md#27-the-engine-already-reads-by-concern) closed by signature; `Findings` → `Catalog`, and `Row { place, kind }` with `places()` and `kinds()` over one vector. `Config` stays as a façade, and a component is finished when it no longer names it — a list that only shrinks, held the way [§AR-system.4](../../architecture/README.md#4-dependency-direction)'s upward reads are held. The public names of [§DISC-core-concerns.5.4](2026-09-30-core-concerns.md#54-the-public-surface) take the deprecation path, their notes naming the removal [§DISC-core-concerns.9.7](2026-09-30-core-concerns.md#97-what-a-deprecation-note-may-name) allows.

Gate: every existing test and e2e case byte-identical, no new upward read, and the derived concern inventory of Phase 0 regenerated from the new signatures and unchanged.

### 9.3 Phase 2: the v2 format

Not shipped. Planned for `0.18.0`, as `agent-grounds/grund#455` to `agent-grounds/grund#462`, beside the section-citation ramp `0.16.1` already names for that release ([§FS-rules.7.8](../../functional-spec/FS-rules.md#78-a-newly-counted-section-citation-warns-until-0180)).


The v2 reader (`config/v2.rs`) with keyed rows and fields, measure tables, the envelope, the language table, `[rules.resolution]`, `[presentation.kinds.<name>]`, `anchors` and `links`; the defaults epoch; fields and `form`; `grund config migrate [--to 2] [--write]`; `init` writing v2 and only what differs from a default; the run flags `--format` and `--path-base`; one caution on a v1 config naming `migrate` and no release ([§FS-config.1.2.1](../../functional-spec/FS-config.md#121-no-release-removes-the-agents-location)'s shape); and [§FS-config.3](../../functional-spec/FS-config.md#3-keys) reordered by concern with the v1 key sections under a chapter the reader keeps forever.

The two changes of [§DISC-core-concerns.7.2](2026-09-30-core-concerns.md#72-places-on-the-rows-and-what-the-lists-make-necessary) and [§DISC-core-concerns.7.3](2026-09-30-core-concerns.md#73-a-kinds-rules-render-where-its-places-are) land here, with the points of [§DISC-core-concerns.8](2026-09-30-core-concerns.md#8-what-v2-contradicts-in-what-is-specified-today) extended in the change that crosses them.

Gate: one e2e case per v2 table and one migration case per row of [§DISC-core-concerns.7.5](2026-09-30-core-concerns.md#75-what-has-no-v2-spelling-and-how-migrate-reports-it); every v1 case unchanged; `init` on an empty tree writes a v2 file that `check` passes; and this repository's own config migrates with an identical report.

### 9.4 Phase 3: adoption, and the last removals before `1.0`

Not shipped. Adoption is planned for `0.18.x` and the removals for `0.19.0` ([§DISC-core-concerns.9.9](2026-09-30-core-concerns.md#99-the-deprecation-ledger)).

This organization migrates, `grund` first (`agent-grounds/grund#464`, then `agent-grounds/grund#465`); grund's own fields arrive (`FS.terms`, `AR.placement` and `AR.terms` with `after`, and `status` on `DF` and `DA` at `warn` first if `agent-grounds/grund#464` adopts it, [§DISC-core-concerns.10.2](2026-09-30-core-concerns.md#102-the-five-decisions-the-plan-makes-one-way)); and the public-surface removals land in the minor their notes named (`agent-grounds/grund#466`). A ramp from warning to error that adoption chooses names a release no earlier than the minor after its notice. Each repository's CI pin moves deliberately, which is what lets every repository move when it chooses.

### 9.5 `1.0.0`

`1.0.0` cuts on the last minor's patch line with no new surface: planned from `0.19.x` (`agent-grounds/grund#467`). The v2 format is the `1.0` format and changes afterwards only by the deprecation path; every v1 config keeps loading forever under its own meaning; the user-visible surface of [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered) and the embedding surface of [§FS-distribution.3.1](../../functional-spec/FS-distribution.md#31-rust-grund-core-crate) are frozen; and the pre-`1.0` licence of [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise) has expired. What `1.0` does not promise: that `grund.toml` stops accepting new keys ([§FS-config.5.1](../../functional-spec/FS-config.md#51-new-keys-are-not-a-new-version)), or that the rule language is finished.

`1.0` is the v2 epoch, and that is the reason the phases are ordered as they are: renaming a format after `1.0` would owe an alias for every key.

### 9.6 Phase 4: after `1.0`, the law in the graph

Not shipped, and placed after `1.0` pending `agent-grounds/grund#468` ([§DISC-core-concerns.10.2](2026-09-30-core-concerns.md#102-the-five-decisions-the-plan-makes-one-way), decision 4).


Chapter rules ([§DF-chapter-rules](../../decisions/functional/DF-chapter-rules.md#df-chapter-rules-chapter-rules-are-grounded-controlled-english-declarations-over-producer-neutral-facts)) are already the grounded spelling of a rule: an ID, a sentence, a rationale that cites its goal. Once the rule grammar admits place subjects — which [§FS-rules.2](../../functional-spec/FS-rules.md#2-subject-selectors) refuses today — `[rules.citations]` and grounding can be written as rule declarations, and the tables become sugar that lowers into the same constraints, turning [§FS-rules.6](../../functional-spec/FS-rules.md#6-semantic-deduplication) from a deduplication into a definition. Additive, so it needs no version and can follow `1.0`.

### 9.7 What a deprecation note may name

A deprecation note is written only in the release that actually ships the replacement, and it names as the removing release the minor **two after** that one: a note written in `0.N.0` says `is removed in 0.N+2.0`. Nothing is noticed in advance of its replacement, and no document claims that a notice appeared in a release that has already shipped without it. If the replacement slips a minor, the note slips with it, and so does the removal it names; the ledger of [§DISC-core-concerns.9.9](2026-09-30-core-concerns.md#99-the-deprecation-ledger) moves in the same change.

The requirement's floor is the next minor: release `N` ships the new form beside the old, naming the release the old stops working in, and the old dies no earlier than `N+1` ([§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)). Two minors is longer than that floor and never shorter, and the reason is the release guard rather than a preference. `is removed in <release>` is a pending clause in the guard's closed vocabulary ([§FS-distribution.4.2.3](../../functional-spec/FS-distribution.md#423-the-vocabulary-is-closed)), and the guard requires the version being cut to be below every release a pending removal in the tree names, on every publication path ([§FS-distribution.4.2.6](../../functional-spec/FS-distribution.md#426-every-publication-path-runs-the-release-guard)). A note naming `N+1` makes `N+1` unpublishable until every removal it names is ready, while the old names are still in internal use until adoption finishes; the arithmetic comes from the pending removals, not from how much `N+1` carries.

### 9.8 What shipped

Three releases have shipped since this plan was proposed, and this is what each carried of it, read from their tags and [`docs/changelog.md`](../../changelog.md), not from the phase titles:

- **`0.15.0`, 2026-10-01.** Phase 0 without the bug fix: the concern vocabulary ([§FS-terms.terms.9](../../functional-spec/FS-terms.md#terms9-the-configurations-concerns)), the `concerns` chapter with its derived inventory ([§FS-config.concerns](../../functional-spec/FS-config.md#concerns-every-key-belongs-to-exactly-one-concern)), the [§FS-config.3](../../functional-spec/FS-config.md#3-keys) retitle, this document, and the two decision records. Terms enforcement shipped beside it as [§RULE-terms](../../rules/RULE-terms.md#rule-terms-each-fs-must-have-exactly-one-terms-chapter) (`agent-grounds/grund#290`). None of Phase 1 shipped, and neither did any deprecation note this plan or [§DISC-grund-core-public-surface.6.4](2026-09-22-grund-core-public-surface.md#64-the-releases) placed in `0.15.0`. Every ramp that promised a change in `0.15.0` was moved to `0.16.0` instead.
- **`0.16.0`, 2026-10-04.** The ramps `0.15.0` moved closed, among them bare `grund` (`agent-grounds/grund#301`), which no longer runs `check .`, and the `refs` exit-code ramp, which took `REFS_QUERY_FAILURE_WARNING` out of the crate root with no notice release ([§DISC-grund-core-public-surface.6.3](2026-09-22-grund-core-public-surface.md#63-frontend-only-seams-left-public-but-hidden)). Nothing of v2 shipped.
- **`0.16.1`, 2026-10-05.** A resolved section citation counts in chapter rules, and what it newly fails is a warning until `0.18.0` ([§FS-rules.7.8](../../functional-spec/FS-rules.md#78-a-newly-counted-section-citation-warns-until-0180)). It is not part of this plan, and [§DISC-core-concerns.9.9](2026-09-30-core-concerns.md#99-the-deprecation-ledger) plans around it.

The `[scan] exclude` refusal of [§DISC-core-concerns.8.3](2026-09-30-core-concerns.md#83-the-two-exclude-keys-mean-opposite-things) (`agent-grounds/grund#452`) landed on `main` after `0.16.1`, so it ships in the next cut, as a bug fix outside every phase.

### 9.9 The deprecation ledger

Every surface this plan deprecates has one row in the table below: the surface, the release whose note deprecates it (**Notice**), the release that removes it (**Removal**), and whether the notice has shipped (**Status**, `planned` or `shipped`). Notice and Removal are release versions; a row's removal is at least one minor after its notice ([§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)) and, by [§DISC-core-concerns.9.7](2026-09-30-core-concerns.md#97-what-a-deprecation-note-may-name), two; and a row whose notice is at or below the last shipped release is marked `shipped`, because a planned notice cannot be placed in a release that is already out.

| Surface | Notice | Removal | Status |
|---|---|---|---|
| `Config`, `KindConfig`, `CitationRules` and `Findings` at the `grund-core` root ([§DISC-core-concerns.5.4](2026-09-30-core-concerns.md#54-the-public-surface)), beside the engine records of `agent-grounds/grund#453` | `0.17.0` | `0.19.0` | planned |
| the integrations names [§DISC-grund-core-public-surface.6.4](2026-09-22-grund-core-public-surface.md#64-the-releases) deprecates, behind their facade, as `agent-grounds/grund#466` refreshes their membership | `0.17.0` | `0.19.0` | planned |

No row is shipped yet, and no notice from this plan has appeared in any release. The releases those rows sit in, with what else each carries:

| Release | What it carries |
|---|---|
| `0.16.x` patches | No notice from this plan. |
| `0.17.0` — notice | The engine records (`agent-grounds/grund#453`, `agent-grounds/grund#454`) ship beside the old names, and the refreshed integrations names get their facade (`agent-grounds/grund#466`). Every old name carries a note naming `0.19.0`. |
| `0.18.0` — v2 | The v2 reader and its semantics (`agent-grounds/grund#455` to `agent-grounds/grund#459`), guidance (`agent-grounds/grund#460`), `migrate` (`agent-grounds/grund#461`) and `init` (`agent-grounds/grund#462`). A v1 config gets a caution naming `migrate` and no release. The section-citation ramp `0.16.1` named closes here too. |
| `0.18.x` — adoption | `grund` adopts v2 (`agent-grounds/grund#464`), then rhei, ephor and fissile move their pins (`agent-grounds/grund#465`). A status ramp adoption chooses names a release no earlier than the minor after its notice. |
| `0.19.0` — retirement | Every surface noticed in `0.17.0` is removed (`agent-grounds/grund#466`). |
| `1.0.0` — freeze | Cut from `0.19.x` with no new surface (`agent-grounds/grund#467`). Phase 4 (`agent-grounds/grund#468`) follows it. |

Every cut from `0.17.0` through `0.18.x` stays below `0.19.0`, so no pending removal in the tree blocks one ([§FS-distribution.4.2.6](../../functional-spec/FS-distribution.md#426-every-publication-path-runs-the-release-guard)). The `is removed in 0.19.0` clauses are written by `agent-grounds/grund#453` and `agent-grounds/grund#466` in the release that ships their replacements, never by this document. If that release is not `0.17.0`, the notice, v2, retirement and freeze releases each move by the same number of minors, and this ledger moves in the same change ([§DISC-core-concerns.9.7](2026-09-30-core-concerns.md#97-what-a-deprecation-note-may-name)).

Three things are deliberately not rows. `REFS_QUERY_FAILURE_WARNING` left in `0.16.0` with no notice release: it is unpaid notice debt, recorded in [§DISC-grund-core-public-surface.6.3](2026-09-22-grund-core-public-surface.md#63-frontend-only-seams-left-public-but-hidden) as a breach of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) and not excused by any number written here. The deprecated `.agents/grund.toml` location keeps its caution with no release ([§FS-config.1.2.1](../../functional-spec/FS-config.md#121-no-release-removes-the-agents-location)), and every v1 config keeps loading forever ([§FS-config.5.2](../../functional-spec/FS-config.md#52-every-older-version-keeps-its-meaning)), so neither is ever removed.

## 10. What this discussion still has to settle

The chapters above fix the model, the format, what it crosses and the schedule. What follows is open, and [§DISC-core-concerns.1.1](2026-09-30-core-concerns.md#11-what-accepting-it-accepts) still holds over all of it. Each item says where it stands: `Open — owner #N` names the ticket of `agent-grounds/grund` that rules on it, and `Settled by` names what settled it. No alternative below is chosen by this document; an item is closed here only by its owner's ruling, recorded in the spec point or decision record that ruling extends.

### 10.1 The five decisions the 2026-09-30 proposal leaves open

Named as open by their author in the comment that proposed them, and carried here unanswered.

1. **What marks the `code` row.** v1 recognizes the homeless kind by shape: non-citable with no home. With places on it that shape is gone. Proposed: `code` is the reserved name of the row that holds source declarations and catches what no place claims ([§DISC-core-concerns.8.2](2026-09-30-core-concerns.md#82-fs-config3452-is-strengthened-not-crossed)). The alternative is a key on the row. **Open — owner #457.**
2. **Deepest place wins, for every row.** [§DISC-core-concerns.7.2](2026-09-30-core-concerns.md#72-places-on-the-rows-and-what-the-lists-make-necessary) proposes it. The alternative keeps today's rule, where nested places of two ordinary kinds fall to `code`, and makes nesting under `code` the one exception. **Open — owner #457.**
3. **Where a v1 config's rules render.** Proposed: a v2 key, `[presentation] rules = "home" | "root"`, default `home`; v1 renders at the root as today and `migrate` writes `root` explicitly, so a migrated config produces the files it produced. The alternative is one layout for everyone, which puts new files in every v1 repository on its next `init`. **Open — owner #460**, with `agent-grounds/grund#461` for what `migrate` writes.
4. **Claude delivery.** Proposed: the symlink per directory. The alternative is path-scoped files under `.claude/rules/`, which avoids the symlinks and adds a second mechanism. **Open — owner #460.**
5. **Fields in the sunk block.** Proposed: when per-kind fields ship, they render in the same block as the kind's rules, so a directory's block says both what a declaration there looks like and what it cites. **Open — owner #460**, with `agent-grounds/grund#458` for the fields themselves.

### 10.2 The five decisions the plan makes one way

Each is a choice `agent-grounds/grund#347` makes and has not been ruled on; saying otherwise changes the plan rather than this model.

1. **#290 now or as a field.** #290 is approved and planned as a `RULE` row for the Terms chapter. Implement it as a rule now and migrate it to a field in Phase 3, or wait for fields? The plan waits. **Settled by** shipped work: `agent-grounds/grund#290` shipped Terms as [§RULE-terms](../../rules/RULE-terms.md#rule-terms-each-fs-must-have-exactly-one-terms-chapter) in `0.15.0` ([§DISC-core-concerns.9.8](2026-09-30-core-concerns.md#98-what-shipped)), so the plan did not wait. Whether it moves into a field is `agent-grounds/grund#464`'s.
2. **Migrate grund's `DF` and `DA` status lines.** About a hundred decision records carry `**Status:**` prose lines. Migrating them to `## status:` headings is mechanical and is what makes `status` a checked field; leaving them makes `DF` and `DA` the two kinds without a checked shape. The plan migrates them in Phase 3. **Open — owner #464.**
3. **The embedding surface at `1.0`.** [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered) covers the user-visible surface; the plan extends the `1.0` promise to the `grund-core` public root as the audit leaves it. The alternative freezes the CLI, config, JSON and LSP surfaces and leaves the Rust API at its own cadence. **Open — owner #466**, with the release evidence `agent-grounds/grund#467` gathers.
4. **Phase 4 after `1.0`.** Rules as declarations is additive and could precede `1.0`, at the cost of another minor. The plan puts it after. **Open — owner #468**, with `agent-grounds/grund#467` owning where the `1.0` boundary falls.
5. **The `languages` default set.** V-A14 fixes the default at the v2 epoch; *which* languages are in it is a list to sign off on before Phase 2. The plan proposes today's `extensions` default, classified. **Open — owner #456**, together with its own question of which release may tighten what a source file is recognized as.

### 10.3 The draft's own open questions, closed

The draft this document replaces carried eight open questions, and the agora answered all eight. They are recorded closed rather than dropped, so that a reader who remembers one can see where it went: the obligation unit stays two composing keys (V-A19); scalar fields are headings and `DF` and `DA` adopt them on a `warn`-first ramp (V-A07); nesting is keyed tables rather than array-of-tables (V-A05); kind-scope schema budgets are deferred (V-A16); `[output]` becomes run flags, with the report path base ruled V-D02; `section_separator` stays (V-A12); the table keeps the name `kinds` (V-A05); and `index` belongs to the citable kind's home rather than to every place (V-A10).
