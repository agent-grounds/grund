//! A section lock file as text (§FS-section-lock.holds, §FS-section-lock.file):
//! where it lives, what one of its lines is, and how a set of entries is written.
//! What an entry should hold is a question about the tree, and is the resolver's.

use std::path::{Path, PathBuf};

use super::SectionInfo;

/// The directory under a config root that holds the lock files
/// (§FS-section-lock.file.shard).
pub(crate) const SECTION_LOCK_DIR: &str = ".grund/sections";

/// One lock file the scan read beside the walk: `id` the rendered ID its name
/// carries, which also places it (§FS-section-lock.file.shard), and `alias` the
/// project its citations go into when that is another one, named by the
/// directory it sits in (§FS-section-lock.file.members).
#[derive(Clone, Debug)]
pub(crate) struct SectionLockFile {
    pub(crate) alias: Option<String>,
    pub(crate) id: String,
    pub(crate) text: String,
}

/// One readable line of a lock file (§FS-section-lock.holds.entry): its 1-based
/// line, its section path below the file's ID, and its fingerprint, `None` for
/// the `-` of a text the declaration repeats (§FS-section-lock.holds.repeated).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LockEntry {
    pub(crate) line: usize,
    pub(crate) section: String,
    pub(crate) fingerprint: Option<String>,
}

/// A lock file read line by line: its entries, and the lines that are not one
/// (§FS-section-lock.checks.invalid-section-lock).
#[derive(Debug, Default)]
pub(crate) struct LockLines {
    pub(crate) entries: Vec<LockEntry>,
    pub(crate) invalid: Vec<usize>,
}

/// Whether `path`, a recorded section path, is a **mixed path** (§FS-config.3.3.1):
/// a numeric component followed, at any later depth, by a named one, as in
/// `1.goals`, `1.goals.3` and `goals.3.latency`. The one test of the shape, read
/// over the path's components and never over digits or letters in an ID: what the
/// lock enrols by ([`locked`]), what a citation's warning window selects
/// (§FS-citations.checks.mixed-path), what a doc-comment holds back until 0.19.0
/// (§FS-declarations.checks.parse-moves.2), and what a value root refuses
/// (§FS-values.3.1).
pub(crate) fn name_under_number(path: &str) -> bool {
    let numeric = |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
    let mut parts = path.split('.');
    parts.by_ref().any(numeric) && parts.any(|part| !part.is_empty() && !numeric(part))
}

/// Whether the section at `path`, recorded as `info`, is a **locked address**
/// once cited (§FS-section-lock.holds.entry): its heading is relative, or its
/// canonical path is mixed, whichever spelling declared it. The one predicate
/// enrolment, entry retention and the formatter's no-lock shortcut read
/// (§DF-number-name-sections.decision.enrolment).
pub(crate) fn locked(path: &str, info: &SectionInfo) -> bool {
    info.relative || name_under_number(path)
}

/// The lock file of `id` under `root` (§FS-section-lock.file.shard), in the
/// directory of `alias` for a citation into another project
/// (§FS-section-lock.file.members).
pub(crate) fn section_lock_path(root: &Path, alias: Option<&str>, id: &str) -> PathBuf {
    let dir = root.join(SECTION_LOCK_DIR);
    alias
        .map_or(dir.clone(), |alias| dir.join(alias))
        .join(format!("{id}.lock"))
}

/// Read `text`, the lock file of `id`: a line is an entry when it is
/// `<id><separator><path> <fingerprint>`, one space between, the fingerprint
/// sixteen lowercase hex digits or `-`; every other line, conflict markers and
/// blank lines included, is invalid (§FS-section-lock.holds.entry).
pub(crate) fn read_lock_lines(text: &str, id: &str, separator: &str) -> LockLines {
    let mut lines = LockLines::default();
    for (index, line) in text.lines().enumerate() {
        let entry = line.split_once(' ').and_then(|(coordinate, fingerprint)| {
            let section = coordinate.strip_prefix(id)?.strip_prefix(separator)?;
            let fingerprint = match fingerprint {
                "-" => None,
                hex if is_fingerprint(hex) => Some(hex.to_string()),
                _ => return None,
            };
            (!section.is_empty() && !section.contains(char::is_whitespace)).then(|| LockEntry {
                line: index + 1,
                section: section.to_string(),
                fingerprint,
            })
        });
        match entry {
            Some(entry) => lines.entries.push(entry),
            None => lines.invalid.push(index + 1),
        }
    }
    lines
}

/// Sixteen lowercase hex digits (§FS-section-lock.holds.fingerprint).
fn is_fingerprint(text: &str) -> bool {
    text.len() == 16
        && text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// A lock file's text for `entries`, `(coordinate, fingerprint)` pairs: one
/// line each, sorted bytewise by coordinate, each ending in LF
/// (§FS-section-lock.holds.entry).
pub(crate) fn render_lock(entries: &[(String, Option<String>)]) -> String {
    let mut lines = entries
        .iter()
        .map(|(coordinate, fingerprint)| {
            format!("{coordinate} {}\n", fingerprint.as_deref().unwrap_or("-"))
        })
        .collect::<Vec<_>>();
    lines.sort();
    lines.concat()
}

#[cfg(test)]
mod tests {
    use super::{LockEntry, name_under_number, read_lock_lines, render_lock};

    #[test]
    fn a_mixed_path_has_a_name_somewhere_under_a_number() {
        for path in [
            "1.goals",
            "1.goals.3",
            "1.goals.latency",
            "goals.3.latency",
            "2.1.x",
        ] {
            assert!(name_under_number(path), "`{path}` is mixed");
        }
        for path in [
            "1",
            "1.2",
            "goals",
            "goals.3",
            "format.goals",
            "goals.3.4",
            "",
        ] {
            assert!(!name_under_number(path), "`{path}` is not mixed");
        }
    }

    #[test]
    fn an_entry_is_a_coordinate_and_a_fingerprint_or_a_dash() {
        let lines = read_lock_lines(
            "FS-login.format.1 8f332b6fe87bb346\nFS-login.parse.1 -\n",
            "FS-login",
            ".",
        );
        assert_eq!(
            lines.entries,
            [
                LockEntry {
                    line: 1,
                    section: "format.1".to_string(),
                    fingerprint: Some("8f332b6fe87bb346".to_string()),
                },
                LockEntry {
                    line: 2,
                    section: "parse.1".to_string(),
                    fingerprint: None,
                },
            ]
        );
        assert!(lines.invalid.is_empty());
    }

    #[test]
    fn conflict_markers_and_foreign_lines_are_invalid() {
        let lines = read_lock_lines(
            "<<<<<<< HEAD\nFS-login.format.1 8f332b6fe87bb346\n=======\n\
             FS-other.format.1 8f332b6fe87bb346\nFS-login.format.1 8F332B6FE87BB346\n\
             FS-login.format.1  8f332b6fe87bb346\n>>>>>>> other\n",
            "FS-login",
            ".",
        );
        assert_eq!(lines.entries.len(), 1);
        assert_eq!(lines.invalid, [1, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn a_lock_is_written_sorted_by_coordinate() {
        let text = render_lock(&[
            ("FS-login.parse.1".to_string(), None),
            (
                "FS-login.format.1".to_string(),
                Some("8f332b6fe87bb346".to_string()),
            ),
        ]);
        assert_eq!(
            text,
            "FS-login.format.1 8f332b6fe87bb346\nFS-login.parse.1 -\n"
        );
    }
}
