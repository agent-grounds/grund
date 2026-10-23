# DF-init-configured-kind-guidance: `grund init`'s guidance names a kind and a home the project declares

**Date:** 2026-10-23

Settles what the `next:` block of `grund init` teaches in a project whose kinds do not give FS a home, as [§FS-init.2.2.2.3](../../functional-spec/FS-init.md#2223-the-kind-the-guidance-names) specifies. Serves [§GOAL-friendliness-first](../../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible): the first command a project runs should hand it a recipe that works in that project. Reported as [grund#940](https://github.com/agent-grounds/grund/issues/940).

## status: Accepted

## 1. Context

The block named FS whatever the config declared. [§FS-init.2.2.2](../../functional-spec/FS-init.md#222-the-next-block) taught `grund id FS` and the effective FS home, and [§FS-init.2.2.2.1](../../functional-spec/FS-init.md#2221-a-refresh-of-a-complete-setup-teaches-nothing) tested the configured FS `file`, else its `folder`, else the built-in `docs/functional-spec`. A project whose only kind is `REQ`, homed in `docs/req`, was therefore told to scaffold `docs/functional-spec` and to run `grund id FS`, which fails with `unknown kind`. It was told so on its first `grund init`, and again on the refresh `grund check` asks for after `grund config migrate --to 2 --write`, when nothing was left to do. [§FS-init.2.2.2.2](../../functional-spec/FS-init.md#2222-a-project-with-no-kinds) already covered a config with no kinds at all; nothing covered one with kinds and no FS. A declared FS row with no home met the same fallback and was sent to a folder it never named.

## 2. Decision

### 2.1 One selection, read by the advice and by the refresh test

The guidance names FS where FS has a home, otherwise the first citable row that has one, otherwise the first citable row ([§FS-init.2.2.2.3](../../functional-spec/FS-init.md#2223-the-kind-the-guidance-names)). A row with a home is preferred over the literal first row because it is the one the advice can place an ID in. The row's first `file`, else its first `folder`, is its place, and the choice never moves to another place because that one exists: a selection that changed with the tree would make the same config teach different kinds on different checkouts. The refresh test reads the same selection, so a refresh whose selected home exists teaches nothing, as an FS project's already did.

### 2.2 The built-in FS folder stays the scaffold's

`docs/functional-spec` is where `--scaffold` lays the FS starter when no FS row has a home, and it stays that. The guidance stops naming it, because a project that never declared it is not missing it. The configured-kind form therefore never offers `--scaffold`: the scaffold lays grund's whole software-development starter at that starter's own places, the FS home included ([§FS-init.2.1.3](../../functional-spec/FS-init.md#213-the-software-development-starter-files)), and is no way to lay the selected kind's home, so offering it would lay a tree the project did not ask for, and for a kind or a folder the starter does not carry would leave the selected home missing still. It asks for the selected home only while that home is missing. What `--scaffold` should lay in a project without FS is a separate question, left open here.

### 2.3 The advice changes now, under the pre-1.0 licence

The advice it replaces was specified, and was wrong: it gave a recipe that fails in every project without an FS home. Keeping it behind a deprecation window would keep handing out that failing recipe for no benefit. Before `1.0` a surface may change without an alias where carrying one would cost more than the change ([§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise)). No exit code moves, no finding appears or disappears, and no managed-block byte changes, so no block version moves ([§FS-init.2.3.7](../../functional-spec/FS-init.md#237-the-block-version)). The data is additive: `InitNext` gains `allocation` beside `fs_home`, which keeps its meaning.

## 3. Consequences

- A project that adopted grund with its own vocabulary can follow the block as printed, and the refresh after migrating to version 2 is quiet when its home exists.
- An FS project with an FS home sees no change.
- stderr changes for every project without an FS home: on a first run, on a `--dry-run` or `--check` preview, and on a refresh whose selected home is missing.
- A Rust caller that builds `InitNext` with a struct literal adds `allocation`; a caller that reads it gains the selected kind and place.

## 4. Alternatives considered

- **Always the first citable row**: its advice names a kind it cannot place an ID in when an earlier row has no home and a later one does.
- **Whichever place exists on disk**: the same config would then teach different kinds on different checkouts.
- **Reuse `fs_home` for the selection**: it cannot say which kind it is, or that a row has no home.
- **Let `--scaffold` lay a starter for the selected kind**: a starter is a template's to define, and no template has one for a kind it does not ship. That is a separate change.

## release-note: Release note

- [§FS-init.2.2.2.3](../../functional-spec/FS-init.md#2223-the-kind-the-guidance-names): **`grund init`'s `next:` block names a kind and a home your project declares.** In a project with kinds but no FS kind, or with an FS row that has no home, `grund init` no longer tells you to scaffold `docs/functional-spec` and to run `grund id FS`. It teaches `grund id <KIND>` for FS where FS has a home, otherwise for the first citable kind with a home, otherwise for the first citable kind, in configuration order, under that kind's first file or folder. It asks you to create that home only while it is missing, and to declare the ID in a source doc-comment where the kind has none. A refresh whose selected home exists, such as the one `grund check` asks for after `grund config migrate --to 2 --write`, now prints no `next:` block. **What moves:** stderr of `grund init`, `--dry-run` and `--check` in such projects; no exit code and no written byte. The Rust `InitNext` gains `allocation: Option<InitAllocation>`, carried as `allocation` by the Python and Node packages; a Rust caller building `InitNext` with a struct literal adds the field. (agent-grounds/grund#940)
