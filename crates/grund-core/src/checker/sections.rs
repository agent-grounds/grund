//! The section-shape rule family: what `grund check` says about a declaration's
//! own citable headings, rather than about a citation of one.
//!
//! - **Heading level** (§FS-declarations.checks.section-heading-level) — the Markdown depth a heading writes must
//!   mirror the dotted path it claims, as strictly as `[id] section_heading_levels`
//!   asks (§FS-config.3.3.2).
//! - **Duplicate path** (§FS-declarations.checks.duplicate-section) — two headings claiming one path give a
//!   section citation two destinations, which is §FS-declarations.checks.duplicate's ambiguity one
//!   level down, and is reported rather than ranked (§DF-duplicate-section-path).
//! - **Outside declaration** (§FS-declarations.checks.section-outside-declaration) — a section-like heading rejected
//!   by the scanner's body-span post-pass is a located hard finding.
//!
//! They sit beside `report.rs` as one family because they read the same two
//! things and nothing else does: the recorded section map, and the
//! `duplicate_sections` list the scanner keeps beside it (§AR-scanner.2.2,
//! §AR-core-module-layout.1).

use crate::model::remedies;
use std::collections::BTreeMap;

use super::reference_scope::ScanScope;
use super::support::{heading_marks, section_depth};
use crate::config::{Frame, Schema};
use crate::grammar::render_catalog_id;
use crate::model::{
    Catalog, CheckReport, Code, Diagnostic, LANDED_CLAUSE, SectionHeadingOutsideDeclaration, Site,
    Switch, UnmarkedHeading, named_section_path, paired_declarations,
};
use crate::resolver::WorkspaceProject;
use crate::scanner::section_path_is_numeric;

