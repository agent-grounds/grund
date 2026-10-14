//! The heading lines a `fmt` run leaves (§FS-fmt.6.3.3), settled before any line
//! is written: a run that expands a citation inside a section heading moves the
//! heading's anchor, and a link the same run writes to it — from another file,
//! its own, or another rewritten heading — must carry the anchor of the line the
//! run writes. The line rewrite is `rewrite_file`'s, the one every written line
//! goes through, with the link pass off: a wrap leaves a heading's rendered text
//! as it was, and the anchors it needs are what this computes.

use anyhow::Result;
use std::path::{Path, PathBuf};

use super::fmt_rewrite::{FmtLineOpts, rewrite_file};
use crate::config::{Config, fmt_excluded};
use crate::grammar::FmtExcluded;
use crate::model::Catalog;
use crate::resolver::{SettledHeadings, ShorthandTargets, WorkspaceContext};
use crate::scanner::walk_scannable_files_reporting;

/// What the rewrite of one walk does to the heading lines of `files`, read off
/// the scan's record of each Markdown file. A file the rewrite never touches
/// (`excluded`, §FS-fmt.2.5.1) and one with no heading a rewrite could reach are
/// passed over without being rewritten, so a tree with no citation in a heading
/// pays one look at each heading line (§GOAL-fast-feedback).
pub(super) fn settle_headings<'a>(
    config: &Config,
    files: impl IntoIterator<Item = &'a PathBuf>,
    excluded: &FmtExcluded,
    add_marker: bool,
    findings: &Catalog,
    workspace: Option<&WorkspaceContext>,
    shorthand_targets: &ShorthandTargets<'_>,
) -> SettledHeadings {
    let mut settled = SettledHeadings::default();
    let unsettled = SettledHeadings::default();
    let opts = FmtLineOpts {
        add_marker,
        cross_refs: false,
        excluded: false,
        index_entry_ids: None,
        findings: Some(findings),
        workspace,
        shorthand_targets,
        settled: &unsettled,
    };
    for path in files {
        if excluded.contains(path) {
            continue;
        }
        let Some(record) = findings.files.get(path).filter(|record| record.markdown) else {
            continue;
        };
        let reachable = |line: &str| {
            add_marker || line.contains(&config.marker) || line.contains(&config.trigger)
        };
        if !record
            .headings
            .iter()
            .any(|heading| heading.atx && record.line(heading.line).is_some_and(reachable))
        {
            continue;
        }
        // The record was read as Markdown, and the resolver says the same again.
        let reading = config.reading(path);
        let rewritten = rewrite_file(record.text(), path, config, reading, &opts, &mut Vec::new());
        for heading in record.headings.iter().filter(|heading| heading.atx) {
            let (Some(read), Some(left)) = (
                record.line(heading.line),
                rewritten.lines.get(heading.line - 1),
            ) else {
                continue;
            };
            if read != left {
                settled.insert(path, heading.line, left.clone());
            }
        }
    }
    settled
}

/// [`settle_headings`] over one project's whole tree, for a workspace run whose
/// links cross into a project before that project's own walk has rewritten it
/// (§FS-workspace.8.5). Its walk is the one the project's own `fmt_tree` makes.
pub(super) fn settle_project_headings(
    config: &Config,
    add_marker: bool,
    findings: &Catalog,
    workspace: &WorkspaceContext,
) -> Result<SettledHeadings> {
    let root: &Path = &config.root;
    let walked = walk_scannable_files_reporting(config.schema(), config.frame(), Some(root), true)?;
    let excluded = fmt_excluded(config)?;
    let targets = ShorthandTargets::new(config.frame(), Some(findings), Some(workspace));
    let files = walked
        .files
        .iter()
        .filter(|path| !walked.outside_root.contains(*path));
    Ok(settle_headings(
        config,
        files,
        &excluded,
        add_marker,
        findings,
        Some(workspace),
        &targets,
    ))
}
