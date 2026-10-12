//! Test module: today's answers read off the file record (§AR-scanner.2.8.3,
//! §DF-one-parse-answers-from-0-19). On each row's tree the compatibility reading
//! gives the answer 0.18.0 keeps byte for byte, and beside it the record already
//! holds the answer 0.19.0 gives instead: the body end (P01, P10, D17a), the
//! section end (P07, P12), the lead a doc-comment heading cuts (P02, P03) and a
//! `--toc` title (closing hashes, D17d). P02 and P03 also hold the two readings
//! the parse-moves warning compares today's answer with.

use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::model::{Catalog, FileRecord, HeadingKind, ShowRenderMode};
use crate::testing::{legacy_fs_folder_config, scan_findings, test_root, write};

use super::compat::{
    TodayOwnerEnds, TodaySink, TodaySite, becomes_section, record_lead, today_slices,
};

fn scan(name: &str, file: &str, lines: &[&str]) -> (Config, Catalog, PathBuf) {
    let root = test_root(name);
    write(&root.join(file), &(lines.join("\n") + "\n"));
    let config = legacy_fs_folder_config(root.clone());
    let findings = scan_findings(&config, &root);
    (config, findings, PathBuf::from(file))
}

fn record_of<'a>(findings: &'a Catalog, file: &Path) -> &'a FileRecord {
    findings
        .files
        .iter()
        .find(|(path, _)| path.ends_with(file))
        .map(|(_, record)| record.as_ref())
        .expect("every scanned file has a record")
}

/// The site `show` pins a whole declaration to on a read-only scan, which
/// publishes no body end (§AR-benchmarks).
fn whole(line: usize) -> TodaySite {
    TodaySite {
        declaration_line: line,
        declaration_body_end: None,
        section_line: None,
    }
}

/// Today's slice of one view, and the `(path, title, depth)` of each section
/// an outline lists.
fn today(
    config: &Config,
    record: &FileRecord,
    site: TodaySite,
    section: Option<&str>,
    view: ShowRenderMode,
) -> (String, Vec<(String, String, usize)>) {
    let mut sinks = [TodaySink::new(view, false)];
    if today_slices(record, site, section, &mut sinks, config.lexical()).is_err() {
        panic!("today's slicer finds the site");
    }
    let [sink] = sinks;
    let (body, sections) = sink.finish();
    let sections = sections
        .into_iter()
        .map(|section| (section.path, section.title, section.depth))
        .collect();
    (body, sections)
}

fn heading(record: &FileRecord, line: usize) -> &crate::model::RecordedHeading {
    record.heading_at(line).expect("a recorded heading")
}

/// P01: today a Markdown body `show --full` reads runs past a plain heading to
/// the next declaration; the record ends it before the heading
/// (§FS-declarations.body).
#[test]
fn p01_today_runs_past_a_plain_heading() {
    let (config, findings, file) = scan(
        "compat_p01",
        "docs/functional-spec/FS-001-alpha.md",
        &[
            "# FS-001-alpha: Alpha", // 1
            "",                      // 2
            "Alpha body.",           // 3
            "",                      // 4
            "# Appendix",            // 5
            "",                      // 6
            "Appendix text.",        // 7
        ],
    );
    let record = record_of(&findings, &file);
    let (full, _) = today(&config, record, whole(1), None, ShowRenderMode::Full);
    assert_eq!(full, "Alpha body.\n\n# Appendix\n\nAppendix text.\n");
    assert_eq!(heading(record, 1).end, 4);
}

/// D17a: today a deeper declaration ends its parent's Markdown body; the record
/// runs the parent to its next heading of its own level or above.
#[test]
fn d17a_today_a_deeper_declaration_ends_the_body() {
    let (config, findings, file) = scan(
        "compat_d17a",
        "docs/functional-spec/FS-001-alpha.md",
        &[
            "# FS-001-alpha: Alpha",  // 1
            "",                       // 2
            "Lead.",                  // 3
            "",                       // 4
            "## FS-002-child: Child", // 5
            "",                       // 6
            "Child body.",            // 7
            "",                       // 8
            "# FS-003-delta: Delta",  // 9
        ],
    );
    let record = record_of(&findings, &file);
    let (full, _) = today(&config, record, whole(1), None, ShowRenderMode::Full);
    assert_eq!(full, "Lead.\n");
    assert_eq!(heading(record, 1).end, 8);
}

