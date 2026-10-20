## Editor Setup

Optional, and only if the user wants editor integration (diagnostics, hover previews, go-to-definition, references, and the live `$$` → `§` transform). `grund init` does not write editor config; wiring an editor to `grund-lsp` is the user's one-time work.

- Confirm the CLI and server are installed: `grund --version` and `grund-lsp --version` (`cargo install grund-lsp` if missing).
- Point the editor's LSP client at `grund-lsp` for Markdown plus the languages in `[scan].extensions`. Full per-editor snippets are in the [user-facing LSP setup guide](https://github.com/agent-grounds/grund/blob/main/docs/user-facing/lsp.md).
- **Recommend placing the client config in the editor's user (global) settings, not a per-repo file.** Register `grund-lsp` once there to make the configuration available across projects; a per-repo config only registers it for that repo. User settings do not grant Workspace Trust. For VSCode / VSCodium, follow the [guide's trust prerequisite and troubleshooting](https://github.com/agent-grounds/grund/blob/main/docs/user-facing/lsp.md#vscode--vscodium) for the affected clients ([editor configuration specification](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-lsp.md#23-editor-configuration-one-time-per-editor)). Use a per-repo file only for a deliberate project-specific override.
