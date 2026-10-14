//! grund's built-in language table (§FS-inline-citation-style.1.1.2,
//! §AR-scanner.4.2.1): every extension grund classifies, with the doc-comment
//! recognizer it takes. Two readers, one table: the scanner asks it which
//! recognizer a file gets, and the v1 `extensions` default reads every extension
//! it names (§FS-scan.scope.extensions.default).

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

/// The doc-comment recognizer the table gives `extension`: a lookup in
/// [`LANGUAGE_TABLE`] (§FS-inline-citation-style.1.1.2, §AR-scanner.4). Built in
/// and not configurable: what a note *is* must not differ between two installs
/// (§FS-non-goals.13). An extension the table does not name has no doc notion,
/// so every one of its comment blocks stays an inline site and nothing a tree
/// already passes changes. Which extension a file is asked under is config's
/// answer (`FileReading`), never a path's alone.
pub(crate) fn extension_rule(extension: &str) -> DocCommentRule {
    LANGUAGE_TABLE
        .iter()
        .find(|(named, _)| *named == extension)
        .map_or(DocCommentRule::None, |(_, rule)| *rule)
}

/// Go's and Ruby's definition starters as they stood at the v2 epoch, frozen for
/// the declarations their profiles admit (§FS-inline-citation-style.1.1.4). The
/// table's sets above may widen for measurement (§FS-inline-citation-style.1.1.3);
/// these are a profile's membership and change only with a new profile.
const GO_EPOCH_STARTERS: &[&str] = &["func", "type", "var", "const", "package"];
const RUBY_EPOCH_STARTERS: &[&str] = &["class", "module", "def"];

/// A frozen doc-comment recognizer a v2 language names, for the declarations it
/// admits (§FS-config-v2.schema.sources.epoch, §FS-inline-citation-style.1.1.4).
/// Each profile is one of the recognizers above, fixed as it stood at the v2
/// epoch; `None` is a language with no notion of documentation, every comment of
/// which may declare (§FS-config-v2.schema.sources.declarations.eligible).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DocProfile {
    None,
    CFamily,
    Python,
    Go,
    Ruby,
    Shell,
    Sql,
    Ldoc,
    Haddock,
    Roxygen,
}

impl DocProfile {
    /// Every profile, in the order `[schema.sources.definitions]` names them.
    pub(crate) const ALL: [DocProfile; 10] = [
        Self::None,
        Self::CFamily,
        Self::Python,
        Self::Go,
        Self::Ruby,
        Self::Shell,
        Self::Sql,
        Self::Ldoc,
        Self::Haddock,
        Self::Roxygen,
    ];

    /// The name a v2 file writes for this profile.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::CFamily => "c-family",
            Self::Python => "python",
            Self::Go => "go",
            Self::Ruby => "ruby",
            Self::Shell => "shell",
            Self::Sql => "sql",
            Self::Ldoc => "ldoc",
            Self::Haddock => "haddock",
            Self::Roxygen => "roxygen",
        }
    }

    pub(crate) fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|profile| profile.name() == name)
    }

    /// The recognizer this profile asks (§FS-inline-citation-style.1.1.4): the
    /// one the v1 table gave the language's own extensions at the v2 epoch, built
    /// from this profile's own starter sets so that widening the table's for
    /// measurement never widens what may declare.
    pub(crate) fn rule(self) -> DocCommentRule {
        match self {
            Self::None => DocCommentRule::None,
            Self::CFamily => DocCommentRule::CFamily,
            Self::Python => DocCommentRule::Docstring,
            Self::Go => DocCommentRule::Position(DefinitionStart::Keywords(GO_EPOCH_STARTERS)),
            Self::Ruby => DocCommentRule::Position(DefinitionStart::Keywords(RUBY_EPOCH_STARTERS)),
            Self::Shell => DocCommentRule::Position(DefinitionStart::ShellFunction),
            Self::Sql => DocCommentRule::Position(DefinitionStart::SqlCreate),
            Self::Ldoc => DocCommentRule::LuaDoc,
            Self::Haddock => DocCommentRule::Haddock,
            Self::Roxygen => DocCommentRule::Roxygen,
        }
    }

    /// The comment prefix whose forms spell this profile's marker or position
    /// block, one of which a definition's prefixes must hold for the profile to
    /// be reachable (§FS-config-v2.schema.sources.definitions); empty for the
    /// profiles no prefix spells.
    pub(crate) fn hosts(self) -> &'static [&'static str] {
        match self {
            Self::None | Self::Python => &[],
            Self::CFamily => &["//", "/*"],
            Self::Go => &["//"],
            Self::Ruby | Self::Shell | Self::Roxygen => &["#"],
            Self::Sql | Self::Ldoc | Self::Haddock => &["--"],
        }
    }
}
