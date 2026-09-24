//! The `[workspace]` expansion walk (§AR-system.2.4): turning one block into the
//! list of projects a run operates on, and naming each of them.
//!
//! Split out of `context.rs` along the seam that was already there — that file
//! answers "what does a query command hold?", this one answers "which projects
//! are there, and what is each one called?" (§AR-workspace.6.1,
//! §AR-core-module-layout.1).

use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::findings::absorbed_scan_diagnostic;
use super::members::{
    AncestorWorkspaces, WorkspaceMember, canonical_workspace_path, expand_workspace_member_list,
};
use super::optional_members::{
    optional_entry_naming, optional_member_alias, qualify_absent_optional,
    register_absent_optional_aliases, workspace_optional_members_error,
};
use super::scope::{
    RootMode, config_location_error, derive_alias, duplicate_alias_sites, project_label,
    project_name_error, workspace_members_error,
};
use crate::config::display_path;
use crate::config::{AbsentOptionalNamespace, Config, RunWarning, load_config_at_with_report_base};
use crate::model::Diagnostic;

/// One project of the run a qualified citation can name, as the scanner needs
/// it: the alias the citation writes and the whole `Config` its ID is parsed and
/// rendered with, because a workspace may mix `[id] format`s (§FS-workspace.1.2,
/// §AR-workspace.2).
///
/// The same two facts as [`WorkspaceProjectEntry`] below, which is why it sits
/// beside it: what the expansion named a project, and the config that spells its
/// IDs. It sat in `model/records.rs` while workspace was a file-name category,
/// which is what made that file read `Config` for something other than the
/// qualified-ID renderer (§AR-system.4). The list of them is built once per run,
/// by the loader in `resolver/context.rs`, so each project's scan parses
/// `§<alias>/<ID>` with the target's own grammar inline rather than in a second
/// disk pass (§AR-system.2.10).
#[derive(Clone)]
pub(crate) struct WorkspaceCitationTarget {
    pub(crate) alias: String,
    pub(crate) config: Config,
}

/// One project the workspace walk reached: the alias path qualified citations
/// name it by — one segment per workspace level, so a nested project carries
/// its whole chain (§FS-workspace.3, §FS-workspace.6.1) — and its own loaded
/// config, whose `root` is the canonical project root.
pub(crate) struct WorkspaceProjectEntry {
    pub(crate) alias: String,
    pub(crate) config: Config,
}

/// §FS-workspace.6.1.5: the alias path of `config`'s own root, read from the
/// outermost workspace above it — empty when this *is* the outermost workspace.
///
/// This is what makes an alias path mean one thing at every scope: narrowing a
/// run to a subtree drops the projects outside it but never re-spells the ones
/// inside, so a citation that passes inside `hardware/` passes in CI at the
/// repository root and vice versa (§FS-workspace.5.1).
///
/// The climb takes the outermost ancestor that both declares `[workspace]` *and*
/// actually lists the directory below it — a workspace config that does not
/// claim this tree says nothing about how it is named.
///
/// Fallible on purpose (§FS-workspace.6.1.7): every block in the claimed chain has
/// to answer. One that claims this directory but cannot expand its members, or
/// cannot name the project below it, fails the run with its own error instead of
/// dropping out of the path — a dropped segment would leave the subtree with a
/// namespace no other scope agrees with, and §FS-check.3.8 would then tell the
/// author to write the one spelling that fails in CI.
fn enclosing_alias_prefix(config: &mut Config) -> Result<String> {
    let mut segments: Vec<String> = Vec::new();
    // The first child is the caller's own config, already loaded; each later one is
    // the claiming ancestor the previous step returned — either way the config in
    // hand is the one being read, so the climb never reloads a level (§AR-workspace.5.1).
    let mut climbed: Option<Config> = None;
    // One cache for the whole climb: every level walks the same ancestors, so
    // each ancestor's config is read once per run rather than once per level
    // (§AR-workspace.6.1.5).
    let mut ancestors = AncestorWorkspaces::for_run_at(&config.root);
    loop {
        let child_config = climbed.as_ref().unwrap_or(config);
        let Some(parent) =
            enclosing_workspace_of(&child_config.root, &config.cli_base, &mut ancestors)?
        else {
            break;
        };
        // A claimed directory is a member of `parent`, so its alias is derived
        // and located exactly as the outer run derives and locates it — the same
        // error text, from the same line (§AR-workspace.5.3).

        // §FS-workspace.2.2.2: which of the two lists claimed it decides where the
        // alias comes from — a present optional member reads its own segment here,
        // the same answer the outer run reads for it.
        let alias = match optional_entry_naming(&parent, &child_config.root) {
            Some(entry) => optional_member_alias(child_config, &parent, entry)?,
            None => member_alias(child_config, &child_config.root, &parent, &parent)?,
        };
        segments.push(alias);
        climbed = Some(parent);
    }
    // §FS-workspace.6.1.7: the climb that spells this run's own alias path is the one
    // that owes the reader an ancestor it could not read, so what it found travels
    // on the run's config with the rest of its warnings (§FS-distribution.3.1).
    config.run_warnings.extend(
        ancestors
            .take_warnings()
            .into_iter()
            .map(RunWarning::Settled),
    );
    segments.reverse();
    Ok(segments.join("/"))
}

