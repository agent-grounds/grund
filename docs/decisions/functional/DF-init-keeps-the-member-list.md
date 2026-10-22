# DF-init-keeps-the-member-list: `grund init` keeps a block's member list while an optional member is absent

**Date:** 2026-10-22

Settles what `grund init` does with the `### Workspace members` section in a checkout without an optional member, as [§FS-init.2.3.4.15.8](../../functional-spec/FS-init.md#234158-while-an-optional-member-is-absent) specifies. Serves [§GOAL-friendliness-first](../../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible): the commands a repository runs in CI, and the blocks it commits from there, should not turn the full checkout red. Reported as [grund#921](https://github.com/agent-grounds/grund/issues/921), split out of #906.

## status: Accepted

## 1. Context

A workspace lists `packages/secret` in `[workspace] optional_members`, and a checkout without it, CI without the private member say, is legitimate ([§FS-workspace.2.2](../../functional-spec/FS-workspace.md#22-a-member-that-may-be-legitimately-absent)). `grund init` there, at the root and in every present member, re-rendered `### Workspace members` without `secret`, and `grund init --check` exited 1 for every block that listed it. Once `grund check` compared the whole block ([§DF-check-compares-the-whole-block](DF-check-compares-the-whole-block.md#df-check-compares-the-whole-block-grund-check-holds-the-whole-managed-block-the-workspace-sections-at-the-outermost-root-of-a-full-checkout)), the full checkout's root `check` reported every block refreshed that way and committed. `check` in the partial checkout already leaves both workspace sections out ([§FS-check.3.5.5](../../functional-spec/FS-check.md#355-the-workspace-sections-are-judged-at-the-outermost-root-of-a-full-checkout), its fourth case); `init` drew no such line.

## 2. Decision

### 2.1 Keep what cannot be seen, rather than infer it

The issue asked for the opposite: list the absent member from what the config says about it. That would have to invent its description, its link target and any nested members, which only the member's own files hold, and [§FS-init.2.3.4.15.7](../../functional-spec/FS-init.md#234157-what-init-does-not-do) forbids `init` to infer topology. So `init` keeps an existing block's `### Workspace members` section byte for byte and refreshes the rest; with no section to keep, it renders what it can read and says the absent member is missing from it ([§FS-init.2.3.4.15.8](../../functional-spec/FS-init.md#234158-while-an-optional-member-is-absent)). The rendering rules of [§FS-init.2.3.4.15.3](../../functional-spec/FS-init.md#234153-foreign-project-rows) to .7 do not move, and neither does `### Project namespaces`, which renders the same in both checkouts.

### 2.2 One boundary for both commands

`init` keeps a section in exactly the runs where `check` leaves it out, so in a partial checkout `init --check` and `check` agree, and the bytes `init` keeps are the bytes `check` does not compare. The member list is refreshed only from a checkout that can see every member, whose root `check` reports the blocks still waiting. That is a blind spot of `init --check` in the partial checkout, declared by one note per absent member and bounded by that run ([§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded)).

### 2.3 It lands at once, with a notice

A verdict moves: `init --check` exits 0 where it exited 1, and `init` writes different bytes. Both move from fail to pass, and nothing that passed now fails, so no deprecation window is owed ([§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) guards a newly failing verdict); the move is announced in the release note below. No render changes, so no block version moves ([§FS-init.2.3.7](../../functional-spec/FS-init.md#237-the-block-version)).

## 3. Consequences

- In a checkout without an optional member, `grund init` and `grund init --check` can run, and what they write can be committed, without turning the full checkout red.
- In that checkout nothing reaches the member list: a member added there, a member's changed description, an older rendering of the list. These wait for `grund init` in a full checkout, and the note says so on every run.
- stderr gains one `note:` per absent member; a note moves no exit code ([§FS-init.4.1](../../functional-spec/FS-init.md#41-what-earns-the-1)).

## 4. Alternatives considered

- **List the absent member from the config** (the issue's proposal): it infers what only the member's files say.
- **Leave `init` as it was, and refresh blocks only from a full checkout**: a convention the commands do not enforce, and `init --check` stays red in exactly the checkout CI runs.
- **Keep both workspace sections**: `### Project namespaces` renders the same in both checkouts, so keeping it would only hide a real refresh.

## release-note: Release note

- [§FS-init.2.3.4.15.8](../../functional-spec/FS-init.md#234158-while-an-optional-member-is-absent): **`grund init` keeps a block's member list while an optional member is absent.** In a checkout without a `[workspace] optional_members` entry, `grund init` and `grund init --check`, at the root and in each present member, keep the `### Workspace members` section of an existing managed block exactly as it is and refresh the rest of the block, as `grund check` already leaves that section out there; `--force` keeps it too. A block written new there lists only the members the run can read. Each run prints one `note:` per absent member saying whether each entrypoint's section was kept or written new. **What moves:** `grund init --check` in such a checkout exits 0 where it exited 1 when the member list is the only difference, and `grund init` there no longer drops the member from the list. Refresh the member list from a checkout that has every member. (agent-grounds/grund#921)
