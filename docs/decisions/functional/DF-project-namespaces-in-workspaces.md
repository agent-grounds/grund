# DF-project-namespaces-in-workspaces: teach project boundaries in workspace blocks and member splitting in setup instructions

**Status:** Accepted
**Date:** 2026-10-14

## 1. Context

Every standalone repository currently receives about 92 words of workspace guidance in its managed block, including instructions pointing to a workspace root it does not have. Removing that guidance must not remove the member-splitting advice an agent needs when setting up a separate project (agent-grounds/grund#574).

## 2. Decision

Emit Project namespaces only for a target in a discovered workspace ([§FS-init.2.3.4.16](../../functional-spec/FS-init.md#23416-project-boundaries)). Reuse the claimed-chain discovery that governs Workspace members ([§FS-init.2.3.4.15.1](../../functional-spec/FS-init.md#234151-which-workspace), [§FS-workspace.6.1](../../functional-spec/FS-workspace.md#61-nested-workspaces)), once per run. Finding the workspace and expanding its members are separate: expansion errors and duplicate aliases keep Project namespaces while suppressing Workspace members. An unrelated ancestor contributes neither section. Nested workspaces retain their discovery semantics.

Extract the fixed section into `templates/AGENTS-project-namespaces.md` and its embedded asset twin. Preserve the current workspace section byte for byte, with configured marker substitution and existing spacing. An independent pre-change expectation pins those bytes. Both setup-instructions surfaces teach the complete member-splitting recipe ([§FS-init.5.4](../../functional-spec/FS-init.md#54-splitting-off-a-member), [§FS-init.5.3](../../functional-spec/FS-init.md#53-one-body-two-surfaces)), using member and alias in new prose ([§FS-terms.terms.4](../../functional-spec/FS-terms.md#terms4-scanning-and-project-structure)).

## 3. Migration

This is a loud mechanical migration ([§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)): earlier supported blocks become outdated, and one `grund init` re-renders them without rewriting unmanaged content ([§FS-init.2.3.10.1](../../functional-spec/FS-init.md#23101-re-rendering-an-existing-block), [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own)). The base and rule-enabled versions advance together; rule-enabled remains base plus one ([§FS-init.2.3.7.1](../../functional-spec/FS-init.md#2371-the-current-version)). Take the baseline from `main` when implementing and synchronize the maximum supported version, checker selector, template heading, and rule-heading promotion.

Implementation-time baseline: `main` at `23010de6dfbfd37864a6c9cef753041594219920` on 2026-10-14 uses base v17 and rule-enabled v18. Revalidation confirmed the selected pair is v19/v20.

## release-note: Release note

- [§FS-init.2.3.4.16](../../functional-spec/FS-init.md#23416-project-boundaries), [§FS-init.2.3.7.1](../../functional-spec/FS-init.md#2371-the-current-version): **the managed block moves v17 → v19, and v18 → v20 where a rule kind is enabled**. Standalone blocks omit Project namespaces; workspace blocks retain its existing text. Both setup-instructions surfaces teach member splitting. **Who this breaks:** every repository with an earlier grund block, including workspace repositories; `grund check` and `grund init --check` report it outdated. Migration: run `grund init`. (agent-grounds/grund#574)