/// P07: today a Markdown section runs past a plain heading of its level to the
/// next numbered one; the record ends it at the plain heading (D17b).
#[test]
fn p07_today_a_section_runs_past_a_plain_heading() {
    let (config, findings, file) = scan(
        "compat_p07",
        "docs/functional-spec/FS-001-alpha.md",
        &[
            "# FS-001-alpha: Alpha", // 1
            "",                      // 2
            "## 1. One",             // 3
            "",                      // 4
            "One body.",             // 5
            "",                      // 6
            "## Notes",              // 7
            "",                      // 8
            "Notes text.",           // 9
            "",                      // 10
            "## 2. Two",             // 11
            "",                      // 12
            "Two body.",             // 13
        ],
    );
    let record = record_of(&findings, &file);
    let site = TodaySite {
        declaration_line: 1,
        declaration_body_end: Some(heading(record, 1).end),
        section_line: Some(3),
    };
    let (full, _) = today(&config, record, site, Some("1"), ShowRenderMode::Full);
    assert_eq!(full, "## 1. One\n\nOne body.\n\n## Notes\n\nNotes text.\n");
    assert_eq!(heading(record, 3).end, 6);
}

/// The `title` row: today a `--toc` title keeps a heading's closing hashes; the
/// record's heading text drops them (§FS-declarations.heading.rendered-text).
#[test]
fn title_today_keeps_closing_hashes() {
    let (config, findings, file) = scan(
        "compat_title",
        "docs/functional-spec/FS-001-alpha.md",
        &[
            "# FS-001-alpha: Alpha", // 1
            "",                      // 2
            "## 1. Steps ##",        // 3
            "",                      // 4
            "Step text.",            // 5
        ],
    );
    let record = record_of(&findings, &file);
    let (_, sections) = today(&config, record, whole(1), None, ShowRenderMode::Outline);
    assert_eq!(sections, vec![("1".to_string(), "Steps ##".to_string(), 1)]);
    assert_eq!(record.heading_text(heading(record, 3)), "1. Steps");
}

/// P10: today a `//` note next to a `///` block continues the body; the record
/// holds it as a block of its own, outside the body (D17c).
#[test]
fn p10_today_a_note_continues_the_body() {
    let (config, findings, file) = scan(
        "compat_p10",
        "src/lib.rs",
        &[
            "/// FS-003-gamma: Gamma", // 1
            "///",                     // 2
            "/// Gamma body.",         // 3
            "// Note.",                // 4
            "pub struct B;",           // 5
        ],
    );
    let record = record_of(&findings, &file);
    let (full, _) = today(&config, record, whole(1), None, ShowRenderMode::Full);
    assert_eq!(full, "Gamma body.\nNote.\n");
    assert_eq!(heading(record, 1).end, 3);
}

/// P02: today a `/** … */` line without ` * ` that reads `## 1. Sub` is listed by
/// `--toc` and cuts the lead, while the scan rejects it; the record keeps the
/// scan's reading, so the lead it gives runs to the block's end until 0.19.0
/// makes the line section `.1`.
#[test]
fn p02_today_toc_lists_a_bare_block_line() {
    let (config, findings, file) = scan(
        "compat_p02",
        "src/lib.rs",
        &[
            "/**",                    // 1
            " * FS-003-gamma: Gamma", // 2
            " *",                     // 3
            " * Gamma body.",         // 4
            " *",                     // 5
            "## 1. Sub",              // 6
            " *",                     // 7
            " * Sub body.",           // 8
            " */",                    // 9
            "pub struct B;",          // 10
        ],
    );
    let record = record_of(&findings, &file);
    let (lead, _) = today(&config, record, whole(2), None, ShowRenderMode::Default);
    let (_, sections) = today(&config, record, whole(2), None, ShowRenderMode::Outline);
    assert_eq!(lead, "Gamma body.\n");
    assert_eq!(sections, vec![("1".to_string(), "Sub".to_string(), 1)]);
    assert_eq!(heading(record, 6).kind, HeadingKind::Rejected);
    assert_eq!(heading(record, 2).lead_end, 9);
    // The line 0.19.0 makes section 1 (§FS-declarations.checks.parse-moves.2).
    assert!(becomes_section(record, heading(record, 6), 1));
}

/// P03: today `--toc` lists `/// # 1. Sub` under a `///` declaration and the
/// lead stops at it; the record holds it as rejected, so the lead runs on
/// (§FS-show.2.3.3).
#[test]
fn p03_today_a_same_level_heading_cuts_the_lead() {
    let (config, findings, file) = scan(
        "compat_p03",
        "src/lib.rs",
        &[
            "/// FS-003-gamma: Gamma", // 1
            "///",                     // 2
            "/// Gamma body.",         // 3
            "///",                     // 4
            "/// # 1. Sub",            // 5
            "///",                     // 6
            "/// Sub body.",           // 7
            "pub struct B;",           // 8
        ],
    );
    let record = record_of(&findings, &file);
    let (lead, _) = today(&config, record, whole(1), None, ShowRenderMode::Default);
    let (_, sections) = today(&config, record, whole(1), None, ShowRenderMode::Outline);
    assert_eq!(lead, "Gamma body.\n");
    assert_eq!(sections, vec![("1".to_string(), "Sub".to_string(), 1)]);
    assert_eq!(heading(record, 5).kind, HeadingKind::Rejected);
    assert_eq!(heading(record, 1).lead_end, 7);
    // 0.19.0's lead runs past it, and it is no section then either
    // (§FS-declarations.checks.parse-moves.2).
    let lead = record_lead(record, 1, 1, 7, config.lexical());
    assert_eq!(lead, "Gamma body.\n\n# 1. Sub\n\nSub body.\n");
    assert!(!becomes_section(record, heading(record, 5), 1));
}

