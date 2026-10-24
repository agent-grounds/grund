---
name: grund-init
description: Use when bootstrapping or adopting grund in a repository, especially when the user wants an interactive guided setup for grund init, grund.toml, AGENTS.md, docs scaffolding, citation format, scan scope, output format, or Markdown link settings.
---

# grund init

Guide the user through `grund` adoption. `grund init` itself is non-interactive, so this skill acts as the interactive wrapper: inspect the repository, recommend suitable settings, ask the user to confirm or override every option, write `grund.toml`, run `grund init`, then validate.

## Fresh repository

A repository is fresh when it has no `grund.toml`, at its root or under `.agents/`, and no specs, decision records or tests to adopt. Say you are taking the defaults, and once the user agrees, run these three commands in order:

```bash
grund init --scaffold --name <name> --agents-md   # add --claude, --gemini, … for each agent in use
grund init --check
grund check
```

Success is all three exiting `0`. On a fresh scaffold the last one also warns `nothing recognized` until the first declaration exists: that is expected, and nothing to fix. A fresh repository takes every default after that one confirmation, so that is the whole setup. Any repository that is not fresh is an adoption, and its per-option interview is the workflow `grund guide setup` prints.

## Contents

This core, the section `setup`, holds Fresh repository, Contents, Workflow, Repo Analysis First, Asking Style, Init Questions, Existing Specs Adoption, Validation and Maintaining Declarations. The rest of the guide is ten reference sections, each printed on request by `grund guide <name>`; where this skill is installed as a directory, the same text is `references/<name>.md`.

- `grund guide config` (`references/config.md`): how to recommend each setting, then the config questions.
- `grund guide citations` (`references/citations.md`): the citation-directions question, version 1's `[citations]` and version 2's `[rules.citations]`, and its reference.
- `grund guide languages` (`references/languages.md`): language and repo-shape examples, version-2 sources, and moving from version 1.
- `grund guide members` (`references/members.md`): splitting off a member.
- `grund guide scoped-guidance` (`references/scoped-guidance.md`): where a version-2 config writes each row's guidance.
- `grund guide schema` (`references/schema.md`): seeing the schema with `grund schema`.
- `grund guide queries` (`references/queries.md`): querying the structure.
- `grund guide editor` (`references/editor.md`): wiring an editor to `grund-lsp`.
- `grund guide integrations` (`references/integrations.md`): clickable-citation integrations.
- `grund guide rules` (`references/rules.md`): writing chapter rules.

`grund guide --full` prints this core and then every section, in this order.

## Workflow

1. Inspect the target repo before asking questions. Find existing specs, artifact types, roadmaps, changelogs, decisions, plans, tests, and agent instruction files before recommending anything.
2. Present a short "detected repo shape" summary and recommended setup.
3. If existing specs or spec-like artifacts are present, show the canonical `grund` artifact types beside the detected project-specific sections/tags/document classes, then ask which artifact model to adopt before writing config or refactoring docs.
4. Fetch the config questions before asking them. They are not in this core: run `grund guide config`, which holds how to recommend each setting and every question but the citation directions (version 1's `[citations]`, version 2's `[rules.citations]`), and `grund guide citations`, which holds that one (`references/config.md` and `references/citations.md` where this skill is installed as a directory). Then ask each remaining setup/config question they hold. For every question, include the recommended value, repo evidence, pros, cons, and when to choose something else.
5. Write `grund.toml` from the analysis before running `grund init`, so generated guidance reflects the repository's actual grammar, marker, strict mode, kinds, artifact folders, and scan scope. On a fresh setup with no `grund.toml`, `grund init` writes the version-2 starter itself: a short file holding only what differs from a default — the project name, each kind's row and places, and each kind's title. Run it first, then add only the settings the user chose to change. `grund config show` prints every effective setting, defaults included, in the file's own version; show it to the user rather than writing defaults out, and copy a line from it into `grund.toml` to change that setting. `grund config explain` says what any key or check code means in that same version; ask it, not a page that may describe the other version or another release. For the existing-structure model, `grund init --template=none` lays a version-2 `grund.toml` that declares no kinds: add one `[schema.kinds.<KIND>]` table per kind of the repo's own, with the `folders` or `files` its declarations live in, then run `grund init` again so the entrypoints teach them.
6. Run `grund init [path] [--name NAME] [--force]`, adding `--scaffold` only when the repo is fresh or the user selected a canonical-layout migration that needs the template's starter files. Omit `--name` to reuse a target config's `project_name`, falling back to the target directory name when the config has none; pass it only to override it. Offer `--dry-run` to preview the changes, and `--check` — the same preview, writing nothing and exiting `1` when a file is still pending — as the pre-commit or CI gate for every file `init` would still write. `grund check` already reports a managed block whose text no longer matches the config; `--check` adds the files `check` does not judge, such as a member's workspace sections when the run is inside the member. `init` refuses a target that no `.git`, `.hg`, `.jj`, or `.svn` marker covers, in it or any ancestor: for a directory not yet under version control, either run the VCS's own init first or pass `--no-vcs`. It also refuses the home directory and the machine-global agent instruction files (`~/.claude/CLAUDE.md`, `~/.codex/AGENTS.md`, and the rest of that set) with no flag at all — those say the path is wrong, not that an option is missing, so re-point the run rather than trying to force it. Follow the `next:` block it prints: it names a kind the config declares and that kind's home. For a kind other than `FS`, create a missing home by hand: `--scaffold` lays the whole software-development starter, not that home alone, holding the home only where the starter does (a `REQ` row's folder), so it can miss it. Without `--scaffold`, no block means only that the home exists.
7. Run `grund config validate [path]` and `grund check [path]`.
8. Summarize generated files, validation results, existing specs/artifacts found, and any follow-up cleanup.
9. Optionally offer to wire the user's editor to the `grund-lsp` server (see Editor Setup: `grund guide editor`). Editor configuration is the user's one-time work, not something `grund init` writes — so only offer it, and prefer editor **user** settings over a per-repo config to make the configuration available across projects. User settings do not grant Workspace Trust; Editor Setup (`grund guide editor`) gives the VSCode / VSCodium prerequisite.

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
| Canonical core plus project-specific extras | Repos with useful existing ADRs/RFCs/requirements but no clear behavior-vs-architecture backbone. | Use `GRUND`, `GOAL`, `FS`, and `AR` as the grounding backbone, then add custom kinds for project-specific artifacts: `[schema.kinds.<KIND>]` in version 2, `[[kinds]]` in version 1. |
| Existing structure with citations | Mature repos with a strong existing taxonomy or high migration cost. | Start from `grund init --template=none`, whose config declares no kinds; preserve current sections/tags/document classes, configure the kinds and their places around them (`[schema.kinds.<KIND>]` in version 2; `[[kinds]]` and `[scan]` in version 1), and add `grund` citations/declarations without forcing canonical folders. |

