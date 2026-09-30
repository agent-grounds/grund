# DF-config-concerns: a key's concern is derived from what a finding from it can be about

**Status:** Accepted
**Date:** 2026-09-30

## 1. Context

[§FS-config.concerns](../../functional-spec/FS-config.md#concerns-every-key-belongs-to-exactly-one-concern) states the cut — schema is one node on its own, rules are at least two, presentation is the bytes `grund` writes — and then claims something stronger than a definition: that *the complete assignment is one row per key of the format in force*. A claim of completeness is worth exactly what its derivation is worth, so the derivation is recorded here to be re-run rather than trusted, in the two-stage shape [§DF-config-scope-override.2.2](DF-config-scope-override.md#22-the-inventory-is-derived-in-two-stages) already uses for the scope inventory.

The claim is worth checking for a reason the scope inventory did not have. [§DISC-core-concerns](../../discussions/proposals/2026-09-30-core-concerns.md#disc-core-concerns-three-concerns-over-two-trees--how-the-configuration-the-core-spec-and-the-engine-are-organized) proposes the concerns as the organizing axis of the v2 format, and [§DISC-core-concerns.2.2](../../discussions/proposals/2026-09-30-core-concerns.md#22-one-table-holds-five-concerns) is the evidence that today's arrangement is not that axis: one `[reference]` table holds five different kinds of thing. An inventory written by reading the key's name, or the table it happens to sit in, would carry exactly the arrangement being left behind into the format `grund` freezes at `1.0`. So the classification is taken from the code that reads the key, and the table it is written under is never the reason.

Serves [§GOAL-configurable](../../goals.md#goal-configurable-every-default-is-overridable): *which of these three is this key?* becomes a row to look up rather than a judgement to re-make, and a key added later is classified by the same question the chapter states.

## 2. Decision

### 2.1 Stage 1 — the candidate set, from the parse sites

Every committed parse site is enumerated, and the union of the keys they accept is the candidate set: `config/parse.rs` (the top-level keys and the project tables), `config/kind_table.rs` (a `[[kinds]]` row), and `config/citations.rs` (`[citations]` and a `[citations.<KIND>]` table). This is complete by construction — a key no parser accepts cannot be written — and it is the same enumeration [§DF-config-scope-override.2.1](DF-config-scope-override.md#21-one-relation-stated-once-and-stated-as-a-relation) makes, because the question *which keys are there* has one answer whatever is then asked of them.

That is **61 keys**. One key the readers recognize is not among them: `[[kinds]] prefix` is matched only in order to refuse it with the release that removed it ([§FS-config.3.4.6](../../functional-spec/FS-config.md#346-prefix-the-former-spelling-of-kind-removed-in-0130)), so it is not a key of the format in force. The exclusion is a claim about the reader rather than about the inventory, so `tests/integration/test_config_concern_inventory.py` holds it to the refusal: the row may stay absent only while no config may write the key.

### 2.2 Stage 2 — the criterion, from the read sites

For each candidate, every site that reads its value is taken, and the question [§FS-config.concerns](../../functional-spec/FS-config.md#concerns-every-key-belongs-to-exactly-one-concern) states is asked of what that site produces: **is the finding about one node, about a relation between nodes, or is it not a finding at all but bytes?**

One read site is excluded from the question, and excluding it is what makes the rest decisive. `grund config show` (`cli_config.rs`) reads every key without exception, because printing the effective configuration back is what it is for ([§FS-config.4.2](../../functional-spec/FS-config.md#42-grund-config-show-path)). A read that only echoes the file classifies nothing; `api/config.rs` and `testing.rs` are excluded for the same reason.

What is left says which concern each key belongs to:

| Concern | The read sites that establish it | What they produce |
|---|---|---|
| Schema | `scanner/*`, `grammar/*`, `checker/sections.rs`, `checker/inline_style.rs`, `checker/sizes.rs`, `checker/homes.rs`, `resolver/body.rs` | A finding naming one declaration, one note, one heading or one file — a malformed ID, an oversized lead, a note over its budget, a heading depth that does not match its path |
| Rules | `checker/grounding.rs`, `checker/citations.rs`, `checker/references.rs`, `checker/index.rs`, `checker/index_entries.rs`, `queries/citation_counts.rs` | A finding naming two nodes or a missing edge — a forbidden citation, an ungrounded file, a citation that does not resolve, a declaration absent from its kind's index |
| Presentation | `writers/*`, `templates/*`, `resolver/link_targets.rs`, `api/check.rs` and the other report surfaces | Bytes: what `fmt` writes, what `init` renders, how a report or a link is spelled. No finding of its own — only drift between what was written and what the config now renders ([§FS-check.3.5](../../functional-spec/FS-check.md#35-invalid-agent-entrypoint-init-block)) |
| Envelope | `config/discovery.rs`, `workspace/scope.rs`, `workspace/optional_members.rs` | Nothing about any node: which file governs, and which projects there are |

Three read sites cross a concern and are known: `scanner/units.rs` reads a grounding rule lowered into *record headings to depth d*, `resolver/link_targets.rs` reads the cross-reference anchor format, and `scanner/agent_entrypoints.rs` reads `conversation`. Each is recorded in [§DISC-core-concerns.2.7](../../discussions/proposals/2026-09-30-core-concerns.md#27-the-engine-already-reads-by-concern), and none of the three changes the key's classification — the key is classified by what a finding from it is about, not by which component happens to hold the read.

### 2.3 The inventory

One row per key of the format in force, in the order [§FS-config.3](../../functional-spec/FS-config.md#3-keys) presents the tables, and alphabetically inside a table. The classification is one of four words: three concerns, and `envelope` for a key that is outside them ([§FS-config.concerns](../../functional-spec/FS-config.md#concerns-every-key-belongs-to-exactly-one-concern)).

| Key | Concern | Why |
|---|---|---|
| `grund_config_version` | envelope | Chooses the reader. It is read before any concern and constrains no node. |
| `project_name` | envelope | Identity, and the workspace alias: it says which project a coordinate belongs to, never what holds inside one. |
| `project_description` | presentation | The line a generated member list renders beside the alias. No checker, scanner, formatter or query reads it. |
| `[reference] conversation` | presentation | How an agent is told to render a citation in conversation. It reaches a finding only as managed-block drift, which is a rule over the generated region. |
| `[reference] grounding_level` | rules | Which unit inside a governed file the grounding obligation is asked of — the subject's node, not a property of one. |
| `[reference] inline_note_layout` | schema | The house shape of one note. |
| `[reference] inline_note_layout_check` | schema | Whether that shape is reported, and how loudly. Still one note. |
| `[reference] inline_note_max_columns` | schema | One note's width. |
| `[reference] inline_note_max_lines` | schema | One note's height. |
| `[reference] inline_note_suggested_lines` | schema | The soft form of the same measure. |
| `[reference] inline_style` | schema | What one note may contain beside its citation. |
| `[reference] lead_size_warning` | schema | One declaration's lead, measured against a budget. |
| `[reference] marker` | schema | Which token is a citation at all. |
| `[reference] require_grounding` | rules | Whether a file must cite something: the finding names a file and the citations it does not have. |
| `[reference] shorthand` | schema | Whether a persisted number-only citation is well formed. One citation site. |
| `[reference] strict` | schema | Whether an unmarked ID-shaped token is a citation. It decides what exists. |
| `[reference] trigger` | presentation | The characters an editor turns into the marker. What it decides is what an editor types, which no check judges. |
| `[reference] warn_on_suggested` | schema | The severity of the suggested-lines budget, which is still about one note. |
| `[id] format` | schema | The shape a well-formed ID has. |
| `[id] named_sections` | schema | Whether a heading may carry a name rather than a number. |
| `[id] number_pattern` | schema | What a numeric ID component may be. |
| `[id] section_heading_levels` | schema | Heading depth against path depth, inside one declaration. |
| `[id] section_separator` | schema | Where one coordinate's components divide. |
| `[id] slug_pattern` | schema | What a slug may be. |
| `[[kinds]] citable` | schema | Whether the row declares an ID namespace or is a place and nothing more. |
| `[[kinds]] fetch` | schema | How a declaration of an external kind comes to exist. Acquisition is not a fourth concern (V-D01, [§DISC-core-concerns.6.3](../../discussions/proposals/2026-09-30-core-concerns.md#63-ruled)); what it decides is what the corpus holds. |
| `[[kinds]] file` | schema | The kind's home, as a single file. |
| `[[kinds]] folder` | schema | The kind's home, as a folder. |
| `[[kinds]] format` | schema | The kind-scope ID template. |
| `[[kinds]] grounding_level` | rules | The unit of the grounding obligation, at the kind scope. |
| `[[kinds]] index` | rules | Which document the kind's declarations must appear in. The finding names a declaration and that document. |
| `[[kinds]] kind` | schema | The name of the namespace or place the row declares. |
| `[[kinds]] require_grounding` | rules | The grounding obligation, at the kind scope. |
| `[[kinds]] resolve` | rules | The strength of the rule that a citation of this kind resolves. |
| `[[kinds]] rules` | schema | That a declaration of this kind is a rule sentence rather than prose — the form of one declaration. |
| `[[kinds]] scan` | schema | Whether the place is read at all, which decides what exists. |
| `[[kinds]] title` | presentation | The Project map's words for the kind. |
| `[[kinds]] value_chapter` | schema | Which chapter's named children are value roots in every declaration of the kind. |
| `[[kinds]] values` | schema | That a declaration of this kind is a value. Again the form of one declaration. |
| `[scan] comment_prefixes` | schema | What opens a comment, and therefore where a source declaration can be. |
| `[scan] docstring_python` | schema | Whether a Python docstring is read as a declaration. |
| `[scan] exclude` | schema | Which directories hold nothing. |
| `[scan] extensions` | schema | Which files are read at all. |
| `[scan] include` | schema | The roots of the scan: what exists. |
| `[scan] respect_gitignore` | schema | Whether the ignore files bound the same question. |
| `[output] color` | presentation | Reserved and read by nothing today ([§FS-config.3.6](../../functional-spec/FS-config.md#36-output--report-format)). What it would decide is bytes. |
| `[output] format` | presentation | Whether the report is written as text or as JSON. |
| `[output] relative_paths` | presentation | Which base a reported path is printed against. |
| `[fmt.cross_refs] anchor_format` | presentation | Which anchor dialect a written link uses. |
| `[fmt.cross_refs] enabled` | presentation | Whether `fmt` writes the link beside a citation. |
| `[workspace] include_root` | envelope | Whether the root is one of the projects. |
| `[workspace] members` | envelope | Which projects there are. |
| `[workspace] optional_members` | envelope | The same, for a member that may be absent. |
| `[citations] default` | rules | The direction every pair no table names takes. |
| `[citations.<KIND>] default` | rules | The same, under one citing name. |
| `[citations.<KIND>] may` | rules | A pair left unchecked: still a statement about a pair. |
| `[citations.<KIND>] must` | rules | An obligation between a citing node and a cited kind. |
| `[citations.<KIND>] must-not` | rules | A prohibition between the same two. |
| `[citations.<KIND>] should` | rules | The suggestion form of the obligation. |
| `[citations.<KIND>] should-not` | rules | The suggestion form of the prohibition. |
| `[fmt] exclude` | presentation | Which files `grund fmt --write` may rewrite. |

That is 33 schema, 13 rules, 10 presentation and 5 envelope.

### 2.4 The rows that were arguable, and which way they went

Six rows had a defensible second answer. Each is recorded with the reason it went the way it did, because a row nobody can argue with does not need a record and these do.

- **`[[kinds]] index` — rules, against schema.** The index document is named on the row, and [§DISC-core-concerns.5.1](../../discussions/proposals/2026-09-30-core-concerns.md#51-the-core-records) keeps it spelled there (V-A10), which is an argument for schema: the key names a node. It goes to rules because the criterion asks about the finding rather than about the field: `missing-index-entry` ([§FS-check.3.18](../../functional-spec/FS-check.md#318-declaration-missing-from-its-kinds-index)) names a declaration *and* the document it is absent from, and [§FS-check.4.1.2](../../functional-spec/FS-check.md#412-an-index-entry-does-not-count) subtracts an index entry from a declaration's inbound citation count. Both need two nodes. Where a key is spelled and which concern it belongs to are separate questions.
- **`[[kinds]] fetch` — schema, against rules.** Its one read inside `check` is `checker/references.rs`, where a kind having a `fetch` command changes which finding an unresolved citation gets — a relation, which argues for rules. It goes to schema because that read selects the *message* while `resolve` beside it selects the *strength*, and the key's own work is done in `writers/fetch.rs`, which brings a declaration into existence. [§DISC-core-concerns.5.1](../../discussions/proposals/2026-09-30-core-concerns.md#51-the-core-records) splits the pair the same way: `origin` on the kind, `resolution` in the rules.
- **`[reference] conversation` — presentation, against rules.** It does reach a finding, which the agora established against the draft (V-C01): the managed block's expected text renders from it, so a block that has drifted is reported. It stays presentation because the finding is the drift comparison over a generated region, and the checker is handed the rendered expectation rather than the key ([§DISC-core-concerns.6.2](../../discussions/proposals/2026-09-30-core-concerns.md#62-corrected)).
- **`[reference] trigger` — presentation, on a corrected reason.** The draft kept it in presentation because it is *never persisted*, which is false — `fmt` rewrites it (V-C07). It stays presentation on the reason that survives: what it decides is what an editor types, which no check ever judges.
- **`[output] color` — presentation, against envelope.** Nothing reads it, so no finding and no byte can be pointed at. A key that decides nothing today is tempting to park outside the concerns, but the envelope is *read before the concerns and constrains no node*, which `color` is not: it is a byte-shaping key whose bytes are not written yet. [§DISC-core-concerns.7.5](../../discussions/proposals/2026-09-30-core-concerns.md#75-what-has-no-v2-spelling-and-how-migrate-reports-it) drops it in v2 for exactly that reason.
- **`project_description` — presentation, against envelope.** It sits beside `project_name` at the top of the file and is written by `init` as identity, which argues for the envelope. It is presentation because it decides bytes a member list renders and nothing else does ([§FS-config.3](../../functional-spec/FS-config.md#3-keys)), and [§FS-config.concerns](../../functional-spec/FS-config.md#concerns-every-key-belongs-to-exactly-one-concern) says so in as many words; v2 moves it to `[presentation] description` (V-A03).

A seventh, `[scan] include`, was not in doubt and is noted because it looks as though it should be: naming a walk root decides what exists, which is schema however much it reads like a rule about descent.

## 3. Alternatives rejected

### 3.1 Classify by the table the key is written under

The cheapest reading, and the one that produces the wrong answer sixteen times: `[reference]` alone splits across schema, rules and presentation, `[[kinds]]` across schema, rules and presentation again, and `[output]` holds one key that is read by nothing. [§DISC-core-concerns.2.2](../../discussions/proposals/2026-09-30-core-concerns.md#22-one-table-holds-five-concerns) is that table read key by key. Classifying by the table would also make the inventory unfalsifiable — it would agree with the file by construction, and say nothing the file does not already say.

### 3.2 A fourth concern

Two candidates were weighed and both are refused upstream of this record: *acquisition*, for `fetch`, which V-D01 refuses because the concerns cover the read-and-check pipeline and a command that runs outside `check` does not make one; and the envelope as a fourth concern, which [§FS-config.concerns](../../functional-spec/FS-config.md#concerns-every-key-belongs-to-exactly-one-concern) refuses because it is not a kind of setting but the thing the concerns are read inside. `envelope` is still one of the four words a row may carry, because *this key is outside the three* is an answer the inventory has to be able to give.

### 3.3 Put the inventory in `FS-config.concerns` itself

The chapter is where the reader looks, and [§FS-config.principle.inventory](../../functional-spec/FS-config.md#principleinventory-the-settings-admitted-at-both-the-project-and-the-kind-scope) sets the precedent of a derived list living in the specification. It is refused for two reasons. The derivation is the argument, and the argument belongs with the decision that makes it, which is the shape [§DF-config-scope-override.2.2](DF-config-scope-override.md#22-the-inventory-is-derived-in-two-stages) already uses for the only comparable list. And `docs/functional-spec/FS-config.md` is at 737 lines against a 750-line budget: a sixty-one-row table plus its method would put the file over it, and shrinking the prose to fit would trade the reason for the list.

## 4. What it costs

Adding a key to the format means adding a row here, and `tests/integration/test_config_concern_inventory.py` is what makes that mandatory rather than polite: a key with no row, a row naming no key, a key with two rows and a classification outside the four words each fail it by name. That is additive surface and never a version change ([§FS-config.5.1](../../functional-spec/FS-config.md#51-new-keys-are-not-a-new-version)).

The test holds the inventory's *shape* against the parse sites. It cannot hold a row's *answer*: a key classified `presentation` that is really a rule passes. That is the price of a criterion stated in English, and it is paid the way [§DF-config-scope-override](DF-config-scope-override.md#df-config-scope-override-the-committed-scopes-are-one-relation-stated-once) pays it — by recording the derivation so the answer can be re-run, and by naming in [§DF-config-concerns.2.4](DF-config-concerns.md#24-the-rows-that-were-arguable-and-which-way-they-went) the rows where re-running it is most likely to be worth someone's time.
