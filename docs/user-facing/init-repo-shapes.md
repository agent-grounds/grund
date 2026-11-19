# Additional `grund init` repository-shape examples

These supplementary examples turn repository evidence into source settings
without making the shipped init skill's primary workflow harder to scan. They
apply the generated-config contract in [§FS-init.2.4](../functional-spec/FS-init.md#24-generated-grundtoml); include only directories
that exist in the repository under analysis.

The shapes below use the version-2 source languages. Name the languages the
evidence shows, map any suffix the epoch lacks, and write generated paths as
globs. For the Ruby / Rails shape:

```toml
[schema.sources]
languages = ["markdown", "ruby"]
exclude = ["vendor/", "tmp/", "log/", "coverage/"]
```

`.git` needs no entry: the hidden-file rule already skips it. The
[sources guide](sources.md) has the language table and the suffix and
definition tables.

List source directories as places on the `code` row, and list each directory
when a kind spans several:

```toml
[schema.kinds.AR]
folders = ["app/models/docs", "app/services/docs"]

[schema.kinds.code]
folders = ["app", "lib", "script"]
```

A file in nested places belongs to the deepest
([§FS-config-v2.schema.places.ownership](../functional-spec/FS-config-v2.md#schemaplacesownership-a-listed-file-then-the-deepest-folder)).

## Ruby / Rails

Evidence: `Gemfile`, `app/`, `lib/`, `spec/`, `test/`.

```toml
[schema.sources]
languages = ["markdown", "ruby"]
exclude = ["vendor/", "tmp/", "log/", "coverage/"]

[schema.kinds.code]
folders = ["app", "lib", "spec", "test"]
```

Pros: covers Rails and library conventions.
Cons: Rails apps may need to skip generated schema or fixture-heavy paths.

## PHP

Evidence: `composer.json`, `src/`, `app/`, `tests/`.

```toml
[schema.sources]
languages = ["markdown", "php"]
exclude = ["vendor/", "var/", "cache/", "build/"]

[schema.kinds.code]
folders = ["src", "app", "tests"]
```

Pros: works for Composer apps and frameworks.
Cons: framework cache dirs vary; inspect before finalizing.

## Swift

Evidence: `Package.swift`, `Sources/`, `Tests/`.

```toml
[schema.sources]
languages = ["markdown", "swift"]
exclude = [".build/", "DerivedData/"]

[schema.kinds.code]
folders = ["Sources", "Tests"]
```

Pros: matches Swift Package Manager.
Cons: Xcode projects may have different app/test directories.

## Scala

Evidence: `build.sbt`, `src/main/scala`, `src/test/scala`.

```toml
[schema.sources]
languages = ["markdown", "scala"]
exclude = ["target/", ".bloop/", ".metals/"]

[schema.kinds.code]
folders = ["src"]
```

Pros: covers sbt source layout; `target` names a directory at every depth, so it
excludes `project/target` too.
Cons: generated sources may need explicit exclusion.

## SQL / Data Projects

Evidence: `db/`, `migrations/`, `models/`, `*.sql`, `dbt_project.yml`.

```toml
[schema.sources]
languages = ["markdown", "sql", "python"]
exclude = ["target/", "logs/", ".venv/"]

[schema.kinds.code]
folders = ["db", "migrations", "models", "tests"]
```

Pros: covers dbt and migration-heavy repos.
Cons: YAML comments are line-only; generated dbt target dirs must stay excluded.
