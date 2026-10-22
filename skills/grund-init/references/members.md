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
   root to refresh its workspace guidance. Every project's block lists the
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

A member-local `unknown project alias` finding still fails. When its alias
names the verified enclosing workspace's root project, follow the root location
and `grund check` remedy in the finding to validate the cross-project citation.
Do not rename that alias merely because a member cannot resolve it. The root
check may still find a missing target; an unknown alias without this verified
relationship is not evidence of a workspace root alias.

See [workspace configuration and command scope](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-workspace.md)
for membership, alias derivation, and nested workspaces.
