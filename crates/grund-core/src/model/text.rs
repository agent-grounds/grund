//! The text spellings every component shares (§AR-system.2.2): the JSON string
//! escape of §FS-errors.5, the English list a message reads out, the plural
//! `s` a count earns, and the repair a forbidden-citation error ends with
//! (§FS-check.3.12).
//!
//! The first two sat in the deprecated path's `output` category while the
//! queries, the writers and the api read them upward (§AR-system.4,
//! §AR-system.2.9.1). None is a renderer: a function of a `&str` or a number in
//! and a `String` out, with no stream, no `Config` and no finding between them,
//! which is §AR-system.2.2's own description of what it holds.
//!
//! `plural` came here when §AR-system.2.11 became a component. It was the
//! writers' template renderer's, then the checker's as the lowest component
//! that read it; the sentence the managed block teaches is the templates' now
//! (§FS-init.2.3), and the two readers sit in components neither of which is
//! below the other, so the spelling goes all the way down (§AR-system.4).

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

/// The plural `s` a count earns, or nothing at one — spelled once, because the
/// inline-size findings of §FS-inline-citation-style.4 and the budget sentence
/// the agent entrypoint teaches (§FS-init.2.3) have to read the same way.
pub(crate) fn plural(value: usize) -> &'static str {
    if value == 1 { "" } else { "s" }
}

/// The repair §FS-check.3.12 appends to every `forbidden-citation` error, so the
/// finding says what to do about the offence and not only that it happened. A
/// rule sentence reuses these bytes with `(<RULE-ID>)` in place of the
/// `(citation direction)` authority tail and nothing else changed
/// (§FS-rules.7.5). Lives here rather than beside either message: the
/// citation-direction pass is the checker's and the rule sentence is the rules',
/// and the rules sit below the checker, so the spelling goes all the way down
/// (§AR-system.4).
pub(crate) const CITATION_DIRECTION_REPAIR: &str =
    " — re-point the citation or downgrade it to a plain Markdown link";
