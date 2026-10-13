//! §FS-declarations.checks.parse-moves: the three shapes of tree on which 0.19.0's one parse
//! moves a `check` verdict (§DF-one-parse-answers-from-0-19.4), the fourth that moves with
//! them, a relative heading in a doc-comment declaration body
//! (§DF-doc-comment-relative-headings), and the fifth, a complete mixed heading there, in
//! `mixed.rs` (§DF-number-name-sections.decision.doc-comments), each warned at the line that
//! moves it. Today's answer is what the catalog already holds; 0.19.0's is read off the file
//! record the scan kept (§AR-scanner.2.8), so the pass reads no file. It goes with today's
//! answers in 0.19.0 (§FS-declarations.checks.parse-moves.4).

mod mixed;

use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use super::sizes::{lead_budgets, site_coordinate};
use crate::config::{Frame, LeadSizeWarning, Schema, Strength, measure_point_text};
use crate::grammar::{expand_relative_heading, flatten_cross_ref_links};
use crate::model::{
    Catalog, CheckReport, Declaration, DeclarationSource, Diagnostic, FileRecord, HeadingKind, Id,
    RecordedHeading, SectionInfo, name_under_number,
};
use crate::resolver::point_body_pair;
use crate::scanner::{
    RelativeAnswer, bare_block_line, becomes_section, doc_comment_relative_answers, enveloped_line,
    record_lead,
};

/// The release whose one parse moves every shape this pass warns of, named by each message
/// and the release that removes the pass (§FS-declarations.checks.parse-moves.4).
pub(crate) const ONE_PARSE_FLIP: &str = "0.19.0";

/// §FS-declarations.checks.parse-moves.3: every shape is a warning, so a run it alone
/// reports exits `0`.
pub(super) fn check_parse_moves(
    findings: &Catalog,
    schema: &Schema,
    frame: Frame<'_>,
    report: &mut CheckReport,
) {
    check_closing_headings(findings, schema, frame, report);
    check_rejected_headings(findings, schema, frame, report);
    check_relative_headings(findings, schema, frame, report);
    mixed::check_mixed_headings(findings, schema, frame, report);
}

/// The first shape of §FS-declarations.checks.parse-moves.2: a doc-comment heading the record
/// ends a section at, above a citation today's owner keeps in that section. One finding per heading, naming
/// the first citation it moves (§FS-declarations.checks.parse-moves.1).
fn check_closing_headings(
    findings: &Catalog,
    schema: &Schema,
    frame: Frame<'_>,
    report: &mut CheckReport,
) {
    let mut closers: BTreeMap<(&Path, usize), (usize, String)> = BTreeMap::new();
    let mut last: Option<(&Path, Option<&FileRecord>)> = None;
    for citation in &findings.citations {
        let (Some(id), Some(section)) =
            (&citation.enclosing_declaration, &citation.enclosing_section)
        else {
            continue;
        };
        // Citations come file by file, so the record lookup is one per file.
        let record = match last {
            Some((path, record)) if path == citation.file.as_path() => record,
            _ => {
                let record = findings
                    .files
                    .get(&citation.file)
                    .map(|record| record.as_ref())
                    .filter(|record| !record.markdown);
                last = Some((&citation.file, record));
                record
            }
        };
        let Some(record) = record else {
            continue;
        };
        let Some(declaration) = findings.declarations.get(id).and_then(|declarations| {
            declarations
                .iter()
                .filter(|declaration| {
                    declaration.file == citation.file
                        && (declaration.body_start..=declaration.body_end).contains(&citation.line)
                })
                .max_by_key(|declaration| declaration.body_start)
        }) else {
            continue;
        };
        let Some(heading) = declaration
            .sections
            .get(section)
            .and_then(|info| record.heading_at(info.line))
            .filter(|heading| heading.kind == HeadingKind::Section)
        else {
            continue;
        };
        if citation.line <= heading.end {
            continue;
        }
        let Some(closer) = record
            .heading_at(heading.end + 1)
            .filter(|closer| closer.kind != HeadingKind::Declaration)
        else {
            continue;
        };
        closers
            .entry((citation.file.as_path(), closer.line))
            .and_modify(|(first, _)| *first = (*first).min(citation.line))
            .or_insert_with(|| {
                let coordinate = site_coordinate(schema, frame, id, Some(section));
                (citation.line, coordinate)
            });
    }
    for ((path, line), (first, coordinate)) in closers {
        push_warning(
            report,
            path,
            line,
            format!(
                "from grund {ONE_PARSE_FLIP} this heading ends {coordinate}, so the citation on line {first} leaves it; make the heading deeper, or move the citation above it, to keep it in {coordinate}"
            ),
        );
    }
}

