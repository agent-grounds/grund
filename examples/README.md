# Examples

Self-contained mini-repos demonstrating each ID scheme `grund` supports.
Each subfolder is shaped like an `tests/e2e/cases/<name>/` directory — `repo/`
holds the fixture, and `expected.exit`/`expected.stdout`/`expected.stderr`
record the contract — so each example doubles as a regression fixture.
The e2e test runner also runs `grund <repo>` against every example on `cargo
test`, so the snippets below cannot drift from what the tool actually does.

Examples are maintained as user-facing walkthroughs for canonical `grund`
workflows, per [FS-examples](../docs/functional-spec/FS-examples.md).
Each example README should name the use-case it teaches, show the command to
run, explain the expected output, and call out the practical trade-offs.
Runnable examples share the same golden-output runner as `tests/e2e/cases/`; the
examples tree should not grow a parallel test harness.

## ID schemes

| Folder                                                       | `[id] format`             | Example IDs                |
|--------------------------------------------------------------|---------------------------|----------------------------|
| [`scheme-numbered-slug/`](scheme-numbered-slug/)             | `{kind}-{number}-{slug}`  | `FS-001-login`             |
| [`scheme-numbered/`](scheme-numbered/)                       | `{kind}-{number}`         | `RFC-001`, `FS-002`        |
| [`scheme-slug/`](scheme-slug/)                               | `{kind}-{slug}`           | `FS-login`, `AR-event-bus` |

Each subfolder's `README.md` lists the trade-offs for that scheme. The
[structure guide](../docs/user-facing/structure.md)
summarizes when to reach for each.

## Workflows

