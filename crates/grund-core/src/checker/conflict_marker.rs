//! The conflict-marker rule (§FS-declarations.checks.conflict-marker), in a file of
//! its own beside the other rule families (§AR-core-module-layout.1). The blocks
//! are the scanner's record (§AR-scanner.2.9); this is the one function that
//! turns a block into its finding, which `check` reports and the proof
//! constructors of `fmt` refuse with (§FS-fmt.7.4), so the two name one block alike.

use crate::config::Frame;
use crate::model::remedies;
use std::collections::BTreeMap;

use crate::grammar::{Grammar, render_catalog_id};
use crate::model::{Catalog, CheckReport, Code, ConflictBlock, Declaration, Diagnostic, Id, Site};

/// §FS-declarations.checks.conflict-marker: one error per recorded block, an error
/// from the release that adds it with no warning first (§DF-conflicted-declaration).
pub(super) fn check_conflict_blocks(catalog: &Catalog, frame: Frame<'_>, report: &mut CheckReport) {
    report.errors.extend(
        catalog
            .conflict_blocks
            .iter()
            .map(|block| conflict_marker_finding(block, frame.grammar(), &catalog.declarations)),
    );
}

/// The `conflict-marker` finding for one block, located at its first line, with
/// its first and last lines as `sites`, one site for a block of one line. Each
/// owner is spelled from `declarations`, the catalog that holds it, with its
/// authored digits, as a `duplicate` finding spells its ID
/// (§FS-declarations.checks.conflict-marker, §FS-config.3.2.8).
pub(crate) fn conflict_marker_finding(
    block: &ConflictBlock,
    grammar: &Grammar,
    declarations: &BTreeMap<Id, Vec<Declaration>>,
) -> Diagnostic {
    let owners: Vec<String> = block
        .owners
        .iter()
        .map(|id| render_catalog_id(grammar, declarations, id))
        .collect();
    let named = match owners.as_slice() {
        [] => "outside any declaration".to_string(),
        [only] => format!("in {only}"),
        [rest @ .., last] => format!("in {} and {last}", rest.join(", ")),
    };
    let (lines, sites) = if block.first == block.last {
        (format!("line {}", block.first), vec![block.first])
    } else {
        let lines = format!("lines {}\u{2013}{}", block.first, block.last);
        (lines, vec![block.first, block.last])
    };
    Diagnostic {
        code: Code::CONFLICT_MARKER,
        path: Some(block.file.clone()),
        line: Some(block.first),
        column: None,
        message: format!(
            "unresolved merge conflict {named}, {lines}: {}",
            remedies::conflict_marker()
        ),
        sites: sites
            .into_iter()
            .map(|line| Site {
                path: block.file.clone(),
                line,
            })
            .collect(),
        authority: Vec::new(),
    }
}
