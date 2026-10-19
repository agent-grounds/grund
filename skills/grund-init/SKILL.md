---
name: grund-init
description: Use when bootstrapping or adopting grund in a repository, especially when the user wants an interactive guided setup for grund init, grund.toml, AGENTS.md, docs scaffolding, citation format, scan scope, output format, or Markdown link settings.
---

# grund init

Guide the user through `grund` adoption. `grund init` itself is non-interactive, so this skill acts as the interactive wrapper: inspect the repository, recommend suitable settings, ask the user to confirm or override every option, write `grund.toml`, run `grund init`, then validate.

## Workflow

1. Inspect the target repo before asking questions. Find existing specs, artifact types, roadmaps, changelogs, decisions, plans, tests, and agent instruction files before recommending anything.
2. Present a short "detected repo shape" summary and recommended setup.
3. If existing specs or spec-like artifacts are present, show the canonical `grund` artifact types beside the detected project-specific sections/tags/document classes, then ask which artifact model to adopt before writing config or refactoring docs.
4. Ask each remaining setup/config question below. For every question, include the recommended value, repo evidence, pros, cons, and when to choose something else.
5. Write `grund.toml` from the analysis before running `grund init`, so generated guidance reflects the repository's actual grammar, marker, strict mode, kinds, artifact folders, and scan scope. On a fresh setup with no `grund.toml`, `grund init` writes the version-2 starter itself: a short file holding only what differs from a default — the project name, each kind's row and places, and each kind's title. Run it first, then add only the settings the user chose to change. `grund config show` prints every effective setting, defaults included, in the file's own version; show it to the user rather than writing defaults out, and copy a line from it into `grund.toml` to change that setting. For the existing-structure model, `grund init --template=none` lays a version-2 `grund.toml` that declares no kinds: add one `[schema.kinds.<KIND>]` table per kind of the repo's own, with the `folders` or `files` its declarations live in, then run `grund init` again so the entrypoints teach them.
6. Run `grund init [path] [--name NAME] [--force]`, adding `--scaffold` only when the repo is fresh or the user selected a canonical-layout migration that needs the template's starter files. Omit `--name` to reuse a target config's `project_name`, falling back to the target directory name when the config has none; pass it only to override that generated identity for this run. Preview the run with `--dry-run` if the user wants to inspect what will change before committing, and offer `--check` — the same preview, writing nothing and exiting `1` when a file is still pending — as the pre-commit or CI gate that catches a managed block whose text drifted while its version heading stayed current. `init` refuses a target that no `.git`, `.hg`, `.jj`, or `.svn` marker covers, in it or any ancestor: for a directory not yet under version control, either run the VCS's own init first or pass `--no-vcs`. It also refuses the home directory and the machine-global agent instruction files (`~/.claude/CLAUDE.md`, `~/.codex/AGENTS.md`, and the rest of that set) with no flag at all — those say the path is wrong, not that an option is missing, so re-point the run rather than trying to force it.
7. Run `grund config validate [path]` and `grund check [path]`.
8. Summarize generated files, validation results, existing specs/artifacts found, and any follow-up cleanup.
9. Optionally offer to wire the user's editor to the `grund-lsp` server (see Editor Setup). Editor configuration is the user's one-time work, not something `grund init` writes — so only offer it, and prefer editor **user** settings over a per-repo config to make the configuration available across projects. User settings do not grant Workspace Trust; see Editor Setup for the VSCode / VSCodium prerequisite.

## Repo Analysis First

Use `rg` and `rg --files` first. Prefer evidence from existing files over generic defaults.

Analyze:

- Existing `AGENTS.md`, a `grund.toml` in either discovery location (root or `.agents/`), and grund-style citations.
- Documentation layout: `docs/`, `e2e/`, `spec/`, `rfcs/`, `adr/`, `decisions/`, `roadmap`, `changelog`.
- Existing artifact types and their homes: specifications, requirements, RFCs, ADRs/decisions, roadmaps, changelogs, plans, end-to-end fixtures, examples, package READMEs, generated reports, and runtime logs. Use these to choose configured places and source exclusions: version 2 uses `[schema.kinds.<KIND>]` and `[schema.sources] exclude`; version 1 uses `[[kinds]]` and `[scan] include`/`exclude`. Do not add generic folders when the repo already has project-specific artifact homes.
- Source layout: `src/`, `lib/`, `crates/`, `packages/`, `apps/`, `services/`, `cmd/`, `internal/`, `pkg/`, `test/`, `tests/`.
- Languages from file extensions and manifests such as `Cargo.toml`, `package.json`, `pyproject.toml`, `go.mod`, `pom.xml`, `build.gradle`, `.csproj`, `Package.swift`, `Gemfile`, `composer.json`, `build.sbt`, `CMakeLists.txt`, `dbt_project.yml`.
- Ignore/build/vendor directories from `.gitignore` and common generated paths.
- Existing ID/citation patterns, including headings and tokens that look like `FS-001-login`, `FS-login`, `ADR-001`, `RFC-42`, or `§FS-...`.
- Existing rendered-doc target: GitHub default, GitLab, MkDocs, Pandoc, or unknown.
- Whether this is a fresh repo, a docs-heavy repo, or an existing codebase adopting grund.

## Asking Style

Ask the user to confirm or change a recommendation, never a blank preference survey. Each prompt must include the recommended value, evidence from repo analysis, pros, cons, and when to choose a different value. Let the user accept the full set quickly while still seeing and overriding every option.

## Init Questions

Ask first:

- Target path: default `.`
- Project name: default target config value, then target directory basename
- Artifact model when existing specs are detected: canonical `grund`, canonical core plus project-specific extras, or existing structure with citations
- Template and starter files: `--template=software-development` (the default, canonical kinds) or `--template=none` (no kinds, for the existing-structure model); `--scaffold` to lay the starter files: default no for existing repos, yes for fresh repos
- Existing file behavior: append/update default, or `--force`

## Existing Specs Adoption

When the repo already contains specs, requirements, ADRs, RFCs, design docs, plans, tests, or other spec-like artifacts, do not silently scaffold over them or refactor them into canonical `grund` folders. First show the user:

- The canonical `grund` artifact types:
  - `GRUND` — reason / grounding doc
  - `GOAL` — project goals
  - `FS` — functional specs
  - `AR` — architecture specs
  - `DF` — functional decisions
  - `DA` — architectural decisions
  - `E2E` — executable scenarios
  - `RM` — roadmap
- The detected project-specific sections, tags, or document classes, such as `ADR`, `RFC`, `REQ`, `SPEC`, `DESIGN`, `PLAN`, `RUNBOOK`, or whatever the repo already uses.

