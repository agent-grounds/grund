# FS-init-fixtures: concrete init fixtures

The exact output of `grund init`'s forms, each over the tree it runs on. They are instances of the rules [§FS-init](FS-init.md#fs-init-grund-bootstraps-a-new-grund-conformant-repo) states and add none of their own.

Each command runs on `{repo_copy}`, a mutable copy of its fixture tree. Every `grund init` run here leaves stdout empty and exits `0` unless its section gives another status, and a successful run names each path relative to its target ([§FS-init.2.2](FS-init.md#22-stdout--stderr)). Every message here is exact: `<path>` and `<flag>` in [§FS-init-fixtures.4](FS-init-fixtures.md#4-target-and-flag-failures) stand for the argument given, and every other `<…>` is printed as written.

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, home), [§FS-terms.terms.2](FS-terms.md#terms2-citations) (citation), and
[§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, workspace, member, alias).

## 1. Default form

`grund init {repo_copy}`, over a target with no `AGENTS.md` and no config in either discovery form ([§FS-config.1](FS-config.md#1-file-location-and-discovery)), exits `0` with this on stderr:

```text
wrote AGENTS.md
wrote grund.toml

next:
  1. re-run with --scaffold to scaffold the FS home (docs/functional-spec), docs/, and tests/ (or create them yourself)
  2. run `grund check` — on a fresh scaffold it warns "nothing recognized" until the first declaration; this is expected (exit 0), no repair needed
  3. allocate an ID:  ID=$(grund id FS "…")  then add it under docs/functional-spec
see AGENTS.md for the full workflow.
```

It adds those two files and nothing else: an `AGENTS.md` holding one managed block at the current block version ([§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)), and the `software-development` starter of [§FS-init.2.4.3](FS-init.md#243-every-written-key-differs-from-its-default), byte for byte. A second run reports `exists AGENTS.md` and `exists grund.toml`, prints the same `next:` block, since the FS home it asks for is still missing, and changes no byte ([§FS-init.2.2.2](FS-init.md#222-the-next-block), [§FS-init.3.3](FS-init.md#33-without---force)).

Steps 1 and 3 name the FS home the config gives. Over a tree that already holds `requirements.md`, `docs/`, `tests/` and a version-1 config whose `FS` row is `file = "requirements.md"`, a first run reports `wrote AGENTS.md` and `exists grund.toml`, then the whole block with `the FS home (requirements.md)` in step 1 and `then add it to requirements.md` in step 3: a first run keeps it even where its home exists ([§FS-init.2.2.2](FS-init.md#222-the-next-block)). Where [§FS-init.2.2.2](FS-init.md#222-the-next-block) calls for it, step 1 ends with `` — until then `grund check` has nothing to scan ``.

A refresh of a complete setup ([§FS-init.2.2.2](FS-init.md#222-the-next-block)), such as a scaffolded tree whose block a `grund.toml` change made stale ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)), prints `updated AGENTS.md` and `exists grund.toml` and no `next:` block; under `--check` it prints `would-update AGENTS.md` and `exists grund.toml` and exits `1` ([§FS-init.4](FS-init.md#4-exit-codes)).

## 2. Scaffold form

`grund init {repo_copy} --scaffold`, over a target with no generated files, exits `0` with this on stderr:

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

It leaves exactly the paths it names. Its `grund.toml` is the starter of [§FS-init.2.4.3](FS-init.md#243-every-written-key-differs-from-its-default), whose `REQ` row comes right after `FS` in the managed block's Project map and `KIND ∈ {…}` vocabulary ([§FS-init.2.4.3](FS-init.md#243-every-written-key-differs-from-its-default)), and its starter files are those of [§FS-init.2.1.3](FS-init.md#213-the-software-development-starter-files). `grund check` and `grund init --check` both exit `0` on that tree unmodified ([§FS-init.requirements.clean](FS-init.md#requirementsclean-what-init-writes-check-accepts)).

## 3. Existing files

Where the managed block is already current and a config exists, `init` writes nothing and reports each as `exists`:

```text
exists AGENTS.md
exists grund.toml
```

Without `--scaffold`, over a tree with no FS home, the `next:` block follows them, its step 1 asking for that home ([§FS-init.2.2.2](FS-init.md#222-the-next-block)). Over an `AGENTS.md` with no managed block, and no config, `init` without `--force` appends the block and reports `appended AGENTS.md` and `wrote grund.toml`, then the `next:` block of [§FS-init-fixtures.1](FS-init-fixtures.md#1-default-form). With `--force`, an existing canonical `AGENTS.md` is rewritten and reported `wrote AGENTS.md` ([§FS-init.3.4](FS-init.md#34-with---force)), and an existing config is reported `exists grund.toml`, or `exists .agents/grund.toml`, as without it ([§FS-init.2.4](FS-init.md#24-generated-grundtoml)).

## 4. Target and flag failures

Each of these exits `2`, leaves the tree unchanged ([§FS-init.requirements.total](FS-init.md#requirementstotal-a-run-that-exits-2-leaves-the-tree-as-it-was)) and prints one line on stderr. A missing target directory is an I/O failure ([§FS-init.4](FS-init.md#4-exit-codes)):

```text
error: target directory does not exist: <path>
```

A target behind a directory the user cannot search is refused rather than missing ([§FS-init.1](FS-init.md#1-inputs)): with `.r` at mode `000`, `grund init .r/sub` prints

```text
error: .r/sub: permission denied; make the path readable
```

An unknown flag, `--docs` among them ([§FS-init.1](FS-init.md#1-inputs)), is a CLI-level failure:

```text
error: unknown flag `<flag>`
```

## 5. Dry-run preview

`grund init {repo_copy} --dry-run`, over the tree of [§FS-init-fixtures.1](FS-init-fixtures.md#1-default-form), prints that run's transcript with `would-write AGENTS.md` and `would-write grund.toml` in place of its `wrote` lines and the same `next:` block after them, writes nothing, and exits `0` ([§FS-init.1](FS-init.md#1-inputs), [§FS-init.4](FS-init.md#4-exit-codes)). A preview that fails, on an I/O error such as a missing target or on a CLI-level error, exits `2` as the real run does ([§FS-init.4](FS-init.md#4-exit-codes)).

## 6. Workspace members

One workspace shows the `### Workspace members` section of [§FS-init.2.3.4](FS-init.md#234-managed-block-content-sections). Its root `grund.toml` sets `project_name = "root"` and `[workspace] members = ["apps/api", "packages/*"]`. Of its three member directories, `apps/api/` holds an `AGENTS.md`, and `packages/core/` and `packages/ui/` hold none. `grund init {repo_copy}` writes this into the root `AGENTS.md`, byte for byte, after `### Project namespaces` ([§FS-init.2.3.4](FS-init.md#234-managed-block-content-sections)) and before `### Declarations and citations`, with one bullet per foreign project, sorted by alias ([§FS-init.2.3.4](FS-init.md#234-managed-block-content-sections)):

```markdown
### Workspace members

Cross-project citations use §alias/<ID>, one alias segment per workspace level.

- [`api`](apps/api/)
- [`core`](packages/core/)
- [`ui`](packages/ui/)
```

`grund init {repo_copy}/apps/api` writes the member's own section into `apps/api/AGENTS.md`, the same bytes whether or not that file or the root's `AGENTS.md` exists. It has the same discoverability line, then bullets for `core`, `root` and `ui`, linked `../../packages/core/`, `../../` and `../../packages/ui/`, and leaves out `api`, the project being written ([§FS-init.2.3.4](FS-init.md#234-managed-block-content-sections)).

With `project_description = "Payment API service"` in `apps/api/grund.toml`, the root's bullet for `api` reads `` - [`api`](apps/api/): Payment API service ``, while `ui`, which has no config, keeps its bare link ([§FS-init.2.3.4](FS-init.md#234-managed-block-content-sections)). A target no workspace claims gets neither this section nor `### Project namespaces` ([§FS-init.2.3.4](FS-init.md#234-managed-block-content-sections)).

## 7. The `none` template

`grund init {repo_copy} --template=none` runs over a target with no `AGENTS.md` and no config in either discovery form; the target may already hold a `docs/` and a `tests/` of its own. The run reports `wrote AGENTS.md` and `wrote grund.toml`, then the `next:` block of [§FS-init.2.2.2](FS-init.md#222-the-next-block), and exits `0`. It adds those two files and changes nothing under `docs/` or `tests/` ([§FS-init.3.6](FS-init.md#36-bare-init-writes-the-config-and-the-entrypoints-and-nothing-in-a-home)): the `grund.toml` is the envelope of [§FS-init.2.4.9](FS-init.md#249-the-none-templates-config), and the block's Project map is the paragraph of [§FS-init.2.3.4](FS-init.md#234-managed-block-content-sections). `grund check` over that tree exits `0`, and `grund init --check` reports `exists AGENTS.md` and `exists grund.toml` ([§FS-init.1.6](FS-init.md#16-templates)).

An unknown template name and `--template=none --scaffold` are refused with exit `2` and nothing written, each with its one line ([§FS-init.1.6](FS-init.md#16-templates)). Over a target that already has a `grund.toml`, `--template=none` keeps it, reports `exists grund.toml` and the note of [§FS-init.1.6](FS-init.md#16-templates), and then prints the `next:` block the kept config earns ([§FS-init.2.2.2](FS-init.md#222-the-next-block)).

## 8. A project without FS

The configured-kind form of [§FS-init.2.2.2](FS-init.md#222-the-next-block), over one fixture: a version-1 `grund.toml` that declares one kind, `REQ`, homed in `docs/req`, and scans `docs` and `README.md`:

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

`docs/req/REQ-login.md` declares `REQ-login`, `docs/req/README.md` indexes it, and `README.md` cites it; the tree is under Git, with no `AGENTS.md`. `grund init {repo_copy}` exits `0` with this on stderr:

```text
wrote AGENTS.md
exists grund.toml

next:
  1. run `grund check`
  2. allocate an ID:  ID=$(grund id REQ "…")  then add it under docs/req
see AGENTS.md for the full workflow.
```

No block names `FS` or `docs/functional-spec`, since the project declares neither, and none offers `--scaffold` ([§FS-init.2.2.2](FS-init.md#222-the-next-block)). Without `docs/req/` the block opens with the step `create the REQ home (docs/req)` and numbers the others from 2, on a first run and on a refresh alike; a row homed in a file prints `create the REQ home (requirements.md)` and `then add it to requirements.md`. Without the `folder` line, so that `REQ` has no home, the allocation step ends `then declare it in a source doc-comment` and the block names no directory.

`grund init {repo_copy} --scaffold` lays grund's whole software-development starter, the FS home included, rather than the selected home alone ([§FS-init.2.2.2](FS-init.md#222-the-next-block)). It exits `0`, and its transcript is that of [§FS-init-fixtures.2](FS-init-fixtures.md#2-scaffold-form) with `exists grund.toml` in place of `wrote grund.toml`, `exists docs/req/README.md` in place of `wrote docs/requirements/README.md`, and step 2 reading `allocate an ID:  ID=$(grund id REQ "…")  then add it under docs/req`. Under `--scaffold`, step 2 over a row homed in a file reads `then add it to requirements.md` with the second line `` (H2: `## <ID>: <one-line statement of the behavior>`) ``, and over a row with no home it ends `then declare it in a source doc-comment` with the second line `` (doc-comment: `<ID>: <one-line statement of the behavior>`) ``.
