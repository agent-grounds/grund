//! What a `value-mismatch` says: the coordinate or root it names, the canonical
//! text, and the kind clause a mixed-kind pair appends (§FS-values.5.2,
//! §FS-values.5.2.1, §FS-values.5.2.2).

use crate::config::{Display, Frame, Schema};
use crate::grammar::render_qualified_id;
use crate::model::{Diagnostic, Site, ValueBinding, ValueComponent, ValueComponentKind};

/// A root or one coordinate as a finding names it: the qualified ID, then the
/// separator and the section path when there is one (§FS-values.3.1.2,
/// §FS-values.5.2).
pub(super) fn rendered_value_path(
    schema: &Schema,
    frame: Frame<'_>,
    binding: &ValueBinding,
    path: Option<&str>,
) -> String {
    let id = render_qualified_id(frame.grammar(), binding.namespace.as_deref(), &binding.id);
    match path {
        Some(path) => format!("{id}{}{path}", schema.ids.section_separator),
        None => id,
    }
}

/// One `value-mismatch` in the canonical text, carrying the one declaration site
/// it names in text and in `sites` alike (§FS-values.5.2, §FS-output-shapes.1.1).
/// The kinds follow only when the two sides differ in kind, because the two
/// sides can then print the same bytes (§FS-values.5.2.1).
pub(super) fn value_mismatch(
    binding: &ValueBinding,
    coordinate: &str,
    bound: &ValueComponent,
    declared: &ValueComponent,
    site: Site,
    display: Display<'_>,
) -> Diagnostic {
    let kinds = mixed_kind_clause(bound.kind, declared.kind);
    value_mismatch_text(
        binding,
        coordinate,
        &bound.decoded,
        &declared.decoded,
        &kinds,
        site,
        display,
    )
}

/// The canonical mismatch text itself, for a caller that has no pair of
/// components to take the kinds from (§FS-values.5.2.2).
pub(super) fn value_mismatch_text(
    binding: &ValueBinding,
    coordinate: &str,
    bound: &str,
    declared: &str,
    kinds: &str,
    site: Site,
    display: Display<'_>,
) -> Diagnostic {
    let declared_site = format!("{}:{}", display.path(&site.path), site.line);
    Diagnostic {
        code: "value-mismatch",
        path: Some(binding.file.clone()),
        line: Some(binding.line),
        column: Some(binding.column),
        message: format!(
            "value mismatch for {coordinate}: bound `{bound}`, declared `{declared}` at {declared_site}{kinds}"
        ),
        sites: vec![site],
        authority: Vec::new(),
    }
}

/// What a mismatch of two different kinds appends to the canonical text
/// (§FS-values.5.2.1). Identical-looking sides are the whole of the inequality
/// there, so the kinds are named in the order the line already named its sides;
/// a same-kind mismatch appends nothing and keeps that text to the byte.
fn mixed_kind_clause(bound: ValueComponentKind, declared: ValueComponentKind) -> String {
    if bound == declared {
        return String::new();
    }
    format!(
        " \u{2014} bound is a {}, declared is a {}; a number and a string are never equal",
        kind_word(bound),
        kind_word(declared)
    )
}

/// The one word a message uses for a component kind (§FS-values.5.2.1).
fn kind_word(kind: ValueComponentKind) -> &'static str {
    match kind {
        ValueComponentKind::Number => "number",
        ValueComponentKind::String => "string",
    }
}
