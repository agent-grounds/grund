//! Reading an `Id` back out of text (§FS-config.3.2): the two `Grammar`
//! operations over a match the compiled grammar produced or a CLI argument, and
//! the one parser that has no grammar to consult at all. The first two sat in
//! the model's record file while the crate was flat, calling
//! `Grammar::parse_token` and the `id_input_re` from there; they are grammar
//! operations over model's types, which is the direction §AR-system.4 allows, so
//! they live here rather than on the record. The `Config` they read the compiled
//! grammar off is config's own (§AR-system.2.3).
//!
//! The third, the member-local fallback of §FS-workspace.5, was parked in the
//! scanner's citation pass while §AR-system.2.5 was a file-name category; it is
//! a lexical reading of the conventional ID shape over plain text, with neither
//! scan state nor a compiled grammar, so it came down here with the move.

use anyhow::{Result, anyhow};

use super::compiled::Grammar;
use crate::model::Id;

/// Pull an `Id` out of a `Grammar` regex match — the `kind` / `num` / `slug`
/// capture groups the `[id] format` defined (§FS-config.3.2, §AR-scanner.2.1).
pub(crate) fn parse_id(caps: &regex::Captures, grammar: &Grammar) -> Option<Id> {
    if let Some(token) = caps.name("id") {
        return grammar.parse_token(token.as_str());
    }
    // Shorthand patterns retain direct component captures because the missing
    // slug is itself the signal carried into shorthand resolution.
    let kind = caps.name("kind")?.as_str().to_string();
    let num = match caps.name("num") {
        Some(m) => Some(m.as_str().parse().ok()?),
        None => None,
    };
    let slug = caps.name("slug").map(|m| m.as_str().to_string());
    Some(Id { kind, num, slug })
}

/// Parse a CLI `<ID>[.<section>]` argument (the form ID queries and `grund refs` take,
/// §FS-show.1, §FS-refs.1) into an `Id` and an optional section path (§FS-config.3.3).
pub(crate) fn parse_id_arg(raw: &str, grammar: &Grammar) -> Result<(Id, Option<String>)> {
    let caps = grammar
        .id_input_re
        .captures(raw)
        .ok_or_else(|| anyhow!("invalid ID `{raw}`"))?;
    let id = parse_id(&caps, grammar).ok_or_else(|| anyhow!("invalid ID `{raw}`"))?;
    Ok((id, caps.name("sec").map(|m| m.as_str().to_string())))
}

/// The member-local fallback ID parser (§FS-workspace.5). Recognises the
/// conventional `KIND[-NUM]-SLUG` shape — uppercase-or-digit kind, optional
/// numeric middle component, non-empty slug — because the member has no
/// access to the citing or target project's `[id] format` at this point.
/// A workspace-root run uses `parse_longest_id_prefix` with the target's
/// grammar (`scan_workspace_qualified_pass`) and is not affected by this
/// fallback's assumptions.
pub(crate) fn parse_loose_qualified_id_prefix(raw: &str) -> Option<(Id, Option<String>, usize)> {
    let mut end = raw
        .char_indices()
        .find(|(_, ch)| !(ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.')))
        .map(|(idx, _)| idx)
        .unwrap_or(raw.len());
    while end > 0
        && raw[..end]
            .chars()
            .next_back()
            .is_some_and(|ch| matches!(ch, '.' | ',' | ';' | ':' | '!' | '?'))
    {
        end -= raw[..end]
            .chars()
            .next_back()
            .map(char::len_utf8)
            .unwrap_or(1);
    }
    let token = raw.get(..end)?;
    let (id_text, section) = split_loose_section(token);
    let (kind, rest) = id_text
        .split_once(['-', '_'])
        .filter(|(kind, rest)| !kind.is_empty() && !rest.is_empty())?;
    if !kind
        .chars()
        .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit())
    {
        return None;
    }
    let (num, slug) = match rest.split_once(['-', '_']) {
        Some((maybe_num, slug)) if maybe_num.chars().all(|ch| ch.is_ascii_digit()) => {
            (maybe_num.parse::<u32>().ok(), slug)
        }
        _ => (None, rest),
    };
    if slug.is_empty() {
        return None;
    }
    Some((
        Id {
            kind: kind.to_string(),
            num,
            slug: Some(slug.to_string()),
        },
        section.map(str::to_string),
        end,
    ))
}

fn split_loose_section(token: &str) -> (&str, Option<&str>) {
    let suffix_start = token
        .char_indices()
        .rev()
        .find(|(_, ch)| !(ch.is_ascii_digit() || *ch == '.'))
        .map(|(idx, ch)| idx + ch.len_utf8())
        .unwrap_or(0);
    let suffix = &token[suffix_start..];
    let Some(section) = suffix.strip_prefix('.') else {
        return (token, None);
    };
    if section.is_empty()
        || !section
            .split('.')
            .all(|part| !part.is_empty() && part.chars().all(|ch| ch.is_ascii_digit()))
    {
        return (token, None);
    }
    (&token[..suffix_start], Some(section))
}
