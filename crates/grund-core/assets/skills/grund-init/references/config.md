## Recommendation Heuristics

| Setting | Recommendation from repo evidence |
| --- | --- |
| `--template` and `--scaffold` | With little or no `docs/` or `tests/` structure, use `grund init --scaffold`: the default `--template=software-development` lays canonical kinds and starter files. With meaningful docs or tests, omit `--scaffold` and add only missing grund files. For an existing taxonomy, use `grund init --template=none`, add the repo's version-2 `[schema.kinds.<KIND>]` tables to `grund.toml`, then run `grund init` again. `--template=none --scaffold` is refused because `none` has no starter files. |
| `[reference].strict` (version 1 only) | Version 2 has fixed marked-only recognition and no strict switch ([config-derived guidance](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-init.md#2381-what-the-config-fills-in)). For version 1, recommend the default `strict = true` for first adoption, especially with noisy ID-like tokens or deliberate citation hygiene. Use `strict = false` only for existing bare citations that need a compatibility window before `grund fmt --marker`. |
| `[rules.citations.grounding] must` (v2); `[reference].require_grounding` (v1) | Recommend leaving it off initially (no v2 `must` rung, v1 `false`); turn it on (v2 `must = "file"`, v1 `true`) only with broad spec-to-code citation coverage or an explicit request for strict co-change discipline. |
| `[schema] id_format` (v2); `[id] format` (v1) | Use `{kind}-{number}-{slug}` for fresh repos or stable IDs with readable names; `{kind}-{number}` for existing ADR/RFC-style numbered IDs; `{kind}-{slug}` for title/slug-based docs where numeric allocation would feel artificial. A kind override uses `[schema.kinds.<KIND>] id_format` in v2, `[[kinds]] format` in v1. |
| `[schema.kinds.<KIND>]` (v2); `[[kinds]]` (v1) | Start from defaults. Add custom kinds for existing categories such as `ADR`, `RFC`, `REQ`, `API`, `SEC`, or `OPS`; remove defaults only when they would clearly confuse the project. |
| Source scope | In v2, configure actual places with `[schema.kinds.<KIND>] folders`/`files`; the `code` complement covers unplaced files, and `[schema.sources] exclude` prunes generated paths. In v1, `[scan] include` lists actual directories, such as `["docs", "e2e", "src", "crates", "tests"]` for a Rust workspace. |
| `[fmt.cross_refs].anchor_format` (v1; v2 writes only `github` anchors) | Use `github` for GitHub or no evident renderer; `gitlab` for GitLab; `mkdocs` when `mkdocs.yml` exists; `pandoc` with evident Pandoc config/build scripts; `none` for file-only links without section anchors. |

## Config Questions

Ask these in order. The last of them, the citation directions (version 1's `[citations]`, version 2's `[rules.citations]`), is not here: `grund guide citations` prints it.

### Top-level

`grund_config_version`

Do not ask. A fresh setup gets `2` from `grund init`. An existing file keeps its version: a file without the key is version 1 and keeps loading unchanged, and moving it is `grund config migrate` (see Version 1 to version 2: `grund guide languages`). The questions below name each setting by its version-1 key; for a version-2 file, `grund config show` prints the table each one lives in.

`project_name`

Pros of explicit name: stable display name for agents/tools. Cons: one more metadata value to maintain if repo is renamed. Default: the target config's existing value, otherwise the target directory name.

### `[reference]`

`marker`

Default: `§`. Pros: visually distinct, avoids false positives. Cons: awkward to type without trigger/editor help. Alternatives: `@`, `#`, or `ref:` only if the team has strong conventions.

`trigger`

Default: `$$`. Pros: easy typing path to `§`. Cons: may conflict with math-heavy Markdown or template languages. Recommend changing only if `$$` is common in the repo.

`strict` (version 1 only)

Version 2 recognizes only marked citations, with no opt-out or strict switch ([config-derived guidance](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-init.md#2381-what-the-config-fills-in)). With `grund_config_version = 2` there is no `[reference].strict`: do not ask about it or write it. For version 1, the default is `true`: citations are intentional and explicit. Use `false` only as a compatibility window for existing bare citations; it also admits more accidental matches. Run `grund fmt --marker` before removing that override or setting strict back to `true`.

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

Default kinds: `GRUND`, `GOAL`, `FS`, `AR`, `DF`, `DA`, `e2e`, `integration`, `RM`. The two test kinds, `e2e` at `tests/e2e` and `integration` at `tests/integration`, are non-citable folder kinds whose files declare no IDs; `E2E` is configured, not a default. The config `grund init` writes also declares `REQ`, hard requirements the project must never break, in `docs/requirements/`. `REQ` has no built-in default, so a version-1 config that leaves `[[kinds]]` out has none. `FS` lives in `docs/functional-spec/`, one declaration per file, with no `grund.toml` and in the config `grund init` writes alike: `grund init --scaffold` writes `docs/functional-spec/README.md`, the `FS` index, and no `requirements.md`. Before grund 0.19.0 a repository with no `grund.toml` homed `FS` in `requirements.md` ([the decision](https://github.com/agent-grounds/grund/blob/main/docs/decisions/functional/DF-fs-home-lands-in-0-19.md)); to keep `FS` there, run `grund init` and write `files = ["requirements.md"]` on its `FS` row.

Ask whether to keep defaults, remove unused kinds, or add project-specific kinds. To start from none instead, `grund init --template=none` writes a version-2 file with no `[schema.kinds]` row, and each kind is then added as its own `[schema.kinds.<KIND>]` table.

Pros of defaults: matches grund docs and generated scaffold. Cons: some repos may not need all categories. Pros of custom kinds: adapts to existing taxonomy. Cons: replacing defaults means the full list must be copied; no merge.

For each kind ask: `kind`, `folder`, `title`.

A directory that holds agent-facing content rather than declarations — skills, runbooks, a prompt library, a test suite — is a kind with `citable = false`: it gets a home, a title and citation rules, and declares no IDs. Ask for one wherever the repo has such a directory and the answer to "should an agent be told this exists?" is yes.

Where the repo keeps tunable numbers (limits, rates, thresholds) that docs and code comments quote, offer a value kind: `form = "value"` in v2, `values = true` in v1, with a folder home such as `values/`. Each direct `.json`, `.yaml` or `.yml` child is a value source. Suggest YAML when a number needs a comment saying where it came from, and quote any component that must stay a string:

```yaml
# Fire code, section 4.2: occupancy per room.
CONST-room-limit: [12, people]

# Quoted, so it stays the string "1.10" rather than a number.
CONST-api-version: ['1.10']
```

A `§` citation inside a YAML comment is read by nothing, so cite provenance from Markdown. Until grund 0.22.0 every finding at a YAML source is a `yaml-value-source-deferred` warning.

### `[scan]`

`include`

Default: `["requirements.md", "docs", "e2e", "src"]`. Pros: focused, avoids scanning root clutter. Cons: misses specs/citations outside these dirs. Every configured version-1 `[[kinds]]` home is walked whether or not it is listed here, so this key names the *extra* roots. Base the recommendation on actual directories. Do not include paths that do not exist unless `--scaffold` will create them.

`exclude`

Default: `["target", "node_modules", ".git", "dist", "build", ".venv"]`. Pros: skips generated/vendor-heavy trees. Cons: can hide intentional generated docs if stored there. Usually keep defaults and add repo-specific build/cache dirs.

`extensions`

Default includes common Markdown and source extensions. Pros: polyglot coverage. Cons: scanning more extensions costs time and may surface noise. Recommend only extensions found in the repo plus Markdown, unless the repo is fresh. With a value kind, leave `yaml` and `yml` out: a listed one keeps a value home's YAML files as scanned text that declares nothing.

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
