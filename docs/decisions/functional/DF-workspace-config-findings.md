# DF-workspace-config-findings: At a workspace root, `config validate` names every config's findings that `check` names, and `config show` names only its own

**Date:** 2026-11-03

## status: Accepted

## context: Context

The three config findings — the redundant pair ([§FS-check.4.3](../../functional-spec/FS-check.md#43-redundant-config-pair)), the deprecated location ([§FS-check.4.11](../../functional-spec/FS-check.md#411-config-read-from-the-deprecated-agents-location)) and the version 1 caution ([§FS-check.checks.config-version-1](../../functional-spec/FS-check.md#checksconfig-version-1-a-config-file-read-as-version-1)) — are carried by `check`, `config validate` and `config show` ([§FS-check.4.11.2](../../functional-spec/FS-check.md#4112-the-same-three-surfaces-as-fs-check43-and-no-more)), once per config ([§FS-check.4.11.3](../../functional-spec/FS-check.md#4113-once-per-config-not-once-per-scope)). At a workspace root `check` named the root's config and each project the workspace expansion returns, while `config validate` and `config show` named the root's alone and said nothing of the members, so the answer to "is this workspace's configuration all right?" depended on which command was asked ([issue #1134](https://github.com/agent-grounds/grund/issues/1134)). `config validate` already loaded every member to report the first that does not load ([§FS-config.4.1.1](../../functional-spec/FS-config.md#411-at-a-workspace-root)); `config show` loads only the config it prints ([§FS-config.4.2](../../functional-spec/FS-config.md#42-grund-config-show-path)).

## decision: Decision

`config validate` at a workspace root names the findings of the root's config and of each project the workspace expansion returns — the set, the bytes, the order and the report base `check` uses there ([§FS-config.4.1.1](../../functional-spec/FS-config.md#411-at-a-workspace-root)). Its exit code and its first-error contract do not move: warnings leave it at `0`, and a failed expansion prints the error alone at `1`. `config show` keeps its bytes and is said to name only the findings of the config it prints, pointing to `config validate` for the workspace ([§FS-config.4.2](../../functional-spec/FS-config.md#42-grund-config-show-path)). `check` does not change. The Node `validateConfig` carries the member findings in `runCautions` and the Python `validate_config` in `warnings`, the fields that already carry the root's; Python's `run_cautions` carry no config finding and stay so.

### decision.rejected: Expanding the workspace under `show`

`show` prints one config, so its findings are that config's. Naming members' findings would also have to decide what a member that does not load does to a command that succeeds today, for a question `validate` already answers.

## consequences: Consequences

A workspace whose members draw config findings prints one `warning:` line per finding under `config validate` at its root where it printed nothing, or only the root's lines. No message changes: each line is the bytes `check` already prints at that root. A grouping block with `include_root = false` is not a project the expansion returns, so neither `check` nor `config validate` names its config.

## release-note: Release note

- [§FS-config.4.1.1](../../functional-spec/FS-config.md#411-at-a-workspace-root), [§FS-check.4.11.3](../../functional-spec/FS-check.md#4113-once-per-config-not-once-per-scope), [§DF-workspace-config-findings](DF-workspace-config-findings.md#df-workspace-config-findings-at-a-workspace-root-config-validate-names-every-configs-findings-that-check-names-and-config-show-names-only-its-own), [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered), [§FS-check.4.11.6](../../functional-spec/FS-check.md#4116-it-stands-in-place-of-the-success-line): **At a workspace root, `grund config validate` names the config findings of the root's config and each project the workspace expansion returns, as `grund check` does.** Such a project on version 1, read from `.agents/grund.toml`, or beside an `.agents/grund.toml` nothing reads now earns its `warning:` line on stderr, at its path from the root and in `check`'s order, where `config validate` printed only the root's; the Node `validateConfig` carries the same findings in `runCautions` and the Python `validate_config` in `warnings`. **What does not move:** every exit code, every message, `grund check`, `grund config show`'s output, and validating a member by its own path. `grund config --help` now says that `show` names only the config it prints. **Who this affects:** a script that expects `grund config validate` to print nothing on stderr at a workspace root whose members draw a config finding; the exit code it reads is unchanged. Closes [issue #1134](https://github.com/agent-grounds/grund/issues/1134).