Then ask the user to choose one adoption model:

| Option | When to recommend it | What the agent does |
|---|---|---|
| Canonical `grund` | Fresh or lightly documented repos, or users who explicitly want to reorganize around `grund` conventions. | Use `GRUND`, `GOAL`, `FS`, `AR`, `DF`, `DA`, `E2E`, and `RM` as the complete artifact model; add or refactor docs toward canonical homes only after user confirmation. |
| Canonical core plus project-specific extras | Repos with useful existing ADRs/RFCs/requirements but no clear behavior-vs-architecture backbone. | Use `GRUND`, `GOAL`, `FS`, and `AR` as the grounding backbone, then add custom `[[kinds]]` for project-specific artifacts. |
| Existing structure with citations | Mature repos with a strong existing taxonomy or high migration cost. | Start from `grund init --template=none`, whose config declares no kinds; preserve current sections/tags/document classes, configure `[[kinds]]` and `[scan]` around them, and add `grund` citations/declarations without forcing canonical folders. |

The question should include a recommendation grounded in the inventory. Do not write `grund.toml`, run `grund init --scaffold`, move documents, rename headings, or add bulk citations until the user chooses the adoption model. Once selected, use the model to decide whether the setup is only config plus entrypoint refresh, or a broader docs refactor with a visible plan.

## Config Questions

Ask these in order.

### Top-level

`grund_config_version`

Do not ask. A fresh setup gets `2` from `grund init`. An existing file keeps its version: a file without the key is version 1 and keeps loading unchanged, and moving it is `grund config migrate` (see Version 1 to version 2). The questions below name each setting by its version-1 key; for a version-2 file, `grund config show` prints the table each one lives in.

`project_name`

Pros of explicit name: stable display name for agents/tools. Cons: one more metadata value to maintain if repo is renamed. Default: the target config's existing value, otherwise the target directory name.

### `[reference]`

`marker`

Default: `§`. Pros: visually distinct, avoids false positives. Cons: awkward to type without trigger/editor help. Alternatives: `@`, `#`, or `ref:` only if the team has strong conventions.

`trigger`

Default: `$$`. Pros: easy typing path to `§`. Cons: may conflict with math-heavy Markdown or template languages. Recommend changing only if `$$` is common in the repo.

`strict` (version 1 only)

