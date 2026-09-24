//! The unlisted-`[workspace]` rule (§FS-check.3.29), in a file of its own beside
//! the rest of the workspace machinery (§AR-system.2.4, §AR-core-module-layout.1,
//! §AR-workspace.6.1.6): a directory this run's walk reached that declares
//! `[workspace]` and that no enclosing block lists among its `members` is
//! claimed by nobody, so its subtree is absorbed into the enclosing namespace
//! instead of named under its own alias path (§FS-workspace.6.1.8).
//!
//! The rule sits **above** the walk, never inside it: the scanner carries out the
//! directories it reached and asks nothing of them (§AR-workspace.1), and every
//! question about claims, configs and messages is answered here.
//!
//! Three filters in cost order, so a tree with no nested config pays the probe and
//! nothing else (§GOAL-fast-feedback): one `config_file_in` probe per walked
//! directory, a text-only `[workspace]`-header read for the directories that carry
//! a config, and the ancestor claim climb only for the ones that declare the table.
//! `load_config_at` is deliberately not used for a candidate — a config that will
//! not parse must not fail the run, and a full load rebuilds the grammar regex set
//! per candidate.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::expand::enclosing_workspace_of;
use super::members::{AncestorWorkspaces, canonical_workspace_path};
use super::scope::{RootMode, derive_alias};
use crate::config::display_path;
use crate::config::{Config, config_file_in, strip_comment};
use crate::model::Diagnostic;
use crate::model::{format_path, relative_from_base};

/// Where a finding of this rule states the block's `[workspace]` line
/// (§FS-check.3.29.7, §DF-unlisted-workspace-error-shape.2.2). The two channels the
/// rule is left with differ in this and in nothing else: one sentence, two
/// placements (§FS-check.3.29.6).
#[derive(Clone, Copy, Eq, PartialEq)]
enum Placement {
    /// The location is the finding's own `path` and `line` and the message does not
    /// repeat it — §FS-errors.2.2.1's shape for a finding a frontend places:
    /// `check`'s report error behind the §FS-check.2.1 prefix, and the editor's
    /// diagnostic (§FS-check.3.29.13).
    Located,
    /// No location fields, and the message opens with `<path>:<line>: `: the five
    /// walking surfaces of §FS-check.3.29.9 print the sentence and place nothing, so
    /// the file and the line have to be in the text a reader greps
    /// (§FS-check.3.29.15).
    InText,
}

/// §FS-check.3.29, §FS-check.3.29.13: one **located error** per outermost
/// `[workspace]` block this run's walk met that no enclosing block lists — `path`
/// the block's own config and `line` its `[workspace]` line.
///
/// An error since the ramp ended in `0.15.0` (§FS-check.3.29.14,
/// §DF-unlisted-workspace-error-shape.2.1): being an error is what reaches the exit
/// code (§FS-check.3), and being located is what puts it on stdout behind the
/// mandatory prefix and the anchor into its JSON object.
///
/// `config` is the project whose walk produced `walked_dirs` — the namespace the
/// block is absorbed into, and the root both remedies are written from. `render`
/// is the config every path renders against (§FS-errors.3.1): the workspace root in
/// a workspace run, `config` itself otherwise. `alias` is the alias path this run
/// spells that project with, so the message can be matched against what
/// §FS-list printed.
///
/// Outermost-only falls out of the claim test rather than needing a pass of its
/// own: a block below an unlisted one *is* claimed — by the unlisted block — so
/// `enclosing_workspace_of` answers it, and one edit fixes the chain.
pub(crate) fn unlisted_workspace_block_errors(
    config: &Config,
    render: &Config,
    alias: Option<&str>,
    walked_dirs: &[PathBuf],
) -> Vec<Diagnostic> {
    block_diagnostics(config, render, alias, walked_dirs, Placement::Located)
}

