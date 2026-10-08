use std::collections::BTreeMap;
use std::path::Path;

use super::value_mismatch::{rendered_value_path, value_mismatch};
use super::value_roots::root_binding_finding;
use crate::config::{Config, kind_uses_values, kind_value_chapter};
use crate::grammar::render_id;
use crate::model::{
    CheckReport, Declaration, Diagnostic, EmbeddedValueRoot, Findings, Id, Site, ValueBinding,
    is_stub_for_inline_decl, value_binding_section_ends_in_coordinate, value_components_equal,
};
use crate::resolver::{WorkspaceCheckTarget, home_as_scanned};

/// The independent explicit-value checker pass (§AR-checker.2.18,
/// §FS-values.5). It consumes scanner records, resolves through the same
/// workspace catalog as ordinary citations, and compares only one valid,
/// unique target: a numbered component, or a root's whole component run.
pub(super) fn check_values(
    findings: &Findings,
    config: &Config,
    path_config: &Config,
    workspace: &BTreeMap<String, WorkspaceCheckTarget<'_>>,
    report: &mut CheckReport,
) {
    for site in &findings.invalid_value_declarations {
        if site.id.as_ref().is_some_and(|id| {
            findings
                .declarations
                .get(id)
                .is_some_and(|decls| value_homes(decls, &config.root).len() > 1)
        }) {
            continue;
        }
        report.errors.push(Diagnostic {
            code: "invalid-value-declaration",
            path: Some(site.file.clone()),
            line: Some(site.line),
            column: site.column,
            message: match &site.id {
                Some(id) => format!(
                    "invalid value declaration for {}: {}",
                    render_id(&config.grammar, id),
                    site.message
                ),
                None => format!("invalid value declaration: {}", site.message),
            },
            sites: Vec::new(),
            authority: Vec::new(),
        });
    }
    for site in &findings.invalid_value_bindings {
        let target = match site.binding_namespace.as_deref() {
            Some(alias) => workspace.get(alias).map(|target| WorkspaceCheckTarget {
                findings: target.findings,
                config: target.config,
            }),
            None => Some(WorkspaceCheckTarget { findings, config }),
        };
        if site.id.as_ref().is_some_and(|id| {
            !target.is_some_and(|target| {
                binding_target_reports_invalid_attempt(
                    target.findings,
                    target.config,
                    id,
                    site.binding_section.as_deref(),
                )
            })
        }) {
            continue;
        }
        report.errors.push(Diagnostic {
            code: "invalid-value-binding",
            path: Some(site.file.clone()),
            line: Some(site.line),
            column: site.column,
            message: format!("invalid value binding: {}", site.message),
            sites: Vec::new(),
            authority: Vec::new(),
        });
    }
    for binding in &findings.value_bindings {
        let target = match binding.namespace.as_deref() {
            Some(alias) => workspace.get(alias).map(|target| WorkspaceCheckTarget {
                findings: target.findings,
                config: target.config,
            }),
            None => Some(WorkspaceCheckTarget { findings, config }),
        };
        let Some(target) = target else { continue };
        match binding_aim(target.findings, target.config, binding) {
            BindingAim::Component {
                declaration,
                section,
            } => {
                let Some(info) = declaration.sections.get(section) else {
                    continue;
                };
                let Some(declared) = info.value.as_ref() else {
                    continue;
                };
                if value_components_equal(&binding.authored, declared) {
                    continue;
                }
                report.errors.push(value_mismatch(
                    binding,
                    &rendered_value_path(target.config, binding, Some(section)),
                    &binding.authored,
                    declared,
                    Site {
                        path: declaration.file.clone(),
                        line: info.line,
                    },
                    path_config,
                ));
            }
            BindingAim::Root { declaration, path } => report.errors.extend(root_binding_finding(
                binding,
                target.config,
                declaration,
                path,
                path_config,
            )),
            BindingAim::Refused => report.errors.push(invalid_binding_diagnostic(binding)),
            BindingAim::Inert => {}
        }
    }
}

/// What a recognized binding's cited path names once it resolves
/// (§FS-values.5.1). Only `Component` and `Root` reach comparison.
pub(super) enum BindingAim<'a> {
    /// One immediate component of one valid root (§FS-values.3.1).
    Component {
        declaration: &'a Declaration,
        section: &'a str,
    },
    /// A valid root aimed at whole; `path` is its section path, or `None` for a
    /// whole declaration (§FS-values.3.1.2).
    Root {
        declaration: &'a Declaration,
        path: Option<&'a str>,
    },
    /// An attempted binding §FS-values.3.1.1 refuses with the grammar message.
    Refused,
    /// Prose, or a site an earlier finding owns, so nothing is compared.
    Inert,
}

