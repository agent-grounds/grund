# Workspace with mixed config versions: the caution names the member to migrate

A two-project workspace whose root config is already on version 2 and whose
one member, `packages/beta`, is still on version 1. `grund check` at the root
passes, and says once which config is still on version 1 and the command that
moves it ([§FS-check.checks.config-version-1](../../docs/functional-spec/FS-check.md#checksconfig-version-1-a-config-file-read-as-version-1)). `grund config validate` at the same
root says the same line, because it names the config findings of the root's
config and each project the workspace expansion returns, as `check` does ([§FS-config.4.1.1](../../docs/functional-spec/FS-config.md#411-at-a-workspace-root)). The example pins the
two runs, `1-check` and `2-config-validate`, each over its own copy of the
same `repo/`.

```toml
# 1-check/repo/grund.toml
grund_config_version = 2
project_name = "root"

[workspace]
members = ["packages/beta"]
```

```toml
# 1-check/repo/packages/beta/grund.toml
grund_config_version = 1

[scan]
include = ["docs"]
```

## What this teaches

- **One line per config still on version 1, and none for version 2.** The
  root is on version 2 and earns nothing; the member earns exactly one line,
  however many scopes the run walks
  ([§FS-check.checks.config-version-1.1](../../docs/functional-spec/FS-check.md#checksconfig-version-11-order-beside-the-other-config-findings)).
- **The line ends with the directory to pass.** `packages/beta` is not the
  report's base, so the command names it; a single project's line ends at
  `--to 2`.
- **It is a caution, never an error.** The exit code stays `0`, and a
  version 1 config keeps loading for as long as grund reads version 1
  ([§FS-config.5.3](../../docs/functional-spec/FS-config.md#53-a-version-1-config-is-cautioned-and-never-retired)).
  The line stands where `success` would have stood.
- **Validating the config at the root names the member too.** `grund config
  validate` loads every member's config to validate it, and names the findings of
  the root's config and each project the workspace expansion returns in `check`'s words; asking it about the root alone never hides the
  member still on version 1. `grund config show` prints only the root's config
  and names only that config's findings
  ([§FS-config.4.2](../../docs/functional-spec/FS-config.md#42-grund-config-show-path)).

## Run it

```bash
grund check examples/workspace-mixed-versions/1-check/repo
# warning: packages/beta/grund.toml is version 1; migrate it with grund config migrate --to 2 packages/beta
echo $?    # 0

grund config validate examples/workspace-mixed-versions/2-config-validate/repo
# warning: packages/beta/grund.toml is version 1; migrate it with grund config migrate --to 2 packages/beta
echo $?    # 0
```

Run the command it names from the repository root, preview first, and add
`--write` once the preview shows no `differs:`; the next `grund check` prints
`success`. The [migration guide](../../docs/user-facing/migrating.md) walks
through the preview.
