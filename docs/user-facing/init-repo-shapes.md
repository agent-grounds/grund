# Additional `grund init` repository-shape examples

These supplementary examples turn repository evidence into `[scan]` settings
without making the shipped init skill's primary workflow harder to scan. They
apply the generated-config contract in [§FS-init.2.4](../functional-spec/FS-init.md#24-generated-grundtoml); include only directories
that exist in the repository under analysis.

The shapes below are version-1 `[scan]` tables. A version-2 `grund.toml`
selects the same files by language instead: name the languages the evidence
shows, map any suffix the v2 epoch lacks, and write the generated paths as
globs. For the Ruby / Rails shape:

```toml
[schema.sources]
languages = ["markdown", "ruby"]
exclude = ["vendor/", "tmp/", "log/", "coverage/"]
```

`.git` needs no entry: the hidden-file rule already skips it. The
[sources guide](sources.md) has the language table and the suffix and
definition tables.

## Ruby / Rails

Evidence: `Gemfile`, `app/`, `lib/`, `spec/`, `test/`.

```toml
include = ["requirements.md", "docs", "e2e", "app", "lib", "spec", "test"]
extensions = ["md", "rb"]
exclude = ["vendor", "tmp", "log", "coverage", ".git"]
comment_prefixes = ["#"]
```

Pros: covers Rails and library conventions.
Cons: Rails apps may need to skip generated schema or fixture-heavy paths.

## PHP

Evidence: `composer.json`, `src/`, `app/`, `tests/`.

```toml
include = ["requirements.md", "docs", "e2e", "src", "app", "tests"]
extensions = ["md", "php"]
exclude = ["vendor", "var", "cache", "build", ".git"]
comment_prefixes = ["//", "#", "/*", "*"]
```

Pros: works for Composer apps and frameworks.
Cons: framework cache dirs vary; inspect before finalizing.

## Swift

Evidence: `Package.swift`, `Sources/`, `Tests/`.

```toml
include = ["requirements.md", "docs", "e2e", "Sources", "Tests"]
extensions = ["md", "swift"]
exclude = [".build", "DerivedData", ".git"]
comment_prefixes = ["//", "/*", "*"]
```

Pros: matches Swift Package Manager.
Cons: Xcode projects may have different app/test directories.

## Scala

Evidence: `build.sbt`, `src/main/scala`, `src/test/scala`.

```toml
include = ["requirements.md", "docs", "e2e", "src"]
extensions = ["md", "scala"]
exclude = ["target", "project/target", ".bloop", ".metals", ".git"]
comment_prefixes = ["//", "/*", "*"]
```

Pros: covers sbt source layout.
Cons: generated sources may need explicit exclusion.

## SQL / Data Projects

Evidence: `db/`, `migrations/`, `models/`, `*.sql`, `dbt_project.yml`.

```toml
include = ["requirements.md", "docs", "e2e", "db", "migrations", "models", "tests"]
extensions = ["md", "sql", "py", "yml", "yaml"]
exclude = ["target", "logs", ".venv", ".git"]
comment_prefixes = ["--", "#"]
```

Pros: covers dbt and migration-heavy repos.
Cons: YAML comments are line-only; generated dbt target dirs must stay excluded.
