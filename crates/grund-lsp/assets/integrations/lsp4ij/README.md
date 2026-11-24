# grund LSP for LSP4IJ

Import this directory as a custom LSP4IJ template:

1. Install the LSP4IJ plugin if IntelliJ prompts for it.
2. Open **Settings | Languages & Frameworks | Language Servers**.
3. Choose **+ | New Language Server**.
4. Select **Import from custom template...**.
5. Choose this generated directory.
6. Create the server and apply it to the project.

To enable the live citation transform, return to **Settings | Languages &
Frameworks | Language Servers**, select the imported **grund LSP** server,
and open **Configuration | Client**. Enter the following JSON in its client
configuration, or merge it into existing settings. Preserve other client
settings and other entries within `format`:

```json
{"format":{"existingFormatterOverrideable":true}}
```

When IntelliJ has a formatter for a language, LSP4IJ otherwise withholds on-type
formatting requests from grund-lsp. This includes Java, and Markdown when a
formatter is installed. The setting lets those requests reach the live transform,
so the default `$$` becomes `§` while typing a citation
([§FS-lsp.2.4](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-lsp.md#24-installed-editor-integrations)).

Open a file containing a grund citation to verify that diagnostics, hover, and
navigation are available. Regenerate this directory after changing
`[scan].extensions` in `grund.toml`.
