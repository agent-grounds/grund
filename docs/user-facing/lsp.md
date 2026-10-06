# Editor Support via LSP

`grund-lsp` is the optional editor server for `grund`. It provides citation completion, diagnostics, hover previews, usage counts on declaration titles, go-to-definition, references, document links, and live `$$` to `§` formatting from the same engine as the CLI ([§FS-lsp](../functional-spec/FS-lsp.md#fs-lsp-grund-ships-an-optional-lsp-server)).

## Install

Install the CLI first:

```bash
cargo install grund
```

Then install the LSP server separately:

```bash
cargo install grund-lsp
grund-lsp --version
```

When testing from this repository before a release, install the workspace crate instead:

```bash
cargo install --path crates/grund-lsp
grund-lsp --version
```

`grund-lsp` speaks LSP over stdio. Configure your editor to launch `grund-lsp` from the workspace root; there is no daemon, socket, or long-running service outside the editor process ([§FS-lsp.2.2](../functional-spec/FS-lsp.md#22-lifecycle)).

Use the same file types you scan in `grund.toml`. Markdown is the usual minimum; add Rust, Python, Go, JavaScript, TypeScript, or any other source languages in your `[scan] extensions`.

## Write a citation

Completion is available in the source version of `grund-lsp`; use the workspace
install above until a release includes it. In a Markdown note in this repository,
type the first line, request completion, and select `FS-lsp`:

```text
See §FS-ls
```

The item shows “grund ships an optional LSP server” and
`docs/functional-spec/FS-lsp.md`. Accepting it leaves:

```text
See §FS-lsp
```

Starting with `$$F` works too; you can also request choices immediately after
`$$` or `§`. Acceptance replaces the entire active token, including an old suffix
after the cursor, with one canonical citation. Suggestions match case-sensitive
ID prefixes, list the exact match first, and then follow configured kind order
and ID. They use your document's member config. A complete loaded alias plus
`/` selects that project's declarations; unqualified IDs stay in your own
project ([§FS-lsp.1.6.2](../functional-spec/FS-lsp.md#162-candidates-resolution-and-ordering)).

Completion follows the formatter's protected contexts: no choices in inline
code, link destinations, fences, declaration headings, source strings,
`grund:fmt off` regions, excluded or unscanned files, or outward file symlinks.
Source comments and Python docstrings are eligible. A nonempty typing trigger
works even with an empty marker; bare IDs never start completion
([§FS-lsp.1.6.1](../functional-spec/FS-lsp.md#161-eligibility-exclusions-and-triggering)).

The server advertises the final character of each initially loaded project's
marker and trigger, including multi-character introducers. Continued typing may
need the client's usual manual completion command. After changing introducer
characters or adding workspace folders, manual requests use current config;
restart the server to update automatic trigger characters.

Acceptance keys belong to your client ([§FS-lsp.2.3](../functional-spec/FS-lsp.md#23-editor-configuration-one-time-per-editor)).
If completion and on-type formatting are pending for the same buffer, applying
either edit makes the competing response obsolete. The client must discard it,
notify the change, and request again; applying both old responses can damage the
token. Completion lists request refresh on further typing. The server supplies
plain text edits and cannot intercept Tab or enforce client response handling
([§FS-lsp.1.6.4](../functional-spec/FS-lsp.md#164-snapshot-and-live-transform-interaction)).

## VSCode / VSCodium

Install a generic LSP client extension and configure it to launch `grund-lsp` for Markdown plus the source file types in your `[scan] extensions`. Use one that is available on your editor's marketplace: VSCodium installs from Open VSX, where "Simple LSP Client" (`wdomitrz.simple-lsp-client`) is published — the older `zsol.vscode-glspc` is only on the Microsoft Marketplace and cannot be installed in VSCodium.

With "Simple LSP Client", add this to your settings (workspace `.vscode/settings.json` or user settings):

```json
{
  "simpleLspClient.servers": {
    "grund-lsp": {
      "cmd": ["${userHome}/.cargo/bin/grund-lsp"],
      "filetypes": [
        "markdown",
        "rust",
        "python",
        "go",
        "javascript",
        "typescript"
      ]
    }
  }
}
```

The `cmd` is the path to the installed binary; `${userHome}/.cargo/bin/grund-lsp` is where `cargo install` places it. If `grund-lsp` is already on the editor's `PATH`, `["grund-lsp"]` works too. The `filetypes` are VS Code language IDs and must match the languages you scan. To get the live `$$` → `§` transform, also enable `"editor.formatOnType": true` for those languages.

**Prefer user (global) settings over a per-repo `.vscode/settings.json`.** Put the `simpleLspClient.servers` block above in your user settings once and `grund-lsp` runs in every project you open. A per-repo `.vscode/settings.json` only wires the server for that one repo — open any repo without it and VSCode silently falls back to its built-in Markdown behavior: Ctrl-click underlines a single hyphen-delimited word instead of the whole `§<ID>` token, and find-references misses the `grund` citation sites. Use a workspace `.vscode/settings.json` only for a deliberate per-project override.

A first-party VSCode extension is intentionally not shipped ([§FS-non-goals](../functional-spec/FS-non-goals.md#fs-non-goals-what-grund-will-deliberately-not-do)).

Request suggestions with Ctrl-Space and accept the selected item with Tab.
For Tab completion even when the menu is closed, optionally set
`"editor.tabCompletion": "on"`. Automatic suggestions in comments may also need
`"editor.quickSuggestions": { "comments": true }`; manual requests remain
available. See the [official IntelliSense guide](https://code.visualstudio.com/docs/editing/intellisense).

## IntelliJ family

The published `grund-lsp` 0.13.1 does not include the `integrations` subcommand;
until the next release, use the source-install command documented above.
Install LSP4IJ, then generate an importable template from the project root. The
generator discovers that project's `grund.toml`, snapshots its effective
`[scan].extensions`, and records the absolute path of the installed server
([§FS-lsp.2.4](../functional-spec/FS-lsp.md#24-installed-editor-integrations)):

```bash
cd /path/to/project
grund-lsp integrations                 # list the templates in this installation
grund-lsp integrations lsp4ij          # preview JSON and import instructions
grund-lsp integrations lsp4ij --write .grund-lsp-lsp4ij
```

Preview writes nothing. The write creates `.grund-lsp-lsp4ij/template.json` and
`.grund-lsp-lsp4ij/README.md`; an identical repeat is harmless. If that
directory has been modified, has a missing file, or contains anything extra,
the command preserves it and asks you to move or remove the whole directory.
Move it aside if you need its contents, or remove it, then run the command again
to deliberately regenerate from the current config.

Import the result explicitly:

1. Open **Settings | Languages & Frameworks | Language Servers**.
2. Choose **+ | New Language Server**.
3. Select **Import from custom template...**.
4. Choose the generated `.grund-lsp-lsp4ij` directory.
5. Create the server and apply it to the project.

The imported template launches the installed `grund-lsp` from the IntelliJ
project root and maps every effective scan extension. It does not install the
plugin or edit IntelliJ-owned configuration. If `[scan].extensions` changes,
regenerate and import the directory again.

Open a file containing a `§` citation and verify hover, navigation, and
diagnostics as described in [Check the wiring](#check-the-wiring). A first-party
JetBrains plugin is intentionally not shipped ([§FS-non-goals](../functional-spec/FS-non-goals.md#fs-non-goals-what-grund-will-deliberately-not-do)).

## Vim / Neovim

Use the built-in LSP client from your Neovim config:

```lua
vim.api.nvim_create_autocmd({ "BufReadPost", "BufNewFile" }, {
  pattern = { "*.md", "*.rs", "*.py", "*.go", "*.js", "*.ts" },
  callback = function(args)
    vim.lsp.start({
      name = "grund-lsp",
      cmd = { "grund-lsp" },
      root_dir = vim.fs.root(args.buf, { "grund.toml", ".agents/grund.toml", "AGENTS.md", ".git" }),
    })
  end,
})
```

If you use `nvim-lspconfig`, keep your usual language servers and add `grund-lsp` as a separate client for the same buffers; it should not replace `rust_analyzer`, `pyright`, `gopls`, `ts_ls`, or other language-specific servers.

For Vim, use an LSP client plugin that can launch a stdio server and point it at `grund-lsp` for Markdown plus the source file types in your `[scan] extensions`.

With Neovim's built-in completion, use Ctrl-X Ctrl-O to request and Ctrl-Y to
accept a selected item. On versions supporting `vim.lsp.completion.enable`,
enable automatic LSP completion from your `LspAttach` callback:

```lua
vim.api.nvim_create_autocmd("LspAttach", {
  callback = function(args)
    local client = vim.lsp.get_client_by_id(args.data.client_id)
    if client and client.name == "grund-lsp" then
      vim.lsp.completion.enable(true, client.id, args.buf, { autotrigger = true })
    end
  end,
})
```

To accept a selected popup item with Tab, add this mapping once:

```lua
vim.keymap.set("i", "<Tab>", function()
  return vim.fn.pumvisible() == 1 and "<C-y>" or "<Tab>"
end, { expr = true })
```

Select an item with Ctrl-N/Ctrl-P first. If you use a completion plugin, configure
its confirmation binding instead. These follow Neovim's
[LSP completion documentation](https://neovim.io/doc/user/lsp/#lsp-completion).

## Emacs

With `eglot`, register `grund-lsp` for Markdown and scanned source modes:

```elisp
(add-to-list 'eglot-server-programs
             '((markdown-mode rust-mode python-mode go-mode js-mode typescript-mode)
               . ("grund-lsp")))
```

Then run `M-x eglot` in a project buffer, or enable your normal project hook.

With `lsp-mode`, add a client registration:

```elisp
(with-eval-after-load 'lsp-mode
  (lsp-register-client
   (make-lsp-client
    :new-connection (lsp-stdio-connection "grund-lsp")
    :activation-fn (lsp-activate-on "markdown" "rust" "python" "go" "javascript" "typescript")
    :server-id 'grund-lsp)))
```

Start it with `M-x lsp` in a project buffer.

Both clients provide `completion-at-point`: invoke it with C-M-i (often M-Tab)
and choose a candidate using your completion UI. To let Tab try completion when
the line is already indented, use:

```elisp
(setq tab-always-indent 'complete)
```

This requests completion rather than forcing popup acceptance; Company/Corfu
users keep their existing selection and acceptance bindings. See the official
[Emacs symbol completion](https://www.gnu.org/software/emacs/manual/html_node/emacs/Symbol-Completion.html),
[Eglot features](https://www.gnu.org/software/emacs/manual/html_node/eglot/Eglot-Features.html),
and [lsp-mode completion settings](https://emacs-lsp.github.io/lsp-mode/page/settings/completion/).
The Tab option is described in [Emacs indentation convenience](https://www.gnu.org/software/emacs/manual/html_node/emacs/Indent-Convenience.html).

## Helix

Add the server to `languages.toml`:

```toml
[language-server.grund-lsp]
command = "grund-lsp"
```

Attach it to Markdown:

```toml
[[language]]
name = "markdown"
language-servers = ["grund-lsp"]
```

Attach it to scanned source languages too. For example, if Rust files are scanned:

```toml
[[language]]
name = "rust"
language-servers = ["rust-analyzer", "grund-lsp"]
```

Request with Ctrl-X, cycle with Tab/Shift-Tab, and accept with Enter. Helix's
documented completion menu cannot be remapped, so this setup uses its standard
acceptance key. See the [official keymap](https://docs.helix-editor.com/keymap.html#completion-menu).

## Zed

Add a local language server entry to `settings.json`:

```json
{
  "lsp": {
    "grund-lsp": {
      "binary": {
        "path": "grund-lsp"
      }
    }
  },
  "languages": {
    "Markdown": {
      "language_servers": ["grund-lsp"]
    },
    "Rust": {
      "language_servers": ["rust-analyzer", "grund-lsp"]
    }
  }
}
```

Add the same `grund-lsp` entry to each scanned source language you want checked while editing.

Request with Ctrl-Space and accept the selected completion with Tab; no extra
binding is needed with Zed's defaults. See the official
[completion guide](https://zed.dev/docs/completions) and
[Tab interaction with edit predictions](https://zed.dev/docs/ai/edit-prediction#default-key-bindings).

## Sublime Text

Install the Sublime `LSP` package, then add a client configuration for `grund-lsp`:

```json
{
  "clients": {
    "grund-lsp": {
      "enabled": true,
      "command": ["grund-lsp"],
      "selector": "text.html.markdown | source.rust | source.python | source.go | source.js | source.ts"
    }
  }
}
```

Adjust the selector to match the syntaxes you scan in `grund.toml`.

## Check the wiring

Open a file containing a resolving citation such as `§FS-check`.

Stored section citations use the same full-ID form. Consider this example input
inside an `FS-check` declaration body with section 2.1 but no section 9.9:

```text
See §2.1 and §9.9.
```

Both live local citations receive canonical-form errors naming their full
replacements, alongside the independent missing-section diagnostic:

```text
local section citation §2.1; write §FS-check.2.1 — unchecked in grund 0.13.1, an error in 0.14.0; run `grund fmt --write`
local section citation §9.9; write §FS-check.9.9 — unchecked in grund 0.13.1, an error in 0.14.0
missing section FS-check.9.9
```

Definition, references, and highlights follow the existing section
2.1 edge. The section 9.9 citation also receives an independent missing-section
diagnostic and has no navigation target. A local path
outside a declaration is diagnosed without a navigation target, because the
server never guesses an owner. Both forms end by naming the two releases the verdict
moved between, and only a site the formatter would write is offered the command
([§FS-check.3.24.1](../functional-spec/FS-check.md#3241-the-release-attribution-and-where-the-command-clause-is-withheld)). Run `grund fmt --write` for safe owned sites and
review protected or unresolved sites manually. Escaping either token as `<§>2.1`
or `<§>9.9` makes it an inert illustration with no citation diagnostic or navigation
([§FS-lsp.1.1](../functional-spec/FS-lsp.md#11-diagnostics), [§FS-lsp.1.3](../functional-spec/FS-lsp.md#13-go-to-definition)).

The live typing transform remains `$$<ID>` to a full citation. `$$2` is not a
local-section typing feature and no LSP quick-fix is added; canonicalization is
the formatter's bulk migration path ([§FS-fmt.2.4](../functional-spec/FS-fmt.md#24-shorthand-to-canonical)).

- Hover should show the same body as `grund FS-check --toc` ([§FS-lsp.1.2](../functional-spec/FS-lsp.md#12-hover-preview)).
- Hover a whole-ID title — the `# FS-check: …` heading, the same declaration written inline in a doc-comment, or the stub that points at it — and the popup reads `` `FS-check: …` — cited at 12 sites across 5 files ``: the same sites `grund refs FS-check` lists, counted. An uncited title reads `not cited`, and `grund refs FS-check --total` prints that same clause in a terminal ([§FS-lsp.1.2](../functional-spec/FS-lsp.md#12-hover-preview), [§FS-refs.3.4](../functional-spec/FS-refs.md#34---total)). Compare from the root the editor opened — in a workspace that is `grund refs <alias>/FS-check` from the workspace root and `grund refs FS-check` from inside the member — or the two are counting different trees.
- Hover a numeric or opted-in named section heading and the count is that section **and everything under it** — the same set the heading's own references return, and the same set `grund refs FS-check.3.2 --descendants` prints from the terminal, counted by `--total`. A bare `--section` is still the narrow question, keeping only citations whose coordinate is exactly the one asked for; which of the two a heading counts is stated in [§FS-lsp.1.2](../functional-spec/FS-lsp.md#12-hover-preview).
- Go-to-definition should jump to the declaration ([§FS-lsp.1.3](../functional-spec/FS-lsp.md#13-go-to-definition)).
- Find references from a declaration title should list citation sites ([§FS-lsp.1.3.1](../functional-spec/FS-lsp.md#131-citations-from-declarations)).
- Clickable citation links target the declaration line; editors that ignore file URI `#L<n>` fragments should still use go-to-definition for the exact jump ([§FS-lsp.1.3.2](../functional-spec/FS-lsp.md#132-document-links)).
- Typing `$$FS-check` should rewrite the trigger to `§FS-check` ([§FS-lsp.1.4](../functional-spec/FS-lsp.md#14-live-trigger-transform)).