The question should include a recommendation grounded in the inventory. Do not write `grund.toml`, run `grund init --scaffold`, move documents, rename headings, or add bulk citations until the user chooses the adoption model. Once selected, use the model to decide whether the setup is only config plus entrypoint refresh, or a broader docs refactor with a visible plan.

## Validation

After writing config, run:

```bash
grund config validate .
grund init .
grund check .
grund check --full .
```

The last line checks beyond configured source scope: `grund check .` uses the ordinary scope, while `grund check --full .` widens the walk to report unresolved references and section headings outside declaration bodies there ([the full-tree scope](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-check.md#13-the-full-tree-scope---full)). Version 1 scopes with `[scan] include`; version 2 uses configured places and the `code` complement. Source exclusions still apply: `[schema.sources] exclude` in v2, `[scan] exclude` in v1. The wider walk stops only at *declared* workspace members, not at any nested `grund.toml`, so a vendored or example project inside the tree is read under this project's grammar; exclude it with the loaded version's source key, or list it in `[workspace] members` if it is one of ours.

If a misplaced-declaration error names an unread nested config, repair ownership before editing declarations: add the directory to the enclosing config’s `[workspace] members`, or exclude it with that config version’s source key. A declaration heading promotion does not fix membership. Exclusions prune descendants, so remove configured scan roots at or beneath the directory too when needed ([placement context](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-declarations.md#checksmisplaced-declarationunlisted-project-explain-the-enclosing-homes-beneath-an-unlisted-project)).

If custom config affects `AGENTS.md`, ensure `grund.toml` exists before `grund init` so the generated managed block reflects the selected ID grammar, marker, strict mode, kinds, and existing artifact layout.

Read an unused warning's index-entry location alongside `grund refs <ID>` and
`grund refs <ID> --summary`: refs retains the citation and labels it as an index
entry, while check excludes it from use. In mixed summaries, read the annotated
lines (and columns) as the excluded sites only. Keep a useful unused declaration
or cite an actual dependency; do not invent a dependency just to silence the
warning. See the [checking guide](https://github.com/agent-grounds/grund/blob/main/docs/user-facing/checking.md#when-only-an-index-cites-a-declaration)
and the [runnable example](https://github.com/agent-grounds/grund/blob/main/examples/authored-citations/README.md#when-only-the-index-cites-a-record).

`grund refs` on an uncited ID notes `grund list --unused`: run that once, not `refs` per ID.

If a project needs its own grammar or independent check, follow
Splitting off a member (`grund guide members`), then validate from the workspace root.

## Maintaining Declarations

Before a declaration rename or file move, run `grund refs <ID>` to find affected
citations. Edit the declaration and affected authored citation IDs, then run
`grund fmt --write` from the config/project root or workspace root. When
cross-reference emission applies, fmt re-derives generated Markdown link
destinations and anchors; the root-scoped write also updates eligible section
locks. Run `grund check` and `grund fmt --check`, and commit the lock changes.
Keep ID-keyed lock names unchanged for file-only moves. Do not hand-edit locks or
expect fmt to replace authored IDs, repair arbitrary URLs, or silently approve a
protected section move; fix citations or leave a pointer at the old address first.
A link the author wrote around a citation is theirs in every Markdown form: an
inline link, one whose text runs onto the next line, a reference with its key, a
definition, an autolink and an HTML `<a>`. fmt leaves it and does not follow a
move for it, so repair such a destination by hand; the citation inside is still
checked. A citation-only image in a kind's index, `![§<ID>](x.png)`, is no
entry: write the link beside it, or the ID is reported missing
([authored links](https://github.com/agent-grounds/grund/blob/main/docs/user-facing/structure.md#linking-what-grund-does),
[example](https://github.com/agent-grounds/grund/blob/main/examples/authored-citations/README.md)).
Use the [maintenance guide](https://github.com/agent-grounds/grund/blob/main/docs/user-facing/structure.md#maintaining-declarations)
and its [copy-based rename example](https://github.com/agent-grounds/grund/blob/main/examples/relative-sections-lock/README.md).
