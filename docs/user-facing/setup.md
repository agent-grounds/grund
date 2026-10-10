# Setting up a repository

`grund init` writes the two files a grounded repository starts from: `grund.toml`,
which declares the shape of what the project knows, and the managed block in
`AGENTS.md`, which tells every agent how to keep it ([§FS-init](../functional-spec/FS-init.md#fs-init-grund-bootstraps-a-new-grund-conformant-repo)).

```bash
grund init           # writes AGENTS.md and grund.toml in the cwd
grund init --docs    # also scaffolds docs/ and tests/ trees
grund init --check   # writes nothing; exits 1 if anything is still pending
```

`init` is non-interactive and idempotent: re-running never errors on existing files. With `--docs`, instructional ID shapes follow the repository `[id].format` and any illustrated kind's `[[kinds]].format` override ([§FS-config.3.2](../functional-spec/FS-config.md#32-id--id-grammar)). For an existing repo with specs, map those homes in `[[kinds]]` before `grund init`, or run `grund agent-setup-instructions` for the packaged adoption workflow and decision table ([§DF-skill-init-existing-specs](../decisions/functional/DF-skill-init-existing-specs.md#df-skill-init-existing-specs-grund-init-adopts-existing-specs-before-scaffolding)). It also checks *where* it was pointed before writing anything: a target no `.git`, `.hg`, `.jj`, or `.svn` marker covers is refused unless you pass `--no-vcs` — use it to scaffold a directory before `git init` — and the home directory and the machine-global agent instruction files are refused outright. `--check` is the `--dry-run` preview taken as a verdict — same report, nothing written, exit `1` when a file is still pending — so a hook can fail on a managed block that drifted in its text while its version heading stayed current, which `grund check` does not see. See [`FS-init`](../functional-spec/FS-init.md) for the full state table.

The generated project name comes from `--name` when supplied, then from the target's existing `project_name`, and otherwise from the target directory name. This keeps the canonical `AGENTS.md` heading stable when `grund init --force` regenerates it; pass `--name` only when that run should override the configured identity ([§FS-init.2.3.8](../functional-spec/FS-init.md#238-substituted-content)).

For the `init` layouts of other ecosystems — Rails, PHP, Swift, Scala — see the
[repository-shape examples](init-repo-shapes.md).
