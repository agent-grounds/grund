# Workspace with an optional member: a checkout without it

A workspace root `shop` lists `packages/billing` as a member and
`packages/secret` under `optional_members`, the list for a member a checkout may
legitimately lack: a private package that CI, a sparse checkout, or an outside
contributor does not get. Every managed block was written in a full checkout, so
the root's `### Workspace members` section still lists `secret`. This checkout
does not have it. The use-case is validating cross-project citations in a
workspace, from a checkout without one of its members.

```toml
# repo/grund.toml
[workspace]
members = ["packages/billing"]
optional_members = ["packages/secret"]
```

```markdown
<!-- repo/AGENTS.md, inside the managed block -->
### Workspace members

Cross-project citations use §alias/<ID>, one alias segment per workspace level.

- [`billing`](packages/billing/)
- [`secret`](packages/secret/)
```

It is for whoever maintains a workspace with a private member and has to keep
CI green in the checkout that lacks it. Open `repo/grund.toml` first, then the
`### Workspace members` section of `repo/AGENTS.md`. The example pins two runs,
one per step ([§FS-examples.5.5](../../docs/functional-spec/FS-examples.md#55-an-example-may-pin-several-runs)), over the same tree.

## Step 1: `grund check` says what it did not cover

```bash
grund check examples/workspace-optional-member/1-check/repo
echo $?    # 0
```

Stdout carries one warning for the absent member, and stderr is empty:

```text
grund.toml:9: warning: optional workspace member `packages/secret` is absent — citations into namespace `secret` were not checked, so this run does not cover it, nor the managed blocks' workspace sections
```

A citation into `secret` cannot be judged without the member, and neither can a
member list that names it, so `check` leaves both out and says so on the
`optional_members` line that made the absence legal
([§FS-check.3.5.5](../../docs/functional-spec/FS-check.md#355-the-workspace-sections-are-judged-at-the-outermost-root-of-a-full-checkout)).
The exit code stays `0`: nothing is misconfigured. The two `declared but never
cited` warnings beside it are this small repository's own and have nothing to
do with the absence.

## Step 2: `grund init --check` keeps the list it cannot see

```bash
grund init --check examples/workspace-optional-member/2-init-check/repo
echo $?    # 0
```

Stdout is empty. Stderr reports both files current, then one note for the
absent member:

```text
exists AGENTS.md
exists grund.toml
note: optional workspace member `packages/secret` (namespace `secret`) is absent, so AGENTS.md would keep its `### Workspace members` section as it is
```

`init` draws the line `check` drew. It does not render the member list over the
one on disk, because what it can read here is not the whole list, so the
section that still names `secret` is kept byte for byte. Every other section of
the block is still refreshed, so a config change still makes `init --check`
fail here. Run in `packages/billing`, it answers the same way. Without
`--check`, `grund init` writes those same bytes, and the full checkout's root
`check` accepts what it wrote
([§FS-init.2.3.4.15.8](../../docs/functional-spec/FS-init.md#234158-while-an-optional-member-is-absent)).

## The trade-off

- **The benefit.** `grund check`, `grund init` and `grund init --check` all pass
  in the partial checkout, and what they write can be committed without turning
  the full checkout red.
- **The cost.** Nothing in this checkout changes the member list: a member
  added here, or a member's new description, waits for `grund init` in a full
  checkout, whose root `check` reports the block until then. A block written
  here for the first time lists only `billing` and says so in its note. The
  note on every run is the reminder.
