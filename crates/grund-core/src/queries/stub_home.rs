//! The declaration a stub pairs with (§FS-show.2.3.7), shared by show's body and
//! section refusal (§FS-show.2.2.2) and by refs' refusals of a stub's ID, bare or
//! with a section (§FS-refs.4.1), so the record a coordinate reads and the record
//! it is refused from are one record.

use std::borrow::Cow;
use std::path::Path;

use super::ambiguity::ambiguous_id_refusal;
use super::show_query::ShowQueryError;
use crate::config::Config;
use crate::model::{Declaration, Id, TextOverlays, paths_same_location};
use crate::resolver::target_records;
use crate::scanner::scan_reads_target;

/// The record whose body, sections and anchors a query on `decl` reads
/// (§FS-show.2.3.7): `decl` itself unless it is a stub, and otherwise the
/// declaration of `id` in the stub's target `file`, found by the ID on whichever
/// line it sits and never at the stub's own line. That is the scanned record
/// when the walk reached `file`. When it did not, it is the record the scanner's
/// own pass over `file` produces, read from the editor's overlay where there is
/// one, by the resolver's reading that `check`'s section lookup takes too
/// (§AR-resolver.5, §FS-check.3.2.1).
///
/// An unscanned target holding two inline declarations of `id` is refused as
/// `ambiguous ID` at their sites, as the scanned tree refuses it (§FS-show.2.2.1),
/// rather than read from the first.
///
/// A target that cannot be read, or whose pass yields no inline declaration of
/// `id`, leaves the stub's own record. So does a target the scan does not read, by
/// its name or its extension, however many headings of `id` it holds: it declares
/// nothing (§FS-declarations.checks.broken-stub.3). refs makes no stub check, so
/// every refs query on a stub's ID, bare or with a section, reaches this for a
/// broken stub and lists its citations (§FS-refs.4.1). `show` has already refused a
/// broken stub (§FS-show.2.3.4), judged on this same overlay-first text
/// (§FS-declarations.checks.broken-stub.1) of a file the scan reads, so it reaches
/// this only where that line test finds a declaration the scanner's pass does not
/// record. The two read fences alike (§FS-declarations.checks.broken-stub.2): a
/// heading inside a fenced block of a Markdown target declares nothing to either.
pub(super) fn stub_home<'a>(
    config: &Config,
    path_config: &Config,
    decls: &'a [Declaration],
    decl: &'a Declaration,
    file: &Path,
    id: &Id,
    overlays: &TextOverlays,
) -> Result<Cow<'a, Declaration>, ShowQueryError> {
    // §FS-declarations.checks.broken-stub.3: judged by the name the stub wrote, before
    // a record at the path it resolves to is taken.
    if !decl.is_stub || !scan_reads_target(file, config) {
        return Ok(Cow::Borrowed(decl));
    }
    if let Some(scanned) = decls
        .iter()
        .find(|other| paths_same_location(&other.file, file))
    {
        return Ok(Cow::Borrowed(scanned));
    }
    // §FS-show.2.2.2.2: the reading `check` takes a stub's sections from (§FS-check.3.2.1).
    let mut homes = target_records(file, config, overlays)
        .remove(id)
        .unwrap_or_default();
    // §FS-show.2.3.7, §FS-show.2.2.1: two homes in the target refuse as if scanned.
    if let Some(refusal) = ambiguous_id_refusal(config, path_config, &homes, id) {
        return Err(refusal);
    }
    Ok(homes.pop().map_or(Cow::Borrowed(decl), Cow::Owned))
}