/// The **outermost** ancestor workspace that declares `child` a member, or
/// `None`. Each step moves strictly upward, so the caller's climb terminates.
///
/// Outermost rather than nearest (§FS-workspace.6.1.6): a multi-segment `members`
/// entry (`grp/inner`) hops the directories between, and a hopped directory may
/// declare `[workspace]` and list the same child. Stopping at the nearer of the
/// two claims would name `inner` from a directory nothing lists and lose every
/// segment above it, while the top-down walk from the outermost root composes the
/// path through the outer claim — so the climb has to follow the same one. Where
/// exactly one block claims a directory, which is ordinary nesting, the two are
/// the same block.
///
/// A block that **claims** this directory and cannot expand its member list is
/// that block's own config error, raised here rather than read as "does not claim
/// this directory": the key that failed is the very one that would have answered
/// the question (§FS-workspace.6.1.7). The claim is read off the entry text first,
/// so a block that names nothing here is never expanded and its errors are not
/// this run's business. The entry text is read from the file on its own
/// (`ancestor_member_entries`) precisely so that a config which does not *load*
/// can still be asked the question: a claiming block that will not parse fails the
/// run like any other claim it cannot answer, while one that names nothing here is
/// climbed past however broken it is — this walk reaches the filesystem root, and a
/// stray `grund.toml` above the repository must not break every run beneath it.
pub(crate) fn enclosing_workspace_of(
    child: &Path,
    cli_base: &Path,
    ancestors: &mut AncestorWorkspaces,
) -> Result<Option<Config>> {
    // The claim is compared canonically: a `members` entry may reach this
    // directory through a symlink, and then only the resolved paths agree
    // (§FS-workspace.6.1.4).
    let canonical_child = canonical_workspace_path(child);
    let mut claiming = None;
    let mut cursor = child.parent();
    while let Some(dir) = cursor {
        if let Some(parent) = ancestors.claiming_block(dir, child, &canonical_child, cli_base)? {
            claiming = Some(parent.clone());
        }
        cursor = dir.parent();
    }
    Ok(claiming)
}

/// Append one alias segment to the path of the workspace that contains it.
/// The outermost root contributes no segment: its members are named from the
/// top of the workspace, which is what keeps a single-level workspace's
/// citations byte-identical to what they were (§FS-workspace.6.1).
pub(super) fn qualify_alias(prefix: &str, alias: &str) -> String {
    if prefix.is_empty() {
        alias.to_string()
    } else {
        format!("{prefix}/{alias}")
    }
}

