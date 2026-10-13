//! grund's built-in language table (§FS-inline-citation-style.1.1.2,
//! §AR-scanner.4.2.1): every extension grund classifies, with the doc-comment
//! recognizer it takes. Two readers, one table: the scanner asks it which
//! recognizer a file gets, and the v1 `extensions` default reads every extension
//! it names (§FS-scan.scope.extensions.default).

use std::path::Path;

use super::comment_block::{DefinitionStart, DocCommentRule};

/// Go's definition keywords — `package` included, so a file-header comment above
/// the package clause is the module doc it looks like.
const GO_DEFINITION_STARTERS: &[&str] = &["func", "type", "var", "const", "package"];
/// Ruby's definition keywords. Leading whitespace is stripped before the test,
/// so an indented `def` inside a `class` still counts.
const RUBY_DEFINITION_STARTERS: &[&str] = &["class", "module", "def"];

const C_FAMILY: DocCommentRule = DocCommentRule::CFamily;
const GO: DocCommentRule =
    DocCommentRule::Position(DefinitionStart::Keywords(GO_DEFINITION_STARTERS));
const RUBY: DocCommentRule =
    DocCommentRule::Position(DefinitionStart::Keywords(RUBY_DEFINITION_STARTERS));
const SHELL: DocCommentRule = DocCommentRule::Position(DefinitionStart::ShellFunction);
const SQL: DocCommentRule = DocCommentRule::Position(DefinitionStart::SqlCreate);
const LISP: DocCommentRule = DocCommentRule::Position(DefinitionStart::LispDefinition);
/// A language read through a comment prefix alone (§FS-languages.prefixes): it
/// has a row, so the default reads it, and no doc-comment notion of its own.
const PREFIX_ONLY: DocCommentRule = DocCommentRule::None;

/// grund's built-in language table: every extension it classifies, each with its
/// doc-comment recognizer (§FS-inline-citation-style.1.1.2, §AR-scanner.4.2.1).
/// One row per extension, in the order the v1 `extensions` default lists them,
/// because that default is `md` followed by this table's extensions
/// (§FS-scan.scope.extensions.default): a language added here is read by
/// default in the same release, and is appended so every earlier default stays
/// a prefix. Extensions match case-sensitively, so `r` and `R` are two rows.
const LANGUAGE_TABLE: &[(&str, DocCommentRule)] = &[
    ("rs", C_FAMILY),
    ("go", GO),
    ("java", C_FAMILY),
    ("kt", C_FAMILY),
    ("ts", C_FAMILY),
    ("tsx", C_FAMILY),
    ("js", C_FAMILY),
    ("py", DocCommentRule::Docstring),
    ("c", C_FAMILY),
    ("cpp", C_FAMILY),
    ("swift", C_FAMILY),
    ("scala", C_FAMILY),
    ("rb", RUBY),
    ("php", C_FAMILY),
    ("cs", C_FAMILY),
    ("lisp", LISP),
    ("scm", LISP),
    ("clj", LISP),
    ("sql", SQL),
    ("hs", DocCommentRule::Haddock),
    ("lhs", DocCommentRule::Haddock),
    ("lua", DocCommentRule::LuaDoc),
    ("ada", PREFIX_ONLY),
    ("adb", PREFIX_ONLY),
    ("ads", PREFIX_ONLY),
    ("h", C_FAMILY),
    ("cc", C_FAMILY),
    ("cxx", C_FAMILY),
    ("hpp", C_FAMILY),
    ("hh", C_FAMILY),
    ("hxx", C_FAMILY),
    ("m", C_FAMILY),
    ("mm", C_FAMILY),
    ("kts", C_FAMILY),
    ("jsx", C_FAMILY),
    ("mjs", C_FAMILY),
    ("cjs", C_FAMILY),
    ("dart", C_FAMILY),
    // Both spellings an R source file is written with; the extension is the
    // only thing about R that is case-sensitive here.
    ("r", DocCommentRule::Roxygen),
    ("R", DocCommentRule::Roxygen),
    ("sh", SHELL),
    ("bash", SHELL),
    ("zsh", SHELL),
    ("cl", LISP),
    ("el", LISP),
    ("ss", LISP),
    ("rkt", LISP),
    ("cljs", LISP),
    ("cljc", LISP),
];

/// Every extension the built-in language table names, in table order — what
/// the v1 `extensions` default reads after `md` (§FS-scan.scope.extensions.default).
pub(crate) fn classified_extensions() -> impl Iterator<Item = &'static str> {
    LANGUAGE_TABLE.iter().map(|(extension, _)| *extension)
}

/// The doc-comment recognizer for one file, keyed on its extension: a lookup in
/// [`LANGUAGE_TABLE`] (§FS-inline-citation-style.1.1.2, §AR-scanner.4). Built in
/// and not configurable: what a note *is* must not differ between two installs
/// (§FS-non-goals.13). An extension the table does not name has no doc notion,
/// so every one of its comment blocks stays an inline site and nothing a tree
/// already passes changes.
pub(crate) fn doc_comment_rule(path: &Path) -> DocCommentRule {
    let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
        return DocCommentRule::None;
    };
    LANGUAGE_TABLE
        .iter()
        .find(|(named, _)| *named == extension)
        .map_or(DocCommentRule::None, |(_, rule)| *rule)
}
