//! The proposed ID a `grund id` run allocates (§FS-id): the slug derived from
//! the title, and the rendering of the allocated `Id` under its effective
//! `[id] format`. The data half of §FS-id — `api.rs`'s `propose_id` and the
//! deprecated `id_cmd.rs` adapter both ask these two questions and then print
//! the answer their own way (§AR-system.2.9).
//!
//! `render_id`, which sat in this file while the crate was flat, is not part of
//! the proposal: it prints an existing `Id` for any report, listing or message,
//! and every component below the writers read it upward. It came down into
//! `grammar/ids.rs` with this move (§AR-system.4).

use regex::Regex;
use unicode_normalization::UnicodeNormalization;

use crate::config::Config;
use crate::model::Id;

/// The repeating character class of a slug pattern — the last `[...]` bracket
/// expression in `slug_pattern` (e.g. `[a-z0-9-]` from `[a-z0-9][a-z0-9-]*`) —
/// used when slugifying a `grund id` title so the result fits the configured
/// `[id] slug_pattern` (§FS-id.3, §FS-config.3.2). Falls back to the canonical
/// default if the pattern has no bracket expression.
fn slug_char_class(slug_pattern: &str) -> String {
    if let Some(end) = slug_pattern.rfind(']')
        && let Some(start) = slug_pattern[..end].rfind('[')
    {
        return slug_pattern[start..=end].to_string();
    }
    "[a-z0-9-]".to_string()
}

/// Derive a slug from a `grund id` title (§FS-id.3).
pub(crate) fn slugify_title(title: &str, slug_pattern: &str) -> String {
    // §FS-id.3: NFKD-normalize, drop combining marks, lower-case to ASCII, then
    // replace every run of characters outside the configured slug character class
    // with a single `-`; trim, collapse, truncate to 60 at a `-` boundary.
    let class = slug_char_class(slug_pattern);
    let valid = Regex::new(&format!("^(?:{class})$"))
        .unwrap_or_else(|_| Regex::new("^(?:[a-z0-9-])$").unwrap());
    let mut buf = [0u8; 4];
    let mut out = String::new();
    let mut last_dash = false;
    for ch in title.nfkd() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii() && valid.is_match(lower.encode_utf8(&mut buf)) {
            out.push(lower);
            last_dash = false;
        } else if !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    while out.contains("--") {
        out = out.replace("--", "-");
    }
    if out.len() > 60 {
        let mut truncated = out[..60].to_string();
        if let Some(cut) = truncated.rfind('-') {
            truncated.truncate(cut);
        }
        out = truncated;
    }
    out
}

/// Render a newly allocated `Id` under its effective format, zero-padding the
/// number to `width` (§FS-config.3.2, §FS-id.2).
pub(crate) fn format_id(id: &Id, config: &Config, width: usize) -> String {
    config.grammar.render_allocation(id, width)
}
