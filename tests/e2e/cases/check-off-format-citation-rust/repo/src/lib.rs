//! The crate follows §FS-a.

// A comment cites §REQ-1x.
/// A doc-comment's inline code, `§REQ-2x`.
pub fn sites() -> [&'static str; 3] {
    ["string §REQ-3x", r"raw §REQ-4x", "escaped \u{a7}REQ-5x"]
}