/// §FS-workspace.6.1 / §AR-workspace.6.1: expand a workspace config into every
/// project in the tree — this block's own root when `include_root = true`, then
/// each member, recursing into any member that declares its own `[workspace]`.
/// The result is one list whatever the depth, because depth lives in the *key*:
/// each entry's alias is its whole path, which the walk composes one segment
/// per level.
///
/// Also sets `workspace_boundary_roots` on `root_config` and on every nested
/// workspace config, so each block's scan stops at its own members
/// (§AR-workspace.6) rather than absorbing them into its namespace, and
/// `workspace_project_roots` on every project the walk reached, so each of them
/// stops at the others in the directions the downward list cannot see
/// (§FS-workspace.6.2).
///
/// Every launch-time invariant fires here, before any scan: the alias grammar,
/// the per-block "something in scope" rule, alias uniqueness among siblings,
/// and the canonical-root check that both rejects a member cycle and bounds the
/// walk.
///
/// §FS-workspace.2.2.10: "at least one project in scope" is read from the config
/// *text* — a non-empty `optional_members` list names members, and whether they are
/// present is a fact about the checkout rather than about the config. So a block
/// that loses its last project to an absence is not the empty block §FS-workspace.6.1.3
/// refuses; failing on a checkout is the verdict that key exists to remove.
///
/// The tree is rendered against its own root, which is what a walking command
/// wants: `resolve_workspace_config` re-roots a narrowed run onto the member it
/// was launched in (§AR-workspace.5.1), so the run's root and the top of the
/// tree it expands are one directory. `init` is the route where they are not —
/// see [`expand_workspace_tree_with_report_base`].
pub(crate) fn expand_workspace_tree(
    root_config: &mut Config,
) -> Result<Vec<WorkspaceProjectEntry>> {
    let report_base = root_config.root.clone();
    expand_workspace_tree_with_report_base(root_config, &report_base)
}

