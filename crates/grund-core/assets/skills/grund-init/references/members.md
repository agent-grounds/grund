## Splitting off a member

Split off a member when an app, package, service, or subproject needs to be
independently checked. An ordinary module or component that still belongs to the
current project does not need a member; a documentation folder alone is not a
project boundary.

1. Give the member its own `grund.toml`, with the kinds and scan scope it owns,
   and a stable `project_name` to use as its alias.
2. Register the member directory in the workspace root's `[workspace]` `members`.
   When splitting a standalone repository, add that workspace configuration to
   its root `grund.toml` first.
3. Run `grund init` in the member to render its own agent entrypoints, and at the
   root to refresh its workspace guidance: once per project, in any order. Every
   project's block lists the
   members, so the change makes the siblings' blocks stale too: run `grund check`
   at the root and `grund init` in every directory its findings name. Init does
   not create the topology or initialize sibling members for you. In a checkout
   without one of the `optional_members`, `grund init` keeps each block's member
   list as it is and says so, so run it from a full checkout to change the list.
4. Keep local citations local. Across members, use `§alias/<ID>` (substitute your
   configured citation marker), with one alias segment per nesting level:
   `§group/member/<ID>` for a nested member.
5. Run `grund check` from the workspace root to validate the whole workspace and
   its qualified citations.

For stale workspace sections, `expected …; found …` shows the first unequal
entries; `<missing>` marks an absent side. Run `grund init` in the finding's
directory (the checked project if omitted). See the [entry-detail rule](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-check.md#3531-first-differing-workspace-entry).

A member-local `unknown project alias` finding still fails. When its alias
names the verified enclosing workspace's root project, follow the root location
and `grund check` remedy in the finding to validate the cross-project citation.
Do not rename that alias merely because a member cannot resolve it. The root
check may still find a missing target; an unknown alias without this verified
relationship is not evidence of a workspace root alias.

`grund check --full` widens the walk beyond configured source scope. The wider walk stops only at *declared* workspace members, not at any nested `grund.toml`, so a vendored or example project inside the tree is read under this project's grammar; exclude it with the loaded version's source key, or list it in `[workspace] members` if it is one of ours.
Source exclusions use `[schema.sources] exclude` in v2, `[scan] exclude` in v1.

If a misplaced-declaration error names an unread nested config, repair ownership before editing declarations: add the directory to the enclosing config’s `[workspace] members`, or exclude it with that config version’s source key. A declaration heading promotion does not fix membership. Exclusions prune descendants, so remove configured scan roots at or beneath the directory too when needed ([placement context](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-declarations.md#checksmisplaced-declarationunlisted-project-explain-the-enclosing-homes-beneath-an-unlisted-project)).

A neighbour the build checks out beside the repository is not a member: declare it under `[workspace.imports.<alias>]` with its `path` and the `project` it must declare, cite it as `§alias/<ID>`, and keep the `import-live` warning, which says it is read live ([sibling imports](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-imports.md)).

See [workspace configuration and command scope](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-workspace.md)
for membership, alias derivation, and nested workspaces.