/// D17d: today a `///` heading's trailing `*/` is dropped from its `--toc`
/// title; the record keeps it, since only a block comment's closer is envelope
/// (§FS-languages.comment-wrapper.headings).
#[test]
fn d17d_today_a_line_comment_title_drops_a_closer() {
    let (config, findings, file) = scan(
        "compat_d17d",
        "src/lib.rs",
        &[
            "/// FS-003-gamma: Gamma", // 1
            "/// ## 1. Step */",       // 2
            "pub struct B;",           // 3
        ],
    );
    let record = record_of(&findings, &file);
    let (_, sections) = today(&config, record, whole(1), None, ShowRenderMode::Outline);
    assert_eq!(sections, vec![("1".to_string(), "Step".to_string(), 1)]);
    assert_eq!(record.heading_text(heading(record, 2)), "1. Step */");
}

/// The accepted sections of the declaration at `line`, as `LineOwners` holds them.
fn accepted_sections(findings: &Catalog, line: usize) -> Vec<(String, usize, usize)> {
    let decl = findings
        .declarations
        .values()
        .flatten()
        .find(|decl| decl.line == line)
        .expect("the declaration");
    let mut sections: Vec<_> = decl
        .sections
        .iter()
        .map(|(path, info)| (path.clone(), info.line, info.heading_level))
        .collect();
    sections.sort_by_key(|(_, line, _)| *line);
    sections
}

/// P12: today a doc-comment section a citation sits in is closed only by the
/// declaration's next accepted section, so a shallower rejected heading leaves
/// it open; the record ends it there (D17b).
#[test]
fn p12_today_a_rejected_heading_leaves_a_section_open() {
    let (_, findings, file) = scan(
        "compat_p12",
        "src/lib.rs",
        &[
            "/// FS-003-gamma: Gamma", // 1
            "///",                     // 2
            "/// Gamma body.",         // 3
            "///",                     // 4
            "/// ## 1. One",           // 5
            "///",                     // 6
            "/// One body.",           // 7
            "///",                     // 8
            "/// # 2. Two",            // 9
            "///",                     // 10
            "/// Two text.",           // 11
            "pub struct B;",           // 12
        ],
    );
    let record = record_of(&findings, &file);
    let sections = accepted_sections(&findings, 1);
    let one = heading(record, 5);
    let today_end = TodayOwnerEnds::new(record).end(&sections, 5, one.level);
    assert_eq!(today_end, usize::MAX);
    assert_eq!(heading(record, 9).kind, HeadingKind::Rejected);
    assert_eq!(one.end, 8);
}

/// P12, with a duplicate: today a second claimant of a path does not close the
/// section before it; the record ends that section there.
#[test]
fn p12_today_a_duplicate_heading_leaves_a_section_open() {
    let (_, findings, file) = scan(
        "compat_p12_duplicate",
        "src/lib.rs",
        &[
            "/// FS-003-gamma: Gamma", // 1
            "///",                     // 2
            "/// Gamma body.",         // 3
            "///",                     // 4
            "/// ## 1. One",           // 5
            "///",                     // 6
            "/// One body.",           // 7
            "///",                     // 8
            "/// ## 2. Two",           // 9
            "///",                     // 10
            "/// Two body.",           // 11
            "///",                     // 12
            "/// ## 1. One again",     // 13
            "///",                     // 14
            "/// Again.",              // 15
            "pub struct B;",           // 16
        ],
    );
    let record = record_of(&findings, &file);
    let sections = accepted_sections(&findings, 1);
    let two = heading(record, 9);
    let today_end = TodayOwnerEnds::new(record).end(&sections, 9, two.level);
    assert_eq!(today_end, usize::MAX);
    assert_eq!(heading(record, 13).kind, HeadingKind::Duplicate);
    assert_eq!(two.end, 12);
}

/// Markdown: today and the record agree that a section a citation sits in ends
/// before the next ATX heading of its level or above, plain or numbered.
#[test]
fn markdown_owner_ends_agree_with_the_record() {
    let (_, findings, file) = scan(
        "compat_markdown_owner",
        "docs/functional-spec/FS-001-alpha.md",
        &[
            "# FS-001-alpha: Alpha", // 1
            "",                      // 2
            "## 1. One",             // 3
            "",                      // 4
            "One body.",             // 5
            "",                      // 6
            "## Notes",              // 7
            "",                      // 8
            "Notes text.",           // 9
        ],
    );
    let record = record_of(&findings, &file);
    let sections = accepted_sections(&findings, 1);
    let one = heading(record, 3);
    let today_end = TodayOwnerEnds::new(record).end(&sections, 3, one.level);
    assert_eq!(today_end, 6);
    assert_eq!(one.end, today_end);
}
