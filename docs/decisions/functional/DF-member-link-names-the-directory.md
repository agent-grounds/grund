# DF-member-link-names-the-directory: A workspace member link names the member's directory

**Date:** 2026-10-23

Settles how a `### Workspace members` bullet links a foreign project, as [§FS-init.2.3.4.15.6](../../functional-spec/FS-init.md#234156-the-link-target) specifies. Serves [§GOAL-friendliness-first](../../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible): the documented setup of a workspace, one `grund init` per project, should end green without a person working out which project to initialize again. Reported as [grund#931](https://github.com/agent-grounds/grund/issues/931), left open by #906 as its R1-04.

## status: Accepted

## 1. Context

A bullet linked a foreign project's `AGENTS.md` when that file existed on disk at the time `init` ran, and otherwise linked the project's directory and ended `*(not yet initialized)*`. In a fresh workspace the first project initialized therefore described every other one as not yet initialized, which was true at that moment. Each later `init` writes only its own project ([§FS-init.2.3.4.15.7](../../functional-spec/FS-init.md#234157-what-init-does-not-do)), so the first block kept saying so. Once `grund check` compared the workspace sections at the outermost root ([§FS-check.3.5.5](../../functional-spec/FS-check.md#355-the-workspace-sections-are-judged-at-the-outermost-root-of-a-full-checkout)), one `grund init` per project, in either order, left the root `check` red with `stale grund init block: workspace members differ from the workspace`, and only a third `init` in whichever project went first cleared it.

## 2. Decision

### 2.1 The link names the directory, always

A foreign project's bullet links its root directory with a trailing `/` and carries no suffix, whether or not that project has an entrypoint ([§FS-init.2.3.4.15.5](../../functional-spec/FS-init.md#234155-the-bullet), [§FS-init.2.3.4.15.6](../../functional-spec/FS-init.md#234156-the-link-target)). The section becomes a function of the workspace's configuration alone ([§FS-init.requirements.reproducible](../../functional-spec/FS-init.md#requirementsreproducible-what-init-writes-is-a-function-of-the-target-its-config-the-flags-and-the-version--realized)), so the order of the inits cannot make one block describe another as it was, and one `grund init` per project leaves the outermost root's `check` clean ([§FS-init.requirements.clean](../../functional-spec/FS-init.md#requirementsclean-what-init-writes-check-accepts--realized)). The directory always exists, since member expansion fails otherwise, so the link is never a 404; the entrypoint is one hop away in the directory it opens.

### 2.2 What it costs

Agents lose the in-block hint that a project has no entrypoint yet; the directory the link opens shows it instead. Every workspace block changes bytes, so the change moves the block version, and it shares the unreleased v39–v42 pairs rather than adding one ([§FS-init.2.3.7.1](../../functional-spec/FS-init.md#2371-the-current-version)).

## 3. Consequences

- One `grund init` in each project of a fresh workspace, in any order, leaves the outermost root's `grund check` green.
- Only a change to the workspace itself makes another project's block stale: a member joining or leaving, or a member's description. Those findings still name each directory to run `grund init` in.
- `grund init --check` no longer reports `would-update` in a project whose sibling has just been initialized.
- Nothing moves for [§FS-init.2.3.4.15.7](../../functional-spec/FS-init.md#234157-what-init-does-not-do), [§FS-check.3.5.4](../../functional-spec/FS-check.md#354-no-part-of-the-block-is-exempt-from-the-comparison) or [§FS-init.2.3.4.15.8](../../functional-spec/FS-init.md#234158-while-an-optional-member-is-absent): `init` still writes only the project it was invoked in, `check` still compares every byte, and a kept member list keeps the bytes it had.

## 4. Alternatives considered

- **`init` rewrites the stale line in sibling blocks**: a write outside the invoked project, which [§FS-init.2.3.4.15.7](../../functional-spec/FS-init.md#234157-what-init-does-not-do) forbids, and `--check`, `--dry-run`, companions and partial checkouts would each need a rule for it.
- **`check` stops comparing the link and the suffix**: a new blind spot to declare, and blocks would keep telling agents that an initialized project is not initialized.
- **`init` names, in its output, each directory to initialize again**: the user still runs a third `init`, and once the bullets no longer depend on disk there is nothing to name.
- **Link the `AGENTS.md` unconditionally**: a dead link between the first and the second `init`, and for good in a member never initialized.

## release-note: Release note

- [§FS-init.2.3.4.15.6](../../functional-spec/FS-init.md#234156-the-link-target): **A workspace member link names the member's directory.** Each bullet under `### Workspace members` now links the foreign project's root directory, as `` - [`billing`](packages/billing/) `` or `` - [`shop`](../../) ``, whether or not that project has been initialized; the link to its `AGENTS.md` and the `*(not yet initialized)*` suffix are gone. One `grund init` in each project of a fresh workspace, in any order, now leaves `grund check` at the workspace root green, where the project initialized first used to be reported stale. **What an upgrader does:** run `grund init` once in each workspace project, which the block versions v39/v40 and v41/v42 of this release already ask for. (agent-grounds/grund#931)
