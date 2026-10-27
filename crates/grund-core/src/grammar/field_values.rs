//! What a declaration's field says (§FS-fields.identity.name,
//! §FS-fields.content.one-of): the one reader the field checks, `list` and
//! `list --size` share, so a whose clause selects in a listing exactly the
//! declarations the field check reads as holding its value (§FS-rules.5.2,
//! §AR-system.principle). The rule engine reads the same display name off the
//! `chapter` fact, which `section_display_name` fills too (§FS-rules.5.1.1).

use std::borrow::Cow;

use super::anchors::section_display_name;
use crate::model::{Declaration, SectionInfo};

/// The display name of the section at `path`, where it is a direct section of
/// its declaration, which only a field can be (§FS-fields.identity.name): the
/// lead is not one, and a nested path is not. A front-matter unit is no chapter
/// (§FS-fields.front-matter.findings), and so no field a whose clause reads
/// (§FS-rules.5.2).
pub(crate) fn direct_section_value<'a>(path: &str, info: &'a SectionInfo) -> Option<Cow<'a, str>> {
    (!path.contains('.') && !info.in_front_matter())
        .then(|| section_display_name(&info.title, path, info.relative))
}

/// The value of `declaration`'s field `name`: the display name of its first
/// direct section of that name, a later duplicate not counting
/// (§FS-fields.identity.name); `None` where it has no such section.
pub(crate) fn field_value<'a>(declaration: &'a Declaration, name: &str) -> Option<Cow<'a, str>> {
    let info = declaration.sections.get(name)?;
    direct_section_value(name, info)
}