/// §FS-check.3.29.15, §FS-check.3.29.9: the same findings for the five walking
/// surfaces that have no report to carry them — `list`, `refs`, `cover`, `fmt` and
/// the ID read — one CLI-level `warning:` each, with the location **inside the
/// text** because the CLI prints the sentence rather than placing it
/// (§FS-errors.2.2.1).
///
/// The channel is what the flip moved, not the anchor. `check` and the editor take
/// this finding off the report, located and as an error; these five keep the
/// warning, whose exit codes §FS-cli.5 freezes, with one word of it changed
/// (§DF-unlisted-workspace-block.2.4, §DF-unlisted-workspace-error-shape.2.3).
pub(crate) fn unlisted_workspace_block_run_warnings(
    config: &Config,
    render: &Config,
    alias: Option<&str>,
    walked_dirs: &[PathBuf],
) -> Vec<Diagnostic> {
    block_diagnostics(config, render, alias, walked_dirs, Placement::InText)
}

fn block_diagnostics(
    config: &Config,
    render: &Config,
    alias: Option<&str>,
    walked_dirs: &[PathBuf],
    placement: Placement,
) -> Vec<Diagnostic> {
    // §GOAL-fast-feedback: one cache for the whole rule, the way
    // `enclosing_alias_prefix` shares one per climb — every candidate walks the same
    // ancestors, and without it each ancestor's config is re-read per candidate.

    // Quiet (§FS-check.3.29.2): this climb spells no alias path, so an ancestor it
    // cannot read is this rule's silence rather than the reader's warning.
    let mut ancestors = AncestorWorkspaces::quiet_for_run_at(&config.root);
    let mut reported = BTreeSet::new();
    let mut diagnostics = Vec::new();
    for dir in walked_dirs {
        // The probe first: two `is_file` calls, and all a tree with no nested config
        // pays (§FS-config.1). It is what finds the `.agents/` form, which the walk
        // never meets as a file entry — it prunes hidden directories (§FS-check.3.29.1).
        let Some(config_path) = config_file_in(dir) else {
            continue;
        };
        let Some(line) = workspace_table_line(&config_path) else {
            continue;
        };
        // §FS-check.3.29.3: a project root of this run is never a candidate — without it
        // `--full`, which makes the config root a walk root (§FS-check.1.3), reports
        // every workspace repository against itself. Canonicalizing is the dear half.
        let canonical = canonical_workspace_path(dir);
        if is_project_root_of_run(config, &canonical) {
            continue;
        }
        // §FS-check.3.29.4 "one finding for one edit": one block the walk reached under
        // two spellings answers the claim test identically, so the first spelling met
        // is the one reported and a symlinked second is not another finding.
        if !reported.insert(canonical) {
            continue;
        }
        match enclosing_workspace_of(dir, &config.cli_base, &mut ancestors) {
            // Claimed, at any depth: inside the chain, so nothing is absorbed.
            Ok(Some(_)) => continue,
            // §FS-workspace.6.1.8: a claim an ancestor names but cannot answer is
            // undecidable in both directions, so the block is left unreported and
            // unexplained (§FS-check.3.29.2) rather than called unlisted.
            Err(_) => continue,
            Ok(None) => {}
        }
        let located = placement == Placement::Located;
        diagnostics.push(Diagnostic {
            code: "unlisted-workspace-block",
            // §FS-check.3.29.7: the located form carries the block's `[workspace]`
            // line in these two fields, and the in-text form carries it in the
            // message instead — the one difference between the two channels.
            path: located.then(|| config_path.clone()),
            line: located.then_some(line),
            column: None,
            message: unlisted_workspace_block_message(
                config,
                render,
                alias,
                dir,
                &config_path,
                line,
                placement,
            ),
            // §FS-check.3.29.13: one block, one site, so `sites` stays null.
            sites: Vec::new(),
        });
    }
    diagnostics
}