/// The second and third shapes of §FS-declarations.checks.parse-moves.2, the two a heading the
/// record rejects makes. Only a source file holding one is read further,
/// so a tree without one pays a walk over each file's headings and nothing else.
fn check_rejected_headings(
    findings: &Catalog,
    schema: &Schema,
    frame: Frame<'_>,
    report: &mut CheckReport,
) {
    let budgets = lead_budgets(schema);
    let mut by_file: Option<HashMap<&Path, Vec<(&Id, &Declaration)>>> = None;
    for (path, record) in &findings.files {
        if record.markdown
            || !record
                .headings
                .iter()
                .any(|heading| heading.kind == HeadingKind::Rejected && heading.path.is_some())
        {
            continue;
        }
        let declarations = by_file
            .get_or_insert_with(|| text_declarations(findings))
            .get(path.as_path())
            .map(Vec::as_slice)
            .unwrap_or_default();
        let at = |line: usize| {
            declarations
                .iter()
                .find(|(_, declaration)| declaration.line == line)
                .copied()
        };
        for owner in record
            .headings
            .iter()
            .filter(|heading| heading.kind == HeadingKind::Declaration)
        {
            let Some((id, declaration)) = at(owner.line) else {
                continue;
            };
            let body = record.headings_between(owner.line + 1, owner.end);
            let level = declaration.heading_level;
            for heading in body {
                if becomes_section(record, heading, level)
                    && let Some(section) = heading.path.as_deref()
                {
                    let coordinate = site_coordinate(schema, frame, id, Some(section));
                    push_warning(
                        report,
                        path,
                        heading.line,
                        format!(
                            "from grund {ONE_PARSE_FLIP} this line is the heading of {coordinate}; prefix it with ` * ` to make it {coordinate} in both releases, or rewrite it as prose to keep it out of both"
                        ),
                    );
                }
            }
            if budgets.is_empty() {
                continue;
            }
            let site = LeadSite {
                record,
                id,
                declaration,
                body,
            };
            site.check(findings, schema, frame, &budgets, owner, None, report);
            for heading in body
                .iter()
                .filter(|heading| heading.kind == HeadingKind::Section)
            {
                let section = heading
                    .path
                    .as_deref()
                    .and_then(|path| declaration.sections.get_key_value(path))
                    .filter(|(_, info)| info.line == heading.line)
                    .map(|(path, info)| (path.as_str(), info));
                if section.is_some() {
                    site.check(findings, schema, frame, &budgets, heading, section, report);
                }
            }
        }
    }
}

/// The fourth shape of §FS-declarations.checks.parse-moves.2: a relative heading in a source
/// doc-comment declaration body, prose today, takes the meaning 0.19.0 gives it. Its answer is
/// `relative_answer`'s, the one the Markdown parent pass gives, and a form the rules refuse is
/// left alone.
/// Only a source file whose record holds a plain heading is read further.
fn check_relative_headings(
    findings: &Catalog,
    schema: &Schema,
    frame: Frame<'_>,
    report: &mut CheckReport,
) {
    let named_sections = schema.ids.named_sections;
    let mut by_file: Option<HashMap<&Path, Vec<(&Id, &Declaration)>>> = None;
    for (path, record) in &findings.files {
        if record.markdown
            || !record
                .headings
                .iter()
                .any(|heading| heading.kind == HeadingKind::Plain)
        {
            continue;
        }
        let declarations = by_file
            .get_or_insert_with(|| text_declarations(findings))
            .get(path.as_path())
            .map(Vec::as_slice)
            .unwrap_or_default();
        for owner in record
            .headings
            .iter()
            .filter(|heading| heading.kind == HeadingKind::Declaration)
        {
            let Some((id, declaration)) = declarations
                .iter()
                .find(|(_, declaration)| declaration.line == owner.line)
            else {
                continue;
            };
            for (heading, answer) in
                doc_comment_relative_answers(record, declaration, owner.end, named_sections)
            {
                // §FS-declarations.checks.parse-moves.1: the two messages shape 4 takes, and the
                // last for a mixed path, whose complete heading is prose in 0.18.0 as well.
                let message = match answer {
                    RelativeAnswer::Section(section) if name_under_number(&section) => {
                        mixed::held_message(&site_coordinate(schema, frame, id, Some(&section)))
                    }
                    RelativeAnswer::Section(section) => {
                        let coordinate = site_coordinate(schema, frame, id, Some(&section));
                        let written = enveloped_line(record, heading.line).unwrap_or_default();
                        let complete = expand_relative_heading(written, &section);
                        let edit = if bare_block_line(record, heading.line) {
                            format!("prefix it with ` * ` and write it `{complete}`")
                        } else {
                            format!("write it `{complete}`")
                        };
                        format!(
                            "from grund {ONE_PARSE_FLIP} this line is the heading of {coordinate}; {edit} to make it {coordinate} in both releases, or make it a bold label or a code span to keep it prose in both"
                        )
                    }
                    RelativeAnswer::NoParent => format!(
                        "from grund {ONE_PARSE_FLIP} this line is a relative section heading with no section one level up, a relative-section-parent error; make it a bold label or a code span to keep it prose in both releases"
                    ),
                    RelativeAnswer::Refused => continue,
                };
                push_warning(report, path, heading.line, message);
            }
        }
    }
}

