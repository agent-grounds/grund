//! What the per-file pass reads off a declaration line once it has matched one
//! (§AR-scanner.2.1): whether it is a stub and where its link points, its title,
//! and the hand-off of the declaration it closes. A declaration and a near miss
//! read these the same way (§FS-config.3.2.5), so they are read once here.

use std::path::PathBuf;

use crate::grammar::STUB_LINK_HEADING;
use crate::model::{Catalog, Declaration};

/// A declaration line's `tail` after its ID: whether it is a stub, the file a
/// stub's link names, and the title of a declaration that is not one
/// (§FS-declarations.stubs, §FS-declarations.line). Only a Markdown line under
/// `docs/` can be a stub.
pub(super) fn stub_and_title(
    tail: &str,
    stub_position: bool,
) -> (bool, Option<PathBuf>, Option<String>) {
    if stub_position && let Some(link_caps) = STUB_LINK_HEADING.captures(tail) {
        let defined_in = PathBuf::from(link_caps.name("path").unwrap().as_str());
        return (true, Some(defined_in), None);
    }
    let trimmed = tail.trim_start();
    let trimmed = trimmed.strip_prefix(':').unwrap_or(trimmed).trim();
    (
        false,
        None,
        (!trimmed.is_empty()).then(|| trimmed.to_string()),
    )
}

/// Hand the declaration the pass was filling to `findings`: a new declaration
/// line, a near miss, or the end of the file closes it.
pub(super) fn close_declaration(current: &mut Option<Declaration>, findings: &mut Catalog) {
    if let Some(decl) = current.take() {
        findings
            .declarations
            .entry(decl.id.clone())
            .or_default()
            .push(decl);
    }
}