/// §FS-check.3.29.6: the sentence, built apart from the reporting so both channels
/// print one text — `check`'s report error and the direct stderr line the five
/// other walking surfaces print (§DF-unlisted-workspace-block.2.4). `placement`
/// decides only whether it opens with the location (§FS-check.3.29.7).
///
/// Four facts and a spent deadline: the block's `[workspace]` line, what the
/// absorption costs, the two config edits that clear it, and the release the
/// finding became an error in (§FS-check.3.29.14). The second remedy is stated as an outcome rather
/// than as a key on purpose — `[scan] exclude` prunes descendants and never the
/// directory a walk starts at (§FS-check.1.3.2), so a block that is itself an
/// `include` root takes `include` and one below it takes `exclude`.
fn unlisted_workspace_block_message(
    config: &Config,
    render: &Config,
    alias: Option<&str>,
    dir: &Path,
    config_path: &Path,
    line: usize,
    placement: Placement,
) -> String {
    // Both remedies are written from the enclosing project's root, so the entry is
    // spelled from there — while the two file paths render against the run's report
    // base like every other path in a diagnostic (§FS-errors.3.1).
    let entry = format_path(&relative_from_base(&config.root, dir));
    let enclosing_config = config
        .config_file
        .clone()
        .unwrap_or_else(|| config.root.join("grund.toml"));
    // §FS-check.3.29.7: only the in-text form opens with the anchor; the located
    // form leaves it to the two fields the frontend reads it from.
    let location = match placement {
        Placement::Located => String::new(),
        Placement::InText => format!("{}:{line}: ", display_path(render, config_path)),
    };
    format!(
        "{location}this [workspace] is listed by no enclosing workspace \
         — the projects under it are absorbed into `{absorbing}` instead of named under \
         their own alias path; add \"{entry}\" to [workspace] members in {enclosing}, \
         or keep it out of that project's [scan] \
         — an unlisted [workspace] became an error in grund 0.15.0",
        absorbing = absorbing_project_name(config, alias),
        enclosing = display_path(render, &enclosing_config),
    )
}

/// The alias path this run spells the absorbing project with (§FS-workspace.3), so
/// the message can be read beside what §FS-list printed. A run that loaded no
/// workspace has no alias path to offer, and falls back to the name the project
/// would carry as a workspace root — its `project_name`, or `root`
/// (§AR-workspace.5.3) — which is what the reader would see the moment the block
/// is listed and the namespace becomes one.
fn absorbing_project_name(config: &Config, alias: Option<&str>) -> String {
    match alias {
        Some(alias) if !alias.is_empty() => alias.to_string(),
        _ => derive_alias(config, None, RootMode::Root)
            .unwrap_or_else(|_| "the enclosing project".to_string()),
    }
}

/// §FS-check.3.29.3: whether this canonical directory is one of the project roots the
/// run names everything else from — its own root, and each member root the walk
/// stops at (§FS-workspace.6).
fn is_project_root_of_run(config: &Config, canonical: &Path) -> bool {
    canonical == canonical_workspace_path(&config.root)
        || config
            .workspace_project_roots
            .iter()
            .chain(&config.workspace_boundary_roots)
            .any(|root| root == canonical)
}

/// The line a config's `[workspace]` table opens on, or `None` when it declares
/// none (§FS-check.3.29.1).
///
/// A text-only read, for the same reason `ancestor_member_entries` is one
/// (§FS-workspace.6.1.7): the question is asked of a config this run does not
/// otherwise load, and a candidate that will not parse must not fail the run. The
/// section header is read exactly as `parse_config_file` reads one, so the forms it
/// *rejects* still count as a declared block — a `[[workspace]]` here is a block
/// somebody meant, and calling it "no block" would silence the finding on the very
/// config that is most confused. An unreadable file declares nothing this run can
/// see and says nothing.
///
/// Reading the header exactly as the parser reads one is the deliberate half, and
/// the identity with `ancestor_member_entries` is worth more than being cleverer
/// here: a textual read also sees a header-shaped line nobody meant as a header —
/// one inside a multi-line value — in a config `parse_config_file` already
/// rejects, and two readers of one file that disagreed about what a section header
/// is would be the worse bug (§FS-workspace.6.1.7).
fn workspace_table_line(config_path: &Path) -> Option<usize> {
    let text = fs::read_to_string(config_path).ok()?;
    text.lines().enumerate().find_map(|(idx, raw_line)| {
        let line = strip_comment(raw_line).trim();
        (line.starts_with('[')
            && line.ends_with(']')
            && line.trim_matches(['[', ']']) == "workspace")
            .then_some(idx + 1)
    })
}
