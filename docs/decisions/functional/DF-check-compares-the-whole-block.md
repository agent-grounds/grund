# DF-check-compares-the-whole-block: `grund check` holds the whole managed block, the workspace sections at the outermost root of a full checkout

**Date:** 2026-10-22

Widens the drift check of [§FS-init.2.3.5.8](../../functional-spec/FS-init.md#2358-the-drift-check) from five generated sections to the whole block, as [§FS-check.3.5.4](../../functional-spec/FS-check.md#354-no-part-of-the-block-is-exempt-from-the-comparison) and [§FS-check.3.5.5](../../functional-spec/FS-check.md#355-the-workspace-sections-are-judged-at-the-outermost-root-of-a-full-checkout) specify. Serves [§GOAL-agent-grounding](../../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work): the block is what the next agent reads instead of `grund.toml`, and a gate that passes a block describing a config or a workspace the repository no longer has hands that agent the wrong rules. Reported as [grund#906](https://github.com/agent-grounds/grund/issues/906).

## status: Accepted

## 1. Context

`check` re-rendered and byte-compared only the block's five generated sections — `### Citation directions`, `### Clickable citations`, `### Declaration shapes`, `### Chapter rules` and `### Scoped guidance` — and everywhere else verified only the `(vN)` heading. `grund init --check` saw the rest. Agents and most CI jobs gate on `check` alone, so a green `check` read as "the block is current" when it was not. In an agent-training batch, 3 of 4 runs that split a package into a new workspace member, and 3 of 4 that moved a config to version 2, ended with `check` green while the root block still listed the old members, or still described version 1: the `strict` sentence, the declaration forms, the Project map. All four places are rendered from the config or the workspace, and none was compared.

## 2. Decision

### 2.1 An error, not a note

A stale block anywhere is an `agents-init` error ([§FS-check.3.5](../../functional-spec/FS-check.md#35-invalid-agent-entrypoint-init-block)), one per differing section, named by its heading ([§FS-check.3.5.3](../../functional-spec/FS-check.md#353-code-and-message)). The alternative was a `note:` line on a run that still exits 0. The issue's own data rules it out: one of the two runs shown #851's `run grund init` note read past it, because it was the last of about 108 lines. An exit code is the one signal a gate cannot read past.

### 2.2 The workspace sections are judged at the outermost root of a full checkout

`### Project namespaces` and `### Workspace members` render from every member's alias, root, description and `AGENTS.md`. A run at the outermost root has loaded all of that already, so comparing them there costs one file-existence probe per project. A run inside a member has not: comparing there would mean climbing to the outermost root and expanding the whole workspace on every run, measured on a release build at 418 ms with 50 members and 1.65 s with 200, against 12–15 ms for the member's `check` at any size. So only the outermost root's run compares them, and every other run leaves them out on both sides ([§FS-check.3.5.5](../../functional-spec/FS-check.md#355-the-workspace-sections-are-judged-at-the-outermost-root-of-a-full-checkout)).

A checkout with an absent optional member is one of the runs that leaves them out. It already knows the member is absent, because it prints `optional-member-absent`; comparing the sections would turn exactly that checkout red, and running `grund init` there would write blocks the full checkout then reports. The warning gains a clause saying the sections went uncompared, but only where the run verified a managed block ([§FS-check.4.9.1](../../functional-spec/FS-check.md#491-one-warning-per-absent-entry)): added every time it would describe blocks that are not there.

That is a blind spot, declared and bounded ([§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded)): the outermost root's `check` in a full checkout sees every block's workspace sections, and a project's own `grund init --check` sees its own.

### 2.3 The finding says where to run `grund init`

A root run reports other projects' blocks, and `grund init` at the root does not refresh them ([§FS-init.2.3.4.15.7](../../functional-spec/FS-init.md#234157-what-init-does-not-do)). The new workspace variant names the directory inside its text, and the existing variants that name a command append `; run it in <dir>` after their last byte ([§FS-errors.3.6.1](../../functional-spec/FS-errors.md#361-the-final-templates)), so a consumer matching their prefix still matches.

### 2.4 `migrate` refuses what it refused before

`config migrate`'s guidance gate keeps refusing a change to the block version or to the five generated sections, and passes the rest, which describes the config and is expected to move under a migration ([§FS-config-migrate.5.3](../../functional-spec/FS-config-migrate.md#53-guidance)). Its report gate ignores the `agents-init` findings that difference produces ([§FS-config-migrate.5.2](../../functional-spec/FS-config-migrate.md#52-the-report-gate)); without that every version-1 migration would be refused, because its citation scheme paragraph always changes.

### 2.5 Every change to the render moves the version

`check` now compares every line, so a change in grund to what the block renders for a given config moves the block version, including a wording fix to a substituted sentence such as the inline-style sentences, which moved none before ([§FS-init.2.3.7](../../functional-spec/FS-init.md#237-the-block-version), [§FS-inline-citation-style.5.6](../../functional-spec/FS-inline-citation-style.md#56-no-managed-block-version)). That is the price later template edits pay: each costs every repository one `grund init`. Bumps that land between two releases share one, so a repository pays at most once per release, and 0.17.0, 0.18.0 and 0.19.0 each already asked for that. A repository editing its own config or workspace needs no version at all: `check` reports the block and `grund init` refreshes it.

## 3. Consequences

- A tree whose block no longer matches its config or its workspace is red where it was green: edits to kinds, titles, the ID format, the marker, named sections or the inline-style keys; a migration; a member joining or leaving; a member's description; a member's first `grund init`. Adding one member can turn up to N+1 blocks red, each naming its directory.
- This change shares the unreleased v39–v42 pairs ([§FS-init.2.3.7.1](../../functional-spec/FS-init.md#2371-the-current-version)), so every block 0.19.0 wrote is already outdated in this release and one `grund init` re-renders it whole; from then on the wider comparison turns nothing red by itself ([§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)).
- `migrate`'s Project map note changes its ending rather than growing, because kept as it was it would state the opposite of what `check` does. That is a message, not a verdict, and pre-1.0 ([§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise)).
- `grund init` is unchanged by this record. In a checkout without an optional member it dropped the member from the lists; [§DF-init-keeps-the-member-list](DF-init-keeps-the-member-list.md#df-init-keeps-the-member-list-grund-init-keeps-a-blocks-member-list-while-an-optional-member-is-absent) has it keep an existing member list there instead.

## 4. Alternatives considered

- **A note with exit 0** ([§DF-check-compares-the-whole-block.2.1](DF-check-compares-the-whole-block.md#21-an-error-not-a-note)): an agent reads past the last line.
- **An error only for the config-derived text**, leaving the workspace sections to `init --check`: the issue's first scenario stays silent.
- **Comparing the workspace sections in every run**: the climb and expansion cost of [§DF-check-compares-the-whole-block.2.2](DF-check-compares-the-whole-block.md#22-the-workspace-sections-are-judged-at-the-outermost-root-of-a-full-checkout) on every member `check`.
- **A one-release warning before the error**: it keeps the issue's CI green for another release, when this release already reports every block outdated.

## release-note: Release note

- `grund check` now compares the whole managed `grund init` block in `AGENTS.md` and its companions with what `grund init` would write, not only its five generated sections: a block that no longer matches its config — after editing kinds, titles, the ID format, the marker, named sections or the inline-style keys, or after `grund config migrate --write` — is an `agents-init` error naming each differing section; run `grund init` to refresh it.
  At the outermost root of a workspace, in a checkout with every member, `check` also compares each block's Project namespaces and Workspace members sections, so a member joining or leaving, a member's description, or a member's first `grund init` reports the blocks it makes stale, each naming the directory to run `grund init` in (``run `grund init` in packages/billing to refresh``); a run inside a member, or in a checkout without an optional member, leaves those two sections out, and the `optional-member-absent` warning says so when the run verified a managed block.
  `agents-init` findings about another project's block that name a command, and the `optional-member-absent` warning, grow at the end (`; run it in <dir>`, `, nor the managed blocks' workspace sections`); the code and the JSON shape are unchanged.
  The block versions stay v39/v40 and v41/v42, shared with the Citation directions change of this release, so every earlier block is already reported outdated and one `grund init` per project clears both; from now on, any change grund makes to the block's text moves its version.
  `grund config migrate` refuses what it refused before; its Project map note now ends `grund check reports the block stale until then`.
