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

Every setting `grund` reads from a repository is written at one of three **committed scopes**, from widest to narrowest: the **built-in default** ([§FS-config.3](FS-config.md#3-keys-version-1)), the **project**, which is the top-level tables of `grund.toml`, and the **kind**, written either as a `[[kinds]]` row ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)) or as a `[citations.<KIND>]` table ([§FS-config.3.9.4](FS-config.md#394-defaults-and-precedence)), two spellings of the one scope. This chapter states once how the scopes relate, so that a key's own section says what the key means and leaves how it resolves to this one place; it adds no key. Its clauses reach only a setting admitted at more than one committed scope, and [§FS-config.principle.inventory](FS-config.md#principleinventory-the-settings-admitted-at-both-the-project-and-the-kind-scope) lists every such setting. Per [§DF-config-scope-override](../decisions/functional/DF-config-scope-override.md#df-config-scope-override-the-committed-scopes-are-one-relation-stated-once).

### principle.rungs: The committed scopes are a relation, not a closed list

Where a setting is admitted at more than one committed scope, the value written at the **narrower** scope is the effective one for everything that scope governs, and the wider value stands wherever the narrower scope is silent: a `[[kinds]]` row is narrower than the project's tables, which are narrower than the built-in defaults. The rule is a relation over scopes rather than a list of three ([§DF-config-scope-override.2.1](../decisions/functional/DF-config-scope-override.md#21-one-relation-stated-once-and-stated-as-a-relation)).

### principle.unit: One leaf key overrides, and one value is whole

What overrides is a single **leaf key**: writing one key at a narrower scope leaves every other key that scope inherits as it was, so a `[reference]` table naming one key keeps every other `[reference]` default. What is overridden is the **whole value**: a key whose value is a list replaces the inherited list rather than merging into it, and writing `[[kinds]]` at all replaces the built-in kind list ([§FS-config.3.4.4](FS-config.md#344-the-default-kinds)). In v2 a grounding ladder is one setting, so a narrower ladder replaces the wider one whole ([§FS-config-v2.rules.grounding](FS-config-v2.md#rulesgrounding-grounding-is-a-ladder-replaced-whole)); v1 spells grounding as two leaf keys, `require_grounding` and `grounding_level`, and a v1 row inherits each of them on its own ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)).

### principle.admission: The relation reaches only scopes that admit the setting

A scope **admits** a fixed set of keys, and the relation holds only between scopes that admit the same setting. A key written at a scope that does not admit it is not a key there at all, and it is refused as an unknown key at its line ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)): `inline_style` on a `[[kinds]]` row is not a narrower spelling of `[reference] inline_style` ([§FS-config.3.1.8](FS-config.md#318-inline_style-and-the-note-budgets)). A key admitted at one scope only is stated by the table section it is documented under, and what is not a setting at all is [§FS-config.6](FS-config.md#6-what-is-not-configured-here)'s.

### principle.cli: A CLI input enters at the scope its flag spells

A command-line input is not a fourth scope that wins over all three: it enters the relation at the scope its flag spells and resolves from there. `grund check --require-grounding` spells the **project** default, so an explicit `require_grounding = false` on a row still wins over the flag ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)).

### principle.boundary: The project is the widest committed scope

Scopes stop at the project. A workspace member is configured by its own `grund.toml`, or by the built-in defaults where it has none, and inherits nothing from the workspace root's ([§FS-config.3.9.3](FS-config.md#393-alias-matching), [§FS-workspace.2](FS-workspace.md#2-workspace-configuration)): a root and a member are a boundary, not a rung. Which config a command discovers from a given directory is a different question, answered by [§FS-config.1](FS-config.md#1-file-location-and-discovery).

### principle.declared-rows: A declared row does not inherit from a built-in one

A `[[kinds]]` row a project writes starts from the defaults of this specification rather than from the built-in row of the same name, because writing `[[kinds]]` replaces the built-in list ([§FS-config.principle.unit](FS-config.md#principleunit-one-leaf-key-overrides-and-one-value-is-whole)): a declared `FS` row takes this specification's default for `citable` rather than the built-in row's, and has no home until it names one. Exactly one built-in attribute still reaches a declared row, keyed on the kind's **name** rather than on a scope: `index` ([§FS-config.3.4.2](FS-config.md#342-index--the-kinds-index-file)), applied only where the row left `index` unset, names a `folder`, and is citable. The built-in home, file, title and `citable` of a same-named built-in kind reach nothing a project declares.

### principle.install-local: The relation governs committed repository state only

Every scope above is something a repository commits. The **install-local** axis lies outside it: the user's conversation citation preference and its per-agent partial are machine state installed by `grund integrations --write`, whose precedence is stated where they live ([§FS-integrations.4.3](FS-integrations.md#43-user-preference-and-global-agent-instructions), [§FS-integrations.4.4](FS-integrations.md#44-per-agent-overrides)), and no machine-local value outside the input ([§REQ-deterministic-output.2](../requirements/REQ-deterministic-output.md#2-input-is-the-tree-the-config-and-the-invocation)) may change whether a repository is well-formed ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)). The author of a new key **classifies first, then verifies by reach**: decide whether the key carries committed repository semantics or install-local presentation, then check that its reach agrees. Reach alone does not classify: inert `[output] color` ([§FS-config.3.6](FS-config.md#36-output--report-format)) changes no verdict and is committed repository state all the same.

### principle.inventory: The settings admitted at both the project and the kind scope

Four settings are admitted at both the project scope and the kind scope, and for the keys of [§FS-config.3](FS-config.md#3-keys-version-1) this list is complete:

| Project scope | Kind scope | Stated at |
|---|---|---|
| `[reference] require_grounding` | a row's `require_grounding` | [§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level) |
| `[reference] grounding_level` | a row's `grounding_level` | [§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level) |
| `[id] format` | a row's `format` | [§FS-config.3.4.10](FS-config.md#3410-format-resolve-and-fetch--external-snapshot-kinds) |
| `[citations] default` | `[citations.<KIND>] default` | [§FS-config.3.9.4](FS-config.md#394-defaults-and-precedence) |

Every other key of [§FS-config.3](FS-config.md#3-keys-version-1) is admitted at one committed scope only, and its table section is the whole statement of where it may be written. Of [§FS-config.3.9.4](FS-config.md#394-defaults-and-precedence)'s ladder only the two `default` terms are this relation: an explicit target list over a `default` is specificity of target match between keys each admitted at one scope, not a narrower scope overriding a wider one. The v2 reader admits `id_format`, the citation `default` and the grounding ladder at both the project and the kind ([§FS-config-v2.schema.places](FS-config-v2.md#schemaplaces-rows-and-their-places), [§FS-config-v2.rules.citations](FS-config-v2.md#rulescitations-rulescitations), [§FS-config-v2.rules.grounding](FS-config-v2.md#rulesgrounding-grounding-is-a-ladder-replaced-whole)).

## concerns: Every key belongs to exactly one concern

Every key of [§FS-config.3](FS-config.md#3-keys-version-1) belongs to exactly one concern, **schema**, **rules** or **presentation** ([§FS-terms.terms.9](FS-terms.md#terms9-the-configurations-concerns)), and the test is what a finding from the key can be about: **one node** on its own is schema, **at least two** is rules, and a key that decides the **bytes** `grund` writes or shows produces no finding of its own and is presentation (a written byte that has drifted from what the config now renders is a rule over the generated region, [§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)). The test, not the table a key is written in, decides its concern. The **envelope**, `grund_config_version`, `project_name` and the three `[workspace]` keys ([§FS-config.3.8](FS-config.md#38-workspace--sub-project-members)), is read before any concern and constrains no node. A key's concern and the scopes it may be written at ([§FS-config.principle](FS-config.md#principle-a-setting-written-at-a-narrower-scope-wins)) are independent answers.

**A rule's unit is its subject's node.** A rules key is applied once per node its subject selects, and the finding names that node: an obligation is asked of each declaration of the citing kind and a prohibition fires at each citation site ([§FS-config.3.9.1](FS-config.md#391-levels)), and the unit of a place that declares nothing is each source file in it ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)).

Version 1's assignment of every key is recorded in [§DF-config-concerns](../decisions/functional/DF-config-concerns.md#df-config-concerns-a-keys-concern-is-derived-from-what-a-finding-from-it-can-be-about). The v2 format is spelled by concern, so a v2 key's table names its concern ([§FS-config-v2](FS-config-v2.md#fs-config-v2-grund-reads-a-version-2-config-by-concern-with-one-strength-vocabulary-and-fixed-defaults)), and its envelope is the same five keys with each `[workspace.imports.<alias>]` table, which also says which projects there are ([§FS-imports.declaration.table](FS-imports.md#declarationtable-the-import-table)). The two formats line up as follows:

| Concern | Version 2 | Version 1 |
|---|---|---|
| Envelope | `grund_config_version`, `project_name`, `[workspace]` | `grund_config_version`, `project_name`, and [§FS-config.3.8](FS-config.md#38-workspace--sub-project-members) |
| Schema | `[schema]`, `[schema.sources]`, `[schema.notes.*]`, `[schema.kinds.*]` ([§FS-config-v2.schema](FS-config-v2.md#schema-the-schema-concern)) | [§FS-config.3.1](FS-config.md#31-reference--citation-form)'s marker, strictness and inline-note limits, [§FS-config.3.2](FS-config.md#32-id--id-grammar), [§FS-config.3.3](FS-config.md#33-section-paths--arbitrary-nesting-depth), [§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds) less `title`, and [§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned) |
| Rules | `[rules.citations.*]`, `[rules.grounding]`, `[rules.resolution]` ([§FS-config-v2.rules](FS-config-v2.md#rules-the-rules-concern)) | [§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules), and the grounding keys of [§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level) |
| Presentation | `[presentation]`, `[presentation.kinds.*]`, `[presentation.fmt]` ([§FS-config-v2.presentation](FS-config-v2.md#presentation-the-presentation-concern)) | `project_description`, `[reference] trigger` and `conversation`, a row's `title`, [§FS-config.3.6](FS-config.md#36-output--report-format), [§FS-config.3.7](FS-config.md#37-fmtcross_refs--cross-reference-emission), and [§FS-config.3.10](FS-config.md#310-fmt--suppressing-the-rewrite) |

## requirements: What the config contract holds to

Eight requirements on the key set as a whole, each carrying a **mark**: *realized* where `grund` behaves this way and a section of this specification says so, and a run that breaks it is a defect in `grund`; *directional* where it binds what is written from here on rather than saying what `grund` does; *deferred* where a named follow-up owes it. A requirement here is realized unless its body marks it, or a part of it, otherwise, and reading a direction as a statement of what `grund` does is the mistake to avoid.

### requirements.1: Every default is overridable where it has a meaning

This requirement is directional. A key added from here on should be admitted at every committed scope where its value would govern something that scope tells apart, so that a key admitted at one scope alone is so by that judgement and not for want of the question being asked; the obligation falls on the admission decision, not on a clause at the key's own section ([§FS-config.principle.admission](FS-config.md#principleadmission-the-relation-reaches-only-scopes-that-admit-the-setting)), and what it reaches is [§FS-config.principle.inventory](FS-config.md#principleinventory-the-settings-admitted-at-both-the-project-and-the-kind-scope). Deliberate refusals stay refusals: everything [§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree) rules out. Serves [§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable).

### requirements.2: Zero config works

Every key is optional, and no config file anywhere up the walk means the canonical defaults of this specification rather than a refusal ([§FS-config.1](FS-config.md#1-file-location-and-discovery), [§FS-config.3](FS-config.md#3-keys-version-1)). Serves [§GOAL-zero-config](../goals.md#goal-zero-config-works-on-any-conformant-tree).

### requirements.3: One key means one thing at every scope

This requirement is directional, for the same reason as [§FS-config.requirements.1](FS-config.md#requirements1-every-default-is-overridable-where-it-has-a-meaning): it binds the next key, and renaming one already written is a compatibility question of its own ([§FS-config.5.2](FS-config.md#52-a-versions-meaning-moves-only-as-backwards-compatibility-allows)). A key admitted at two scopes keeps its name, its value domain and its meaning at both; only its reach changes ([§FS-config.principle.rungs](FS-config.md#principlerungs-the-committed-scopes-are-a-relation-not-a-closed-list)). Two keys that share a spelling without sharing a setting are not an instance of this: `[scan]` and a row's `scan` are two settings, reconciled where they are stated ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned)), and `[id] format` is not `[output] format`.

### requirements.4: Every value has exactly one resolution

The effective value of any key at any scope is a function of the committed config files and the CLI inputs alone. It never depends on the order two keys were written in, on an environment variable, or on machine state, and no two rules ever both claim one value. Serves [§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes) and [§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree).

### requirements.5: A mistake in the config fails loudly

An unknown key and an invalid value are errors naming the line, never a silent default ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)), per [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible). A value the key's grammar can never match is an invalid value, not a silent no-op: a `[scan] exclude` entry containing `/` is refused at its line ([§FS-config.3.5.16](FS-config.md#3516-an-exclude-entry-containing--is-refused)). Version 2 refuses at its line three values that version 1 keeps reading under [§FS-config.5.2](FS-config.md#52-a-versions-meaning-moves-only-as-backwards-compatibility-allows), and version 1 meets each at `config migrate`: a string that closes early ([§FS-config-migrate.3.1.8](FS-config-migrate.md#318-a-string-that-closes-early)), an empty string in `[scan] extensions` ([§FS-config-migrate.3.1.9](FS-config-migrate.md#319-an-empty-string-in-scan-extensions)), and a `fetch` that names no executable, which it also meets at `grund fetch` ([§FS-fetch.2.1](FS-fetch.md#21-a-value-that-names-no-executable), [§FS-config-migrate.3.1.10](FS-config-migrate.md#3110-a-fetch-that-names-no-executable)).

One case is deferred: a key written at a scope that does not admit it is refused as an unknown key rather than as not admitted there.

### requirements.6: The effective configuration is inspectable

`grund config show` prints the effective configuration, and what it prints loads back to the same effective values ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)). The round trip is of the **effective** config, not of the authored text: a key the run resolved to its default prints as that default, and a key that is inert under another key's value is still parsed and still printed ([§FS-config.3.1.8](FS-config.md#318-inline_style-and-the-note-budgets)). Which scope a value came from, default, project, row or flag, is deferred; what the output shows of it is one step, a row printing a key only where its effective value differs from what it inherits ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)).

### requirements.7: An upgrade changes an existing config's meaning only as backwards compatibility allows

A new key, or a new scope for an existing key, is additive and does not bump `grund_config_version` ([§FS-config.5.1](FS-config.md#51-new-keys-are-not-a-new-version)); any other change to what an existing config means is [§FS-config.5.2](FS-config.md#52-a-versions-meaning-moves-only-as-backwards-compatibility-allows)'s, and a binary refuses a config whose version it does not know ([§FS-config.5](FS-config.md#5-schema-versioning)). Serves [§REQ-backwards-compatibility](../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly).

### requirements.8: A project's meaning is self-contained

No config value of one project is inherited across a project boundary ([§FS-config.principle.boundary](FS-config.md#principleboundary-the-project-is-the-widest-committed-scope)), and none is read from the machine ([§FS-config.principle.install-local](FS-config.md#principleinstall-local-the-relation-governs-committed-repository-state-only)). What else reaches a run from outside the checkout is not a value: a `[workspace]` block above it whose `members` claims it spells its alias paths, so a qualified citation that passes in the checkout alone can fail inside that workspace ([§FS-workspace.6.1.5](FS-workspace.md#615-alias-paths-do-not-change-with-scope)), and fails the run with exit `2` when it cannot answer ([§FS-workspace.6.1.7](FS-workspace.md#617-a-claiming-block-that-cannot-answer-fails-the-run)); under `respect_gitignore` the machine's global git excludes decide which files are read ([§FS-scan.scope.ignore-files](FS-scan.md#scopeignore-files-respect_gitignore--the-ignore-files)); and a config that declares imports adds the tree each import names, read live at the path the config spells ([§FS-imports.mount](FS-imports.md#mount-an-import-is-a-mounted-root-read-live-and-read-only)). Serves [§REQ-runs-offline](../requirements/REQ-runs-offline.md#req-runs-offline-verification-never-depends-on-an-external-service) and [§FS-workspace.2](FS-workspace.md#2-workspace-configuration).

## 1. File location and discovery

The config file is named **`grund.toml`** and is discovered at **two locations per directory**: the bare `grund.toml` beside the project's own metadata files, then `.agents/grund.toml`. Discovery walks upward from the path argument, or from a file argument's parent directory, which for a bare file name is the working directory, so `grund <command> NAME` and `grund <command> ./NAME` are one run; with no path it starts at the working directory. It probes both names in that order at every level and stops at the first directory where either exists. That directory is the config root, and relative paths inside the config resolve against it, never against `.agents/`. The rule is the same at every level, so a repository root and a workspace member each pick the form that suits them, and a workspace may mix the two ([§FS-workspace.2](FS-workspace.md#2-workspace-configuration)). Per [§DF-config-file-location](../decisions/functional/DF-config-file-location.md#df-config-file-location-grundtoml-is-discovered-at-two-names-per-directory-and-init-writes-the-bare-one).

If neither name is found anywhere up the walk, `grund` runs with the built-in defaults this spec defines, which are stored in no file.

### 1.1 When one directory carries both

The bare `grund.toml` wins, and the `.agents/grund.toml` beside it is read by nothing at all, so the form `grund init` generates ([§FS-init.2.4](FS-init.md#24-generated-grundtoml)) is the form that governs. `grund check` reports the pair ([§FS-check.4.3](FS-check.md#43-redundant-config-pair)), and not the deprecated location as well ([§FS-check.4.11](FS-check.md#411-config-read-from-the-deprecated-agents-location)). Per [§DF-config-file-location.2.2](../decisions/functional/DF-config-file-location.md#22-the-bare-grundtoml-wins-a-tie-and-check-warns-about-the-pair).

### 1.2 The `.agents/` location is deprecated

Both names keep working, and the bare `grund.toml` is the one a project should carry ([§DF-config-file-location.2.4](../decisions/functional/DF-config-file-location.md#24-a-projects-grounding-must-be-visible-from-its-root-listing)): it is the form `grund init` generates ([§FS-init.2.4](FS-init.md#24-generated-grundtoml)). A run whose config resolved to `.agents/grund.toml` is told to move it ([§FS-check.4.11](FS-check.md#411-config-read-from-the-deprecated-agents-location)), and moving is a `git mv` with no other edit ([§DF-config-file-location.2.3](../decisions/functional/DF-config-file-location.md#23-grund-init-writes-the-bare-grundtoml)). Of `.agents/`, which other agent tools may share, `grund` owns only `.agents/grund.toml`.

**No release removes the `.agents/` location**, so the warning names none, unlike the deprecation path of [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): a named release is owed only where something will break, and a repository on `.agents/` is a correct configuration ([§DF-config-file-location.2.5](../decisions/functional/DF-config-file-location.md#25-the-agents-form-is-deprecated-and-never-removed)).

### 1.5 The ancestor search keeps the path through directory links

The discovery start is made absolute against the working directory and searched through its lexical parents; through its last `..` it is the directory the operating system gives, and only the components after that are kept as written. Only where no lexical parent carries a config, as for a link from outside every project into one, are the resolved path's parents searched, so neither a stranger's config above a link's target nor the defaults replace the project the path reaches. This governs every upward config lookup in both versions: discovery, `config explain`, `config migrate`, the workspace `grund init` teaches ([§FS-init.2.3.4](FS-init.md#234-managed-block-content-sections)), an editor folder ([§FS-lsp.2.2.1](FS-lsp.md#221-workspace-folders-anchor-discovery)), and the member a scope lies in ([§FS-scan.projects.discovery](FS-scan.md#projectsdiscovery-the-nearest-config-decides)). Links are resolved for identity and reads only after the config is selected, and the selected config's validation applies, so discovery cannot replace a refusal or an in-scope read failure with successful defaults ([§FS-scan.path.check](FS-scan.md#pathcheck-check-reads-more-than-it-reports), [§FS-check.1.3.6](FS-check.md#136-an-explicit-path-still-narrows)). Per [§DF-config-discovery-keeps-lexical-parents](../decisions/functional/DF-config-discovery-keeps-lexical-parents.md#df-config-discovery-keeps-lexical-parents-config-discovery-climbs-the-paths-lexical-parents-before-resolving-directory-links).

## 2. Precedence

Two sources decide a setting: the committed `grund.toml`, whose values stand over the built-in defaults ([§FS-config.3](FS-config.md#3-keys-version-1)), and the CLI inputs of the one run, which enter at the scope their flag spells rather than as a third source above both ([§FS-config.principle.cli](FS-config.md#principlecli-a-cli-input-enters-at-the-scope-its-flag-spells)). How the scopes inside the file relate is stated once in [§FS-config.principle](FS-config.md#principle-a-setting-written-at-a-narrower-scope-wins). Per [§DF-config-scope-override](../decisions/functional/DF-config-scope-override.md#df-config-scope-override-the-committed-scopes-are-one-relation-stated-once).

## 3. Keys (version 1)

This chapter is the version-1 format, read by the v1 reader ([§FS-config.5.2](FS-config.md#52-a-versions-meaning-moves-only-as-backwards-compatibility-allows)); the version-2 format is [§FS-config-v2](FS-config-v2.md#fs-config-v2-grund-reads-a-version-2-config-by-concern-with-one-strength-vocabulary-and-fixed-defaults), and [§FS-config.concerns](FS-config.md#concerns-every-key-belongs-to-exactly-one-concern) maps the two. Every key is optional, and an omitted key takes its default. An unknown key is an **error**, not a warning: a typo in a config file is a bug ([§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible)).

The config file is TOML, and the recognized surface is the line-oriented subset the keys below use: one `key = value` per line, basic (double-quoted) strings, booleans, integers, and single-line `["…", "…"]` arrays of basic strings, whose elements end only at a comma outside a string ([§FS-config.3.11](FS-config.md#311-a-list-element-ends-at-a-comma-outside-a-string)); `#` comments; `[table]` and `[[array.of.tables]]` headers. Multi-line arrays, inline `{ … }` tables other than the closed `lead_size_warning` table ([§FS-config.3.1.2](FS-config.md#312-lead_size_warning--the-oversized-lead-opt-in)), and other TOML constructs are not parsed. A line that does not fit this shape is an error at that line ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)).

Top-level keys:

```toml
grund_config_version = 1
project_name = "Example" # optional metadata written by `grund init`
project_description = "One line describing what this project is for" # optional
```

`project_name` is free-form metadata that nothing depends on outside a workspace; in one, it names the project's alias, except for a member listed in `optional_members` ([§FS-workspace.3](FS-workspace.md#3-aliases)). A `project_name` that is not a valid alias ([§FS-workspace.1](FS-workspace.md#1-citation-syntax)) loads, and fails at workspace expansion with `invalid workspace project alias <name>`.

`project_description` is the free-form one-line description that generated workspace member lists render beside the project's alias, and nothing else depends on it ([§FS-workspace.3.1](FS-workspace.md#31-project-descriptions), [§DF-workspace-member-descriptions](../decisions/functional/DF-workspace-member-descriptions.md#df-workspace-member-descriptions-member-side-project_description-for-workspace-member-lists)). A value containing a line break (a `\n` or `\r` escape in the TOML string) is a config error at the `project_description` line ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)).

### 3.1 `[reference]` — citation form

```toml
[reference]
marker            = "§"      # default; rare character that prefixes a citation in prose
trigger           = "$$"     # default; typed sequence rewritten to marker by IDE plugin and `grund fmt`
strict            = true     # default; if false, bare citations are also recognized
shorthand         = "canonical" # canonical | accepted; persisted number-only citation policy
require_grounding = false    # default; if true, `check` flags source files that cite no declared ID; …and the default for every [[kinds]] row (§3.4.8)
# grounding_level = 1        # default; 1 = the file — the unit inside each governed file (§3.4.8)
# conversation    = "link"   # optional; committed conversation-rendering opinion — see below
# lead_size_warning = { max = 600, unit = "words" } # optional; oversized lead warning

inline_style                 = "citation-with-note"   # default; alt: "citation-only"; see [§FS-inline-citation-style](FS-inline-citation-style.md#fs-inline-citation-style-configurable-shape-of-inline-code-comment-citations)
inline_note_suggested_lines  = 1                       # soft cap; advisory unless warn_on_suggested = true
inline_note_max_lines        = 3                       # hard cap (error)
inline_note_max_columns      = 100                     # hard cap (error)
inline_note_layout           = "any"                   # default; alt: "citation-first-colon"
inline_note_layout_check     = "off"                   # off | warn | error — how `check` reports a layout deviation
warn_on_suggested            = false                   # if true, soft-cap overruns surface as `check` warnings
```

`marker` may contain word characters, including `_`; where a marked citation then starts is [§FS-citations.recognized.marker-start](FS-citations.md#recognizedmarker-start-the-configured-marker-establishes-the-citation-start). `strict = true` requires a non-empty `marker`; `strict = false` also recognizes bare citations. Per [§DF-reference-marker](../decisions/functional/DF-reference-marker.md#df-reference-marker-use--as-the-reference-marker-with--as-the-typing-trigger).

#### 3.1.1 `shorthand` — persisted number-only citations

`shorthand` is a closed two-value policy for a uniquely resolving, marker-origin number-only shorthand ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)). `canonical`, the default when the key is absent, reports a persisted shorthand and lets `grund fmt` expand it to the full ID; `accepted` permits the shorthand and the full citation to coexist and leaves the shorthand token byte-identical. It does not change recognition or resolution, and it does not apply to trigger input, an unresolved or ambiguous shorthand, or a grammar without both `{number}` and `{slug}`. Any other string, or a value that is not a string, is a load-time configuration error at this key ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)).

#### 3.1.2 `lead_size_warning` — the oversized-lead opt-in

`lead_size_warning` opts the project into the lead-only warning of [§FS-declarations.checks.oversized-lead](FS-declarations.md#checksoversized-lead-oversized-lead-opt-in). Its inline table has exactly two required fields, separated as a list's elements are ([§FS-config.3.11](FS-config.md#311-a-list-element-ends-at-a-comma-outside-a-string)): `max`, a non-negative integer, and `unit`, one of the case-sensitive closed set `lines`, `words` or `bytes`. A missing, extra or duplicate field and any other unit, `tokens` included, are load-time config errors ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)). The key is absent by default and omitted from a fresh `grund init` scaffold; `grund config show` omits it when absent and otherwise prints its canonical inline form, `lead_size_warning = { max = <N>, unit = "<unit>" }`.

#### 3.1.3 `conversation` — one name, two scopes

`conversation` selects how agents render citations in **local conversations**, the answers, reviews and transcripts an agent writes, not the citations on disk ([§DF-repo-conversation-opinion](../decisions/functional/DF-repo-conversation-opinion.md#df-repo-conversation-opinion-repositories-may-commit-a-link-only-conversation-rendering-opinion)). It is absent by default, meaning no opinion, and it changes no scanning, checking or formatting: it only selects entrypoint guidance. The key has one name and two scopes, and the scope decides both who is instructed and which values are legal:

| Where | File | Accepted | Instructs |
| --- | --- | --- | --- |
| Repository *opinion* | the project's `grund.toml` ([§FS-config.1](FS-config.md#1-file-location-and-discovery)) | `link` only | every agent that clones the repo, through the generated entrypoint ([§FS-init.2.3.6](FS-init.md#236-clickable-citations)) |
| User *preference* | `~/.config/grund/config.toml`, resolved like every `~/.config` target ([§FS-integrations.4.1.7](FS-integrations.md#417-where-a--target-resolves)) | `plain` \| `link` | every agent on this machine, through its global instruction file ([§FS-integrations.4.3](FS-integrations.md#43-user-preference-and-global-agent-instructions)) |

The repository value set is closed, with the single member `link` ([§DF-repo-conversation-opinion.2.2](../decisions/functional/DF-repo-conversation-opinion.md#22-only-link-is-committable)), so any other value, `plain` included, is a load-time error ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)), in `grund init` too. `link` makes the declaration's location travel with the citation, in the form [§FS-init.2.3.6](FS-init.md#236-clickable-citations) gives each agent's entrypoint, which the reader's own `conversation_target` may override ([§DF-conversation-link-target.2.3](../decisions/functional/DF-conversation-link-target.md#23-the-target-is-user-scoped-but-the-default-is-committable)). It is the fallback for machines that never stated a preference: a recorded user preference, `plain` included, wins over it ([§FS-integrations.4.3](FS-integrations.md#43-user-preference-and-global-agent-instructions), [§DF-repo-conversation-opinion.2.3](../decisions/functional/DF-repo-conversation-opinion.md#23-precedence)). Repository-web guidance stays fixed in the generated agent entrypoint ([§FS-init.2.3.6](FS-init.md#236-clickable-citations)).

#### 3.1.4 `conversation_target` — how a link addresses its declaration

`conversation_target` selects how a linked citation addresses its declaration. It is **user-scope only**: there is no repository spelling, and setting it in the project's `grund.toml` is the unknown-key error ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)). Its values are `file` (the default), `path`, `web`, `vscode`, `vscodium` and `cursor`; the templates each one fills, and the per-agent gate that decides where the linked form is instructed at all, are [§FS-integrations.4.3](FS-integrations.md#43-user-preference-and-global-agent-instructions), decided in [§DF-conversation-link-target](../decisions/functional/DF-conversation-link-target.md#df-conversation-link-target-the-conversation-link-form-is-a-markdown-link-over-an-absolute-uri-addressed-per-machine). The key is inert unless the effective `conversation` is `link`, and it is still parsed and reported either way. It is also accepted per agent under `[reference.agents.<agent>]`, a partial merged over the machine-wide value ([§FS-integrations.4.4](FS-integrations.md#44-per-agent-overrides)).

#### 3.1.7 `require_grounding` and `grounding_level` — defaults for `[[kinds]]`

`require_grounding = true`, off by default, turns on the `ungrounded` check ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)), which says which files must be grounded and how ([§DF-require-grounding](../decisions/functional/DF-require-grounding.md#df-require-grounding-an-opt-in-check-that-every-source-file-cites-a-spec)). `grounding_level` names the unit inside each governed file, and `1`, the default, is the file; it is inert, and a config error, where nothing turns grounding on ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)). Written here, both keys are the defaults every `[[kinds]]` row inherits ([§FS-config.principle.inventory](FS-config.md#principleinventory-the-settings-admitted-at-both-the-project-and-the-kind-scope)), and a flag enters as [§FS-config.principle.cli](FS-config.md#principlecli-a-cli-input-enters-at-the-scope-its-flag-spells) says.

#### 3.1.8 `inline_style` and the note budgets

`inline_style`, the three budget keys `inline_note_suggested_lines`, `inline_note_max_lines` and `inline_note_max_columns`, and `warn_on_suggested` govern the shape of inline citations in code comments: whether a `§<ID>` token may carry a short rationale, and how long it may run. They bound *inline* comments only; a doc-comment is documentation, so a citation inside one is checked for everything except its shape ([§FS-inline-citation-style.1.1](FS-inline-citation-style.md#11-doc-comments-are-not-sites)). The full contract is [§FS-inline-citation-style](FS-inline-citation-style.md#fs-inline-citation-style-configurable-shape-of-inline-code-comment-citations). `inline_style` is `citation-with-note` (the default) or `citation-only`, and any other value is a load-time error ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)) that names the rejected value and the accepted set:

```text
error: grund.toml:4: unknown [reference] inline_style `note` (expected citation-with-note or citation-only)
```

`inline_note_suggested_lines` may not exceed `inline_note_max_lines`, and `warn_on_suggested` is a boolean; any other value is a load-time error. Under `inline_style = "citation-only"` the three budget keys are inert, since no note is ever permitted, but they are still parsed and printed by `grund config show`.

#### 3.1.9 `inline_note_layout` and `inline_note_layout_check`

`inline_note_layout` adds the third axis of that shape, where the `§<ID>` tokens sit inside the note, and `inline_note_layout_check` selects whether `grund check` reports a deviation and through which channel. The layout is `any` (the default, no constraint) or `citation-first-colon`, the check `off` (the default), `warn` or `error`, and any other value is a load-time error ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)) that names the rejected value and the accepted set, as `inline_style`'s does ([§FS-config.3.1.8](FS-config.md#318-inline_style-and-the-note-budgets)). The layout key is the house style and the check key is the gate, so a project can publish the style to its agents ([§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)) before it starts failing on it. `inline_note_layout_check` is inert under `inline_note_layout = "any"`, and both are inert under `inline_style = "citation-only"`, still parsed and still printed. The canonical form and the per-line rule are [§FS-inline-citation-style.3.3](FS-inline-citation-style.md#33-inline_note_layout--where-the-citations-sit).

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

`format` is a template: `{kind}`, `{number}` and `{slug}` are placeholders, and everything else, `-`, `_`, `.`, `:` or any other character, is literal. `{kind}` is required, and at least one of `{number}` and `{slug}` must appear, because a bare kind would not identify a declaration.

`[id] format` is the repository default. A citable `[[kinds]]` row may set its own `format` ([§FS-config.3.4.10](FS-config.md#3410-format-resolve-and-fetch--external-snapshot-kinds)), which is authoritative for IDs of that kind, and a kind without one keeps the `[id] format` grammar exactly. Every consumer, alias-qualified tokens included, selects the kind from the token by the full-ID or shorthand parser first and then applies that kind's grammar, never the first kind name that prefixes the text, so ordinary slug IDs and numeric ticket IDs coexist without ambiguity. Separator-free formats such as `{kind}{number}` and a non-leading `{kind}` are supported; attributing a malformed token is a separate ASCII punctuation-boundary test ([§FS-citations.recognized.off-format](FS-citations.md#recognizedoff-format-a-marked-token-its-kinds-format-does-not-spell-is-reported-never-dropped)).

#### 3.2.1 The three canonical shapes

| `format`                       | Example ID            | Disambiguator                |
|--------------------------------|-----------------------|------------------------------|
| `"{kind}-{number}-{slug}"`     | `FS-NNN-<slug>`       | number; slug is descriptive  |
| `"{kind}-{number}"`            | `FS-NNN`              | number                       |
| `"{kind}-{slug}"`              | `FS-<slug>`           | slug must be unique per kind |

Without `{number}`, slugs must be unique within each kind: two declarations sharing a kind and slug collide on the same ID and are reported as duplicate declarations ([§FS-check.3](FS-check.md#3-errors-detected)). With `{number}`, slugs are descriptive only and may repeat across declarations with different numbers.

#### 3.2.2 `section_separator` must stay distinguishable

`section_separator` must not collide lexically with any literal in `format` or with `slug_pattern`, and it must not be or contain a `/`, because a citation's alias-path boundary is its **last** `/` ([§FS-config.3.2.3](FS-config.md#323-no-id-contains-a-)): with `section_separator = "/"`, `<§>root/fs-x/1`, section 1 of `fs-x` in project `root`, would read as alias path `root/fs-x` and ID `1`. A separator that collides is refused on load, and one that contains `/` is refused at that key's line.

#### 3.2.3 No ID contains a `/`

No ID the grammar can build may contain a `/`, which belongs to the alias path: a qualified citation splits on its **last** `/`, and so does every command that takes an `<alias>/<ID>` argument ([§FS-workspace.1](FS-workspace.md#1-citation-syntax)). `format` and a `[[kinds]]` `kind` name ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)) contribute literal text, a citable kind's name being the leading component of every ID in it, so neither may contain the character. `number_pattern` and `slug_pattern` are regexes, judged by what they **match**, not what they spell: a pattern that can produce a `/` is rejected even with none in its text (`[^.[:space:]]+`, `.+`, `[^[:space:]]+`), and one that names the character to exclude it (`[^/.]+`) loads. The refusal is on load at the offending key's line, and its message says *must not contain* for a literal key and *must not match* for a pattern.

#### 3.2.4 Each pattern compiles on its own

`number_pattern` and `slug_pattern` must each be a valid regex **on its own**, not merely once spliced into the ID pattern, because the number-only shorthand derives a narrower pattern from a subset of the format's components ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)). Two that balance only against each other, `number_pattern = "("` with `slug_pattern = "a)"`, are refused on load with the underlying regex error.

#### 3.2.5 Off-grammar declarations stay readable

The effective format is the **authoring and conformance grammar**, not a reason to make a persisted declaration unreadable. A heading in declaration position whose exact token starts with a configured citable kind, ends at the declaration colon, and carries no `/` ([§FS-config.3.2.3](FS-config.md#323-no-id-contains-a-)) stays in that project's catalog even when the token does not match its kind's effective format: its exact written spelling, body, sections and location stay readable, and it earns the `declaration-near-miss` finding ([§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss)). A marker-prefixed token the grammar rejects becomes a citation only when that same project's catalog holds an exact declaration of that spelling; an unmarked one, or a marked one with no exact declaration, gains no compatibility meaning. Such a marked token still cites nothing, but where it opens with a configured kind and no other reading claims its marker, `check` reports it as an off-format citation ([§FS-citations.recognized.off-format](FS-citations.md#recognizedoff-format-a-marked-token-its-kinds-format-does-not-spell-is-reported-never-dropped)). Decided in [§DF-off-grammar-declaration-compatibility](../decisions/functional/DF-off-grammar-declaration-compatibility.md#df-off-grammar-declaration-compatibility-persisted-declarations-remain-readable-without-relaxing-the-authoring-grammar).

#### 3.2.6 Resolving an off-grammar citation

A token the configured grammar reads as a full ID resolves as one first. Otherwise an exact off-grammar declaration and the number-only shorthand are considered together: no target keeps the invalid-ID result, one target resolves, and more than one fails as ambiguous. For a possible inline section, an exact whole-token declaration wins; otherwise every catalog prefix followed by the configured section separator is considered, and competing valid readings fail. Duplicate exact declarations keep the ordinary duplicate and ambiguity behavior, with sorted sites.

#### 3.2.7 `named_sections` — the gate for explicit section names

`named_sections` is an absent-by-default boolean gate for explicit section names. Absent or `false`, section scanning, citation recognition, queries, formatting, completion, the LSP and operational output read section paths as numbers only. With `true`, a named component has the fixed, configuration-independent grammar `[a-z][a-z0-9-]*`, derived neither from `slug_pattern` nor from the displayed heading title. `grund init` writes the teaching default `named_sections = false` ([§FS-init.2.4](FS-init.md#24-generated-grundtoml)). Any other value is invalid config. A relative heading whose one component is a name needs the gate, as a complete-path heading carrying that name does, and one whose component is a number needs nothing ([§FS-declarations.section.relative.form](FS-declarations.md#sectionrelativeform-the-form)).

#### 3.2.8 Numeric spelling survives reading and reporting

For an ID the effective grammar accepts, the authored digits of `{number}` are its spelling, and reading and reporting keep it, leading zeroes included, in text reports, machine-readable ID strings, diagnostics and suggestions alike, under the repository default and a per-kind override ([§FS-config.3.4.10](FS-config.md#3410-format-resolve-and-fetch--external-snapshot-kinds)) and in version 2 ([§FS-config-v2.schema.1](FS-config-v2.md#schema1-schema)). A citation keeps its own visible spelling and does not rename its target, and a generated link anchors on the target's spelling, whatever the citation's ([§FS-fmt.6.2.2](FS-fmt.md#622-the-anchor-slugifies-the-headings-rendered-text)): `# ADR-0001: Use Postgres` yields `#adr-0001-use-postgres` even when cited as `ADR-1`.

Spelling is not identity: `ADR-1`, `ADR-001` and `ADR-0001` select the same kind and number, so declaring two of them is a duplicate ([§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration)), numeric fields stay numbers, and lookup and collisions use the numeric identity. Only allocation chooses a minimum width ([§FS-id.1.2](FS-id.md#12-number-width)): reading a four-digit declaration changes no allocation width, default or explicit, and repads no heading, file name or citation ([§DF-id-number-width.4](../decisions/functional/DF-id-number-width.md#4-what-happens-at-the-rollover)).

#### 3.2.9 Kind head

A kind's **head** is its name followed by the literal immediately after a leading `{kind}` in its effective format, its own override when present and otherwise the inherited repository default. An empty following literal yields the bare name, as in `{kind}{number}`, and a format that does not begin with `{kind}` has no known head. Thus `GR` with `{kind}-{number}` has head `GR-`, and `GRUND` with `{kind}-{slug}` has head `GRUND-`. Collision validation ([§FS-config.3.4.5](FS-config.md#345-name-rules)) and coordinate-glob notes ([§FS-coordinate-globs.errors.note](FS-coordinate-globs.md#errorsnote-a-pattern-no-kind-can-start-gets-a-note)) share this definition.

### 3.3 Section paths — arbitrary nesting depth

Section coordinates are **dotted paths of any depth**, with no maximum:

```
§FS-check.3
§FS-check.3.1
§FS-check.3.1
§FS-check.3.1
```

A citation's section path must name a citable heading of the declaration at that depth, so four-deep numeric nesting (`## 1.`, `### 1.1`, `#### 1.1.1`, `##### 1.1.1.1`) needs no config.

#### 3.3.1 Named components

With `named_sections = true` ([§FS-config.3.2.7](FS-config.md#327-named_sections--the-gate-for-explicit-section-names)), a path containing a named component uses the explicit colon form `<path>: <title>`. A heading writes its complete path or, in a Markdown declaration body, a relative path: one `.` and one component, the rest supplied by its parent ([§FS-declarations.section.relative](FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds)). Only the complete path is citable. The title declares nothing and may change without changing the coordinate:

```markdown
## goals: Goals
### goals.performance: Performance
### goals.3: Third ordered goal
```

Named components may occur at every depth, under a name or under a number (`goals.performance.latency`, `1.goals`), and a number may follow a name (`goals.3`), positional beneath that stable name. A path in which a name follows a number at any depth (`1.goals`, `goals.3.latency`) is a **mixed path**: it is positional beneath its number, so renumbering section `1` moves `1.goals` to `2.goals`, and once cited it is recorded in the section lock, so the move is reported rather than silently retargeting the citation ([§FS-section-lock.holds.entry](FS-section-lock.md#holdsentry-one-line-per-locked-address)). A mixed path is one address: `1.goals` is never an alias for section `1` ([§DF-number-name-sections](../decisions/functional/DF-number-name-sections.md#df-number-name-sections-a-name-may-follow-a-number-1goals-and-a-cited-address-with-one-is-locked)). Every proper prefix of a name-bearing path must itself be recorded in the same declaration: `### goals.performance: …` requires `goals`, and `### missing.performance: …` is an orphan even if its Markdown placement looks nested under another heading ([§FS-declarations.checks.orphan-section](FS-declarations.md#checksorphan-section-orphan-name-bearing-section-path)); a relative heading always needs its actual parent, numeric or named ([§FS-declarations.section.relative.parent](FS-declarations.md#sectionrelativeparent-the-parent)). A purely numeric heading keeps its optional trailing period and title syntax; the colon form is mandatory whenever any component is named.

#### 3.3.2 `section_heading_levels` — heading depth against path depth

`section_heading_levels` sets how a Markdown heading's depth must line up with its dotted section path. `"strict"`, the default, requires the heading depth to equal the declaration heading's depth plus the number of path components, named or numeric, so under an H1 declaration `## 1.1 …` and `## goals.performance: …` are `section heading level mismatch` errors in `grund check` ([§FS-declarations.checks.section-heading-level](FS-declarations.md#checkssection-heading-level-section-heading-level-mismatch)). `"warn"` reports the same mismatch as a warning. `"loose"` lets any deeper heading declare a syntactically legal complete path. Any other value is invalid config. Depth is judged on the canonical path: a relative heading sits one level below its parent by construction, so it has no depth finding of its own, and Markdown's six levels bound how far relative headings nest, not how long a path is ([§FS-declarations.section.relative.parent](FS-declarations.md#sectionrelativeparent-the-parent)).

The mode does not govern an ATX heading with no coordinate: inside a Markdown declaration body, [§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading) reports a deeper ATX heading that is neither a declaration nor a recognized section, and no key sets its severity or turns this project-wide rule off ([§DF-unmarked-markdown-headings](../decisions/functional/DF-unmarked-markdown-headings.md#df-unmarked-markdown-headings-in-body-markdown-atx-headings-participate-in-the-knowledge-graph)).

#### 3.3.4 The outer section separator

The default `section_separator` is `.`; a project may set `:` (`<§>FS-check:3.1.2`) or `#` (`RFC-42#3.1.2`) instead. The dotted **components** stay separated by `.` whatever the outer separator, which keeps the section grammar regular at any depth, and so does the leading `.` of a relative heading ([§FS-declarations.section.relative.form](FS-declarations.md#sectionrelativeform-the-form)).

### 3.4 `[[kinds]]` — recognized kinds

One `[[kinds]]` table per kind. `kind` is its name — mandatory, and what everything else keys on: `[citations.<kind>]` ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)), `grund list --kind`, and, for a kind that declares IDs, the literal prefix of every ID in it.

A kind is either *multi-file* (`folder = "<dir>"`) — each declaration is the H1 of its own file under `<dir>` — or *single-file* (`file = "<path>"`) — every declaration of the kind is a heading inside that one document. Setting both `folder` and `file` on the same kind is invalid; setting neither leaves the kind with no configured home. Every configured home is scanned whether or not `[scan] include` names it ([§FS-scan.scope.homes](FS-scan.md#scopehomes-every-configured-kind-home-is-scanned)), except one marked `scan = false` ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned)).

#### 3.4.1 `citable` — kinds that declare no IDs

A kind has two independent properties: it has a home, and it declares IDs. `citable = false` (default `true`) turns the second off, so a non-citable kind with a home is a **place** and nothing more. It is for directories that hold agent-facing content rather than specification — skills, prompt libraries, runbooks, test suites — whose citations are checked and directed like citations anywhere else ([§DF-non-citable-kinds.2.1](../decisions/functional/DF-non-citable-kinds.md#21-citable--false-on-the-existing-kinds-table)).

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

A non-citable kind keeps:

- **A home**, `folder` or `file`. An entry that leaves both out is the homeless kind instead ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)).
- **A row in the generated Project map and in the generated citation directions**, named by its place, never by its name ([§FS-init.2.3.4](FS-init.md#234-managed-block-content-sections), [§FS-init.2.3.5](FS-init.md#235-citation-directions)).
- **Citation-direction rules.** A citation inside its home counts as that kind when exactly one home contains its file, even where no declaration encloses it ([§FS-citations.citing-kind](FS-citations.md#citing-kind-which-kind-a-citation-site-counts-as)), and an obligation is asked per file ([§FS-check.3.11](FS-check.md#311-missing-required-citation)).
- **Grounding** of the files of its home, Markdown included ([§FS-check.3.6.1](FS-check.md#361-which-files-a-row-governs)), asked by the row's grounding keys ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)).

It loses:

- **The ID grammar.** `<name>-<slug>` is not an ID and never tokenizes as a citation. The name is left out of the `KIND ∈ {…}` vocabulary line, and `grund list --kind` and `grund id` refuse it as a kind that declares no IDs rather than as an unknown one ([§FS-list.1](FS-list.md#1-inputs), [§FS-id.1](FS-id.md#1-inputs), [§FS-errors.2.5.1](FS-errors.md#251-a-kind-that-declares-no-ids)).
- **Declarations.** Its home admits none: a declaration inside it, where it is the file's only home, is a misplaced declaration ([§FS-declarations.checks.misplaced-declaration](FS-declarations.md#checksmisplaced-declaration-misplaced-declaration-configured-kind-home)).
- **An index.** Setting `index` on it is a config error ([§FS-config.3.4.2](FS-config.md#342-index--the-kinds-index-file)).
- **Being cited.** A `[citations.<kind>]` rule may not name it as a target ([§FS-config.3.9.5](FS-config.md#395-validation)).

A non-citable kind whose files ship verbatim elsewhere sets `scan = false` as well ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned)).

#### 3.4.2 `index` — the kind's index file

`index` names the kind's **index file** — the document under `folder` that must list every declaration in it ([§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)). It is resolved relative to `folder` and takes a file name or `false`, which is for a folder whose declarations are exercised rather than navigated, such as a corpus of e2e cases:

```toml
[[kinds]]
kind = "DF"
folder = "docs/decisions/functional"
# index = "README.md"   # the default, relative to `folder`
# index = false         # opt out — the folder is not navigated
# index = "INDEX.md"    # or name a different file
```

The default is keyed on the kind's name, for a built-in kind and a declared one alike: `E2E` defaults to `index = false`, and every other citable folder kind to `README.md` ([§DF-index-compatibility-ramp.3](../decisions/functional/DF-index-compatibility-ramp.md#3-consequences)), the file a reader arriving at the folder opens and `grund init --scaffold` writes ([§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place)).

`index` requires `folder` and a citable kind. On a single-file kind, on a kind with no configured home, or on a `citable = false` kind it is a config error ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)), and so is `index = true`, which names no file.

A named `index` must be **a relative path inside `folder`, naming a file the configuration reads as Markdown** ([§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule)). An absolute path, one that climbs out with `..`, and `.` are config errors in both versions: the value is joined onto `folder` and must name a file inside it, the boundary [§FS-fmt.2.3.2](FS-fmt.md#232-a-link-that-leaves-the-config-root-is-not-written-through) holds a rewrite to, for the reason [§REQ-no-data-loss.2](../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own) gives. Whether the file is read as Markdown is asked of its reading, never of its name ([§DF-one-reading-answer](../decisions/functional/DF-one-reading-answer.md#df-one-reading-answer-whether-a-file-is-read-as-markdown-or-has-its-docstrings-read-is-the-configurations-one-answer-never-its-name-and-every-home-of-a-row-is-judged-by-it)). Version 1 reads a file as Markdown when its final extension is `md`, so a named `index` not named `.md` is a config error. Version 2 asks the language the name resolves to: `index = "index.txt"` under `txt = "markdown"` is an index, and a named `index` that is not read as Markdown — `index.md` under `languages = ["rust"]`, or `index = "x.rs"` — is refused at load at the key ([§FS-config-v2.checks.config-index-not-markdown](FS-config-v2.md#checksconfig-index-not-markdown-a-named-index-is-not-read-as-markdown)). The default, `README.md` where the row writes no `index`, is judged the same way and draws no warning: where it is not read as Markdown the folder is judged as under `index = false`, because the author named nothing ([§DF-markdown-homes-refused-in-1-0.2.5](../decisions/functional/DF-markdown-homes-refused-in-1-0.md#25-only-a-named-index-is-refused)).

#### 3.4.3 `title`

`title` is human-readable metadata. It is the text of the kind's Project map row ([§FS-init.2.3.4](FS-init.md#234-managed-block-content-sections)), which for a non-citable kind is the only thing that says what the place is for, and it surfaces in `grund list --summary --format json` ([§FS-output-shapes.5](FS-output-shapes.md#5-list---formatjson)), in show and refs JSON as `kind_title`, and in IDE hover previews; `md` does not inject it ([§FS-show.3.1.2](FS-show.md#312-md)).

`kind_title` is the `title` of the kind the queried target resolves to, in the project that target is in: a default kind's built-in title counts, a qualified workspace query reads the target project's kinds, and each query of a batch reads its own target's. For declaration and section show objects, a present title, even an empty string, adds `kind_title` immediately before the terminal `path`, `line` pair. An absent effective title omits the field; it is neither `null` nor a fallback kind name. Where each other object carries the field, and which carry none, is [§FS-output-shapes.4](FS-output-shapes.md#4-show---formatjson), [§FS-show.2.4.2](FS-show.md#242-the-manifest-as-json) and [§FS-refs.3.2](FS-refs.md#32---format-json)'s. Hover appends it as a separate literal Kind paragraph ([§FS-lsp.1.2](FS-lsp.md#12-hover-preview)). Decided in [§DF-configured-title-metadata](../decisions/functional/DF-configured-title-metadata.md#df-configured-title-metadata-kind-titles-are-separate-target-metadata).

#### 3.4.4 The default kinds

The defaults declare these nine, in this order, for a project with no config and for a `grund.toml` that omits `[[kinds]]` alike:

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

`GRUND` is the H1 of the single file `docs/grund.md`, `GOAL` and `RM` declarations are H2 headings inside their single files, and `FS`, `AR`, `DF` and `DA` declarations are each the H1 of a file in their `folder`.

A project that writes `[[kinds]]` replaces this list entirely — there is no merge ([§FS-config.principle.unit](FS-config.md#principleunit-one-leaf-key-overrides-and-one-value-is-whole)). The list has no `REQ`: the `software-development` template declares one, with no built-in default behind it ([§FS-init.2.4.3](FS-init.md#243-every-written-key-differs-from-its-default)). The two test kinds are non-citable, because a test cites the claim it proves and is never cited back, and lowercase, because their names never appear in an ID ([§DF-non-citable-kinds.3](../decisions/functional/DF-non-citable-kinds.md#3-consequences)).

`E2E` is not a default and is fully supported: a repository whose e2e suite is a corpus of case directories declares it (`kind = "E2E"`, `folder = "e2e/cases"`, `index = false`) and gets the case-declaration machinery of [§FS-declarations.e2e-case](FS-declarations.md#e2e-case-what-an-e2e-case-directory-declares) — `E2E-<case>` IDs, `grund <ID>` over the case manifest, per-case obligations, and the fixture-tree pruning that keeps a nested case repo out of the host scan. That machinery follows the configured `E2E` home, so a config that wants it names it ([§DF-non-citable-kinds.3](../decisions/functional/DF-non-citable-kinds.md#3-consequences)).

A v1 config that writes the top-level key `kinds = []` declares an empty kind set: no row, no ID namespace, no home, and none of the defaults above, where omitting `[[kinds]]` keeps them. It is how a repository that declares its own kinds starts ([§FS-init.1.6](FS-init.md#16-templates)), and the one kind set with no citable kind that v1 loads: a `[[kinds]]` table whose every row sets `citable = false` is refused. Only the empty array is accepted. A non-empty one is refused at its line with the instruction to write `[[kinds]]` tables, and a `[[kinds]]` table after `kinds = []` is refused at its header, as TOML refuses a table appended to a static array. A version-2 file needs no key for this: one with no `[schema.kinds]` row has no kinds ([§FS-config-v2.defaults](FS-config-v2.md#defaults-the-v2-epoch)). What each command does with no kinds is its own point's: the scan ([§FS-scan.scope.homes](FS-scan.md#scopehomes-every-configured-kind-home-is-scanned)), citations ([§FS-citations.recognized.no-kinds](FS-citations.md#recognizedno-kinds-with-no-kind-no-token-is-a-citation)), `grund id` ([§FS-list.1.1](FS-list.md#11---kind)) and the managed block ([§FS-init.2.3.4](FS-init.md#234-managed-block-content-sections)).

#### 3.4.5 Name rules

**Names are unique across the whole table**, citable and non-citable rows alike: `[citations.<kind>]` and `grund list --kind` key on a name. Validation then considers every pair of citable kinds in this order:

1. If neither name is a proper prefix of the other, this pair passes, under any format, a non-leading `{kind}` included.
2. For a proper-prefix pair, the character immediately after the shorter name in the longer name must be an ASCII letter or digit. Otherwise the pair is refused: malformed-token attribution could name both kinds. `DA` beside `DA-X` is invalid even with different format separators; rename a kind.
3. Both effective heads ([§FS-config.3.2.9](FS-config.md#329-kind-head)) must be known, and neither head may prefix the other. Otherwise the pair is refused. Choose unambiguous formats; an arbitrary separator is not necessarily enough.

Thus `DA` beside `DAT` loads under `{kind}-{slug}`, as do `GR` tickets with `{kind}-{number}` beside `GRUND` with `{kind}-{slug}`. Separator-free shorter `GR` beside `GRUND` is refused because head `GR` prefixes `GRUND-`. Non-citable kinds are exempt: `skill` beside citable `SKI` loads. Decided in [§DF-format-aware-kind-collisions](../decisions/functional/DF-format-aware-kind-collisions.md#df-format-aware-kind-collisions-admit-prefix-related-kinds-when-their-formats-distinguish-them).

A refused pair is one load error ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)) that names the pair, says `(one is a prefix of the other)`, and ends in a reason that asks for a rename after a name-boundary collision, or for unambiguous formats after an unknown or overlapping head ([§FS-errors.3](FS-errors.md#3-message-text)).

`code` is reserved to the homeless kind ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)): a row may take it only by *being* that kind — `citable = false`, no `folder`, no `file` — because any other row wearing it would collide with the kind every citation outside a home resolves to.

#### 3.4.6 `prefix` — refused, in favor of `kind`

A `[[kinds]]` entry that writes `prefix`, with or without `kind` beside it, is refused with an ordinary config error ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)) located at the line `prefix` is written on, naming `kind` as the key to write instead ([§DF-non-citable-kinds.2.4](../decisions/functional/DF-non-citable-kinds.md#24-the-field-is-a-kind-not-a-prefix)):

```text
error: grund.toml:4: [[kinds]] `prefix` was removed in grund 0.13.0 — rename it to `kind`
```

#### 3.4.7 `scan` — a place that is listed, not scanned

`scan = false` (default `true`) keeps a non-citable kind's home out of the scan. The kind is still a **place**: it gets its Project map row with its title ([§FS-init.2.3.4](FS-init.md#234-managed-block-content-sections)), so an agent is told the directory exists and what it is for — and nothing in it is read, so nothing in it is checked, and the managed block gives it no directions bullet and leaves it out of the grounding sentence ([§FS-init.2.3.5](FS-init.md#235-citation-directions)). It is for content that ships verbatim somewhere else, such as scaffold templates, embedded assets and example configs, where a `§` citation would dangle in every tree it is copied into. Decided in [§DF-unwalked-kind-home](../decisions/functional/DF-unwalked-kind-home.md#df-unwalked-kind-home-a-kind-may-be-a-place-that-is-listed-but-not-walked).

```toml
[[kinds]]
kind = "template"
folder = "templates"
citable = false
scan = false
title = "Init scaffold templates: what grund init writes, verbatim"
```

`grund config show` ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)) prints `scan = false` where it is set and nothing where it is not.

The home is left out of the scan roots, *and* pruned when a scan meets it on the way down — under the config root, or under an `include` entry it sits inside, as `docs/templates` sits inside `docs`. An `include` entry that names the home itself does not scan it either: where the two disagree, the narrower key, the one written on the kind, wins. A `file` home is unscanned on the same terms as a `folder` one. The key describes the default scope: an explicit path argument reads the home anyway, as `grund check docs/templates` does ([§FS-scan.path.check](FS-scan.md#pathcheck-check-reads-more-than-it-reports)), and `grund check --full` reads it as territory nobody configured, outside the default scope even where a scan root encloses it ([§FS-scan.scope.not-a-fence](FS-scan.md#scopenot-a-fence-include-is-a-scan-scope-not-a-fence), [§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only)).

Three combinations are config errors ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior), [§DF-unwalked-kind-home.2.2](../decisions/functional/DF-unwalked-kind-home.md#22-only-on-a-non-citable-kind-with-a-home), [§DF-unwalked-kind-home.2.3](../decisions/functional/DF-unwalked-kind-home.md#23-no-citation-rules-on-it)):

- `scan = false` on a **citable** kind, whose declarations would be invisible rather than declared;
- `scan = false` with **no home**, since what of the homeless kind is scanned is `[scan] include`'s to say;
- a `[citations.<kind>]` table naming an unscanned kind as the **citing** kind, a rule that could never fire.

#### 3.4.8 `require_grounding` and `grounding_level` — grounding per place and per level

Two keys say, per kind, **whether** the files of a place must cite a declared ID and **how finely** that is asked. Each has a `[reference]` twin ([§FS-config.3.1.7](FS-config.md#317-require_grounding-and-grounding_level--defaults-for-kinds)) that is the default for every row not setting it, and the row wins; `grund config show` prints a row's key only where it differs from the effective global ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)). Which files a row governs is [§FS-check.3.6.1](FS-check.md#361-which-files-a-row-governs)'s predicate, and the units and their findings are [§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)'s. Decided in [§DF-require-grounding.4](../decisions/functional/DF-require-grounding.md#4-grounding-per-place-and-per-level).

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

`grounding_level` is an integer in Markdown heading levels, `1..=6`: `1` is the whole file, text before its first heading included; `2` adds every `##` subtree, `3` every `###` as well, and `6` every heading Markdown can have. A source file has no headings, so it gets the two ranks grund can see by indentation ([§FS-check.3.6.2](FS-check.md#362-the-unit)). `[citations]` obligations are asked of the same unit ([§FS-check.3.11](FS-check.md#311-missing-required-citation)).

`grund check --require-grounding` ([§FS-check.1](FS-check.md#1-inputs)) is the run-level spelling of the global boolean and sets the same default, so an explicit `require_grounding = false` on a row wins over the flag. The flag moves findings only; the managed block reads the key ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)). The level comes from config only — there is no flag for it. The homeless kind ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)) takes both keys as any row does: written on its row they govern the complement alone, which is how a project grounds its source tree and nothing else, and a config that never declared it writes the row to set them.

Five combinations are config errors, reported per [§FS-config.4.3](FS-config.md#43-invalid-config-behavior) at the offending line ([§DF-require-grounding.4.3](../decisions/functional/DF-require-grounding.md#43-consequences)):

- `require_grounding = true` on a `scan = false` row ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned)).
- Either key on a **citable** `file = "<path>"` row. A **non-citable** `file` row takes both: its one document is governed like every other file of a non-citable home and cut into heading subtrees like any Markdown file ([§FS-check.3.6.1](FS-check.md#361-which-files-a-row-governs), [§FS-check.3.6.2](FS-check.md#362-the-unit)).
- `grounding_level` outside `1..=6`, on a row or in `[reference]`.
- `grounding_level` on a row whose **effective** `require_grounding` is off — written `false` on the row, or inherited off from `[reference]`.
- `[reference] grounding_level` where the global boolean is off and no row turns grounding on.

#### 3.4.9 `values` — first-class value declarations

`values = true` opts whole declarations in this row's home into [§FS-values](FS-values.md#fs-values-opted-in-kinds-bind-authored-components-to-one-declared-value); it is absent and false by default, `grund config show` prints it only where enabled ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)), and it is refused on a row that also sets `value_chapter` ([§FS-config.3.4.13](FS-config.md#3413-value_chapter--the-chapter-whose-named-children-are-values)). An enabled row must be citable and normalize its homes inside the project root, every one of them. In version 1 it has exactly one existing `file` or `folder`. In version 2 its homes sit under one key, `files` or `folders`, as many as it lists, and every one must exist, each judged on its own: a row that lists both keys is refused with version 1's message for a row without exactly one home; an absent first entry is refused as version 1's one home is; and an absent later entry, or one in the wrong shape, outside the root, or that cannot be looked up, is refused at the key that lists it, in one line ending `or drop the entry` ([§FS-config-v2.checks.config-value-home-absent](FS-config-v2.md#checksconfig-value-home-absent-a-value-rows-later-home-does-not-exist), [§FS-config-v2.checks.config-value-home-invalid](FS-config-v2.md#checksconfig-value-home-invalid-a-value-rows-later-home-is-the-wrong-shape-outside-the-root-or-not-looked-up), [§DF-markdown-homes-refused-in-1-0.2.4](../decisions/functional/DF-markdown-homes-refused-in-1-0.md#24-a-later-value-home-keeps-its-own-message)). Each violation is a located config error, checked without parsing declaration content. Version 2 spells the key `form = "value"`, with the same meaning and prerequisites ([§FS-fields.forms](FS-fields.md#forms-form-lowers-to-the-version-1-contracts)).

The home is the complete boundary of its value sources ([§FS-values.2.2](FS-values.md#22-declarations-from-value-sources)). The `<!-- grund:value -->` section suffix is a separate authority: it makes a section a value root in any supported scanned declaration, independently of this key and its home, and neither opts the enclosing declaration in nor changes value-source discovery ([§FS-values.2.4](FS-values.md#24-embedded-section-value-roots)). `value_chapter` is the third, taken per kind ([§FS-values.2.5](FS-values.md#25-chapter-declared-value-roots)).

#### 3.4.10 `format`, `resolve`, and `fetch` — external snapshot kinds

A citable kind may override the repository grammar and describe how a missing committed snapshot is materialized:

```toml
[[kinds]]
kind = "TICKET"
file = "docs/tickets.md"
title = "External tickets (generated snapshots)"
format = "{kind}-{number}"
resolve = "should"
fetch = "scripts/fetch-ticket"
```

`format` is [§FS-config.3.2](FS-config.md#32-id--id-grammar)'s `format` written at the kind scope, with the same template placeholders and the repository's `number_pattern` and `slug_pattern` validation: it overrides only this kind, and `[id].format` remains the default for every other kind ([§FS-config.principle.rungs](FS-config.md#principlerungs-the-committed-scopes-are-a-relation-not-a-closed-list)). It is valid without `resolve` or `fetch`, but invalid on a non-citable kind. Numeric spelling follows [§FS-config.3.2.8](FS-config.md#328-numeric-spelling-survives-reading-and-reporting): this override sets a grammar, not a report width.

`fetch` names the integration executable of [§FS-fetch.2](FS-fetch.md#2-integration-invocation). A value that names no executable ([§FS-fetch.2.1](FS-fetch.md#21-a-value-that-names-no-executable)) loads ([§FS-config.5.2](FS-config.md#52-a-versions-meaning-moves-only-as-backwards-compatibility-allows)) and counts as `fetch` being present. `grund fetch`, `config migrate` ([§FS-config-migrate.3.1.10](FS-config-migrate.md#3110-a-fetch-that-names-no-executable)) and `grund check` ([§FS-check.3.1](FS-check.md#31-dangling-citation)) each print the key's one repair, `grund check` as the tail of a missing ID's message for the kind. `resolve` is the target-side obligation used when that kind's citation has no declaration: the closed enum is `must | should`, with no `may`. It selects one of two fixed finding classes rather than remapping severity — `must` the `dangling` error ([§FS-check.3.1](FS-check.md#31-dangling-citation)), `should` the `missing-snapshot` warning ([§FS-check.4.12](FS-check.md#412-missing-snapshot)) — independently of the citing-side `[citations]` rules ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)). `fetch` without `resolve` has the effective value `must`, and an explicit `resolve` is valid only beside `fetch`. Both keys require exactly one `file` or `folder` home, and are invalid on a non-citable kind.

`config show` ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)) prints an explicit `format` and `fetch`, and the effective `resolve` of a fetch-enabled kind, including the defaulted `must`, so the shown TOML loads back to the same effective values.

#### 3.4.11 What a home is used for

A `folder` is what `grund id` reports as the kind's `folder` field, and a kind with no configured home reports none ([§FS-id.2.2](FS-id.md#22---format-json)); editors use it for their "create new declaration" and "go to home folder" actions. Every configured `folder` and `file` is also a declaration-home boundary, and a file that zero or several homes cover has no expected kind, so the boundary says nothing about it ([§FS-declarations.checks.misplaced-declaration](FS-declarations.md#checksmisplaced-declaration-misplaced-declaration-configured-kind-home)). Declarations are recognized outside configured homes too, inline source declarations included.

#### 3.4.12 `rules` — rule declaration kinds

`rules = true` opts every declaration of a citable Markdown kind into the rule contract of [§FS-rules](FS-rules.md#fs-rules-grounded-declarations-state-and-enforce-chapter-rules): the declaration title is one executable controlled-English sentence and its body is a non-empty rationale. It is absent and false by default, `config show` ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)) prints it only when enabled, and several local kinds may enable it.

The key requires a citable kind and `file` or `folder` homes whose declarations are Markdown: each home, not only the first, is asked whether it is read as Markdown ([§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule)), never what it is named. Version 1's `.md` name test is that question; version 2's refusals of a home read otherwise are [§FS-config-v2.schema.sources.rules](FS-config-v2.md#schemasourcesrules-every-home-of-a-rule-kind-is-judged-by-its-reading). It is invalid on a non-citable, homeless, unscanned, external-snapshot or value-source kind, each of which declares no IDs, supplies no scanned rule catalog, or gives its title another source contract. These relationships are config validation; sentence titles are scanned declaration data, validated later as located findings ([§FS-rules.4](FS-rules.md#4-validation-lifecycle)). Version 2 spells it `form = "rule"`, with the same meaning and prerequisites ([§FS-fields.forms](FS-fields.md#forms-form-lowers-to-the-version-1-contracts)).

#### 3.4.13 `value_chapter` — the chapter whose named children are values

`value_chapter` names, for one `[[kinds]]` row, the chapter whose named direct children are value roots in every declaration of that kind ([§FS-values.2.5](FS-values.md#25-chapter-declared-value-roots)):

```toml
[[kinds]]
kind = "AR"
folder = "docs/architecture"
value_chapter = "values"
```

The value is a section name: a string matching the fixed named-component grammar `[a-z][a-z0-9-]*` of [§FS-config.3.2.7](FS-config.md#327-named_sections--the-gate-for-explicit-section-names), never a displayed heading title and never a `slug_pattern` match. The key is absent by default, absent and set to nothing mean the same, and `config show` ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)) prints it only on a row that sets it.

An enabled row must be citable, must have exactly one existing `file` or `folder` home normalizing inside the project root, and must not set `scan = false` or `values = true`: no coordinate, declaration or read file could hold the chapter otherwise, and a whole-declaration value admits no named chapter ([§FS-values.2.1](FS-values.md#21-markdown-declarations)). `[id] named_sections = true` is a prerequisite: a project that sets `value_chapter` without it is a located config error naming the missing gate, not a silent no-op ([§FS-config.3.2.7](FS-config.md#327-named_sections--the-gate-for-explicit-section-names)). Each violation is a located config error, checked without parsing declaration content; a chapter a declaration fills wrongly is reported later as a located finding ([§FS-check.3.20](FS-check.md#320-invalid-value-declaration)). Version 2 spells it as a field with `content = "values"`, which lowers to the same record ([§FS-fields.content.values](FS-fields.md#contentvalues-a-values-field-is-the-value-chapter)); a row that also sets `fetch` has no version-2 spelling, because a fetched kind takes no fields ([§FS-config-migrate.3.1.7](FS-config-migrate.md#317-settings-this-grund-has-no-v2-spelling-for-yet)).

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

Which files these keys make a run read is [§FS-scan.scope](FS-scan.md#scope-which-files-a-run-reads). `comment_prefixes` are the tokens recognized when looking for source declarations and citations in source files, composed with `extensions` ([§FS-config.3.5.14](FS-config.md#3514-comment_prefixes-compose-with-extensions)); `docstring_python` enables Python triple-quoted-string scanning in addition to `#` comments. The `extensions` default is `md` plus every extension grund's built-in language table names ([§FS-scan.scope.extensions.default](FS-scan.md#scopeextensionsdefault-the-extensions-default-follows-the-language-table)). The `include` default names `requirements.md`, though no built-in kind homes there ([§FS-config.3.4.4](FS-config.md#344-the-default-kinds), [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)).

An opted-in kind's value sources ([§FS-values.terms](FS-values.md#terms-terms)) are catalog input rather than part of this scan, and this table touches them only through the YAML claim: a `.yaml` or `.yml` file whose extension is in the effective `extensions` is scan input and so not a value source, while a JSON one never is ([§FS-config.3.4.9](FS-config.md#349-values--first-class-value-declarations), [§FS-values.2.2](FS-values.md#22-declarations-from-value-sources)).

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

See [§FS-scan.scope.homes](FS-scan.md#scopehomes-every-configured-kind-home-is-scanned).

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

Each `[scan] exclude` entry is one directory **component name**, compared against each directory's own name at every depth; it is never a path and never a glob. An entry containing `/` could never match a directory, so it is an invalid value ([§FS-config.requirements.5](FS-config.md#requirements5-a-mistake-in-the-config-fails-loudly)): the config fails to load with the single located error of [§FS-config.4.3](FS-config.md#43-invalid-config-behavior), and no document is read. The **first** entry in list order that contains `/` is the one reported, located at the physical line of the `exclude` key, and the error names both repairs:

```text
error: grund.toml:4: [scan] exclude entry `docs/plans` contains `/`, not a directory name; remove it to keep the scan unchanged, or, if `plans` is a directory, use `plans` to exclude every directory of that name at any depth
```

Writing the last non-empty component (`plans`) excludes every directory of that name at every depth, not only `docs/plans`, and is offered as a condition because loading does not look at the tree; an entry with no last component that could name a directory, such as `./`, is offered the first repair alone. Only an entry containing `/` is refused: `.`, `..` and a file name such as `b.md` load and exclude nothing, since no directory's own name is `.` or `..` and a file is never compared against an entry. Excluding one path rather than a name everywhere is not a v1 capability: v2's source exclusions are globs ([§DISC-core-concerns.8.3](../discussions/proposals/2026-09-30-core-concerns.md#83-the-two-exclude-keys-mean-opposite-things)), and so are the entries of the separate `[fmt] exclude` ([§FS-config.3.10](FS-config.md#310-fmt--suppressing-the-rewrite)). Per [§DF-scan-exclude-component-names](../decisions/functional/DF-scan-exclude-component-names.md#df-scan-exclude-component-names-a-scan-exclude-entry-containing--is-a-config-error).

### 3.6 `[output]` — report format

```toml
[output]
format         = "text"   # text | json
color          = "auto"   # auto | always | never
relative_paths = true     # show paths relative to config root in reports
```

`format` and `relative_paths` are the v1 defaults of the run flags ([§FS-cli.3](FS-cli.md#3-cross-subcommand-flags)): `format` of `--format`, for the commands that honour it, and `relative_paths` of `--path-base`. A flag passed on the command line outranks the key for that run, and `config show` still prints the file's own values.

`relative_paths = true` (default) renders every `<path>` in a report relative to the config root ([§FS-config.1](FS-config.md#1-file-location-and-discovery)), and `relative_paths = false` relative to the CLI base ([§FS-config.3.6.1](FS-config.md#361-relative_paths--false--the-cli-base)). Either way `grund` **never** emits an absolute path, nor a path that escapes the loaded root other than the `..` path of a config above the run's root that the run had to read (`../grund.toml:16`, [§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)) or the `..` path of a file in an import ([§FS-imports.paths](FS-imports.md#paths-imported-files-are-reported-where-they-are)); this keeps the report deterministic ([§FS-errors.4](FS-errors.md#4-determinism)). `color` has no run-flag counterpart and changes no byte: output is plain whatever its value ([§FS-errors.3](FS-errors.md#3-message-text)).

#### 3.6.1 `relative_paths = false` — the CLI base

The CLI base is the path argument passed on the command line, or the working directory when none is given, and it is exactly what `--path-base=invocation` means ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)), so the key and the flag produce the same bytes and the flag reaches them without committing the key. A target elsewhere inside the loaded project or workspace takes the fewest `..` components that reach it from that base, and parent components are allowed only while the resolved target stays inside the loaded root ([§DF-cli-base-parent-paths](../decisions/functional/DF-cli-base-parent-paths.md#df-cli-base-parent-paths-relative_paths--false-keeps-one-cli-base-and-may-climb-within-the-loaded-root)). Only the part of the path argument above the loaded root resolves through links, and the components from the root down keep their spelling, so a link inside the project is never followed in choosing the base, and the report scope the path bounds ([§FS-check.1.3.6](FS-check.md#136-an-explicit-path-still-narrows)) is the same whichever ancestor link the path was spelled through.

### 3.7 `[fmt.cross_refs]` — cross-reference emission

```toml
[fmt.cross_refs]
enabled       = true       # default; false opts out of generated Markdown links
anchor_format = "github"   # default; one of github | gitlab | mkdocs | pandoc | none
```

What `enabled` does, the named `anchor_format` profiles, and when the cross-reference pass runs are [§FS-fmt.6.7](FS-fmt.md#67-configurability), per [§DF-md-link-default-on](../decisions/functional/DF-md-link-default-on.md#df-md-link-default-on-markdown-cross-reference-links-default-on-for-github-review-and-discovery) and [§DF-md-link-anchor-strategy](../decisions/functional/DF-md-link-anchor-strategy.md#df-md-link-anchor-strategy-heading-text-slugs-re-derived-on-every-fmt-pass). The generated `grund.toml` ([§FS-init.2.4](FS-init.md#24-generated-grundtoml)) writes every key of this table explicitly, `enabled = true` included, so the default file teaches that `grund fmt --write` emits Markdown inline links in files read as Markdown ([§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule)).

### 3.8 `[workspace]` — sub-project members

```toml
[workspace]
members          = ["apps/api", "packages/*"]
optional_members = ["vendored"]
include_root     = true
```

`members`, `optional_members` and `include_root` are specified by [§FS-workspace](FS-workspace.md#fs-workspace-grund-validates-cross-project-citations-in-a-workspace). The table is optional, and without it the repository is a single project.

### 3.9 `[citations]` — citation direction rules

```toml
[citations]               # absent section = no direction checks
default = "may"           # global default level for unlisted (citing → cited) pairs

[citations.FS]
should = ["GOAL|FS"]      # an FS declaration should cite a GOAL or a parent FS
must-not = ["AR"]         # an FS citation site may never cite an AR

[citations.E2E]
must = ["FS"]             # every E2E case must cite the FS it tests

[citations.code]          # the homeless kind (§3.9.2), named `code` by default
should = ["FS|AR"]
```

Each `[citations.<kind>]` subsection names the **citing** kind, by the `kind` name of [§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds); its arrays name the **cited** kinds. The citing side may be any configured kind — citable or not ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)), but never an unscanned one ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned)) — or the homeless kind ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)); the cited side must be a **citable** kind, because a kind with no IDs has nothing a citation could point at. The section is optional, and without it no direction check runs. Decided in [§DF-citation-directions](../decisions/functional/DF-citation-directions.md#df-citation-directions-encode-citation-directions-as-checked-config-with-rfc-2119-levels); its user-facing explanation is [Citation directions](../user-facing/citation-directions.md).

#### 3.9.1 Levels

Five keys form an RFC-2119 ladder, split into two rule classes and two enforcement surfaces:

| Level | Rule class | Checked per | Surface |
|---|---|---|---|
| `must` | obligation | declaration | `grund check` error — `missing-citation` ([§FS-check.3.11](FS-check.md#311-missing-required-citation)) |
| `should` | obligation | declaration | suggestion — `suggested-citation` ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)) |
| `may` | permission | — | never checked; punches a hole in a stricter `default` |
| `should-not` | prohibition | citation site | suggestion — `discouraged-citation` ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)) |
| `must-not` | prohibition | citation site | `grund check` error — `forbidden-citation` ([§FS-check.3.12](FS-check.md#312-forbidden-citation)) |

An **obligation** asks whether each top-level declaration of the citing kind contains at least one citation to the target kind, anywhere in its body. Multiple array entries are **conjunctive** — `must = ["GOAL", "GRUND"]` requires a citation to each — while a `|` disjunction inside one entry is satisfied by any one alternative. A **prohibition** fires once per offending citation site, located at its exact `file:line`.

`E2E` obligations are per case declaration ([§FS-check.3.11](FS-check.md#311-missing-required-citation)): they evaluate the case's scanned files plus the case manifest's `spec.refs` entries, and an otherwise empty evidence set fails a `must` entry rather than satisfying it vacuously. A `spec.refs` entry is kind-shaped evidence: it may name an idealized ID that does not resolve locally, so the dangling check that governs ordinary citations does not apply to it.

The table's level→surface mapping is fixed, never a project knob, so two installs reading one config agree on what gates and what is suggested ([§FS-non-goals.9](FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization), [§DF-citation-directions.2.2](../decisions/functional/DF-citation-directions.md#22-errors-gate-suggestions-report--a-fixed-split-not-a-knob)).

The per-file obligation of a non-citable kind and of the homeless kind judges only a file that carries a citation ([§FS-check.3.11](FS-check.md#311-missing-required-citation)); whether a file cites at all is `require_grounding`'s question ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)). The descriptions of `must` and `should` that `grund config explain` prints ([§FS-config-explain.3](FS-config-explain.md#3-a-keys-answer)) each end with the sentence ``A file of a place or the complement that cites nothing is not checked by this key; `require_grounding` makes it a finding.`` Version 2 states the same for its keys ([§FS-config-v2.rules.citations.uncited](FS-config-v2.md#rulescitationsuncited-an-obligation-never-judges-a-file-that-cites-nothing)).

#### 3.9.2 The homeless kind

Every citation site that no single configured kind home claims — outside every home, or in a file two overlapping homes contain — resolves to one citing kind ([§FS-citations.citing-kind](FS-citations.md#citing-kind-which-kind-a-citation-site-counts-as)). That kind is the **complement** of the whole `[[kinds]]` table, the one kind that is not a place: it has no `folder` and no `file`, and there is exactly one of it. Its name is `code` by default, and a project names it otherwise by declaring it:

```toml
[[kinds]]
kind = "src"                              # or `modules`, `implementation`, `code`…
citable = false                           # required: the complement declares no IDs
title = "Terraform modules and shell"     # optional: what it covers, for the generated block

[citations.src]
should = ["FS|AR"]
```

An entry is the homeless kind exactly when it sets `citable = false` and neither `folder` nor `file`; that shape is the declaration, not a separate key. Declaring two is a config error ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)). Naming the kind moves the rules with it: `[citations.src]` governs those sites, and `[citations.code]` in that config names an unknown kind ([§FS-config.3.9.5](FS-config.md#395-validation)). `code` stays reserved to this kind ([§FS-config.3.4.5](FS-config.md#345-name-rules)), so declaring `code` with a `title` is how a project keeps the name and says what it covers.

Its obligations are asked per **source file** ([§FS-check.3.11](FS-check.md#311-missing-required-citation)), under the exact predicate `require_grounding` uses: a scanned file not read as Markdown ([§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule), [§DF-require-grounding.2.2](../decisions/functional/DF-require-grounding.md#22-grounded-is-defined-syntactically)). Markdown outside a kind home (a README, the changelog) is therefore prohibition-checked but obligation-exempt. Its row takes the grounding keys of [§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level). A file here is a spelling, as in every v1 home rule: a physical file reached through a home and from outside it answers at each path by what that path is in ([§FS-scan.identity.findings.place](FS-scan.md#identityfindingsplace-a-place-finding-is-judged-at-each-spelling)).

The homeless kind gets no Project map row ([§FS-init.2.3.4](FS-init.md#234-managed-block-content-sections)), and its citation-directions row renders last, wherever in the table it was declared, under the subject [§FS-init.2.3.5](FS-init.md#235-citation-directions) gives it.

#### 3.9.3 Alias matching

Rule entries reuse the citation grammar of [§FS-workspace.1](FS-workspace.md#1-citation-syntax): a bare `AR` matches the **local** project only; `alias/AR` pins one workspace member, spelled with the same whole alias path a citation uses (`group/api/AR`, [§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)); `*/AR` matches the kind in **any** project, the local one included. `*/` is valid in rule entries only and is never a citation. Each entry parses as `[alias-path-or-*/]KIND`, split at the last `/` as a citation is. A malformed qualifier is rejected with a citation-target finding that names the target's kind and the first invalid qualifier segment; an empty qualifier or segment is named explicitly. A `*` segment is invalid unless it is the whole qualifier, and the finding says so.

The match is textual on the qualifier and kind; which kinds a qualifier's project has is validation's question, not matching's ([§FS-config.3.9.5](FS-config.md#395-validation), [§FS-scan.projects.kinds](FS-scan.md#projectskinds-whose-kinds-a-run-reads)). Resolution failures are separate errors ([§FS-check.3.8](FS-check.md#38-cross-project-citation-failure)), so the direction check never loads a foreign config. Each member's own `[citations]` governs the citation sites in that member's tree and inherits nothing from the workspace root, so a rule at the root says nothing about a member's sites ([§FS-config.principle.boundary](FS-config.md#principleboundary-the-project-is-the-widest-committed-scope)). A member that must cite no other project therefore says so in its own config: it closes its `default` to `must-not` ([§FS-config.3.9.4](FS-config.md#394-defaults-and-precedence)) and lists each citing kind's own targets bare under `may`, each non-citable kind and the homeless kind included, so a citation of any other project falls to that default. Decided in [§DF-citation-directions.2.5](../decisions/functional/DF-citation-directions.md#25-namespace-matching-mirrors-citation-grammar).

#### 3.9.4 Defaults and precedence

`default` (top-level, or per-kind inside a `[citations.<KIND>]` table) sets the level for unlisted target kinds; the global default is `may`, so adoption is incremental. Precedence is **explicit target list > per-kind `default` > global `default`**. Obligations come only from explicit `must` / `should` entries — a `default` of `must` or `should` never invents an obligation toward every unlisted kind; `default` governs only how a citation to an otherwise-unlisted target is leveled for the prohibition pass.

##### 3.9.4.1 A row closed to every citation

A citing row is **closed to every citation** when it permits no target: its effective fallback is `must-not`, its own `default` or else the global `default`, and every non-empty list it has is `must-not`. Under the precedence above each citation its units make is then leveled `must-not`, listed or not, so each one is a forbidden citation ([§FS-check.3.12](FS-check.md#312-forbidden-citation)) and a unit that cites nothing already satisfies the row. The test matches no targets, because two targets that could match one citation at different levels are refused ([§FS-config.3.9.5](FS-config.md#395-validation)), and it reads the lowered rules, so a version-1 `[citations.<kind>]` table and a version-2 `[rules.citations.<row>]` table give one answer ([§FS-config-v2.rules.citations](FS-config-v2.md#rulescitations-rulescitations)). A row with any `must`, `warn`, `should`, `may`, `should-not` or `warn-not` entry is not closed, and neither is one whose effective fallback is any other level, `warn-not` and `should-not` included. The predicate moves no verdict, since `grund check` levels each site as before; the Citation directions bullet reads it ([§FS-init.2.3.5](FS-init.md#235-citation-directions)).

#### 3.9.5 Validation

Config validation rejects:

- a `[citations.<kind>]` table whose kind is neither a configured `[[kinds]]` name nor the homeless kind's name ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)), so `[citations.code]` is rejected in a config that named its complement something else;
- one naming an unscanned kind ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned));
- a target naming a kind that is not a configured *citable* kind of the project its qualifier names, reported as *an unknown target kind* for a name the table does not hold and *a non-citable target kind* for one it does, since those are different mistakes;
- `code` used as a `[[kinds]]` name by anything but the homeless kind ([§FS-config.3.4.5](FS-config.md#345-name-rules));
- two entries declaring the homeless kind ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind));
- an unknown level key;
- two targets of the same cited kind at different levels whose alias matchers can match the same citation. The rule is on **overlap**, not textual equality: `*/AR` (any project) overlaps a bare `AR` (local), so listing one at `should` and the other at `must-not` is rejected, while a local `AR` permitted beside a pinned `alias/AR` forbidden is allowed, because those matchers are disjoint ([§FS-config.3.9.3](FS-config.md#393-alias-matching)).

A target is judged against the kinds of the projects its qualifier names, read as [§FS-scan.projects.kinds](FS-scan.md#projectskinds-whose-kinds-a-run-reads) says: a bare `KIND` against the local project's, `alias/KIND` against the aliased project's, and `*/KIND` against every loaded project's. So `[citations.FS] must = ["api/API"]` at a workspace root whose member `api` configures `API` loads and is enforced exactly as the rule `Each FS must cite at least one api/API.` is ([§FS-check.3.11](FS-check.md#311-missing-required-citation)).

The citing side, `[citations.<kind>]`, and every bare target are judged at load. A pinned or `*/` target is judged when the workspace is expanded, the first point that holds every project's kinds, so `grund config validate` judges it against the workspace its file declares and gives the load's answer. A narrowed `cover`, which reads that workspace's kinds without loading its files ([§FS-scan.projects.kinds](FS-scan.md#projectskinds-whose-kinds-a-run-reads)), judges none of them where the workspace does not expand — a member whose config fails to load, say — because the kinds it can read are then not every project's.

A pinned or `*/` target the run cannot accept fails the load ([§DF-one-grammar-lands-in-0-19](../decisions/functional/DF-one-grammar-lands-in-0-19.md#df-one-grammar-lands-in-0-19-the-one-grammar-window-closes-in-0190-and-a-pinned-target-the-run-cannot-accept-fails-the-load)): the run prints the first one's line as a CLI-level `error:` ([§FS-errors.2.2](FS-errors.md#22-cli-level-message)), writes nothing to stdout, and exits `2`. The first is taken in the order the run judges them: its projects in the order it loads them, and within one config the citing tables in byte order of their kind, each table's levels as `must`, `warn`, `should`, `may`, `should-not`, `warn-not`, `must-not`, and each level's targets as written.

At a workspace root whose member `api` configures no `FS`, `[citations.FS] must-not = ["api/FS"]` fails the load with:

```text
error: grund.toml: [citations.FS] must-not names an unknown target kind `FS` in namespace `api`; this became an error in grund 0.19.0
```

The level key is the one the target is listed under, and the path is the config's as every config message names it. A run that holds no workspace — a member's own run, or a project with no `[workspace]` — cannot judge a target pinned to another project, and a `*/KIND` target it cannot accept locally, at all: such a target fails its load the same way, in the wording of [§FS-errors.3.7](FS-errors.md#37-the-rule-site-unknown-alias-reasons) ([§DF-one-kind-verdict.decision.routes](../decisions/functional/DF-one-kind-verdict.md#decisionroutes-which-route-each-move-takes)). A target the run accepts prints nothing, and a bare target keeps its own load error.

### 3.10 `[fmt]` — suppressing the rewrite

```toml
[fmt]
exclude = ["docs/architecture/AR-topology.md", "docs/diagrams"]
```

`exclude` is the set of files `grund fmt` performs **no** rewrite in; a suppressed file is still scanned and still checked, and the whole contract is [§FS-fmt.2.5.1](FS-fmt.md#251-fmt-exclude--a-file-at-a-time).

Each entry is a gitignore-style glob resolved against the config root ([§FS-config.1](FS-config.md#1-file-location-and-discovery)), the dialect `respect_gitignore` brings to the scan ([§FS-config.3.5.15](FS-config.md#3515-respect_gitignore--the-ignore-files)), so `docs/diagrams` takes every file under that directory. A pattern the glob parser rejects is a config error at its own line ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)). The list is empty by default, and `grund config show` prints the table only where it is non-empty. `[fmt]` holds the settings of the `fmt` command as a whole, over every rewrite it performs, and `[fmt.cross_refs]` those of the cross-reference pass ([§FS-config.3.7](FS-config.md#37-fmtcross_refs--cross-reference-emission)); the per-region counterpart of this key is written in the file it governs ([§FS-fmt.2.5.2](FS-fmt.md#252-grundfmt-off--grundfmt-on--a-region-at-a-time)). Per [§DF-fmt-suppression](../decisions/functional/DF-fmt-suppression.md#df-fmt-suppression-fmt-suppression-is-per-file-and-per-region-and-the-index-carve-out-outranks-both).

### 3.11 A list element ends at a comma outside a string

A string list, the `["…", "…"]` array of basic strings that [§FS-config.3](FS-config.md#3-keys-version-1)'s surface reads, is cut into its elements only at a comma outside a string, and an element is read as written: `files = ["specs/a,b/spec.md"]` names the file `specs/a,b/spec.md`. A string closes at the first `"` that is not part of a `\"` escape, so `["a\",b"]` is the one element `a",b`. A comma belongs to an element only when that element is one basic string, an opening `"` and its closing one with nothing outside them; text between two separating commas that is not one basic string is cut at each of its commas. Every list key of both versions reads its elements this way, a list written across lines too once it is joined ([§FS-config-v2.reader.5](FS-config-v2.md#reader5-an-array-may-span-lines)), and the two members of the closed `lead_size_warning` inline table ([§FS-config.3.1.2](FS-config.md#312-lead_size_warning--the-oversized-lead-opt-in)) are separated by the same rule, so `unit = "lines,words"` is refused as an unknown unit that names `lines,words`.

A malformed element is refused at its key's line, in the shape of [§FS-config.4.3](FS-config.md#43-invalid-config-behavior), with the refusal of that element itself ([§FS-config-v2.reader.3](FS-config-v2.md#reader3-one-setting-one-form)): a comma missing between two strings is `expected string` in both versions, and in version 1 so is a dropped quote, a trailing comma or an unclosed string. Version 2 reads a trailing comma, a string that closes early and an array that does not close by its own rules ([§FS-config-v2.reader.5.commas](FS-config-v2.md#reader5commas-an-array-takes-one-trailing-comma-after-an-element), [§FS-config-v2.reader.6](FS-config-v2.md#reader6-a-string-ends-at-its-first-unescaped-quote), [§FS-config-v2.reader.5](FS-config-v2.md#reader5-an-array-may-span-lines)). A key whose own grammar cannot take a comma refuses it at that key, naming the value, as `[scan] exclude` refuses a `/` ([§FS-config.3.5.16](FS-config.md#3516-an-exclude-entry-containing--is-refused)), never as a string that is not one.

## 4. Validation and inspection

Beside the readers, the repository publishes `schemas/grund.schema.json`, a JSON Schema (draft-07) of every key of both versions ([§FS-config.3](FS-config.md#3-keys-version-1), [§FS-config-v2](FS-config-v2.md#fs-config-v2-grund-reads-a-version-2-config-by-concern-with-one-strength-vocabulary-and-fixed-defaults)) with its type, accepted values and default, each table closed to keys it does not name, for editors whose TOML language server reads JSON Schema. It agrees with the readers, every key it names accepted and every committed configuration `config validate` accepts valid against it, but it is not a second validator: what relates keys to one another or to the tree, such as a `resolve` that needs a `fetch`, stays with `grund config validate` ([§FS-config.4.1](FS-config.md#41-grund-config-validate-path)). `grund config migrate --to 2 [path]` shows the version-2 file that means what a version-1 config means, and with `--write` is the one command that may replace an authored config ([§REQ-no-data-loss.4](../requirements/REQ-no-data-loss.md#4-one-migration-may-replace-the-config), [§FS-config-migrate](FS-config-migrate.md#fs-config-migrate-grund-config-migrate---to-2-shows-the-version-2-file-that-means-what-a-version-1-config-means-and-writes-it-only-when-asked-and-only-when-it-does)). `grund config explain [target] [path]` says, from the installed binary and in the version the discovered file is read as, what any key accepts and means and what any check code means and what turns it on ([§FS-config-explain](FS-config-explain.md#fs-config-explain-grund-config-explain-answers-from-the-installed-binary-and-in-the-version-the-projects-config-is-read-as-what-any-key-accepts-and-means-and-what-any-check-code-means-and-what-turns-it-on)).

### 4.1 `grund config validate [path]`

Loads the config discovered by walking up from `path` (or `.` when omitted), checks its keys, and reports problems without scanning the tree: exit `0` on success, `1` on a validation error, in the `error: <path>:<line>: <message>` shape of [§FS-config.4.3](FS-config.md#43-invalid-config-behavior). The config findings — the redundant pair ([§FS-check.4.3](FS-check.md#43-redundant-config-pair)), the deprecated location ([§FS-check.4.11](FS-check.md#411-config-read-from-the-deprecated-agents-location)) and the version 1 caution ([§FS-check.checks.config-version-1](FS-check.md#checksconfig-version-1-a-config-file-read-as-version-1)) — are reported in the bytes `check` prints for them, one line on stderr per finding, with stdout empty and the exit code unchanged.

When the discovered config declares `[workspace]` ([§FS-config.3.8](FS-config.md#38-workspace--sub-project-members), [§FS-workspace.2](FS-workspace.md#2-workspace-configuration)), `config validate` also expands `members` and loads every member config the run would load, nested workspaces included ([§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)): the launch-time pass `grund check` runs before it scans anything. The first problem, a member config that does not load or a `members` entry that cannot be resolved, is reported once, in the line `grund check` prints for it, with paths rendered from the workspace root when `[output] relative_paths = true` ([§FS-workspace.5](FS-workspace.md#5-command-scope)), and exit `1`, with no warning line. A run that loads names the config findings of every config in the set `grund check` names at the same root ([§FS-check.4.11](FS-check.md#411-config-read-from-the-deprecated-agents-location)), each at the path its config was loaded under, spelled from the root's report base, in `check`'s report order ([§FS-errors.4](FS-errors.md#4-determinism)). A path inside a member validates that member alone and names its findings alone, as `grund check <member>` does ([§FS-workspace.5](FS-workspace.md#5-command-scope)).

### 4.2 `grund config show [path]`

Prints the **effective** configuration, the defaults merged with the config discovered by walking up from `path` (or `.` when omitted) and with CLI flags, as TOML that loads back to the same effective values; every rule below about which keys are printed keeps that so. A config finding of the printed config, such as a redundant pair at the config root ([§FS-config.1.1](FS-config.md#11-when-one-directory-carries-both), [§FS-check.4.3](FS-check.md#43-redundant-config-pair)), is a `warning:` on stderr before the TOML.

Selection stops at a workspace member boundary, `path` inside a member included: a member with neither config form shows [§FS-workspace.2](FS-workspace.md#2-workspace-configuration)'s config root and the canonical defaults, never the workspace root's file, and the effective-config API selects the same root, file or its absence, and version as the project derived from it. At a workspace root `show` inspects the root alone: it loads no sibling member config and names no member config's finding, which `grund config validate` does ([§FS-config.4.1](FS-config.md#41-grund-config-validate-path)).

`show` prints the settings in effect, defaults included, in the file's own version ([§DF-v2-onboarding](../decisions/functional/DF-v2-onboarding.md#df-v2-onboarding-grund-init-writes-a-minimal-version-2-file-and-grund-config-show-does-the-teaching)); what a key means and accepts is `grund config explain`'s answer ([§FS-config-explain.3](FS-config-explain.md#3-a-keys-answer), [§DF-config-explain.2.2](../decisions/functional/DF-config-explain.md#22-it-narrows-config-show-is-the-teaching-surface)). A v2 config prints in v2 spelling. A v1 config prints in v1 spelling with v1's defaults, and its bytes are frozen ([§REQ-backwards-compatibility.1](../requirements/REQ-backwards-compatibility.md#1-what-is-covered)): a key the file omits prints at its v1 default ([§FS-config.3](FS-config.md#3-keys-version-1)), never at the v2 default for the same setting ([§FS-config-v2.defaults](FS-config-v2.md#defaults-the-v2-epoch)). Showing a version-1 file never migrates it ([§FS-config-migrate](FS-config-migrate.md#fs-config-migrate-grund-config-migrate---to-2-shows-the-version-2-file-that-means-what-a-version-1-config-means-and-writes-it-only-when-asked-and-only-when-it-does)).

What applies is not every setting. Every `[[kinds]]` row prints under the canonical `kind` key, `citable` only where it is `false`, and `require_grounding` and `grounding_level` only where the row's effective value differs from the effective global printed under `[reference]` ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)). `[id]` prints `named_sections = true` only when enabled, and a value kind prints `values = true` only when enabled ([§FS-config.3.4.9](FS-config.md#349-values--first-class-value-declarations)). An opt-in table that is off prints nothing, such as `[schema.leads.words]` while no lead is capped, or a `[rules.citations.<KIND>]` table the file does not write. `[reference]` always prints the effective `shorthand`, as `shorthand = "canonical"` where the key is absent ([§FS-config.3.1](FS-config.md#31-reference--citation-form)). No text written for a reader may therefore claim that `show` prints all of grund's settings: each place that sends a reader to `show` names `grund config explain` for what `show` leaves out, in this one sentence wherever the place's own subject allows:

> `grund config show` prints what applies, `grund config explain` every setting.

The places are the comment that opens the `grund.toml` `grund init` writes ([§FS-init.2.4.3](FS-init.md#243-every-written-key-differs-from-its-default)); the help pages of `grund init` and `grund config` ([§FS-cli.2.3](FS-cli.md#23-a-subcommands-help-page)), where `show`'s own entry says in its words that it leaves out an opt-in table that is off and that `grund config explain` answers for every setting; the configuration questions `grund guide config` prints ([§FS-init.5.5](FS-init.md#55-the-core-and-its-sections)); and the guides, examples and README that say what `show` prints or present it as where a reader finds a setting. `show`'s own output says none of this.

The output's order is fixed by the config alone ([§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)). A v1 config prints its tables in one fixed order, top-level keys first, with its `[[kinds]]` rows in the order the file writes them. A v2 config prints the envelope keys, then `[schema]`, `[rules]` and `[presentation]`, each followed by its tables. Every table keyed by a row name, `[schema.kinds.<NAME>]`, `[rules.citations.<NAME>]` and `[presentation.kinds.<NAME>]`, prints in the order the file writes its `[schema.kinds]` rows ([§FS-config-v2.reader.4](FS-config-v2.md#reader4-order-and-location-are-kept)), never alphabetically, and a citation table keyed by a name that is no row, such as `code` when the file writes no `code` row, prints after every row's, in the order the file writes it.

### 4.3 Invalid config behavior

A `grund.toml` that fails validation makes every `grund` subcommand exit with code `2` and a single error pointing at the first problem, in the form `error: <path>:<line>: <message>` on stderr ([§FS-errors.2.2](FS-errors.md#22-cli-level-message), [§FS-check.2.1.1](FS-check.md#211-cli-level-messages)), where `<path>:<line>:` points at the offending key or line; in a version-2 file the first problem is the one [§FS-config-v2.reader.4.order](FS-config-v2.md#reader4order-which-refusal-a-file-with-several-is-refused-for) picks. Three commands differ: `grund config validate` exits `1` ([§FS-config.4.1](FS-config.md#41-grund-config-validate-path)), the hidden `complete` helper prints nothing and exits `0` ([§FS-completions.2](FS-completions.md#2-internal-dynamic-helper)), and `grund config explain` answers, exiting `2` only for a version this grund does not read ([§FS-config-explain.2](FS-config-explain.md#2-which-version-answers)). A failing config includes an invalid `values` row ([§FS-config.3.4.9](FS-config.md#349-values--first-class-value-declarations)) and, for every command that loads the workspace, a `[workspace] members` line that swallows its block's own scan ([§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan)); source-content validity is the scan's and the checker's. Later problems are not reported until the first is fixed, so no cascade obscures the root cause. Concrete stderr examples are [§FS-output-shapes.6](FS-output-shapes.md#6-cli-and-config-failures).

In `check --watch`, an invalid config ends a run with those same error bytes and status `2`, while the process stays resident for repair ([§FS-check.6.3.1](FS-check.md#631-recoverable-runs)).

Under v2, a clause the format defines but this grund does not execute is refused at its authored line, and an omitted default this grund cannot execute is refused at the `grund_config_version` line ([§FS-config-v2.rollout](FS-config-v2.md#rollout-what-this-grund-executes-and-what-it-refuses)). Neither is accepted and ignored.

## 5. Schema versioning

The TOML file may include a top-level `grund_config_version = N`. This grund reads versions **1** and **2**, each by its own reader, and refuses to load a config whose version is greater than the binary's known maximum, with an error suggesting an upgrade: `unsupported config version `<N>` (this grund understands grund_config_version = 1 or 2; upgrade grund if the config is newer)`.

Which reader reads a project is decided by its own file. A file that starts with a byte-order mark is read by none: it is refused at line 1, before its version is read, as ``<path>:1: the file starts with a byte-order mark; save it as UTF-8 without one``. An explicit `2` selects the v2 reader ([§FS-config-v2](FS-config-v2.md#fs-config-v2-grund-reads-a-version-2-config-by-concern-with-one-strength-vocabulary-and-fixed-defaults)). An omitted version and an explicit `1` select v1, and so does a project with no config file at all, whether discovery finds none or a workspace member has none of its own; each of those gets the fixed v1 defaults ([§GOAL-zero-config](../goals.md#goal-zero-config-works-on-any-conformant-tree)), so only an explicit `2` opts into v2's defaults and its stricter validation. A workspace member is read by the reader its own file selects, so a v2 root may list v1 members, and the reverse.

### 5.1 New keys are not a new version

Adding an optional table or key does not bump the version, because a config that uses it is only ever written for a binary that understands it, and an older binary meeting it fails loudly and locatably through the unknown-section or unknown-key rejection ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)) rather than silently misreading it. The safety net for the forward direction is the closed section and key allow-list, not the version integer.

### 5.2 A version's meaning moves only as backwards compatibility allows

An upgrade reads a config of a known version as the release before it did, except where a verdict moves as [§REQ-backwards-compatibility.1](../requirements/REQ-backwards-compatibility.md#1-what-is-covered) allows. A default, a language row or a doc profile the version fixes never moves: a different one is a different version.

Version 1's reading is frozen: no release changes what a version 1 file means, in the reports or in the records the engine reads. A v1 state that v2 cannot spell, such as a positive citation default or `strict = false`, keeps its meaning in those records ([§FS-config-v2.mapping](FS-config-v2.md#mapping-what-the-records-keep-apart)). The v1 reader keeps v1's validation too: v2's stricter rules never refuse a file v1 accepts.

### 5.3 A version 1 config is cautioned and never retired

A config file read as version 1, whether it writes `grund_config_version = 1` or no version at all, is cautioned ([§FS-check.checks.config-version-1](FS-check.md#checksconfig-version-1-a-config-file-read-as-version-1)) and keeps its meaning ([§FS-config.5.2](FS-config.md#52-a-versions-meaning-moves-only-as-backwards-compatibility-allows)): the caution is the deprecation path's warning without a deadline ([§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)). It keys on a *file*: a project with no config file is read as v1 too, but wrote nothing to migrate, so it earns no caution ([§GOAL-zero-config](../goals.md#goal-zero-config-works-on-any-conformant-tree)).

## 6. What is NOT configured here

Per [§GOAL-friendliness-first.2](../goals.md#2-what-this-rules-out), the following are deliberately **not** configurable:

- The set of severities: only `error` and `warning` exist, in both versions ([§FS-config-v2.rules.strengths](FS-config-v2.md#rulesstrengths-one-strength-vocabulary-two-severities)). A suggestion is not a third one: the `should` and `should-not` citation-direction suggestions ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)) are carried on a separate non-severity advisory channel ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)).
- The exit code mapping (`0`/`1`/`2` per [§FS-cli.5](FS-cli.md#5-exit-code-mapping-is-fixed)).
- The ordering of the report (always deterministic).
- Anything that would let two installs disagree on the same input ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)).
- The local conversation citation *preference*: it follows the user's TUI setup and is installed through `grund integrations --write` ([§FS-integrations.4.3](FS-integrations.md#43-user-preference-and-global-agent-instructions)). What a repository may commit instead is [§FS-config.3.1.3](FS-config.md#313-conversation--one-name-two-scopes).

## v2: The version-2 format

Version 2 spells the same project by concern, with one strength vocabulary and fixed defaults. It is specified in its own declaration, [§FS-config-v2](FS-config-v2.md#fs-config-v2-grund-reads-a-version-2-config-by-concern-with-one-strength-vocabulary-and-fixed-defaults). A section here that says what a key means is about v1 unless it names v2.