/// One declaration whose body holds a heading the record rejects, with that body's
/// recorded headings: the sites whose lead the third shape of
/// §FS-declarations.checks.parse-moves.2 judges.
struct LeadSite<'a> {
    record: &'a FileRecord,
    id: &'a Id,
    declaration: &'a Declaration,
    body: &'a [RecordedHeading],
}

impl LeadSite<'_> {
    /// Warn where a rejected heading cuts this site's lead today and 0.19.0's lead, which
    /// runs past it to the record's lead end, is over a budget today's lead is not
    /// (§FS-declarations.checks.parse-moves.2, the third shape). The budget is the strongest the longer
    /// lead is over, the one its `oversized-lead` would be reported against.
    #[allow(clippy::too_many_arguments)]
    fn check(
        &self,
        findings: &Catalog,
        schema: &Schema,
        frame: Frame<'_>,
        budgets: &[(Strength, LeadSizeWarning)],
        site: &RecordedHeading,
        section: Option<(&str, &SectionInfo)>,
        report: &mut CheckReport,
    ) {
        let level = self.declaration.heading_level;
        // A line 0.19.0 makes a section still cuts the lead there.
        let lead_end = self
            .body
            .iter()
            .filter(|heading| heading.line > site.line && heading.line <= site.lead_end)
            .find(|heading| becomes_section(self.record, heading, level))
            .map_or(site.lead_end, |heading| heading.line - 1);
        let Some(cutter) = self.body.iter().find(|heading| {
            heading.line > site.line
                && heading.line <= lead_end
                && heading.kind == HeadingKind::Rejected
                && heading.path.is_some()
        }) else {
            return;
        };
        let lexical = frame.compiled.lexical(schema);
        let lead = record_lead(
            self.record,
            self.declaration.line,
            site.line,
            lead_end,
            lexical,
        );
        let lead = flatten_cross_ref_links(&lead, lexical, false);
        let Some((warning, actual)) = budgets.iter().find_map(|(_, warning)| {
            let actual = measure_point_text(&lead, warning.unit);
            (actual > warning.max).then_some((*warning, actual))
        }) else {
            return;
        };
        let Ok(Some((today, _))) = point_body_pair(
            &findings.files,
            schema,
            frame,
            self.id,
            self.declaration,
            section,
        ) else {
            return;
        };
        if measure_point_text(&today, warning.unit) > warning.max {
            return;
        }
        let coordinate = site_coordinate(schema, frame, self.id, section.map(|(path, _)| path));
        push_warning(
            report,
            &self.declaration.file,
            cutter.line,
            format!(
                "from grund {ONE_PARSE_FLIP} this heading no longer cuts the lead of {coordinate}, which then measures {actual} {}, over the configured maximum of {}; make it a section of {coordinate} to keep the cut",
                warning.unit.as_str(),
                warning.max,
            ),
        );
    }
}

/// The scanned text declarations of each file, the sites a rejected heading can sit in.
fn text_declarations(findings: &Catalog) -> HashMap<&Path, Vec<(&Id, &Declaration)>> {
    let mut by_file: HashMap<&Path, Vec<(&Id, &Declaration)>> = HashMap::new();
    for (id, declarations) in &findings.declarations {
        for declaration in declarations {
            if declaration.is_stub
                || declaration.e2e_case.is_some()
                || !matches!(declaration.source, DeclarationSource::Text)
            {
                continue;
            }
            by_file
                .entry(declaration.file.as_path())
                .or_default()
                .push((id, declaration));
        }
    }
    by_file
}

fn push_warning(report: &mut CheckReport, path: &Path, line: usize, message: String) {
    report.warnings.push(Diagnostic {
        code: "parse-moves",
        path: Some(path.to_path_buf()),
        line: Some(line),
        column: None,
        message,
        sites: Vec::new(),
        authority: Vec::new(),
    });
}
