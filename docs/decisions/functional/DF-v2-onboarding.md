# DF-v2-onboarding: `grund init` writes a minimal version-2 file, and `grund config show` does the teaching

**Status:** Accepted
**Date:** 2026-10-15

## 1. Context

Until this decision `grund init` wrote a version-1 `grund.toml` that spelled out every default beside a comment, 131 lines of them, on the argument that the generated file was a teaching surface ([§DF-config-file-location.2.3](DF-config-file-location.md#23-grund-init-writes-the-bare-grundtoml)). Version 2 fixes its defaults at the version ([§FS-config-v2.defaults](../../functional-spec/FS-config-v2.md#defaults-the-v2-epoch)), and the published design record had already ruled that "`init` writes only what differs from a default, and `config show` does the teaching" ([§DISC-core-concerns.7.4](../../discussions/proposals/2026-09-30-core-concerns.md#74-what-the-format-leaves-out)). agent-grounds/grund#462 asked for that onboarding; the owner approved its proposal on 2026-10-15 (comment 6183022223) for the `init` and `show` parts, and moved the caution on version-1 configs to agent-grounds/grund#756, which waits for grund to migrate its own config first. This record serves [§GOAL-zero-config](../../goals.md#goal-zero-config-works-on-any-conformant-tree): a file that holds only the difference is that goal's sentence in practice.

## 2. Decision

### 2.1 The starter holds only what differs from a default

A new project's file is version 2 and holds the envelope, its rows with their places, and their titles, because version 2 has no built-in kinds and no default titles; nothing else ([§FS-init.2.4.3](../../functional-spec/FS-init.md#243-every-written-key-differs-from-its-default)). A user reading it reads only their own project's choices, and removing any line of it changes what grund does. Spelling out every default was rejected: in a file of 131 lines a user cannot tell their own choices from grund's, and a default written into the file is pinned there, so the file stops saying which settings the project actually chose.

The starter is written fresh rather than produced by migrating the old one, which `grund config migrate` refuses as it stands ([§DF-config-migrate.3](DF-config-migrate.md#3-consequences)). It takes version 2's slug-only ID format, because writing the numbered format back in would be an override in a file meant to hold none.

### 2.2 `config show` is the teaching surface

What the old file taught — every setting, at its value — `grund config show` prints for any config, in the file's own version, in a fixed order, and what it prints loads back to the same meaning ([§FS-config.4.2](../../functional-spec/FS-config.md#42-grund-config-show-path)). A version-1 file shows as version 1 with version-1 defaults, never reinterpreted as version 2 ([§FS-config.4.2.5](../../functional-spec/FS-config.md#425-a-v1-display-never-takes-a-v2-default)). The file's one comment points there.

## 3. Consequences

A new repository's first file is about twenty lines rather than 131, and its first IDs read `FS-login` rather than `FS-001-login`. Its scan widens from v1's `requirements.md`, `docs`, `e2e` and `src` to the whole config root less `.gitignore`, which is the version-2 default; a project narrows it with `[schema.kinds.code] folders`. A repository initialised with this grund needs a grund that reads version 2, so its CI pin must be this release or later. No existing file changes: `init` never overwrites a config, and every version-1 file keeps loading with the meaning it had.

## release-note: Release note

- [§FS-init.2.4.3](../../functional-spec/FS-init.md#243-every-written-key-differs-from-its-default): `grund init` in a repository with no config now writes a minimal version-2 `grund.toml`: the rows, their places and titles, and nothing grund would assume anyway. New repositories therefore get slug-only IDs (`FS-login`, not `FS-001-login`); run `grund config show` to see every effective setting. Existing version-1 files keep loading unchanged, and `init` never rewrites one.