/// [`expand_workspace_tree`] with the base every block's config path is rendered
/// against named explicitly (§FS-check.3.29.7, §FS-errors.4).
///
/// One run renders every block against one base, and that base is the root the
/// run was launched at — the same one [`AncestorWorkspaces::for_run_at`] carries
/// up the climb, so a block above the run's root and a block below it are spelled
/// from the same place the reader is standing. `init` expands the *outermost*
/// workspace above its target rather than re-rooting onto it, so there the top of
/// the tree lies above the run: without this the blocks below the run's root were
/// rendered from the workspace root while the blocks above it were rendered from
/// the launch directory, and one `grund init` printed two spellings of one tree —
/// the nested ones naming a `grund.toml` that does not exist from where it ran.
///
/// Re-basing the other way — every block onto the workspace root — is what
/// §FS-workspace.6.1 already forbids for an ancestor's `members` line: from a
/// narrowed directory a *different* `grund.toml` exists, so that spelling names a
/// real file that is the wrong one.
///
/// §FS-check.4.10.7 is asked at the end rather than where each block is met, because
/// the answer depends on `workspace_project_roots` and this is where the run first
/// has it: a probe that did not stop where the scan stops would report a directory
/// another project of this run reads, which is the false positive the finding's
/// condition exists to avoid. Only *when* the blocks are asked moves — the order
/// does not, since they are asked in the order they were reached, after the block
/// the run is rooted at has already had its turn in `apply_workspace_boundary`. Two
/// consequences worth stating: the count reaches `check` on the run's own config
/// either way (§FS-check.2.1), and a tree whose expansion fails is never cautioned
/// about, which is the same line §FS-check.4.10.8 already draws around a block the
/// run refuses outright.
pub(crate) fn expand_workspace_tree_with_report_base(
    root_config: &mut Config,
    report_base: &Path,
) -> Result<Vec<WorkspaceProjectEntry>> {
    let expanded = expand_workspace_member_list(root_config)?;
    let members = expanded.members;
    // §FS-check.3.29: no warning here. Every route in asks this block first —
    // `resolve_workspace_config` (§AR-workspace.5.1), or `find_init_workspace_root`
    // for `init` — so this only repopulates that boundary; asking again says it twice.
    root_config.workspace_boundary_roots = members.iter().map(|m| m.root.clone()).collect();

    let mut entries: Vec<WorkspaceProjectEntry> = Vec::new();
    let mut visited: Vec<PathBuf> = vec![root_config.root.clone()];
    // §FS-workspace.3: the root project and the top-level members share one
    // level of the namespace, so they are checked for collisions together.
    let mut siblings: BTreeMap<String, PathBuf> = BTreeMap::new();
    // §FS-workspace.6.1.5: the alias path of *this* workspace's own root, read from
    // the outermost workspace. Empty unless the run was narrowed to a subtree — and
    // that is what keeps a narrowed run resolving a subset of the same paths.
    let self_path = enclosing_alias_prefix(root_config)?;
    // §FS-check.4.9.2: the namespaces this walk did not read, gathered as it goes and
    // named by the whole alias path the run spells them with — this block's under
    // the run's own path, each nested block's under that block's.
    let mut absent_optional = qualify_absent_optional(expanded.absent, &self_path);
    if root_config.workspace_include_root {
        let alias = if self_path.is_empty() {
            derive_alias(root_config, None, RootMode::Root).map_err(|err| {
                let message = format!(
                    "{err} for {}",
                    project_label(root_config, &root_config.root)
                );
                project_name_error(root_config, message)
            })?
        } else {
            self_path.clone()
        };
        // Only at the outermost root do the root project and the members share
        // a level. In a narrowed run this project sits one level *up* from the
        // members below it, exactly where the outer run puts it.
        if self_path.is_empty() {
            siblings.insert(alias.clone(), root_config.root.clone());
        }
        entries.push(WorkspaceProjectEntry {
            alias,
            config: root_config.clone(),
        });
    }
    // §FS-workspace.3: the absent entries claim their aliases before the present
    // members are derived — this block's level holds one project per alias whether
    // or not the run can read it.
    register_absent_optional_aliases(
        root_config,
        root_config,
        &self_path,
        &absent_optional,
        &mut siblings,
    )?;
    // §FS-check.4.10.7: every block below the run's root is posed the question down
    // there and answered below, once this run knows where its projects are.
    let (absorbed, unread_blocks) = collect_workspace_members(
        &members,
        root_config,
        root_config,
        report_base,
        &self_path,
        &mut siblings,
        &mut visited,
        &mut entries,
        &mut absent_optional,
    )?;
    // §FS-check.4.7: in the order the walk reached them, which is the order the
    // reader sees them in — ahead of §FS-check.4.10's blocks below, exactly where
    // they were printed from (§FS-errors.4).
    root_config
        .run_warnings
        .extend(absorbed.into_iter().map(RunWarning::Settled));
    // §FS-workspace.2.2.10: read from the config text — see this function's docs.
    if entries.is_empty() && root_config.workspace_optional_members.is_empty() {
        return Err(empty_workspace_error(root_config));
    }
    // §AR-workspace.6: every project the run loaded learns where the others are.
    // `workspace_boundary_roots` above points only downward, so a leaf member has none
    // and a symlink could cross out of it; §FS-workspace.6.2 forbids that in every direction.
    let project_roots: Vec<PathBuf> = entries
        .iter()
        .map(|entry| entry.config.root.clone())
        .collect();
    // §FS-check.3.8.3: every project the run loaded carries the run's own scope, so
    // the diagnostic that would re-spell a citation knows whether it is looking at
    // the whole tree or a slice of it.

    // §FS-workspace.4.3: and the namespaces it did not read, because resolution asks
    // that of the project the citing file belongs to, wherever in the tree that is.
    for entry in &mut entries {
        // The run's warning channel belongs to the run and not to a project of it
        // (§FS-distribution.3.1): the root project's config is a clone of the one
        // the boundary pass wrote to, and a second copy is a second line.
        entry.config.run_warnings = Vec::new();
        entry.config.workspace_scope_path = self_path.clone();
        entry.config.workspace_project_roots = project_roots.clone();
        entry.config.workspace_absent_optional = absent_optional.clone();
    }
    root_config.workspace_project_roots = project_roots;
    // §FS-check.4.9: the root config is what the report is rendered from, so it is
    // where the announcement is read back off (`run_workspace_check`).
    root_config.workspace_absent_optional = absent_optional;
    // §FS-check.4.10.7: the blocks that opted out, posed now that `project_roots`
    // exists — see this function's docs. The walk that answers one is the
    // resolver's, so the channel carries the block (§AR-resolver.placement).
    let project_roots = root_config.workspace_project_roots.clone();
    root_config.run_warnings.extend(
        unread_blocks
            .iter()
            .filter_map(|block| RunWarning::unread_block(block, project_roots.clone())),
    );
    Ok(entries)
}

