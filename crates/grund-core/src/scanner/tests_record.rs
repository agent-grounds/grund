//! Test module: the file record (§AR-scanner.2.8). One pass fills it on every
//! run: the text as `sources` returned it, every heading outside a fence with its
//! kind and rendered text, every comment block by form, the docstring zones, and
//! the end and lead end of every declaration and section by the rules of
//! §FS-declarations.body, §FS-declarations.lead and §AR-scanner.2.4.4.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::model::{BlockForm, Catalog, FileRecord, FileText, HeadingKind, RecordedHeading};
use crate::sources::{Buffers, Sources};
use crate::testing::{legacy_fs_folder_config, scan_findings, test_root, write};

fn scan(name: &str, file: &str, lines: &[&str]) -> (Catalog, PathBuf) {
    let root = test_root(name);
    write(&root.join(file), &(lines.join("\n") + "\n"));
    let config = legacy_fs_folder_config(root.clone());
    (scan_findings(&config, &root), PathBuf::from(file))
}

fn record_of<'a>(findings: &'a Catalog, file: &Path) -> &'a FileRecord {
    findings
        .files
        .iter()
        .find(|(path, _)| path.ends_with(file))
        .map(|(_, record)| record.as_ref())
        .expect("every scanned file has a record")
}

/// `(line, kind, rendered text, end, lead end)` of every recorded heading.
fn headings(record: &FileRecord) -> Vec<(usize, HeadingKind, &str, usize, usize)> {
    record
        .headings
        .iter()
        .map(|heading: &RecordedHeading| {
            (
                heading.line,
                heading.kind,
                record.heading_text(heading),
                heading.end,
                heading.lead_end,
            )
        })
        .collect()
}

/// §AR-scanner.2.8.1: a Markdown file's record holds its text and every heading
/// outside a fence, with the end and lead end the body, section and lead rules
/// give (§FS-declarations.body, §FS-declarations.lead, §AR-scanner.2.4.4).
#[test]
fn markdown_record_holds_headings_and_their_extents() {
    let (findings, file) = scan(
        "record_markdown",
        "docs/functional-spec/FS-001-alpha.md",
        &[
            "# FS-001-alpha: Alpha", // 1
            "",                      // 2
            "The lead.",             // 3
            "",                      // 4
            "## 1. Steps ##",        // 5
            "",                      // 6
            "```text",               // 7
            "## 2. Fenced",          // 8
            "```",                   // 9
            "### 1.1 Detail",        // 10
            "",                      // 11
            "## Notes",              // 12
            "",                      // 13
            "Prose.",                // 14
            "",                      // 15
            "# Appendix",            // 16
            "",                      // 17
            "After the body.",       // 18
        ],
    );
    let record = record_of(&findings, &file);
    assert!(record.markdown);
    assert_eq!(record.total_lines, 18);
    assert_eq!(record.line(5), Some("## 1. Steps ##"));
    assert_eq!(
        headings(record),
        vec![
            (1, HeadingKind::Declaration, "FS-001-alpha: Alpha", 15, 4),
            // D17b: the plain `## Notes` ends section 1; its child cuts its lead.
            (5, HeadingKind::Section, "1. Steps", 11, 9),
            (10, HeadingKind::Section, "1.1 Detail", 11, 11),
            (12, HeadingKind::Plain, "Notes", 0, 0),
            (16, HeadingKind::Plain, "Appendix", 0, 0),
        ]
    );
    let steps = record.heading_at(5).expect("section 1 is recorded");
    assert_eq!(&record.text()[steps.span.clone()], "## 1. Steps ##");
    assert_eq!(steps.path.as_deref(), Some("1"));
    assert!(
        record.blocks.is_empty(),
        "a Markdown file has no comment blocks"
    );
}

