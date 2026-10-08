//! The declaration a stub pairs with (§FS-show.2.3.7), shared by show's body and
//! the section refusals show and refs make (§FS-show.2.2.2, §FS-refs.4), so the
//! record a coordinate reads and the record it is refused from are one record.

use std::borrow::Cow;
use std::path::Path;

use super::ambiguity::ambiguous_id_refusal;
use super::show_query::ShowQueryError;
use crate::config::Config;
use crate::model::{Declaration, Id, TextOverlays, paths_same_location};
use crate::scanner::scan_unwalked_file;

/// The record whose body, sections and anchors a query on `decl` reads
/// (§FS-show.2.3.7): `decl` itself unless it is a stub, and otherwise the
/// declaration of `id` in the stub's target `file`, found by the ID on whichever
/// line it sits and never at the stub's own line. That is the scanned record
/// when the walk reached `file`. When it did not, it is the record the scanner's
/// own pass over `file` produces, read from the editor's overlay where there is
/// one.
///
/// An unscanned target holding two inline declarations of `id` is refused as
/// `ambiguous ID` at their sites, as the scanned tree refuses it (§FS-show.2.2.1),
/// rather than read from the first.
///
/// A target that cannot be read, or whose pass yields no inline declaration of
/// `id`, leaves the stub's own record. refs' section refusal makes no stub check,
/// so it reaches this for a broken stub. `show` has already refused a broken
/// stub (§FS-show.2.3.4), but that check reads the disk, so it reaches this too
/// when an editor's overlay drops a declaration the disk still holds.
pub(super) fn stub_home<'a>(
    config: &Config,
    path_config: &Config,
    decls: &'a [Declaration],
    decl: &'a Declaration,
    file: &Path,
    id: &Id,
    overlays: &TextOverlays,
) -> Result<Cow<'a, Declaration>, ShowQueryError> {
    if !decl.is_stub {
        return Ok(Cow::Borrowed(decl));
    }
    if let Some(scanned) = decls
        .iter()
        .find(|other| paths_same_location(&other.file, file))
    {
        return Ok(Cow::Borrowed(scanned));
    }
    let mut homes: Vec<Declaration> = scan_unwalked_file(file, config, overlays)
        .ok()
        .and_then(|mut findings| findings.declarations.remove(id))
        .unwrap_or_default()
        .into_iter()
        .filter(|record| !record.is_stub)
        .collect();
    // §FS-show.2.3.7, §FS-show.2.2.1: two homes in the target refuse as if scanned.
    if let Some(refusal) = ambiguous_id_refusal(config, path_config, &homes, id) {
        return Err(refusal);
    }
    Ok(homes.pop().map_or(Cow::Borrowed(decl), Cow::Owned))
}
