//! Black-box: in a source file an off-format citation is reported exactly where a marked
//! citation written the same way is read, in comments, a doc-comment's inline code, string
//! literals and raw strings, and is silent where such a citation is: behind a marker spelled as
//! an escape sequence and in an assigned Python data span (§FS-citations.recognized.off-format,
//! §FS-citations.recognized.string-literals). Each red test has a guard that runs today: the same
//! file with in-format tokens, which dangle at exactly the lines the red test expects.

#[path = "support/off_format_citation.rs"]
mod support;

use support::*;

/// A Python string's spelled-out marker, built in two halves so this file holds none.
const PY_ESCAPE: &str = concat!("\\", "u00a7");

/// A Rust file citing one `REQ` token per site: a comment, a doc-comment's inline code, a string,
/// a raw string, and a string whose marker is an escape sequence.
fn rust(tokens: [&str; 5]) -> String {
    let [comment, doc, string, raw, escaped] = tokens;
    format!(
        "//! Crate doc citing \u{a7}FS-a.\n\
         // A comment cites \u{a7}REQ-{comment}.\n\
         /// A doc-comment's inline code cites `\u{a7}REQ-{doc}`.\n\
         pub fn string() -> &'static str {{ \"a string cites \u{a7}REQ-{string}\" }}\n\
         pub fn raw() -> &'static str {{ r\"a raw string cites \u{a7}REQ-{raw}\" }}\n\
         pub fn escaped() -> &'static str {{ \"an escaped marker \\u{{a7}}REQ-{escaped}\" }}\n"
    )
}

/// A Python file citing one `REQ` token per site: a docstring, an assigned data span, a string,
/// a comment, and a string whose marker is an escape sequence.
fn python(tokens: [&str; 5]) -> String {
    let [docstring, data, string, comment, escaped] = tokens;
    format!(
        "\"\"\"A docstring cites \u{a7}FS-a and \u{a7}REQ-{docstring}.\"\"\"\n\
         DATA = \"\"\"\nassigned data \u{a7}REQ-{data}\n\"\"\"\n\
         x = \"a string cites \u{a7}REQ-{string}\"\n\
         # A comment cites \u{a7}REQ-{comment}.\n\
         y = \"an escaped marker {PY_ESCAPE}REQ-{escaped}\"\n"
    )
}

fn dangling(path: &str, sites: &[(usize, &str)]) -> String {
    sites
        .iter()
        .map(|(line, id)| text(path, *line, "error", &format!("unknown reference {id}")))
        .collect()
}

fn off_format_at(path: &str, sites: &[(usize, &str)]) -> String {
    sites
        .iter()
        .map(|(line, token)| text(path, *line, "error", &off_format(token, REQ_FORMAT)))
        .collect()
}

/// Guard: every Rust site but the escaped marker is read today, as a dangling citation.
#[test]
fn rust_sites_are_read_as_citations() {
    let fx = Fixture::source("src/lib.rs", &rust(["a1", "a2", "a3", "a4", "a5"]));
    let run = fx.expect(1, &["check"]);
    let sites = [(2, "REQ-a1"), (3, "REQ-a2"), (4, "REQ-a3"), (5, "REQ-a4")];
    assert_eq!(run.stdout, dangling("src/lib.rs", &sites));
}

/// The same Rust sites report an off-format token, and the escaped marker stays silent.
#[test]
#[ignore = "red until #1073 reports an off-format citation; implement removes this"]
fn rust_sites_report_an_off_format_token() {
    let fx = Fixture::source("src/lib.rs", &rust(["1a", "2a", "3a", "4a", "5a"]));
    let run = fx.expect(1, &["check"]);
    let sites = [(2, "REQ-1a"), (3, "REQ-2a"), (4, "REQ-3a"), (5, "REQ-4a")];
    assert_eq!(run.stdout, off_format_at("src/lib.rs", &sites));
    assert_eq!(run.stderr, "");
}

/// Guard: the docstring, the string and the comment are read today; the assigned data span and
/// the escaped marker are not (§FS-citations.recognized.string-literals.assigned-data).
#[test]
fn python_sites_are_read_as_citations() {
    let fx = Fixture::source("src/m.py", &python(["a1", "a2", "a3", "a4", "a5"]));
    let run = fx.expect(1, &["check"]);
    let sites = [(1, "REQ-a1"), (5, "REQ-a3"), (6, "REQ-a4")];
    assert_eq!(run.stdout, dangling("src/m.py", &sites));
}

/// The same Python sites report an off-format token, and the data span and escape stay silent.
#[test]
#[ignore = "red until #1073 reports an off-format citation; implement removes this"]
fn python_sites_report_an_off_format_token() {
    let fx = Fixture::source("src/m.py", &python(["1a", "2a", "3a", "4a", "5a"]));
    let run = fx.expect(1, &["check"]);
    let sites = [(1, "REQ-1a"), (5, "REQ-3a"), (6, "REQ-4a")];
    assert_eq!(run.stdout, off_format_at("src/m.py", &sites));
    assert_eq!(run.stderr, "");
}