/// §AR-scanner.2.8.1: a source file's record holds each comment block by form
/// (D17c: a `//` note beside a `///` block is a block of its own), and the
/// headings inside a declaration's block read through their envelope. A
/// shallower `/// # 2. Two` is rejected and, with a plain heading, ends the
/// section above it (D17b), while only the accepted section cuts the lead.
#[test]
fn source_record_holds_blocks_and_doc_comment_headings() {
    let (findings, file) = scan(
        "record_source",
        "src/lib.rs",
        &[
            "/// FS-003-gamma: Gamma", // 1
            "///",                     // 2
            "/// The lead.",           // 3
            "///",                     // 4
            "/// ## 1. One",           // 5
            "///",                     // 6
            "/// One's body.",         // 7
            "/// # 2. Two",            // 8
            "///",                     // 9
            "/// ### Notes",           // 10
            "// A note.",              // 11
            "pub fn gamma() {}",       // 12
        ],
    );
    let record = record_of(&findings, &file);
    assert!(!record.markdown);
    let forms: Vec<_> = record
        .blocks
        .iter()
        .map(|block| (block.start, block.end, block.form.clone(), block.doc))
        .collect();
    assert_eq!(
        forms,
        vec![
            (1, 10, BlockForm::Line("///".to_string()), true),
            (11, 11, BlockForm::Line("//".to_string()), false),
        ]
    );
    assert_eq!(
        headings(record),
        vec![
            (1, HeadingKind::Declaration, "FS-003-gamma: Gamma", 10, 4),
            (5, HeadingKind::Section, "1. One", 7, 7),
            (8, HeadingKind::Rejected, "2. Two", 0, 0),
            (10, HeadingKind::Plain, "Notes", 0, 0),
        ]
    );
}

/// §FS-languages.comment-wrapper.headings: a `/** … */` line written without
/// ` * ` is kept as written, and a trailing `*/` is envelope only there (D17d).
/// The line the scan reads as a level-1 path is recorded as rejected, the
/// reading 0.18.0 keeps (§DF-one-parse-answers-from-0-19).
#[test]
fn block_comment_lines_are_read_through_their_envelope() {
    let (findings, file) = scan(
        "record_block_comment",
        "src/lib.rs",
        &[
            "/**",                    // 1
            " * FS-004-delta: Delta", // 2
            " *",                     // 3
            "## 1. Sub",              // 4
            " * ## 2. Next */",       // 5
            "pub struct D;",          // 6
        ],
    );
    let record = record_of(&findings, &file);
    assert_eq!(record.blocks.len(), 1);
    assert_eq!(record.blocks[0].form, BlockForm::Block);
    assert_eq!(
        headings(record),
        vec![
            (2, HeadingKind::Declaration, "FS-004-delta: Delta", 5, 4),
            (4, HeadingKind::Rejected, "1. Sub", 0, 0),
            (5, HeadingKind::Section, "2. Next", 5, 5),
        ]
    );
}

/// §AR-scanner.2.8.1: a docstring line's zone is the part of it the scan reads,
/// inside its delimiters (§AR-scanner.4.4).
#[test]
fn docstring_lines_record_their_zones() {
    let (findings, file) = scan(
        "record_docstring",
        "src/mod.py",
        &[
            "def f():",                  // 1
            "    \"\"\"FS-005-eps: Eps", // 2
            "",                          // 3
            "    Body.\"\"\"",           // 4
        ],
    );
    let record = record_of(&findings, &file);
    let zones: Vec<_> = record
        .zones
        .iter()
        .map(|zone| {
            (
                zone.line,
                &record.line(zone.line).unwrap()[zone.start..zone.end],
            )
        })
        .collect();
    assert_eq!(zones, vec![(2, "FS-005-eps: Eps"), (3, ""), (4, "Body.")]);
    assert_eq!(record.blocks[0].form, BlockForm::Docstring);
}

/// §AR-scanner.2.8.1: the record holds an open buffer itself, never a copy, and
/// a disk read's own string.
#[test]
fn a_buffer_is_shared_with_the_record() {
    let root = test_root("record_buffer");
    let path = root.join("docs/a.md");
    write(&path, "# On disk\n");
    let buffers = Buffers::new([(path.clone(), "# In the buffer\n".to_string())]);
    let sources = Sources::of(&buffers);
    let (FileText::Buffer(first), FileText::Buffer(second)) = (
        sources.shared_text(&path).expect("buffer"),
        sources.shared_text(&path).expect("buffer"),
    ) else {
        panic!("an open buffer is read as the buffer");
    };
    assert!(Arc::ptr_eq(&first, &second));
    let disk = Sources::on_disk().shared_text(&path).expect("disk");
    assert!(matches!(&disk, FileText::Disk(_)));
    assert_eq!(disk.as_str(), "# On disk\n");
}

/// The record's lines are `str::lines`' lines, a `\r` going only with its `\n`.
#[test]
fn record_lines_match_str_lines() {
    let text = "a\r\nb\rc\n\nd";
    let record = FileRecord::new(FileText::Disk(Arc::new(text.to_string())), true);
    let lines: Vec<_> = (1..=5).map(|line| record.line(line)).collect();
    let expected: Vec<_> = text.lines().map(Some).chain([None]).collect();
    assert_eq!(lines, expected);
    assert_eq!(record.line(0), None);
}
