use std::collections::BTreeMap;
use std::path::Path;

use crate::config::{Config, display_path, kind_uses_values, kind_value_chapter};
use crate::grammar::{render_id, render_qualified_id};
use crate::model::{
    CheckReport, Declaration, Diagnostic, EmbeddedValueRoot, Findings, Id, Site, ValueBinding,
    is_stub_for_inline_decl, value_components_equal,
};
use crate::resolver::WorkspaceCheckTarget;

/// The independent explicit-value checker pass (§AR-checker.2.18,
/// §FS-values.5). It consumes scanner records, resolves through the same
/// workspace catalog as ordinary citations, and compares only one valid,
/// unique numbered target.
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
        let Some(decls) = target.findings.declarations.get(&binding.id) else {
            continue;
        };
        let homes = value_homes(decls, &target.config.root);
        if homes.len() != 1 {
            continue;
        }
        let declaration = homes[0];
        let whole_authority = declaration.value_valid.is_some();
        let embedded = (!whole_authority)
            .then(|| embedded_root_for_binding(declaration, &binding.section))
            .flatten();
        let section = match embedded {
            Some((_, EmbeddedBindingRelation::ImmediateComponent)) => {
                let Some(section) = declaration.sections.get(&binding.section) else {
                    continue;
                };
                section
            }
            Some((_, EmbeddedBindingRelation::RootOrDescendant)) => {
                report.errors.push(invalid_binding_diagnostic(binding));
                continue;
            }
            Some((_, EmbeddedBindingRelation::InvalidImmediateComponent)) => continue,
            None if whole_authority && !binding.section.contains('.') => {
                if declaration.value_valid != Some(true) {
                    continue;
                }
                let Some(section) = declaration.sections.get(&binding.section) else {
                    continue;
                };
                section
            }
            None if whole_authority => {
                report.errors.push(invalid_binding_diagnostic(binding));
                continue;
            }
            None => continue,
        };
        if declaration
            .duplicate_sections
            .iter()
            .any(|(duplicate, _)| duplicate == &binding.section)
        {
            continue;
        }
        let Some(declared) = section.value.as_ref() else {
            continue;
        };
        if value_components_equal(&binding.authored, declared) {
            continue;
        }
        let coordinate = format!(
            "{}{}{}",
            render_qualified_id(
                &target.config.grammar,
                binding.namespace.as_deref(),
                &binding.id
            ),
            target.config.section_separator,
            binding.section
        );
        let declared_site = format!(
            "{}:{}",
            display_path(path_config, &declaration.file),
            section.line
        );
        report.errors.push(Diagnostic {
            code: "value-mismatch",
            path: Some(binding.file.clone()),
            line: Some(binding.line),
            column: Some(binding.column),
            message: format!(
                "value mismatch for {coordinate}: bound `{}`, declared `{}` at {declared_site}",
                binding.authored.decoded, declared.decoded
            ),
            sites: vec![Site {
                path: declaration.file.clone(),
                line: section.line,
            }],
        });
    }
}

#[derive(Clone, Copy)]
enum EmbeddedBindingRelation {
    ImmediateComponent,
    InvalidImmediateComponent,
    RootOrDescendant,
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
        return Some((root, EmbeddedBindingRelation::RootOrDescendant));
    }
    let remainder = section.strip_prefix(&format!("{root_path}."))?;
    let relation = if remainder.contains('.') {
        EmbeddedBindingRelation::RootOrDescendant
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
    findings
        .declarations
        .get(id)
        .into_iter()
        .flatten()
        .any(|declaration| {
            // §FS-values.3.1.1: the declared chapter heading is a refusal of its
            // own — the form aims at value authority even though the chapter is
            // not itself a root, so it may not fall back to ordinary prose.
            binding_aims_at_declared_chapter(config, id, declaration, section)
                || embedded_root_for_binding(declaration, section).is_some_and(|(_, relation)| {
                    !matches!(relation, EmbeddedBindingRelation::InvalidImmediateComponent)
                })
        })
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
/// one of these (§FS-values.3.1.1), and a chapter root's path carries names, so
/// this is the part of the refusal that [`value_binding_section_shape_is_valid`]
/// cannot see (§FS-values.8).
pub(crate) fn binding_aims_at_embedded_value_authority(
    findings: &Findings,
    config: &Config,
    id: &Id,
    section: &str,
) -> bool {
    findings
        .declarations
        .get(id)
        .into_iter()
        .flatten()
        .any(|declaration| {
            binding_aims_at_declared_chapter(config, id, declaration, section)
                || embedded_root_for_binding(declaration, section).is_some_and(|(_, relation)| {
                    !matches!(relation, EmbeddedBindingRelation::InvalidImmediateComponent)
                })
        })
}

pub(crate) fn binding_target_has_any_value_authority(
    findings: &Findings,
    config: &Config,
    id: &Id,
    section: &str,
) -> bool {
    kind_uses_values(config, &id.kind)
        || findings
            .declarations
            .get(id)
            .into_iter()
            .flatten()
            .any(|declaration| embedded_root_for_binding(declaration, section).is_some())
}

fn invalid_binding_diagnostic(binding: &ValueBinding) -> Diagnostic {
    Diagnostic {
        code: "invalid-value-binding",
        path: Some(binding.file.clone()),
        line: Some(binding.line),
        column: Some(binding.column),
        message: "invalid value binding: value binding must be exactly `literal` (marker-prefixed full value ID with one positive numeric field)".to_string(),
        sites: Vec::new(),
    }
}

fn value_homes<'a>(decls: &'a [Declaration], root: &Path) -> Vec<&'a Declaration> {
    decls
        .iter()
        .filter(|decl| !is_stub_for_inline_decl(root, decl, decls))
        .collect()
}