/// One level of member expansion (§FS-workspace.6.1). `parent_config` is the
/// block that listed `member_roots` — it owns the `members` line a member error
/// points at; `top_config` is the outermost workspace root — it owns project
/// naming, so every diagnostic names a project against the same base;
/// `report_base` is where this run was launched, which is what each member's own
/// config paths are spelled from (§FS-errors.4, and the doc on
/// [`expand_workspace_tree_with_report_base`] for why it is not `top_config.root`);
/// `prefix` is the alias path of the enclosing workspace, empty at the top.
///
/// What bounds the recursion is containment: every member root resolves strictly
/// inside the block that listed it (`workspace_member_root`) and no member of one
/// block contains another (`reject_overlapping_workspace_members`), so the blocks form
/// a strict containment tree — every step consumes a canonical root strictly deeper
/// than the block that named it, and no two roots in the tree can be equal. Should a
/// duplicate reach the visited check anyway it is a config error at the line that
/// introduced it, named as the entry was written and beside the root it lands on,
/// since those are two different strings and the author wrote only one of them.
///
/// §FS-workspace.2.2.2: an optional entry's alias is its own last path segment, and
/// a `project_name` that disagrees with it is refused rather than accepted as a
/// second name — one citation text has to name one namespace in a full checkout and
/// in a partial one.
///
/// Returns what this subtree earned, in the order it reached it: the
/// §FS-check.4.7 warnings, settled here because a block's own members answer
/// them, and the §FS-check.4.10 blocks, for the caller to pose once it knows
/// where the run's projects are (§FS-check.2.1).
#[allow(clippy::too_many_arguments)]
fn collect_workspace_members(
    members: &[WorkspaceMember],
    parent_config: &Config,
    top_config: &Config,
    report_base: &Path,
    prefix: &str,
    siblings: &mut BTreeMap<String, PathBuf>,
    visited: &mut Vec<PathBuf>,
    entries: &mut Vec<WorkspaceProjectEntry>,
    absent_optional: &mut Vec<AbsentOptionalNamespace>,
) -> Result<(Vec<Diagnostic>, Vec<Config>)> {
    let mut absorbed = Vec::new();
    let mut unread = Vec::new();
    for member in members {
        let member_root = &member.root;
        // §FS-workspace.6.1.4: an **unreachable backstop**, kept because it is the one
        // that would name the line if the containment rule that bounds this recursion
        // (see the fn docs) ever stopped holding.
        if visited.iter().any(|seen| seen == member_root) {
            return Err(workspace_members_error(
                parent_config,
                format!(
                    "workspace member `{}` resolves to `{}`, already a project in this workspace",
                    member.written,
                    display_path(top_config, member_root)
                ),
            ));
        }
        visited.push(member_root.clone());
        let mut member_config =
            load_config_at_with_report_base(member_root, &top_config.cli_base, Some(report_base))?;
        // The alias is derived whether or not the member turns out to be a
        // project: a member that is itself a workspace still contributes its
        // segment to every alias path below it (§FS-workspace.6.1.2).

        // §FS-workspace.2.2.2: which list claimed it decides where the alias comes
        // from — see this function's docs.
        let alias = if member.optional {
            optional_member_alias(&member_config, parent_config, &member.written)?
        } else {
            member_alias(&member_config, member_root, parent_config, top_config)?
        };
        // §FS-workspace.3: uniqueness is a property of one level. Two projects
        // under different parents may share a segment — their alias paths still
        // differ — so the check is per sibling set, not per tree.
        if let Some(first) = siblings.get(&alias) {
            let message = format!(
                "duplicate workspace project alias `{}` ({})",
                qualify_alias(prefix, &alias),
                duplicate_alias_sites(top_config, first, member_root)
            );
            // §FS-errors.4: at the line of the list this member was written on — the
            // second claimant is as often an optional entry, and a block that lists
            // only optional members has no `members` line to carry the location.
            return Err(if member.optional {
                workspace_optional_members_error(parent_config, message)
            } else {
                workspace_members_error(parent_config, message)
            });
        }
        siblings.insert(alias.clone(), member_root.clone());
        let qualified = qualify_alias(prefix, &alias);
        if !member_config.workspace_declared {
            entries.push(WorkspaceProjectEntry {
                alias: qualified,
                config: member_config,
            });
            continue;
        }
        // §FS-workspace.6.1.2: a member that is itself a workspace root
        // contributes its whole subtree, and `include_root` on *its* block
        // decides whether the grouping directory is one of the projects.
        let nested = expand_workspace_member_list(&member_config)?;
        // §FS-check.3.29: a block below the run's root is populated here and
        // nowhere else, so this is where it is asked — once, at its own
        // `members` line (§FS-errors.4).
        absorbed.extend(absorbed_scan_diagnostic(&member_config, &nested.members));
        member_config.workspace_boundary_roots =
            nested.members.iter().map(|m| m.root.clone()).collect();
        // §FS-check.4.10.7: and whether its own tree holds anything unread, at the
        // same line — no outermost-block privilege either way. The boundary above
        // is set first: the block is what the question travels as.
        if !member_config.workspace_include_root && !nested.members.is_empty() {
            unread.push(member_config.clone());
        }
        // §FS-check.4.9: and its absent optional entries, at its own
        // `optional_members` line, under the alias path this run reaches it by —
        // there is no outermost-block privilege in either direction.

        // §FS-workspace.3: they claim their aliases in the nested block's own
        // sibling set, which is why it is built here rather than at the call.
        let mut nested_siblings = BTreeMap::new();
        register_absent_optional_aliases(
            &member_config,
            top_config,
            &qualified,
            &nested.absent,
            &mut nested_siblings,
        )?;
        absent_optional.extend(qualify_absent_optional(nested.absent, &qualified));
        let before = entries.len();
        if member_config.workspace_include_root {
            entries.push(WorkspaceProjectEntry {
                alias: qualified.clone(),
                config: member_config.clone(),
            });
        }
        let (nested_absorbed, nested_unread) = collect_workspace_members(
            &nested.members,
            &member_config,
            top_config,
            report_base,
            &qualified,
            &mut nested_siblings,
            visited,
            entries,
            absent_optional,
        )?;
        absorbed.extend(nested_absorbed);
        unread.extend(nested_unread);
        // §FS-workspace.2.2.10: the same reading one block down — a nested block whose
        // only members may be absent named members, so it is not an empty block.
        if entries.len() == before && member_config.workspace_optional_members.is_empty() {
            return Err(empty_workspace_error(&member_config));
        }
    }
    Ok((absorbed, unread))
}

