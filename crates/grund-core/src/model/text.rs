//! The two text spellings every component shares (§AR-system.2.2): the JSON
//! string escape of §FS-errors.5 and the English list a message reads out.
//!
//! Both sat in the deprecated path's `output` category while the queries, the
//! writers and the api read them upward (§AR-system.4, §AR-system.2.9). Neither
//! is a renderer: a function of a `&str` in and a `String` out, with no stream,
//! no `Config` and no finding between them, which is §AR-system.2.2's own
//! description of what it holds.

/// One string escaped for a JSON document (§FS-errors.5): the five mandatory
/// escapes, and `\u00XX` for anything else the format calls a control character,
/// so a message carrying a tab or a newline stays one parsable value.
pub(crate) fn json_escape(raw: &str) -> String {
    let mut escaped = String::with_capacity(raw.len());
    for ch in raw.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            other if other.is_control() => escaped.push_str(&format!("\\u{:04x}", other as u32)),
            other => escaped.push(other),
        }
    }
    escaped
}

/// `a`, `b`, `c`, or `d` — a list spelled the way the message reads, joined by
/// the `conjunction` that message wants before the last item. Lives with the
/// other shared spellings rather than beside any one message: the refusals in
/// `writers/init_target.rs` and the duplicate-entrypoint note in
/// `writers/init_notes.rs` both spell a list, and neither owns the spelling
/// (§AR-core-module-layout.1).
pub(crate) fn format_list(items: &[&str], conjunction: &str) -> String {
    match items {
        [] => String::new(),
        [only] => (*only).to_string(),
        [first, second] => format!("{first} {conjunction} {second}"),
        [rest @ .., last] => format!("{}, {conjunction} {last}", rest.join(", ")),
    }
}
