# FS-init-fixtures: concrete init fixtures

This file is the verbose fixture companion to [§FS-init](FS-init.md#fs-init-grund-bootstraps-a-new-grund-conformant-repo). It does not add a new command; it gives implementers concrete states and transcripts for the `init` behavior that [§FS-init](FS-init.md#fs-init-grund-bootstraps-a-new-grund-conformant-repo) defines in prose.

The fixtures use `{repo}` for an existing target directory and `{repo_copy}` for a mutable copy of that directory. Per [§FS-init.2.2](FS-init.md#22-stdout--stderr), every success case has empty stdout and every path `init` prints is relative to the target directory. The stderr blocks below are exact apart from the placeholder path in the missing-target case.

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, home), [§FS-terms.terms.2](FS-terms.md#terms2-citations) (citation), and
[§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, workspace, member, alias).

## 1. Default form

Command:

```text
grund init {repo_copy}
```

Precondition: `{repo_copy}` exists and contains no `AGENTS.md` and no config in either discovery form ([§FS-config.1](FS-config.md#1-file-location-and-discovery)).

Exit `0`, stdout empty, stderr:

```text
wrote AGENTS.md
wrote grund.toml

next:
  1. re-run with --scaffold to scaffold the FS home (docs/functional-spec), docs/, and tests/ (or create them yourself)
  2. run `grund check` — on a fresh scaffold it warns "nothing recognized" until the first declaration; this is expected (exit 0), no repair needed
  3. allocate an ID:  ID=$(grund id FS "…")  then add it under docs/functional-spec
see AGENTS.md for the full workflow.
```

Step 1 carries no ` — until then \`grund check\` has nothing to scan` suffix: the version-2 starter scans its config root, and `AGENTS.md` is there to read ([§FS-init.2.2.2](FS-init.md#222-the-next-block)).

Final tree:

```text
AGENTS.md
grund.toml
```

`AGENTS.md` contains exactly one delimiter-bounded managed block — `<!-- BEGIN GRUND MANAGED BLOCK -->` through `<!-- END GRUND MANAGED BLOCK -->` ([§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)) — headed `## Grounding with grund (block vN)` at the current block version. `grund.toml` is the `software-development` starter of [§FS-init.2.4.3](FS-init.md#243-every-written-key-differs-from-its-default), byte for byte: the comment line, `grund_config_version = 2`, `project_name`, the ten `[schema.kinds]` rows with their places, `REQ` after `FS` ([§FS-init.2.4.10](FS-init.md#2410-req-is-declared-in-docsrequirements)), and their ten `[presentation.kinds]` titles. Nothing else is written: no `[schema.sources]`, no `[schema] id_format`, no `[rules]`, and no commented-out key. A second `grund init {repo_copy}` reports `exists AGENTS.md` and `exists grund.toml`, prints no `next:` block, and leaves the tree byte-identical ([§FS-init.3.3](FS-init.md#33-without---force)).

### 1.1 Populated configured form

Command:

```text
grund init {repo_copy} --agents-md
```

Precondition: `{repo_copy}` has no `AGENTS.md`; its existing `grund.toml`
configures populated `[scan] include` paths and scanned kind homes, so the
effective scanner would read at least one file ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)).

Exit `0`, stdout empty, stderr:

```text
wrote AGENTS.md
exists grund.toml

next:
  1. re-run with --scaffold to scaffold the FS home (requirements.md), docs/, and tests/ (or create them yourself)
  2. run `grund check` — on a fresh scaffold it warns "nothing recognized" until the first declaration; this is expected (exit 0), no repair needed
  3. allocate an ID:  ID=$(grund id FS "…")  then add it to requirements.md
see AGENTS.md for the full workflow.
```

The counter-case is a version-1 config whose effective scanner would read no
file, such as one whose `[scan] include` names only paths that do not exist yet:
step 1 then retains the suffix ` — until then \`grund check\` has nothing to
scan` byte-for-byte. The default form above no longer is that counter-case,
because the version-2 starter scans its config root ([§FS-init.2.2.2](FS-init.md#222-the-next-block)). The populated form changes no other guidance
byte and does not suppress the scaffold advice.

### 1.2 Refresh of a scaffolded tree

Command:

```text
grund init {repo_copy}
```

Precondition: `{repo_copy}` has an `AGENTS.md` whose managed block a
`grund.toml` change made stale — the state `grund check` reports with "run
`grund init` to refresh" ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)) — and its effective FS home, `docs/`
and `tests/` exist: either the `docs/functional-spec` a `grund init --scaffold`
scaffold wrote, or a configured FS file such as `requirements.md`.

Exit `0`, stdout empty, stderr:

```text
updated AGENTS.md
exists grund.toml
```

No `next:` block follows: the run is a refresh of a complete setup
([§FS-init.2.2.2.1](FS-init.md#2221-a-refresh-of-a-complete-setup-teaches-nothing)). Under `--check` the same tree reports `would-update AGENTS.md`
and `exists grund.toml`, again with no `next:` block, and exits `1`
([§FS-init.4](FS-init.md#4-exit-codes)).

The counter-case is the same refresh in a tree whose effective FS home is
absent: it keeps the whole block, step 1 included, as the
`init-citation-directions-canonical` e2e case pins — `updated AGENTS.md` over a
configured `docs/functional-spec` that does not exist. So does
[§FS-init-fixtures.1.1](FS-init-fixtures.md#11-populated-configured-form), a first run rather than a refresh.

## 2. Scaffold form

Command:

```text
grund init {repo_copy} --scaffold
```

Precondition: `{repo_copy}` exists and contains no generated files.

Exit `0`, stdout empty, stderr:

```text
wrote AGENTS.md
wrote grund.toml
wrote docs/functional-spec/README.md
wrote docs/requirements/README.md
wrote docs/grund.md
wrote docs/goals.md
wrote docs/roadmap.md
wrote docs/changelog.md
wrote docs/architecture/README.md
wrote docs/decisions/architectural/README.md
wrote docs/decisions/functional/README.md
wrote tests/e2e/README.md
wrote tests/integration/.gitkeep

next:
  1. run `grund check` — on a fresh scaffold it warns "nothing recognized" until the first declaration; this is expected (exit 0), no repair needed
  2. allocate an ID:  ID=$(grund id FS "…")  then add it under docs/functional-spec
     (H1: `# <ID>: <one-line statement of the behavior>`)
  3. cite it as §<ID> from the docs and e2e tests that depend on it, then `grund check` again
see AGENTS.md for the full workflow.
```

`grund init {repo_copy} --docs` writes nothing: `--docs` is an unknown flag since 0.19.0, refused as [§FS-init-fixtures.4](FS-init-fixtures.md#4-target-and-flag-failures) says ([§FS-init.1.7](FS-init.md#17---docs-is-the-deprecated-spelling-of---scaffold)).

Final tree includes the default-form files plus exactly these docs and tests scaffold paths:

```text
docs/functional-spec/README.md
docs/requirements/README.md
docs/grund.md
docs/goals.md
docs/roadmap.md
docs/changelog.md
docs/architecture/README.md
docs/decisions/architectural/README.md
docs/decisions/functional/README.md
tests/e2e/README.md
tests/integration/.gitkeep
```

The scaffold's `AGENTS.md` project map lists `REQ` right after `FS`, and its vocabulary line lists it in the same place ([§FS-init.2.4.10](FS-init.md#2410-req-is-declared-in-docsrequirements)):

```text
- [FS](docs/functional-spec): What: behavior, requirements, and constraints
- [REQ](docs/requirements): Hard requirements: what the project must never break
- [AR](docs/architecture): How: high-level implementation, structure, and design
```

```text
(`KIND ∈ {GRUND, GOAL, FS, REQ, AR, DF, DA, RM}`)
```

Every scaffold-owned ID example takes the starter's slug-only format, so `docs/goals.md` teaches `# GOAL-<slug>: …` and `docs/functional-spec/README.md` teaches the `FS-<slug>` ID ([§FS-init.2.1.3](FS-init.md#213-the-software-development-starter-files)). No `requirements.md` is written: the template homes `FS` in `docs/functional-spec/` ([§FS-init.2.4.10](FS-init.md#2410-req-is-declared-in-docsrequirements)).

`grund check` and `grund init --check` both exit `0` on the scaffolded tree, unmodified, and `grund.toml` is the version-2 starter of [§FS-init-fixtures.1](FS-init-fixtures.md#1-default-form).

## 3. Existing files

When the managed block and config are already current, `init` is a no-op on disk:

```text
exists AGENTS.md
exists grund.toml
```

Every reported path is `exists `, so the `next:` block is suppressed ([§FS-init.2.2](FS-init.md#22-stdout--stderr)).

When `AGENTS.md` exists without a managed block and `--force` is not passed, `init` appends the block and reports:

```text
appended AGENTS.md
wrote grund.toml
```

When `AGENTS.md` exists and `--force` is passed, `init` rewrites the canonical file and reports `wrote AGENTS.md`. An existing config, in either discovery form ([§FS-config.1](FS-config.md#1-file-location-and-discovery)), is never clobbered, not even by `--force`: it is reported `exists ` under the name it was found at, such as `exists grund.toml` ([§FS-init.2.4](FS-init.md#24-generated-grundtoml)).

## 4. Target and flag failures

A missing target directory is an I/O failure ([§FS-init.4](FS-init.md#4-exit-codes)): exit `2`, stdout empty, stderr:

```text
error: target directory does not exist: <path>
```

An unknown flag is a CLI-level failure: exit `2`, stdout empty, stderr:

```text
error: unknown flag `<flag>`
```

These failures leave the target tree unchanged.

## 5. Dry-run preview

`grund init --dry-run` reports exactly what a real run would do, without writing anything: `wrote `, `appended `, and `updated ` become `would-write `, `would-append `, and `would-update `, `exists ` lines are unchanged, and the `next:` block follows a real run's rule ([§FS-init.2.2](FS-init.md#22-stdout--stderr)). Exit code is `0` for a clean preview, `2` for an I/O error (e.g. missing target) or a CLI-level error, the same as a real run.

## 6. Workspace members

[§FS-init-fixtures.6.1](FS-init-fixtures.md#61-workspace-root-init) to [§FS-init-fixtures.6.4](FS-init-fixtures.md#64-workspace-member-descriptions) together cover [§FS-init.2.3.4.15](FS-init.md#23415-workspace-members): a root-side and a member-side run over one workspace, a non-workspace repo, and projects that carry `project_description`.

### 6.1 Workspace root init

Precondition: `{repo_copy}` exists with `grund.toml`:

```toml
project_name = "root"

[workspace]
members = ["apps/api", "packages/*"]
```

and the on-disk layout:

```text
grund.toml
apps/api/AGENTS.md
packages/core/
packages/ui/
```

`packages/core/` and `packages/ui/` are real directories with no `AGENTS.md`. `apps/api/AGENTS.md` already exists from a prior member-side `grund init`.

Command: `grund init {repo_copy}`.

The generated `{repo_copy}/AGENTS.md` contains, between `### Project map` and `### Declarations and citations`, exactly this block — byte-identical, including blank lines:

```markdown
### Workspace members

Cross-project citations use §alias/<ID>, one alias segment per workspace level.

- [`api`](apps/api/)
- [`core`](packages/core/)
- [`ui`](packages/ui/)
```

The list is sorted lexicographically by alias. Every bullet links its project's root directory ([§FS-init.2.3.4.15.6](FS-init.md#234156-the-link-target)): `api` has an `AGENTS.md` and `core` and `ui`, expanded from the glob `packages/*`, have none, and their bullets take the same form, with no suffix. The root's own row is absent even though `include_root` defaults to `true`: [§FS-init.2.3.4.15](FS-init.md#23415-workspace-members) omits the project whose entrypoint is being rendered.

### 6.2 Workspace member init

Same precondition as [§FS-init-fixtures.6.1](FS-init-fixtures.md#61-workspace-root-init), but with `apps/api/AGENTS.md` removed so the member is uninitialized. The fixture pins that the bytes do not depend on it: the section below is the same byte for byte whether `apps/api/AGENTS.md` exists or not, and whether `{repo_copy}/AGENTS.md` does.

Command: `grund init {repo_copy}/apps/api`.

The generated `{repo_copy}/apps/api/AGENTS.md` contains a `### Workspace members` section for the same logical workspace as [§FS-init-fixtures.6.1](FS-init-fixtures.md#61-workspace-root-init), with `api` itself omitted and the remaining paths rewritten relative to the AGENTS.md being written:

```markdown
### Workspace members

Cross-project citations use §alias/<ID>, one alias segment per workspace level.

- [`core`](../../packages/core/)
- [`root`](../../)
- [`ui`](../../packages/ui/)
```

`root` remains a foreign row because `include_root` is true, and its link is the workspace root directory, as every foreign link is a directory ([§FS-init.2.3.4.15.6](FS-init.md#234156-the-link-target)); that `{repo_copy}/AGENTS.md` does not exist changes nothing in it. The discoverability line and foreign-row grammar match [§FS-init-fixtures.6.1](FS-init-fixtures.md#61-workspace-root-init) exactly; each perspective omits its own canonical project and sorts the remaining aliases.

### 6.3 Non-workspace repo

Precondition: `{repo_copy}` exists and contains no `[workspace]` block in its config (or no config at all, in which case `init` writes the starter of [§FS-init.2.4](FS-init.md#24-generated-grundtoml)), and no ancestor workspace claims it ([§FS-init.2.3.4.15.1](FS-init.md#234151-which-workspace)).

Command: `grund init {repo_copy}`.

The generated `AGENTS.md` contains neither `### Workspace members` nor `### Project namespaces` ([§FS-init.2.3.4.16](FS-init.md#23416-project-boundaries)). The `### Project map` block is byte-identical to the default-form fixture ([§FS-init-fixtures.1](FS-init-fixtures.md#1-default-form)). Re-running `grund init` followed by `grund init --check` succeeds without restoring either section; configured markers and selected companion entrypoints follow the same omission.

### 6.4 Workspace member descriptions

Same shape as [§FS-init-fixtures.6.1](FS-init-fixtures.md#61-workspace-root-init), but the root and two members carry `project_description` metadata ([§FS-config.3](FS-config.md#3-keys-version-1), [§FS-workspace.3](FS-workspace.md#3-aliases), [§DF-workspace-member-descriptions](../decisions/functional/DF-workspace-member-descriptions.md#df-workspace-member-descriptions-member-side-project_description-for-workspace-member-lists)). Precondition: `{repo_copy}` exists with `grund.toml`:

```toml
project_name = "root"
project_description = "Workspace root: shared specs and tooling"

[workspace]
members = ["apps/api", "packages/*"]
```

`apps/api/grund.toml` sets `project_name = "api"` and `project_description = "Payment API service"`, and `apps/api/AGENTS.md` exists from a prior member-side `grund init`. `packages/core/grund.toml` sets `project_name = "core"` and `project_description = "Core domain library"` but `packages/core/AGENTS.md` does not exist. `packages/ui/` is a real directory with no config and no `AGENTS.md`.

Command: `grund init {repo_copy}`.

The generated `{repo_copy}/AGENTS.md` contains exactly this block:

```markdown
### Workspace members

Cross-project citations use §alias/<ID>, one alias segment per workspace level.

- [`api`](apps/api/): Payment API service
- [`core`](packages/core/): Core domain library
- [`ui`](packages/ui/)
```

Each described foreign project appends `: <description>` after its link, and the bullet ends there; `api`, which has an `AGENTS.md`, and `core`, which has none, take the same form; `ui` has no config, therefore no description, and its bullet is byte-identical to the [§FS-init-fixtures.6.1](FS-init-fixtures.md#61-workspace-root-init) form. The root's configured description is absent with its omitted self row, but remains available when the root is foreign in a member entrypoint.

## 7. The `none` template

Command:

```text
grund init {repo_copy} --template=none
```

Precondition: `{repo_copy}` has no `AGENTS.md` and no config in either discovery form, and may already hold a `docs/` and a `tests/` of its own.

Exit `0`, stdout empty, stderr:

```text
wrote AGENTS.md
wrote grund.toml

next:
  1. declare a first kind in grund.toml, then run `grund init` again so AGENTS.md lists it
  2. run `grund check`
  3. allocate an ID:  ID=$(grund id <KIND> "…")  then add it to that kind's home
see AGENTS.md for the full workflow.
```

The final tree is the precondition's plus `AGENTS.md` and `grund.toml`, and nothing under `docs/` or `tests/` changes ([§FS-init.3.6](FS-init.md#36-bare-init-writes-the-config-and-the-entrypoints-and-nothing-in-a-home)). `grund.toml` is the version-2 envelope of [§FS-init.2.4.9](FS-init.md#249-the-none-templates-config), with no `[schema.kinds]` row, and the block's Project map is the paragraph of [§FS-init.2.3.4.4.2](FS-init.md#23442-no-kinds-yet). `grund check` over that tree exits `0` and `grund init --check` reports `exists AGENTS.md` and `exists grund.toml`.

### 7.1 Refused template runs

Each of these exits `2`, writes nothing, and prints one line on stderr ([§FS-init.1.6.3](FS-init.md#163-an-unknown-name), [§FS-init.1.6.4](FS-init.md#164-none-has-nothing-to-scaffold)):

```text
$ grund init {repo_copy} --template=docs
error: unknown template `docs` — the built-in templates are software-development and none; a template from a path is not supported yet
$ grund init {repo_copy} --template=none --scaffold
error: --scaffold has nothing to lay: the none template has no starter files
```

### 7.2 A template over an existing config

Command: `grund init {repo_copy} --template=none`, where `{repo_copy}` already has a `grund.toml`. The config is kept and the run notes it ([§FS-init.1.6.5](FS-init.md#165-a-config-that-is-already-there)):

```text
wrote AGENTS.md
exists grund.toml
note: grund.toml exists, so the none template's grund.toml is not written — init never overwrites a config
```

The `next:` block that follows is the one the kept config earns ([§FS-init.2.2.2](FS-init.md#222-the-next-block)).

## 8. A project without FS

The configured-kind form of [§FS-init.2.2.2.3](FS-init.md#2223-the-kind-the-guidance-names), over one fixture: a version-1 `grund.toml` that declares one kind, `REQ`, homed in `docs/req`, and scans `docs` and `README.md`:

```toml
[id]
format = "{kind}-{slug}"
named_sections = true

[[kinds]]
kind = "REQ"
folder = "docs/req"

[scan]
include = ["docs", "README.md"]
```

`docs/req/REQ-login.md` declares `REQ-login`, `docs/req/README.md` indexes it, and `README.md` cites it. Each command below runs on a copy of that tree under Git with no `AGENTS.md` yet, unless its case says otherwise. No block names `FS` or `docs/functional-spec`, because the project declares neither, and none offers `--scaffold`, which lays the FS starter and no `REQ` one.

### 8.1 The home exists

Command: `grund init {repo_copy}`. Exit `0`, stdout empty, stderr:

```text
wrote AGENTS.md
exists grund.toml

next:
  1. run `grund check`
  2. allocate an ID:  ID=$(grund id REQ "…")  then add it under docs/req
see AGENTS.md for the full workflow.
```

No step asks for a home, because `docs/req` exists. After `grund config migrate --to 2 --write`, `grund check` reports the block stale and exits `1`. The refresh it asks for, `grund init {repo_copy}`, exits `0` with exactly this on stderr:

```text
updated AGENTS.md
exists grund.toml
```

It is a refresh of a complete setup, since its selected home exists ([§FS-init.2.2.2.1](FS-init.md#2221-a-refresh-of-a-complete-setup-teaches-nothing)). The `grund check` after it exits `0`, and so does `grund id REQ "…"`. Under `--check` the same refresh prints `would-update AGENTS.md` and `exists grund.toml`, again with no block, and exits `1` ([§FS-init.4](FS-init.md#4-exit-codes)).

### 8.2 The home is missing

The same tree without `docs/req/`. Command: `grund init {repo_copy}`. Exit `0`, stdout empty, stderr:

```text
wrote AGENTS.md
exists grund.toml

next:
  1. create the REQ home (docs/req)
  2. run `grund check`
  3. allocate an ID:  ID=$(grund id REQ "…")  then add it under docs/req
see AGENTS.md for the full workflow.
```

A refresh of the same tree keeps the whole block below `updated AGENTS.md`, step 1 included. Step 1 carries no ` — until then \`grund check\` has nothing to scan` here because the scanner still reads `README.md`. Over a tree whose scanner would read no file it does, as the `init-force-keeps-existing-config` e2e case pins. A row homed in a file, such as `file = "requirements.md"`, prints `create the REQ home (requirements.md)` and `then add it to requirements.md`.

### 8.3 No home at all

The tree of [§FS-init-fixtures.8](FS-init-fixtures.md#8-a-project-without-fs) with the `folder` line removed, so `REQ` is citable and has no home. Command: `grund init {repo_copy}`. Exit `0`, stdout empty, stderr:

```text
wrote AGENTS.md
exists grund.toml

next:
  1. run `grund check`
  2. allocate an ID:  ID=$(grund id REQ "…")  then declare it in a source doc-comment
see AGENTS.md for the full workflow.
```

The block names no directory and invents none. A refresh of that tree prints no `next:` block, because there is no home to create.

### 8.4 Scaffold form

Command: `grund init {repo_copy} --scaffold`, on the tree of [§FS-init-fixtures.8](FS-init-fixtures.md#8-a-project-without-fs). Exit `0`, stdout empty, stderr:

```text
wrote AGENTS.md
exists grund.toml
wrote docs/functional-spec/README.md
exists docs/req/README.md
wrote docs/grund.md
wrote docs/goals.md
wrote docs/roadmap.md
wrote docs/changelog.md
wrote docs/architecture/README.md
wrote docs/decisions/architectural/README.md
wrote docs/decisions/functional/README.md
wrote tests/e2e/README.md
wrote tests/integration/.gitkeep

next:
  1. run `grund check` — on a fresh scaffold it warns "nothing recognized" until the first declaration; this is expected (exit 0), no repair needed
  2. allocate an ID:  ID=$(grund id REQ "…")  then add it under docs/req
     (H1: `# <ID>: <one-line statement of the behavior>`)
  3. cite it as §<ID> from the docs and e2e tests that depend on it, then `grund check` again
see AGENTS.md for the full workflow.
```

The writes are the ones this run made before [§FS-init.2.2.2.3](FS-init.md#2223-the-kind-the-guidance-names), the FS starter in the built-in `docs/functional-spec` included; which starter `--scaffold` lays in a project without FS is not settled here. Only the allocation step names a different kind. A row homed in a file prints the H2 line the FS form prints for one, `(H2: \`## <ID>: <one-line statement of the behavior>\`)`. A row with no home prints `then declare it in a source doc-comment`, followed by `(doc-comment: \`<ID>: <one-line statement of the behavior>\`)`.
