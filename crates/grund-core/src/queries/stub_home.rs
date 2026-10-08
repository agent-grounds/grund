//! The declaration a stub pairs with (§FS-show.2.3.7), shared by show's body and
//! the section refusals show and refs make (§FS-show.2.2.2, §FS-refs.4), so the
//! record a coordinate reads and the record it is refused from are one record.

use std::borrow::Cow;
use std::path::Path;

use crate::config::Config;
use crate::model::{Declaration, Id, TextOverlays, paths_same_location};
use crate::scanner::scan_unwalked_file;

/// The record whose body and sections a query on `decl` reads (§FS-show.2.3.7):
/// `decl` itself unless it is a stub; the scanned record in its target `file`
/// when the walk reached it; and otherwise the record the scanner's own pass
/// over `file` produces, read from the editor's overlay where there is one. The
/// target declaration is found by its ID, on whichever line it sits, and never
/// at the stub's own line.
///
/// A target that cannot be read, or that holds no inline declaration of `id`,
/// leaves the stub's own record. `show` has refused such a stub as broken before
/// it asks (§FS-show.2.3.4), so only refs' section refusal, which makes no stub
/// check, can see it.
pub(super) fn stub_home<'a>(
    config: &Config,
    decls: &'a [Declaration],
    decl: &'a Declaration,
    file: &Path,
    id: &Id,
    overlays: &TextOverlays,
) -> Cow<'a, Declaration> {
    if !decl.is_stub {
        return Cow::Borrowed(decl);
    }
    if let Some(scanned) = decls
        .iter()
        .find(|other| paths_same_location(&other.file, file))
    {
        return Cow::Borrowed(scanned);
    }
    scan_unwalked_file(file, config, overlays)
        .ok()
        .and_then(|mut findings| findings.declarations.remove(id))
        .and_then(|records| records.into_iter().find(|record| !record.is_stub))
        .map_or(Cow::Borrowed(decl), Cow::Owned)
}
