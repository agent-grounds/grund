//! Whether an edit left one file's declarations as a catalog holds them (§FS-lsp.responsiveness.4,
//! §AR-lsp.3.2): the file read again by `scan_file_again`'s one step and behind its read predicate
//! (§AR-system.2.8.1), and the declarations that reading gives held to the ones the catalog holds
//! for the file. It reads the file and nothing else, so a keystroke's judgement costs the document,
//! never the tree.

use std::path::Path;

use super::tree::read_file_again;
use crate::config::{Frame, Schema};
use crate::model::{Catalog, Declaration, Id, scanned_path_key};
use crate::workspace::WorkspaceCitationTarget;

/// One declaration as a reading gives it, less where it sits: an edit above a heading moves its
/// line and its body span and leaves the declaration what it was.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Standing<'c> {
    id: &'c Id,
    authored: Option<&'c str>,
    title: Option<&'c str>,
    level: usize,
    /// A stub, a case or a value is judged by what lies outside the file too, so neither side
    /// may hold one for the judgement to say anything but "changed".
    is_stub: bool,
    case: bool,
    value: Option<bool>,
    front_matter: bool,
    sections: Vec<(&'c str, &'c str, usize, bool)>,
    duplicates: Vec<&'c str>,
}

impl<'c> Standing<'c> {
    fn of(decl: &'c Declaration) -> Self {
        Self {
            id: &decl.id,
            authored: decl.authored_id.as_deref(),
            title: decl.title.as_deref(),
            level: decl.heading_level,
            is_stub: decl.is_stub,
            case: decl.e2e_case.is_some(),
            value: decl.value_valid,
            front_matter: decl.front_matter.is_some(),
            sections: decl
                .sections
                .iter()
                .map(|(path, section)| {
                    (
                        path.as_str(),
                        section.title.as_str(),
                        section.heading_level,
                        section.relative,
                    )
                })
                .collect(),
            duplicates: decl
                .duplicate_sections
                .iter()
                .map(|(path, _)| path.as_str())
                .collect(),
        }
    }

    fn judged(&self) -> bool {
        !self.is_stub && !self.case && self.value.is_none()
    }
}

/// Whether reading `file` again, on the text the frame holds for it, gives the declarations
/// `before` holds for it (§FS-lsp.responsiveness.4). `file` is spelled as `before`'s walk read
/// it. Conservative and never wrong: a file the walk did not read, a reading that fails, and a
/// file whose declarations hold a stub, a case or a value on either side all answer `false`,
/// because the one-file reading cannot judge them exactly (§AR-lsp.3.2).
pub(crate) fn file_again_keeps_declarations(
    before: &Catalog,
    file: &Path,
    schema: &Schema,
    frame: Frame<'_>,
    workspace_targets: &[WorkspaceCitationTarget],
) -> bool {
    // §AR-system.2.8.1: the walk's own gate, with "would the walk read it" answered no, since
    // asking reads the ignore files and a file no walk read counts as changed here.
    let Ok(Some(read)) = read_file_again(before, file, schema, frame, workspace_targets, || false)
    else {
        return false;
    };
    let (was, now) = (standing(before, file), standing(&read, file));
    was.iter().chain(&now).all(Standing::judged) && was == now
}

/// The declarations `catalog` holds for `file`, in one order whatever order the reading met them.
fn standing<'c>(catalog: &'c Catalog, file: &Path) -> Vec<Standing<'c>> {
    let key = scanned_path_key(file);
    let mut held: Vec<Standing<'c>> = catalog
        .declarations
        .values()
        .flatten()
        // The name first, so a keystroke builds no key for the files it did not edit.
        .filter(|decl| decl.file.file_name() == file.file_name())
        .filter(|decl| scanned_path_key(&decl.file) == key)
        .map(Standing::of)
        .collect();
    held.sort();
    held
}
