//! The editor's two writers on the interior line of a string literal that spans
//! lines (§FS-languages.multi-line-string, §FS-lsp.1.4.3, §FS-lsp.1.6.1): the
//! on-type transform and completion refuse there exactly as `grund fmt` does,
//! and act again in a comment after the literal's closer.

use std::path::Path;

use super::tests_editor_completion::{Fixture, ids};
use super::{DeclaredId, on_type_line_edits};

/// A Rust raw string whose interior line is `line`, closed two lines later,
/// followed by a comment line `tail`.
fn raw_string_around(line: &str, tail: &str) -> String {
    format!("fn usage() -> &'static str {{\n    r#\"\n{line}\n\"#\n}}\n{tail}\n")
}

/// Replay `typed` one character at a time onto line `line_index` of the
/// document `before{line}{after}`, applying each keystroke's edits as an LSP
/// client would. The closer stays in `after` throughout, so the literal is
/// closed later in the file the whole time the line is being typed.
fn type_on_line(root: &Path, before: &str, typed: &str, after: &str) -> String {
    let path = root.join("src/lib.rs");
    let declarations = vec![DeclaredId {
        path: root,
        id: "FS-login",
    }];
    let line_index = before.lines().count();
    let mut line = String::new();
    for ch in typed.chars() {
        line.push(ch);
        let text = format!("{before}{line}{after}");
        let edits = on_type_line_edits(&path, &text, line_index, line.len(), &declarations)
            .expect("on-type edits");
        for edit in edits.iter().rev() {
            line.replace_range(edit.start..edit.end, &edit.text);
        }
    }
    line
}

#[test]
fn on_type_leaves_an_interior_line_of_a_raw_string_alone() {
    let f = Fixture::new();
    let before = "fn usage() -> &'static str {\n    r#\"\n";
    let after = "\n\"#\n}\n";
    assert_eq!(
        type_on_line(&f.0, before, "see $$FS-login here", after),
        "see $$FS-login here"
    );
    // The control: the same keystrokes in a comment after the closer convert.
    let before = "fn usage() -> &'static str {\n    r#\"\ntext\n\"#\n}\n";
    assert_eq!(
        type_on_line(&f.0, before, "// see $$FS-login here", "\n"),
        "// see \u{a7}FS-login here"
    );
}

#[test]
fn completion_offers_nothing_on_an_interior_line_of_a_raw_string() {
    let f = Fixture::new();
    let snapshot = f.load();
    let text = raw_string_around("$$FS-lo", "// done");
    assert!(
        f.items(&snapshot, "src/lib.rs", &text, 2, 7).is_empty(),
        "{text}"
    );
    let text = raw_string_around("text", "// $$FS-lo");
    assert_eq!(
        ids(&f.items(&snapshot, "src/lib.rs", &text, 5, 10)),
        ["FS-login", "FS-logout"]
    );
}
