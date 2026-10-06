# Guides and examples

The user guides for `grund`, each beside the runnable example that shows it at
work. This is the page `grund --help` links in its footer
([§FS-cli.2.2](../functional-spec/FS-cli.md#22-the-top-level-help-page)); a
command's own `--help` page links the guide and example that cover it
([§FS-cli.2.3](../functional-spec/FS-cli.md#23-a-subcommands-help-page)).

| Guide | Example |
|---|---|
| [Python API](python-api.md) | [`examples/python-api`](../../examples/python-api/) |
| [Citation directions](citation-directions.md) | — |
| [Clickable citations](clickable-citations.md) | — |
| [Coordinate sizes](coordinate-sizes.md) | — |
| [Git co-change evidence](cochange.md) | [`examples/cochange`](../../examples/cochange/) |
| [External facts](external-facts.md) | [`examples/external-tickets`](../../examples/external-tickets/) |
| [`grund init` repository shapes](init-repo-shapes.md) | [`examples/scheme-*`](../../examples/) |
| [Editor support via LSP](lsp.md) | — |
| [Terminal feedback with watch](watch.md) | — |
| [Writing chapter rules](rules.md) | [`examples/rules`](../../examples/rules/) |
| [First-class values](values.md) | [`examples/values`](../../examples/values/) |
| — | [`examples/workspace`](../../examples/workspace/) |

Each example is a self-contained mini-repository whose recorded output the test
suite reruns, so what it shows is what `grund` does; [`examples/`](../../examples/)
lists them all.