/// §AR-workspace.5.3: a member's alias, with the error anchored where the bad
/// name was written — the member's own `project_name` line when it set one,
/// otherwise the `members` line of the block that named the directory.
fn member_alias(
    member_config: &Config,
    member_root: &Path,
    parent_config: &Config,
    top_config: &Config,
) -> Result<String> {
    derive_alias(member_config, Some(member_root), RootMode::Member).map_err(|err| {
        let message = format!("{err} for {}", project_label(top_config, member_root));
        if member_config.project_name_source.is_some() {
            project_name_error(member_config, message)
        } else {
            workspace_members_error(parent_config, message)
        }
    })
}

/// §FS-workspace.6.1.3: every `[workspace]` block must put at least one project
/// in scope — a block that contributes nothing would silently drop its whole
/// subtree from the check.
///
/// The `members` line is the anchor when there is one; the block that reaches this
/// error with `include_root = false` and *no* `members` key has none, so it falls
/// back to its own `[workspace]` line. Without the fallback the message carried no
/// location at all, in a tree that may hold many blocks (§FS-errors.4). A
/// non-empty list can reach here only when every entry is a glob that matched no
/// directories; §FS-workspace.6.1.3 names the first in config order instead of
/// falsely claiming there were no members.
fn empty_workspace_error(config: &Config) -> anyhow::Error {
    let source = config
        .workspace_members_source
        .as_ref()
        .or(config.workspace_section_source.as_ref());
    let message = config.workspace_members.first().map_or_else(
        || "workspace has no projects in scope (include_root = false and no members)".to_string(),
        |glob| format!("the glob `{glob}` matched no directories"),
    );
    config_location_error(source, message)
}
