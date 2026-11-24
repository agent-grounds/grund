# Setting up a repository

`grund init` writes the two files a grounded repository starts from: `grund.toml`,
which declares the shape of what the project knows, and the managed block in
the agent entrypoint `grund init` selects, which tells agents how to keep it
([§FS-init](../functional-spec/FS-init.md#fs-init-grund-bootstraps-a-new-grund-conformant-repo)). It updates detected entrypoints, and writes
`AGENTS.md` only when no entrypoint is found.

```bash
grund init                  # updates detected entrypoints, or writes AGENTS.md
grund init --scaffold       # also lays the starter files: docs/ and tests/
grund init --template=none  # a grund.toml that declares no kinds, for a repo with its own
grund init --check          # no writes; exits 1 for rule findings or pending paths
```

Use an entrypoint flag to create or update that agent's file even when another
entrypoint exists:

| Flag | File |
|---|---|
| `--agents-md` | `AGENTS.md` |
| `--claude` | `CLAUDE.md` or `.claude/CLAUDE.md` |
| `--gemini` | `GEMINI.md` |
| `--pi` | `.pi/AGENTS.md` |
| `--copilot` | `.github/copilot-instructions.md` |
| `--cursor` | `.cursor/rules/grund.mdc` or `.cursorrules` |
| `--windsurf` | `.windsurfrules` |
| `--zed` | `.rules` |

For example, `grund init --agents-md --claude` writes both canonical entrypoints.

`init` is non-interactive and idempotent: re-running never errors on existing files. It lays a template ([§FS-init.1.6](../functional-spec/FS-init.md#16-templates)): `software-development`, the default, with the canonical kinds, or `none`, whose `grund.toml` declares no kind, for a repository that declares its own. Either way the file is a minimal version-2 `grund.toml` holding only what differs from a default ([§FS-init.2.4.3](../functional-spec/FS-init.md#243-every-written-key-differs-from-its-default)); `grund config show` prints what applies, `grund config explain` every setting ([§FS-config.4.2](../functional-spec/FS-config.md#42-grund-config-show-path)). Bare `init` writes only the config and the agent entrypoint, never a file under `docs/` or `tests/` ([§FS-init.3.6](../functional-spec/FS-init.md#36-bare-init-writes-the-config-and-the-entrypoints-and-nothing-in-a-home)); `--scaffold` adds the template's starter files (its old spelling, `--docs`, is an unknown flag since 0.19.0). The [`grund init` templates guide](init.md) walks through each run. With `--scaffold`, instructional ID shapes follow the repository's ID format (slug-only, `FS-login`, in the version-2 starter) and any illustrated kind's override ([§FS-config.3.2](../functional-spec/FS-config.md#32-id--id-grammar)). For an existing repo with specs, map those homes as `[schema.kinds.<KIND>]` tables (see [`--template=none`](init.md#--templatenone-a-repository-with-its-own-kinds)), start from `grund init --template=none` and add them, or run `grund guide setup` for the packaged adoption workflow and decision table ([§DF-skill-init-existing-specs](../decisions/functional/DF-skill-init-existing-specs.md#df-skill-init-existing-specs-grund-init-adopts-existing-specs-before-scaffolding)). It prints the core of that guide, whose Contents names each reference section `--section NAME` prints, and `--full` prints the core and every section ([§FS-init.5.5](../functional-spec/FS-init.md#55-the-core-and-its-sections)). It also checks *where* it was pointed before writing anything: a target no `.git`, `.hg`, `.jj`, or `.svn` marker covers is refused unless you pass `--no-vcs` — use it to scaffold a directory before `git init` — and the home directory and the machine-global agent instruction files are refused outright. `--check` is the `--dry-run` preview taken as a verdict — same report, nothing written, exit `1` when a file is still pending — so a hook can fail on any file `init` would still write. See [`FS-init`](../functional-spec/FS-init.md) for the full state table.

Rule findings independently earn `1` in ordinary, `--check` and `--dry-run`
runs, even after writing or finding all paths current. Located rule errors
are on stdout; path events are on stderr. Pending events such as `would-write`,
`would-append` and `would-update` can coexist with a finding; a finding alone
does not establish staleness. `exists` or `updated` describes only that path.
A member's unverifiable alias-pinned rule still renders as authored and earns
`1`, so the exit alone cannot be its staleness gate. Run `grund check` from
the workspace root to judge that rule; genuinely invalid rules withhold writes.
Operational failures earn `2` before either verdict. With neither failure,
`--dry-run` keeps `0` even with pending paths. See
[init exits and reports](init.md#exit-codes-and-reports)
([§FS-init.4.1](../functional-spec/FS-init.md#41-what-earns-the-1),
[§FS-init.4](../functional-spec/FS-init.md#4-exit-codes)).

The generated project name comes from `--name` when supplied, then from the target's existing `project_name`, and otherwise from the target directory name. This keeps the canonical `AGENTS.md` heading stable when `grund init --force` regenerates it; pass `--name` only when that run should override the configured identity ([§FS-init.2.3.8](../functional-spec/FS-init.md#238-substituted-content)).

For the `init` layouts of other ecosystems — Rails, PHP, Swift, Scala — see the
[repository-shape examples](init-repo-shapes.md).