/// The section-shape rules, as independent passes over the declarations
/// (§AR-checker.2.15). Order does not matter — the report is sorted before it is
/// printed (§FS-errors.4.1).
///
/// `schema` and `frame` are the project being checked (it owns the ID grammar
/// and the separator), and the frame's display is the one the printed report
/// renders paths against, so in a workspace a path named *inside* a message
/// points where the finding's own anchor points (§FS-config.3.6,
/// §FS-workspace.8.1).
///
/// Why a later item's doc comment and a stub's prose stay out of the duplicate-path
/// rule: `duplicate_sections` is already scoped to the declaration's own body
/// (§AR-scanner.2.2.5).
///
/// Why the anchor does not hang on the section map: the same insert that starts a
/// collision list fills that map, so the lookup always hits — but the finding is
/// written not to depend on it (§REQ-no-missed-citation). `lines` is non-empty
/// either way, since a path is in `colliding` only because a heading claimed it
/// twice.
pub(super) fn check_section_headings(
    findings: &Catalog,
    schema: &Schema,
    frame: Frame<'_>,
    report: &mut CheckReport,
) {
    report.errors.extend(
        findings
            .section_headings_outside_declarations
            .iter()
            .map(section_outside_declaration_diagnostic),
    );

    // §FS-declarations.checks.unmarked-heading.5 / §AR-checker.2.20: scanner-owned Markdown
    // candidates are errors since 0.16.0, independent of the marked-section heading-level mode.
    report
        .errors
        .extend(findings.unmarked_headings.iter().map(|heading| {
            let rendered_owner =
                render_catalog_id(frame.grammar(), &findings.declarations, &heading.owner);
            let separator = if named_section_path(&heading.suggested_path) {
                ":"
            } else {
                "."
            };
            let suggested_heading = format!(
                "{} {}{} {}",
                heading_marks(heading.heading_level),
                heading.suggested_path,
                separator,
                if heading.title.is_empty() {
                    "Untitled"
                } else {
                    heading.title.as_str()
                }
            );
            // §FS-declarations.checks.unmarked-heading.2.1: appended; the prefix reads as before.
            let relative = heading
                .relative_hint
                .as_ref()
                .map_or(String::new(), |hint| {
                    format!(
                        "; under {} a relative heading is written {}",
                        coordinate(&rendered_owner, &schema.ids.section_separator, &hint.parent),
                        hint.heading
                    )
                });
            // §FS-declarations.checks.unmarked-heading.2.2: append after the whole message.
            let field =
                field_heading_hint(heading, findings, schema, frame.version).unwrap_or_default();
            Diagnostic {
                code: Code::UNMARKED_HEADING,
                path: Some(heading.file.clone()),
                line: Some(heading.line),
                column: None,
                message: format!(
                    "unmarked heading inside {rendered_owner}; {}{LANDED_CLAUSE}{relative}{field}",
                    remedies::unmarked_heading(
                        &suggested_heading,
                        format!(
                            "{rendered_owner}{}{}",
                            schema.ids.section_separator, heading.suggested_path
                        )
                    ),
                ),
                sites: Vec::new(),
                authority: Vec::new(),
            }
        }));

    // §FS-declarations.checks.relative-section-parent: a relative heading with no usable parent.
    report
        .errors
        .extend(findings.relative_section_parents.iter().map(|heading| {
            let rendered_owner =
                render_catalog_id(frame.grammar(), &findings.declarations, &heading.owner);
            Diagnostic {
                code: Code::RELATIVE_SECTION_PARENT,
                path: Some(heading.file.clone()),
                line: Some(heading.line),
                column: None,
                message: format!(
                    "relative heading {} needs an H{} parent under {}, and there is none; {}",
                    heading.written,
                    heading.needed_level,
                    coordinate(
                        &rendered_owner,
                        &schema.ids.section_separator,
                        &heading.under
                    ),
                    remedies::relative_section_parent()
                ),
                sites: Vec::new(),
                authority: Vec::new(),
            }
        }));

    // §FS-declarations.checks.section-heading-level / §FS-config.3.3.2: in strict mode, the
    // Markdown heading level must mirror the dotted section depth so `## 1`, `### 1.1`, ...
    // communicate the same tree that `§ID.1.1` addresses.

    // §FS-config-v2.schema.1: v2's `heading_depth = "should"` is the `suggest` mode.
    if matches!(
        schema.ids.section_heading_levels.as_str(),
        "strict" | "warn" | "suggest"
    ) {
        report.armed.arm(Switch::HeadingDepth);
        let target = match schema.ids.section_heading_levels.as_str() {
            "strict" => &mut report.errors,
            "warn" => &mut report.warnings,
            _ => &mut report.suggestions,
        };
        for (id, decls) in &findings.declarations {
            for decl in decls {
                for (section_path, section) in &decl.sections {
                    // §FS-config.3.3.3: a relative heading's depth is its parent's plus one.
                    // §FS-fields.front-matter.coordinates: a front-matter coordinate has none.
                    if section.relative || section.in_front_matter() {
                        continue;
                    }
                    let expected_level = decl.heading_level + section_depth(section_path);
                    if section.heading_level != expected_level {
                        target.push(Diagnostic {
                            code: Code::SECTION_HEADING_LEVEL,
                            path: Some(decl.file.clone()),
                            line: Some(section.line),
                            column: None,
                            message: format!(
                                "section {}{}{} heading level mismatch: expected {} (level {}), found {} (level {}); {}",
                                render_catalog_id(frame.grammar(), &findings.declarations, id),
                                schema.ids.section_separator,
                                section_path,
                                heading_marks(expected_level),
                                expected_level,
                                heading_marks(section.heading_level),
                                section.heading_level,
                                remedies::section_heading_level()
                            ),
                            sites: Vec::new(),
                        authority: Vec::new(),});
                    }
                }
            }
        }
    }
    // §FS-declarations.checks.orphan-section / §AR-checker.2.17: every name-bearing path is
    // addressable only when each proper prefix exists in the same scanner-recorded map.
    if schema.ids.named_sections {
        report.armed.arm(Switch::NamedSections);
        for (id, decls) in &findings.declarations {
            for decl in decls {
                for (path, info) in &decl.sections {
                    if let Some(prefix) = decl.missing_section_prefix(path) {
                        report.errors.push(Diagnostic {
                            code: Code::ORPHAN_SECTION,
                            path: Some(decl.file.clone()),
                            line: Some(info.line),
                            column: None,
                            message: format!(
                                "orphan section {}{}{}: missing prefix {}{}{}; {}",
                                render_catalog_id(frame.grammar(), &findings.declarations, id),
                                schema.ids.section_separator,
                                path,
                                render_catalog_id(frame.grammar(), &findings.declarations, id),
                                schema.ids.section_separator,
                                prefix,
                                remedies::orphan_section()
                            ),
                            sites: Vec::new(),
                            authority: Vec::new(),
                        });
                    }
                }
            }
        }
    }
    // §FS-declarations.checks.duplicate-section: two headings in one declaration claiming one
    // dotted path give `§<ID>.<path>` two destinations — §FS-declarations.checks.duplicate's
    // ambiguity one level down, reported in its shape (§DF-duplicate-section-path.2.1).
    for (id, decls) in &findings.declarations {
        // §FS-declarations.checks.duplicate-section.2: judged on the records the ID pairs with,
        // at their lines; a scanned target is one record (§FS-declarations.stubs.target.5).
        for paired in paired_declarations(decls).iter() {
            let decl = paired.record;
            let mut colliding: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
            for (path, info) in &decl.duplicate_sections {
                colliding.entry(path.as_str()).or_default().push(info.line);
            }
            for (path, mut lines) in colliding {
                // The map holds the first heading (§AR-scanner.2.2.3), which is where
                // the finding anchors; the rest are named in the message. With no map
                // entry the earliest recorded claimant anchors it instead.
                lines.extend(decl.sections.get(path).map(|first| first.line));
                lines.sort_unstable();
                let sites: Vec<Site> = lines
                    .iter()
                    .map(|line| Site {
                        path: decl.file.clone(),
                        line: *line,
                    })
                    .collect();
                let others = lines[1..]
                    .iter()
                    .map(|line| format!("{}:{}", frame.display_path(&decl.file), line))
                    .collect::<Vec<_>>()
                    .join(", ");
                report.errors.push(Diagnostic {
                    code: Code::DUPLICATE_SECTION,
                    path: Some(decl.file.clone()),
                    line: Some(lines[0]),
                    column: None,
                    message: format!(
                        "duplicate section {}{}{} (also declared at {others}); {}",
                        render_catalog_id(frame.grammar(), &findings.declarations, id),
                        schema.ids.section_separator,
                        path,
                        // §FS-fields.front-matter.keys: a key written twice is no heading.
                        match decl.sections.get(path) {
                            Some(first) if first.in_front_matter() => {
                                remedies::duplicate_section_keys()
                            }
                            _ => remedies::duplicate_section_headings(),
                        }
                    ),
                    sites,
                    authority: Vec::new(),
                });
            }
        }
    }
}