/// Read a binding's cited path one way or the other (§FS-values.5.1): a path that
/// resolves to a value root is root-aimed, and one whose parent does and whose
/// final numeric segment names an immediate component is component-aimed. No
/// path reads both ways over valid authority, and an invalid root, a dangling or
/// duplicate declaration, or a missing component leaves the site to the finding
/// that owns it, so it is `Inert` here.
pub(super) fn binding_aim<'a>(
    findings: &'a Findings,
    config: &Config,
    binding: &'a ValueBinding,
) -> BindingAim<'a> {
    // §FS-values.5.1: compared as if a stub's target were scanned (§FS-check.3.2.1).
    let homes: Vec<&Declaration> = findings
        .declarations
        .get(&binding.id)
        .map(|decls| value_homes(decls, &config.root))
        .unwrap_or_default()
        .into_iter()
        .map(|home| home_as_scanned(findings, config, &binding.id, home))
        .collect();
    let declaration = match homes[..] {
        [home] => Some(home),
        _ => None,
    };
    let section = binding.section.as_deref();
    // A path that ends in a name has no coordinate, so it is a chapter root or
    // the attempt §FS-values.3.1.1 classifies, under the predicate an attempt
    // the scanner refused outright is held to.
    if let Some(section) = section.filter(|path| !value_binding_section_ends_in_coordinate(path)) {
        let root = declaration.and_then(|declaration| {
            embedded_root_for_binding(declaration, section).map(|found| (declaration, found))
        });
        return match root {
            Some((declaration, (root, EmbeddedBindingRelation::Root))) => {
                root_aim(declaration, Some(section), root.valid)
            }
            // §FS-values.5.1: a duplicate declaration owns the site of a root it declares.
            None if declaration.is_none()
                && homes.iter().any(|home| {
                    matches!(
                        embedded_root_for_binding(home, section),
                        Some((_, EmbeddedBindingRelation::Root))
                    )
                }) =>
            {
                BindingAim::Inert
            }
            _ if binding_target_reports_invalid_attempt(
                findings,
                config,
                &binding.id,
                Some(section),
            ) =>
            {
                BindingAim::Refused
            }
            _ => BindingAim::Inert,
        };
    }
    let Some(declaration) = declaration else {
        return BindingAim::Inert;
    };
    if declaration.value_valid.is_some() {
        // §FS-values.2.1: whole-declaration authority owns its whole section
        // tree, so the only component is one top-level coordinate.
        let valid = declaration.value_valid == Some(true);
        return match section {
            None => root_aim(declaration, None, valid),
            Some(section) if !section.contains('.') && valid => component_aim(declaration, section),
            Some(section) if !section.contains('.') => BindingAim::Inert,
            Some(_) => BindingAim::Refused,
        };
    }
    // §FS-values.3.1.1: a bare citation of a declaration that is not a value is
    // ordinary prose, whatever its kind.
    let Some(section) = section else {
        return BindingAim::Inert;
    };
    match embedded_root_for_binding(declaration, section) {
        Some((root, EmbeddedBindingRelation::Root)) => {
            root_aim(declaration, Some(section), root.valid)
        }
        Some((_, EmbeddedBindingRelation::ImmediateComponent)) => {
            component_aim(declaration, section)
        }
        Some((_, EmbeddedBindingRelation::BelowComponent)) => BindingAim::Refused,
        Some((_, EmbeddedBindingRelation::InvalidImmediateComponent)) | None => BindingAim::Inert,
    }
}

/// §FS-values.5.1: an invalid root suppresses the comparison and the space
/// refusal alike, so it reports its declaration error alone.
fn root_aim<'a>(
    declaration: &'a Declaration,
    path: Option<&'a str>,
    valid: bool,
) -> BindingAim<'a> {
    if valid {
        BindingAim::Root { declaration, path }
    } else {
        BindingAim::Inert
    }
}

/// §FS-values.5.1: a duplicated coordinate is the ordinary duplicate-section
/// finding's site, not a comparison.
fn component_aim<'a>(declaration: &'a Declaration, section: &'a str) -> BindingAim<'a> {
    if declaration
        .duplicate_sections
        .iter()
        .any(|(duplicate, _)| duplicate == section)
    {
        BindingAim::Inert
    } else {
        BindingAim::Component {
            declaration,
            section,
        }
    }
}

#[derive(Clone, Copy)]
enum EmbeddedBindingRelation {
    /// The path is the root's own (§FS-values.3.1.2).
    Root,
    ImmediateComponent,
    InvalidImmediateComponent,
    /// A section below one of the root's components (§FS-values.3.1.1).
    BelowComponent,
}

