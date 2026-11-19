# Guides and examples

The user guides for `grund`, each beside the runnable example that shows it at
work. This is the page `grund --help` links in its footer
([§FS-cli.2.2](../functional-spec/FS-cli.md#22-the-top-level-help-page)); a
command's own `--help` page links the guide and example that cover it
([§FS-cli.2.3](../functional-spec/FS-cli.md#23-a-subcommands-help-page)).

| Guide | Example |
|---|---|
| [Installation](installation.md) | — |
| [Your first grounded repository: init, a first declaration, a first citation](init.md) | [`examples/init-template-none`](../../examples/init-template-none/) |
| [The structure that gets cited: IDs, citations, sections](structure.md) | — |
| [Checking a repository](checking.md) | [`examples/authored-citations`](../../examples/authored-citations/) |
| [`grund init` flags and exit codes](setup.md) | [`examples/scheme-*`](../../examples/) |
| [The `grund.toml` reference](configuration.md), with its [JSON Schema](../../schemas/grund.schema.json) | [`examples/places`](../../examples/places/) |
| [Migrating a config to version 2](migrating.md) | [`examples/migrate`](../../examples/migrate/) |
| [Asking the binary what a key or a check code means](explain.md) | [`examples/config-explain`](../../examples/config-explain/) |
| [Python API](python-api.md) | [`examples/python-api`](../../examples/python-api/) |
| [Rust API](rust-api.md) | [`examples/rust-api`](../../examples/rust-api/) |
| [Citation directions](citation-directions.md) | [`examples/workspace-isolated-member`](../../examples/workspace-isolated-member/) |
| [Repository options](repository-options.md): [citers in a repository you cannot read](repository-options.md#citers-in-a-repository-you-cannot-read) | [`examples/downstream-citers`](../../examples/downstream-citers/) |
| [Seeing a repository's schema](schema.md) | [`examples/schema`](../../examples/schema/) |
| [Clickable citations](clickable-citations.md) | — |
| [Coordinate sizes](coordinate-sizes.md) | — |
| [Grund maintainer release runbook](distribution-runbook.md) (for installing, see [Installation](installation.md)) | — |
| [Git co-change evidence](cochange.md) | [`examples/cochange`](../../examples/cochange/) |
| [External facts](external-facts.md) | [`examples/external-tickets`](../../examples/external-tickets/) |
| [`grund init` repository shapes](init-repo-shapes.md) | [`examples/scheme-*`](../../examples/) |
| [Reviewing changes to declarations](reviewing.md) | — |
| [Editor support via LSP](lsp.md) | — |
| [Node Promise API](node-api.md) | [`examples/node-api`](../../examples/node-api/) |
| [Querying grund](querying.md) | [`examples/refs-unused`](../../examples/refs-unused/) |
| [Terminal feedback with watch](watch.md) | — |
| [Writing chapter rules](rules.md) | [`examples/rules`](../../examples/rules/) |
| [Sources: which files grund reads](sources.md) | [`examples/sources`](../../examples/sources/) |
| [Kind fields](fields.md) | [`examples/fields`](../../examples/fields/), [`examples/front-matter`](../../examples/front-matter/) |
| [First-class values](values.md) | [`examples/values`](../../examples/values/) |
| — | [`examples/workspace`](../../examples/workspace/) |

Each example is a self-contained mini-repository whose recorded output the test
suite reruns, so what it shows is what `grund` does; [`examples/`](../../examples/)
lists them all.
