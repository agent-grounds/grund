//! The two `Grammar` operations that read a regex match back as an `Id`
//! (§FS-config.3.2): one over a match the compiled grammar produced, one over a
//! CLI argument. They sat in the model's record file while the crate was flat,
//! calling `Grammar::parse_token` and the `id_input_re` from there; they are
//! grammar operations over model's types, which is the direction §AR-system.4
//! allows, so they live here rather than on the record. The `Config` they read
//! the compiled grammar off is config's own (§AR-system.2.3).

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