| Folder                                                       | Use-case                                                 |
|--------------------------------------------------------------|----------------------------------------------------------|
| [`quickstart/`](quickstart/) | Declare a login requirement with a named section, cite it from code, retrieve it, then break and repair the citation ([§FS-examples.2](../docs/functional-spec/FS-examples.md#2-canonical-use-cases)) |
| [`refs-unused/`](refs-unused/) | Asking `grund refs` about a declaration nothing cites: the empty answer's note names `grund list --unused`, which lists every such declaration in one call ([§FS-refs.2.1.1](../docs/functional-spec/FS-refs.md#211-a-declared-id-nothing-cites-points-at-list---unused)) |
| [`review-checklist/`](review-checklist/) | Reviewing a checklist with coordinate globs: count its points, read their bodies in one scan, then name the points a changed line range touched ([§FS-coordinate-globs](../docs/functional-spec/FS-coordinate-globs.md#fs-coordinate-globs-a-coordinate-glob-selects-catalog-coordinates-by-pattern)) |
| [`init-template-none/`](init-template-none/) | Setting up a repository that declares its own kinds with `grund init --template=none` ([§FS-init.1.6.2](../docs/functional-spec/FS-init.md#162-none)) |
| [`workspace/`](workspace/)                                   | Cross-project citation in a monorepo ([§FS-workspace](../docs/functional-spec/FS-workspace.md#fs-workspace-grund-validates-cross-project-citations-in-a-workspace)) |
| [`workspace-optional-member/`](workspace-optional-member/) | A checkout without one of the workspace's `optional_members`: `check` warns what it did not cover, and `init --check` keeps each block's member list and passes ([§FS-init.2.3.4.15.8](../docs/functional-spec/FS-init.md#234158-while-an-optional-member-is-absent)) |
| [`workspace-mixed-versions/`](workspace-mixed-versions/) | A version-2 root with a version-1 member: one caution, naming the member to migrate ([§FS-check.checks.config-version-1](../docs/functional-spec/FS-check.md#checksconfig-version-1-a-config-file-read-as-version-1)) |
| [`rules/`](rules/)                                           | Controlled-English rules over declarations, named chapters, and citations, including per-target `at most once` and its numeric-one refusal ([§FS-rules](../docs/functional-spec/FS-rules.md#fs-rules-grounded-declarations-state-and-enforce-chapter-rules)) |
| [`config-explain/`](config-explain/) | Asking the installed grund what a key or a check code means, in the project's config version: the index, a key, a code, and a table from the other version ([§FS-config-explain](../docs/functional-spec/FS-config-explain.md#fs-config-explain-grund-config-explain-answers-from-the-installed-binary-and-in-the-version-the-projects-config-is-read-as-what-any-key-accepts-and-means-and-what-any-check-code-means-and-what-turns-it-on)) |
| [`schema/`](schema/)                                         | What a repository is held to, read from its configuration alone: kinds, places, fields, grounding and citation directions ([§FS-schema](../docs/functional-spec/FS-schema.md#fs-schema-grund-schema-shows-what-a-repository-is-held-to-read-from-its-configuration-alone)) |
| [`rfc/`](rfc/) | An Internet-Draft in RFC style: sections cited by the number xml2rfc prints, the required Security and IANA Considerations as fields, published RFCs as snapshots of their section maps, and code comments that cite the sections they implement ([§FS-examples.2](../docs/functional-spec/FS-examples.md#2-canonical-use-cases), [§FS-fields](../docs/functional-spec/FS-fields.md#fs-fields-a-v2-kind-states-the-shape-of-its-declarations-and-check-holds-every-declaration-to-it), [§FS-config.3.3](../docs/functional-spec/FS-config.md#33-section-paths--arbitrary-nesting-depth)) |
| [`fields/`](fields/)                                         | A version-2 kind's chapters declared as fields: order, titles, scalar values and closed kinds ([§FS-fields](../docs/functional-spec/FS-fields.md#fs-fields-a-v2-kind-states-the-shape-of-its-declarations-and-check-holds-every-declaration-to-it)) |
| [`sources/`](sources/)                                       | A version-2 project's languages: a remapped suffix, a language of its own, an excluded glob, and declarations only in doc comments ([§FS-config-v2.schema.sources](../docs/functional-spec/FS-config-v2.md#schemasources-schemasources)) |
| [`places/`](places/)                                         | A version-2 kind in two folders, a listed file and a nested folder deciding which row owns a file, and `code` with places drawn as the complement ([§FS-config-v2.schema.places.ownership](../docs/functional-spec/FS-config-v2.md#schemaplacesownership-a-listed-file-then-the-deepest-folder)) |
| [`migrate/`](migrate/) | Moving a version-1 config to version 2: the preview that refuses and names its repairs, then the candidate after them ([§FS-config-migrate](../docs/functional-spec/FS-config-migrate.md#fs-config-migrate-grund-config-migrate---to-2-shows-the-version-2-file-that-means-what-a-version-1-config-means-and-writes-it-only-when-asked-and-only-when-it-does)) |
| [`scoped-guidance/`](scoped-guidance/) | A version-2 project's guidance written into the `AGENTS.md` above each kind's places, with the root block pointing to it ([§FS-init.2.3.5.11](../docs/functional-spec/FS-init.md#23511-scoped-guidance-a-rows-guidance-lands-above-its-places)) |
| [`authored-citations/`](authored-citations/) | A citation in every link form its author writes, left as written by `fmt` and still checked, and a citation-only image in a kind's index, which is no entry ([§FS-citations.wrap.enclosing.scope](../docs/functional-spec/FS-citations.md#wrapenclosingscope-every-form-a-link-is-written-in)) |
| [`relative-sections/`](relative-sections/)                   | Relative section headings and the full path each resolves to ([§FS-declarations.section.relative](../docs/functional-spec/FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds)) |
| [`relative-sections-lock/`](relative-sections-lock/)         | The committed section lock catching a cited relative section it does not record ([§FS-section-lock](../docs/functional-spec/FS-section-lock.md#fs-section-lock-a-cited-relative-section-keeps-its-text-or-check-says-where-the-text-went)) |
| [`values/`](values/)                                         | Markdown/JSON value declarations and explicit consistency bindings ([§FS-values](../docs/functional-spec/FS-values.md#fs-values-opted-in-kinds-bind-authored-components-to-one-declared-value)) |
| [`external-tickets/`](external-tickets/)                     | Explicitly materialized external facts resolved from committed snapshots ([§FS-fetch](../docs/functional-spec/FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot)) |
| [`cochange/`](cochange/) | Opt-in Git/CI evidence of related spec and test edits ([§FS-cochange-recipe.examples](../docs/functional-spec/FS-cochange-recipe.md#examples-maintained-walkthrough-tests-and-opt-in-guidance)) |

The [Python API workflow](python-api/) embeds the same engine using a local
extension installation ([§FS-distribution.3.3.7](../docs/functional-spec/FS-distribution.md#337-local-source-and-typing-handoff)); its native acceptance
coverage is in the shared binding corpus.

## Run an example

The [Node API consumer](node-api/) is tested through a fresh packed-package
installation ([§FS-distribution.3.2.4](../docs/functional-spec/FS-distribution.md#324-acceptance-evidence));
follow its README and the [Node guide](../docs/user-facing/node-api.md).
Its consumer capture uses the existing json-report fixture.

From the repo root, with a built `grund` binary on `$PATH` (or invoked
via `cargo run --`):

```bash
grund examples/scheme-slug/repo
echo $?    # 0
```

A passing scheme prints `success` on stdout and exits 0.