Version 2 recognizes only marked citations, with no opt-out or strict switch ([config-derived guidance](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-init.md#2381-what-the-config-fills-in)). Do not ask about or write `[reference].strict` for version 2. For version 1, the default is `true`: citations are intentional and explicit. Use `false` only as a compatibility window for existing bare citations; it also admits more accidental matches. Run `grund fmt --marker` before removing that override or setting strict back to `true`.

`require_grounding`

Default: `false`. Pros true: every scanned source file must cite or declare a grounding ID. Cons true: high adoption cost and noisy until coverage discipline exists. Recommend false initially; enable later or in CI once coverage is deliberate.

### `[id]`

`format`

Default: `{kind}-{number}-{slug}`.

```
       ┌──────────── citation ──────────┐
           ┌────────── ID ──────────┐
  [§] KIND - [number -] slug [.section]
   │   │       │         │       │
   │   │       │         │       └─ optional dotted path, arbitrary depth (.3, .3.1, .3.1.5, …)
   │   │       │         └───────── [a-z][a-z0-9-]*
   │   │       └─────────────────── optional ordinal (e.g. 001)
   │   └─────────────────────────── G│FS│AR│DA│DF│E2E│RM│DISC
   └─────────────────────────────── citation marker (writing only)
```

Pick one per repo and keep it stable — mixing is unsupported because citations would look identical but resolve under different rules.

| Scheme                                     | Example             | Benefit                                                                                                          | Trade-off                                                                |
|--------------------------------------------|---------------------|------------------------------------------------------------------------------------------------------------------|--------------------------------------------------------------------------|
| `{kind}-{number}-{slug}` *(default)*       | `FS-014-user-login` | Number is the stable identifier; slug is descriptive and can be **renamed freely** without breaking citations.   | Two tokens to type; needs `grund id` to allocate the next number.        |
| `{kind}-{number}` (RFC-style)              | `FS-014`            | Maximally stable — no slug to drift. Familiar from RFCs/PEPs/JEPs/ADRs.                                          | Opaque at the call site: `§FS-014` tells you nothing without resolving it. |
| `{kind}-{slug}` *(`grund` itself uses this)* | `FS-user-login`     | Self-describing — reads like English in prose and code. No number to allocate.                                   | Renaming a slug rewrites every citation. Slug must be unique per kind.   |

Rule of thumb: pick `{kind}-{slug}` until rename churn or ID count starts to hurt; switch to `{kind}-{number}-{slug}` when it does.

If existing IDs are detected, prefer matching them over the canonical default.

`section_separator`

Default: `.`. Pros: natural `FS-login.3.1` syntax. Cons: can collide with custom ID formats or slug rules. Change only for existing conventions, e.g. `#` or `:`.

`number_pattern`

Default: `\d+`. Pros: simple numbered IDs. Cons: does not enforce fixed width like `001`. Use `\d{3}` only if the team wants strict padded numbers.

`slug_pattern`

Default: `[a-z0-9][a-z0-9-]*`. Pros: URL-friendly, portable, predictable. Cons: excludes uppercase and underscores. Relax only to match existing IDs.

### `[[kinds]]`

Default kinds: `GRUND`, `GOAL`, `FS`, `AR`, `DF`, `DA`, `E2E`, `RM`. The config `grund init` writes also declares `REQ`, hard requirements the project must never break, in `docs/requirements/`. `REQ` has no built-in default, so a config that leaves `[[kinds]]` out has none. `FS` lives in `docs/functional-spec/`, one declaration per file, with no `grund.toml` and in the config `grund init` writes alike: `grund init --scaffold` writes `docs/functional-spec/README.md`, the `FS` index, and no `requirements.md`. Before grund 0.19.0 a repository with no `grund.toml` homed `FS` in `requirements.md` ([the decision](https://github.com/agent-grounds/grund/blob/main/docs/decisions/functional/DF-fs-home-lands-in-0-19.md)); to keep `FS` there, run `grund init` and write `files = ["requirements.md"]` on its `FS` row.

Ask whether to keep defaults, remove unused kinds, or add project-specific kinds. To start from none instead, `grund init --template=none` writes a version-2 file with no `[schema.kinds]` row, and each kind is then added as its own `[schema.kinds.<KIND>]` table.

Pros of defaults: matches grund docs and generated scaffold. Cons: some repos may not need all categories. Pros of custom kinds: adapts to existing taxonomy. Cons: replacing defaults means the full list must be copied; no merge.

For each kind ask: `kind`, `folder`, `title`.

A directory that holds agent-facing content rather than declarations — skills, runbooks, a prompt library, a test suite — is a kind with `citable = false`: it gets a home, a title and citation rules, and declares no IDs. Ask for one wherever the repo has such a directory and the answer to "should an agent be told this exists?" is yes.

### `[scan]`

`include`

Default: `["requirements.md", "docs", "e2e", "src"]`. Pros: focused, avoids scanning root clutter. Cons: misses specs/citations outside these dirs. Every configured `[[kinds]]` home is walked whether or not it is listed here, so this key names the *extra* roots. Base the recommendation on actual directories. Do not include paths that do not exist unless `--scaffold` will create them.

`exclude`

Default: `["target", "node_modules", ".git", "dist", "build", ".venv"]`. Pros: skips generated/vendor-heavy trees. Cons: can hide intentional generated docs if stored there. Usually keep defaults and add repo-specific build/cache dirs.

`extensions`

Default includes common Markdown and source extensions. Pros: polyglot coverage. Cons: scanning more extensions costs time and may surface noise. Recommend only extensions found in the repo plus Markdown, unless the repo is fresh.

`comment_prefixes`

Default: `["//", "#", ";", "--", "*", "/*"]`. Pros: broad language support. Cons: may match prose-like comments in some languages. Usually keep defaults, or narrow to the detected language set for established repos.

`docstring_python`

Default: `true`. Pros: Python docstrings can carry inline declarations/citations. Cons: docstring scanning can surface intentional prose examples. Recommend true if Python files exist.

`respect_gitignore`

Default: `true`. Pros: avoids ignored/generated/vendor files. Cons: ignored files with real specs will not be scanned. Keep true unless the repo intentionally stores tracked specs in ignored paths.

### `[output]`

`format`

Default: `text`. Pros text: readable locally and in CI logs. Cons text: harder for tools to consume. Use `json` for automation dashboards or custom CI parsing.

`color`

Default: `auto`. Pros: readable terminal output without polluting non-TTY logs. Cons: not fully meaningful until colored output lands. Keep `auto`.

`relative_paths`

Default: `true`. Pros: deterministic, repo-root-relative reports. Cons: less convenient when running from a subdirectory and expecting local paths. Keep true for CI and shared logs.

### `[fmt.cross_refs]`

`enabled`

Default: `true`. Pros true: Markdown citations can become normal links for rendered docs. Cons true: extra churn and renderer-specific anchors. Recommend true unless the repo intentionally keeps citations unwrapped in Markdown; set false to opt out.

`anchor_format`

Default: `github`. Options: `github`, `gitlab`, `mkdocs`, `pandoc`, `none`.

Pros of matching renderer: links work in published docs. Cons: wrong profile creates broken anchors. Use `none` if only file links are desired.

### `[citations]`

<!-- BEGIN citation-directions -->
# Citation directions

`[citations]` describes which kinds of documents cite which other kinds. The
five levels use two rule classes and two enforcement surfaces, as specified by
[the `[citations]` section of grund's config spec](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-config.md#39-citations--citation-direction-rules):

| Level | Rule class | Checked per | Surface |
| --- | --- | --- | --- |
| `must` | obligation | declaration | `grund check` error, and the generated entrypoint |
| `should` | obligation | declaration | `--suggestions` and the generated entrypoint |
| `may` | permission | — | never checked |
| `should-not` | prohibition | citation site | `--suggestions` and the generated entrypoint |
| `must-not` | prohibition | citation site | `grund check` error, and the generated entrypoint |

The grammar is: entries in one array are all required, while `|` inside one
entry means any one of its alternatives. Therefore `must = ["FS|GOAL"]`
requires a citation to either `FS` or `GOAL`, whereas `must = ["FS", "GOAL"]`
requires citations to both. Nearly every real rule is one entry with `|`.
`grund schema` draws a repository's directions, each `|` entry as one
obligation.

From grund 0.19.0, a prohibited citation of a name under a number produces the ordinary `forbidden-citation` error and makes the check fail. The mixed-only warning window from 0.18.0 has ended. Recommended directions remain opt-in suggestions, and explicit `warn`/`warn-not` strengths remain warnings ([the citation-direction rules](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-rules.md#712-a-finding-only-a-citation-of-a-name-under-a-number-raises-warns-until-0190)).

The citing side may be any configured kind, including a non-citable kind, or
the homeless `code` kind. The cited side must be citable, because a citation
needs an ID to point at. A non-citable kind may cite and is labelled by its
home, such as `skills/`; the homeless kind covers any file outside a configured
kind home and renders last. In a workspace a target may name another project. A bare `KIND` is judged against the local project's kinds when the config loads. `alias/KIND` is judged against the kinds of the project that `alias` names, and `*/KIND` against the kinds of every loaded project, both when the workspace is expanded. So `[citations.FS] must = ["api/API"]` at a workspace root loads once member `api` configures `API`, and it is enforced exactly as the rule `Each FS must cite at least one api/API.` is. A pinned or `*/` target the workspace cannot accept is never matched. Until grund 0.19.0 it prints one `warning:` line, and from 0.19.0 that line is an error. This covers an unknown or non-citable kind in that project, and an alias the workspace does not load. A run with no workspace, such as a member's own run, cannot judge a target pinned to another project at all, and its warning says to check from the workspace root.

An obligation on a citable kind that declares no IDs has no unit: no
per-declaration unit fires. If a `must` or `should` obligation is configured,
the kind's folder is walked, and at least one non-entry file is successfully
scanned, `grund check` emits the non-fatal run-level
`empty-citation-obligation` warning ([the check specification says exactly when](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-check.md#221-citation-direction-obligation-applies-to-nothing)).
A directory without declarations is usually `citable = false`, not a citable
kind with an empty rule; make that setting when the directory is a place rather
than an ID namespace. This is the no-unit trap described by
[the non-citable-kinds decision](https://github.com/agent-grounds/grund/blob/main/docs/decisions/functional/DF-non-citable-kinds.md#25-obligations-get-a-per-file-unit-and-grounding-follows-the-home).

A kind's own marked index entry is still subject to citation directions. Keep
the entry when repairing a prohibition: replacing it with a plain link fails
the index rule. For a v2 kind or global negative default, add the target kind
to the citing kind's `may` list while preserving existing permissions. For
example, in `[rules.citations.REQ]`, extend `may = ["GOAL"]` to
`may = ["GOAL", "REQ"]`. An explicit prohibition instead requires changing
the matching `must-not` or `warn-not` list; an independent rule sentence
requires changing the named rule. Adding `may` does not override either.
The finding names the applicable repair; ordinary citations keep their
plain-link advice, and suggestions keep their existing wording
([the index-entry repair specification](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-check.md#3122-repair-advice-preserves-a-required-index-entry)).

The config and the generated section below are the same example. The TOML is
shown beside the Markdown it renders:

```toml
[citations]
default = "may"

[citations.FS]
should = ["GOAL|FS"]        # either
must-not = ["AR"]

[citations.DA]
must = ["AR", "FS"]         # both

[citations.skill]           # citable = false: cites, is never cited
must = ["FS"]
must-not = ["AR"]

[citations.code]
should = ["FS|AR"]
```

```markdown
### Citation directions

`must`/`never` are `grund check` errors; `should`/`avoid` are suggestions (`grund check --suggestions`).

- Each **FS** declaration should cite GOAL or FS; never cite AR.
- Each **DA** declaration must cite AR and FS.
- Each file in **skills/** must cite FS; never cite AR.
- Each source file outside the Project map (**code**) that cites anything should cite FS or AR.
Anything not listed above is allowed.
```
<!-- END citation-directions -->

Source: [grund's canonical citation-directions page](https://github.com/agent-grounds/grund/blob/main/docs/user-facing/citation-directions.md); the
marked region above is kept byte-identical to it by the asset-sync test.

Pros: one complete explanation teaches the grammar before a user edits rules,
and the checked copy keeps repository and installed setup surfaces aligned.

Cons: the explanation adds setup text for repositories that never enable
`[citations]`.

Recommend starting with the canonical climbing ruleset from the template
comment at `should`, then promoting each kind's rule to `must` or `must-not`
once the repository passes it. Choose a different set only when the repo's
artifact dependencies genuinely differ; keep one array entry per required
target and use `|` for alternatives.

## Recommendation Heuristics

| Setting | Recommendation from repo evidence |
| --- | --- |
| `--template` and `--scaffold` | With little or no `docs/` or `tests/` structure, use `grund init --scaffold`: the default `--template=software-development` lays canonical kinds and starter files. With meaningful docs or tests, omit `--scaffold` and add only missing grund files. For an existing taxonomy, use `grund init --template=none`, add the repo's `[schema.kinds.<KIND>]` tables to `grund.toml`, then run `grund init` again. `--template=none --scaffold` is refused because `none` has no starter files. |
| `[reference].strict` (version 1 only) | Version 2 has fixed marked-only recognition and no strict switch ([config-derived guidance](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-init.md#2381-what-the-config-fills-in)). For version 1, recommend the default `strict = true` for first adoption, especially with noisy ID-like tokens or deliberate citation hygiene. Use `strict = false` only for existing bare citations that need a compatibility window before `grund fmt --marker`. |
| `[reference].require_grounding` | Recommend `false` initially; `true` only with broad spec-to-code citation coverage or an explicit request for strict co-change discipline. |
| `[schema] id_format` (v2); `[id] format` (v1) | Use `{kind}-{number}-{slug}` for fresh repos or stable IDs with readable names; `{kind}-{number}` for existing ADR/RFC-style numbered IDs; `{kind}-{slug}` for title/slug-based docs where numeric allocation would feel artificial. A kind override uses `[schema.kinds.<KIND>] id_format` in v2, `[[kinds]] format` in v1. |
| `[[kinds]]` | Start from defaults. Add custom kinds for existing categories such as `ADR`, `RFC`, `REQ`, `API`, `SEC`, or `OPS`; remove defaults only when they would clearly confuse the project. |
| Source scope | In v2, configure actual places with `[schema.kinds.<KIND>] folders`/`files`; the `code` complement covers unplaced files, and `[schema.sources] exclude` prunes generated paths. In v1, `[scan] include` lists actual directories, such as `["docs", "e2e", "src", "crates", "tests"]` for a Rust workspace. |
| `[fmt.cross_refs].anchor_format` | Use `github` for GitHub or no evident renderer; `gitlab` for GitLab; `mkdocs` when `mkdocs.yml` exists; `pandoc` with evident Pandoc config/build scripts; `none` for file-only links without section anchors. |

## Language And Repo Shape Examples

Use these examples to turn repo evidence into recommended `[scan]` settings. Include only directories that exist, unless `--scaffold` will create them.

### Rust

Evidence: `Cargo.toml`, `Cargo.lock`, `crates/`, `src/**/*.rs`.

```toml
[scan]
include = ["docs", "e2e", "src", "crates", "tests"]
extensions = ["md", "rs"]
exclude = ["target", ".git"]
comment_prefixes = ["//", "/*", "*"]
```

Pros: covers workspace crates, integration tests, and Rust doc comments.
Cons: may miss generated docs outside `docs/`.

### TypeScript / JavaScript

Evidence: `package.json`, `pnpm-workspace.yaml`, `tsconfig.json`, `src/`, `apps/`, `packages/`.

```toml
include = ["docs", "e2e", "src", "apps", "packages", "tests"]
extensions = ["md", "ts", "tsx", "js", "jsx"]
exclude = ["node_modules", "dist", "build", "coverage", ".next", ".turbo", ".git"]
comment_prefixes = ["//", "/*", "*"]
```

Pros: works for frontend apps and monorepos.
Cons: broad monorepos may need narrower package selection.

### Python

Evidence: `pyproject.toml`, `setup.py`, `requirements.txt`, `src/`, package dirs, `tests/`.

```toml
include = ["docs", "e2e", "src", "tests"]
extensions = ["md", "py"]
exclude = [".venv", "__pycache__", ".pytest_cache", ".mypy_cache", "build", "dist", ".git"]
comment_prefixes = ["#"]
docstring_python = true
```

Pros: supports citations in comments and docstrings.
Cons: docstring scanning can surface intentional prose examples.

### Go

Evidence: `go.mod`, `cmd/`, `internal/`, `pkg/`, `*.go`.

```toml
include = ["docs", "e2e", "cmd", "internal", "pkg", "tests"]
extensions = ["md", "go"]
exclude = ["vendor", "dist", "build", ".git"]
comment_prefixes = ["//", "/*", "*"]
```

Pros: matches common Go project layout.
Cons: single-package repos may only need `["docs", "src"]` or `["docs", "."]` if code is at root.

### Java / Kotlin / Gradle

Evidence: `pom.xml`, `build.gradle`, `settings.gradle`, `src/main`, `src/test`.

```toml
include = ["docs", "e2e", "src"]
extensions = ["md", "java", "kt"]
exclude = ["target", "build", ".gradle", ".git"]
comment_prefixes = ["//", "/*", "*"]
```

Pros: covers Maven and Gradle conventions.
Cons: multi-module builds may need module directories added explicitly.

### C / C++

Evidence: `CMakeLists.txt`, `Makefile`, `src/`, `include/`, `lib/`, `tests/`.

```toml
include = ["docs", "e2e", "src", "include", "lib", "tests"]
extensions = ["md", "c", "cpp", "h", "hpp"]
exclude = ["build", "dist", "out", ".git"]
comment_prefixes = ["//", "/*", "*"]
```

Pros: covers implementation and public headers.
Cons: vendored headers should be excluded if present.

### C# / .NET

Evidence: `*.csproj`, `*.sln`, `src/`, `test/`, `tests/`.

```toml
include = ["docs", "e2e", "src", "test", "tests"]
extensions = ["md", "cs"]
exclude = ["bin", "obj", "TestResults", ".git"]
comment_prefixes = ["//", "/*", "*"]
```

Pros: covers normal solution layout.
Cons: generated code folders may need extra excludes.

More repository-shape examples for Ruby/Rails, PHP, Swift, Scala, and SQL live in
the [supplementary init examples](https://github.com/agent-grounds/grund/blob/main/docs/user-facing/init-repo-shapes.md).

### Polyglot Monorepo

Evidence: multiple manifests and top-level `apps/`, `packages/`, `services/`, `libs/`, `tools/`.

```toml
include = ["docs", "e2e", "apps", "packages", "services", "libs", "tools", "tests"]
extensions = ["md", "rs", "go", "java", "kt", "ts", "tsx", "js", "py", "c", "cpp", "cs", "rb", "php"]
exclude = ["target", "node_modules", ".git", "dist", "build", ".venv", "coverage", ".next", ".turbo"]
comment_prefixes = ["//", "#", ";", "--", "*", "/*"]
docstring_python = true
```

Pros: broad coverage for adoption across teams.
Cons: can be noisy; recommend narrowing after the first `grund check`, then `grund check --full .` to see what the narrowing left out.

### Version 2: `[schema.sources]`

With `grund_config_version = 2` there is no `[scan]`: name the languages the evidence shows, map each suffix the v2 epoch lacks, and write generated paths as gitignore-style globs, which have no built-in defaults ([sources guide](https://github.com/agent-grounds/grund/blob/main/docs/user-facing/sources.md)). For C with headers:

```toml
[schema.sources]
languages = ["markdown", "c"]
exclude = ["build/"]
[schema.sources.extensions]
h = "c"
```

Only doc comments declare an ID in v2; `declarations = "any-comment"` keeps v1's reading.

A v2 row may list several places, `[schema.kinds.FS] folders = ["docs/spec/core", "docs/spec/plugins"]`, and `[schema.kinds.code] folders = ["src", "scripts"]` names what v1's `include` did.
Ownership in one line: a file listed in a row's `files` is that row's, otherwise the deepest folder holding it decides, and a file in no place is `code`'s ([ownership](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-config-v2.md#schemaplacesownership-a-listed-file-then-the-deepest-folder)).

When multiple language examples apply, merge them conservatively: union the real include dirs, union the extensions actually present, and union generated/cache excludes. Prefer a narrower first config that passes cleanly over an over-broad config that floods the user with findings — and then run `grund check --full .`, which reports the citations that resolve to nothing in the directories `include` left out. A narrow `include` is a good starting point precisely because that check exists; without it, the citations the first config forgot are invisible rather than merely unchecked ([the full-tree scope](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-check.md#13-the-full-tree-scope---full)).

### Version 1 to version 2

To move an existing version-1 `grund.toml` to version 2, never hand-translate it: run `grund config migrate --to 2` without `--write` first, which previews the candidate on stdout and the report on stderr and writes nothing; make each refusal's repair as a version-1 edit, checked with `grund check` and committed on its own, and preview again; then add `--write`, and add `--drop-comments` only after reading the comments the preview lists, because it drops exactly those ([migrating guide](https://github.com/agent-grounds/grund/blob/main/docs/user-facing/migrating.md)).

Review each carried-comment warning at its named source and destination: migration retains the comment but translated the setting's value vocabulary, so any listed values may need updating. Check it even when the prose seems neutral; `--drop-comments` retains carried comments and their warnings ([carried-comment guidance](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-config-migrate.md#411-check-carried-comments-when-the-value-vocabulary-changes)).

After a successful write, the final stderr note reminds you that existing `AGENTS.md` managed instructions may still describe version 1. Run `grund init` in the migrated project to refresh them: migration writes only its config and leaves those instructions unchanged. The reminder is qualified and appears even when `AGENTS.md` is absent; it does not mean a file was inspected or found stale. Previews, refused or failed writes and already-version-2 runs omit it ([after-write guidance](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-config-migrate.md#65-what-it-says-after-the-write)).

When a run prints `grund.toml is version 1; migrate it with grund config migrate --to 2`, preview that command first, then apply it with `--write` only once the preview shows no `differs:`; do not silence the warning with `--ignore config-version-1`.

## Splitting off a member

Split off a member when an app, package, service, or subproject needs to be
independently checked. An ordinary module or component that still belongs to the
current project does not need a member; a documentation folder alone is not a
project boundary.

1. Give the member its own `grund.toml`, with the kinds and scan scope it owns,
   and a stable `project_name` to use as its alias.
2. Register the member directory in the workspace root's `[workspace]` `members`.
   When splitting a standalone repository, add that workspace configuration to
   its root `grund.toml` first.
3. Run `grund init` in the member to render its own agent entrypoints, and at the
   root to refresh its workspace guidance. Init does not create the topology or
   initialize sibling members for you.
4. Keep local citations local. Across members, use `§alias/<ID>` (substitute your
   configured citation marker), with one alias segment per nesting level:
   `§group/member/<ID>` for a nested member.
5. Run `grund check` from the workspace root to validate the whole workspace and
   its qualified citations.

A member-local `unknown project alias` finding still fails. When its alias
names the verified enclosing workspace's root project, follow the root location
and `grund check` remedy in the finding to validate the cross-project citation.
Do not rename that alias merely because a member cannot resolve it. The root
check may still find a missing target; an unknown alias without this verified
relationship is not evidence of a workspace root alias.

See [workspace configuration and command scope](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-workspace.md)
for membership, alias derivation, and nested workspaces.

## Validation

After writing config, run:

```bash
grund config validate .
grund init .
grund check .
grund check --full .
```

The last line checks beyond configured source scope: `grund check .` uses the ordinary scope, while `grund check --full .` widens the walk to report unresolved references and section headings outside declaration bodies there ([the full-tree scope](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-check.md#13-the-full-tree-scope---full)). Version 1 scopes with `[scan] include`; version 2 uses configured places and the `code` complement. Source exclusions still apply: `[schema.sources] exclude` in v2, `[scan] exclude` in v1. The wider walk stops only at *declared* workspace members, not at any nested `grund.toml`, so a vendored or example project inside the tree is read under this project's grammar; exclude it with the loaded version's source key, or list it in `[workspace] members` if it is one of ours.

If custom config affects `AGENTS.md`, ensure `grund.toml` exists before `grund init` so the generated managed block reflects the selected ID grammar, marker, strict mode, kinds, and existing artifact layout.

If a project needs its own grammar or independent check, follow
[Splitting off a member](#splitting-off-a-member), then validate from the workspace root.

## Scoped Guidance

In a version-2 config, `grund init` writes each row's guidance — its citation directions, its clause of the grounding sentence, its kind's declaration shape and the chapter rules about its kind — into the managed block of an `AGENTS.md` in the deepest directory holding the row's places, rather than into the root entrypoint ([scoped guidance](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-init.md#23511-scoped-guidance-a-rows-guidance-lands-above-its-places)). Folders `docs/spec/core` and `docs/spec/plugins` land in `docs/spec/AGENTS.md`; a row whose places share no directory, a kind with no place, and the complement over the whole root stay in the root block. The root block's `### Scoped guidance` section lists every scoped file it wrote, and `init` owns exactly the files on that list: it updates them, mirrors a root `CLAUDE.md` or `GEMINI.md` symlink beside them, and removes the ones a later config no longer places. `grund check` reports a scoped file that is missing, stale or orphaned as `agents-init`, and `grund init` fixes it.

Before writing in a directory, read the scoped `AGENTS.md` of that directory or the nearest one above it when your agent has not already loaded it: it holds the rules and the shape the files you are about to write are checked against. Some agents load a nested `AGENTS.md` by themselves and some do not ([which do](https://github.com/agent-grounds/grund/blob/main/docs/decisions/functional/DF-scoped-guidance.md#loading-which-agents-load-a-scoped-file-by-themselves)); the pointer in the root block is there for the ones that do not.

A team that wants every rule in the root file sets `[presentation] rules = "root"` in `grund.toml`; the root block is then the one a version-1 config renders, plus the declaration shapes. Version-1 configs always render at the root. Offer the opt-out when the team uses an agent that reads only the root entrypoint and will not follow the pointer.

## Seeing the Schema

Before writing a declaration or a citation, run `grund schema` to see what the repository is held to: its kinds and places, each kind's fields, which units must be grounded, and who must cite whom and how strongly. It reads the configuration alone, so it answers on a tree whose citations are still broken:

```text
api · grund.toml · v1

kinds and places
  FS    kind        docs/
  code  complement  every scanned file no other row claims

citations
  FS    should  FS (itself)
  code  —
```

`--format mermaid` and `--format dot` print the same facts as graph source. `grund config show` is for the effective TOML, every value filled in.

## Querying the Structure

Once the tree checks clean, read its structure with grund's queries rather than by parsing heading lines: `grund list --glob` takes a coordinate pattern — `*` inside one component, `**` any number of whole components, `{a,b}` each alternative — and `--total` counts what it selects, while `grund show --batch --glob` reads the same coordinates in one scan. Each answer's `.result.sections[]` lists the unit's subpoints as `path`, `title` and `depth`:

```bash
grund list --glob '<KIND>-*.<chapter>' --total
grund show --batch --glob '<KIND>-*.<chapter>' --toc --format json
```

The [Querying grund guide](https://github.com/agent-grounds/grund/blob/main/docs/user-facing/querying.md) has the rest: a point and its subtree with `<ID>{,.**}`, bodies in bulk with `show --batch --brief|--full`, `--glob` or `--all`, units by rule subject with `list --selector`, `refs --descendants`, who cites a set with `refs --glob` and what a point cites with `refs --from`, a checklist review that ends with `cover FILE --lines N-M`, and the whole citation graph from `cover --format json`, or the part a pattern names with `cover --glob`.

## Editor Setup

Optional, and only if the user wants editor integration (diagnostics, hover previews, go-to-definition, references, and the live `$$` → `§` transform). `grund init` does not write editor config; wiring an editor to `grund-lsp` is the user's one-time work.

- Confirm the CLI and server are installed: `grund --version` and `grund-lsp --version` (`cargo install grund-lsp` if missing).
- Point the editor's LSP client at `grund-lsp` for Markdown plus the languages in `[scan].extensions`. Full per-editor snippets are in the [user-facing LSP setup guide](https://github.com/agent-grounds/grund/blob/main/docs/user-facing/lsp.md).
- **Recommend placing the client config in the editor's user (global) settings, not a per-repo file.** Register `grund-lsp` once there to make the configuration available across projects; a per-repo config only registers it for that repo. User settings do not grant Workspace Trust. For VSCode / VSCodium, follow the [guide's trust prerequisite and troubleshooting](https://github.com/agent-grounds/grund/blob/main/docs/user-facing/lsp.md#vscode--vscodium) for the affected clients ([editor configuration specification](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-lsp.md#23-editor-configuration-one-time-per-editor)). Use a per-repo file only for a deliberate project-specific override.

## Clickable Citation Integrations

Optional, and only if the user wants a plain `§<ID>` citation to be clickable in their terminal or editor (`grund integrations`, [the integrations reference](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-integrations.md#fs-integrations-grund-prints-and-installs-its-rendering-layer-integrations)). These are one-time, user-side, and env-specific, so propose rather than apply silently:

- Detect what applies: `grund integrations --format json` names the terminal/editor found in the environment and, for each, the exact `--write` command.
- Show the user the change before touching disk: `grund integrations <client>` prints the config snippet and the `grund-open` resolver (or, for `vscode`, the unpacked extension) so they can read it first.
- Install only on confirmation: `grund integrations <client> --write` installs the client integration, records the user-local conversation preference, and synchronizes managed blocks into the global instruction files of the six file-backed agents (Codex, Claude, Gemini, GitHub Copilot, Zed, Pi — [user preference and global agent instructions](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-integrations.md#43-user-preference-and-global-agent-instructions)). It is idempotent; never write without the user's go-ahead.
- The installed default asks agents for plain local citations. Use `grund integrations --write --conversation link` as a preference-only user override when the TUI has no rendering support; no arbitrary client is installed. The editor choice (`GRUND_OPEN_CMD` or `EDITOR`) never belongs in shared repository text.
- One opinion *is* committable: a repository may set `[reference] conversation = "link"` in `grund.toml` ([the repository-opinion decision](https://github.com/agent-grounds/grund/blob/main/docs/decisions/functional/DF-repo-conversation-opinion.md#df-repo-conversation-opinion-repositories-may-commit-a-link-only-conversation-rendering-opinion)), and the generated entrypoint then teaches linked local citations to every cloner with zero per-user setup. The form follows the per-agent gate ([gated per agent, with `path` as the fallback](https://github.com/agent-grounds/grund/blob/main/docs/decisions/functional/DF-conversation-link-target.md#24-the-form-is-gated-per-agent-and-the-fallback-is-path)): `CLAUDE.md` teaches a Markdown link whose visible text is the citation and whose target is `file://<absolute path>#L<line>`, while `AGENTS.md` — the file Codex reads, where that form erases the citation — keeps the location as plain `path:line` text. A `CLAUDE.md` symlinked to `AGENTS.md` is one file and keeps the plain form; `grund init --claude` writes a real Claude entrypoint (`.claude/CLAUDE.md`, the path the symlink has not taken) instead. It is the fallback for machines that never stated a preference (fresh clones, cloud sessions, Cursor and Windsurf, whose only grund channel is the committed entrypoint); a machine whose recorded preference is `plain` keeps bare citations there, because its rendering layer already resolves them. Without the key, repository instructions carry only the fixed repository-web rule. Offer it when a team wants clickable conversation citations without asking each member to run `grund integrations`.

<!-- BEGIN chapter-rules -->
### Chapter rules

Write one controlled-English sentence as a rule declaration title and put the
reason in its non-empty body. Kind names and fixed words are case-sensitive;
every sentence ends in one `.`. `must` and `must not` produce errors. `should`
and `should not` produce suggestions, visible with `grund check --suggestions`
and never changing the exit status.

Subjects select local units only:

- `Each FS` selects every FS declaration.
- `FS-login` selects one declaration.
- `The requirements chapter of each FS` selects that named chapter in every FS.
- `FS-login.requirements` selects one exact named chapter.

Named chapter subjects select handles: `The goal chapter of each FS` reaches `## goal: Goal and hypothesis` by `goal` ([chapter subjects](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-rules.md#2-subject-selectors)). Presence rules compare direct chapter display names case-insensitively: `Each FS must have exactly one goal chapter.` counts zero for that heading and one for `## goal: Goal` ([presence matching](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-rules.md#31-chapter-presence)). The finding appends the expected display name and observed pairs such as `"goal: Goal and hypothesis"`, or `none` ([comparison context](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-rules.md#721-display-name-comparison-context)). Presence `NAME` must be one non-empty token: `Goal and hypothesis` is refused with `NAME forbids whitespace anywhere.` appended to the existing guidance; authored display titles may still contain spaces ([whitespace refusal](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-rules.md#351-presence-name-whitespace-refusal)).

Named-chapter subjects require `[id] named_sections = true`. Phase 1 has no
paths, files, folders, wildcards, subject namespaces, numbered-chapter subjects,
exceptions, definitions, derived terms, settings, or source-code symbols.

<!-- BEGIN chapter-rules-emptiness -->
A quantified chapter subject selects only the chapters that exist, and a
declaration of the kind that has none is reported rather than passed over, so a
chapter-scoped citation rule reaches every declaration of its kind.

`The requirements chapter of each FS must cite at least one REQ.` reports an FS
whose `requirements` chapter cites no REQ, and reports an FS with no
`requirements` chapter at all — the edit the rule could not see before:

```text
docs/fs/FS-login.md:1: error: FS-login has no requirements chapter, so RULE-requirements cannot reach it; add the chapter, or narrow the rule to the declarations that have one; this became an error in grund 0.16.0
```

Add the chapter or narrow the rule: those two are the whole answer. The error
fails the run like every other `must` finding; `--ignore unreached-declaration`
is the opt-out for a repository that wants the absence reported by nothing, and
a `should` rule reports it as a suggestion that never moves the exit status.

A chapter-presence rule beside the citation rule is not a third answer — it
raises its own `chapter-cardinality` and suppresses nothing, so one absent
chapter under both prints both lines. Write the two sentences together to state
the count as well as the citation:

```text
Each FS must have at least one requirements chapter.
The requirements chapter of each FS must cite at least one REQ.
```

The exact spelling reports the same absence as a different kind of finding.
`FS-login.requirements must cite at least one REQ.` names one unit, so deleting
that chapter makes the rule itself an `invalid-rule` finding — `literal subject
FS-login.requirements does not resolve` — located where the sentence is written
rather than at the declaration. Where both spellings stand over the same absent
chapter, both fire.

`grund list --selector FS.requirements` prints one row per declaration that has
the chapter, so the declarations it omits are the ones the rule now reports. A
subject that selects nothing at all prints nothing and exits 0.
<!-- END chapter-rules-emptiness -->

<!-- BEGIN chapter-rules-accepted -->
The complete accepted sentence forms, with representative findings, are:

- `Each FS must have at least one requirements chapter.` → `chapter-cardinality`.
- `Each FS must have at least 2 requirements chapters.` → `chapter-cardinality`.
- `FS-login should have at most 2 goals chapters.` → suggestion `chapter-cardinality`.
- `Each FS must have exactly one requirements chapter.` → `chapter-cardinality`.
- `Each FS should have exactly 2 review chapters.` → suggestion `chapter-cardinality`.
- `Each FS must cite at least one GOAL or REQ.` → zero matches reuse `missing-citation`.
- `The requirements chapter of each FS must cite at least one REQ.` → zero matches reuse `missing-citation`; a declaration with no such chapter, `unreached-declaration`.
- `Each FS should cite at least one GOAL.` → suggestion `suggested-citation`.
- `Each FS must cite at least 2 REQ.` → `citation-cardinality`, including at zero, because the floor is above one.
- `FS-login.requirements should cite at most 2 REQ.` → suggestion `citation-cardinality`.
- `FS-login.requirements must cite exactly one REQ.` → `citation-cardinality`.
- `AR-overview.system-overview must cite each AR at least once.` → one `citation-cardinality` per missed AR.
- `AR-overview.system-overview must cite each AR at least 2 times.` → one `citation-cardinality` per short-counted AR.
- `AR-overview.system-overview must cite each AR at most once.` → one `citation-cardinality` per repeated AR.
- `AR-overview.system-overview should cite each AR at most 2 times.` → suggestion `citation-cardinality`.
- `AR-overview.system-overview must cite each AR exactly once.` → one `citation-cardinality` per off-count AR.
- `AR-overview.system-overview should cite each AR exactly 2 times.` → suggestion `citation-cardinality`.
- `Each FS must be cited by at least one AR.` → `uncited-unit`.
- `Each FS must be cited by at least 2 AR.` → `uncited-unit`.
- `FS-login.requirements should be cited by at most 2 AR or GOAL.` → suggestion `uncited-unit`.
- `Each FS must be cited by exactly one AR.` → `uncited-unit`.
- `Each FS must not cite any AR.` → one site-anchored `forbidden-citation` per citation.
- `FS-login.requirements should not cite any AR or GOAL.` → one site-anchored `discouraged-citation` suggestion per citation.
<!-- END chapter-rules-accepted -->

Counts are positive decimal integers. Spell one as `one` where shown, and as
`once` for every per-target bound; numeric counts above one use plural
`chapters` or `times`. Chapter, outbound and inbound ceilings keep numeric one:
`at most 1 requirements chapter`, `cite at most 1 AR`, `be cited by at most 1 AR`.
Object kinds may be local
(`REQ`), pinned to a workspace member (`api/REQ`), or any member (`*/REQ`), and
`or` forms one normalized target set. A member nested inside another pins
itself with its whole alias path (`group/api/REQ` in member `group/api`), which
judges exactly like the bare kind. A namespaced object kind needs the workspace
in scope: a run without one says so at the rule's heading and still writes the block.

Per-target `at most once` allows zero or one physical citation to each target;
two sites citing the same target produce `citation-cardinality`, with
`requires at most 1` in the finding ([per-target counts](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-rules.md#33-per-target-coverage)).
Replace an existing per-target `at most 1 times` with `at most once`, then rerun
`grund init` to refresh rendered guidance. The numeric spelling is refused and
stops init before it writes ([migration decision](https://github.com/agent-grounds/grund/blob/main/docs/decisions/functional/DF-per-target-once-ceiling.md#compatibility-use-the-pre-10-allowance-not-a-verdict-correction)).

<!-- BEGIN chapter-rules-refused -->
Common refusals are intentional. A sentence is refused for the first stage it fails, and no later stage is read: the final `.`, the fixed word and the phase-1 subject, a conjunction, the modality, the rest of the subject, then the predicate. A configured kind that declares no IDs is refused as that, naming its home, rather than as an unknown kind. Until 0.19.0 the old reason stays in front of the new one, as in `unknown kind "e2e" — kind "e2e" declares no IDs — tests/e2e/ is not a citable home; this wording changes in grund 0.19.0`. Each names what failed and offers your sentence with only the refused part replaced, which is accepted when you paste it back; where grund cannot know what belongs in its place, it offers nothing, and `grund check --rule` lists the known kinds when what is missing is a kind:

- `Each FS may not cite any AR.` → `modality "may not" is not accepted; accepted form: Each FS must not cite any AR.`
- `Each FS must cite no AR.` → `"cite no" is not accepted; accepted form: Each FS must not cite any AR.`
- `Each FS must cite a GOAL.` → `quantifier "a" is ambiguous; accepted forms: "Each FS must cite at least one GOAL." or "Each FS must cite exactly one GOAL."`
- `Each FS must cite at least 1 GOAL.` → `numeric "at least 1" is not canonical; accepted form: Each FS must cite at least one GOAL.`
- `Each FS must cite exactly 1 GOAL.` → `numeric "exactly 1" is not canonical; accepted form: Each FS must cite exactly one GOAL.`
- `AR-overview.system-overview must cite each AR at most 1 times.` → `numeric "at most 1 times" is not canonical; accepted form: AR-overview.system-overview must cite each AR at most once.`
- `Each FS must cite at least one GOAL and must not cite any AR.` → `conjunctions are not accepted; accepted forms: "Each FS must cite at least one GOAL." and "Each FS must not cite any AR."`
- `Each FS must have exactly one  chapter.` → `chapter name must be a non-empty NAME with no surrounding whitespace; NAME forbids whitespace anywhere.`
- `Each FS must cite at least one GOAL` → `rule must end with "."; accepted form: Each FS must cite at least one GOAL.`
- `each FS must cite at least one GOAL.` → `fixed word "Each" is case-sensitive; accepted form: Each FS must cite at least one GOAL.`
- `Each file in vendor/ must cite at least one FS.` → `path subjects are not accepted in phase 1`, then `known kinds: GOAL, REQ, FS, AR, RULE`
- `Each */FS must cite at least one GOAL.` → `subject namespaces must be local in phase 1; accepted form: Each FS must cite at least one GOAL.`
- `FS-login.* must cite at least one REQ.` → `section-component wildcards are not accepted in phase 1; accepted form: FS-login must cite at least one REQ.`
- `Each chapter of each FS must cite at least one REQ.` → `chapter-quantified subjects are not accepted in phase 1; accepted form: Each FS must cite at least one REQ.`
- `FS-login.2 must cite at least one REQ.` → `numbered chapter subjects can detach when headings move; accepted form: FS-login must cite at least one REQ.`
- With named sections off, `FS-login.requirements must cite at least one REQ.` → `named chapter subjects require [id] named_sections = true; accepted form after enabling it: FS-login.requirements must cite at least one REQ.`
- `Each POLICY must cite at least one GOAL.` → `unknown kind "POLICY"`, then `known kinds: GOAL, REQ, FS, AR, RULE`
<!-- END chapter-rules-refused -->

Try a sentence without adding a declaration:

```bash
grund check --rule "Each FS should have exactly one security chapter." --suggestions
```

List the units a subject denotes:

```bash
grund list --selector FS.requirements
grund list --selector FS.requirements --format json
```

Configured syntax failures are located `invalid-rule` findings; `--rule`
syntax/vocabulary failures stop before scanning with exit 2. A syntactically
valid missing literal, such as `FS-missing`, is resolved after scanning and is
an `invalid-rule` finding with exit 1.

Findings sort bytewise by path, line, then message. Two identical declarations
collapse semantically and name sorted authorities, for example
`(RULE-a, RULE-b)`. If an existing `[citations]` entry says the same bare-kind
rule, the existing config finding wins byte-for-byte and no rule tail is added.
<!-- END chapter-rules -->
