//! What a `value-mismatch` says: the coordinate or root it names, the canonical
//! text, and the kind clause a mixed-kind pair appends (§FS-values.5.2,
//! §FS-values.5.2.1, §FS-values.5.2.2), and the staged form a root-aimed one
//! takes until its window closes (§FS-errors.3.9).

use crate::config::{Display, Schema};
use crate::model::remedies;
use crate::model::{Code, Diagnostic, Site, ValueBinding, ValueComponent, ValueComponentKind};

/// §FS-errors.3.9: the suffix a staged root-aimed mismatch ends with, naming the
/// release that drops the legacy prefix, on one line so the release guard reads
/// it (§FS-distribution.4.2.2).
const ROOT_WORDING_CLAUSE: &str = "; this wording changes in grund 0.22.0";

/// A root or one coordinate as a finding names it: the qualified ID, then the
/// separator and the section path when there is one (§FS-values.3.1.2,
/// §FS-values.5.2).
pub(super) fn rendered_value_path(
    schema: &Schema,
    binding: &ValueBinding,
    spelling: &str,
    path: Option<&str>,
) -> String {
    let id = super::support::qualify(binding.namespace.as_deref(), spelling);
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
    let kinds = mixed_kind_clause(None, bound.kind, declared.kind);
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
    Diagnostic {
        code: Code::VALUE_MISMATCH,
        path: Some(binding.file.clone()),
        line: Some(binding.line),
        column: Some(binding.column),
        message: format!(
            "{}; {}",
            mismatch_message(coordinate, bound, declared, &site, kinds, display),
            remedies::value_mismatch()
        ),
        sites: vec![site],
        authority: Vec::new(),
    }
}

/// The root a root-aimed mismatch names and the one component it reports,
/// each already rendered as a finding names it (§FS-values.5.2.2).
pub(super) struct RootComponent<'a> {
    /// The root as cited, rendered as the part-count branch renders it.
    pub(super) root: &'a str,
    /// The first unequal component's section path after the ID.
    pub(super) section_path: &'a str,
    /// That component by its own coordinate, as a component-aimed binding names it.
    pub(super) coordinate: &'a str,
}

/// One `value-mismatch` at a root-aimed binding whose literal splits into as
/// many parts as the root has components: the root as cited and the literal
/// whole against the joined components, naming the first unequal component by
/// its section path and carrying that component's one declaration site
/// (§FS-values.5.2.2). Its kind clause names that component, since the whole
/// literal and the joined components can print the same bytes either way
/// (§FS-values.5.2.1). Until 0.22.0 the component's canonical text stays whole
/// in front of it, a verbatim prefix for a consumer that matches it
/// (§FS-errors.3.9).
pub(super) fn root_value_mismatch(
    binding: &ValueBinding,
    at: RootComponent<'_>,
    bound: &ValueComponent,
    declared: &ValueComponent,
    joined: &str,
    site: Site,
    display: Display<'_>,
) -> Diagnostic {
    let legacy = value_mismatch(binding, at.coordinate, bound, declared, site, display);
    let kinds = mixed_kind_clause(Some(at.section_path), bound.kind, declared.kind);
    let whole = mismatch_message(
        &format!("{} (component {})", at.root, at.section_path),
        &binding.authored.decoded,
        joined,
        &legacy.sites[0],
        &kinds,
        display,
    );
    // §FS-errors.5.5.1: the remedy closes the whole message, after the wording clause.
    let remedy = format!("; {}", remedies::value_mismatch());
    let component = legacy
        .message
        .strip_suffix(&remedy)
        .unwrap_or(&legacy.message);
    Diagnostic {
        message: format!(
            "{component} \u{2014} whole binding: {whole}{ROOT_WORDING_CLAUSE}{remedy}"
        ),
        ..legacy
    }
}

/// The one shape every `value-mismatch` message takes: what it names, both
/// sides in backticks, and the declaration site, then any kind clause
/// (§FS-values.5.2, §FS-values.5.2.2).
fn mismatch_message(
    subject: &str,
    bound: &str,
    declared: &str,
    site: &Site,
    kinds: &str,
    display: Display<'_>,
) -> String {
    let declared_site = format!("{}:{}", display.path(&site.path), site.line);
    format!(
        "value mismatch for {subject}: bound `{bound}`, declared `{declared}` at {declared_site}{kinds}"
    )
}

/// What a mismatch of two different kinds appends to the canonical text
/// (§FS-values.5.2.1). Identical-looking sides are the whole of the inequality
/// there, so the kinds are named in the order the line already named its sides;
/// a same-kind mismatch appends nothing and keeps that text to the byte. A
/// root-aimed message passes the component's section path, so the clause says
/// which pair the kinds belong to.
fn mixed_kind_clause(
    component: Option<&str>,
    bound: ValueComponentKind,
    declared: ValueComponentKind,
) -> String {
    if bound == declared {
        return String::new();
    }
    let pair = component
        .map(|path| format!("component {path}: "))
        .unwrap_or_default();
    format!(
        " \u{2014} {pair}bound is a {}, declared is a {}; a number and a string are never equal",
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
