//! §AR-checker.1.1: `conform`, the node-local half of the checker. Each
//! declaration and heading is judged against its own kind's shape and nothing
//! else, so the half is handed the schema and no `Rules` (§AR-config.5).

use super::conflict_marker::check_conflict_blocks;
use super::fields::check_fields;
use super::glob_citations::check_glob_citations;
use super::homes::KindHomeIndex;
use super::inline_style::check_inline_citation_style;
use super::near_miss::check_declaration_near_misses;
use super::sections::check_section_headings;
use super::sizes::check_oversized_leads;
use crate::config::{Extent, Frame, Schema};
use crate::grammar::render_catalog_id;
use crate::model::{
    Catalog, CheckReport, Code, Declaration, Diagnostic, ExcludingRule, Id, MissingStubRepair,
    Site, StubResolution, Switch, format_path, paired_declarations, sort_path_key,
};

/// §AR-checker.1.1: duplicate, misplaced-declaration, broken-stub,
/// declaration-near-miss and conflict-marker (§FS-declarations.checks), the section family, inline
/// citation style, the opt-in lead budget and the field family (§FS-fields.checks),
/// with shared duplicate/broken-stub guidance (§AR-system.2.2).
/// Unsorted: the merge sorts (§AR-checker.1.4).
pub(crate) fn conform(schema: &Schema, catalog: &Catalog, frame: Frame<'_>) -> CheckReport {
    let mut report = CheckReport::default();
    check_declaration_homes(catalog, frame, &mut report);
    check_misplaced(catalog, schema, frame, &mut report);
    // §FS-declarations.checks.misplaced-declaration.unlisted-project: all placement forms.
    explain_unlisted_project_placement(schema, frame, &mut report);
    // §FS-declarations.checks.section-heading-level / §FS-declarations.checks.duplicate-section:
    // the depth a declaration's own section headings write, and whether two claim one path. One
    // file per invariant family in `sections.rs` (§AR-checker.2.15, §AR-core-module-layout.1).
    check_section_headings(catalog, schema, frame, &mut report);
    // §FS-inline-citation-style.4: inline source-comment citation sites are
    // checked from scanner-provided site metadata; Markdown citations and
    // declaration bodies carry no site and are ignored here.
    check_inline_citation_style(catalog, schema, frame, &mut report);
    // §FS-declarations.checks.oversized-lead: an absent key stops before any body read; an opted-in
    // project judges only the already-scoped scanner sites, including duplicate
    // claimants, through the shared show slicer. Warnings never affect exit.
    check_oversized_leads(catalog, schema, frame, &mut report);
    // §FS-declarations.checks.declaration-near-miss: headings that open like a declaration and
    // parse as none.
    check_declaration_near_misses(catalog, schema, frame, &mut report);
    // §FS-declarations.checks.conflict-marker: each block the file pass recorded.
    check_conflict_blocks(catalog, frame, &mut report);
    // §FS-check.checks.glob-citation: patterns written where a citation belongs.
    check_glob_citations(catalog, schema, &mut report);
    // §FS-fields.checks: each declaration against its kind's fields and `closed`.
    check_fields(catalog, schema, frame, &mut report);
    report
}

/// §FS-declarations.checks.duplicate: an ID with more than one non-stub home is a duplicate.
/// Its declarations are counted once its stubs are paired by their verdicts
/// (§AR-scanner.4.6). Both findings share one missing-target repair derivation
/// per ID (§AR-system.2.2).
fn check_declaration_homes(findings: &Catalog, frame: Frame<'_>, report: &mut CheckReport) {
    for (id, decls) in &findings.declarations {
        // §FS-declarations.stubs.pairing.3: a stub's home is named at its target, at
        // each line there that declares the ID (§FS-declarations.stubs.pairing.1).
        let paired = paired_declarations(decls);
        let repair = MissingStubRepair::new(&paired);
        if paired.len() > 1 {
            let mut sites: Vec<Site> = paired
                .iter()
                .map(|declaration| Site {
                    path: declaration.record.file.clone(),
                    line: declaration.record.line,
                })
                .collect();
            sites.sort_by(|a, b| {
                (sort_path_key(&a.path), a.line).cmp(&(sort_path_key(&b.path), b.line))
            });
            let primary = sites[0].clone();
            let others = sites[1..]
                .iter()
                // §FS-errors.3.1 / §FS-workspace.8.1: the report's display, not the
                // member's — the printer anchors this finding from the report root,
                // so the sites named inside its message come from there too.
                .map(|site| format!("{}:{}", frame.display_path(&site.path), site.line))
                .collect::<Vec<_>>();
            let suffix = if others.is_empty() {
                String::new()
            } else {
                format!(" (also declared at {})", others.join(", "))
            };
            let rendered_id = render_catalog_id(frame.grammar(), &findings.declarations, id);
            let mut message = format!("duplicate declaration of {rendered_id}{suffix}");
            message.push_str(&repair.located_suffix(&rendered_id, |path| frame.display_path(path)));
            report.errors.push(Diagnostic {
                code: Code::DUPLICATE,
                path: Some(primary.path),
                line: Some(primary.line),
                column: None,
                message,
                sites,
                authority: Vec::new(),
            });
        }
        check_broken_stubs(findings, id, decls, &repair, frame, report);
    }
}

