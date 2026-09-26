# FS-config: grund reads a TOML config file found by walking up

`grund` is zero-config out of the box ([§GOAL-zero-config](../goals.md#goal-zero-config-works-on-any-conformant-tree)) and fully configurable when a project's conventions diverge ([§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable)). This spec defines the contract: where the config lives, what it contains, what it overrides, and how malformed configs are reported.

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, home, citable, body, section, coordinate,
lead, index, catalog), [§FS-terms.terms.2](FS-terms.md#terms2-citations) (marker, citation, qualified citation, shorthand,
citation site), [§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (source declaration, stub, doc-comment, note),
[§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope, config root, workspace, member, alias), [§FS-terms.terms.5](FS-terms.md#terms5-findings)
(finding, severity, suggestion), [§FS-terms.terms.6](FS-terms.md#terms6-rules-and-directions) (direction, level, rule, grounded), and
[§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations) (value, snapshot).

- **place** — Where a per-place key is written, and therefore what it governs: on a `[[kinds]]`
  row it says what one place does, at the top of the file what every place does.
- **homeless kind** — The row with neither `folder` nor `file`: the complement of every home,
  whose default name is `code`.
- **citing kind** — The kind a citation is attributed to — the kind of the home its file sits
  in, or the homeless kind when no home claims it.
- **CLI base** — The directory a report's paths are rendered against under `relative_paths =
  false`, in place of the config root.

## principle: A setting written at a narrower scope wins

Every setting `grund` reads from a repository is written at one of three **committed scopes**, from widest to narrowest: the **built-in default** the tool supplies ([§FS-config.3](FS-config.md#3-schema)), the **project** — the top-level tables of `grund.toml` — and the **kind**, written either as a `[[kinds]]` row ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)) or as a `[citations.<KIND>]` table ([§FS-config.3.9.4](FS-config.md#394-defaults-and-precedence)), which are two spellings of writing at the one scope. This chapter states once how those scopes relate, so that a key's own point may say what the key means and leave how it resolves to one place. It describes what `grund` already does: it is the general form of sentences this specification carries one at a time ([§FS-config.3.4.8.3](FS-config.md#3483-precedence-is-row-over-global), [§FS-config.3.4.7.2](FS-config.md#3472-not-walked-however-the-walk-arrives), [§FS-config.4.2.1](FS-config.md#421-kinds-rows-print-what-they-do-not-inherit)), and it adds no behavior and no key.

It serves [§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable) by making *can I set this per kind?* one read instead of a hunt through ten table sections. Every clause below is conditional on a setting being **admitted** at more than one committed scope ([§FS-config.principle.admission](FS-config.md#principleadmission-the-relation-reaches-only-scopes-that-admit-the-setting)); none of them claims that every default is overridable everywhere, and [§FS-config.principle.inventory](FS-config.md#principleinventory-the-settings-admitted-at-both-the-project-and-the-kind-scope) is the complete list of the settings this chapter's override clauses reach at all.

### principle.rungs: The committed scopes are a relation, not a closed list

Where a setting is admitted at more than one committed scope, the value written at the **narrower** scope is the effective one for everything that scope governs, and the wider value stands wherever the narrower scope is silent. Narrowness is the ordering above — a `[[kinds]]` row is narrower than the project's tables, which are narrower than the built-in defaults — and it is stated as a relation rather than as an enumeration of three: a scope introduced later, whether finer than a kind or between the project and the kind, takes its place in the same relation without this chapter being rewritten. No such scope exists today, and adding one would be additive surface rather than a new schema version ([§FS-config.5.1](FS-config.md#51-new-keys-are-not-a-new-version)).

### principle.unit: One leaf key overrides, and one value is whole

Two units, and they are not the same one. What overrides is a single **leaf key**: writing one key at a narrower scope leaves every other key that scope inherits exactly as it was, which is why a `[reference]` table naming one key does not discard the rest of the `[reference]` defaults — every key is optional and an omitted key takes the default value ([§FS-config.3](FS-config.md#3-schema)). What is overridden is the **whole value**, an array included: a key whose value is a list replaces the inherited list rather than merging into it. Writing `[[kinds]]` at all replaces the built-in kind list entirely ([§FS-config.3.4.4](FS-config.md#344-the-default-kinds)) — that is this whole-value rule applied to the list of rows, not a second rule beside it.

### principle.admission: The relation reaches only scopes that admit the setting

A scope **admits** a fixed set of keys, and the relation above holds only between the scopes that admit the same setting. A key written at a scope that does not admit it is not a weaker override: it is not a key there at all, and it is refused as an unknown key pointing at its line ([§FS-config.3](FS-config.md#3-schema), [§FS-config.4.3](FS-config.md#43-invalid-config-behavior)). So `inline_style` under `[reference]` and `inline_style` on a `[[kinds]]` row are not a wide and a narrow spelling of one setting ([§FS-config.3.1.8](FS-config.md#318-inline_style-and-the-note-budgets)) — the second does not exist. A key's own point states its scopes only where it is admitted at more than one; where it is admitted at one, the table it is documented under is that statement. This chapter says where a key may be written, and what is not a key at all stays [§FS-config.6](FS-config.md#6-what-is-not-configured-here)'s: a thing named there has no scopes, because it is not a setting.

### principle.cli: A CLI input enters at the scope its flag spells

A command-line input is not a fourth scope that wins over all three. It enters the relation at the scope its flag spells and resolves from there like any value written at that scope. `grund check --require-grounding` spells the **project** default, so an explicit `require_grounding = false` on a row still wins over the flag ([§FS-config.3.4.8.3](FS-config.md#3483-precedence-is-row-over-global)): the flag and the key are one knob, and the row's word is the more specific one.

### principle.boundary: The project is the widest committed scope

Scopes stop at the project. A workspace member is configured by its own `grund.toml`, or by the built-in defaults where it has none, and no section of a member's config inherits from the workspace root's ([§FS-config.3.9.3](FS-config.md#393-namespace-matching), [§FS-workspace.2](FS-workspace.md#2-workspace-configuration)). A root and a member are therefore a **boundary rather than a rung**: there is no precedence between them, because neither is ever consulted for the other's values. This is a statement about which config governs a member's own files, and not about which config a command discovers from a given directory — that is [§FS-config.1](FS-config.md#1-file-location-and-discovery)'s, and the two answer different questions.

### principle.declared-rows: A declared row does not inherit from a built-in one

A `[[kinds]]` row a project writes starts from the schema defaults rather than from the built-in row of the same name: writing `[[kinds]]` replaces the built-in list entirely ([§FS-config.principle.unit](FS-config.md#principleunit-one-leaf-key-overrides-and-one-value-is-whole)), so there is no wider row left to inherit from, and a declared `FS` row takes the schema's own default for `citable` rather than the built-in row's and has no home at all until it names one. Exactly one built-in attribute still reaches a declared row, and it is keyed on the kind's **name** rather than on a scope: `index` ([§FS-config.3.4.2.4](FS-config.md#3424-the-default-is-per-kind-name)), applied only where the row left `index` unset, names a `folder`, and is citable. The built-in home, file, title, and `citable` of a same-named built-in kind reach nothing a project declares.

### principle.exceptions: Two shapes that look like rungs and are not

Two places in this schema resemble the relation above without being it, and each keeps the wording it already has. The first is the top term of [§FS-config.3.9.4](FS-config.md#394-defaults-and-precedence)'s ladder — an explicit target list over a `default` — which is specificity of **target match** between keys each admitted at one scope, not a narrower scope overriding a wider one; its lower two terms are an ordinary instance and appear in the inventory below. The second is [§FS-config.3.4.2.4](FS-config.md#3424-the-default-is-per-kind-name)'s `index`, which is **produced from the kind name** at the default-producing end rather than resolved between two written values; overriding it on a row is the ordinary instance, bounded by [§FS-config.principle.declared-rows](FS-config.md#principledeclared-rows-a-declared-row-does-not-inherit-from-a-built-in-one). Neither is an exception to the rule: each is a different shape the rule does not describe.

### principle.install-local: The relation governs committed repository state only

Every scope above is something a repository commits. The **install-local** axis lies outside it: the user's conversation citation preference and its per-agent partial are machine state installed by `grund integrations --write`, and their precedence is stated where they live ([§FS-integrations.4.3](FS-integrations.md#43-user-preference-and-global-agent-instructions), [§FS-integrations.4.4](FS-integrations.md#44-per-agent-overrides)) rather than restated here. Naming it here is what keeps the boundary visible from the chapter that states the relation, and the line it does not cross is unmoved: no machine-local value may change whether a repository is well-formed ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)). For the author of a new key the order is **classify first, verify by reach** — decide whether the key carries committed repository semantics or install-local presentation, then check that its reach agrees. Reach alone does not classify: inert `[output] color` ([§FS-config.3.6](FS-config.md#36-output--report-format)) changes no verdict and is committed repository state all the same.

### principle.inventory: The settings admitted at both the project and the kind scope

Four settings are admitted at both the project scope and the kind scope, and for the schema of [§FS-config.3](FS-config.md#3-schema) this list is complete:

| Project scope | Kind scope | Stated at |
|---|---|---|
| `[reference] require_grounding` | a row's `require_grounding` | [§FS-config.3.4.8.3](FS-config.md#3483-precedence-is-row-over-global) |
| `[reference] grounding_level` | a row's `grounding_level` | [§FS-config.3.4.8.3](FS-config.md#3483-precedence-is-row-over-global) |
| `[id] format` | a row's `format` | [§FS-config.3.4.10.1](FS-config.md#34101-format) |
| `[citations] default` | `[citations.<KIND>] default` | [§FS-config.3.9.4](FS-config.md#394-defaults-and-precedence) |

Every other key of [§FS-config.3](FS-config.md#3-schema) is admitted at one committed scope only, so [§FS-config.principle.rungs](FS-config.md#principlerungs-the-committed-scopes-are-a-relation-not-a-closed-list) never reaches it and its table section is the whole statement of where it may be written. The list is derived from the parse sites and the resolution sites rather than read off this prose, and the derivation is recorded with the decision behind this chapter ([§DF-config-scope-override.2.2](../decisions/functional/DF-config-scope-override.md#22-the-inventory-is-derived-in-two-stages)), so that a reader re-runs it instead of trusting it. A key that gains a second scope is added here in the same change, and that is additive surface ([§FS-config.5.1](FS-config.md#51-new-keys-are-not-a-new-version)).

## requirements: What the config contract holds to

Eight requirements on the schema as a whole, each carrying a **mark**: *realized* where `grund` behaves this way today and a point of this specification says so; *directional* where it binds what is written from here on rather than describing everything already written; *deferred* where a named follow-up owes it. The mark is part of the requirement — a direction is never a statement about how `grund` behaves today, and reading one as the other is the mistake this chapter exists to prevent.

There is no ninth. *Every key appears in a user-facing reference* is a property of this repository's documentation rather than of what `grund` does, and `FS` states behavior; its behavioral half — that a generated `grund.toml` carries exactly this schema, no extra keys and none missing — is already required, and more strongly, by [§FS-init.2.4.3](FS-init.md#243-every-written-key-is-the-default).

### requirements.1: Every default is overridable where it has a meaning — directional

The obligation this places on a key added from here on falls on the admission decision itself rather than on any further clause at the key's own point: it should be admitted at every committed scope where its value would govern something that scope tells apart, so that a key which ends up admitted at one scope alone is so by that judgement and not for want of the question being asked. A setting admitted at more than one committed scope is overridable at each of them, and where a setting is admitted at one scope only the table section it is written under is the whole statement of that, rather than a scope clause added to its own point ([§FS-config.principle.admission](FS-config.md#principleadmission-the-relation-reaches-only-scopes-that-admit-the-setting)). This binds keys added from here on; it is not a claim about every key already written, and what it reaches today is exactly [§FS-config.principle.inventory](FS-config.md#principleinventory-the-settings-admitted-at-both-the-project-and-the-kind-scope), where a key that gains a second scope is recorded in the same change — the obligation is discharged there and not key by key. Deliberate refusals stay refusals — everything [§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree) rules out. Serves [§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable).

### requirements.2: Zero config works — realized

Every key is optional, and no config file anywhere up the walk means the canonical defaults of this specification rather than a refusal ([§FS-config.1](FS-config.md#1-file-location-and-discovery), [§FS-config.3](FS-config.md#3-schema)). Serves [§GOAL-zero-config](../goals.md#goal-zero-config-works-on-any-conformant-tree).

### requirements.3: One key means one thing at every scope — directional

A key admitted at two scopes keeps its name, its value domain, and its meaning at both; only its reach changes ([§FS-config.principle.rungs](FS-config.md#principlerungs-the-committed-scopes-are-a-relation-not-a-closed-list)). Two keys that share a spelling across scopes without sharing a setting are not an instance of this and are not made into one: `[scan]` and a row's `scan` are two settings whose reconciliation is stated where they are ([§FS-config.3.4.7.2](FS-config.md#3472-not-walked-however-the-walk-arrives)), and `[id] format` is not `[output] format`. Directional for the same reason as [§FS-config.requirements.1](FS-config.md#requirements1-every-default-is-overridable-where-it-has-a-meaning--directional) — it binds the next key, and renaming one already written is a compatibility question of its own ([§FS-config.5.2](FS-config.md#52-every-older-version-keeps-its-meaning)).

### requirements.4: Every value has exactly one resolution — realized

The effective value of any key at any scope is a function of the committed config files and the CLI inputs alone. It never depends on the order two keys were written in, on an environment variable, or on machine state, and no two rules ever both claim one value — which is why [§FS-config.principle.exceptions](FS-config.md#principleexceptions-two-shapes-that-look-like-rungs-and-are-not) names the two shapes that are not rungs rather than leaving them implied. Serves [§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes) and [§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree).

### requirements.5: A mistake in the config fails loudly — realized, one case deferred

An unknown key and an invalid value are errors naming the line, never a silent default ([§FS-config.3](FS-config.md#3-schema), [§FS-config.4.3](FS-config.md#43-invalid-config-behavior)), per [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible). A key written at a scope that does not admit it is refused today as an unknown key, which is loud and locatable but names the wrong mistake; saying *this key is not admitted here* instead is owed and **deferred** to its own change, and nothing in this specification depends on the current wording.

### requirements.6: The effective configuration is inspectable — realized, provenance deferred

`grund config show` prints the effective configuration, and what it prints loads back to the same effective values ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)). The round trip is of the **effective** config, not of the authored text: a key the run resolved to its default prints as that default, and a key that is inert under another key's value is still parsed and still printed ([§FS-config.3.1.8](FS-config.md#318-inline_style-and-the-note-budgets)), so the output is what applies rather than what was typed. Which scope a value came from — default, project, row, or flag — is **deferred**; what the output shows today is one step of it, a row printing a key only where its effective value differs from what it inherits ([§FS-config.4.2.1](FS-config.md#421-kinds-rows-print-what-they-do-not-inherit)).

### requirements.7: An upgrade never changes the meaning of an existing config — realized

A new key, or a new scope for an existing key, is additive and does not bump `grund_config_version` ([§FS-config.5.1](FS-config.md#51-new-keys-are-not-a-new-version)); an incompatible change to the meaning of an existing key does, and a binary refuses a config whose version it does not know ([§FS-config.5](FS-config.md#5-schema-versioning)). Serves [§REQ-backwards-compatibility](../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly).

### requirements.8: A project's meaning is self-contained — realized

The committed config of one project decides that project's verdict: nothing is inherited across a project boundary ([§FS-config.principle.boundary](FS-config.md#principleboundary-the-project-is-the-widest-committed-scope)) and nothing is read from the machine ([§FS-config.principle.install-local](FS-config.md#principleinstall-local-the-relation-governs-committed-repository-state-only)). Serves [§REQ-runs-offline](../requirements/REQ-runs-offline.md#req-runs-offline-verification-never-depends-on-an-external-service) and [§FS-workspace.2](FS-workspace.md#2-workspace-configuration).

## 1. File location and discovery

The config file is named **`grund.toml`** and is discovered at **two locations per directory**: the bare `grund.toml` beside the project's own metadata files, then `.agents/grund.toml`. Discovery walks upward from the path argument — a file argument's parent directory, and the working directory when no path is given — probing both names in that order at every level, and stops at the first directory where either exists — mirroring how `cargo` finds `Cargo.toml`. That directory is the **config root**; relative paths inside the config are resolved against it, never against `.agents/`. One uniform rule at every level: a repository root and a workspace member each pick the form that suits them, and a workspace may mix the two ([§FS-workspace.2](FS-workspace.md#2-workspace-configuration)). Per [§DF-config-file-location](../decisions/functional/DF-config-file-location.md#df-config-file-location-grundtoml-is-discovered-at-two-names-per-directory-and-init-writes-the-bare-one).

If neither name is found anywhere up the walk, `grund` runs with the built-in defaults defined in this spec. The defaults are the canonical `grund` grammar — they are not stored in any file.

### 1.1 When one directory carries both

The bare `grund.toml` wins, and the `.agents/grund.toml` beside it is read by nothing at all. The form `grund init` generates is the form that governs ([§FS-init.2.4](FS-init.md#24-generated-grundtoml)), so a project never has to hold one rule for the file grund writes and a contradicting one for the file grund reads. It is also what a user reaching for a root `grund.toml` means: a repository acquires the pair only when someone deliberately puts a bare file beside an existing `.agents/` one, and the reason to do that is to move to the recommended form ([§FS-config.1.3](FS-config.md#13-the-agents-directory-and-the-recommended-form)).

#### 1.1.1 The pair earns one warning

Because a config `grund` ignores is still a config a user edits, `grund check` reports the pair as a warning naming both files ([§FS-check.4.3](FS-check.md#43-redundant-config-pair)), so the losing file is never silently ignored and a config quietly replaced is reported at the first `check` — which is what makes this order safe to state ([§DF-config-file-location.2.2](../decisions/functional/DF-config-file-location.md#22-the-bare-grundtoml-wins-a-tie-and-check-warns-about-the-pair)). It is a warning and not an error because the pair is the ordinary transient state of a move in either direction: warnings never affect the exit code ([§FS-check.2](FS-check.md#2-outputs)), so a repository mid-migration stays green while the diagnostic stays visible. It is the *only* warning the pair earns: the run read the bare `grund.toml`, the location [§FS-config.1.2](FS-config.md#12-the-agents-location-is-deprecated) deprecates the other one in favour of, so nothing about the config in force is left to deprecate.

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

## 2. Precedence

Two sources decide a setting: the committed `grund.toml` and the CLI inputs of the one run. A value written in `grund.toml` stands over the built-in default ([§FS-config.3](FS-config.md#3-schema)), and a CLI input is not a third source above both — it enters at the scope its flag spells and resolves from there ([§FS-config.principle.cli](FS-config.md#principlecli-a-cli-input-enters-at-the-scope-its-flag-spells)), which is why `grund check --require-grounding` sets the project default and an explicit `require_grounding = false` on a row still wins over it. Both statements are about one leaf key at a time ([§FS-config.principle.unit](FS-config.md#principleunit-one-leaf-key-overrides-and-one-value-is-whole)): a table naming one key leaves every other key of that table at its default, and what a key does override is its whole value, an array included. How the scopes inside the file relate is stated once in [§FS-config.principle](FS-config.md#principle-a-setting-written-at-a-narrower-scope-wins), and this section does not restate it. Per [§DF-config-scope-override](../decisions/functional/DF-config-scope-override.md#df-config-scope-override-the-committed-scopes-are-one-relation-stated-once), which records what this section said before and why it was withdrawn.

## 3. Schema

The config file is TOML. Every key is optional; omitted keys take the default value. Unknown keys are an **error**, not a warning, per [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible) — typos in config files are bugs and grund surfaces them loudly.

The recognized surface is the line-oriented subset that the schema below uses: one `key = value` per line, basic (double-quoted) strings, booleans, integers, and single-line `["…", "…"]` arrays of basic strings; `#` comments; `[table]` and `[[array.of.tables]]` headers. Multi-line arrays, inline `{ … }` tables, and other TOML constructs are not parsed, except for the closed `lead_size_warning` inline table defined in [§FS-config.3.1.2](FS-config.md#312-lead_size_warning--the-oversized-lead-opt-in) — keep each value on one line. A line that does not fit this shape is reported as an error pointing at the offending line, per [§FS-config.4.3](FS-config.md#43-invalid-config-behavior).

Top-level keys:

```toml
grund_config_version = 1
project_name = "Example" # optional metadata written by `grund init`
project_description = "One line describing what this project is for" # optional
```

`project_name` is free-form metadata. When the project participates in a workspace (its own config sets `[workspace]`, or its directory is listed as a member by a parent), `project_name` is also the project's workspace alias — but only when it matches the alias grammar in [§FS-workspace.1](FS-workspace.md#1-citation-syntax), and never for a member listed in `optional_members`, whose alias is the entry's last path segment and whose disagreeing `project_name` is a config error ([§FS-workspace.3](FS-workspace.md#3-aliases)). A `project_name` that is not a valid alias is not a load-time error; it errors loudly at workspace expansion with `invalid workspace project alias <name>`. Outside any workspace context `project_name` is purely metadata: no checker, scanner, formatter, or query behavior depends on it.

`project_description` is a free-form one-line description of the project, chosen in [§DF-workspace-member-descriptions](../decisions/functional/DF-workspace-member-descriptions.md#df-workspace-member-descriptions-member-side-project_description-for-workspace-member-lists). It is presentation metadata only: generated workspace member lists render it next to the project's alias ([§FS-init.2.3.4.15](FS-init.md#23415-workspace-members), [§FS-workspace.3](FS-workspace.md#3-aliases)), and no checker, scanner, formatter, or query behavior depends on it. A value containing a line break (a `\n` or `\r` escape in the TOML string) is a config error at the `project_description` line, reported per [§FS-config.4.3](FS-config.md#43-invalid-config-behavior) — the key exists to feed single-line list bullets, so a multi-line value is a bug surfaced loudly.

### 3.1 `[reference]` — citation form

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

`lead_size_warning` opts this project into the lead-only warning defined by
[§FS-declarations.checks.oversized-lead](FS-declarations.md#checksoversized-lead-oversized-lead-opt-in). Its inline table has
exactly two required fields: `max`, a non-negative integer, and `unit`, one of
the case-sensitive closed set `lines`, `words`, or `bytes`. Missing, extra, or
duplicate fields and any other unit — including `tokens` — are load-time config
errors ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)). There is no severity field: an enabled finding is always a
warning and cannot change `check`'s exit status. The key is absent by default
and omitted from a fresh `grund init` scaffold, so an unconfigured repository's
check output stays byte-identical. `grund config show` omits the key when absent
and otherwise prints its canonical inline form,
`lead_size_warning = { max = <N>, unit = "<unit>" }`.

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

`require_grounding` and `grounding_level` are the two keys of this section admitted at both the project scope and the kind scope: each may also be written on a `[[kinds]]` row, and the row is the narrower scope ([§FS-config.principle.rungs](FS-config.md#principlerungs-the-committed-scopes-are-a-relation-not-a-closed-list), [§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)). Written here they say what every place does; written on a row they say what one place does. They are an instance of that relation rather than an exception to this table: every other key of this section is admitted at the project scope alone, which is what [§FS-config.principle.admission](FS-config.md#principleadmission-the-relation-reaches-only-scopes-that-admit-the-setting) says of a key whose point names no second scope, and [§FS-config.principle.inventory](FS-config.md#principleinventory-the-settings-admitted-at-both-the-project-and-the-kind-scope) carries the complete list. `grounding_level` names the unit inside each governed file — `1`, the default, is the file, which is the unit every config had before the key existed. It is inert, and a config error, where nothing turns grounding on ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)).

#### 3.1.8 `inline_style` and the note budgets

`inline_style`, the three budget keys (`inline_note_suggested_lines`, `inline_note_max_lines`, `inline_note_max_columns`), and `warn_on_suggested` govern the shape of inline citations in code comments — whether a `§<ID>` token may be accompanied by a short rationale, and how long that rationale may run. The budgets and the style bound *inline* comments only; a doc comment is documentation and lies outside all of them, so a citation inside one is checked for everything except its shape ([§FS-inline-citation-style.1.1](FS-inline-citation-style.md#11-doc-comments-are-not-sites)). The full contract — modes, enforcement, agent-facing rendering — lives in [§FS-inline-citation-style](FS-inline-citation-style.md#fs-inline-citation-style-configurable-shape-of-inline-code-comment-citations). Load-time invariant: `inline_note_suggested_lines ≤ inline_note_max_lines`, and `warn_on_suggested` is a boolean; any other value is a load-time error ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)). Under `inline_style = "citation-only"` the three budget keys are inert (no note is ever permitted), but they are still parsed and printed by `grund config show` — the file is the canonical machine-readable form.

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

No ID the grammar can build may contain a `/`, and what that forbids depends on how the key reaches an ID. `format` and a `[[kinds]]` `kind` name ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)) contribute literal text — a citable kind's name is the leading component of every ID in it — so neither may carry the character: a `/` in the key is a `/` in the ID. `number_pattern` and `slug_pattern` are regexes, and the rule asks what they **match**, not what they spell. A pattern with no `/` in its text may produce one freely (`[^.[:space:]]+`, `.+`, `[^[:space:]]+`) and is rejected; a pattern that names the character to *exclude* it (`[^/.]+`) can never produce one and loads. A `/` belongs to the citation namespace and never to an ID — a qualified citation splits on its **last** `/`, and every command that takes an `<alias>/<ID>` argument splits it the same way ([§FS-workspace.1](FS-workspace.md#1-citation-syntax)). So a grammar permitting `FS-a/b` would declare and resolve an ID that grund cannot accept back as a query, and would make the alias-path boundary depend on which project's grammar the reader had in mind. Rejected on load at the offending key's line, like the regex check of [§FS-config.3.2.4](FS-config.md#324-each-pattern-compiles-on-its-own); the message says *must not contain* for a literal key and *must not match* for a pattern, since only one of those is a question about the key's text.

#### 3.2.4 Each pattern compiles on its own

`number_pattern` and `slug_pattern` must each be a valid regex **on their own**, not merely valid once spliced into the ID pattern. Two that balance only against each other — `number_pattern = "("` with `slug_pattern = "a)"` — would compile as one ID pattern and then fall apart the moment grund derives a narrower pattern from a subset of the format's components, which is what the number-only shorthand does ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)). Such a config is rejected on load with the underlying regex error, rather than accepted and failed later.

#### 3.2.5 Off-grammar declarations stay readable

The effective format is the **authoring and conformance grammar**, not a reason
to make a persisted declaration unreadable. A heading in declaration position
whose exact token starts with a configured citable kind, ends at the
declaration colon, and carries no `/` ([§FS-config.3.2.3](FS-config.md#323-no-id-contains-a-)) is retained in that project's catalog
even when the token does not match the kind's effective format. Its exact
written spelling, body,
sections, and location remain available to readers, and it earns the
`declaration-near-miss` finding ([§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss)). Exact marker-prefixed candidates
that normal grammar rejects become citations only when that same project's
catalog contains an exact declaration spelling; an unmarked candidate or a
marked candidate with no exact declaration gains no compatibility meaning.
This catalog-backed boundary is decided in
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

#### 3.2.7 `named_sections` — the gate for section handles

`named_sections` is an absent-by-default Boolean gate for explicit section handles. When absent or `false`, section scanning, citation recognition, queries, formatting, completion, LSP behavior, and operational output remain the numeric-only behavior of earlier configurations. When `true`, a named component has the fixed, configuration-independent grammar `[a-z][a-z0-9-]*`; it is not derived from `slug_pattern` or from the displayed heading title. `grund init` writes the teaching default `named_sections = false` ([§FS-init.2.4](FS-init.md#24-generated-grundtoml)). Unknown values are invalid config.

### 3.3 Section paths — arbitrary nesting depth

Section coordinates are **dotted paths of arbitrary depth**. There is no maximum nesting level. Purely numeric paths retain their existing heading form and behavior:

```
§FS-check.3
§FS-check.3.1
§FS-check.3.1.2
§FS-check.3.1.2.7.4
```

Section depth in the citation must match a heading at that depth in the declaration. The scanner records every citable heading inside a declaration body and validates citations against the recorded set, so a project that wants four-deep numeric nesting (`## 1.`, `### 1.1`, `#### 1.1.1`, `##### 1.1.1.1`) is supported with no config changes — the dotted path simply grows.

#### 3.3.1 Named components

With `named_sections = true` ([§FS-config.3.2.7](FS-config.md#327-named_sections--the-gate-for-section-handles)), any path containing a named component uses the explicit colon form `<complete-path>: <title>`. The path is written in full at every depth; the title declares nothing and may change without changing the coordinate:

```markdown
## goals: Goals
### goals.performance: Performance
### goals.3: Third ordered goal
```

Named components may occur at every depth, so all-name paths such as `goals.performance.latency` are legal. A numeric component may follow a named prefix (`goals.3`), knowingly retaining positional behavior beneath that stable handle. A named component may not follow a numeric component: `1.goals` is reserved and is neither a named heading nor an alias for section `1`. Purely numeric headings keep their existing optional trailing period and title syntax; the colon form is mandatory whenever any component is named. There is no inferred title slug, dual numeric/name address, rename map, or automatic migration.

#### 3.3.2 `section_heading_levels` — heading depth against path depth

`section_heading_levels` controls how the Markdown heading depth must line up with the dotted section path. The default, `"strict"`, requires the heading level to equal the declaration heading level plus the number of path components, named or numeric, so under an H1 declaration `## 1.1 …` and `## goals.performance: …` are `section heading level mismatch` errors in `grund check` ([§FS-declarations.checks.section-heading-level](FS-declarations.md#checkssection-heading-level-section-heading-level-mismatch)). `"warn"` reports the same mismatch as a warning, so CI can stay green while a repo migrates. `"loose"` preserves the historical depth behavior: any deeper heading can declare a syntactically legal complete path. Unknown values are invalid config.

The mode does not govern an ATX heading with no coordinate: inside a Markdown declaration body, every deeper ATX heading must instead be a declaration or a recognized numeric or enabled named section. [§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading) reports one that is neither and names the headings the rule leaves alone; there is no severity or opt-out key for this project-wide rule ([§DF-unmarked-markdown-headings](../decisions/functional/DF-unmarked-markdown-headings.md#df-unmarked-markdown-headings-in-body-markdown-atx-headings-participate-in-the-knowledge-graph)).

#### 3.3.3 Every prefix of a name-bearing path is recorded

Every proper prefix of a name-bearing path must itself be recorded in the same declaration. Thus `### goals.performance: …` requires `goals`, while `### missing.performance: …` is an orphan even if its Markdown placement looks nested under some other heading ([§FS-declarations.checks.orphan-section](FS-declarations.md#checksorphan-section-orphan-name-bearing-section-path)). The invariant applies to name-bearing paths only; purely numeric paths preserve their historical behavior.

#### 3.3.4 The outer section separator

The default `section_separator` is `.`. Projects that prefer `:` (`<§>FS-check:3.1.2`) or `#` (`RFC-42#3.1.2`) override it; the dotted **components** stay separated by `.` regardless of the outer separator. This split keeps the section grammar regular at any depth.

### 3.4 `[[kinds]]` — recognized kinds

One `[[kinds]]` table per kind. `kind` is its name — mandatory, and the handle everything else keys on: `[citations.<kind>]` ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)), `grund list --kind`, and, for a kind that declares IDs, the literal prefix of every ID in it.

A kind is either *multi-file* (`folder = "<dir>"`) — each declaration is the H1 of its own file under `<dir>` — or *single-file* (`file = "<path>"`) — every declaration of the kind is a heading inside that one document — an H2 by convention, and the H1 where the file holds the kind's single declaration ([§FS-config.3.4.4](FS-config.md#344-the-default-kinds)). Setting both `folder` and `file` on the same kind is invalid; setting neither leaves the kind with no configured home. What a home means to `grund id` and to the checker is [§FS-config.3.4.11](FS-config.md#3411-what-a-home-is-used-for).

Every configured home is also **in the scan scope by construction**, whether or not `[scan] include` names it ([§FS-config.3.5.8](FS-config.md#358-every-configured-kind-home-is-walked)), except one marked `scan = false`, which is listed and not walked ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-walked)).

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

`citable` is additive and does not move `grund_config_version` ([§FS-config.5](FS-config.md#5-schema-versioning)). A non-citable kind whose files are not this repository's to read — content that ships verbatim — sets `scan = false` as well ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-walked)).

##### 3.4.1.1 What a non-citable kind keeps

- **A home** — `folder` or `file` — wherever the kind is a *place*. Leaving both out is not an omission but a different thing: the entry becomes the **homeless kind**, the complement of every home, whose default name is `code` ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)). Everything below is written about a non-citable kind with a home; [§FS-config.3.9.2](FS-config.md#392-the-homeless-kind) says where the homeless one differs.
- **A row in the generated Project map and in the generated citation directions** ([§FS-init.2.3.4.4](FS-init.md#2344-project-map), [§FS-init.2.3.5](FS-init.md#235-citation-directions)) — rendered by **place**, never by name, because the name is a config handle and the place is the thing a reader can open.
- **Citation-direction rules.** The citing-side classification already reaches it: a citation inside a kind's home, when exactly one home contains its file, is classified as that kind even where no declaration encloses it ([AR-scanner.2.4](../architecture/AR-scanner.md#24-citing-side-classification)). Obligations attach per file rather than per declaration ([§FS-check.3.11](FS-check.md#311-missing-required-citation)), since there is no declaration to attach them to.
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

The default follows from `folder` rather than from a second key restating it: a kind whose declarations live in a directory has a directory a reader arrives at, and `grund init --docs` already scaffolds that README and writes the convention into it ([§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place)). `index = false` is for a kind whose declarations are *exercised* rather than navigated — the canonical case is a repository's own `E2E` kind, whose home holds case directories and no README, and whose `e2e/README.md` one level up documents the case layout in English instead of naming `E2E-` IDs. That is why the opt-out spells a file name or `false` rather than being inferred from a README's absence: "any folder README is an index" is not true of every tree.

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

The defaults declare these nine, in this order (an existing `grund.toml` that omits `[[kinds]]` gets them with the older `FS` home of [§FS-config.3.4.4.4](FS-config.md#3444-a-config-that-omits-kinds-keeps-the-older-fs-home)):

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
file   = "requirements.md"
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

A project that overrides this list replaces the defaults entirely — there is no merge. To extend rather than replace, copy the defaults and add to them.

##### 3.4.4.1 Where each citable default declares

`GRUND` is the H1 of the single file `docs/grund.md` (the project's reason for being — one declaration, all of it inline); `GOAL` declarations are H2 headings inside the single file `docs/goals.md` (one file, all goals inline); `FS` declarations are H2 headings inside the single file `requirements.md` (one obvious requirements entry for new projects); `RM` declarations are likewise H2 headings inside the single file `docs/roadmap.md` (one file, all milestones inline) — those four are single-file kinds (`file = "<path>"`); `AR`, `DF`, and `DA` declarations are the H1 of a file in their `folder` (an `AR` declaration may instead live inline in a source doc-comment with an optional stub in `folder` — [AR-scanner.4](../architecture/AR-scanner.md#4-inline-declarations-in-language-doc-comments)). A single-file kind can always be broken up later by swapping `file = "<path>"` for `folder = "<dir>"` and moving the document into that folder — the schema models the transition as exchanging one key for the other, not setting both.

##### 3.4.4.2 The two test kinds are non-citable, and lowercase

A test cites the document whose claim it proves, and is never cited back: an `e2e` scenario proves the What as a user sees it (`must` cite `FS`, and `should-not` cite `AR` — a black-box scenario that reads the design is not black-box), an `integration` test proves the How — that the parts fit as designed (`should` cite `AR`). Unit tests live with the code and follow `code`'s rule, so there is no third kind for them. Lowercase because these names never appear in an ID, so a reader should not mistake one for a prefix; the `KIND ∈ {…}` vocabulary line lists the citable seven.

##### 3.4.4.3 `E2E` is configured, not a default

`E2E` is **not** in this list, and is still a fully supported kind: a repository whose e2e suite is a corpus of case directories declares it (`kind = "E2E"`, `folder = "e2e/cases"`, `index = false`) and gets the case-declaration machinery of [AR-scanner.6](../architecture/AR-scanner.md#6-e2e-case-declarations) — `E2E-<case>` IDs, `grund <ID>` over the case manifest, per-case obligations, and the fixture-tree pruning that keeps a nested case repo out of the host scan. That machinery follows the configured `E2E` home, so a config that wants it names it. Decided in [§DF-non-citable-kinds.3](../decisions/functional/DF-non-citable-kinds.md#3-consequences), which also records what a default-config repository with an `e2e/cases` tree sees on upgrade.

##### 3.4.4.4 A config that omits `[[kinds]]` keeps the older `FS` home

Compatibility note: a pre-existing `grund.toml` that omits `[[kinds]]` keeps the pre-`requirements.md` implicit FS home (`folder = "docs/functional-spec"`) until the project writes an explicit `[[kinds]]` table. New zero-config projects and freshly generated configs use the canonical defaults in [§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds), where `FS` is `file = "requirements.md"`. This preserves existing configs without adding a new schema version.

#### 3.4.5 Name rules

**Names are unique across the whole table.** `[citations.<kind>]` and `grund list --kind` key on a name, so two rows wearing one name is a config with no answer to "which".

##### 3.4.5.1 Citable names are prefix-free

No citable kind's name may be a prefix of another citable kind's name. `kind = "DA"` and `kind = "DAT"` together are invalid because a token starting with `DAT-` would parse as either kind. The rule is about *tokenization*, so it stops where tokenization does: a non-citable kind's name never appears in an ID, so `skill` beside a citable `SKI` is fine and a config that spells it loads. grund validates this on load and refuses ambiguous configs with a single error pointing at the offending pair (per [§FS-config.4.3](FS-config.md#43-invalid-config-behavior)).

##### 3.4.5.2 `code` is reserved to the homeless kind

`code` is the homeless kind's ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)): it is the default name of the complement of every configured home, so a row may take it only by *being* that complement — `citable = false`, no `folder`, no `file`. Any other row wearing it would collide with the kind every citation outside a home resolves to.

#### 3.4.6 `prefix`, the former spelling of `kind` *(removed in 0.13.0)*

`prefix` was this key's name while every kind declared IDs and its name really was one. It stopped loading in grund **0.13.0**, at the end of the deprecation window [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) asks of a renamed config key: 0.12.0 shipped `kind` beside it and warned every config that still spelled it, naming this release. A config that still spells it is **refused**, not read with the key ignored — an ignored name leaves a `[[kinds]]` row with no kind, which changes what the configuration means without saying so. The refusal is an ordinary config error ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)) that names `kind` as the key to write instead, anchored at the line `prefix` is written on. An entry that sets both `kind` and `prefix` earns that same error at that same line: with one of the two names gone there is nothing left to disambiguate, so the pair is no longer a rule of its own.

```text
error: grund.toml:4: [[kinds]] `prefix` was removed in grund 0.13.0 — rename it to `kind`
```

##### 3.4.6.1 The migration

The migration is that rename, and the error names the line to make it on. A grund before 0.13.0 did it in one command — `config show` printed every entry under the canonical spelling whichever the file used, so `grund config show > grund.toml` rewrote the file — but from 0.13.0 nothing that has to load the config can help.

##### 3.4.6.2 Why the key was renamed

The rename is what `citable = false` forces. `prefix` was accurate for every row of the table and stopped being accurate for half of it; *kind* is what the rest of grund already calls this value — the `{kind}` placeholder of `[id] format` ([§FS-config.3.2](FS-config.md#32-id--id-grammar)), the `--kind <KIND>` selector of [§FS-list.1](FS-list.md#1-inputs), and the `[citations.<kind>]` table key ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)). Under the new name, prefix-ness is a *derived* property of citable kinds ([§FS-config.3.4.5](FS-config.md#345-name-rules)) rather than the schema's word for the whole concept. Decided in [§DF-non-citable-kinds.2.4](../decisions/functional/DF-non-citable-kinds.md#24-the-field-is-a-kind-not-a-prefix).

#### 3.4.7 `scan` — a place that is listed, not walked

`scan = false` (default `true`) keeps a non-citable kind's home out of the walk. The kind is still a **place**: it gets its Project map row ([§FS-init.2.3.4.4](FS-init.md#2344-project-map)) with its title, so an agent is told the directory exists and what it is for — and nothing in it is read, so nothing in it is checked.

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

The case it exists for is content that ships verbatim somewhere else: scaffold templates, embedded assets, example configs. Such files cannot be grounded — a `§` citation in one lands in every tree it is copied into as a dangling reference to a declaration that tree does not have — and leaving the kind unconfigured would leave the directory out of the map. [§FS-config.3.5](FS-config.md#35-scan--what-gets-walked)'s rule that a home is a walk root `exclude` cannot prune is about a config that says both "this directory matters" and "skip its descendants"; this key is the config saying one thing: listed, not walked.

##### 3.4.7.2 Not walked, however the walk arrives

The home is left out of the walk roots of [§FS-config.3.5](FS-config.md#35-scan--what-gets-walked), *and* pruned when a walk meets it on the way down — under the config root, or under an `include` entry it sits inside, as `docs/templates` sits inside `docs`. An `include` entry that names the home itself does not walk it either: the narrower key, the one written on the kind, is the config's answer where the two disagree. A `file` home ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)) is unwalked on the same terms as a `folder` one, and it is the case that shows why the rule cannot be a rule about directories: `docs/template.md` is never a directory to skip, and skipping its parent is not on offer, `docs` being an ordinary scanned home. Anything less would make `scan = false` a silent no-op for every repository that keeps such a file or directory under a scanned one, which is where a scaffold usually is.

##### 3.4.7.3 An explicit path argument still reads it

An explicit path argument still reads it — `grund check docs/templates` scans the directory it names, the same way it reads past `[scan] include` ([§FS-config.3.5](FS-config.md#35-scan--what-gets-walked)). The key describes the *default* scope, which is what a run with no argument reads and what [§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only) tiers against; a path a user typed is that user narrowing the run to a directory they are asking about.

##### 3.4.7.4 What an unwalked kind keeps and loses

What an unwalked kind keeps: its home, its title, and its Project map row. What it loses, beyond what `citable = false` already takes ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)), is every rule that reaches a file — citation checking, the directions bullet and `[citations.<kind>]` rules, and the grounding clause of [§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids) — because no file in it is scanned. That is why `require_grounding = true` on this row is a config error rather than a no-op ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)).

##### 3.4.7.5 Under `--full`

Under `grund check --full` ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)) the whole config root is walked and its files are reached like any directory nobody configured: resolution failures only, never a convention it did not adopt. They are reached from *outside* the configured scope even when a walk root encloses them ([§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only)), because the scope is what a run without the flag reads, and that run does not read them.

##### 3.4.7.6 Three config errors

Three combinations are config errors, reported per [§FS-config.4.3](FS-config.md#43-invalid-config-behavior), each closing a state the key cannot describe:

- `scan = false` on a **citable** kind. Its declarations would be invisible rather than declared — the trap [§FS-config.3.5](FS-config.md#35-scan--what-gets-walked) closes — so a kind that declares IDs is always walked. Set `citable = false`, or drop the key.
- `scan = false` with **no home**. The homeless kind ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)) is the complement of every home, and what of that complement is walked is `[scan] include`'s to say.
- a `[citations.<kind>]` table naming an unwalked kind as the **citing** kind. No file in the home is scanned, so the rule could never fire — the vacuous pass [§DF-non-citable-kinds.2.5](../decisions/functional/DF-non-citable-kinds.md#25-obligations-get-a-per-file-unit-and-grounding-follows-the-home) refused for a kind with no declarations, one level up.

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

- `require_grounding = true` on a `scan = false` row ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-walked)). No file in the home is read, so the rule could never fire — the reasoning [§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-walked) already gives for a `[citations.<kind>]` rule on an unwalked kind.
- Either key on a **citable** `file = "<path>"` row. Such a kind's document is where its declarations live, and [§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in) leaves Markdown alone outside a non-citable home, so there is nothing the key could mean — as `index` means nothing on a file kind ([§FS-config.3.4.2](FS-config.md#342-index--the-kinds-index-file)). A **non-citable** `file` row is not rejected: its document is governed like every other file of a non-citable home ([§FS-check.3.6.1](FS-check.md#361-which-files-a-row-governs)), so the row is exactly where that one document's grounding is said, and `grounding_level` cuts it into heading subtrees like any other Markdown file ([§FS-check.3.6.2](FS-check.md#362-the-unit)).
- `grounding_level` outside `1..=6`, on a row or in `[reference]`. There is no heading it could name.
- `grounding_level` on a row whose **effective** `require_grounding` is off — written `false` on the row, or inherited off from `[reference]`. The level could never fire, and a level nothing reads would still switch on the scanner's per-file structure pass ([AR-scanner.2.7](../architecture/AR-scanner.md#27-grounding-units-per-file)) for a tree that grounds nothing.
- `[reference] grounding_level` where the global boolean is off and no row turns grounding on. The same reason, one scope up.

##### 3.4.8.6 `config show`, and why the globals stay

`grund config show` prints each key on a row only where it differs from the effective global ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)), as it does for `citable` and `scan`, so the printed config loads back as itself. The global keys are kept rather than deprecated — every existing config keeps its exact meaning with no edit, and `--require-grounding` needs a global meaning regardless.

#### 3.4.9 `values` — first-class value declarations

`values = true` opts whole declarations in this row's home into [§FS-values](FS-values.md#fs-values-opted-in-kinds-bind-authored-components-to-one-declared-value). It is absent and false by default. It is one of the two value-related config keys: `values` opts whole declarations in, while `value_chapter` ([§FS-config.3.4.13](FS-config.md#3413-value_chapter--the-chapter-whose-named-children-are-values)) makes the named children of one chapter into roots inside declarations. The two are refused on one row, and there is still no `value_sources` key. An enabled row must be citable, have exactly one existing `file` or `folder`, and normalize that home inside the project root. Each violation is a located config error. Validation checks these structural relationships without parsing declaration content.

`grund config show` prints `values = true` only for an enabled row; a false or absent value prints no key. The key is additive and does not change `grund_config_version` ([§FS-config.5](FS-config.md#5-schema-versioning)).

##### 3.4.9.1 The value suffix is a separate authority

The exact `<!-- grund:value -->` section suffix is a separate in-document authority ([§FS-values.2.4](FS-values.md#24-embedded-section-value-roots)). It works in any supported scanned declaration independently of this key and its home, and it neither opts the enclosing declaration in nor changes JSON discovery. No new config key, scan input, or migration accompanies it. Chapter authority is the third and last one, taken per kind rather than per document: `value_chapter` makes every named child of one named chapter a root without any mark ([§FS-values.2.5](FS-values.md#25-chapter-declared-value-roots)).

##### 3.4.9.2 The home is the JSON source boundary

The home is also the complete JSON source boundary: a `.json` `file` is the one source, while a `folder` contributes its normalized, bytewise-ordered direct `.json` children. Nested or outside JSON is never a value source. Generic scan extensions and filters, explicit path narrowing, and `--full` neither add nor suppress those home inputs ([§FS-values.2.2](FS-values.md#22-json-declarations-from-the-kind-home)).

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

`format` uses exactly the template placeholders and the repository's
`number_pattern` and `slug_pattern` validation from [§FS-config.3.2](FS-config.md#32-id--id-grammar). It is
[§FS-config.3.2](FS-config.md#32-id--id-grammar)'s `format` written at the kind scope, and resolves like any setting
admitted at two scopes ([§FS-config.principle.rungs](FS-config.md#principlerungs-the-committed-scopes-are-a-relation-not-a-closed-list)): it overrides only this kind, and
`[id].format` remains the default for every other kind. It is valid
without `resolve` or `fetch`, but invalid on a non-citable kind.

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
are Markdown. It is invalid on a non-citable, homeless, unwalked, external-
snapshot, or JSON-value kind: each either declares no IDs, supplies no scanned
rule catalog, or gives its title another source contract. These relationships
are config validation; sentence titles are scanned declaration data and are
validated later as located findings ([§FS-rules.4](FS-rules.md#4-validation-lifecycle)).

The key is optional and additive, so `grund_config_version` remains 1 ([§FS-config.5](FS-config.md#5-schema-versioning)).
An older binary rejects the unknown key loudly rather than silently ignoring a
rule-enabled repository.

#### 3.4.13 `value_chapter` — the chapter whose named children are values

`value_chapter` names, for one `[[kinds]]` row, the chapter whose named direct
children are value roots in every declaration of that kind ([§FS-values.2.5](FS-values.md#25-chapter-declared-value-roots)):

```toml
[[kinds]]
kind = "AR"
folder = "docs/architecture"
value_chapter = "values"
```

The value is a section handle: a string matching the fixed named-component
grammar `[a-z][a-z0-9-]*` of [§FS-config.3.2.7](FS-config.md#327-named_sections--the-gate-for-section-handles), never a displayed heading title
and never a `slug_pattern` match. The key is absent by default, absent and set
to nothing mean the same, and `grund config show` prints it only on a row that
sets it. It is optional and additive, so `grund_config_version` remains 1 ([§FS-config.5](FS-config.md#5-schema-versioning)),
and an older binary rejects the unknown key loudly rather than silently reading
a chapter-enabled repository as if it declared no values.

An enabled row must be citable, must have exactly one existing `file` or
`folder` home normalizing inside the project root, and must not set `scan =
false`: a non-citable row declares no IDs so no coordinate could exist, a
homeless row has no declarations of its own, and a listed-but-unwalked place is
never read, so its chapter could never be seen. The row must not also set
`values = true`: a whole-declaration value's immediate children must be a
contiguous numeric run ([§FS-values.2.1](FS-values.md#21-markdown-declarations)), so a named chapter cannot legally
exist inside one, and the pair is refused rather than merged. Setting the key
twice on one row is an error, as `values` and `rules` already are.

`[id] named_sections = true` is a prerequisite of the key rather than a
consequence of it: a project that sets `value_chapter` while `named_sections` is
absent or false is a located config error naming the missing gate, not a silent
no-op, and the `named_sections` default does not change ([§FS-config.3.2.7](FS-config.md#327-named_sections--the-gate-for-section-handles)). Each
violation above is a located config error. Validation checks these structural
relationships without parsing declaration content; a chapter that a declaration
fills wrongly is scanned data and is reported later as a located finding ([§FS-check.3.20](FS-check.md#320-invalid-value-declaration)).

### 3.5 `[scan]` — what gets walked

```toml
[scan]
include            = ["requirements.md", "docs", "e2e", "src"]
exclude            = ["target", "node_modules", ".git", "dist", "build", ".venv"]
extensions         = ["md", "rs", "go", "java", "kt", "ts", "tsx", "js", "py", "c", "cpp", "swift", "scala", "rb", "php", "cs", "lisp", "scm", "clj", "sql", "hs", "lhs", "lua", "ada", "adb", "ads"]
comment_prefixes   = ["//", "#", ";", "--", "*", "/*"]
docstring_python   = true
respect_gitignore  = true
```

`include` is the set of paths walked from the config root ([§FS-config.3.5.7](FS-config.md#357-include-is-walked-from-the-config-root)), beside every configured kind home ([§FS-config.3.5.8](FS-config.md#358-every-configured-kind-home-is-walked)). `exclude` is the set of directory names skipped at any depth, `extensions` filters which files are read ([§FS-config.3.5.13](FS-config.md#3513-an-extension-makes-a-file-readable-not-declarable)), and `respect_gitignore` has the walk honor the ignore files as well ([§FS-config.3.5.15](FS-config.md#3515-respect_gitignore--the-ignore-files)). `comment_prefixes` are the markers recognized when looking for inline declarations and citations in source files, composed with `extensions` ([§FS-config.3.5.14](FS-config.md#3514-comment_prefixes-compose-with-extensions)); `docstring_python` enables Python triple-quoted-string scanning in addition to `#` comments. A hidden file is not read ([§FS-config.3.5.12](FS-config.md#3512-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent)), and `include` is a scope, not a fence ([§FS-config.3.5.11](FS-config.md#3511-include-is-a-scan-scope-not-a-fence)). A walk that ends up reading no files at all is reported, not silently passed ([§FS-check.2.2](FS-check.md#22-empty-scan)).

An opted-in kind's home JSON is catalog input rather than part of this walk. The exact discovery and independence from every key in this table are fixed by [§FS-config.3.4.9](FS-config.md#349-values--first-class-value-declarations) and [§FS-values.2.2](FS-values.md#22-json-declarations-from-the-kind-home).

**Symlinks ([§FS-config.3.5.1](FS-config.md#351-a-symlink-in-the-tree-is-followed)–[§FS-config.3.5.6](FS-config.md#356-which-unresolvable-links-are-owed-a-report)).** Decided in [§DF-symlink-scan](../decisions/functional/DF-symlink-scan.md#df-symlink-scan-a-symlink-in-the-scanned-tree-is-followed-and-the-report-names-the-link).

#### 3.5.1 A symlink in the tree is followed

A **file symlink** inside a walked tree is followed wherever its target resolves. A
**directory symlink** is followed only while its canonical target remains inside
the independently checked project's canonical root; when the target is outside
that root, the directory is pruned and no file below it is scanned. The same gate
applies when a configured or explicit scan root is itself a directory symlink.

This is a boundary on link traversal, not on scan paths generally: a non-symlink
parent-relative `include` may still name external content. Nor does resolving the
project's own config root make it an outward link: when the repository itself was
reached through a symlink, its canonical root is the fence and its tree remains
readable. In-root directory links keep their in-tree spelling and are followed as
before. A loaded workspace adds the stronger ownership boundary of
[§FS-workspace.6](FS-workspace.md#6-nested-project-boundary), so a link into
another loaded project is pruned even when that project is physically inside the
current project's root.

#### 3.5.2 A finding names the in-tree link path

Every finding from a link met **inside a walked tree** is reported at the in-tree link path, never the target's: that is the path a reader can act on, and it is what keeps `relative_paths` output ([§FS-config.3.6](FS-config.md#36-output--report-format)) and the additivity rule of [§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full) meaningful. An explicit path argument follows the same reporting rule: resolving `grund check docs/beta.md` identifies and reads the target, but every text and JSON report names `docs/beta.md` as reached through the configured CLI base. This remains true when the target resolves outside the config root — the in-tree link is the bounded, actionable spelling and the external physical path never becomes report output.

##### 3.5.2.1 The walk root keeps the path the run was handed

The same rule reaches the **walk root itself**: a repository whose own path is reached through a link — a symlinked `~/work`, macOS resolving `/var` to `/private/var` — is walked and reported under the path the run was handed, never the physical one it resolves to. Resolving a root is how a run recognizes that a scope *is* the config root; it is not a decision about what the report calls it, and a finding spelled physically is one [`relative_paths`](#36-output--report-format) cannot render and no reader of that repository ever wrote.

#### 3.5.3 The directory rules apply under the link name

The directory rules above still apply to a followed directory under its **link** name, so `docs/node_modules -> ../../node_modules` is excluded exactly as a real directory of that name would be. The two boundaries that are *not* name rules are another project's root, which a link may not carry a walk across in any direction ([§FS-workspace.6](FS-workspace.md#6-nested-project-boundary)), and an E2E case directory, whose fixture tree stays out of the host scan however a link reaches it ([§FS-config.3.4.4.3](FS-config.md#3443-e2e-is-configured-not-a-default)).

#### 3.5.4 One physical file is read once

One **physical** file is read once however many spellings reach it; when two do, the surviving spelling is the earlier root's, and within a single root the lexicographically first one ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full), [§FS-errors.4](FS-errors.md#4-determinism)).

#### 3.5.5 A link the walk cannot resolve is reported, and not walked into

A link the walk cannot resolve is not a silent skip: a broken target, or a loop such as `docs/self -> .`, is the per-file scan failure of [§FS-check.2](FS-check.md#2-outputs) — reported at the link's own path, the walk continuing past it, the run exiting `2` ([§REQ-no-missed-citation.1](../requirements/REQ-no-missed-citation.md#1-no-silent-skips)). Past it, and never *into* it: a loop is pruned where it is met, including the kind whose target reaches back over the walk root itself (`docs/up -> ..`), so the run never reports findings out of a tree it has just called unreadable and `include` still bounds what was read.

#### 3.5.6 Which unresolvable links are owed a report

That report is owed only where the walk would otherwise have read through the link, judged by the same rules as any other entry: the ignore files for both kinds, and `extensions` as well for a broken link, which names a file where a loop names a directory. So a dangling `docs/logo.png -> nowhere`, and a link of either kind that `.gitignore` covers, stay silent — the walk was never going to read them.

##### 3.5.6.1 A broken link with no extension is a declared blind spot

A broken link with **no extension at all** is silent for the same reason and is worth naming, because it is the one case where that answer can be wrong: `docs/shared -> ../nonexistent-dir` would have been a directory to descend into had it resolved. Nothing on disk distinguishes it from `bin/tool -> nowhere`, since the target does not exist and only the target could have said which it was, and reporting every extensionless dangling link is the noise this gate exists to prevent. That is a declared, bounded blind spot ([§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded)) and not a silent one: a link you need scanned is one you need to resolve.

#### 3.5.7 `include` is walked from the config root

`include` is walked **from the config root** — the directory the discovered `grund.toml` was found at ([§FS-config.1](FS-config.md#1-file-location-and-discovery)), or, when no config was discovered, the current working directory (never a subdirectory that merely happened to be passed as `grund`'s path argument). So in a config-less repo `grund` (no path) and `grund check .` both walk `requirements.md`, `docs/`, `e2e/`, `src/` relative to the cwd, while `grund check src/foo` or `grund check lib/` scans exactly the file or directory it is handed — an explicit path argument overrides `include` rather than being filtered by it. A plain parent-relative entry such as `../shared` intentionally names external content and is still walked; [§FS-config.3.5.1](FS-config.md#351-a-symlink-in-the-tree-is-followed)'s project-root boundary applies only when a **directory symlink** carries traversal outside the project.

#### 3.5.8 Every configured kind home is walked

**Every configured kind home is walked, whether or not `include` names it** ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)). A home is the repository saying "declarations and citations live here", so `include` names the *extra* roots — `src`, `crates`, a `README.md` — rather than having to repeat the homes the `[[kinds]]` table already spelled. A home that does not exist walks as nothing and earns no finding, so a fresh repository whose default homes are not scaffolded yet stays silent. The one home that is not a walk root is the one the config says so about: `scan = false` ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-walked)) lists a place without walking it.

#### 3.5.9 A walk root outruns every rule about descent

A home is a **walk root**, and no walk root is pruned by `exclude`, an ignore file, or the hidden-directory rule ([AR-scanner.1](../architecture/AR-scanner.md#1-tree-walk)) — so a home the repository also excludes is read, while everything *below* it is filtered as usual. That is the honest reading of a config that says both: the `[[kinds]]` entry names the directory, and `exclude` was written about descendants. All three are rules about a descent, which is why a root outruns them; the hidden-file rule is about a name, so a root does not outrun it ([§FS-config.3.5.12](FS-config.md#3512-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent)).

#### 3.5.10 Why every home is walked

Walking every home closes a trap that had nothing to do with non-citable kinds and everything to do with why one would be configured: a `folder` or `file` outside `include` was never walked, so its declarations did not exist and its citations were **invisible rather than dangling** — no resolution, no finding, nothing to notice. A kind whose entire content is "this directory matters" would have fallen into it on its first line of config. **Upgrade note:** a repository that had a home outside `include` starts seeing that home's findings; they were always true of the tree, and the run was simply not reading it.

#### 3.5.11 `include` is a scan scope, not a fence

A citation in a file that neither `include` nor a kind home brings into the walk is invisible rather than merely unchecked ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)), so `grund check --full` walks the whole config root past this key. The flag cancels `include` and, with it, the `scan = false` prune of [§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-walked), and nothing else: every other rule of this table applies to that walk unchanged ([§FS-check.1.3.1](FS-check.md#131-the-walk-covers-the-whole-config-root)), and what it reports outside the configured scope is [§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)'s to say.

#### 3.5.12 A hidden file is not read, and the rule is not about descent

The walk skips a hidden directory by not descending into it; it skips a file whose own name begins with `.` by the name that file wears, before `extensions` is consulted at all ([AR-scanner.1](../architecture/AR-scanner.md#1-tree-walk)). So `docs/.notes.md` is not scanned though `md` is listed, and a citation inside it neither resolves nor dangles — it is invisible the way one outside `include` is, and `grund check --full` does not reach it either ([§FS-config.3.5.11](FS-config.md#3511-include-is-a-scan-scope-not-a-fence)). Being a rule about a name rather than about a descent, it also reaches a walk **root**, the one exception to [§FS-config.3.5.9](FS-config.md#359-a-walk-root-outruns-every-rule-about-descent): a `file` home whose name is hidden, like an `include` entry naming one, is not read as a root. It is a blind spot the repository can see and plan around ([§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded)): what decides is the file's own name, so a document that must be checked is one not named as a dotfile, and a run handed such a file whose extension `extensions` *does* allow says so rather than blaming the list ([§FS-check.2.2](FS-check.md#22-empty-scan)).

#### 3.5.13 An extension makes a file readable, not declarable

Listing an extension makes a file *readable*, not declarable. A prose markup format other than Markdown — AsciiDoc, reStructuredText, LaTeX — has its citations checked as soon as its extension appears in `extensions`, because the citation grammar is format-agnostic; its native heading syntax still declares nothing, since a declaration is a `#`-prefixed heading or a comment-prefixed line. Whether those formats should be declaration homes of their own is an open discussion ([§DISC-markup-format-declarations](../discussions/proposals/2026-05-25-markup-format-declarations.md#disc-markup-format-declarations-declarations-in-asciidoc-restructuredtext-latex-and-similar-markup-document-formats)), not a configured behavior.

#### 3.5.14 `comment_prefixes` compose with `extensions`

The two lists compose: adding `sql` without `--`, or `--` without `sql` (or another extension using that marker), does not enable SQL doc-comments. Every default comment prefix has a path through the default extension list: `;` pairs with Lisp, Scheme, and Clojure extensions; `--` pairs with SQL, Haskell, Lua, and Ada extensions; and `*` / `/*` are block-comment continuation and opener forms in the C-family extensions. Any line whose first non-whitespace run is a configured prefix is eligible to host a declaration heading or a citation. Each claimed form has a strict-mode executable case that plants a marked dangling citation in that form ([§REQ-no-missed-citation.3](../requirements/REQ-no-missed-citation.md#3-proven-per-host-language)).

#### 3.5.15 `respect_gitignore` — the ignore files

`respect_gitignore` (default `true`) makes the scanner honor every form of ignore file the `ignore` crate recognizes — `.gitignore` at any depth, `.git/info/exclude`, the global `core.excludesFile`, and `.ignore` files. Set to `false` only when you genuinely need to scan ignored paths. The directory-level `exclude` list is applied **in addition** to ignore-file rules, never instead of them. See [AR-scanner.1.1](../architecture/AR-scanner.md#11-respecting-gitignore-and-friends).

### 3.6 `[output]` — report format

```toml
[output]
format         = "text"   # text | json
color          = "auto"   # auto | always | never
relative_paths = true     # show paths relative to config root in reports
```

`relative_paths = true` (default) renders every `<path>` in a report relative to the config root ([§FS-config.1](FS-config.md#1-file-location-and-discovery)); `relative_paths = false` renders it relative to the CLI base instead ([§FS-config.3.6.1](FS-config.md#361-relative_paths--false--the-cli-base)). Either way `grund` **never** emits an absolute path, nor a path that escapes the loaded root other than the `..` path of a config above the run's root that the run had to read (`../grund.toml:16`, [§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)); this is what keeps the report deterministic per [§FS-errors.4](FS-errors.md#4-determinism). `color` controls ANSI styling once the colored-output feature lands ([§FS-errors.3](FS-errors.md#3-message-text)); until then output is plain bytes regardless of this value, and a change to that default goes through the [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path) path.

#### 3.6.1 `relative_paths = false` — the CLI base

`relative_paths = false` renders every `<path>` relative to the path argument passed on the command line — or to the current working directory when no path is given. A target elsewhere inside the loaded project or workspace uses the minimum `..` components needed to reach it from that base; those parent components are allowed only while the resolved target remains inside the loaded root. The CLI-base choice and the rejected workspace-root alternative are recorded in [§DF-cli-base-parent-paths](../decisions/functional/DF-cli-base-parent-paths.md#df-cli-base-parent-paths-relative_paths--false-keeps-one-cli-base-and-may-climb-within-the-loaded-root).

### 3.7 `[fmt.cross_refs]` — cross-reference emission

```toml
[fmt.cross_refs]
enabled       = true       # default; false opts out of generated Markdown links
anchor_format = "github"   # default; one of github | gitlab | mkdocs | pandoc | none
```

The full contract for this block — what `enabled` does, the named `anchor_format` profiles, and when the cross-reference pass runs — lives in [§FS-fmt.6.7](FS-fmt.md#67-configurability), [§DF-md-link-default-on](../decisions/functional/DF-md-link-default-on.md#df-md-link-default-on-markdown-cross-reference-links-default-on-for-github-review-and-discovery), and [§DF-md-link-anchor-strategy](../decisions/functional/DF-md-link-anchor-strategy.md#df-md-link-anchor-strategy-heading-text-slugs-re-derived-on-every-fmt-pass). It is part of the schema here because the generated `grund.toml` ([§FS-init.2.4](FS-init.md#24-generated-grundtoml)) writes every key in this section explicitly, including `enabled = true`, so the default generated file teaches that `grund fmt --write` emits Markdown inline links in `.md` files.

#### 3.7.1 One block for every cross-reference form

`[fmt.cross_refs]` is the home for cross-reference settings; today `grund fmt --cross-refs` only emits the Markdown inline-link form ([§FS-fmt.6](FS-fmt.md#6-cross-reference-emission)), so `anchor_format` is the only knob — a future markup family adds its settings under this same block ([§FS-fmt.6.7](FS-fmt.md#67-configurability)), additively, with no `grund_config_version` bump ([§FS-config.5](FS-config.md#5-schema-versioning)). The sibling `[fmt]` table is a different thing ([§FS-config.3.10.3](FS-config.md#3103-fmt-is-the-commands-home-fmtcross_refs-the-passs)).

### 3.8 `[workspace]` — sub-project namespaces

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

Each `[citations.<kind>]` subsection names the **citing** kind, by the `kind` name of [§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds); its arrays name the **cited** kinds. The citing side may be any configured kind — citable or not ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)), but never an unwalked one ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-walked)) — or the homeless kind, `code` unless the project named it otherwise ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)); the cited side must be a **citable** kind, because a kind with no IDs has nothing a citation could point at. The section is decided in [§DF-citation-directions](../decisions/functional/DF-citation-directions.md#df-citation-directions-encode-citation-directions-as-checked-config-with-rfc-2119-levels) and proposed in [§DISC-citation-directions](../discussions/proposals/2026-06-13-citation-directions.md#disc-citation-directions-encode-citation-directions-as-checked-config). It is optional; without it no direction check runs and `grund check` behaves exactly as before. Its complete user-facing explanation is [Citation directions](../user-facing/citation-directions.md); the skill carries that page as a checked copy, while the generated entrypoint carries a checked render of configured rules.

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

An **obligation** asks: does each top-level declaration of the citing kind contain at least one citation to the target kind, anywhere in its body? Multiple array entries are **conjunctive** — `must = ["GOAL", "GRUND"]` requires a citation to each — while a `|` disjunction inside one entry is satisfied by any one alternative — `must = ["GOAL|GRUND"]` requires a citation to either. A **prohibition** fires once per offending citation site, anchored at its exact `file:line`.

##### 3.9.1.2 `E2E` obligations are per case

`E2E` obligations are per case declaration: they evaluate the case's scanned files plus the case manifest's `spec.refs` entries, but an otherwise empty evidence set still fails a `must` entry rather than satisfying it vacuously. A `spec.refs` entry is kind-shaped evidence — it may name an idealized ID that does not resolve locally, so it is not subject to the dangling check that governs ordinary citations.

##### 3.9.1.3 The level→surface mapping is fixed

`must` and `must-not` gate (`grund check` errors); `should` and `should-not` are machine-checked suggestions that never appear in `grund check`'s standing output and are surfaced only at write time (the generated entrypoint, [§FS-init.2.3.5](FS-init.md#235-citation-directions)) and on demand (`grund check --suggestions`, [§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)). The level→surface mapping is fixed, never a project knob, so two installs reading one config agree on what gates and what is suggested ([§FS-non-goals.9](FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization)).

#### 3.9.2 The homeless kind

Every citation site that no single configured kind home claims — outside every home, or in a file two overlapping homes contain — resolves to one citing kind ([AR-scanner.2.4](../architecture/AR-scanner.md#24-citing-side-classification)). That kind is the **complement** of the whole `[[kinds]]` table: it is the one kind that is not a place, which is why it has no `folder` and no `file` and why there is exactly one of it.

Its name is `code` by default, and a project may name it something truer by declaring it ([§FS-config.3.9.2.1](FS-config.md#3921-declaring-it)).

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

#### 3.9.3 Namespace matching

Rule entries reuse the citation grammar of [§FS-workspace.1](FS-workspace.md#1-citation-syntax): a bare `AR` matches the **local** namespace only; `alias/AR` pins one workspace member, spelled with the same whole alias path a citation uses (`group/api/AR`, [§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)); `*/AR` matches the kind in **any** namespace including the local one. `*/` is new syntax valid in rule entries only — it is never a citation. Each entry parses as `[alias-path-or-*/]KIND`, split at the last `/` exactly as a citation is; a malformed qualifier is rejected ([§FS-config.3.9.3.1](FS-config.md#3931-a-malformed-qualifier-is-rejected)). The match is textual on the qualifier and prefix; resolution failures are separate errors ([§FS-check.3.8](FS-check.md#38-cross-project-citation-failure)), so the direction check never loads a foreign config. Each member's own `[citations]` governs the citation sites in that member's tree — like `strict`, `require_grounding`, and `[id]`, no section inherits from the workspace root.

##### 3.9.3.1 A malformed qualifier is rejected

A malformed qualifier is rejected with a citation-target diagnostic that names the target's kind and the first invalid qualifier segment; an empty qualifier or segment is named explicitly. A `*` segment is invalid unless it is the whole qualifier, and the diagnostic says so.

#### 3.9.4 Defaults and precedence

`default` (top-level, or per-kind inside a `[citations.<KIND>]` table) sets the level for unlisted target kinds; the global default is `may`, so adoption is incremental. Precedence is **explicit target list > per-kind `default` > global `default`**. Obligations come only from explicit `must` / `should` entries — a `default` of `must` or `should` never invents an obligation toward every unlisted kind; `default` governs only how a citation to an otherwise-unlisted target is leveled for the prohibition pass.

#### 3.9.5 Validation

Config validation rejects: a `[citations.<kind>]` table whose kind is neither a configured `[[kinds]]` name nor the homeless kind's name ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)) — so `[citations.code]` is rejected in a config that named its complement something else; one naming an unwalked kind ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-walked)); a target naming a kind that is not a configured *citable* kind — reported as *an unknown target kind* for a name the table does not hold and *a non-citable target kind* for one it does, since those are different mistakes; `code` used as a `[[kinds]]` name by anything but the homeless kind ([§FS-config.3.4.5](FS-config.md#345-name-rules)); two entries declaring the homeless kind ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)); an unknown level key; and two targets of the same cited kind at different levels whose namespace matchers can match the same citation ([§FS-config.3.9.5.1](FS-config.md#3951-overlap-not-textual-equality)). The section composes unchanged with project-defined `[[kinds]]` — a new kind is one more `[citations.<KIND>]` table.

Adding `[citations]` does **not** bump `grund_config_version` ([§FS-config.5](FS-config.md#5-schema-versioning)): it is additive surface, like `[workspace]` and `require_grounding`. An older binary meeting it fails loudly with `unknown config section`.

##### 3.9.5.1 Overlap, not textual equality

The rule on two targets of one cited kind is on namespace **overlap**, not textual equality — `*/AR` (any namespace) overlaps a bare `AR` (local), so listing one at `should` and the other at `must-not` is rejected, while a local `AR` permitted alongside a pinned `alias/AR` forbidden is allowed because those matchers are disjoint ([§FS-config.3.9.3](FS-config.md#393-namespace-matching)).

### 3.10 `[fmt]` — suppressing the rewrite

```toml
[fmt]
exclude = ["docs/architecture/AR-topology.md", "docs/diagrams"]
```

`exclude` is the set of files `grund fmt` performs **no** rewrite in. The full contract — that it covers all four rewrite classes, that a suppressed file is still walked and still checked, and that a kind's index entries are wrapped there anyway — is [§FS-fmt.2.5.1](FS-fmt.md#251-fmt-exclude--a-file-at-a-time). Nothing else reads the key: no scanner, checker, or query behavior depends on it, and `grund check` says exactly the same thing about a tree with the key and without it.

#### 3.10.1 Entries are gitignore-style globs

Each entry is a gitignore-style glob resolved against the config root ([§FS-config.1](FS-config.md#1-file-location-and-discovery)) — the same dialect `respect_gitignore` already brings to the walk ([§FS-config.3.5.15](FS-config.md#3515-respect_gitignore--the-ignore-files)) — so `docs/diagrams` takes every file under that directory, `AR-*.md` matches at any depth, and `docs/architecture/AR-topology.md` names one file. A pattern the glob parser rejects is a config error at its own line, per [§FS-config.4.3](FS-config.md#43-invalid-config-behavior).

#### 3.10.2 Optional, empty by default, and additive

The table is optional and defaults to the empty list, which is what every config written before the key existed means. It is additive, so `grund_config_version` stays 1 ([§FS-config.5](FS-config.md#5-schema-versioning)), and an older binary meeting it fails loudly through the unknown-section rejection ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)) rather than silently ignoring it. `grund config show` ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)) prints the table only where the list is non-empty, so a shown config still loads back as itself.

#### 3.10.3 `[fmt]` is the command's home, `[fmt.cross_refs]` the pass's

`[fmt]` is the home for settings about the `fmt` command as a whole — it governs every rewrite `grund fmt` performs, not just the cross-reference pass; `[fmt.cross_refs]` ([§FS-config.3.7](FS-config.md#37-fmtcross_refs--cross-reference-emission)) remains the home for cross-reference settings specifically. The per-region counterpart to this key is not configured here at all — it is written in the file it governs ([§FS-fmt.2.5.2](FS-fmt.md#252-grundfmt-off--grundfmt-on--a-region-at-a-time)), and the reasoning for both is in [§DF-fmt-suppression](../decisions/functional/DF-fmt-suppression.md#df-fmt-suppression-fmt-suppression-is-per-file-and-per-region-and-the-index-carve-out-outranks-both).

## 4. Validation and inspection

### 4.1 `grund config validate [path]`

Loads the config discovered by walking up from `path` (or `.` when omitted), checks the schema, and reports problems. Exits 0 on success, 1 on validation errors — the error in the same `error: <path>:<line>: <message>` shape [§FS-config.4.3](FS-config.md#43-invalid-config-behavior) defines. No tree scan is performed. A redundant config pair at the config root is reported as a `warning:` here too ([§FS-config.1.1](FS-config.md#11-when-one-directory-carries-both), [§FS-check.4.3](FS-check.md#43-redundant-config-pair)); it is a warning, so it does not change the exit code.

#### 4.1.1 At a workspace root

When the discovered config declares `[workspace]` ([§FS-config.3.8](FS-config.md#38-workspace--sub-project-namespaces), [§FS-workspace.2](FS-workspace.md#2-workspace-configuration)), `config validate` also expands `members` and loads every member config the run would load — nested workspaces included ([§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)) — the same launch-time pass `grund check` runs before it scans anything. The first problem, whether a member config that does not load or a `members` entry that cannot be resolved, is reported once in the same `error: <path>:<line>: <message>` line `grund check` prints for it, paths rendered from the workspace root when `[output] relative_paths = true` ([§FS-workspace.5](FS-workspace.md#5-command-scope)), exit 1 ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)). No tree scan is performed. A path inside a member validates that member alone, as `grund check <member>` does ([§FS-workspace.5](FS-workspace.md#5-command-scope)). `config show` is unchanged: it prints the discovered project's effective config ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)).

### 4.2 `grund config show [path]`

Prints the **effective** configuration — defaults merged with the config discovered by walking up from `path` (or `.` when omitted), plus CLI flags — as TOML, and the TOML that comes out loads back to the same effective values it went in with: every rule below about which keys are printed keeps that so. Useful for debugging "why did grund recognize this citation" or "what does my config actually evaluate to." A redundant config pair at the config root is reported as a `warning:` on stderr before the TOML ([§FS-config.1.1](FS-config.md#11-when-one-directory-carries-both), [§FS-check.4.3](FS-check.md#43-redundant-config-pair)), so the answer to "why is this key not taking effect" is on screen next to the effective value.

#### 4.2.1 `[[kinds]]` rows print what they do not inherit

Every `[[kinds]]` entry is printed under the canonical `kind` key, and `citable` is printed **only where it is `false`**: absence *is* `citable = true`. `require_grounding` and `grounding_level` follow the same rule one scope down: a row prints either key only where its effective value differs from the effective global, which is printed under `[reference]` ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)). A row that inherits both prints neither.

#### 4.2.2 `[reference]` always prints `shorthand`

The `[reference]` table always prints the effective `shorthand` policy. An
absent key therefore appears as `shorthand = "canonical"`, while an opted-in
project appears as `shorthand = "accepted"`; either emitted form loads back to
the same effective policy ([§FS-config.3.1](FS-config.md#31-reference--citation-form)).

#### 4.2.3 `named_sections` and `values` print only when enabled

The `[id]` table prints `named_sections = true` only when enabled. Absent and explicit `false` configurations therefore retain the previous `config show` bytes; an enabled repository exposes the opt-in that explains its named heading and citation grammar. An enabled value kind likewise prints `values = true` on its row and a disabled one prints no value key ([§FS-config.3.4.9](FS-config.md#349-values--first-class-value-declarations)).

### 4.3 Invalid config behavior

A `grund.toml` that fails validation causes every `grund` subcommand to exit with code 2 (code 1 for `grund config validate` itself, [§FS-config.4.1](FS-config.md#41-grund-config-validate-path); no output and code 0 for the hidden `complete` helper, [§FS-completions.2](FS-completions.md#2-internal-dynamic-helper)) and a single error message pointing at the first problem, in the form `error: <path>:<line>: <message>` on stderr ([§FS-errors.2.2](FS-errors.md#22-cli-level-message), [§FS-check.2.1.1](FS-check.md#211-cli-level-messages)) — the `error:` prefix marks it a CLI-level failure, the `<path>:<line>:` inside the text points at the offending key or line. That includes an invalid `values` row ([§FS-config.3.4.9](FS-config.md#349-values--first-class-value-declarations)); source-content validity is deferred to the scan and checker. Subsequent problems are not reported until the first is fixed — this avoids cascading errors that obscure the root cause.

For concrete stderr examples and the distinction between `config validate` exit `1` and config-blocked command exit `2`, see [§FS-output-shapes.6](FS-output-shapes.md#6-cli-and-config-failures).

## 5. Schema versioning

The TOML file may include a top-level `grund_config_version = N`. The current version is **1**. Future incompatible schema changes increment this; grund refuses to load a config whose version is greater than the grund binary's known maximum, with an error suggesting an upgrade. Configs with no version key are interpreted as version 1. A new key is not a new version ([§FS-config.5.1](FS-config.md#51-new-keys-are-not-a-new-version)), and an older version keeps its meaning ([§FS-config.5.2](FS-config.md#52-every-older-version-keeps-its-meaning)).

### 5.1 New keys are not a new version

The version tracks **incompatible** changes to the meaning of existing keys, not the arrival of new ones. Adding an optional table or key — `[workspace]`, `[citations]`, `[[kinds]].values` ([§FS-config.3.4.9](FS-config.md#349-values--first-class-value-declarations)), `[[kinds]].format` / `resolve` / `fetch` ([§FS-config.3.4.10](FS-config.md#3410-format-resolve-and-fetch--external-snapshot-kinds)), a future `anchor_format` profile — is additive and does not bump the version, because a config that uses it is only ever written for a binary that understands it, and an older binary meeting it fails loudly and locatably through the unknown-section / unknown-key rejection ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)) rather than silently misreading it. The safety net for the forward direction is the closed section and key allow-list, not the version integer.

### 5.2 Every older version keeps its meaning

In the backward direction the gate is what [§REQ-backwards-compatibility.1](../requirements/REQ-backwards-compatibility.md#1-what-is-covered) rests on: a binary that supports version `N` keeps interpreting every version `≤ N` under the semantics that version shipped with, so upgrading the binary never re-reads a config it already understood.

## 6. What is NOT configured here

Per [§GOAL-friendliness-first.2](../goals.md#2-what-this-rules-out), the following are deliberately **not** configurable, to avoid the trap of every grund repo behaving differently in surprising ways:

- The set of severity levels (only `error` and `warning` exist); a suggestion is not a third one ([§FS-config.6.1](FS-config.md#61-suggestions-are-not-a-third-severity)).
- The exit code mapping (`0`/`1`/`2` per [§FS-cli.5](FS-cli.md#5-exit-code-mapping-is-fixed)).
- The ordering of the report (always deterministic).
- Anything that would let two correctly-configured grund installs disagree on whether a given repo is well-formed ([§GOAL-configurable.2](../goals.md#2-what-is-not-configurable)).
- The local conversation citation *preference*: it follows the user's TUI setup and is installed through `grund integrations --write` ([§FS-integrations.4.3](FS-integrations.md#43-user-preference-and-global-agent-instructions)). What a repository may commit instead is [§FS-config.6.2](FS-config.md#62-the-repositorys-conversation-opinion).

### 6.1 Suggestions are not a third severity

The `should` and `should-not` citation-direction suggestions ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)) do **not** add a third severity: they are carried on a separate non-severity advisory channel ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)), so this frozen `{error, warning}` set stays exactly two.

### 6.2 The repository's conversation opinion

A repository may commit the `link`-only *opinion* via `[reference] conversation` ([§FS-config.3.1](FS-config.md#31-reference--citation-form), [§DF-repo-conversation-opinion](../decisions/functional/DF-repo-conversation-opinion.md#df-repo-conversation-opinion-repositories-may-commit-a-link-only-conversation-rendering-opinion)), the fallback for machines that never stated a preference; an explicitly recorded user preference wins over it ([§DF-repo-conversation-opinion.2.3](../decisions/functional/DF-repo-conversation-opinion.md#23-precedence)). Repository-web guidance stays fixed in the generated agent entrypoint ([§FS-init.2.3.6](FS-init.md#236-clickable-citations)).
