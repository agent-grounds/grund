//! Which row of the forms table and which quote reading one file reads
//! (§FS-languages.multi-line-string.suffix), held as one value: what the multi-line string
//! pre-pass and the qualified-comment search read a file by, and what the records gate
//! compares suffix by suffix (§FS-config-migrate.5.1). Which suffix names it is config's
//! answer (§AR-config.1.6); this file only says what a suffix names.

use super::multi_line_forms::Language;

/// The multi-line string row and the quote reading one file reads
/// (§FS-languages.multi-line-string.suffix). Rust's quote reading, a character or byte literal
/// as one unit and a lifetime or label as no quote at all, belongs to the Rust row, which only
/// `rs` names, so the row carries it; the Haskell prime is the one quote reading no row names.
/// Two values are equal exactly when they read a file alike, which is what the records gate
/// compares: `.cc` read by its own suffix and `.cc` mapped to the built-in `cpp` are equal.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct StringReading {
    /// The row of §FS-languages.multi-line-string.forms, `None` where the file reads none.
    pub(super) row: Option<Language>,
    /// Whether a `'` after an identifier character is a Haskell prime, which opens no literal
    /// in the comment search (§FS-languages.multi-line-string.suffix).
    pub(super) prime: bool,
}

impl StringReading {
    /// No row and the per-line quote toggle of §FS-fmt.2.3.1: what a Markdown file, a `document`
    /// or `data` definition's file, and a reader that asks no literal question read.
    pub(crate) const NONE: Self = Self {
        row: None,
        prime: false,
    };

    /// What `suffix` names (§FS-languages.multi-line-string.suffix): its row of the forms table,
    /// and the Haskell prime on `hs` and `lhs`, which no row names.
    pub(crate) fn of_suffix(suffix: &str) -> Self {
        Self {
            row: Language::of(suffix),
            prime: matches!(suffix, "hs" | "lhs"),
        }
    }
}
