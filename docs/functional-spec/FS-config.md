# FS-config: grund reads a TOML config file found by walking up

`grund` is zero-config out of the box ([§GOAL-zero-config](../goals.md#goal-zero-config-works-on-any-conformant-tree)) and fully configurable when a project's conventions diverge ([§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable)). This spec defines the contract: where the config lives, what it contains, what it overrides, and how malformed configs are reported.

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, home, citable, body, section, coordinate, lead, index, catalog), [§FS-terms.terms.2](FS-terms.md#terms2-citations) (marker, citation, qualified citation, shorthand, citation site), [§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (source declaration, stub, doc-comment, note), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope, config root, place, complement place, workspace, member, alias), [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, severity, suggestion), [§FS-terms.terms.6](FS-terms.md#terms6-rules-and-directions) (direction, level, rule, unit, grounded), [§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations) (value, snapshot), and [§FS-terms.terms.9](FS-terms.md#terms9-the-configurations-concerns) (schema, rules, presentation, envelope).

- **homeless kind** — The row with neither `folder` nor `file`: v1's spelling of the
  complement place ([§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure)), whose default name is `code`.
- **citing kind** — The kind a citation is attributed to — the kind of the home its file sits
  in, or the homeless kind when no home claims it.
- **CLI base** — The directory a report's paths are rendered against under `relative_paths =
  false`, in place of the config root.

## principle: A setting written at a narrower scope wins

Every setting `grund` reads from a repository is written at one of three **committed scopes**, from widest to narrowest: the **built-in default** the tool supplies ([§FS-config.3](FS-config.md#3-keys-version-1)), the **project** — the top-level tables of `grund.toml` — and the **kind**, written either as a `[[kinds]]` row ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)) or as a `[citations.<KIND>]` table ([§FS-config.3.9.4](FS-config.md#394-defaults-and-precedence)), which are two spellings of writing at the one scope. This chapter states once how those scopes relate, so that a key's own section may say what the key means and leave how it resolves to one place. It describes what `grund` already does: it is the general form of sentences this specification carries one at a time ([§FS-config.3.4.8.3](FS-config.md#3483-precedence-is-row-over-global), [§FS-config.3.4.7.2](FS-config.md#3472-not-scanned-however-the-scan-arrives), [§FS-config.4.2.1](FS-config.md#421-kinds-rows-print-what-they-do-not-inherit)), and it adds no behavior and no key.

It serves [§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable) by making *can I set this per kind?* one read instead of a hunt through ten table sections. Every clause below is conditional on a setting being **admitted** at more than one committed scope ([§FS-config.principle.admission](FS-config.md#principleadmission-the-relation-reaches-only-scopes-that-admit-the-setting)); none of them claims that every default is overridable everywhere, and [§FS-config.principle.inventory](FS-config.md#principleinventory-the-settings-admitted-at-both-the-project-and-the-kind-scope) is the complete list of the settings this chapter's override clauses reach at all.

### principle.rungs: The committed scopes are a relation, not a closed list

Where a setting is admitted at more than one committed scope, the value written at the **narrower** scope is the effective one for everything that scope governs, and the wider value stands wherever the narrower scope is silent. Narrowness is the ordering above — a `[[kinds]]` row is narrower than the project's tables, which are narrower than the built-in defaults — and it is stated as a relation rather than as an enumeration of three: a scope introduced later, whether finer than a kind or between the project and the kind, takes its place in the same relation without this chapter being rewritten. No such scope exists today, and adding one would be additive surface rather than a new schema version ([§FS-config.5.1](FS-config.md#51-new-keys-are-not-a-new-version)).

### principle.unit: One leaf key overrides, and one value is whole

Two units, and they are not the same one. What overrides is a single **leaf key**: writing one key at a narrower scope leaves every other key that scope inherits exactly as it was, which is why a `[reference]` table naming one key does not discard the rest of the `[reference]` defaults — every key is optional and an omitted key takes the default value ([§FS-config.3](FS-config.md#3-keys-version-1)). What is overridden is the **whole value**, an array included: a key whose value is a list replaces the inherited list rather than merging into it. Writing `[[kinds]]` at all replaces the built-in kind list entirely ([§FS-config.3.4.4](FS-config.md#344-the-default-kinds)) — that is this whole-value rule applied to the list of rows, not a second rule beside it.

In v2 a grounding ladder is one setting, so a narrower ladder replaces the wider one whole and inherits none of its rungs ([§FS-config-v2.rules.grounding](FS-config-v2.md#rulesgrounding-grounding-is-a-ladder-replaced-whole)). v1 spells grounding as two leaf keys, `require_grounding` and `grounding_level`, and a v1 row still inherits each of them independently ([§FS-config.3.4.8.3](FS-config.md#3483-precedence-is-row-over-global)).


### principle.admission: The relation reaches only scopes that admit the setting

A scope **admits** a fixed set of keys, and the relation above holds only between the scopes that admit the same setting. A key written at a scope that does not admit it is not a weaker override: it is not a key there at all, and it is refused as an unknown key pointing at its line ([§FS-config.3](FS-config.md#3-keys-version-1), [§FS-config.4.3](FS-config.md#43-invalid-config-behavior)). So `inline_style` under `[reference]` and `inline_style` on a `[[kinds]]` row are not a wide and a narrow spelling of one setting ([§FS-config.3.1.8](FS-config.md#318-inline_style-and-the-note-budgets)) — the second does not exist. A key's own section states its scopes only where it is admitted at more than one; where it is admitted at one, the table it is documented under is that statement. This chapter says where a key may be written, and what is not a key at all stays [§FS-config.6](FS-config.md#6-what-is-not-configured-here)'s: a thing named there has no scopes, because it is not a setting.

### principle.cli: A CLI input enters at the scope its flag spells

A command-line input is not a fourth scope that wins over all three. It enters the relation at the scope its flag spells and resolves from there like any value written at that scope. `grund check --require-grounding` spells the **project** default, so an explicit `require_grounding = false` on a row still wins over the flag ([§FS-config.3.4.8.3](FS-config.md#3483-precedence-is-row-over-global)): the flag and the key are one knob, and the row's word is the more specific one.

### principle.boundary: The project is the widest committed scope

Scopes stop at the project. A workspace member is configured by its own `grund.toml`, or by the built-in defaults where it has none, and no section of a member's config inherits from the workspace root's ([§FS-config.3.9.3](FS-config.md#393-alias-matching), [§FS-workspace.2](FS-workspace.md#2-workspace-configuration)). A root and a member are therefore a **boundary rather than a rung**: there is no precedence between them, because neither is ever consulted for the other's values. This is a statement about which config governs a member's own files, and not about which config a command discovers from a given directory — that is [§FS-config.1](FS-config.md#1-file-location-and-discovery)'s, and the two answer different questions.

### principle.declared-rows: A declared row does not inherit from a built-in one

A `[[kinds]]` row a project writes starts from the defaults of this specification rather than from the built-in row of the same name: writing `[[kinds]]` replaces the built-in list entirely ([§FS-config.principle.unit](FS-config.md#principleunit-one-leaf-key-overrides-and-one-value-is-whole)), so there is no wider row left to inherit from, and a declared `FS` row takes this specification's own default for `citable` rather than the built-in row's and has no home at all until it names one. Exactly one built-in attribute still reaches a declared row, and it is keyed on the kind's **name** rather than on a scope: `index` ([§FS-config.3.4.2.4](FS-config.md#3424-the-default-is-per-kind-name)), applied only where the row left `index` unset, names a `folder`, and is citable. The built-in home, file, title, and `citable` of a same-named built-in kind reach nothing a project declares.

### principle.exceptions: Two shapes that look like rungs and are not

Two places in these keys resemble the relation above without being it, and each keeps the wording it already has. The first is the top term of [§FS-config.3.9.4](FS-config.md#394-defaults-and-precedence)'s ladder — an explicit target list over a `default` — which is specificity of **target match** between keys each admitted at one scope, not a narrower scope overriding a wider one; its lower two terms are an ordinary instance and appear in the inventory below. The second is [§FS-config.3.4.2.4](FS-config.md#3424-the-default-is-per-kind-name)'s `index`, which is **produced from the kind name** at the default-producing end rather than resolved between two written values; overriding it on a row is the ordinary instance, bounded by [§FS-config.principle.declared-rows](FS-config.md#principledeclared-rows-a-declared-row-does-not-inherit-from-a-built-in-one). Neither is an exception to the rule: each is a different shape the rule does not describe.

### principle.install-local: The relation governs committed repository state only

Every scope above is something a repository commits. The **install-local** axis lies outside it: the user's conversation citation preference and its per-agent partial are machine state installed by `grund integrations --write`, and their precedence is stated where they live ([§FS-integrations.4.3](FS-integrations.md#43-user-preference-and-global-agent-instructions), [§FS-integrations.4.4](FS-integrations.md#44-per-agent-overrides)) rather than restated here. Naming it here is what keeps the boundary visible from the chapter that states the relation, and the line it does not cross is unmoved: no machine-local value may change whether a repository is well-formed ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)). For the author of a new key the order is **classify first, verify by reach** — decide whether the key carries committed repository semantics or install-local presentation, then check that its reach agrees. Reach alone does not classify: inert `[output] color` ([§FS-config.3.6](FS-config.md#36-output--report-format)) changes no verdict and is committed repository state all the same.

### principle.inventory: The settings admitted at both the project and the kind scope

Four settings are admitted at both the project scope and the kind scope, and for the keys of [§FS-config.3](FS-config.md#3-keys-version-1) this list is complete:

| Project scope | Kind scope | Stated at |
|---|---|---|
| `[reference] require_grounding` | a row's `require_grounding` | [§FS-config.3.4.8.3](FS-config.md#3483-precedence-is-row-over-global) |
| `[reference] grounding_level` | a row's `grounding_level` | [§FS-config.3.4.8.3](FS-config.md#3483-precedence-is-row-over-global) |
| `[id] format` | a row's `format` | [§FS-config.3.4.10.1](FS-config.md#34101-format) |
| `[citations] default` | `[citations.<KIND>] default` | [§FS-config.3.9.4](FS-config.md#394-defaults-and-precedence) |

Every other key of [§FS-config.3](FS-config.md#3-keys-version-1) is admitted at one committed scope only, so [§FS-config.principle.rungs](FS-config.md#principlerungs-the-committed-scopes-are-a-relation-not-a-closed-list) never reaches it and its table section is the whole statement of where it may be written. The list is derived from the parse sites and the resolution sites rather than read off this prose, and the derivation is recorded with the decision behind this chapter ([§DF-config-scope-override.2.2](../decisions/functional/DF-config-scope-override.md#22-the-inventory-is-derived-in-two-stages)), so that a reader re-runs it instead of trusting it. A key that gains a second scope is added here in the same change, and that is additive surface ([§FS-config.5.1](FS-config.md#51-new-keys-are-not-a-new-version)).

The table is the v1 reader's. The v2 reader has its own inventory, derived the same way from its own parse sites: `id_format`, the citation `default` and the grounding ladder are each admitted at the project and at the kind ([§FS-config-v2.schema.places](FS-config-v2.md#schemaplaces-rows-and-their-places), [§FS-config-v2.rules.citations](FS-config-v2.md#rulescitations-rulescitations), [§FS-config-v2.rules.grounding](FS-config-v2.md#rulesgrounding-grounding-is-a-ladder-replaced-whole)).


## concerns: Every key belongs to exactly one concern

A key of [§FS-config.3](FS-config.md#3-keys-version-1) is one of three kinds of thing, and what tells the three apart is what a finding from the key can be about. **Schema** is what exists and what a well-formed one looks like: a finding from a schema key is about **one node** on its own. **Rules** are how nodes relate: a finding from a rules key needs **at least two**. **Presentation** is the **bytes** `grund` writes or shows: a presentation key produces no finding of its own, and reaches one only where a written byte has drifted from what the config now renders, which is a rule over the generated region rather than a finding of the key ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)). Every key belongs to exactly one of the three, and the test is the definition: a key added later is classified by asking what a finding from it would be about, not by which table it happens to be written in.

Outside the three sits the **envelope** — `grund_config_version`, `project_name`, and the three `[workspace]` keys ([§FS-config.3.8](FS-config.md#38-workspace--sub-project-members)). They are read before any concern, to decide which file governs and which projects there are, and they constrain no node. The envelope is not a fourth concern; it is what the concerns are read inside. `project_description` is not in it: it decides bytes a member list renders and is presentation ([§FS-config.3](FS-config.md#3-keys-version-1)).

[§FS-config.principle](FS-config.md#principle-a-setting-written-at-a-narrower-scope-wins) and this chapter answer different questions, and neither decides a case from the other: `principle` says at which **scope** a key may be written, `concerns` says what **kind of thing** it is. Every key has both, independently, and there is no precedence rule between them for a reader to look for.

**A rule's unit is its subject's node.** A rules key is applied once per node its subject selects, and the finding names that node. That is stated once here and has two instances already written: [§FS-config.3.9.1.1](FS-config.md#3911-obligations-and-prohibitions), where an obligation is asked of each declaration of the citing kind and a prohibition fires at each citation site, and [§FS-config.3.9.2.4](FS-config.md#3924-obligations-apply-per-source-file), where the unit of a place that declares nothing is each source file in it. Neither is a rule of its own; both are this one read at the node its subject names.

The complete assignment is one row per key of the format in force. It is derived from the reader's parse sites rather than read off this prose, and it is recorded with the decision that argues it — the decision record [§DF-config-concerns](../decisions/functional/DF-config-concerns.md#df-config-concerns-a-keys-concern-is-derived-from-what-a-finding-from-it-can-be-about), in the two-stage shape [§DF-config-scope-override.2.2](../decisions/functional/DF-config-scope-override.md#22-the-inventory-is-derived-in-two-stages) already uses — so that a reader re-runs it instead of trusting it. `tests/integration/test_config_concern_inventory.py` holds that inventory to the parse sites, so a key added later cannot silently miss a concern; adding the row is part of adding the key, and that is additive surface ([§FS-config.5.1](FS-config.md#51-new-keys-are-not-a-new-version)).

Each reader has its own assignment, derived from its own parse sites. The one above is v1's. The v2 format is spelled by concern, so a v2 key's table names its concern and the envelope is the same five keys ([§FS-config-v2](FS-config-v2.md#fs-config-v2-grund-reads-a-version-2-config-by-concern-with-one-strength-vocabulary-and-fixed-defaults)), with each `[workspace.imports.<alias>]` table, which also says which projects there are ([§FS-imports.declaration.table](FS-imports.md#declarationtable-the-import-table)).

Read by concern, the two formats line up as follows, and the version-1 sections stay where they are in [§FS-config.3](FS-config.md#3-keys-version-1), the permanent version-1 chapter, so a citation that reaches one keeps reaching it ([§REQ-spec-section-names.permanence](../requirements/REQ-spec-section-names.md#permanence-a-name-is-permanent)): **Envelope** is `grund_config_version`, `project_name`, `[workspace]` in version 2 and the top-level keys of [§FS-config.3](FS-config.md#3-keys-version-1), [§FS-config.3.8](FS-config.md#38-workspace--sub-project-members) in version 1; **Schema** is `[schema]`, `[schema.sources]`, `[schema.notes.*]`, `[schema.kinds.*]` ([§FS-config-v2.schema](FS-config-v2.md#schema-the-schema-concern)) in version 2 and [§FS-config.3.1](FS-config.md#31-reference--citation-form) marker, strictness and inline-note limits, [§FS-config.3.2](FS-config.md#32-id--id-grammar), [§FS-config.3.3](FS-config.md#33-section-paths--arbitrary-nesting-depth), [§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds) less `title`, [§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned) in version 1; **Rules** is `[rules.citations.*]`, `[rules.grounding]`, `[rules.resolution]` ([§FS-config-v2.rules](FS-config-v2.md#rules-the-rules-concern)) in version 2 and [§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules), the grounding keys of [§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level) in version 1; **Presentation** is `[presentation]`, `[presentation.kinds.*]`, `[presentation.fmt]` ([§FS-config-v2.presentation](FS-config-v2.md#presentation-the-presentation-concern)) in version 2 and `project_description`, `[reference] trigger` and `conversation`, a row's `title`, [§FS-config.3.6](FS-config.md#36-output--report-format), [§FS-config.3.7](FS-config.md#37-fmtcross_refs--cross-reference-emission), [§FS-config.3.10](FS-config.md#310-fmt--suppressing-the-rewrite) in version 1.


## requirements: What the config contract holds to

Eight requirements on the key set as a whole, each carrying a **mark**: *realized* where `grund` behaves this way today and a section of this specification says so; *directional* where it binds what is written from here on rather than describing everything already written; *deferred* where a named follow-up owes it. The mark is part of the requirement — a direction is never a statement about how `grund` behaves today, and reading one as the other is the mistake this chapter exists to prevent.

There is no ninth. *Every key appears in a user-facing reference* is a property of this repository's documentation rather than of what `grund` does, and `FS` states behavior; its behavioral half — that a generated `grund.toml` carries exactly these keys, no extra keys and none missing — is already required, and more strongly, by [§FS-init.2.4.3](FS-init.md#243-every-written-key-differs-from-its-default).

### requirements.1: Every default is overridable where it has a meaning — directional

The obligation this places on a key added from here on falls on the admission decision itself rather than on any further clause at the key's own section: it should be admitted at every committed scope where its value would govern something that scope tells apart, so that a key which ends up admitted at one scope alone is so by that judgement and not for want of the question being asked. A setting admitted at more than one committed scope is overridable at each of them, and where a setting is admitted at one scope only the table section it is written under is the whole statement of that, rather than a scope clause added to its own section ([§FS-config.principle.admission](FS-config.md#principleadmission-the-relation-reaches-only-scopes-that-admit-the-setting)). This binds keys added from here on; it is not a claim about every key already written, and what it reaches today is exactly [§FS-config.principle.inventory](FS-config.md#principleinventory-the-settings-admitted-at-both-the-project-and-the-kind-scope), where a key that gains a second scope is recorded in the same change — the obligation is discharged there and not key by key. Deliberate refusals stay refusals — everything [§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree) rules out. Serves [§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable).

### requirements.2: Zero config works — realized

Every key is optional, and no config file anywhere up the walk means the canonical defaults of this specification rather than a refusal ([§FS-config.1](FS-config.md#1-file-location-and-discovery), [§FS-config.3](FS-config.md#3-keys-version-1)). Serves [§GOAL-zero-config](../goals.md#goal-zero-config-works-on-any-conformant-tree).

### requirements.3: One key means one thing at every scope — directional

A key admitted at two scopes keeps its name, its value domain, and its meaning at both; only its reach changes ([§FS-config.principle.rungs](FS-config.md#principlerungs-the-committed-scopes-are-a-relation-not-a-closed-list)). Two keys that share a spelling across scopes without sharing a setting are not an instance of this and are not made into one: `[scan]` and a row's `scan` are two settings whose reconciliation is stated where they are ([§FS-config.3.4.7.2](FS-config.md#3472-not-scanned-however-the-scan-arrives)), and `[id] format` is not `[output] format`. Directional for the same reason as [§FS-config.requirements.1](FS-config.md#requirements1-every-default-is-overridable-where-it-has-a-meaning--directional) — it binds the next key, and renaming one already written is a compatibility question of its own ([§FS-config.5.2](FS-config.md#52-every-older-version-keeps-its-meaning)).

### requirements.4: Every value has exactly one resolution — realized

The effective value of any key at any scope is a function of the committed config files and the CLI inputs alone. It never depends on the order two keys were written in, on an environment variable, or on machine state, and no two rules ever both claim one value — which is why [§FS-config.principle.exceptions](FS-config.md#principleexceptions-two-shapes-that-look-like-rungs-and-are-not) names the two shapes that are not rungs rather than leaving them implied. Serves [§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes) and [§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree).

### requirements.5: A mistake in the config fails loudly — realized, one case deferred

An unknown key and an invalid value are errors naming the line, never a silent default ([§FS-config.3](FS-config.md#3-keys-version-1), [§FS-config.4.3](FS-config.md#43-invalid-config-behavior)), per [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible). A value the key's grammar can never match is an invalid value, not a silent no-op: a `[scan] exclude` entry containing `/` names no directory component and is refused at its line ([§FS-config.3.5.16](FS-config.md#3516-an-exclude-entry-containing--is-refused)). A key written at a scope that does not admit it is refused today as an unknown key, which is loud and locatable but names the wrong mistake; saying *this key is not admitted here* instead is owed and **deferred** to its own change, and nothing in this specification depends on the current wording. A string that closes before the end of its value, `key = "one" "two"`, is an invalid value too, read by the readers before grund 0.21.0 as the one string `one" "two`: version 2 warned for it at its line through grund 0.21.x and refuses it there from 0.22.0 ([§FS-config-v2.reader.6](FS-config-v2.md#reader6-a-string-ends-at-its-first-unescaped-quote)), and version 1 keeps that reading under [§FS-config.5.2](FS-config.md#52-every-older-version-keeps-its-meaning) and meets the rule at `config migrate` ([§FS-config-migrate.3.1.8](FS-config-migrate.md#318-a-string-that-closes-early)).

### requirements.6: The effective configuration is inspectable — realized, provenance deferred

`grund config show` prints the effective configuration, and what it prints loads back to the same effective values ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)). The round trip is of the **effective** config, not of the authored text: a key the run resolved to its default prints as that default, and a key that is inert under another key's value is still parsed and still printed ([§FS-config.3.1.8](FS-config.md#318-inline_style-and-the-note-budgets)), so the output is what applies rather than what was typed. Which scope a value came from — default, project, row, or flag — is **deferred**; what the output shows today is one step of it, a row printing a key only where its effective value differs from what it inherits ([§FS-config.4.2.1](FS-config.md#421-kinds-rows-print-what-they-do-not-inherit)).

### requirements.7: An upgrade never changes the meaning of an existing config — realized

A new key, or a new scope for an existing key, is additive and does not bump `grund_config_version` ([§FS-config.5.1](FS-config.md#51-new-keys-are-not-a-new-version)); an incompatible change to the meaning of an existing key does, and a binary refuses a config whose version it does not know ([§FS-config.5](FS-config.md#5-schema-versioning)). Serves [§REQ-backwards-compatibility](../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly).

### requirements.8: A project's meaning is self-contained — realized

No config value of one project is inherited across a project boundary ([§FS-config.principle.boundary](FS-config.md#principleboundary-the-project-is-the-widest-committed-scope)) and none is read from the machine ([§FS-config.principle.install-local](FS-config.md#principleinstall-local-the-relation-governs-committed-repository-state-only)). What else reaches a run from outside the checkout is not a value: a `[workspace]` block above it whose `members` claims it spells its alias paths, so a qualified citation that passes in the checkout alone can fail inside that workspace ([§FS-workspace.6.1.5](FS-workspace.md#615-alias-paths-do-not-change-with-scope)), and fails the run with exit `2` when it cannot answer ([§FS-workspace.6.1.7](FS-workspace.md#617-a-claiming-block-that-cannot-answer-fails-the-run)); and under `respect_gitignore` the machine's global git excludes decide which files are read ([§FS-scan.scope.ignore-files](FS-scan.md#scopeignore-files-respect_gitignore--the-ignore-files)). A config that declares imports adds the tree each import names, read live at the path the config spells ([§FS-imports.mount](FS-imports.md#mount-an-import-is-a-mounted-root-read-live-and-read-only)). Serves [§REQ-runs-offline](../requirements/REQ-runs-offline.md#req-runs-offline-verification-never-depends-on-an-external-service) and [§FS-workspace.2](FS-workspace.md#2-workspace-configuration).

## 1. File location and discovery

The config file is named **`grund.toml`** and is discovered at **two locations per directory**: the bare `grund.toml` beside the project's own metadata files, then `.agents/grund.toml`. Discovery walks upward from the path argument — a file argument's parent directory (the working directory for a bare file name, [§FS-config.1.4](FS-config.md#14-a-file-argument-with-no-directory-part)), and the working directory when no path is given — probing both names in that order at every level, and stops at the first directory where either exists — mirroring how `cargo` finds `Cargo.toml`. That directory is the **config root**; relative paths inside the config are resolved against it, never against `.agents/`. One uniform rule at every level: a repository root and a workspace member each pick the form that suits them, and a workspace may mix the two ([§FS-workspace.2](FS-workspace.md#2-workspace-configuration)). Per [§DF-config-file-location](../decisions/functional/DF-config-file-location.md#df-config-file-location-grundtoml-is-discovered-at-two-names-per-directory-and-init-writes-the-bare-one).

If neither name is found anywhere up the walk, `grund` runs with the built-in defaults defined in this spec. The defaults are the canonical `grund` grammar — they are not stored in any file.

### 1.1 When one directory carries both

The bare `grund.toml` wins, and the `.agents/grund.toml` beside it is read by nothing at all. The form `grund init` generates is the form that governs ([§FS-init.2.4](FS-init.md#24-generated-grundtoml)), so a project never has to hold one rule for the file grund writes and a contradicting one for the file grund reads. It is also what a user reaching for a root `grund.toml` means: a repository acquires the pair only when someone deliberately puts a bare file beside an existing `.agents/` one, and the reason to do that is to move to the recommended form ([§FS-config.1.3](FS-config.md#13-the-agents-directory-and-the-recommended-form)).

#### 1.1.1 The pair earns one warning

Because a config `grund` ignores is still a config a user edits, `grund check` reports the pair as a warning naming both files ([§FS-check.4.3](FS-check.md#43-redundant-config-pair)), so the losing file is never silently ignored and a config quietly replaced is reported at the first `check` — which is what makes this order safe to state ([§DF-config-file-location.2.2](../decisions/functional/DF-config-file-location.md#22-the-bare-grundtoml-wins-a-tie-and-check-warns-about-the-pair)). It is a warning and not an error because the pair is the ordinary transient state of a move in either direction: warnings never affect the exit code ([§FS-check.2](FS-check.md#2-outputs)), so a repository mid-migration stays green while the finding stays visible. It is the *only* warning the pair earns: the run read the bare `grund.toml`, the location [§FS-config.1.2](FS-config.md#12-the-agents-location-is-deprecated) deprecates the other one in favour of, so nothing about the config in force is left to deprecate.

### 1.2 The `.agents/` location is deprecated

Both names keep working ([§FS-config.1](FS-config.md#1-file-location-and-discovery)), and the bare `grund.toml` is the one a project should carry, for [§FS-config.1.3](FS-config.md#13-the-agents-directory-and-the-recommended-form)'s reason. A recommendation only this specification states is one a repository never hears, so a run whose config resolved to `.agents/grund.toml` says so — once, naming the file it read and the bare `grund.toml` it should move to ([§FS-check.4.11](FS-check.md#411-config-read-from-the-deprecated-agents-location)). Nothing else about that run changes: the file is read exactly as before, every key means what it meant, and the exit code is untouched. Moving is a `git mv` with no other edit ([§DF-config-file-location.2.3](../decisions/functional/DF-config-file-location.md#23-grund-init-writes-the-bare-grundtoml)), and the config root does not move with it — relative paths already resolve against the directory, never against `.agents/` ([§FS-config.1](FS-config.md#1-file-location-and-discovery)).

**A directory carrying both names is [§FS-config.1.1](FS-config.md#11-when-one-directory-carries-both)'s case and not this one:** the config in force is already on the home path, so the directory earns the redundant pair's warning ([§FS-check.4.3](FS-check.md#43-redundant-config-pair)) and never this one. A run says either *the file you edited is ignored* or *the file you read should move*, and a repository mid-migration only ever needs one of them.

#### 1.2.1 No release removes the `.agents/` location

**Deprecated here means the tool asks you to move, not that it is going to stop reading.** There is **no release in which `.agents/grund.toml` stops being a config location**, and the message therefore names none. That is a deliberate departure from the default deprecation path ([§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)), which ships the new form beside the old with a warning naming the release the old one dies in, and it is stated here rather than left for a reader to notice the omission.

#### 1.2.2 Why no deadline is owed

A named release buys a repository the time to move before something breaks, and it is owed only where something will break. `.agents/` was `grund`'s sole config location for its whole life before dual discovery, so every repository grounded under the old rule is on it — and every one of those is a **correct** configuration rather than a broken one, because [§FS-config.1](FS-config.md#1-file-location-and-discovery) reads the two names as equals and [§DF-config-file-location.2.1](../decisions/functional/DF-config-file-location.md#21-symmetric-dual-discovery)'s one rule at every level depends on a project being free to pick the form that suits it. Naming a release would promise to break configurations nothing is wrong with, to buy a uniformity this spec does not ask for.

What the warning is actually for is narrower and needs no deadline: it stops a *new* project landing on the old path by copying an old one, at the moment the tools around `grund` are moving their own agent-facing files ([§DF-config-file-location.2.5](../decisions/functional/DF-config-file-location.md#25-the-agents-form-is-deprecated-and-never-removed)). A nudge that never expires is still a nudge; a deadline it cannot keep would be a lie.

### 1.3 The `.agents/` directory and the recommended form

`.agents/` is a single-purpose directory: it holds agent-facing tooling configuration that does not belong at the repo root next to the project's own metadata files. Other agent tools may colocate their configuration here; `grund` only owns `.agents/grund.toml`.

The bare `grund.toml` is the form `grund init` generates ([§FS-init.2.4](FS-init.md#24-generated-grundtoml)) and the default this spec recommends, because it is the form that makes a project's grounding **visible from its root listing**: `.agents/` is a dot-directory, hidden by `ls`, by editor file trees, and by shell globs, so under that form the question "is this a grund project?" has no answer a reader can see. That matters most where several grund projects are used together — a workspace root with members, or sibling checkouts side by side — where the cost is paid once per project and the one property a reader needs at a glance is the one the layout hides. A root `grund.toml` answers it the way `Cargo.toml` answers "is this a Rust crate". See [§DF-config-file-location.2.4](../decisions/functional/DF-config-file-location.md#24-a-projects-grounding-must-be-visible-from-its-root-listing).

### 1.4 A file argument with no directory part

A file argument named without a directory part — `FS-widget.md`, not `./FS-widget.md` — has the **working directory** as its parent directory, so discovery starts there and climbs exactly as it does for `./FS-widget.md`. `grund <command> NAME` and `grund <command> ./NAME` are one run: they find the same config root, are rooted at the same workspace member when the working directory lies inside one ([§FS-workspace.5.1](FS-workspace.md#51-a-member-run)), and render every path from the same base ([§FS-config.3.6](FS-config.md#36-output--report-format)). How a file argument is spelled never decides which config is loaded; a run that finds no `grund.toml` above a bare name does so only because there is none, never because the spelling cut the walk short.

## 2. Precedence

Two sources decide a setting: the committed `grund.toml` and the CLI inputs of the one run. A value written in `grund.toml` stands over the built-in default ([§FS-config.3](FS-config.md#3-keys-version-1)), and a CLI input is not a third source above both — it enters at the scope its flag spells and resolves from there ([§FS-config.principle.cli](FS-config.md#principlecli-a-cli-input-enters-at-the-scope-its-flag-spells)), which is why `grund check --require-grounding` sets the project default and an explicit `require_grounding = false` on a row still wins over it. Both statements are about one leaf key at a time ([§FS-config.principle.unit](FS-config.md#principleunit-one-leaf-key-overrides-and-one-value-is-whole)): a table naming one key leaves every other key of that table at its default, and what a key does override is its whole value, an array included. How the scopes inside the file relate is stated once in [§FS-config.principle](FS-config.md#principle-a-setting-written-at-a-narrower-scope-wins), and this section does not restate it. Per [§DF-config-scope-override](../decisions/functional/DF-config-scope-override.md#df-config-scope-override-the-committed-scopes-are-one-relation-stated-once), which records what this section said before and why it was withdrawn.

## 3. Keys (version 1)

This chapter is the version-1 format, and it stays: every key, default and meaning below is read by the v1 reader for as long as a version-1 file exists ([§FS-config.5.2](FS-config.md#52-every-older-version-keeps-its-meaning)), and every section keeps its address. The version-2 format is [§FS-config-v2](FS-config-v2.md#fs-config-v2-grund-reads-a-version-2-config-by-concern-with-one-strength-vocabulary-and-fixed-defaults), spelled by concern; [§FS-config.concerns](FS-config.md#concerns-every-key-belongs-to-exactly-one-concern) maps each concern to its v2 home and to the sections here that spell the same things. The config file is TOML. Every key is optional; omitted keys take the default value. Unknown keys are an **error**, not a warning, per [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible) — typos in config files are bugs and grund surfaces them loudly.

The recognized surface is the line-oriented subset that the keys below use: one `key = value` per line, basic (double-quoted) strings, booleans, integers, and single-line `["…", "…"]` arrays of basic strings, whose elements end only at a comma outside a string ([§FS-config.3.11](FS-config.md#311-a-list-element-ends-at-a-comma-outside-a-string)); `#` comments; `[table]` and `[[array.of.tables]]` headers. Multi-line arrays, inline `{ … }` tables, and other TOML constructs are not parsed, except for the closed `lead_size_warning` inline table defined in [§FS-config.3.1.2](FS-config.md#312-lead_size_warning--the-oversized-lead-opt-in) — keep each value on one line. A line that does not fit this shape is reported as an error pointing at the offending line, per [§FS-config.4.3](FS-config.md#43-invalid-config-behavior).

Top-level keys:

```toml
grund_config_version = 1
project_name = "Example" # optional metadata written by `grund init`
project_description = "One line describing what this project is for" # optional
```

`project_name` is free-form metadata. When the project participates in a workspace (its own config sets `[workspace]`, or its directory is listed as a member by a parent), `project_name` is also the project's workspace alias — but only when it matches the alias grammar in [§FS-workspace.1](FS-workspace.md#1-citation-syntax), and never for a member listed in `optional_members`, whose alias is the entry's last path segment and whose disagreeing `project_name` is a config error ([§FS-workspace.3](FS-workspace.md#3-aliases)). A `project_name` that is not a valid alias is not a load-time error; it errors loudly at workspace expansion with `invalid workspace project alias <name>`. Outside any workspace context `project_name` is purely metadata: no checker, scanner, formatter, or query behavior depends on it.

`project_description` is a free-form one-line description of the project, chosen in [§DF-workspace-member-descriptions](../decisions/functional/DF-workspace-member-descriptions.md#df-workspace-member-descriptions-member-side-project_description-for-workspace-member-lists). It is presentation metadata only: generated workspace member lists render it next to the project's alias ([§FS-init.2.3.4.15](FS-init.md#23415-workspace-members), [§FS-workspace.3](FS-workspace.md#3-aliases)), and no checker, scanner, formatter, or query behavior depends on it. A value containing a line break (a `\n` or `\r` escape in the TOML string) is a config error at the `project_description` line, reported per [§FS-config.4.3](FS-config.md#43-invalid-config-behavior) — the key exists to feed single-line list bullets, so a multi-line value is a bug surfaced loudly.

### 3.1 `[reference]` — citation form

The marker may contain word characters, including `_`; where a marked citation then starts is [§FS-citations.recognized.marker-start](FS-citations.md#recognizedmarker-start-the-configured-marker-establishes-the-citation-start).

```toml
[reference]
marker            = "§"      # default; rare character that prefixes a citation in prose
trigger           = "$$"     # default; typed sequence rewritten to marker by IDE plugin and `grund fmt`
strict            = true     # default; if false, bare citations are also recognized
shorthand         = "canonical" # canonical | accepted; persisted number-only citation policy
require_grounding = false    # default; if true, `check` flags source files that cite no declared ID
#                            # …and the default for every [[kinds]] row (§3.4.8)
# grounding_level = 1        # default; 1 = the file — the unit inside each governed file (§3.4.8)
# conversation    = "link"   # optional; committed conversation-rendering opinion — see below
# lead_size_warning = { max = 600, unit = "words" } # optional; oversized lead warning

# Inline citation style — see [§FS-inline-citation-style](FS-inline-citation-style.md#fs-inline-citation-style-configurable-shape-of-inline-code-comment-citations)
inline_style                 = "citation-with-note"   # default; alt: "citation-only"
inline_note_suggested_lines  = 1                       # soft cap; advisory unless warn_on_suggested = true
inline_note_max_lines        = 3                       # hard cap (error)
inline_note_max_columns      = 100                     # hard cap (error)
inline_note_layout           = "any"                   # default; alt: "citation-first-colon"
inline_note_layout_check     = "off"                   # off | warn | error — how `check` reports a layout deviation
warn_on_suggested            = false                   # if true, soft-cap overruns surface as `check` warnings
```

Per [§DF-reference-marker](../decisions/functional/DF-reference-marker.md#df-reference-marker-use--as-the-reference-marker-with--as-the-typing-trigger). `strict = true` requires a non-empty `marker`; `strict = false` is the compatibility mode for repositories that still rely on bare citations.

#### 3.1.1 `shorthand` — persisted number-only citations

`shorthand` is a closed two-value policy for a uniquely resolving, marker-origin
number-only shorthand ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)).
`canonical`, the default when the key is absent, reports a persisted shorthand
and lets `grund fmt` expand it to the full ID. `accepted` permits the shorthand
and the full citation to coexist and leaves the shorthand token byte-identical.
It does not change recognition or resolution, and it does not apply to trigger
input, unresolved or ambiguous shorthand, or a grammar without both `{number}`
and `{slug}`. The value set is closed: any other string or any non-string value
is a load-time configuration error at this key ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)). This additive key does not
bump `grund_config_version` ([§FS-config.5](FS-config.md#5-schema-versioning)).

#### 3.1.2 `lead_size_warning` — the oversized-lead opt-in

`lead_size_warning` opts this project into the lead-only warning defined by [§FS-declarations.checks.oversized-lead](FS-declarations.md#checksoversized-lead-oversized-lead-opt-in). Its inline table has exactly two required fields, separated as a list's elements are ([§FS-config.3.11](FS-config.md#311-a-list-element-ends-at-a-comma-outside-a-string)): `max`, a non-negative integer, and `unit`, one of the case-sensitive closed set `lines`, `words`, or `bytes`. Missing, extra, or duplicate fields and any other unit — including `tokens` — are load-time config errors ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)). There is no severity field: an enabled finding is always a warning and cannot change `check`'s exit status. The key is absent by default and omitted from a fresh `grund init` scaffold, so an unconfigured repository's check output stays byte-identical. `grund config show` omits the key when absent and otherwise prints its canonical inline form, `lead_size_warning = { max = <N>, unit = "<unit>" }`.

#### 3.1.3 `conversation` — one name, two scopes

`conversation` selects how agents render citations in **local conversations** — the answers, reviews, and transcripts an agent writes, not the citations on disk ([§DF-repo-conversation-opinion](../decisions/functional/DF-repo-conversation-opinion.md#df-repo-conversation-opinion-repositories-may-commit-a-link-only-conversation-rendering-opinion)). It is absent by default (no opinion), and it does not affect scanning, checking, or formatting — it only selects entrypoint guidance.

The key has **one name and two scopes**, and the scope decides both who is instructed and which values are legal:

| Where | File | Accepted | Instructs |
| --- | --- | --- | --- |
| Repository *opinion* | the project's `grund.toml` ([§FS-config.1](FS-config.md#1-file-location-and-discovery)) | `link` only | every agent that clones the repo, through the generated entrypoint ([§FS-init.2.3.6](FS-init.md#236-clickable-citations)) |
| User *preference* | `~/.config/grund/config.toml`, resolved like every `~/.config` target ([§FS-integrations.4.1.7](FS-integrations.md#417-where-a--target-resolves)) | `plain` \| `link` | every agent on this machine, through its global instruction file ([§FS-integrations.4.3](FS-integrations.md#43-user-preference-and-global-agent-instructions)) |

The same spelling in both files is deliberate: one setting the user already knows by name, read at two scopes, rather than a second vocabulary for the same idea. Only the *values* narrow, and only in the direction a repository can actually justify.

#### 3.1.4 `conversation_target` — how a link addresses its declaration

A second key, **`conversation_target`**, selects how a linked citation addresses its declaration. It is
**user-scope only** — there is no repository spelling, and setting it in the project's `grund.toml` is the
same unknown-key error as any other ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)). Its accepted values are `file` (default), `path`, `web`,
`vscode`, `vscodium`, and `cursor`; the templates each one fills, and the per-agent gate that decides
where the linked form is instructed at all, are specified in [§FS-integrations.4.3](FS-integrations.md#43-user-preference-and-global-agent-instructions) and decided in
[§DF-conversation-link-target](../decisions/functional/DF-conversation-link-target.md#df-conversation-link-target-the-conversation-link-form-is-a-markdown-link-over-an-absolute-uri-addressed-per-machine). The key is inert unless the effective `conversation` is
`link`; it is still parsed and reported either way, like the `inline_note_*` keys ([§FS-config.3.1.8](FS-config.md#318-inline_style-and-the-note-budgets), [§FS-config.3.1.9](FS-config.md#319-inline_note_layout-and-inline_note_layout_check)). One machine
may read several agents that do not render alike, so the same key is also accepted per agent under
`[reference.agents.<agent>]`, a partial merged over the machine-wide value ([§FS-integrations.4.4](FS-integrations.md#44-per-agent-overrides)).

#### 3.1.5 What `link` commits, and when to set it

`link` makes the declaration's location travel with the citation; the committed form depends on the entrypoint's agent — a Markdown link over the machine-independent `file` target in the entrypoints of the agents the gate clears for it (Claude's and Pi's), plain `path:line` text in every other ([§DF-conversation-link-target.2.4](../decisions/functional/DF-conversation-link-target.md#24-the-form-is-gated-per-agent-and-the-fallback-is-path)) — and the reader's own `conversation_target` may override it ([§FS-init.2.3.4.17](FS-init.md#23417-clickable-citations), [§DF-conversation-link-target.2.3](../decisions/functional/DF-conversation-link-target.md#23-the-target-is-user-scoped-but-the-default-is-committable)).

**Set the repository key to `link`** when the citations your agents write should carry their declaration location for readers whose machines grund never touched — teammates on a fresh clone, cloud agent sessions, CI reviewers, and Cursor or Windsurf users, who have no user-level file grund can write. It costs installed users nothing: their recorded `plain` still wins ([§FS-integrations.4.3](FS-integrations.md#43-user-preference-and-global-agent-instructions), [§DF-repo-conversation-opinion.2.3](../decisions/functional/DF-repo-conversation-opinion.md#23-precedence)). **Leave it absent** when local-conversation rendering is each contributor's own business — then the user preference governs alone, and a machine that never stated one gets today's bare citations.

#### 3.1.6 `plain` is deliberately not a repository value

`plain` presumes an installed rendering layer, which is machine state a repository cannot know; committing it would break exactly the clones the key exists to serve ([§DF-repo-conversation-opinion.2.2](../decisions/functional/DF-repo-conversation-opinion.md#22-only-link-is-committable)). The repository value set is therefore a closed enum with the single member `link`, widenable later without a `grund_config_version` bump ([§FS-config.5](FS-config.md#5-schema-versioning)); any other value — including `plain` — is a load-time error ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)), `grund init` included.

#### 3.1.7 `require_grounding` and `grounding_level` — defaults for `[[kinds]]`

`require_grounding = true` adds the ungrounded-source-file error ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)), which says which files must be grounded and how. `grund check --require-grounding` sets the same default for one run, and an explicit `require_grounding = false` on a row wins over it ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)). Per [§DF-require-grounding](../decisions/functional/DF-require-grounding.md#df-require-grounding-an-opt-in-check-that-every-source-file-cites-a-spec); off by default so adopting the discipline is a deliberate step, like `strict`.

`require_grounding` and `grounding_level` are the two keys of this section admitted at both the project scope and the kind scope: each may also be written on a `[[kinds]]` row, and the row is the narrower scope ([§FS-config.principle.rungs](FS-config.md#principlerungs-the-committed-scopes-are-a-relation-not-a-closed-list), [§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)). Written here they say what every place does; written on a row they say what one place does. They are an instance of that relation rather than an exception to this table: every other key of this section is admitted at the project scope alone, which is what [§FS-config.principle.admission](FS-config.md#principleadmission-the-relation-reaches-only-scopes-that-admit-the-setting) says of a key whose section names no second scope, and [§FS-config.principle.inventory](FS-config.md#principleinventory-the-settings-admitted-at-both-the-project-and-the-kind-scope) carries the complete list. `grounding_level` names the unit inside each governed file — `1`, the default, is the file, which is the unit every config had before the key existed. It is inert, and a config error, where nothing turns grounding on ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)).

#### 3.1.8 `inline_style` and the note budgets

`inline_style`, the three budget keys (`inline_note_suggested_lines`, `inline_note_max_lines`, `inline_note_max_columns`), and `warn_on_suggested` govern the shape of inline citations in code comments — whether a `§<ID>` token may be accompanied by a short rationale, and how long that rationale may run. The budgets and the style bound *inline* comments only; a doc-comment is documentation and lies outside all of them, so a citation inside one is checked for everything except its shape ([§FS-inline-citation-style.1.1](FS-inline-citation-style.md#11-doc-comments-are-not-sites)). The full contract — modes, enforcement, agent-facing rendering — lives in [§FS-inline-citation-style](FS-inline-citation-style.md#fs-inline-citation-style-configurable-shape-of-inline-code-comment-citations). `inline_style` is a closed enum: `citation-with-note` (default) or `citation-only`. An unrecognized value is a load-time error ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)) whose message names the rejected value in backticks and the accepted set in parentheses, the form the layout enums of [§FS-config.3.1.9](FS-config.md#319-inline_note_layout-and-inline_note_layout_check) use — ``unknown [reference] inline_style `note` (expected citation-with-note or citation-only)`` — so the reader is pointed at the value rather than at a key that exists. The value and the set are appended after the message's earlier text, which survives as its prefix, so this is an append under [§FS-errors.3](FS-errors.md#3-message-text) and not a change of wording. Load-time invariant: `inline_note_suggested_lines ≤ inline_note_max_lines`, and `warn_on_suggested` is a boolean; any other value is a load-time error ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)). Under `inline_style = "citation-only"` the three budget keys are inert (no note is ever permitted), but they are still parsed and printed by `grund config show` — the file is the canonical machine-readable form.

#### 3.1.9 `inline_note_layout` and `inline_note_layout_check`

`inline_note_layout` adds the third axis of that shape — where the `§<ID>` tokens sit inside the note — and `inline_note_layout_check` selects whether `grund check` reports a deviation and through which channel. Both are closed enums: `any` (default, no constraint) or `citation-first-colon` for the layout, and `off` (default), `warn`, or `error` for the check; an unrecognized value is a load-time error ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)), and either set may be widened later without a `grund_config_version` bump ([§FS-config.5](FS-config.md#5-schema-versioning)). The layout key is the house style and the check key is the gate, so a project can publish the style to its agents ([§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)) before it starts failing on it. `inline_note_layout_check` is inert under `inline_note_layout = "any"` and both are inert under `inline_style = "citation-only"` — still parsed, still printed, like the budgets ([§FS-config.3.1.8](FS-config.md#318-inline_style-and-the-note-budgets)). The canonical form and the per-line rule live in [§FS-inline-citation-style.3.3](FS-inline-citation-style.md#33-inline_note_layout--where-the-citations-sit).

### 3.2 `[id]` — ID grammar

```toml
[id]
format             = "{kind}-{number}-{slug}"
section_separator  = "."
section_heading_levels = "strict"
named_sections     = false
number_pattern     = "\\d+"
slug_pattern       = "[a-z0-9][a-z0-9-]*"
```

`format` is a template: `{kind}`, `{number}`, `{slug}` are placeholders; everything else is literal. `{kind}` is required. `{number}` and `{slug}` are individually optional — but **at least one** of them must appear, because a bare kind would not identify a declaration. The literal characters between placeholders may be anything — `-`, `_`, `.`, `:`, etc.

The chosen format is the repository default. A citable `[[kinds]]` row may set
its own `format` ([§FS-config.3.4.10](FS-config.md#3410-format-resolve-and-fetch--external-snapshot-kinds)), which is authoritative for IDs of that kind. Every
consumer selects the kind from the token first and then applies that kind's
grammar, so ordinary slug IDs and numeric ticket IDs may coexist without
ambiguity. Kinds without an override retain the `[id].format` grammar exactly.

#### 3.2.1 The three canonical shapes

| `format`                       | Example ID            | Disambiguator                |
|--------------------------------|-----------------------|------------------------------|
| `"{kind}-{number}-{slug}"`     | `FS-NNN-<slug>`       | number; slug is descriptive  |
| `"{kind}-{number}"`            | `FS-NNN`              | number                       |
| `"{kind}-{slug}"`              | `FS-<slug>`           | slug must be unique per kind |

When `{number}` is omitted, slugs must be unique within each kind — two declarations sharing a kind and slug collide on the same ID and are reported as duplicate declarations (per [§FS-check.3](FS-check.md#3-errors-detected)). When `{number}` is present, slugs are descriptive only and may repeat across declarations with different numbers.

#### 3.2.2 `section_separator` must stay distinguishable

`section_separator` must not collide lexically with any literal in `format` or with `slug_pattern`. grund validates this on load and refuses ambiguous configs. It must not be — or contain — a `/` either, which is [§FS-config.3.2.3](FS-config.md#323-no-id-contains-a-)'s invariant seen from the other side: a citation is `[<alias path>/]<ID>[<sep><section>]` and its alias-path boundary is the **last** `/`, so a `/` separator makes the two boundaries the same character. With `section_separator = "/"`, `<§>root/fs-x/1` — section 1 of `fs-x` in project `root` — reads as alias path `root/fs-x` and ID `1`: a citation that resolved before alias *paths* existed stops resolving, and a `[citations]` obligation ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)) resting on it turns red with no config change. Rejected at that key's line.

#### 3.2.3 No ID contains a `/`

No ID the grammar can build may contain a `/`, and what that forbids depends on how the key reaches an ID. `format` and a `[[kinds]]` `kind` name ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)) contribute literal text — a citable kind's name is the leading component of every ID in it — so neither may carry the character: a `/` in the key is a `/` in the ID. `number_pattern` and `slug_pattern` are regexes, and the rule asks what they **match**, not what they spell. A pattern with no `/` in its text may produce one freely (`[^.[:space:]]+`, `.+`, `[^[:space:]]+`) and is rejected; a pattern that names the character to *exclude* it (`[^/.]+`) can never produce one and loads. A `/` belongs to the alias path and never to an ID — a qualified citation splits on its **last** `/`, and every command that takes an `<alias>/<ID>` argument splits it the same way ([§FS-workspace.1](FS-workspace.md#1-citation-syntax)). So a grammar permitting `FS-a/b` would declare and resolve an ID that grund cannot accept back as a query, and would make the alias-path boundary depend on which project's grammar the reader had in mind. Rejected on load at the offending key's line, like the regex check of [§FS-config.3.2.4](FS-config.md#324-each-pattern-compiles-on-its-own); the message says *must not contain* for a literal key and *must not match* for a pattern, since only one of those is a question about the key's text.

#### 3.2.4 Each pattern compiles on its own

`number_pattern` and `slug_pattern` must each be a valid regex **on their own**, not merely valid once spliced into the ID pattern. Two that balance only against each other — `number_pattern = "("` with `slug_pattern = "a)"` — would compile as one ID pattern and then fall apart the moment grund derives a narrower pattern from a subset of the format's components, which is what the number-only shorthand does ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)). Such a config is rejected on load with the underlying regex error, rather than accepted and failed later.

#### 3.2.5 Off-grammar declarations stay readable

The effective format is the **authoring and conformance grammar**, not a reason
to make a persisted declaration unreadable. A heading in declaration position
whose exact token starts with a configured citable kind, ends at the
declaration colon, and carries no `/` ([§FS-config.3.2.3](FS-config.md#323-no-id-contains-a-)) is retained in that project's catalog
even when the token does not match the kind's effective format. Its exact written spelling, body,
sections, and location remain available to readers, and it earns the
`declaration-near-miss` finding ([§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss)). Exact marker-prefixed candidates
that normal grammar rejects become citations only when that same project's
catalog contains an exact declaration spelling; an unmarked candidate or a
marked candidate with no exact declaration gains no compatibility meaning. Such a marked candidate
still cites nothing, but where it opens with a configured kind and no other reading claims its
marker, `check` reports it as an off-format citation ([§FS-citations.recognized.off-format](FS-citations.md#recognizedoff-format-a-marked-token-its-kinds-format-does-not-spell-is-reported-never-dropped)). This catalog-backed boundary is decided in
[§DF-off-grammar-declaration-compatibility](../decisions/functional/DF-off-grammar-declaration-compatibility.md#df-off-grammar-declaration-compatibility-persisted-declarations-remain-readable-without-relaxing-the-authoring-grammar).

#### 3.2.6 Resolving an off-grammar citation

Configured full IDs keep their existing precedence. Otherwise an exact
off-grammar declaration and number-only shorthand are considered together:
zero targets preserves the current invalid-ID result, one target resolves, and
multiple targets fail as ambiguous. For a possible inline section, an exact
whole-token declaration wins; otherwise every catalog prefix followed by the
configured section separator is considered, and competing valid
interpretations fail. Duplicate exact declarations retain the ordinary
duplicate/ambiguity behavior and sorted sites. None of these read rules changes
`grund id`, `init`, `fetch`, config validation, `fmt --marker`, JSON schemas, or
the forms those authoring surfaces create.

#### 3.2.7 `named_sections` — the gate for explicit section names

`named_sections` is an absent-by-default Boolean gate for explicit section names. When absent or `false`, section scanning, citation recognition, queries, formatting, completion, LSP behavior, and operational output remain the numeric-only behavior of earlier configurations. When `true`, a named component has the fixed, configuration-independent grammar `[a-z][a-z0-9-]*`; it is not derived from `slug_pattern` or from the displayed heading title. `grund init` writes the teaching default `named_sections = false` ([§FS-init.2.4](FS-init.md#24-generated-grundtoml)). Unknown values are invalid config. A relative heading whose one component is a name needs the gate as a complete-path heading carrying that name does, and one whose component is a number needs nothing ([§FS-declarations.section.relative.form](FS-declarations.md#sectionrelativeform-the-form)).

#### 3.2.8 Numeric spelling survives reading and reporting

For an ID accepted by the effective grammar, the authored digits of `{number}` are its source spelling. Reading and reporting preserve that spelling, including leading zeroes, under both the repository default and a per-kind override ([§FS-config.3.4.10.1](FS-config.md#34101-format)); the version-2 spellings of those settings have the same behavior ([§FS-config-v2.schema.1](FS-config-v2.md#schema1-schema)). With `{kind}-{number}`, declarations `ADR-0001` and `ADR-0002` therefore appear with those IDs in text reports and machine-readable ID strings. A diagnostic for an unresolved citation `ADR-0007` names `ADR-0007`, and a suggestion of the first declaration names `ADR-0001`. Visible citation text retains its own spelling; it does not rename the target declaration. The target declaration's source spelling is part of the heading text a generated link anchors on ([§FS-fmt.6.2.2](FS-fmt.md#622-the-anchor-slugifies-the-headings-rendered-text)), independently of an equivalent citation's spelling: `# ADR-0001: Use Postgres` and `# ADR-0002: Use Redis` yield `#adr-0001-use-postgres` and `#adr-0002-use-redis`, even when cited as `ADR-1` or `ADR-002`. A section headed `## 1. Why` still yields `#1-why`.

Spelling is distinct from numeric identity: `ADR-1`, `ADR-001` and `ADR-0001` still select the same kind and number, and declaring two of them is a duplicate under the existing collision rules ([§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration)). Numeric fields remain numbers. Lookup and collisions use numeric identity; ordering and next-number derivation retain their existing rules. Allocation alone chooses a minimum width ([§FS-id.1.2](FS-id.md#12-number-width)); reading a four-digit declaration does not change the default or explicit allocation width, nor repad existing headings, filenames or citations ([§DF-id-number-width.4](../decisions/functional/DF-id-number-width.md#4-what-happens-at-the-rollover)).

### 3.3 Section paths — arbitrary nesting depth

Section coordinates are **dotted paths of arbitrary depth**. There is no maximum nesting depth. Purely numeric paths retain their existing heading form and behavior:

```
§FS-check.3
§FS-check.3.1
§FS-check.3.1.2
§FS-check.3.1.2.7.4
```

Section depth in the citation must match a heading at that depth in the declaration. The scanner records every citable heading inside a declaration body and validates citations against the recorded set, so a project that wants four-deep numeric nesting (`## 1.`, `### 1.1`, `#### 1.1.1`, `##### 1.1.1.1`) is supported with no config changes — the dotted path simply grows.

#### 3.3.1 Named components

With `named_sections = true` ([§FS-config.3.2.7](FS-config.md#327-named_sections--the-gate-for-explicit-section-names)), any path containing a named component uses the explicit colon form `<path>: <title>`. A heading writes its complete path, or, in a Markdown declaration body, a relative path: one `.` and one component, the rest supplied by its parent ([§FS-declarations.section.relative](FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds)). Only the complete path is citable. The title declares nothing and may change without changing the coordinate:

```markdown
## goals: Goals
### goals.performance: Performance
### goals.3: Third ordered goal
```

Named components may occur at every depth, under a name or under a number, so all-name paths such as `goals.performance.latency` are legal, and so are `1.goals` and `1.goals.latency` beneath a numbered chapter. A numeric component may follow a named prefix (`goals.3`), knowingly retaining positional behavior beneath that stable name. A path in which a named component follows a numeric one, at any later depth, is a **mixed path** (`1.goals`, `1.goals.3`, `goals.3.latency`); it retains positional behavior beneath its number in the same way, so renumbering section `1` moves `1.goals` to `2.goals`. Once cited, a mixed address is recorded in the section lock, so such a move is reported rather than silently retargeting the citation ([§FS-section-lock.holds.entry](FS-section-lock.md#holdsentry-one-line-per-locked-address)). A mixed path is one address: `1.goals` is never an alias for section `1`. Decided in [§DF-number-name-sections](../decisions/functional/DF-number-name-sections.md#df-number-name-sections-a-name-may-follow-a-number-1goals-and-a-cited-address-with-one-is-locked). Purely numeric headings keep their existing optional trailing period and title syntax; the colon form is mandatory whenever any component is named. There is no inferred title slug, dual numeric/name address, rename map, or automatic migration.

#### 3.3.2 `section_heading_levels` — heading depth against path depth

`section_heading_levels` controls how the Markdown heading depth must line up with the dotted section path. The default, `"strict"`, requires the heading depth to equal the declaration heading depth plus the number of path components, named or numeric, so under an H1 declaration `## 1.1 …` and `## goals.performance: …` are `section heading level mismatch` errors in `grund check` ([§FS-declarations.checks.section-heading-level](FS-declarations.md#checkssection-heading-level-section-heading-level-mismatch)). `"warn"` reports the same mismatch as a warning, so CI can stay green while a repo migrates. `"loose"` preserves the historical depth behavior: any deeper heading can declare a syntactically legal complete path. Unknown values are invalid config. Depth is judged on the canonical path: a relative heading sits one level below its parent by construction, so it has no depth finding of its own, and Markdown's six levels bound how far relative headings nest, not how long a path is ([§FS-declarations.section.relative.parent](FS-declarations.md#sectionrelativeparent-the-parent)).

The mode does not govern an ATX heading with no coordinate: inside a Markdown declaration body, every deeper ATX heading must instead be a declaration or a recognized numeric or enabled named section. [§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading) reports one that is neither and names the headings the rule leaves alone; there is no severity or opt-out key for this project-wide rule ([§DF-unmarked-markdown-headings](../decisions/functional/DF-unmarked-markdown-headings.md#df-unmarked-markdown-headings-in-body-markdown-atx-headings-participate-in-the-knowledge-graph)).

#### 3.3.3 Every prefix of a name-bearing path is recorded

Every proper prefix of a name-bearing path must itself be recorded in the same declaration. Thus `### goals.performance: …` requires `goals`, while `### missing.performance: …` is an orphan even if its Markdown placement looks nested under some other heading ([§FS-declarations.checks.orphan-section](FS-declarations.md#checksorphan-section-orphan-name-bearing-section-path)). The invariant applies to name-bearing paths only; purely numeric paths preserve their historical behavior. A relative heading always needs its actual parent, numeric or named ([§FS-declarations.section.relative.parent](FS-declarations.md#sectionrelativeparent-the-parent)).

#### 3.3.4 The outer section separator

The default `section_separator` is `.`. Projects that prefer `:` (`<§>FS-check:3.1.2`) or `#` (`RFC-42#3.1.2`) override it; the dotted **components** stay separated by `.` regardless of the outer separator. This split keeps the section grammar regular at any depth. The leading `.` of a relative heading ([§FS-declarations.section.relative.form](FS-declarations.md#sectionrelativeform-the-form)) is a component separator, so it is `.` whatever `section_separator` says.

### 3.4 `[[kinds]]` — recognized kinds

One `[[kinds]]` table per kind. `kind` is its name — mandatory, and what everything else keys on: `[citations.<kind>]` ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)), `grund list --kind`, and, for a kind that declares IDs, the literal prefix of every ID in it.

A kind is either *multi-file* (`folder = "<dir>"`) — each declaration is the H1 of its own file under `<dir>` — or *single-file* (`file = "<path>"`) — every declaration of the kind is a heading inside that one document — an H2 by convention, and the H1 where the file holds the kind's single declaration ([§FS-config.3.4.4](FS-config.md#344-the-default-kinds)). Setting both `folder` and `file` on the same kind is invalid; setting neither leaves the kind with no configured home. What a home means to `grund id` and to the checker is [§FS-config.3.4.11](FS-config.md#3411-what-a-home-is-used-for).

Every configured home is also **in the scan scope by construction**, whether or not `[scan] include` names it ([§FS-config.3.5.8](FS-config.md#358-every-configured-kind-home-is-scanned)), except one marked `scan = false`, which is listed and not scanned ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned)).

#### 3.4.1 `citable` — kinds that declare no IDs

A kind has two independent properties: it *has a home*, and it *declares IDs*. `citable = false` (default `true`) is the second one turned off — a kind that is a **place** and nothing more.

```toml
[[kinds]]
kind = "skill"
folder = "skills"
title = "Agent review and automation skills"
citable = false

[citations.skill]
must = ["FS"]
must-not = ["AR"]
```

Some directories hold agent-facing content rather than specification — skills, prompt libraries, runbooks, test suites. An agent has to be told they exist and what they are for, and the citations inside them should be checked and directed like citations anywhere else; but their files are not declarations and carry no IDs. *Citable* is already this spec's word for "can be the target of a `§` citation" — citable sections, citable IDs — and this key says it of a whole kind.

`citable` is additive and does not move `grund_config_version` ([§FS-config.5](FS-config.md#5-schema-versioning)). A non-citable kind whose files are not this repository's to read — content that ships verbatim — sets `scan = false` as well ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned)).

##### 3.4.1.1 What a non-citable kind keeps

- **A home** — `folder` or `file` — wherever the kind is a *place*. Leaving both out is not an omission but a different thing: the entry becomes the **homeless kind**, the complement of every home, whose default name is `code` ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)). Everything below is written about a non-citable kind with a home; [§FS-config.3.9.2](FS-config.md#392-the-homeless-kind) says where the homeless one differs.
- **A row in the generated Project map and in the generated citation directions** ([§FS-init.2.3.4.4](FS-init.md#2344-project-map), [§FS-init.2.3.5](FS-init.md#235-citation-directions)) — rendered by **place**, never by name, because the name is a config key and the place is the thing a reader can open.
- **Citation-direction rules.** The citing-side classification already reaches it: a citation inside a kind's home, when exactly one home contains its file, is classified as that kind even where no declaration encloses it ([§FS-citations.citing-kind](FS-citations.md#citing-kind-which-kind-a-citation-site-counts-as)). Obligations attach per file rather than per declaration ([§FS-check.3.11](FS-check.md#311-missing-required-citation)), since there is no declaration to attach them to.
- **Grounding**, over every scanned file in its home, `.md` included ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)) — asked of this home alone with `require_grounding` on the row, or of every place at once with the `[reference]` default the row inherits ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)).

##### 3.4.1.2 What it loses

- **The ID grammar.** Its name is not a recognized prefix, so `<name>-<slug>` is not an ID and never tokenizes as a citation. It is left out of the `KIND ∈ {…}` vocabulary line, out of `grund list --kind`, and out of `grund id` — both selectors refuse it by name, saying that it declares no IDs rather than that it is unknown ([§FS-list.1](FS-list.md#1-inputs), [§FS-id.1](FS-id.md#1-inputs)).
- **Declarations.** Its home admits none: a declaration inside it, where it is the file's only home, is a misplaced declaration ([§FS-declarations.checks.misplaced-declaration](FS-declarations.md#checksmisplaced-declaration-misplaced-declaration-configured-kind-home)).
- **An index.** `index` lists a folder's declarations ([§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)) and this kind has none, so setting both keys is a config error rather than a no-op — a statement about a set that can never be non-empty.
- **Being cited.** A `[citations.<kind>]` rule may not *name it as a target*; there is no ID to point at ([§FS-config.3.9.5](FS-config.md#395-validation)).

#### 3.4.2 `index` — the kind's index file

`index` names the kind's **index file** — the document under `folder` that must list every declaration in it ([§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)). It is resolved relative to `folder`, defaults to `README.md`, and takes either a file name or `false`:

```toml
[[kinds]]
kind = "DF"
folder = "docs/decisions/functional"
# index = "README.md"   # the default, relative to `folder`
# index = false         # opt out — the folder is not navigated
# index = "INDEX.md"    # or name a different file
```

The key is additive and does not move `grund_config_version` ([§FS-config.5](FS-config.md#5-schema-versioning)).

##### 3.4.2.1 The default follows `folder`

The default follows from `folder` rather than from a second key restating it: a kind whose declarations live in a directory has a directory a reader arrives at, and `grund init --scaffold` already scaffolds that README and writes the convention into it ([§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place)). `index = false` is for a kind whose declarations are *exercised* rather than navigated — the canonical case is a repository's own `E2E` kind, whose home holds case directories and no README, and whose `e2e/README.md` one level up documents the case layout in English instead of naming `E2E-` IDs. That is why the opt-out spells a file name or `false` rather than being inferred from a README's absence: "any folder README is an index" is not true of every tree.

##### 3.4.2.2 Where `index` is valid

`index` requires `folder`, and requires a citable kind. On a single-file kind (`file = "<path>"`), on a kind with no configured home, or on a `citable = false` kind ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)) there is nothing to index, and the key is a config error reported per [§FS-config.4.3](FS-config.md#43-invalid-config-behavior). `index = true` is an error for the same reason a bare `true` names no file — write the name, or leave the key out for the default.

##### 3.4.2.3 A named index is a Markdown file inside `folder`

A named `index` must be **a relative path inside `folder`, naming a Markdown file**; anything else is a config error per [§FS-config.4.3](FS-config.md#43-invalid-config-behavior). Both halves close a state the rules built on the key cannot describe. The value is joined onto `folder`, so an absolute path or one that climbs out with `..` does not name a file *in* the folder — it silently replaces the folder, and `grund check` would read a file outside the tree the config describes — the same boundary [§FS-fmt.2.3.2](FS-fmt.md#232-a-link-that-leaves-the-config-root-is-not-written-through) holds a rewrite to, for the reason [§REQ-no-data-loss.2](../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own) gives; `.` is refused with them, because it names the same file by a path no message should have to print. And an entry has to be a Markdown link that `grund fmt --write` can write ([§FS-check.3.17](FS-check.md#317-index-entry-is-not-a-link)), while the cross-reference pass runs on `.md` files only ([§FS-fmt.6.1](FS-fmt.md#61-scope)) — so an index named `INDEX.rst` would carry an error class whose one documented fix declines to act on it.

##### 3.4.2.4 The default is per kind name

It is the same default for a declared kind and a built-in one. `E2E` defaults to `index = false` and every other citable folder kind to `README.md`, whether the name comes from the built-in list or from a `[[kinds]]` block that omits the key. A `[[kinds]]` block replaces the built-in list rather than merging into it ([§FS-config.3.4.4](FS-config.md#344-the-default-kinds)), so without this the generated config would mean one thing when it spells `index = false` out and another when it does not — and every config written before this key existed, which is every config on disk, would inherit an obligation the built-in default deliberately declines. `E2E` keeps its entry in that table after leaving the default kind set ([§FS-config.3.4.4](FS-config.md#344-the-default-kinds)) for exactly the same reason: the configs that name it are the ones written before it left. A project that names its cases folder `E2E` *and* wants an index writes `index = "README.md"`, which is the ordinary way to override a default.

The name-keyed default is additive like the key ([§FS-config.3.4.2](FS-config.md#342-index--the-kinds-index-file)) — it can only *remove* an obligation that no released `grund` has ever imposed.

#### 3.4.3 `title`

`title` is human-readable metadata: it surfaces in `grund list --summary --format json` ([§FS-list.3.3](FS-list.md#33---summary)), the one JSON output that carries it, and in IDE hover previews, and is **not** injected into `grund <ID> --format=md` text (which is the declaration verbatim — [§FS-show.3.1](FS-show.md#31-format-variants)). It is also the text of the kind's Project map row ([§FS-init.2.3.4.4](FS-init.md#2344-project-map)), which for a non-citable kind is the only thing that says what the place is for.

The resolved target kind owns this metadata, including ordinary configuration defaults,
qualified workspace queries and each batch query's selection. For declaration and
section show objects, a present title, even an empty string, adds `kind_title`
immediately before the terminal `path`, `line` pair. The distinct E2E manifest keeps
its `id`, `kind`, `path` prefix and appends `kind_title` as its final field. Detailed
refs citation objects also append it as their final field. An absent effective title
omits the field; it is neither `null` nor a fallback kind name. Refs summaries keep
their aggregate shape. Hover appends a separate literal Kind paragraph as specified
by [§FS-lsp.1.2](FS-lsp.md#12-hover-preview); authored preview content and CLI Markdown
remain unchanged. The bounded compatibility choice and its installed-resolver ordering
correction are recorded in
[§DF-configured-title-metadata](../decisions/functional/DF-configured-title-metadata.md#df-configured-title-metadata-kind-titles-are-separate-target-metadata).

#### 3.4.4 The default kinds

The defaults declare these nine, in this order, for a project with no config and for a `grund.toml` that omits `[[kinds]]` alike (one that writes `kinds = []` gets none, [§FS-config.3.4.4.5](FS-config.md#3445-kinds---declares-no-kinds)):

```toml
[[kinds]]
kind   = "GRUND"
file   = "docs/grund.md"
title  = "Why: project motivation"

[[kinds]]
kind   = "GOAL"
file   = "docs/goals.md"
title  = "Where: project direction and outcomes"

[[kinds]]
kind   = "FS"
folder = "docs/functional-spec"
title  = "What: behavior, requirements, and constraints"

[[kinds]]
kind   = "AR"
folder = "docs/architecture"
title  = "How: high-level implementation, structure, and design"

[[kinds]]
kind   = "DF"
folder = "docs/decisions/functional"
title  = "Product behavior decisions and tradeoffs"

[[kinds]]
kind   = "DA"
folder = "docs/decisions/architectural"
title  = "Architecture decisions and tradeoffs"

[[kinds]]
kind    = "e2e"
folder  = "tests/e2e"
citable = false
title   = "User scenarios: black-box proof of the spec"

[[kinds]]
kind    = "integration"
folder  = "tests/integration"
citable = false
title   = "Integration tests: proof that the parts fit as designed"

[[kinds]]
kind   = "RM"
file   = "docs/roadmap.md"
title  = "Planned milestones and sequencing"
```

A project that overrides this list replaces the defaults entirely — there is no merge. To extend rather than replace, copy the defaults and add to them. The list has no `REQ`: the `software-development` template declares one, with no built-in default behind it ([§FS-init.2.4.10](FS-init.md#2410-req-is-declared-in-docsrequirements)).

The `FS` row is the home grund 0.19.0 moved it to from `file = "requirements.md"`, after 0.18.0 warned every project the move touched ([§DF-fs-home-lands-in-0-19](../decisions/functional/DF-fs-home-lands-in-0-19.md#df-fs-home-lands-in-0-19-the-built-in-fs-home-is-docsfunctional-spec-from-0190)); the `software-development` template's `FS` row names the same folder ([§FS-init.2.4.10](FS-init.md#2410-req-is-declared-in-docsrequirements)).

##### 3.4.4.1 Where each citable default declares

`GRUND` is the H1 of the single file `docs/grund.md` (the project's reason for being — one declaration, all of it inline); `GOAL` declarations are H2 headings inside the single file `docs/goals.md` (one file, all goals inline); `RM` declarations are likewise H2 headings inside the single file `docs/roadmap.md` (one file, all milestones inline) — those three are single-file kinds (`file = "<path>"`); `FS`, `AR`, `DF`, and `DA` declarations are the H1 of a file in their `folder` (an `AR` declaration may instead live inline in a source doc-comment with an optional stub in `folder` — [AR-scanner.4](../architecture/AR-scanner.md#4-inline-declarations-in-language-doc-comments)). A single-file kind can always be broken up later by swapping `file = "<path>"` for `folder = "<dir>"` and moving the document into that folder — this specification models the transition as exchanging one key for the other, not setting both.

##### 3.4.4.2 The two test kinds are non-citable, and lowercase

A test cites the document whose claim it proves, and is never cited back: an `e2e` scenario proves the What as a user sees it (`must` cite `FS`, and `should-not` cite `AR` — a black-box scenario that reads the design is not black-box), an `integration` test proves the How — that the parts fit as designed (`should` cite `AR`). Unit tests live with the code and follow `code`'s rule, so there is no third kind for them. Lowercase because these names never appear in an ID, so a reader should not mistake one for a prefix; the `KIND ∈ {…}` vocabulary line lists the citable seven.

##### 3.4.4.3 `E2E` is configured, not a default

`E2E` is **not** in this list, and is still a fully supported kind: a repository whose e2e suite is a corpus of case directories declares it (`kind = "E2E"`, `folder = "e2e/cases"`, `index = false`) and gets the case-declaration machinery of [§FS-declarations.e2e-case](FS-declarations.md#e2e-case-what-an-e2e-case-directory-declares) — `E2E-<case>` IDs, `grund <ID>` over the case manifest, per-case obligations, and the fixture-tree pruning that keeps a nested case repo out of the host scan. That machinery follows the configured `E2E` home, so a config that wants it names it. Decided in [§DF-non-citable-kinds.3](../decisions/functional/DF-non-citable-kinds.md#3-consequences), which also records what a default-config repository with an `e2e/cases` tree sees on upgrade.

##### 3.4.4.4 A config that omits `[[kinds]]` keeps the older `FS` home

Retired in grund 0.19.0: a `grund.toml` that omits `[[kinds]]` kept `folder = "docs/functional-spec"` as its `FS` home while a project with no config took `file = "requirements.md"`. The built-in `FS` row is now that folder ([§FS-config.3.4.4](FS-config.md#344-the-default-kinds)), so both get the same rows and there is no special case left ([§DF-fs-home-lands-in-0-19](../decisions/functional/DF-fs-home-lands-in-0-19.md#df-fs-home-lands-in-0-19-the-built-in-fs-home-is-docsfunctional-spec-from-0190)). This address is kept so citations written before the change still resolve.

##### 3.4.4.5 `kinds = []` declares no kinds

A v1 config that writes the top-level key `kinds = []` declares an empty kind set: no row, no ID namespace, no home, and none of the defaults above, where omitting `[[kinds]]` keeps them ([§FS-config.3.4.4](FS-config.md#344-the-default-kinds)). It is how a repository that declares its own kinds starts ([§FS-init.1.6.2](FS-init.md#162-none)), and the one kind set with no citable kind that v1 loads: a `[[kinds]]` table whose every row sets `citable = false` is still refused. Only the empty array is accepted. A non-empty one is refused at its line with the instruction to write `[[kinds]]` tables, and a `[[kinds]]` table after `kinds = []` is refused at its header, as TOML refuses a table appended to a static array. A version-2 file needs no key for this: one with no `[schema.kinds]` row already has no kinds ([§FS-config-v2.defaults.1](FS-config-v2.md#defaults1-what-v2-fixes)). What follows is each command's to say: the scan reads `[scan] include` and nothing else, because there is no home to add to it ([§FS-scan.scope.homes](FS-scan.md#scopehomes-every-configured-kind-home-is-scanned)); no marked token is a citation, so `check` reports none ([§FS-citations.recognized.no-kinds](FS-citations.md#recognizedno-kinds-with-no-kind-no-token-is-a-citation)); `grund id` has no kind to allocate in ([§FS-list.1.1.1](FS-list.md#111-a-scope-with-no-citable-kind)); and the managed block teaches the first kind rather than listing none ([§FS-init.2.3.4.4.2](FS-init.md#23442-no-kinds-yet)).

#### 3.4.5 Name rules

**Names are unique across the whole table.** `[citations.<kind>]` and `grund list --kind` key on a name, so two rows wearing one name is a config with no answer to "which".

##### 3.4.5.1 Citable names are prefix-free

No citable kind's name may be a prefix of another citable kind's name. `kind = "DA"` and `kind = "DAT"` together are invalid because a token starting with `DAT-` would parse as either kind. The rule is about *tokenization*, so it stops where tokenization does: a non-citable kind's name never appears in an ID, so `skill` beside a citable `SKI` is fine and a config that spells it loads. grund validates this on load and refuses ambiguous configs with a single error pointing at the offending pair (per [§FS-config.4.3](FS-config.md#43-invalid-config-behavior)).

##### 3.4.5.2 `code` is reserved to the homeless kind

`code` is the homeless kind's ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)): it is the default name of the complement of every configured home, so a row may take it only by *being* that complement — `citable = false`, no `folder`, no `file`. Any other row wearing it would collide with the kind every citation outside a home resolves to.

#### 3.4.6 `prefix`, the former spelling of `kind` *(removed in 0.13.0)*

`prefix` was this key's name while every kind declared IDs and its name really was one. It stopped loading in grund **0.13.0**, at the end of the deprecation window [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) asks of a renamed config key: 0.12.0 shipped `kind` beside it and warned every config that still spelled it, naming this release. A config that still spells it is **refused**, not read with the key ignored — an ignored name leaves a `[[kinds]]` row with no kind, which changes what the configuration means without saying so. The refusal is an ordinary config error ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)) that names `kind` as the key to write instead, located at the line `prefix` is written on. An entry that sets both `kind` and `prefix` earns that same error at that same line: with one of the two names gone there is nothing left to disambiguate, so the pair is no longer a rule of its own.

```text
error: grund.toml:4: [[kinds]] `prefix` was removed in grund 0.13.0 — rename it to `kind`
```

##### 3.4.6.1 The migration

The migration is that rename, and the error names the line to make it on. A grund before 0.13.0 did it in one command — `config show` printed every entry under the canonical spelling whichever the file used, so `grund config show > grund.toml` rewrote the file — but from 0.13.0 nothing that has to load the config can help.

##### 3.4.6.2 Why the key was renamed

The rename is what `citable = false` forces. `prefix` was accurate for every row of the table and stopped being accurate for half of it; *kind* is what the rest of grund already calls this value — the `{kind}` placeholder of `[id] format` ([§FS-config.3.2](FS-config.md#32-id--id-grammar)), the `--kind <KIND>` selector of [§FS-list.1](FS-list.md#1-inputs), and the `[citations.<kind>]` table key ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)). Under the new name, prefix-ness is a *derived* property of citable kinds ([§FS-config.3.4.5](FS-config.md#345-name-rules)) rather than this specification's word for the whole concept. Decided in [§DF-non-citable-kinds.2.4](../decisions/functional/DF-non-citable-kinds.md#24-the-field-is-a-kind-not-a-prefix).

#### 3.4.7 `scan` — a place that is listed, not scanned

`scan = false` (default `true`) keeps a non-citable kind's home out of the scan. The kind is still a **place**: it gets its Project map row ([§FS-init.2.3.4.4](FS-init.md#2344-project-map)) with its title, so an agent is told the directory exists and what it is for — and nothing in it is read, so nothing in it is checked.

```toml
[[kinds]]
kind = "template"
folder = "templates"
citable = false
scan = false
title = "Init scaffold templates: what grund init writes, verbatim"
```

`grund config show` ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)) prints `scan = false` where it is set and nothing where it is not, as it does for `citable`. The key is additive and does not move `grund_config_version` ([§FS-config.5](FS-config.md#5-schema-versioning)). Decided in [§DF-unwalked-kind-home](../decisions/functional/DF-unwalked-kind-home.md#df-unwalked-kind-home-a-kind-may-be-a-place-that-is-listed-but-not-walked).

##### 3.4.7.1 What it is for

The case it exists for is content that ships verbatim somewhere else: scaffold templates, embedded assets, example configs. Such files cannot be grounded — a `§` citation in one lands in every tree it is copied into as a dangling citation to a declaration that tree does not have — and leaving the kind unconfigured would leave the directory out of the map. [§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)'s rule that a home is a scan root `exclude` cannot prune is about a config that says both "this directory matters" and "skip its descendants"; this key is the config saying one thing: listed, not scanned.

##### 3.4.7.2 Not scanned, however the scan arrives

The home is left out of the scan roots of [§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned), *and* pruned when a scan meets it on the way down — under the config root, or under an `include` entry it sits inside, as `docs/templates` sits inside `docs`. An `include` entry that names the home itself does not scan it either: the narrower key, the one written on the kind, is the config's answer where the two disagree. A `file` home ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)) is unscanned on the same terms as a `folder` one, and it is the case that shows why the rule cannot be a rule about directories: `docs/template.md` is never a directory to skip, and skipping its parent is not on offer, `docs` being an ordinary scanned home. Anything less would make `scan = false` a silent no-op for every repository that keeps such a file or directory under a scanned one, which is where a scaffold usually is.

##### 3.4.7.3 An explicit path argument still reads it

An explicit path argument still reads it — `grund check docs/templates` scans the directory it names, the same way it reads past `[scan] include` ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)). The key describes the *default* scope, which is what a run with no argument reads and what [§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only) reports against; a path a user typed is that user narrowing the run to a directory they are asking about.

##### 3.4.7.4 What an unscanned kind keeps and loses

What an unscanned kind keeps: its home, its title, and its Project map row. What it loses, beyond what `citable = false` already takes ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)), is every rule that reaches a file — citation checking, the directions bullet and `[citations.<kind>]` rules, and the grounding clause of [§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids) — because no file in it is scanned. That is why `require_grounding = true` on this row is a config error rather than a no-op ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)).

##### 3.4.7.5 Under `--full`

Under `grund check --full` ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)) the whole config root is scanned and its files are reached like any directory nobody configured: the five-finding tier of [§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only), admitted by the test of [§FS-check.3.14.2](FS-check.md#3142-what-is-judged-outside-the-configured-scope), never a convention it did not adopt. They are reached from *outside* the default scope even when a scan root encloses them ([§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only)), because the scope is what a run without the flag reads, and that run does not read them.

##### 3.4.7.6 Three config errors

Three combinations are config errors, reported per [§FS-config.4.3](FS-config.md#43-invalid-config-behavior), each closing a state the key cannot describe:

- `scan = false` on a **citable** kind. Its declarations would be invisible rather than declared — the trap [§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned) closes — so a kind that declares IDs is always scanned. Set `citable = false`, or drop the key.
- `scan = false` with **no home**. The homeless kind ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)) is the complement of every home, and what of that complement is scanned is `[scan] include`'s to say.
- a `[citations.<kind>]` table naming an unscanned kind as the **citing** kind. No file in the home is scanned, so the rule could never fire — the vacuous pass [§DF-non-citable-kinds.2.5](../decisions/functional/DF-non-citable-kinds.md#25-obligations-get-a-per-file-unit-and-grounding-follows-the-home) refused for a kind with no declarations, one level up.

#### 3.4.8 `require_grounding` and `grounding_level` — grounding per place and per level

Two keys say, per kind, **whether** the files of a place must cite a declared ID and **how finely** that is asked. Each has a `[reference]` twin ([§FS-config.3.1.7](FS-config.md#317-require_grounding-and-grounding_level--defaults-for-kinds)) that is the default for every row not setting it — the shape `index` already has, though its default is built in per kind name rather than configured ([§FS-config.3.4.2.4](FS-config.md#3424-the-default-is-per-kind-name)): a default, the row wins.

```toml
[reference]
require_grounding = false      # whether — the default for every row below
grounding_level   = 1          # how fine — 1 = the file; the default for every row below

[[kinds]]
kind = "skill"
folder = "skills"
citable = false
require_grounding = true       # every scanned file here must cite a declared ID
grounding_level = 2            # …and so must every `##` section of it

[[kinds]]
kind = "code"                  # the homeless row (§3.9.2)
citable = false
require_grounding = true       # …must cite one, or declare one inline
```

**Which files a row governs** is [§FS-check.3.6.1](FS-check.md#361-which-files-a-row-governs)'s own predicate, asked per row. Both keys are additive and do not move `grund_config_version` ([§FS-config.5](FS-config.md#5-schema-versioning)). Decided in [§DF-require-grounding.4](../decisions/functional/DF-require-grounding.md#4-grounding-per-place-and-per-level).

##### 3.4.8.1 Why grounding is asked per place

The keys exist because *whether* a file must cite is already reasoned about per place. Direction rules constrain how you ground and never whether ([§DISC-citation-directions](../discussions/proposals/2026-06-13-citation-directions.md#disc-citation-directions-encode-citation-directions-as-checked-config)), and for a non-citable kind grounding follows the home rather than the file extension ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)). One global boolean cannot say "every skill must cite" without also saying it of every workflow and build script in the scan, so the repository that wants the first declines the second and leaves the hole open ([§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing) can then only warn about it).

##### 3.4.8.2 `grounding_level` picks the unit inside each governed file

It is an integer in Markdown heading levels, `1..=6`: `1` is the whole file, text before its first heading included, so one citation anywhere in it, which is exactly the unit every config had before this key existed — `2` adds every `##` subtree, `3` every `###` as well, and `6` every heading Markdown can have. Authors already think in `##`, and `[id] section_heading_levels` uses *level* for the same count, so there is no second numbering to learn. A source file has no headings, so it gets the two ranks grund can see without parsing code ([§FS-non-goals.3](FS-non-goals.md#3-code-ast-parsing)): by indentation, not by syntax. The units and the findings they produce are specified in [§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in); the same unit is what `[citations]` obligations are asked of ([§FS-check.3.11](FS-check.md#311-missing-required-citation)), so *whether* and *what* are asked of the same thing.

##### 3.4.8.3 Precedence is row over global

For both keys, the row wins over `[reference]`. `grund check --require-grounding` ([§FS-check.1](FS-check.md#1-inputs)) is the run-level spelling of the global boolean and sets the same default, so an explicit `require_grounding = false` on a row still wins over the flag: the flag and the key are one knob, and the row's word is the more specific one. The level comes from config only — there is no flag for it.

##### 3.4.8.4 The homeless kind takes both keys like any row

The homeless kind ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)) takes `require_grounding` and `grounding_level` as any row does. A config that never declared it writes the row to set them, the same way it writes one to take a `title`.

##### 3.4.8.5 Five config errors

Five combinations are config errors, reported per [§FS-config.4.3](FS-config.md#43-invalid-config-behavior) at the offending line, each closing a state the keys cannot describe:

- `require_grounding = true` on a `scan = false` row ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned)). No file in the home is read, so the rule could never fire — the reasoning [§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned) already gives for a `[citations.<kind>]` rule on an unscanned kind.
- Either key on a **citable** `file = "<path>"` row. Such a kind's document is where its declarations live, and [§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in) leaves Markdown alone outside a non-citable home, so there is nothing the key could mean — as `index` means nothing on a file kind ([§FS-config.3.4.2](FS-config.md#342-index--the-kinds-index-file)). A **non-citable** `file` row is not rejected: its document is governed like every other file of a non-citable home ([§FS-check.3.6.1](FS-check.md#361-which-files-a-row-governs)), so the row is exactly where that one document's grounding is said, and `grounding_level` cuts it into heading subtrees like any other Markdown file ([§FS-check.3.6.2](FS-check.md#362-the-unit)).
- `grounding_level` outside `1..=6`, on a row or in `[reference]`. There is no heading it could name.
- `grounding_level` on a row whose **effective** `require_grounding` is off — written `false` on the row, or inherited off from `[reference]`. The level could never fire, and a level nothing reads would still switch on the scanner's per-file structure pass ([AR-scanner.2.7](../architecture/AR-scanner.md#27-grounding-units-per-file)) for a tree that grounds nothing.
- `[reference] grounding_level` where the global boolean is off and no row turns grounding on. The same reason, one scope up.

##### 3.4.8.6 `config show`, and why the globals stay

`grund config show` prints each key on a row only where it differs from the effective global ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)), as it does for `citable` and `scan`, so the printed config loads back as itself. The global keys are kept rather than deprecated — every existing config keeps its exact meaning with no edit, and `--require-grounding` needs a global meaning regardless.

#### 3.4.9 `values` — first-class value declarations

`values = true` opts whole declarations in this row's home into [§FS-values](FS-values.md#fs-values-opted-in-kinds-bind-authored-components-to-one-declared-value). It is absent and false by default. It is one of the two value-related config keys: `values` opts whole declarations in, while `value_chapter` ([§FS-config.3.4.13](FS-config.md#3413-value_chapter--the-chapter-whose-named-children-are-values)) makes the named children of one chapter into roots inside declarations. The two are refused on one row, and there is still no `value_sources` key. An enabled row must be citable, have exactly one existing `file` or `folder`, and normalize that home inside the project root. Each violation is a located config error. Validation checks these structural relationships without parsing declaration content.

`grund config show` prints `values = true` only for an enabled row; a false or absent value prints no key. The key is additive and does not change `grund_config_version` ([§FS-config.5](FS-config.md#5-schema-versioning)). Its version-2 spelling is `form = "value"`, with the same meaning and prerequisites ([§FS-fields.forms](FS-fields.md#forms-form-lowers-to-the-version-1-contracts)).

##### 3.4.9.1 The value suffix is a separate authority

The exact `<!-- grund:value -->` section suffix is a separate in-document authority ([§FS-values.2.4](FS-values.md#24-embedded-section-value-roots)). It works in any supported scanned declaration independently of this key and its home, and it neither opts the enclosing declaration in nor changes value-source discovery. No new config key, scan input, or migration accompanies it. Chapter authority is the third and last one, taken per kind rather than per document: `value_chapter` makes every named child of one named chapter a root without any mark ([§FS-values.2.5](FS-values.md#25-chapter-declared-value-roots)).

##### 3.4.9.2 The home is the boundary of its value sources

The home is also the complete boundary of its value sources ([§FS-values.terms](FS-values.md#terms-terms)): a `file` home is the one source when its extension is `.json`, `.yaml` or `.yml`, while a `folder` contributes its normalized, bytewise-ordered direct children with those extensions. Nested or outside files are never value sources. Generic scan extensions and filters, explicit path narrowing, and `--full` neither add nor suppress those home inputs ([§FS-values.2.2](FS-values.md#22-declarations-from-value-sources)).

#### 3.4.10 `format`, `resolve`, and `fetch` — external snapshot kinds

A citable kind may override the repository grammar and describe how a missing
committed snapshot is materialized:

```toml
[[kinds]]
kind = "TICKET"
file = "docs/tickets.md"
title = "External tickets (generated snapshots)"
format = "{kind}-{number}"
resolve = "should"
fetch = "scripts/fetch-ticket"
```

All three keys are optional and additive: a row that omits them retains the
previous grammar, dangling output, and scan cost, and `grund_config_version`
remains 1 ([§FS-config.5](FS-config.md#5-schema-versioning)).

##### 3.4.10.1 `format`

`format` uses exactly the template placeholders and the repository's `number_pattern` and `slug_pattern` validation from [§FS-config.3.2](FS-config.md#32-id--id-grammar). It is [§FS-config.3.2](FS-config.md#32-id--id-grammar)'s `format` written at the kind scope, and resolves like any setting admitted at two scopes ([§FS-config.principle.rungs](FS-config.md#principlerungs-the-committed-scopes-are-a-relation-not-a-closed-list)): it overrides only this kind, and `[id].format` remains the default for every other kind. It is valid without `resolve` or `fetch`, but invalid on a non-citable kind. Numeric spelling follows [§FS-config.3.2.8](FS-config.md#328-numeric-spelling-survives-reading-and-reporting), just as it does under the repository default; this override sets a grammar, not a report width.

##### 3.4.10.2 `fetch` and `resolve`

`fetch` names the direct integration executable specified by [§FS-fetch.2](FS-fetch.md#2-integration-invocation). It requires exactly one `file` or `folder`
home. `resolve` is the target-side obligation used when that kind's citation
has no declaration: the closed enum is `must | should`, with no `may`.
Explicit `resolve` is valid only when `fetch` is also present; `fetch` without
`resolve` is valid and has the effective value `must`. Both keys are invalid on
a non-citable kind, a row with neither home, or a row with both homes.

##### 3.4.10.3 `resolve` selects a finding class

This obligation is independent of the citing-side `[citations]` rules ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)).
It selects one of two fixed finding classes rather than remapping severity:
`must` selects the `dangling` error and `should` selects the
`missing-snapshot` warning ([§FS-check.3.1](FS-check.md#31-dangling-citation),
[§FS-check.4.12](FS-check.md#412-missing-snapshot)).

##### 3.4.10.4 `config show`

`config show` prints an explicit `format` and `fetch`, and prints the effective
`resolve` for a fetch-enabled kind, including the defaulted `must`. The shown
TOML loads back to the same effective values.

#### 3.4.11 What a home is used for

`folder` is used by `grund id` ([§FS-id.2.2](FS-id.md#22---format-json) emits it as the `folder` field; a kind with no configured home prints none) and by editor "create new declaration" / "go to home folder" actions. A home is also a declaration-home boundary: a declaration inside exactly one configured `folder` home must declare that folder's kind ([§FS-declarations.checks.misplaced-declaration](FS-declarations.md#checksmisplaced-declaration-misplaced-declaration-configured-kind-home)). `file` is stricter: declarations of a single-file kind found outside the configured path are reported under [§FS-declarations.checks.misplaced-declaration](FS-declarations.md#checksmisplaced-declaration-misplaced-declaration-configured-kind-home), and a different-kind declaration inside that exact file is likewise a misplaced declaration.

Declarations are still recognized outside configured homes — including inline source declarations — and declarations in files covered by zero or multiple configured homes are not rejected by the home-kind rule because there is no single expected kind. So a kind with no configured home meets [§FS-declarations.checks.misplaced-declaration](FS-declarations.md#checksmisplaced-declaration-misplaced-declaration-configured-kind-home) only when one of its declarations sits inside some other kind's unique configured home.

#### 3.4.12 `rules` — rule declaration kinds

`rules = true` opts every declaration of a citable Markdown kind into the rule
contract of [§FS-rules](FS-rules.md#fs-rules-grounded-declarations-state-and-enforce-chapter-rules): the declaration title is one executable controlled-English
sentence and its body is a non-empty rationale. The Boolean is absent and false
by default, and `config show` prints it only when enabled. Multiple local kinds
may enable it.

The key requires a citable kind and a `file` or `folder` home whose declarations
are Markdown. It is invalid on a non-citable, homeless, unscanned, external-
snapshot, or value-source kind: each either declares no IDs, supplies no scanned
rule catalog, or gives its title another source contract. These relationships
are config validation; sentence titles are scanned declaration data and are
validated later as located findings ([§FS-rules.4](FS-rules.md#4-validation-lifecycle)).

The key is optional and additive, so `grund_config_version` remains 1 ([§FS-config.5](FS-config.md#5-schema-versioning)).
An older binary rejects the unknown key loudly rather than silently ignoring a
rule-enabled repository. Its version-2 spelling is `form = "rule"`, with the same
meaning and prerequisites ([§FS-fields.forms](FS-fields.md#forms-form-lowers-to-the-version-1-contracts)).

#### 3.4.13 `value_chapter` — the chapter whose named children are values

`value_chapter` names, for one `[[kinds]]` row, the chapter whose named direct
children are value roots in every declaration of that kind ([§FS-values.2.5](FS-values.md#25-chapter-declared-value-roots)):

```toml
[[kinds]]
kind = "AR"
folder = "docs/architecture"
value_chapter = "values"
```

The value is a section name: a string matching the fixed named-component
grammar `[a-z][a-z0-9-]*` of [§FS-config.3.2.7](FS-config.md#327-named_sections--the-gate-for-explicit-section-names), never a displayed heading title
and never a `slug_pattern` match. The key is absent by default, absent and set
to nothing mean the same, and `grund config show` prints it only on a row that
sets it. It is optional and additive, so `grund_config_version` remains 1 ([§FS-config.5](FS-config.md#5-schema-versioning)),
and an older binary rejects the unknown key loudly rather than silently reading
a chapter-enabled repository as if it declared no values.

An enabled row must be citable, must have exactly one existing `file` or
`folder` home normalizing inside the project root, and must not set `scan =
false`: a non-citable row declares no IDs so no coordinate could exist, a
homeless row has no declarations of its own, and a listed-but-unscanned place is
never read, so its chapter could never be seen. The row must not also set
`values = true`: a whole-declaration value's immediate children must be a
contiguous numeric run ([§FS-values.2.1](FS-values.md#21-markdown-declarations)), so a named chapter cannot legally
exist inside one, and the pair is refused rather than merged. Setting the key
twice on one row is an error, as `values` and `rules` already are.

`[id] named_sections = true` is a prerequisite of the key rather than a
consequence of it: a project that sets `value_chapter` while `named_sections` is
absent or false is a located config error naming the missing gate, not a silent
no-op, and the `named_sections` default does not change ([§FS-config.3.2.7](FS-config.md#327-named_sections--the-gate-for-explicit-section-names)). Each
violation above is a located config error. Validation checks these structural
relationships without parsing declaration content; a chapter that a declaration
fills wrongly is scanned data and is reported later as a located finding ([§FS-check.3.20](FS-check.md#320-invalid-value-declaration)).
Its version-2 spelling is a field with `content = "values"`, which means the
same and lowers to the same record ([§FS-fields.content.values](FS-fields.md#contentvalues-a-values-field-is-the-value-chapter)).

### 3.5 `[scan]` — what gets scanned

```toml
[scan]
include            = ["requirements.md", "docs", "e2e", "src"]
exclude            = ["target", "node_modules", ".git", "dist", "build", ".venv"]
extensions         = ["md", "rs", "go", "java", "kt", "ts", "tsx", "js", "py", "c", "cpp", "swift", "scala", "rb", "php", "cs", "lisp", "scm", "clj", "sql", "hs", "lhs", "lua", "ada", "adb", "ads", "h", "cc", "cxx", "hpp", "hh", "hxx", "m", "mm", "kts", "jsx", "mjs", "cjs", "dart", "r", "R", "sh", "bash", "zsh", "cl", "el", "ss", "rkt", "cljs", "cljc"]
comment_prefixes   = ["//", "#", ";", "--", "*", "/*"]
docstring_python   = true
respect_gitignore  = true
```

Which files these keys make a run read is [§FS-scan.scope](FS-scan.md#scope-which-files-a-run-reads). `comment_prefixes` are the tokens recognized when looking for source declarations and citations in source files, composed with `extensions` ([§FS-config.3.5.14](FS-config.md#3514-comment_prefixes-compose-with-extensions)); `docstring_python` enables Python triple-quoted-string scanning in addition to `#` comments. The `extensions` default is not a list of its own choosing: it is `md` plus every extension grund's built-in language table names ([§FS-scan.scope.extensions.default](FS-scan.md#scopeextensionsdefault-the-extensions-default-follows-the-language-table)).

The `include` default keeps `requirements.md` although no built-in kind homes there any more ([§FS-config.3.4.4](FS-config.md#344-the-default-kinds)): dropping it would stop reading a file in a project that changed nothing, a verdict no release warned about ([§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)).

An opted-in kind's value sources ([§FS-values.terms](FS-values.md#terms-terms)) are catalog input rather than part of this scan. The one way this table touches them is the YAML claim: a `.yaml` or `.yml` file whose extension is in the effective `extensions` — the list written here, or the default where none is — is scan input and so not a value source, while a JSON one never is. The exact discovery and independence from every other key in this table are fixed by [§FS-config.3.4.9](FS-config.md#349-values--first-class-value-declarations) and [§FS-values.2.2](FS-values.md#22-declarations-from-value-sources).

#### 3.5.1 A symlink in the tree is followed

See [§FS-scan.scope.links](FS-scan.md#scopelinks-a-symlink-in-the-tree-is-followed).

#### 3.5.2 A finding names the in-tree link path

See [§FS-scan.identity.link-path](FS-scan.md#identitylink-path-a-finding-names-the-in-tree-link-path).

##### 3.5.2.1 The scan root keeps the path the run was handed

See [§FS-scan.identity.root-path](FS-scan.md#identityroot-path-the-scan-root-keeps-the-path-the-run-was-handed).

#### 3.5.3 The directory rules apply under the link name

See [§FS-scan.scope.links.names](FS-scan.md#scopelinksnames-the-directory-rules-apply-under-the-link-name).

#### 3.5.4 One physical file is read once

See [§FS-scan.identity.read-once](FS-scan.md#identityread-once-one-physical-file-is-read-once).

#### 3.5.5 A link the scan cannot resolve is reported, and not scanned into

See [§FS-scan.scope.links.unresolvable](FS-scan.md#scopelinksunresolvable-a-link-the-scan-cannot-resolve-is-reported-and-not-scanned-into).

#### 3.5.6 Which unresolvable links are owed a report

See [§FS-scan.scope.links.owed](FS-scan.md#scopelinksowed-which-unresolvable-links-are-owed-a-report).

##### 3.5.6.1 A broken link with no extension is a declared blind spot

See [§FS-scan.scope.links.blind-spot](FS-scan.md#scopelinksblind-spot-a-broken-link-with-no-extension-is-a-declared-blind-spot).

#### 3.5.7 `include` is scanned from the config root

See [§FS-scan.scope.include](FS-scan.md#scopeinclude-include-is-scanned-from-the-config-root).

#### 3.5.8 Every configured kind home is scanned

See [§FS-scan.scope.homes](FS-scan.md#scopehomes-every-configured-kind-home-is-scanned).

#### 3.5.9 A scan root outruns every rule about descent

See [§FS-scan.scope.roots](FS-scan.md#scoperoots-a-scan-root-outruns-every-rule-about-descent).

#### 3.5.10 Why every home is scanned

See [§FS-scan.scope.homes.why](FS-scan.md#scopehomeswhy-why-every-home-is-scanned).

#### 3.5.11 `include` is a scan scope, not a fence

See [§FS-scan.scope.not-a-fence](FS-scan.md#scopenot-a-fence-include-is-a-scan-scope-not-a-fence).

#### 3.5.12 A hidden file is not read, and the rule is not about descent

See [§FS-scan.scope.hidden](FS-scan.md#scopehidden-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent).

#### 3.5.13 An extension makes a file readable, not declarable

See [§FS-scan.scope.extensions](FS-scan.md#scopeextensions-an-extension-makes-a-file-readable-not-declarable).

#### 3.5.14 `comment_prefixes` compose with `extensions`

How `comment_prefixes` and `extensions` compose, and which lines a configured prefix makes eligible, is [§FS-languages.prefixes](FS-languages.md#prefixes-comment_prefixes-compose-with-extensions).

#### 3.5.15 `respect_gitignore` — the ignore files

See [§FS-scan.scope.ignore-files](FS-scan.md#scopeignore-files-respect_gitignore--the-ignore-files).

#### 3.5.16 An `exclude` entry containing `/` is refused

Each `[scan] exclude` entry is one directory **component name**, compared against each directory's own name at every depth; it is never a path and never a glob. An entry containing `/` therefore names no directory and could never match one, so it is not accepted in silence: the config fails to load as an invalid value ([§FS-config.requirements.5](FS-config.md#requirements5-a-mistake-in-the-config-fails-loudly--realized-one-case-deferred)), with the single located error of [§FS-config.4.3](FS-config.md#43-invalid-config-behavior) — `check` exits `2`, `config validate` exits `1`, the error is on stderr, stdout is empty, and no document is read, so no document finding is reported.

The entries are judged in list order and the **first** one containing `/` is the one reported. The error names that entry, is located at the config file's path and the physical line of the `exclude` key, and names both repairs:

```text
error: grund.toml:4: [scan] exclude entry `docs/plans` contains `/`, not a directory name; remove it to keep the scan unchanged, or use `plans` to exclude that name at any depth
```

The two repairs are different actions, and neither is the path-specific exclusion the entry looked like. Removing the entry keeps exactly the scan the repository already had, because the entry never excluded anything. Writing its last non-empty component (`plans`) is a new policy: it excludes every directory of that name, at every depth, not only `docs/plans`. An entry with no non-empty component, such as `/`, is offered the first repair alone: its error ends at `remove it to keep the scan unchanged`. Excluding one path rather than a name everywhere is not a v1 capability; it belongs to the v2 sources contract, whose exclusions are globs ([§DISC-core-concerns.8.3](../discussions/proposals/2026-09-30-core-concerns.md#83-the-two-exclude-keys-mean-opposite-things)).

Only `/` is refused. Every other entry is accepted as it always was, and an entry that is a well-formed name keeps matching at every depth. `[fmt] exclude` is a separate key with a glob grammar ([§FS-config.3.10.1](FS-config.md#3101-entries-are-gitignore-style-globs)), so `docs/plans/**` stays valid there. Why this ships as an immediate error is [§DF-scan-exclude-component-names](../decisions/functional/DF-scan-exclude-component-names.md#df-scan-exclude-component-names-a-scan-exclude-entry-containing--is-a-config-error).

### 3.6 `[output]` — report format

```toml
[output]
format         = "text"   # text | json
color          = "auto"   # auto | always | never
relative_paths = true     # show paths relative to config root in reports
```

These three keys are the v1 defaults of the run flags ([§FS-cli.3](FS-cli.md#3-cross-subcommand-flags)): `format` is the default of `--format` for the commands that honour it, `relative_paths` the default of `--path-base`, and a flag passed on the command line outranks the key for that run — flag, then key, then default. `config show` still prints the file's own values. `color` has no run-flag counterpart.

`relative_paths = true` (default) renders every `<path>` in a report relative to the config root ([§FS-config.1](FS-config.md#1-file-location-and-discovery)); `relative_paths = false` renders it relative to the CLI base instead ([§FS-config.3.6.1](FS-config.md#361-relative_paths--false--the-cli-base)). Either way `grund` **never** emits an absolute path, nor a path that escapes the loaded root other than the `..` path of a config above the run's root that the run had to read (`../grund.toml:16`, [§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)) or the `..` path of a file in an import ([§FS-imports.paths](FS-imports.md#paths-imported-files-are-reported-where-they-are)); this is what keeps the report deterministic per [§FS-errors.4](FS-errors.md#4-determinism). `color` controls ANSI styling once the colored-output feature lands ([§FS-errors.3](FS-errors.md#3-message-text)); until then output is plain bytes regardless of this value, and a change to that default goes through the [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path) path.

#### 3.6.1 `relative_paths = false` — the CLI base

`relative_paths = false` renders every `<path>` relative to the path argument passed on the command line — or to the current working directory when no path is given. A target elsewhere inside the loaded project or workspace uses the minimum `..` components needed to reach it from that base; those parent components are allowed only while the resolved target remains inside the loaded root. The CLI-base choice and the rejected workspace-root alternative are recorded in [§DF-cli-base-parent-paths](../decisions/functional/DF-cli-base-parent-paths.md#df-cli-base-parent-paths-relative_paths--false-keeps-one-cli-base-and-may-climb-within-the-loaded-root). The CLI base is also exactly what `--path-base=invocation` means ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)), so `relative_paths = false` and that flag produce the same bytes, and the flag reaches it without committing the key.

### 3.7 `[fmt.cross_refs]` — cross-reference emission

```toml
[fmt.cross_refs]
enabled       = true       # default; false opts out of generated Markdown links
anchor_format = "github"   # default; one of github | gitlab | mkdocs | pandoc | none
```

The full contract for this block — what `enabled` does, the named `anchor_format` profiles, and when the cross-reference pass runs — lives in [§FS-fmt.6.7](FS-fmt.md#67-configurability), [§DF-md-link-default-on](../decisions/functional/DF-md-link-default-on.md#df-md-link-default-on-markdown-cross-reference-links-default-on-for-github-review-and-discovery), and [§DF-md-link-anchor-strategy](../decisions/functional/DF-md-link-anchor-strategy.md#df-md-link-anchor-strategy-heading-text-slugs-re-derived-on-every-fmt-pass). It is documented here with the other keys because the generated `grund.toml` ([§FS-init.2.4](FS-init.md#24-generated-grundtoml)) writes every key in this section explicitly, including `enabled = true`, so the default generated file teaches that `grund fmt --write` emits Markdown inline links in `.md` files.

#### 3.7.1 One block for every cross-reference form

`[fmt.cross_refs]` is the home for cross-reference settings; today `grund fmt --cross-refs` only emits the Markdown inline-link form ([§FS-fmt.6](FS-fmt.md#6-cross-reference-emission)), so `anchor_format` is the only knob — a future markup family adds its settings under this same block ([§FS-fmt.6.7](FS-fmt.md#67-configurability)), additively, with no `grund_config_version` bump ([§FS-config.5](FS-config.md#5-schema-versioning)). The sibling `[fmt]` table is a different thing ([§FS-config.3.10.3](FS-config.md#3103-fmt-is-the-commands-home-fmtcross_refs-the-passs)).

### 3.8 `[workspace]` — sub-project members

```toml
[workspace]
members          = ["apps/api", "packages/*"]
optional_members = ["vendored"]
include_root     = true
```

`members`, `optional_members` and `include_root` are specified by [§FS-workspace](FS-workspace.md#fs-workspace-grund-validates-cross-project-citations-in-a-workspace). The table is optional; without it the repository is a single project exactly as before. Unknown keys under `[workspace]` are errors like any other config typo.

`optional_members` is purely additive — a config that omits it behaves exactly as it did before the key existed — so `grund_config_version` stays `1` ([§FS-config.5](FS-config.md#5-schema-versioning)), and a binary older than the key refuses it through the unknown-key rule above rather than ignoring it, which is the loud failure [§FS-config.5](FS-config.md#5-schema-versioning) asks of a config a binary cannot honour.

### 3.9 `[citations]` — citation direction rules

```toml
[citations]               # absent section = no direction checks (backward compatible)
default = "may"           # global default level for unlisted (citing → cited) pairs

[citations.FS]
should = ["GOAL|FS"]      # an FS declaration should cite a GOAL or a parent FS
must-not = ["AR"]         # an FS citation site may never cite an AR

[citations.E2E]
must = ["FS"]             # every E2E case must cite the FS it tests

[citations.code]          # the homeless kind (§3.9.2), named `code` by default
should = ["FS|AR"]
```

Each `[citations.<kind>]` subsection names the **citing** kind, by the `kind` name of [§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds); its arrays name the **cited** kinds. The citing side may be any configured kind — citable or not ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)), but never an unscanned one ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned)) — or the homeless kind, `code` unless the project named it otherwise ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)); the cited side must be a **citable** kind, because a kind with no IDs has nothing a citation could point at. The section is decided in [§DF-citation-directions](../decisions/functional/DF-citation-directions.md#df-citation-directions-encode-citation-directions-as-checked-config-with-rfc-2119-levels) and proposed in [§DISC-citation-directions](../discussions/proposals/2026-06-13-citation-directions.md#disc-citation-directions-encode-citation-directions-as-checked-config). It is optional; without it no direction check runs and `grund check` behaves exactly as before. Its complete user-facing explanation is [Citation directions](../user-facing/citation-directions.md); the skill carries that page as a checked copy, while the generated entrypoint carries a checked render of configured rules.

#### 3.9.1 Levels

Five keys form an RFC-2119 ladder, split into two rule classes and two enforcement surfaces:

| Level | Rule class | Checked per | Surface |
|---|---|---|---|
| `must` | obligation | declaration | `grund check` error — `missing-citation` ([§FS-check.3.11](FS-check.md#311-missing-required-citation)) |
| `should` | obligation | declaration | suggestion — `suggested-citation` ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)) |
| `may` | permission | — | never checked; punches a hole in a stricter `default` |
| `should-not` | prohibition | citation site | suggestion — `discouraged-citation` ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)) |
| `must-not` | prohibition | citation site | `grund check` error — `forbidden-citation` ([§FS-check.3.12](FS-check.md#312-forbidden-citation)) |

##### 3.9.1.1 Obligations and prohibitions

An **obligation** asks: does each top-level declaration of the citing kind contain at least one citation to the target kind, anywhere in its body? Multiple array entries are **conjunctive** — `must = ["GOAL", "GRUND"]` requires a citation to each — while a `|` disjunction inside one entry is satisfied by any one alternative — `must = ["GOAL|GRUND"]` requires a citation to either. A **prohibition** fires once per offending citation site, located at its exact `file:line`.

##### 3.9.1.2 `E2E` obligations are per case

`E2E` obligations are per case declaration: they evaluate the case's scanned files plus the case manifest's `spec.refs` entries, but an otherwise empty evidence set still fails a `must` entry rather than satisfying it vacuously. A `spec.refs` entry is kind-shaped evidence — it may name an idealized ID that does not resolve locally, so it is not subject to the dangling check that governs ordinary citations.

##### 3.9.1.3 The level→surface mapping is fixed

`must` and `must-not` gate (`grund check` errors); `should` and `should-not` are machine-checked suggestions that never appear in `grund check`'s standing output and are surfaced only at write time (the generated entrypoint, [§FS-init.2.3.5](FS-init.md#235-citation-directions)) and on demand (`grund check --suggestions`, [§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)). The level→surface mapping is fixed, never a project knob, so two installs reading one config agree on what gates and what is suggested ([§FS-non-goals.9](FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization)).


##### 3.9.1.4 A file that cites nothing is not judged

The per-file obligation of a non-citable kind and of the homeless kind judges only a file that carries a citation ([§FS-check.3.11.5](FS-check.md#3115-a-file-with-no-citation-is-no-unit)); whether a file cites at all is `require_grounding`'s question ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)). The descriptions of `must` and `should`, which `grund config explain` prints ([§FS-config-explain.3](FS-config-explain.md#3-a-keys-answer)), say so: each is followed by the sentence ``A file of a place or the complement that cites nothing is not checked by this key; `require_grounding` makes it a finding.``, so `must` reads `Kinds each declaration (or file, for a place) must cite; a miss is an error.` and then that sentence, and `should` reads `Kinds it should cite; a miss is a suggestion.` and then that sentence. Version 2 states the same fact for its keys at [§FS-config-v2.rules.citations.uncited](FS-config-v2.md#rulescitationsuncited-an-obligation-never-judges-a-file-that-cites-nothing).

#### 3.9.2 The homeless kind

Every citation site that no single configured kind home claims — outside every home, or in a file two overlapping homes contain — resolves to one citing kind ([§FS-citations.citing-kind](FS-citations.md#citing-kind-which-kind-a-citation-site-counts-as)). That kind is the **complement** of the whole `[[kinds]]` table: it is the one kind that is not a place, which is why it has no `folder` and no `file` and why there is exactly one of it. Its name is `code` by default, and a project may name it something truer by declaring it ([§FS-config.3.9.2.1](FS-config.md#3921-declaring-it)).

##### 3.9.2.1 Declaring it

```toml
[[kinds]]
kind = "src"                              # or `modules`, `implementation`, `code`…
citable = false                           # required: the complement declares no IDs
title = "Terraform modules and shell"     # optional: what it covers, for the generated block

[citations.src]
should = ["FS|AR"]
```

An entry is the homeless kind exactly when it sets `citable = false` and neither `folder` nor `file` — that shape is the declaration, not a separate key. Declaring two is a config error ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)): a complement is one place, and two rows claiming it leave the fallback with no single answer.

Naming the kind moves the rules with it: `[citations.src]` governs those sites, and `[citations.code]` in that config names an unknown kind ([§FS-config.3.9.5](FS-config.md#395-validation)) rather than sitting inert.

##### 3.9.2.2 `code` is the default name, and reserved

`code` is the default rather than a fixed name because it is the right word for most repositories and the wrong one for some — a Terraform tree, a SQL tree, a prose tree. It is still **reserved** to this kind ([§FS-config.3.4.5.2](FS-config.md#3452-code-is-reserved-to-the-homeless-kind)), so declaring `code` with a `title` is how a project keeps the name and says what it covers.

##### 3.9.2.3 Grounding the source tree and nothing else

The row takes `require_grounding` and `grounding_level` like any other ([§FS-config.3.4.8.4](FS-config.md#3484-the-homeless-kind-takes-both-keys-like-any-row)), and that is how a project asks for grounding of its source tree and of nothing else: `kind = "code"`, `citable = false`, `require_grounding = true`. Written on this row the keys govern the complement alone; written in `[reference]` they are the default this row inherits with every other.

##### 3.9.2.4 Obligations apply per source file

**Obligations apply per file** — and per section unit as well where the row's `grounding_level` is above `1` ([§FS-check.3.11](FS-check.md#311-missing-required-citation)) — only to files that contain at least one citation, and only to **source files** under the exact predicate `require_grounding` uses — a scanned file whose extension is not `.md` ([§DF-require-grounding.2.2](../decisions/functional/DF-require-grounding.md#22-grounded-is-defined-syntactically)). Markdown outside a kind home (a README, the changelog) is therefore prohibition-checked but obligation-exempt. A configured non-citable kind *with* a home ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)) is the same species and differs on exactly that point: its unit is every scanned file in its home, `.md` included ([§FS-check.3.11](FS-check.md#311-missing-required-citation)).

##### 3.9.2.5 No Project map row, and the last directions row

The homeless kind **gets no Project map row** ([§FS-init.2.3.4.4](FS-init.md#2344-project-map)) — every row there links a place, and this is the kind that has none. Its citation-directions row renders **last**, wherever in the table it was declared, under the subject [§FS-init.2.3.5](FS-init.md#235-citation-directions) gives the homeless kind.

#### 3.9.3 Alias matching

Rule entries reuse the citation grammar of [§FS-workspace.1](FS-workspace.md#1-citation-syntax): a bare `AR` matches the **local** project only; `alias/AR` pins one workspace member, spelled with the same whole alias path a citation uses (`group/api/AR`, [§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)); `*/AR` matches the kind in **any** project including the local one. `*/` is new syntax valid in rule entries only — it is never a citation. Each entry parses as `[alias-path-or-*/]KIND`, split at the last `/` exactly as a citation is; a malformed qualifier is rejected ([§FS-config.3.9.3.1](FS-config.md#3931-a-malformed-qualifier-is-rejected)). The match is textual on the qualifier and kind, and which kinds a qualifier's project has is validation's question, not matching's ([§FS-config.3.9.5.2](FS-config.md#3952-a-pinned-target-is-judged-against-its-projects-kinds), [§FS-scan.projects.kinds](FS-scan.md#projectskinds-whose-kinds-a-run-reads)); resolution failures are separate errors ([§FS-check.3.8](FS-check.md#38-cross-project-citation-failure)), so the direction check never loads a foreign config. Each member's own `[citations]` governs the citation sites in that member's tree — like `strict`, `require_grounding`, and `[id]`, no section inherits from the workspace root. So a member that must cite no other project says so in its own config: it closes its `default` to `must-not` ([§FS-config.3.9.4](FS-config.md#394-defaults-and-precedence)) and lists each citing kind's own targets bare under `may`, each non-citable kind ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)) and the homeless `code` kind included, so a citation of any other project falls to that default without the member naming one; a rule at the workspace root says nothing about that member's sites ([§FS-config.principle.boundary](FS-config.md#principleboundary-the-project-is-the-widest-committed-scope)).

##### 3.9.3.1 A malformed qualifier is rejected

A malformed qualifier is rejected with a citation-target finding that names the target's kind and the first invalid qualifier segment; an empty qualifier or segment is named explicitly. A `*` segment is invalid unless it is the whole qualifier, and the finding says so.

#### 3.9.4 Defaults and precedence

`default` (top-level, or per-kind inside a `[citations.<KIND>]` table) sets the level for unlisted target kinds; the global default is `may`, so adoption is incremental. Precedence is **explicit target list > per-kind `default` > global `default`**. Obligations come only from explicit `must` / `should` entries — a `default` of `must` or `should` never invents an obligation toward every unlisted kind; `default` governs only how a citation to an otherwise-unlisted target is leveled for the prohibition pass.

#### 3.9.5 Validation

Config validation rejects: a `[citations.<kind>]` table whose kind is neither a configured `[[kinds]]` name nor the homeless kind's name ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)) — so `[citations.code]` is rejected in a config that named its complement something else; one naming an unscanned kind ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned)); a target naming a kind that is not a configured *citable* kind of the project its qualifier names ([§FS-config.3.9.5.2](FS-config.md#3952-a-pinned-target-is-judged-against-its-projects-kinds)) — reported as *an unknown target kind* for a name the table does not hold and *a non-citable target kind* for one it does, since those are different mistakes; `code` used as a `[[kinds]]` name by anything but the homeless kind ([§FS-config.3.4.5](FS-config.md#345-name-rules)); two entries declaring the homeless kind ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)); an unknown level key; and two targets of the same cited kind at different levels whose alias matchers can match the same citation ([§FS-config.3.9.5.1](FS-config.md#3951-overlap-not-textual-equality)). The section composes unchanged with project-defined `[[kinds]]` — a new kind is one more `[citations.<KIND>]` table.

Adding `[citations]` does **not** bump `grund_config_version` ([§FS-config.5](FS-config.md#5-schema-versioning)): it is additive surface, like `[workspace]` and `require_grounding`. An older binary meeting it fails loudly with `unknown config section`.

##### 3.9.5.1 Overlap, not textual equality

The rule on two targets of one cited kind is on alias **overlap**, not textual equality — `*/AR` (any project) overlaps a bare `AR` (local), so listing one at `should` and the other at `must-not` is rejected, while a local `AR` permitted alongside a pinned `alias/AR` forbidden is allowed because those matchers are disjoint ([§FS-config.3.9.3](FS-config.md#393-alias-matching)).

##### 3.9.5.2 A pinned target is judged against its project's kinds

A target is judged against the kinds of the projects its qualifier names, read as [§FS-scan.projects.kinds](FS-scan.md#projectskinds-whose-kinds-a-run-reads) says: a bare `KIND` against the local project's, `alias/KIND` against the aliased project's, and `*/KIND` against every loaded project's. So `[citations.FS] must = ["api/API"]` at a workspace root whose member `api` configures `API` loads and is enforced exactly as the rule `Each FS must cite at least one api/API.` is, and an `FS` declaration that cites no `api/API` declaration gets `<path>:<line>: error: FS-x must cite api/API (citation direction)`. The citing side, `[citations.<kind>]`, and every bare target are judged at load, as before. A pinned or `*/` target is judged when the workspace is expanded, the first point that holds every project's kinds, so `grund config validate` judges it against the workspace its file declares and gives the load's answer. A narrowed `cover`, which reads that workspace's kinds without loading its files ([§FS-scan.projects.kinds](FS-scan.md#projectskinds-whose-kinds-a-run-reads)), judges none of them where the workspace does not expand — a member whose config fails to load, say — because the kinds it can read are then not every project's. A pinned or `*/` target the run cannot accept was once refused only when its kind was not a citable kind of the *local* project, which is not the question. Through `0.18.x` each such target loaded, was never matched, and printed one load warning on stderr naming `0.19.0` as the release it became an error in ([§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)). Since `0.19.0` each such target fails the load ([§DF-one-grammar-lands-in-0-19](../decisions/functional/DF-one-grammar-lands-in-0-19.md#df-one-grammar-lands-in-0-19-the-one-grammar-window-closes-in-0190-and-a-pinned-target-the-run-cannot-accept-fails-the-load)): the run prints the first one's line as a CLI-level `error:` ([§FS-errors.2.2](FS-errors.md#22-cli-level-message)), the warning's text with `; this became an error in grund 0.19.0` in place of its clause, writes nothing to stdout, and exits `2`. The first is taken in the order the run judges them: its projects in the order it loads them, and within one config the citing tables in byte order of their kind, each table's levels as `must`, `warn`, `should`, `may`, `should-not`, `warn-not`, `must-not`, and each level's targets as written.

In a workspace whose member `api` configures `FS` not at all and `SKILL` with `citable = false`, the targets `api/FS`, `api/SKILL`, `*/POLICY` (a kind no loaded project configures) and `nope/FS` (an alias the workspace does not load) each fail the load with the line below that names them, and a member `api` whose own config pins `[citations.API] should = ["web/WEB", "*/WEB"]` and configures no `WEB` fails its own run with one of the last two:

```text
error: grund.toml: [citations.FS] must-not names an unknown target kind `FS` in namespace `api`; this became an error in grund 0.19.0
error: grund.toml: [citations.FS] must names a non-citable target kind `SKILL` in namespace `api`; this became an error in grund 0.19.0
error: grund.toml: [citations.FS] must names an unknown target kind `POLICY` in any workspace namespace; this became an error in grund 0.19.0
error: grund.toml: [citations.FS] must names an unknown project alias `nope`; this became an error in grund 0.19.0
error: grund.toml: [citations.API] should names `web/WEB`: unknown project alias web; no workspace is in scope here, so the alias cannot be resolved — check from the workspace root; this became an error in grund 0.19.0
error: grund.toml: [citations.API] should names `*/WEB`: no workspace is in scope here, so no namespace can be searched for WEB — check from the workspace root; this became an error in grund 0.19.0
```

The level key is the one the target is listed under, and the path is the config's as every config message names it. A run that holds no workspace — a member's own run, or a project with no `[workspace]` — cannot judge a target pinned to another project, and a `*/KIND` target it cannot accept locally, at all: such a target fails its load the same way, in the wording of [§FS-errors.3.7.1](FS-errors.md#371-the-final-templates). A target the run accepts prints nothing, and a bare target keeps the load error it had. The routes are [§DF-one-kind-verdict](../decisions/functional/DF-one-kind-verdict.md#df-one-kind-verdict-every-command-reads-one-kind-verdict-in-its-own-scope-and-refuses-a-rule-sentence-for-the-first-stage-it-fails)'s; retiring the warnings was [§RM-one-grammar-flip](../roadmap.md#rm-one-grammar-flip-end-the-one-grammar-window-in-0190).

### 3.10 `[fmt]` — suppressing the rewrite

```toml
[fmt]
exclude = ["docs/architecture/AR-topology.md", "docs/diagrams"]
```

`exclude` is the set of files `grund fmt` performs **no** rewrite in. The full contract — that it covers all four rewrite classes, that a suppressed file is still scanned and still checked, and that a kind's index entries are wrapped there anyway — is [§FS-fmt.2.5.1](FS-fmt.md#251-fmt-exclude--a-file-at-a-time). Nothing else reads the key: no scanner, checker, or query behavior depends on it, and `grund check` says exactly the same thing about a tree with the key and without it.

#### 3.10.1 Entries are gitignore-style globs

Each entry is a gitignore-style glob resolved against the config root ([§FS-config.1](FS-config.md#1-file-location-and-discovery)) — the same dialect `respect_gitignore` already brings to the scan ([§FS-config.3.5.15](FS-config.md#3515-respect_gitignore--the-ignore-files)) — so `docs/diagrams` takes every file under that directory, `AR-*.md` matches at any depth, and `docs/architecture/AR-topology.md` names one file. A pattern the glob parser rejects is a config error at its own line, per [§FS-config.4.3](FS-config.md#43-invalid-config-behavior).

#### 3.10.2 Optional, empty by default, and additive

The table is optional and defaults to the empty list, which is what every config written before the key existed means. It is additive, so `grund_config_version` stays 1 ([§FS-config.5](FS-config.md#5-schema-versioning)), and an older binary meeting it fails loudly through the unknown-section rejection ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)) rather than silently ignoring it. `grund config show` ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)) prints the table only where the list is non-empty, so a shown config still loads back as itself.

#### 3.10.3 `[fmt]` is the command's home, `[fmt.cross_refs]` the pass's

`[fmt]` is the home for settings about the `fmt` command as a whole — it governs every rewrite `grund fmt` performs, not just the cross-reference pass; `[fmt.cross_refs]` ([§FS-config.3.7](FS-config.md#37-fmtcross_refs--cross-reference-emission)) remains the home for cross-reference settings specifically. The per-region counterpart to this key is not configured here at all — it is written in the file it governs ([§FS-fmt.2.5.2](FS-fmt.md#252-grundfmt-off--grundfmt-on--a-region-at-a-time)), and the reasoning for both is in [§DF-fmt-suppression](../decisions/functional/DF-fmt-suppression.md#df-fmt-suppression-fmt-suppression-is-per-file-and-per-region-and-the-index-carve-out-outranks-both).

### 3.11 A list element ends at a comma outside a string

A string list, the `["…", "…"]` array of basic strings that [§FS-config.3](FS-config.md#3-keys-version-1)'s surface reads, is cut into its elements only at a comma that stands outside a string. A comma inside a basic string belongs to that element, and the element is read as written: `files = ["specs/a,b/spec.md"]` names the file `specs/a,b/spec.md`, `exclude = ["a,b"]` names the directory `a,b`, and a brace pattern such as `"specs/{001-a,002-b}/spec.md"` reaches its key as one element. A string closes at the first `"` that is not part of a `\"` escape, so `["a\",b"]` is the one element `a",b` and `["a\\", "b"]` the two elements `a\` and `b`. Every list key of both versions reads its elements this way, and so does a list written across lines once it is joined ([§FS-config-v2.reader.5](FS-config-v2.md#reader5-an-array-may-span-lines)). The two members of the closed `lead_size_warning` inline table ([§FS-config.3.1.2](FS-config.md#312-lead_size_warning--the-oversized-lead-opt-in)) are separated by the same rule, so `unit = "lines,words"` is refused as an unknown unit that names `lines,words`, not as a value that is not a string.

The rule reads more and refuses nothing new, and no list read before it is read differently: in version 1 for good, and in version 2 through grund 0.21.x, after which an element whose string closes before its end is refused, from grund 0.22.0 on ([§FS-config-v2.reader.6](FS-config-v2.md#reader6-a-string-ends-at-its-first-unescaped-quote)), and so is an array with a comma before its `]` that has no element before it ([§FS-config-v2.reader.5.commas](FS-config-v2.md#reader5commas-an-array-takes-one-trailing-comma-after-an-element)). A comma belongs to an element only when that element is one basic string: it opens with a `"`, closes with a `"` that is not part of a `\"` escape, and holds no other such `"`. Text between two separating commas that is not one basic string is cut at each of its commas, as every list was before this rule, so a list with a comma missing between two strings, `["target" "a,b"]`, is still `expected string` at its key's line, and so is a version-1 list with a quote dropped, `["a,"b"]`, which version 2 refuses before its elements are read, as an array that does not close ([§FS-config-v2.reader.5](FS-config-v2.md#reader5-an-array-may-span-lines)). A malformed element is refused at its key's line, in the shape of [§FS-config.4.3](FS-config.md#43-invalid-config-behavior), with the refusal of that element itself ([§FS-config-v2.reader.3](FS-config-v2.md#reader3-one-setting-one-form)): a version-1 list that ends in a trailing comma, `["a",]`, is `expected string` at its line as before, and so is a version-1 unclosed string such as `["a,b]`, while version 2 takes off the one trailing comma TOML allows, after an element, before the elements are read, from grund 0.22.0, and through 0.21.x took off the whole run of commas before the `]`, warning where the run held a comma with no element before it ([§FS-config-v2.reader.5.commas.window](FS-config-v2.md#reader5commaswindow-through-grund-021x-the-run-of-commas-before-the--is-taken-off-as-before-and-warns)), and refuses an unclosed string as an array that does not close, naming the line the string is still open at the end of: `["a,b]` is `unclosed array (a string is still open at the end of line <n>)` at its key's line <n> ([§FS-config-v2.reader.5](FS-config-v2.md#reader5-an-array-may-span-lines), [§FS-config-v2.reader.5.strings](FS-config-v2.md#reader5strings-a-string-closes-on-the-line-it-opens-on)). A key whose own grammar cannot take a comma refuses it at that key, naming the value, as `[scan] exclude` refuses a `/` ([§FS-config.3.5.16](FS-config.md#3516-an-exclude-entry-containing--is-refused)) — never as a string that is not one.

## 4. Validation and inspection

Beside the readers, the repository publishes `schemas/grund.schema.json`, a JSON Schema (draft-07) of every key of both versions — [§FS-config.3](FS-config.md#3-keys-version-1) and [§FS-config-v2](FS-config-v2.md#fs-config-v2-grund-reads-a-version-2-config-by-concern-with-one-strength-vocabulary-and-fixed-defaults) — with its type, accepted values and default, each table closed to keys it does not name, for editors whose TOML language server reads JSON Schema; the user-facing key reference, `docs/user-facing/configuration.md`, is rendered from it. It is not a second validator: what relates keys to one another or to the tree, such as a `resolve` that needs a `fetch`, stays with `grund config validate` ([§FS-config.4.1](FS-config.md#41-grund-config-validate-path)), and a test holds the schema to the readers — every key it names is accepted, and every committed configuration `config validate` accepts validates against it. `grund config migrate --to 2 [path]` shows the version-2 file that means what a version-1 config means, and with `--write` is the one command that may replace an authored config ([§REQ-no-data-loss.4](../requirements/REQ-no-data-loss.md#4-one-migration-may-replace-the-config)); its contract is [§FS-config-migrate](FS-config-migrate.md#fs-config-migrate-grund-config-migrate---to-2-shows-the-version-2-file-that-means-what-a-version-1-config-means-and-writes-it-only-when-asked-and-only-when-it-does). `grund config explain [target] [path]` says, from the installed binary and in the version the discovered file is read as, what any key accepts and means and what any check code means and what turns it on, while `show` says what this project sets; its contract is [§FS-config-explain](FS-config-explain.md#fs-config-explain-grund-config-explain-answers-from-the-installed-binary-and-in-the-version-the-projects-config-is-read-as-what-any-key-accepts-and-means-and-what-any-check-code-means-and-what-turns-it-on).

### 4.1 `grund config validate [path]`

Loads the config discovered by walking up from `path` (or `.` when omitted), checks the keys, and reports problems. Exits 0 on success, 1 on validation errors — the error in the same `error: <path>:<line>: <message>` shape [§FS-config.4.3](FS-config.md#43-invalid-config-behavior) defines. No tree scan is performed. A redundant config pair at the config root is reported as a `warning:` here too ([§FS-config.1.1](FS-config.md#11-when-one-directory-carries-both), [§FS-check.4.3](FS-check.md#43-redundant-config-pair)); it is a warning, so it does not change the exit code.

#### 4.1.1 At a workspace root

When the discovered config declares `[workspace]` ([§FS-config.3.8](FS-config.md#38-workspace--sub-project-members), [§FS-workspace.2](FS-workspace.md#2-workspace-configuration)), `config validate` also expands `members` and loads every member config the run would load — nested workspaces included ([§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)) — the same launch-time pass `grund check` runs before it scans anything. The first problem, whether a member config that does not load or a `members` entry that cannot be resolved, is reported once in the same `error: <path>:<line>: <message>` line `grund check` prints for it, paths rendered from the workspace root when `[output] relative_paths = true` ([§FS-workspace.5](FS-workspace.md#5-command-scope)), exit 1 ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)). No tree scan is performed. A path inside a member validates that member alone, as `grund check <member>` does ([§FS-workspace.5](FS-workspace.md#5-command-scope)). `config show` is unchanged: it prints the discovered project's effective config ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)).

### 4.2 `grund config show [path]`

Prints the **effective** configuration — defaults merged with the config discovered by walking up from `path` (or `.` when omitted), plus CLI flags — as TOML, and the TOML that comes out loads back to the same effective values it went in with: every rule below about which keys are printed keeps that so. Useful for debugging "why did grund recognize this citation" or "what does my config actually evaluate to." A redundant config pair at the config root is reported as a `warning:` on stderr before the TOML ([§FS-config.1.1](FS-config.md#11-when-one-directory-carries-both), [§FS-check.4.3](FS-check.md#43-redundant-config-pair)), so the answer to "why is this key not taking effect" is on screen next to the effective value.

A v2 config prints in v2 spelling, and what it prints loads back as v2 to the same effective values. A v1 config prints the bytes it printed before v2 existed ([§REQ-backwards-compatibility.1](../requirements/REQ-backwards-compatibility.md#1-what-is-covered)). `show` teaches the configuration in effect. The file `grund init` writes holds only what differs from a default ([§FS-init.2.4.3](FS-init.md#243-every-written-key-differs-from-its-default)), and `show` prints the settings in effect beside it, defaults included, in the file's own version: what a user reads to see what applies, and what they copy a key from to change it ([§DF-v2-onboarding](../decisions/functional/DF-v2-onboarding.md#df-v2-onboarding-grund-init-writes-a-minimal-version-2-file-and-grund-config-show-does-the-teaching)). An opt-in table that is off is not in effect and prints nothing, and the text that sends a reader to `show` says where it is answered instead ([§FS-config.4.2.3.1](FS-config.md#4231-what-sends-a-reader-to-show-names-explain)). What a key means and what values it accepts is `grund config explain`'s answer, not `show`'s ([§FS-config-explain.3](FS-config-explain.md#3-a-keys-answer), [§DF-config-explain.2.2](../decisions/functional/DF-config-explain.md#22-it-narrows-config-show-is-the-teaching-surface)). The order it prints in is [§FS-config.4.2.4](FS-config.md#424-the-order-show-prints-in), and the version it prints in is [§FS-config.4.2.5](FS-config.md#425-a-v1-display-never-takes-a-v2-default).


#### 4.2.1 `[[kinds]]` rows print what they do not inherit

Every `[[kinds]]` entry is printed under the canonical `kind` key, and `citable` is printed **only where it is `false`**: absence *is* `citable = true`. `require_grounding` and `grounding_level` follow the same rule one scope down: a row prints either key only where its effective value differs from the effective global, which is printed under `[reference]` ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)). A row that inherits both prints neither.

#### 4.2.2 `[reference]` always prints `shorthand`

The `[reference]` table always prints the effective `shorthand` policy. An absent key therefore appears as `shorthand = "canonical"`, while an opted-in project appears as `shorthand = "accepted"`; either emitted form loads back to the same effective policy ([§FS-config.3.1](FS-config.md#31-reference--citation-form)).

#### 4.2.3 `named_sections` and `values` print only when enabled

The `[id]` table prints `named_sections = true` only when enabled. Absent and explicit `false` configurations therefore retain the previous `config show` bytes; an enabled repository exposes the opt-in that explains its named heading and citation grammar. An enabled value kind likewise prints `values = true` on its row and a disabled one prints no value key ([§FS-config.3.4.9](FS-config.md#349-values--first-class-value-declarations)).

A setting `show` leaves out is still answered by `grund config explain` ([§FS-config-explain.3](FS-config-explain.md#3-a-keys-answer)).

##### 4.2.3.1 What sends a reader to `show` names `explain`

`show` prints what applies, so `named_sections` and `values` are not all it leaves out: an opt-in table that is off prints nothing either, such as `[schema.leads.words]` while no lead is capped, or a `[rules.citations.<KIND>]` table the file does not write. No text written for a reader may therefore claim that `show` prints all of grund's settings. Each place that sends a reader to `show` names `grund config explain` for what `show` leaves out, and says it in this one sentence wherever the place's own subject allows:

> `grund config show` prints what applies, `grund config explain` every setting.

The places are the comment that opens the `grund.toml` `grund init` writes, whose bytes are [§FS-init.2.4.5](FS-init.md#245-one-comment-is-kept)'s; the help pages of `grund init` and `grund config` ([§FS-cli.2.3](FS-cli.md#23-a-subcommands-help-page)), where `show`'s own entry says in its words that it leaves out an opt-in table that is off and that `grund config explain` answers for every setting; the configuration questions `grund guide config` prints ([§FS-init.5.5](FS-init.md#55-the-core-and-its-sections)); and the guides, examples and README that say what `show` prints or present it as where a reader finds a setting. `show`'s own output says none of this, so its bytes stay those [§FS-config.4.2.3](FS-config.md#423-named_sections-and-values-print-only-when-enabled) keeps.

#### 4.2.4 The order `show` prints in

The output's order is fixed by the config alone, so the same file always shows the same bytes ([§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)). A v1 config prints its tables in one fixed order, top-level keys first, with its `[[kinds]]` rows in the order the file writes them; that order is the bytes v1 printed before v2 existed and does not change. A v2 config prints the envelope keys, then `[schema]` and its tables, then `[rules]` and its tables, then `[presentation]` and its tables. Every table keyed by a row name — `[schema.kinds.<NAME>]`, `[rules.citations.<NAME>]` and `[presentation.kinds.<NAME>]` — prints in the order the file writes its `[schema.kinds]` rows ([§FS-config-v2.reader.4](FS-config-v2.md#reader4-order-and-location-are-kept)), never alphabetically. A citation table keyed by a name that is no row, such as `code` when the file writes no `code` row, prints after every row's, in the order the file writes it.

#### 4.2.5 A v1 display never takes a v2 default

A v1 config shows in v1 spelling with v1's defaults, whatever version `grund init` now writes. A key the v1 file omits prints at its v1 default ([§FS-config.3](FS-config.md#3-keys-version-1)), never at the v2 default for the same setting ([§FS-config-v2.defaults.1](FS-config-v2.md#defaults1-what-v2-fixes)): a v1 file with no `[id]` shows `format = "{kind}-{number}-{slug}"`, not `{kind}-{slug}`, and `[scan] include` at v1's list. Showing a version-1 file never migrates it; that is `grund config migrate`'s, run on purpose ([§FS-config-migrate](FS-config-migrate.md#fs-config-migrate-grund-config-migrate---to-2-shows-the-version-2-file-that-means-what-a-version-1-config-means-and-writes-it-only-when-asked-and-only-when-it-does)).

### 4.3 Invalid config behavior

A `grund.toml` that fails validation causes every `grund` subcommand to exit with code 2 (code 1 for `grund config validate` itself, [§FS-config.4.1](FS-config.md#41-grund-config-validate-path); no output and code 0 for the hidden `complete` helper, [§FS-completions.2](FS-completions.md#2-internal-dynamic-helper); and an answer from `grund config explain`, which reads only the file's version, so that a file whose version cannot be read, an unsupported one included, is the only one it refuses, [§FS-config-explain.2](FS-config-explain.md#2-which-version-answers)) and a single error message pointing at the first problem, in the form `error: <path>:<line>: <message>` on stderr ([§FS-errors.2.2](FS-errors.md#22-cli-level-message), [§FS-check.2.1.1](FS-check.md#211-cli-level-messages)) — the `error:` prefix marks it a CLI-level failure, the `<path>:<line>:` inside the text points at the offending key or line. That includes an invalid `values` row ([§FS-config.3.4.9](FS-config.md#349-values--first-class-value-declarations)) and, since grund 0.16.0 and for every command that loads the workspace, a `[workspace] members` line that swallows its block's own scan ([§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan)); source-content validity is deferred to the scan and checker. Subsequent problems are not reported until the first is fixed — this avoids cascading errors that obscure the root cause.

For concrete stderr examples and the distinction between `config validate` exit `1` and config-blocked command exit `2`, see [§FS-output-shapes.6](FS-output-shapes.md#6-cli-and-config-failures).

In `check --watch`, invalid config ends a run with those same error bytes and status `2`, while the process remains resident for repair ([§FS-check.6.3.1](FS-check.md#631-recoverable-runs)). Discovery coverage survives an invalid initial config; later failures retain the last usable inventory and anchors. This exception changes no one-shot command's refusal.

Under v2, a clause the format defines but this grund does not execute is refused at its authored line, and an omitted default this grund cannot execute is refused at the `grund_config_version` line ([§FS-config-v2.rollout](FS-config-v2.md#rollout-what-this-grund-executes-and-what-it-refuses), [§FS-config-v2.defaults.2](FS-config-v2.md#defaults2-the-language-set-is-fixed-before-it-is-executed)). Neither is accepted and ignored.


## 5. Schema versioning

The TOML file may include a top-level `grund_config_version = N`. This grund reads versions **1** and **2**, each by its own reader. Future incompatible schema changes increment this; grund refuses to load a config whose version is greater than the grund binary's known maximum, with an error suggesting an upgrade: `unsupported config version `<N>` (this grund understands grund_config_version = 1 or 2; upgrade grund if the config is newer)`.

Which reader reads a project is decided by its own file. An explicit `2` selects the v2 reader ([§FS-config-v2](FS-config-v2.md#fs-config-v2-grund-reads-a-version-2-config-by-concern-with-one-strength-vocabulary-and-fixed-defaults)). An omitted version and an explicit `1` select v1, and so does a project with no config file at all: discovery that finds none, and a workspace member with no config of its own. Each of those gets the fixed v1 defaults, so zero-config keeps every default it had ([§GOAL-zero-config](../goals.md#goal-zero-config-works-on-any-conformant-tree)). Only a newly written explicit `2` opts into v2's defaults and its stricter validation. A workspace member is read by the reader its own file selects, so a v2 root may list v1 members, and the reverse. A new key is not a new version ([§FS-config.5.1](FS-config.md#51-new-keys-are-not-a-new-version)), and an older version keeps its meaning ([§FS-config.5.2](FS-config.md#52-every-older-version-keeps-its-meaning)).

### 5.1 New keys are not a new version

The version tracks **incompatible** changes to the meaning of existing keys, not the arrival of new ones. Adding an optional table or key — `[workspace]`, `[citations]`, `[[kinds]].values` ([§FS-config.3.4.9](FS-config.md#349-values--first-class-value-declarations)), `[[kinds]].format` / `resolve` / `fetch` ([§FS-config.3.4.10](FS-config.md#3410-format-resolve-and-fetch--external-snapshot-kinds)), a future `anchor_format` profile — is additive and does not bump the version, because a config that uses it is only ever written for a binary that understands it, and an older binary meeting it fails loudly and locatably through the unknown-section / unknown-key rejection ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)) rather than silently misreading it. The safety net for the forward direction is the closed section and key allow-list, not the version integer.

### 5.2 Every older version keeps its meaning

In the backward direction the gate is what [§REQ-backwards-compatibility.1](../requirements/REQ-backwards-compatibility.md#1-what-is-covered) rests on: a binary that supports version `N` keeps interpreting every version `≤ N` under the semantics that version shipped with, so upgrading the binary never re-reads a config it already understood.

That holds of the records, not only of the reports. A v1 state that v2 cannot spell, such as a positive citation default or `strict = false`, keeps its meaning in the records the engine reads, and the tests that hold it read the records ([§FS-config-v2.mapping](FS-config-v2.md#mapping-what-the-records-keep-apart)). The v1 reader keeps v1's validation too: v2's stricter rules never refuse a file v1 accepts.

### 5.3 A version 1 config is cautioned and never retired

A config file read as version 1, whether it writes `grund_config_version = 1` or writes no version at all, earns one warning naming the file and the `grund config migrate --to 2` that moves it ([§FS-check.checks.config-version-1](FS-check.md#checksconfig-version-1-a-config-file-read-as-version-1)). The caution is the deprecation path's warning without a deadline ([§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)): it never becomes an error or a refusal, no release is named in which it would, and the file keeps its meaning under [§FS-config.5.2](FS-config.md#52-every-older-version-keeps-its-meaning). It keys on a *file*: a project with no config file is read as v1 too, but wrote nothing to migrate, so it earns no caution ([§GOAL-zero-config](../goals.md#goal-zero-config-works-on-any-conformant-tree)).


## 6. What is NOT configured here

Per [§GOAL-friendliness-first.2](../goals.md#2-what-this-rules-out), the following are deliberately **not** configurable, to avoid the trap of every grund repo behaving differently in surprising ways:

- The set of severities (only `error` and `warning` exist); a suggestion is not a third one ([§FS-config.6.1](FS-config.md#61-suggestions-are-not-a-third-severity)).
- The exit code mapping (`0`/`1`/`2` per [§FS-cli.5](FS-cli.md#5-exit-code-mapping-is-fixed)).
- The ordering of the report (always deterministic).
- Anything that would let two correctly-configured grund installs disagree on whether a given repo is well-formed ([§GOAL-configurable.2](../goals.md#2-what-is-not-configurable)).
- The local conversation citation *preference*: it follows the user's TUI setup and is installed through `grund integrations --write` ([§FS-integrations.4.3](FS-integrations.md#43-user-preference-and-global-agent-instructions)). What a repository may commit instead is [§FS-config.6.2](FS-config.md#62-the-repositorys-conversation-opinion).

### 6.1 Suggestions are not a third severity

The `should` and `should-not` citation-direction suggestions ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)) do **not** add a third severity: they are carried on a separate non-severity advisory channel ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)), so this frozen `{error, warning}` set stays exactly two.

v2 keeps the set at two. `warn` and `warn-not` reach the warning channel, and `should` and `should-not` the suggestion channel ([§FS-config-v2.rules.strengths](FS-config-v2.md#rulesstrengths-one-strength-vocabulary-two-severities)).


### 6.2 The repository's conversation opinion

A repository may commit the `link`-only *opinion* via `[reference] conversation` ([§FS-config.3.1](FS-config.md#31-reference--citation-form), [§DF-repo-conversation-opinion](../decisions/functional/DF-repo-conversation-opinion.md#df-repo-conversation-opinion-repositories-may-commit-a-link-only-conversation-rendering-opinion)), the fallback for machines that never stated a preference; an explicitly recorded user preference wins over it ([§DF-repo-conversation-opinion.2.3](../decisions/functional/DF-repo-conversation-opinion.md#23-precedence)). Repository-web guidance stays fixed in the generated agent entrypoint ([§FS-init.2.3.6](FS-init.md#236-clickable-citations)).

## v2: The version-2 format

Version 2 spells the same project by concern, with one strength vocabulary and fixed defaults. It is specified in its own declaration, [§FS-config-v2](FS-config-v2.md#fs-config-v2-grund-reads-a-version-2-config-by-concern-with-one-strength-vocabulary-and-fixed-defaults), so every section above keeps its ID and its meaning. A section here that says what a key means is about v1 unless it names v2.
