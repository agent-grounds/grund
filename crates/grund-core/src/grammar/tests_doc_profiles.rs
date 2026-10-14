//! Test module: the v2 doc profiles as the epoch froze them for declarations
//! (§FS-inline-citation-style.1.1.4). Each profile's recognizer is pinned exactly,
//! starter sets included, so a measurement-only widening of the language table
//! (§FS-inline-citation-style.1.1.3) cannot widen what may declare unnoticed; and
//! each profile's stated corners are pinned on the block itself.

use super::comment_block::{
    CommentBlockKind, DefinitionStart, DocCommentRule, block_is_doc_comment,
};
use super::language_table::DocProfile;

/// §FS-inline-citation-style.1.1.4: every profile names one recognizer, and the
/// position profiles' starters are exactly the epoch's.
#[test]
fn each_profile_names_its_epoch_recognizer() {
    use DefinitionStart::{Keywords, LispDefinition, ShellFunction, SqlCreate};
    let expected = [
        (DocProfile::None, DocCommentRule::None),
        (DocProfile::CFamily, DocCommentRule::CFamily),
        (DocProfile::Python, DocCommentRule::Docstring),
        (
            DocProfile::Go,
            DocCommentRule::Position(Keywords(&["func", "type", "var", "const", "package"])),
        ),
        (
            DocProfile::Ruby,
            DocCommentRule::Position(Keywords(&["class", "module", "def"])),
        ),
        (DocProfile::Shell, DocCommentRule::Position(ShellFunction)),
        (DocProfile::Sql, DocCommentRule::Position(SqlCreate)),
        (DocProfile::Ldoc, DocCommentRule::LuaDoc),
        (DocProfile::Haddock, DocCommentRule::Haddock),
        (DocProfile::Roxygen, DocCommentRule::Roxygen),
        (DocProfile::Lisp, DocCommentRule::Position(LispDefinition)),
    ];
    assert_eq!(
        DocProfile::ALL.len(),
        expected.len(),
        "a new profile needs its pin"
    );
    for (profile, rule) in expected {
        assert_eq!(profile.rule(), rule, "doc profile `{}`", profile.name());
    }
}

fn line(marker: &str) -> CommentBlockKind {
    CommentBlockKind::Line(marker.to_string())
}

/// Whether `profile` calls a block of `kind` with `lines` documentation, the line
/// after it being `next` and the block not the file's leading one.
fn doc(profile: DocProfile, kind: &CommentBlockKind, lines: &[&str], next: &str) -> bool {
    block_is_doc_comment(profile.rule(), kind, lines, Some(next), false)
}

/// §FS-inline-citation-style.1.1.4 `c-family`: the dedicated markers qualify,
/// dangling or not; a plain comment and the lookalikes do not.
#[test]
fn c_family_admits_its_markers_and_no_lookalike() {
    let c = DocProfile::CFamily;
    assert!(doc(c, &line("///"), &["/// FS-a: A"], ""));
    assert!(doc(c, &line("//!"), &["//! FS-a: A"], "fn f() {}"));
    let block = CommentBlockKind::Block { closed: true };
    assert!(doc(c, &block, &["/** FS-a: A */"], "fn f() {}"));
    assert!(doc(c, &block, &["/*! FS-a: A */"], "fn f() {}"));
    assert!(!doc(c, &line("//"), &["// FS-a: A"], "fn f() {}"));
    assert!(!doc(c, &line("////"), &["//// FS-a: A"], "fn f() {}"));
    assert!(!doc(c, &block, &["/* FS-a: A */"], "fn f() {}"));
    assert!(!doc(c, &block, &["/**/"], "fn f() {}"));
}

/// §FS-inline-citation-style.1.1.4 `ldoc`, `haddock`, `roxygen`: the marker on the
/// run's first line decides.
#[test]
fn ldoc_haddock_and_roxygen_admit_exactly_their_markers() {
    let dashes = line("--");
    assert!(doc(DocProfile::Ldoc, &dashes, &["--- FS-a: A"], "local x"));
    assert!(!doc(
        DocProfile::Ldoc,
        &dashes,
        &["---- FS-a: A"],
        "local x"
    ));
    assert!(!doc(DocProfile::Ldoc, &dashes, &["-- FS-a: A"], "local x"));
    assert!(doc(
        DocProfile::Haddock,
        &dashes,
        &["-- | FS-a: A"],
        "f = 1"
    ));
    assert!(doc(DocProfile::Haddock, &dashes, &["--^ FS-a: A"], "f = 1"));
    assert!(!doc(DocProfile::Haddock, &dashes, &["-- FS-a: A"], "f = 1"));
    let hash = line("#");
    assert!(doc(DocProfile::Roxygen, &hash, &["#' FS-a: A"], "f <- 1"));
    assert!(!doc(DocProfile::Roxygen, &hash, &["# FS-a: A"], "f <- 1"));
}

/// §FS-inline-citation-style.1.1.4 `go`, `ruby`, `shell`, `sql`: the next line
/// opens one of the epoch's definitions, with no blank line between.
#[test]
fn position_profiles_admit_exactly_the_epoch_starters() {
    let (slashes, hash, dashes) = (line("//"), line("#"), line("--"));
    for next in [
        "func F() {}",
        "type T int",
        "var x = 1",
        "const c = 1",
        "package p",
    ] {
        assert!(
            doc(DocProfile::Go, &slashes, &["// FS-a: A"], next),
            "{next}"
        );
    }
    assert!(!doc(DocProfile::Go, &slashes, &["// FS-a: A"], "x := 1"));
    assert!(!doc(DocProfile::Go, &slashes, &["// FS-a: A"], ""));
    for next in ["class C", "module M", "  def f"] {
        assert!(doc(DocProfile::Ruby, &hash, &["# FS-a: A"], next), "{next}");
    }
    assert!(!doc(
        DocProfile::Ruby,
        &hash,
        &["# FS-a: A"],
        "private def f"
    ));
    for next in ["function f {", "f() {", "f () {"] {
        assert!(
            doc(DocProfile::Shell, &hash, &["# FS-a: A"], next),
            "{next}"
        );
    }
    assert!(!doc(DocProfile::Shell, &hash, &["# FS-a: A"], "echo f"));
    assert!(doc(
        DocProfile::Sql,
        &dashes,
        &["-- FS-a: A"],
        "CREATE OR REPLACE FUNCTION f"
    ));
    assert!(doc(
        DocProfile::Sql,
        &dashes,
        &["-- FS-a: A"],
        "create table t"
    ));
    assert!(!doc(
        DocProfile::Sql,
        &dashes,
        &["-- FS-a: A"],
        "alter table t"
    ));
}

/// §FS-config-v2.schema.sources.definitions `lisp`: v1's Lisp position recognizer,
/// a `;` block above a `def…`, `cl-def…` or `ns` form, spelled only by `;`.
#[test]
fn lisp_admits_a_block_above_a_definition_form() {
    let semi = line(";");
    for next in [
        "(defun f (x) x)",
        "(cl-defun f ())",
        "(ns app.core)",
        "(define x 1)",
    ] {
        assert!(doc(DocProfile::Lisp, &semi, &["; FS-a: A"], next), "{next}");
    }
    assert!(!doc(
        DocProfile::Lisp,
        &semi,
        &["; FS-a: A"],
        "(let ((x 1)) x)"
    ));
    assert_eq!(DocProfile::Lisp.hosts(), [";"]);
    assert_eq!(DocProfile::parse("lisp"), Some(DocProfile::Lisp));
}