/// Find the existing marked section whose path owns this binding. Invalid roots
/// still count as authority for syntax classification, but only a valid root's
/// immediate child reaches comparison (§FS-values.3.1.1, §FS-values.5.1).
fn embedded_root_for_binding<'a>(
    declaration: &'a Declaration,
    section: &str,
) -> Option<(&'a EmbeddedValueRoot, EmbeddedBindingRelation)> {
    let (root_path, root) = declaration
        .sections
        .iter()
        .filter_map(|(root_path, info)| {
            let root = info.value_root.as_ref()?;
            (section == root_path || section.starts_with(&format!("{root_path}.")))
                .then_some((root_path, root))
        })
        // Nested invalid roots still own their immediate children. Choosing the
        // longest path first prevents an outer root from manufacturing a
        // secondary binding error (§FS-values.5.1).
        .max_by_key(|(root_path, _)| root_path.split('.').count())?;
    if section == root_path {
        return Some((root, EmbeddedBindingRelation::Root));
    }
    let remainder = section.strip_prefix(&format!("{root_path}."))?;
    let relation = if remainder.contains('.') {
        EmbeddedBindingRelation::BelowComponent
    } else if root.valid {
        EmbeddedBindingRelation::ImmediateComponent
    } else {
        EmbeddedBindingRelation::InvalidImmediateComponent
    };
    Some((root, relation))
}

fn binding_target_reports_invalid_attempt(
    findings: &Findings,
    config: &Config,
    id: &Id,
    section: Option<&str>,
) -> bool {
    if kind_uses_values(config, &id.kind) {
        return true;
    }
    let Some(section) = section else { return false };
    // §FS-values.3.1.1: one predicate, so check's refusal set and the set
    // fmt protects (§FS-values.8) cannot drift apart.
    binding_aims_at_embedded_value_authority(findings, config, id, section)
}

/// Whether `section` is the declaration's own declared value chapter
/// (§FS-values.2.5). A same-named chapter nested deeper is not one, because the
/// path the key names is the declaration's direct chapter and nothing else.
fn binding_aims_at_declared_chapter(
    config: &Config,
    id: &Id,
    declaration: &Declaration,
    section: &str,
) -> bool {
    kind_value_chapter(config, &id.kind) == Some(section)
        && declaration.sections.contains_key(section)
}

/// Whether a delimited form aims at embedded value authority at all — a value
/// root's own heading whichever authority made it, a section below one of its
/// components, or the kind's declared chapter heading. `check` refuses every
/// one of these (§FS-values.3.1.1) and `fmt --cross-refs` leaves every one of
/// their citation bytes alone, because a link would carry the refusal away
/// (§FS-values.8, §FS-values.9.1). The single predicate behind both, so the
/// refused set and the protected set cannot drift apart; it is also the part of
/// the refusal that [`value_binding_section_shape_is_valid`] cannot see, a
/// chapter root's path carrying names and a marked root's subtree reaching
/// below a component.
pub(crate) fn binding_aims_at_embedded_value_authority(
    findings: &Findings,
    config: &Config,
    id: &Id,
    section: &str,
) -> bool {
    declarations_as_scanned(findings, config, id).any(|declaration| {
        binding_aims_at_declared_chapter(config, id, declaration, section)
            || embedded_root_for_binding(declaration, section).is_some_and(|(_, relation)| {
                !matches!(relation, EmbeddedBindingRelation::InvalidImmediateComponent)
            })
    })
}

/// Whether a binding-shaped citation has value authority behind it: a kind
/// with `values = true`, which a bare ID aims at whole (§FS-values.3.1.2), or a
/// marked or chapter root that owns the cited path (§FS-values.2.4,
/// §FS-values.2.5).
pub(crate) fn binding_target_has_any_value_authority(
    findings: &Findings,
    config: &Config,
    id: &Id,
    section: Option<&str>,
) -> bool {
    kind_uses_values(config, &id.kind)
        || section.is_some_and(|section| {
            declarations_as_scanned(findings, config, id)
                .any(|declaration| embedded_root_for_binding(declaration, section).is_some())
        })
}

/// Every declaration of `id`, a stub's read from its target outside the walk: the
/// value authority an attempt is classified by and `fmt` protects, as it would be
/// were the target scanned (§FS-check.3.2.1, §FS-values.3.1.1, §FS-values.8).
fn declarations_as_scanned<'a>(
    findings: &'a Findings,
    config: &'a Config,
    id: &'a Id,
) -> impl Iterator<Item = &'a Declaration> {
    findings
        .declarations
        .get(id)
        .into_iter()
        .flatten()
        .map(move |declaration| home_as_scanned(findings, config, id, declaration))
}

fn invalid_binding_diagnostic(binding: &ValueBinding) -> Diagnostic {
    Diagnostic {
        code: "invalid-value-binding",
        path: Some(binding.file.clone()),
        line: Some(binding.line),
        column: Some(binding.column),
        message: "invalid value binding: value binding must be exactly `literal` (marker-prefixed full value ID with one positive numeric field)".to_string(),
        sites: Vec::new(),
    authority: Vec::new(),}
}

fn value_homes<'a>(decls: &'a [Declaration], root: &Path) -> Vec<&'a Declaration> {
    decls
        .iter()
        .filter(|decl| !is_stub_for_inline_decl(root, decl, decls))
        .collect()
}
