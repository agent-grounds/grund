//! §AR-checker.1.1: `conform`, the node-local half of the checker. Each
//! declaration and heading is judged against its own kind's shape and nothing
//! else, so the half is handed the schema and no `Rules` (§AR-config.5).

use std::path::Path;

use super::homes::{KindHomeIndex, paths_same_location_key};
use super::inline_style::check_inline_citation_style;
use super::near_miss::check_declaration_near_misses;
use super::sections::check_section_headings;
use super::sizes::check_oversized_leads;
use crate::config::{Frame, Schema};
use crate::grammar::render_id;
use crate::model::{
    Catalog, CheckReport, Declaration, Diagnostic, Site, TextOverlays, format_path,
    is_stub_for_inline_decl, resolve_stub_target, sort_path_key,
};
use crate::scanner::file_declares_inline_home;

/// §AR-checker.1.1: duplicate, misplaced-declaration, broken-stub and
/// declaration-near-miss (§FS-declarations.checks), the section family, inline
/// citation style and the opt-in lead budget, each pass in the order the single
/// driver ran it. Unsorted: the merge sorts (§AR-checker.1.4).
pub(crate) fn conform(
    schema: &Schema,
    catalog: &Catalog,
    frame: Frame<'_>,
    overlays: &TextOverlays,
) -> CheckReport {
    let mut report = CheckReport::default();
    check_duplicates(catalog, frame, &mut report);
    check_misplaced(catalog, schema, frame, &mut report);
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
    check_oversized_leads(catalog, schema, frame, overlays, &mut report);
    check_broken_stubs(catalog, schema, frame, overlays, &mut report);
    // §FS-declarations.checks.declaration-near-miss: headings that open like a declaration and
    // parse as none.
    check_declaration_near_misses(catalog, &mut report);
    report
}

/// §FS-declarations.checks.duplicate: an ID with more than one non-stub home is a duplicate.
fn check_duplicates(findings: &Catalog, frame: Frame<'_>, report: &mut CheckReport) {
    for (id, decls) in &findings.declarations {
        // §FS-declarations.checks.duplicate.3: a stub's home is named at its target, at
        // each line there that declares the ID (§FS-declarations.checks.duplicate.1).
        let home_sites: Vec<(&Path, usize)> = decls
            .iter()
            .filter(|decl| !is_stub_for_inline_decl(frame.root(), decl, decls))
            .flat_map(Declaration::home_sites)
            .collect();
        if home_sites.len() > 1 {
            let mut sites: Vec<Site> = home_sites
                .into_iter()
                .map(|(path, line)| Site {
                    path: path.to_path_buf(),
                    line,
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
            report.errors.push(Diagnostic {
                code: "duplicate",
                path: Some(primary.path),
                line: Some(primary.line),
                column: None,
                message: format!(
                    "duplicate declaration of {}{suffix}",
                    render_id(frame.grammar(), id)
                ),
                sites,
                authority: Vec::new(),
            });
        }
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
    for (id, decls) in &findings.declarations {
        for decl in decls {
            if let Some(expected) = kind_homes.single_file_for_kind(&id.kind)
                && !decl.is_stub
                && !paths_same_location_key(&decl.file, &expected.physical_path)
            {
                report.errors.push(Diagnostic {
                    code: "misplaced-declaration",
                    path: Some(decl.file.clone()),
                    line: Some(decl.line),
                    column: None,
                    message: format!(
                        "{} must be declared in {} (single-file kind)",
                        render_id(frame.grammar(), id),
                        expected.path
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
                        render_id(frame.grammar(), id),
                        id.kind,
                        home.kind,
                        home.path
                    )
                } else {
                    format!(
                        "{} must not be declared in {} (not a citable home)",
                        render_id(frame.grammar(), id),
                        home.place()
                    )
                };
                report.errors.push(Diagnostic {
                    code: "misplaced-declaration",
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

/// §FS-declarations.checks.broken-stub: a `# <ID>: [text](path)` stub is broken if `path` does
/// not exist, or exists but does not itself declare `<ID>` inline (§AR-checker.2.5).
fn check_broken_stubs(
    findings: &Catalog,
    schema: &Schema,
    frame: Frame<'_>,
    overlays: &TextOverlays,
    report: &mut CheckReport,
) {
    for (id, decls) in &findings.declarations {
        for decl in decls {
            if !decl.is_stub {
                continue;
            }
            let Some(target) = &decl.defined_in else {
                continue;
            };
            let resolved = resolve_stub_target(frame.root(), &decl.file, target);
            if !resolved.exists() {
                report.errors.push(Diagnostic {
                    code: "broken-stub",
                    path: Some(decl.file.clone()),
                    line: Some(decl.line),
                    column: None,
                    message: format!("stub link target missing: {}", format_path(target)),
                    sites: Vec::new(),
                    authority: Vec::new(),
                });
                continue;
            }
            // §FS-declarations.checks.broken-stub.1, §FS-declarations.checks.broken-stub.3:
            // the editor's text first, of a file the scan reads, the reader `show` takes.
            if !file_declares_inline_home(&resolved, id, schema, frame, overlays).unwrap_or(false) {
                report.errors.push(Diagnostic {
                    code: "broken-stub",
                    path: Some(decl.file.clone()),
                    line: Some(decl.line),
                    column: None,
                    message: format!(
                        "stub link target lacks {}: {}",
                        render_id(frame.grammar(), id),
                        format_path(target)
                    ),
                    sites: Vec::new(),
                    authority: Vec::new(),
                });
            }
        }
    }
}
