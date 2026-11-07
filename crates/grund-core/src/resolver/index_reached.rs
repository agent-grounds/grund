//! The name an index is met under, which every link `fmt` writes into it is relative to
//! (§AR-scanner.1.8).

use std::borrow::Cow;
use std::path::Path;

use crate::config::Frame;
use crate::model::{Catalog, Id, paired_declarations};

/// The name `fmt` meets this index under, which a link it writes is relative to
/// (§AR-scanner.1.8): the one listed first, or, for an index that does not exist
/// yet, the same file beside the declaration when a directory alias reaches both. A tree with no
/// second name reads no path here.
pub(super) fn index_as_reached<'a>(
    index: &'a Path,
    id: &Id,
    frame: Frame<'_>,
    findings: &Catalog,
) -> Cow<'a, Path> {
    let spellings = &frame.run.spellings;
    if !spellings.has_second_names() {
        return Cow::Borrowed(index);
    }
    if let Some(first) = spellings.names_of(index).into_iter().next() {
        return Cow::Owned(first);
    }
    let home = findings.declarations.get(id).and_then(|decls| {
        paired_declarations(decls)
            .sole()
            .map(|sole| sole.record.file.clone())
    });
    let (Some(home), Some(parent), Some(name)) = (home, index.parent(), index.file_name()) else {
        return Cow::Borrowed(index);
    };
    let physical_parent = spellings.resolve(parent);
    home.ancestors()
        .skip(1)
        .find(|ancestor| spellings.resolve(ancestor) == physical_parent)
        .map_or(Cow::Borrowed(index), |ancestor| {
            Cow::Owned(ancestor.join(name))
        })
}