/// §FS-declarations.checks.misplaced-declaration: declarations must respect configured kind
/// homes. A single-file kind must live in its exact `file`; any declaration inside a unique
/// configured home must match that home's kind.
fn check_misplaced(
    findings: &Catalog,
    schema: &Schema,
    frame: Frame<'_>,
    report: &mut CheckReport,
) {
    let kind_homes = KindHomeIndex::new(schema, frame);
    // §FS-check.1.4.1: a home is what a declaration can be misplaced against.
    if schema
        .places()
        .any(|(_, place)| place.extent != Extent::Complement)
    {
        report.armed.arm(Switch::KindHome);
    }
    for (id, decls) in &findings.declarations {
        for decl in decls {
            if let Some(expected) = kind_homes.single_file_for_kind(&id.kind)
                && !decl.is_stub
                && !expected.holds(&decl.file)
            {
                report.errors.push(Diagnostic {
                    code: Code::MISPLACED_DECLARATION,
                    path: Some(decl.file.clone()),
                    line: Some(decl.line),
                    column: None,
                    message: format!(
                        "{} must be declared in {} (single-file kind)",
                        render_catalog_id(frame.grammar(), &findings.declarations, id),
                        expected.label()
                    ),
                    sites: Vec::new(),
                    authority: Vec::new(),
                });
                continue;
            }

            let Some(home) = kind_homes.unique_decl_home_for_file(&decl.file) else {
                continue;
            };
            if home.kind != id.kind {
                // §FS-declarations.checks.misplaced-declaration.3: a non-citable home has no kind
                // an author could have declared instead, so the message names the place and says
                // why, rather than pointing at a kind that does not exist.
                let message = if home.citable {
                    format!(
                        "{} declares kind {} inside {} home {}",
                        render_catalog_id(frame.grammar(), &findings.declarations, id),
                        id.kind,
                        home.kind,
                        home.path
                    )
                } else {
                    format!(
                        "{} must not be declared in {} (not a citable home)",
                        render_catalog_id(frame.grammar(), &findings.declarations, id),
                        home.place()
                    )
                };
                report.errors.push(Diagnostic {
                    code: Code::MISPLACED_DECLARATION,
                    path: Some(decl.file.clone()),
                    line: Some(decl.line),
                    column: None,
                    message,
                    sites: Vec::new(),
                    authority: Vec::new(),
                });
            }
        }
    }
}

/// §FS-declarations.checks.misplaced-declaration.unlisted-project: enrich all
/// placement forms at their common boundary. Every frontend receives the same
/// message; diagnostic locations, codes, fields and verdicts remain untouched.
fn explain_unlisted_project_placement(schema: &Schema, frame: Frame<'_>, report: &mut CheckReport) {
    let directories = report
        .errors
        .iter()
        .filter(|error| error.code == Code::MISPLACED_DECLARATION)
        .filter_map(|error| error.path.as_ref().and_then(|path| path.parent()));
    let contexts = crate::workspace::unlisted_project_contexts(schema, frame, directories);
    for error in &mut report.errors {
        if error.code == Code::MISPLACED_DECLARATION
            && let Some(path) = &error.path
            && let Some(context) = path.ancestors().find_map(|dir| contexts.get(dir))
        {
            error.message.push_str(context);
        }
    }
}

/// §FS-declarations.checks.broken-stub: the verdict the scan recorded on each stub
/// (§AR-scanner.4.6), its target missing, not read, or not declaring
/// the ID (§AR-checker.2.5). Extension and exclusion refusals explain why the target
/// is not scanned rather than claiming it lacks the ID (§FS-declarations.stubs.target.3).
/// Missing-target messages connect to the duplicate without adding sites
/// (§AR-system.2.2).
fn check_broken_stubs(
    findings: &Catalog,
    id: &Id,
    decls: &[Declaration],
    repair: &MissingStubRepair<'_>,
    frame: Frame<'_>,
    report: &mut CheckReport,
) {
    for decl in decls {
        let (Some(verdict), Some(target)) = (&decl.stub_resolution, &decl.defined_in) else {
            continue;
        };
        let message = match verdict {
            StubResolution::Missing => {
                format!(
                    "stub link target missing: {}{}",
                    format_path(target),
                    repair.site_free_suffix(
                        &render_catalog_id(frame.grammar(), &findings.declarations, id),
                        decl,
                    )
                )
            }
            StubResolution::UnlistedExtension(extension) => format!(
                "stub link target {} is not scanned: extension {} is not in [scan] extensions",
                format_path(target),
                extension
            ),
            StubResolution::Excluded(rule) => excluded_target_message(target, rule),
            StubResolution::NotRead | StubResolution::LacksId => format!(
                "stub link target lacks {}: {}",
                render_catalog_id(frame.grammar(), &findings.declarations, id),
                format_path(target)
            ),
            StubResolution::OwnFile | StubResolution::Declares(_) => continue,
        };
        report.errors.push(Diagnostic {
            code: Code::BROKEN_STUB,
            path: Some(decl.file.clone()),
            line: Some(decl.line),
            column: None,
            message,
            sites: Vec::new(),
            authority: Vec::new(),
        });
    }
}

/// Render the saved refusal without rechecking the target or its rules
/// (§FS-declarations.stubs.target.3, §AR-checker.2.5).
fn excluded_target_message(target: &std::path::Path, rule: &ExcludingRule) -> String {
    let (reason, remedy) = match rule {
        ExcludingRule::Exclude(entry) => (
            format!("[scan] exclude names {entry}"),
            format!("move the target out of {entry}, or take {entry} out of [scan] exclude"),
        ),
        ExcludingRule::Pattern(pattern) => (
            format!("[schema.sources] exclude matches it by {pattern}"),
            format!(
                "move the target out of what {pattern} matches, or take {pattern} out of \
                 [schema.sources] exclude"
            ),
        ),
        ExcludingRule::IgnoreFile { file, pattern } => (
            format!("{file} ignores it by {pattern}"),
            format!("move the target out of what {file} ignores, or stop ignoring it there"),
        ),
    };
    format!(
        "stub link target {} is not scanned: {reason}; \
         this became an error in grund 0.19.0 — {remedy}",
        format_path(target)
    )
}
