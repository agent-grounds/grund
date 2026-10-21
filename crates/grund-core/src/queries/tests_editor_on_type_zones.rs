//! The on-type transform in a reference key (§FS-lsp.1.4,
//! §FS-citations.zones.authored-markup): it writes nothing there, as `grund fmt`
//! writes nothing there, and converts in a full reference's visible text, which
//! is no key. The key is read over the whole document, so the case is the
//! document the editor holds rather than the line alone.

use super::tests_editor_completion::Fixture;
use super::{DeclaredId, on_type_line_edits};

/// The edits for a trigger the author just finished typing on line 0 of
/// `text`, the cursor right after `token`.
fn edits_after(f: &Fixture, text: &str, token: &str) -> Vec<(usize, usize, String)> {
    let path = f.0.join("docs/notes.md");
    let declarations = vec![DeclaredId {
        path: &f.0,
        id: "FS-login",
    }];
    let line = text.lines().next().expect("a first line");
    let cursor = line.find(token).expect("the token is on line 0") + token.len();
    on_type_line_edits(&path, text, 0, cursor, &declarations)
        .expect("on-type edits")
        .into_iter()
        .map(|edit| (edit.start, edit.end, edit.text))
        .collect()
}

/// §FS-lsp.1.4, §FS-citations.zones.authored-markup: a trigger in a collapsed
/// reference's text, a full reference's key or a definition's label stays a
/// trigger, because each is a key its definition is found by; in a full
/// reference's visible text it converts, as `fmt` converts it there.
#[test]
fn on_type_leaves_a_trigger_in_a_reference_key_as_written() {
    let f = Fixture::new();
    for text in [
        "See [the $$FS-login record][].\n\n[the $$FS-login record]: x.md\n",
        "See [it][the $$FS-login key].\n\n[the $$FS-login key]: x.md\n",
        "[the $$FS-login record]: x.md\n\nSee [the $$FS-login record][].\n",
    ] {
        assert_eq!(
            edits_after(&f, text, "$$FS-login"),
            Vec::new(),
            "a key keeps its bytes: {text:?}"
        );
    }
    let text = "See [the $$FS-login rationale][pg].\n\n[pg]: x.md\n";
    assert_eq!(
        edits_after(&f, text, "$$FS-login"),
        vec![(9, 11, "\u{a7}".to_string())],
        "the visible text of a full reference is no key"
    );
}
