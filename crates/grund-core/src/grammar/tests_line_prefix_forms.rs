//! Test module: the forms a configured `--` or `;` stands for
//! (§FS-languages.prefixes.forms, §AR-scanner.4.3) — `---`, the Haddock `-- |` and
//! `-- ^`, and any run of semicolons — read at the per-line seam every
//! declaration reader shares, the comment wrapper of a `;` run
//! (§FS-languages.comment-wrapper), and the Lisp definition-starters of the
//! position row (§FS-inline-citation-style.1.1.2). The whole-tree reading is
//! pinned by the e2e cases `declare-line-prefix-forms*` and `inline-site-lisp`.

use super::{comment_envelope, declaration_id_on_line, render_id};
use crate::model::BlockForm;
use crate::testing::{inline_style_findings, numbered_config, over_the_line_cap, test_root};

/// The ID a comment-prefixed source line declares, or `None`.
fn declared(name: &str, line: &str) -> Option<String> {
    let mut config = numbered_config(test_root(name));
    config.rebuild_grammar().expect("rebuild the test grammar");
    declaration_id_on_line(&config.grammar, line, false, false)
        .map(|(id, _)| render_id(&config.grammar, &id))
}

/// `--` stands for `---` and the Haddock markers, with or without the space,
/// as `//` stands for `///` and `//!`.
#[test]
fn every_dash_form_declares_on_its_line() {
    for line in [
        "-- FS-042-user-login: the user logs in",
        "--- FS-042-user-login: the user logs in",
        "-- | FS-042-user-login: the user logs in",
        "--| FS-042-user-login: the user logs in",
        "-- ^ FS-042-user-login: the user logs in",
        "--^ FS-042-user-login: the user logs in",
        "  -- ^ FS-042-user-login: an indented record field's Haddock",
    ] {
        assert_eq!(
            declared("every_dash_form_declares_on_its_line", line).as_deref(),
            Some("FS-042-user-login"),
            "line {line:?}"
        );
    }
}

/// `;` stands for any run of semicolons, the Lisp file header's `;;;` and the
/// outline heading's `;;;;` included.
#[test]
fn every_semicolon_run_declares_on_its_line() {
    for line in [
        "; FS-042-user-login: the user logs in",
        ";; FS-042-user-login: the user logs in",
        ";;; FS-042-user-login: the user logs in",
        ";;;; FS-042-user-login: the user logs in",
        "  ;; FS-042-user-login: an indented definition's comment",
    ] {
        assert_eq!(
            declared("every_semicolon_run_declares_on_its_line", line).as_deref(),
            Some("FS-042-user-login"),
            "line {line:?}"
        );
    }
}

/// `----` is a rule line, as `////` is, and anything else glued to the prefix
/// is no form of it.
#[test]
fn a_rule_line_or_a_glued_character_declares_nothing() {
    for line in [
        "---- FS-042-user-login: a rule line, not a doc comment",
        "-- * FS-042-user-login: no Haddock marker",
        ";x FS-042-user-login: not a run of semicolons",
        ";;FS-042-user-login: no space after the run",
    ] {
        assert_eq!(
            declared("a_rule_line_or_a_glued_character_declares_nothing", line),
            None,
            "line {line:?}"
        );
    }
}

/// The wrapper of a `;` line is its whole run of semicolons and one space, so
/// `show` prints no leftover `;` (§FS-languages.comment-wrapper).
#[test]
fn the_wrapper_of_a_semicolon_line_is_its_whole_run() {
    let form = BlockForm::Line(";".to_string());
    for (line, text) in [
        ("; One route per path.", "One route per path."),
        (";; One route per path.", "One route per path."),
        (";;;; One route per path.", "One route per path."),
        ("  ;; ## 1. Contract", "## 1. Contract"),
        (";;   indented", "  indented"),
        (";;", ""),
    ] {
        assert_eq!(comment_envelope(line, &form), text, "line {line:?}");
    }
}

/// A four-line `;;` block that cites, directly above `next`, in a `.lisp` file.
fn lisp_block_above(name: &str, next: &str) -> Vec<(usize, String)> {
    let source = format!(
        concat!(
            "(in-package :app)\n",
            "\n",
            ";; Documents \u{a7}FS-001-login in a sentence\n",
            ";; that wraps to a second line,\n",
            ";; and a third,\n",
            ";; and a fourth.\n",
            "{}\n",
        ),
        next
    );
    inline_style_findings(name, "src/app.lisp", &source)
}

/// A Lisp definition-starter is a form whose head symbol starts with `def` or
/// `cl-def`, or is `ns`; the block above one is documentation and is not
/// measured. `(default-value` is the accepted corner of recognition, not
/// parsing (§FS-inline-citation-style.1.1.3).
#[test]
fn a_lisp_block_above_a_definition_is_documentation() {
    for next in [
        "(defun route (p) p)",
        "(defn- total [items] (reduce + items))",
        "(define (route p) p)",
        "(defmacro with-route (&body body) `(progn ,@body))",
        "(cl-defun route (&key path) path)",
        "(ns app.core)",
        "  (defvar *routes* nil)",
        "(default-value 'fill-column)",
    ] {
        assert_eq!(
            lisp_block_above("a_lisp_block_above_a_definition_is_documentation", next),
            Vec::new(),
            "starter {next:?}"
        );
    }
}

/// Any other form under the block leaves it a note, measured by the cap.
#[test]
fn a_lisp_block_above_anything_else_is_a_note() {
    for next in ["(let ((p 1)) p)", "(nsx-route)", "(route \"/\")", "x"] {
        assert_eq!(
            lisp_block_above("a_lisp_block_above_anything_else_is_a_note", next),
            over_the_line_cap(3, 4, "\u{a7}FS-001-login"),
            "next line {next:?}"
        );
    }
}

/// The file's leading `;;;` header is its module doc, as a Go file header is.
#[test]
fn a_leading_lisp_header_is_documentation() {
    let findings = inline_style_findings(
        "a_leading_lisp_header_is_documentation",
        "src/app.lisp",
        concat!(
            ";;; app.lisp --- documents \u{a7}FS-001-login in a sentence\n",
            ";;; that wraps to a second line,\n",
            ";;; and a third,\n",
            ";;; and a fourth.\n",
            "\n",
            "(in-package :app)\n",
        ),
    );
    assert_eq!(findings, Vec::new());
}
