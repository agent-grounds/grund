## Language And Repo Shape Examples

Use these examples to turn repo evidence into recommended `[scan]` settings for a version-1 file; a version-2 file writes the same evidence as `[schema.sources]`, after the examples. Include only directories that exist, unless `--scaffold` will create them.

### Rust

Evidence: `Cargo.toml`, `Cargo.lock`, `crates/`, `src/**/*.rs`. For a version-1 file:

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

Evidence: `package.json`, `pnpm-workspace.yaml`, `tsconfig.json`, `src/`, `apps/`, `packages/`. For a version-1 file:

```toml
[scan]
include = ["docs", "e2e", "src", "apps", "packages", "tests"]
extensions = ["md", "ts", "tsx", "js", "jsx"]
exclude = ["node_modules", "dist", "build", "coverage", ".next", ".turbo", ".git"]
comment_prefixes = ["//", "/*", "*"]
```

Pros: works for frontend apps and monorepos.
Cons: broad monorepos may need narrower package selection.

### Python

Evidence: `pyproject.toml`, `setup.py`, `requirements.txt`, `src/`, package dirs, `tests/`. For a version-1 file:

```toml
[scan]
include = ["docs", "e2e", "src", "tests"]
extensions = ["md", "py"]
exclude = [".venv", "__pycache__", ".pytest_cache", ".mypy_cache", "build", "dist", ".git"]
comment_prefixes = ["#"]
docstring_python = true
```

Pros: supports citations in comments and docstrings.
Cons: docstring scanning can surface intentional prose examples.

### Go

Evidence: `go.mod`, `cmd/`, `internal/`, `pkg/`, `*.go`. For a version-1 file:

```toml
[scan]
include = ["docs", "e2e", "cmd", "internal", "pkg", "tests"]
extensions = ["md", "go"]
exclude = ["vendor", "dist", "build", ".git"]
comment_prefixes = ["//", "/*", "*"]
```

Pros: matches common Go project layout.
Cons: single-package repos may only need `["docs", "src"]` or `["docs", "."]` if code is at root.

### Java / Kotlin / Gradle

Evidence: `pom.xml`, `build.gradle`, `settings.gradle`, `src/main`, `src/test`. For a version-1 file:

```toml
[scan]
include = ["docs", "e2e", "src"]
extensions = ["md", "java", "kt"]
exclude = ["target", "build", ".gradle", ".git"]
comment_prefixes = ["//", "/*", "*"]
```

Pros: covers Maven and Gradle conventions.
Cons: multi-module builds may need module directories added explicitly.

### C / C++

Evidence: `CMakeLists.txt`, `Makefile`, `src/`, `include/`, `lib/`, `tests/`. For a version-1 file:

```toml
[scan]
include = ["docs", "e2e", "src", "include", "lib", "tests"]
extensions = ["md", "c", "cpp", "h", "hpp"]
exclude = ["build", "dist", "out", ".git"]
comment_prefixes = ["//", "/*", "*"]
```

Pros: covers implementation and public headers.
Cons: vendored headers should be excluded if present.

### C# / .NET

Evidence: `*.csproj`, `*.sln`, `src/`, `test/`, `tests/`. For a version-1 file:

```toml
[scan]
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

Evidence: multiple manifests and top-level `apps/`, `packages/`, `services/`, `libs/`, `tools/`. For a version-1 file:

```toml
[scan]
include = ["docs", "e2e", "apps", "packages", "services", "libs", "tools", "tests"]
extensions = ["md", "rs", "go", "java", "kt", "ts", "tsx", "js", "py", "c", "cpp", "cs", "rb", "php"]
exclude = ["target", "node_modules", ".git", "dist", "build", ".venv", "coverage", ".next", ".turbo"]
comment_prefixes = ["//", "#", ";", "--", "*", "/*"]
docstring_python = true
```

Pros: broad coverage for adoption across teams.
Cons: can be noisy; recommend narrowing after the first `grund check`, then `grund check --full .` to see what the narrowing left out.

### Front Matter

Evidence: Markdown files that open with a `---` or `+++` block (Jekyll, Hugo, Zola, Pandoc, Docusaurus). A repository whose Markdown opens with YAML or TOML front matter can keep its relationships there as `§` citations: they count for the file's first declaration, a `#` comment in the block declares nothing, and `grund fmt` leaves the block as written.

When existing records keep facts that define their kind in YAML front matter — a MADR `status`, an OKF `type` — declare those facts as the kind's `front-matter` field and its keys, not as chapters that mirror them. Version 2 only; the block's top-level keys become the field's sections, `check` holds each key, and a `+++` (TOML) block is not read as a field yet ([fields guide](https://github.com/agent-grounds/grund/blob/main/docs/user-facing/fields.md#front-matter-is-a-field)):

```toml
[schema.kinds.ADR.fields.front-matter]
presence = "must"
closed = "must"

[schema.kinds.ADR.fields.front-matter.keys.status]
presence = "must"
one_of = ["proposed", "accepted", "rejected", "deprecated", "superseded"]

[schema.kinds.ADR.fields.front-matter.keys.date]
presence = "must"

[schema.kinds.ADR.fields.front-matter.keys.decision-makers]

[schema.kinds.ADR.fields.front-matter.keys.consulted]

[schema.kinds.ADR.fields.front-matter.keys.informed]
```

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

To move an existing version-1 `grund.toml` to version 2, never hand-translate it: run `grund config migrate --to 2` without `--write` first, which writes nothing, previews the candidate on stdout and the report on stderr when `--write` would pass, and otherwise ends with `would not write:` and the causes; make each refusal's repair as a version-1 edit, checked with `grund check` and committed on its own, and preview again; then add `--write`, and add `--drop-comments` only after reading the comments the preview lists, because it drops exactly those ([migrating guide](https://github.com/agent-grounds/grund/blob/main/docs/user-facing/migrating.md)).

Review each carried-comment warning at its named source and destination: migration retains the comment but translated the setting's value vocabulary, so any listed values may need updating. Check it even when the prose seems neutral; `--drop-comments` retains carried comments and their warnings ([carried-comment guidance](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-config-migrate.md#411-check-carried-comments-when-the-value-vocabulary-changes)).

After a successful write, stdout is empty and stderr gives a compact report: the outcome, dropped-comment ranges, notices and carried-comment warnings, member commands, Project map note and undo/re-preview advice, omitting inapplicable items. To get the candidate on stdout and the complete mapping/default/gate report again, use `grund config migrate --to 2 --write --full`; on a preview only listed comments refuse, `--full` shows the candidate too, and it changes no other preview or refused write ([migration flags](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-config-migrate.md#13-the-other-flags)).

The final stderr note reminds you that existing `AGENTS.md` managed instructions may still describe version 1. Run `grund init` in the migrated project to refresh them: migration writes only its config and leaves those instructions unchanged. The reminder is qualified and appears even when `AGENTS.md` is absent; it does not mean a file was inspected or found stale. Previews, refused or failed writes and already-version-2 runs omit it ([after-write guidance](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-config-migrate.md#65-what-it-says-after-the-write)).

When a run prints `grund.toml is version 1; migrate it with grund config migrate --to 2`, preview that command first, then apply it with `--write` only once the preview shows no `differs:`; do not silence the warning with `--ignore config-version-1`.
