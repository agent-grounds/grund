//! Test module: §AR-checker.placement names every checker rule that reads a
//! file's text after the scan, so the one answer to "which code must take the
//! editor's text before the disk" is the code's answer.
//!
//! The list in the placement section is prose, and prose that counts its readers
//! has drifted twice: a rule started reading and nobody recounted, and a read
//! moved into the scanner while the sentence calling it the only one stayed. So
//! the readers are found in the code instead. Every non-test file of this
//! component is searched for the calls that read a file's text, each one is
//! matched to the point that owns it through `READERS`, and the placement must
//! cite every owner. An existence probe or a canonicalized scope path is not a
//! read here: there is no text an overlay could stand in for.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The calls that read a file's text: the shared read of an effective input
/// (§FS-check.6.1.1) or the standard library's, the scanner's reading of a
/// stub's target (§AR-scanner.4.6), and the resolver's point-body slicer.
const READ_CALLS: &[&str] = &[
    "input_read_to_string",
    "read_to_string",
    "fs::read",
    "File::open",
    "file_declares_inline_home",
    "point_body_pair",
];

/// Which point owns the reads each checker file makes, by the call it makes them
/// through. A read the table does not list fails the test, and so does a row
/// whose read has gone.
const READERS: &[(&str, &str, &str)] = &[
    // The inline-code hint on a dangling Markdown citation.
    ("support.rs", "input_read_to_string", "AR-checker.2.3"),
    ("report.rs", "file_declares_inline_home", "AR-checker.2.5"),
    ("agents.rs", "input_read_to_string", "AR-checker.2.7"),
    ("index.rs", "input_read_to_string", "AR-checker.2.16"),
    (
        "index_entries.rs",
        "input_read_to_string",
        "AR-checker.2.16",
    ),
    // The opt-in lead budget, which reads each home through the resolver's slicer.
    (
        "sizes.rs",
        "point_body_pair",
        "FS-declarations.checks.oversized-lead",
    ),
];

fn component_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/checker")
}

/// Every `.rs` file under `dir` that is not a test module, by path below the
/// component.
fn source_files(dir: &Path, below: &str, out: &mut Vec<(String, String)>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("read {}: {err}", dir.display()))
        .map(|entry| entry.expect("directory entry").path())
        .collect();
    entries.sort();
    for path in entries {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let relative = format!("{below}{name}");
        if path.is_dir() {
            source_files(&path, &format!("{relative}/"), out);
        } else if name.ends_with(".rs") && !name.starts_with("tests_") {
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
            out.push((relative, text));
        }
    }
}

/// The read calls on one line of code, each named as `READ_CALLS` spells it. A
/// call counts only where its name starts a word, so `input_read_to_string(`
/// is not also a `read_to_string(`.
fn read_calls_on(line: &str) -> Vec<&'static str> {
    let mut calls = Vec::new();
    for call in READ_CALLS {
        let needle = format!("{call}(");
        let mut from = 0;
        while let Some(at) = line[from..].find(&needle) {
            let start = from + at;
            let before = line[..start].chars().next_back();
            if !before.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_') {
                calls.push(*call);
            }
            from = start + needle.len();
        }
    }
    calls
}

/// The placement section of `report.rs`, from its heading to the next chapter,
/// with the doc-comment markers taken off.
fn placement_section(report: &str) -> String {
    let body = report
        .lines()
        .map(|line| line.trim_start().trim_start_matches("///"));
    let mut section = String::new();
    let mut inside = false;
    for line in body {
        let heading = line.trim_start();
        if heading.starts_with("## placement:") {
            inside = true;
        } else if inside && heading.starts_with("## ") {
            break;
        } else if inside {
            section.push_str(line);
            section.push('\n');
        }
    }
    assert!(
        !section.is_empty(),
        "no `## placement:` chapter found in report.rs"
    );
    section
}

/// Every ID a `§` cites in `text`, sections included, without a trailing period.
fn cited_ids(text: &str) -> BTreeSet<String> {
    text.split('\u{a7}')
        .skip(1)
        .map(|rest| {
            let id: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_' | '/'))
                .collect();
            id.trim_end_matches('.').to_string()
        })
        .collect()
}

/// §AR-checker.placement: the section names the owner of every read of a file's
/// text the checker makes after the scan, and `READERS` is exactly the reads the
/// code makes.
#[test]
fn the_placement_names_every_rule_that_reads_a_file_after_the_scan() {
    let mut files = Vec::new();
    source_files(&component_dir(), "", &mut files);
    let mut found = BTreeSet::new();
    let mut problems = Vec::new();
    for (file, text) in &files {
        for (index, line) in text.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            for call in read_calls_on(line) {
                found.insert((file.as_str(), call));
                if !READERS.iter().any(|(f, c, _)| f == file && *c == call) {
                    problems.push(format!(
                        "checker/{file}:{} reads a file's text through `{call}`, and READERS \
                         names no point that owns it: name the rule in \u{a7}AR-checker.placement \
                         and add its row",
                        index + 1
                    ));
                }
            }
        }
    }
    for (file, call, _) in READERS {
        if !found.contains(&(*file, *call)) {
            problems.push(format!(
                "READERS lists checker/{file} reading through `{call}`, and it no longer does: \
                 take the row out, and the rule out of the placement if nothing else of it reads"
            ));
        }
    }
    let report = files
        .iter()
        .find(|(file, _)| file == "report.rs")
        .map(|(_, text)| text.as_str())
        .expect("checker/report.rs");
    let cited = cited_ids(&placement_section(report));
    let owners: BTreeSet<&str> = READERS.iter().map(|(_, _, owner)| *owner).collect();
    for owner in owners {
        if !cited.contains(owner) {
            problems.push(format!(
                "the placement section in checker/report.rs does not cite \u{a7}{owner}, which \
                 reads a file's text after the scan"
            ));
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