fn section_outside_declaration_diagnostic(
    heading: &SectionHeadingOutsideDeclaration,
) -> Diagnostic {
    Diagnostic {
        code: Code::SECTION_OUTSIDE_DECLARATION,
        path: Some(heading.file.clone()),
        line: Some(heading.line),
        column: None,
        message: format!(
            "{}; {}",
            if section_path_is_numeric(&heading.path) {
                "numbered section outside any declaration"
            } else {
                "named section outside any declaration"
            },
            remedies::section_outside_declaration()
        ),
        sites: Vec::new(),
        authority: Vec::new(),
    }
}

/// §FS-declarations.checks.unmarked-heading.2.2: exact direct field-title/name
/// matches in v2 suggest the named field form, in configuration header order.
/// The scanner's concrete owner site supplies the depth; another declaration
/// of the same ID must not change the guidance.
fn field_heading_hint(
    heading: &UnmarkedHeading,
    findings: &Catalog,
    schema: &Schema,
    version: u32,
) -> Option<String> {
    if version != 2 {
        return None;
    }
    let declaration = findings
        .declarations
        .get(&heading.owner)?
        .iter()
        .find(|decl| decl.file == heading.file && decl.line == heading.owner_line)?;
    if heading.heading_level != declaration.heading_level + 1 {
        return None;
    }
    let (kind_name, kind) = schema
        .kinds()
        .find(|(name, _)| *name == declaration.id.kind)?;
    let field = kind.fields.iter().find(|field| {
        field.name == heading.title || field.title.as_deref() == Some(heading.title.as_str())
    })?;
    Some(format!(
        "; kind {kind_name} declares the field {name}, written {} {name}: {}",
        heading_marks(heading.heading_level),
        field.title.as_deref().unwrap_or(&heading.title),
        name = field.name,
    ))
}

/// `<ID>` for the declaration, `<ID><sep><path>` for a section of it.
fn coordinate(owner: &str, separator: &str, path: &str) -> String {
    if path.is_empty() {
        owner.to_string()
    } else {
        format!("{owner}{separator}{path}")
    }
}

pub(super) fn retain_heading_findings_in_scope(findings: &mut Catalog, scope: &ScanScope) {
    findings
        .section_headings_outside_declarations
        .retain(|heading| scope.contains(&heading.file));
    // §FS-declarations.checks.unmarked-heading: `--full` widens only the reference tier; this
    // Markdown convention remains restricted to configured scan scope.
    findings
        .unmarked_headings
        .retain(|heading| scope.contains(&heading.file));
    findings
        .relative_section_parents
        .retain(|heading| scope.contains(&heading.file));
}

/// §FS-declarations.checks.section-outside-declaration.3: unlike the reference tier, an outside-declaration heading
/// keeps the same public code and message when `--full` discovers it beyond
/// `[scan] include`.
pub(crate) fn out_of_scope_section_headings(
    findings: &Catalog,
    scope: Option<&ScanScope>,
) -> Vec<Diagnostic> {
    let Some(scope) = scope else {
        return Vec::new();
    };
    findings
        .section_headings_outside_declarations
        .iter()
        .filter(|heading| !scope.contains(&heading.file))
        .map(section_outside_declaration_diagnostic)
        .collect()
}

pub(crate) fn workspace_out_of_scope_section_headings(
    projects: &[WorkspaceProject],
    scopes: &[Option<ScanScope>],
) -> Vec<Diagnostic> {
    projects
        .iter()
        .zip(scopes)
        .flat_map(|(project, scope)| {
            out_of_scope_section_headings(&project.findings, scope.as_ref())
        })
        .collect()
}
