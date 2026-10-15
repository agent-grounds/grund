# DF-config-version-1-caution: A config file on version 1 is cautioned, once, in place of `success`, with no deadline

**Date:** 2026-10-15

## status: Accepted

## context: Context

`grund init` writes a version 2 config ([issue #462](https://github.com/agent-grounds/grund/issues/462)) and grund's own config is on version 2 ([issue #464](https://github.com/agent-grounds/grund/issues/464)), yet a repository that copied a version 1 file from an older one is never told that version 2 exists or how to move to it: a clean run prints `success` and says nothing about the format. Version 1 keeps its meaning permanently ([§FS-config.5.2](../../functional-spec/FS-config.md#52-every-older-version-keeps-its-meaning)), so the deprecation path can only nudge, never retire ([§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)).

## decision: Decision

A config file read as version 1 earns the warning `config-version-1` ([§FS-check.checks.config-version-1](../../functional-spec/FS-check.md#checksconfig-version-1-a-config-file-read-as-version-1), [§FS-config.5.3](../../functional-spec/FS-config.md#53-a-version-1-config-is-cautioned-and-never-retired)), shaped on every count as the `.agents/` location's ([§FS-check.4.11](../../functional-spec/FS-check.md#411-config-read-from-the-deprecated-agents-location)): a CLI-level `warning:` on `check`, `config validate` and `config show`, once per config, standing in place of `success`, with no release named in which it becomes an error. Its text is the line `config migrate` already prints for a version 1 member ([§FS-config-migrate.8](../../functional-spec/FS-config-migrate.md#8-workspaces)), and `config migrate`'s report gate drops it from both runs ([§FS-config-migrate.5.2](../../functional-spec/FS-config-migrate.md#52-the-report-gate)).

### decision.rejected: A `note:` that keeps `success`

A line that leaves `success` in place would be a third kind of run-level line [§FS-check.2.1.3](../../functional-spec/FS-check.md#213-the-success-line) does not have, invented for one finding. A warning that stands where `success` stood is what every other warning does, and `--ignore config-version-1` gives back the old bytes.

## consequences: Consequences

Every clean `grund check`, `grund config validate` and `grund config show` on a version 1 config prints the caution on stderr and no longer prints `success`; a clean `--format json` run on one is no longer silent on stderr. Exit codes do not move, nothing stops loading, and a project with no config file is untouched.

## release-note: Release note

- [§FS-check.checks.config-version-1](../../functional-spec/FS-check.md#checksconfig-version-1-a-config-file-read-as-version-1), [§FS-config.5.3](../../functional-spec/FS-config.md#53-a-version-1-config-is-cautioned-and-never-retired), [§DF-config-version-1-caution](DF-config-version-1-caution.md#df-config-version-1-caution-a-config-file-on-version-1-is-cautioned-once-in-place-of-success-with-no-deadline): **A config file on version 1 is cautioned.** A clean `grund check`, `grund config validate` or `grund config show` on a `grund.toml` that writes `grund_config_version = 1`, or no version, printed `success` and now prints `warning: grund.toml is version 1; migrate it with grund config migrate --to 2` on stderr instead, one line per version 1 config in a workspace, naming the member's directory; under `--format json` it is a stderr finding with code `config-version-1`. **What does not move:** every exit code, every other finding, a version 2 config and a project with no config file, which earn no line, and version 1's meaning, which the caution never retires. **Who this breaks:** a script that compares stdout to `success` or expects a clean JSON run to be silent on stderr on a version 1 config; pass `--ignore config-version-1`, or run `grund config migrate --to 2 --write`.
