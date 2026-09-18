//! An `Id` and text, in both directions (§FS-config.3.2): the two `Grammar`
//! operations that read one out of a match the compiled grammar produced or out
//! of a CLI argument, the one parser that has no grammar to consult at all, the
//! two renderers that print one back, and the record of one citation token found
//! on a Markdown line. The first two sat in the model's record file while the
//! crate was flat, calling `Grammar::parse_token` and the `id_input_re` from
//! there; they are grammar operations over model's types, which is the direction
//! §AR-system.4 allows, so they live here rather than on the record. The
//! `Config` they read the compiled grammar off is config's own
//! (§AR-system.2.3).
//!
//! The member-local fallback of §FS-workspace.5 was parked in the scanner's
//! citation pass while §AR-system.2.5 was a file-name category; it is a lexical
//! reading of the conventional ID shape over plain text, with neither scan state
//! nor a compiled grammar, so it came down here with the move.
//!
//! The last three came down out of the writers when §AR-system.2.8 became a
//! module. `render_id` is one line over `Grammar::render` that the scanner, the
//! checker, the workspace, the queries and this component all read, and
//! `render_qualified_id` is the same line with an alias in front of it
//! (§FS-workspace.1) — it sat on the model's record and was the last reason that
//! file read `Config`. `MarkdownLineCitation` is where one marked citation
//! starts and ends on a line, plus what it parsed to: a lexical fact the
//! formatter's link pass, its shorthand pass and the scanner's off-grammar pass
//! each record and only the link pass reads (§FS-fmt.6.2).

use anyhow::{Result, anyhow};

use super::compiled::Grammar;
use crate::model::Id;
// §AR-system.4: one upward read — `Config` is config's record, above this
// component, and the two renderers below read the compiled grammar off it.
use crate::config::Config;

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

/// One marked citation token as it sits on a Markdown line: where the marker
/// starts, where the token ends, the project alias it named if it was qualified
/// (§FS-workspace.1), and what it parsed to. Three passes record these — the
/// formatter's link pass, its accepted-shorthand pass (§FS-fmt.2.4) and the
/// scanner's off-grammar pass (§FS-config.3.2) — and the wrapper of §FS-fmt.6.2
/// reads them, which is why the record is here rather than with any one of them.
pub(crate) struct MarkdownLineCitation {
    pub(crate) marker_start: usize,
    pub(crate) token_end: usize,
    pub(crate) namespace: Option<String>,
    pub(crate) id: Id,
    pub(crate) section: Option<String>,
}

/// Render an existing `Id` for a report, listing, or message, preserving the
/// provider spelling policy of a per-kind override (§FS-config.3.2).
pub(crate) fn render_id(config: &Config, id: &Id) -> String {
    config.grammar.render(id, 3)
}

/// The same spelling with the project alias a qualified citation writes in front
/// of it (§FS-workspace.1), and the bare one when there is no alias.
pub(crate) fn render_qualified_id(config: &Config, namespace: Option<&str>, id: &Id) -> String {
    match namespace {
        Some(namespace) => format!("{}/{}", namespace, render_id(config, id)),
        None => render_id(config, id),
    }
}
