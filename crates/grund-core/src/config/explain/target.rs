//! §FS-config-explain.1.1: what a target names in one version, read against
//! the tables and keys that version's reader admits, and the code catalog.

use super::{ExplainUnknown, is_member_of};
use crate::config::v2::v1_homes;
use crate::config::vocabulary::admitted;
use crate::model::Code;

/// What a target names in one version.
pub(super) enum Resolved<'a> {
    /// The admitted table at this index.
    Table(usize),
    /// The admitted table at this index and the key of it, as the gate spells it.
    Key(usize, &'a str),
    Code(Code),
    Unknown(ExplainUnknown),
}

/// §FS-config-explain.1.1: a target starting with `[` is a table, or a key or
/// a member of one; anything else is a top-level key the version admits, and
/// otherwise a check code.
pub(super) fn resolve<'a>(
    version: u32,
    tables: &'a [(String, Vec<String>)],
    asked: &str,
) -> Resolved<'a> {
    let asked = asked.trim();
    let Some((header, key)) = split(asked) else {
        if let Some(key) = find_key(&tables[0].1, asked) {
            return Resolved::Key(0, key);
        }
        return match Code::named(asked) {
            Some(code) => Resolved::Code(code),
            None => Resolved::Unknown(dotted(tables, asked).unwrap_or(ExplainUnknown::Other {
                target: asked.to_string(),
            })),
        };
    };
    let Some(at) = find_table(tables, header) else {
        return Resolved::Unknown(unknown_table(version, header, asked));
    };
    if key.is_empty() {
        return Resolved::Table(at);
    }
    match find_key(&tables[at].1, key) {
        Some(key) => Resolved::Key(at, key),
        None => Resolved::Unknown(ExplainUnknown::Key {
            table: tables[at].0.clone(),
            key: key.to_string(),
            admitted: written_keys(&tables[at].1),
        }),
    }
}

/// §FS-config-explain.5: `asked` read as a dotted TOML key, the bracketed
/// spelling that answers it: the longest leading run of its segments that
/// spells an admitted table is the table, and the rest is a key of it.
fn dotted(tables: &[(String, Vec<String>)], asked: &str) -> Option<ExplainUnknown> {
    let segments: Vec<&str> = asked.split('.').collect();
    let (header, at, rest) = (1..=segments.len()).rev().find_map(|run| {
        let path = segments[..run].join(".");
        [format!("[{path}]"), format!("[[{path}]]")]
            .into_iter()
            .find_map(|header| find_table(tables, &header).map(|at| (header, at)))
            .map(|(header, at)| (header, at, segments[run..].join(".")))
    })?;
    let spelling = if rest.is_empty() {
        header
    } else {
        find_key(&tables[at].1, &rest)?;
        format!("{header} {rest}")
    };
    Some(ExplainUnknown::Bracketed {
        target: asked.to_string(),
        spelling,
        key: !rest.is_empty(),
    })
}

/// §FS-config-explain.5: the keys a table's reader admits under its header, as
/// its refusal names them: an inline member is asked about, never written there.
fn written_keys(keys: &[String]) -> Vec<String> {
    let written = |key: &&String| !keys.iter().any(|outer| is_member_of(key, outer));
    keys.iter().filter(written).cloned().collect()
}

/// A bracketed target's header and its key, `""` for the table itself: the key
/// follows the header after whitespace or a `.` (`'[schema.leads.words].warn'`).
fn split(asked: &str) -> Option<(&str, &str)> {
    let close = if asked.starts_with("[[") { "]]" } else { "]" };
    if !asked.starts_with('[') {
        return None;
    }
    let end = asked.find(close).map_or(asked.len(), |at| at + close.len());
    let (header, rest) = asked.split_at(end);
    let rest = rest.trim_start();
    Some((header, rest.strip_prefix('.').unwrap_or(rest).trim()))
}

/// The first admitted table `header` spells, in the order the reader tries
/// them: a name stands for any placeholder, and a placeholder only for one.
fn find_table(tables: &[(String, Vec<String>)], header: &str) -> Option<usize> {
    tables
        .iter()
        .position(|(admitted, _)| !admitted.is_empty() && spells(header, admitted))
}

/// Whether `spelled` names a table of `tables`.
pub(super) fn admits(tables: &[(String, Vec<String>)], spelled: &str) -> bool {
    find_table(tables, spelled).is_some()
}

/// Whether `spelled` names the table `admitted`, both written with brackets.
fn spells(spelled: &str, admitted: &str) -> bool {
    let array = |table: &str| table.starts_with("[[");
    let segments = |table: &str| -> Vec<String> {
        let path = table.trim_matches(['[', ']']);
        path.split('.').map(str::to_string).collect()
    };
    let (spelled_segments, admitted_segments) = (segments(spelled), segments(admitted));
    array(spelled) == array(admitted)
        && spelled.ends_with(if array(spelled) { "]]" } else { "]" })
        && spelled_segments.len() == admitted_segments.len()
        && spelled_segments
            .iter()
            .zip(&admitted_segments)
            .all(|(got, want)| got == want || want.starts_with('<') && !got.is_empty())
}

/// The key of a table's gate that `asked` names: itself, or for a table of
/// names its placeholder, which a single name stands for.
fn find_key<'a>(keys: &'a [String], asked: &str) -> Option<&'a str> {
    let named = keys.iter().find(|key| *key == asked);
    let any = || {
        let single = !asked.is_empty() && !asked.contains('.');
        keys.iter().find(|key| single && key.starts_with('<'))
    };
    named.or_else(any).map(String::as_str)
}

/// §FS-config-explain.5: why `version` has no table `header`.
fn unknown_table(version: u32, header: &str, asked: &str) -> ExplainUnknown {
    if version >= 2 {
        if let Some(homes) = v1_homes(header) {
            return ExplainUnknown::VersionOneTable {
                table: header.to_string(),
                homes,
            };
        }
    } else if find_table(&admitted(2), header).is_some() {
        return ExplainUnknown::VersionTwoTable {
            table: header.to_string(),
        };
    }
    ExplainUnknown::Other {
        target: asked.to_string(),
    }
}

/// Every table a catalog clause spells under a setting's root, such as
/// `[schema.leads.words]` or `[rules.citations.<kind>.grounding]`.
pub(super) fn spelled_tables(text: &str) -> Vec<String> {
    let mut tables = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find('[') {
        let candidate = &rest[start..];
        let open = if candidate.starts_with("[[") { 2 } else { 1 };
        let body = &candidate[open..];
        let len = body
            .find(|c: char| !(c.is_ascii_alphanumeric() || "_.<>-".contains(c)))
            .unwrap_or(body.len());
        if len > 0 && body[len..].starts_with(&"]".repeat(open)) {
            tables.push(format!(
                "{}{}{}",
                "[".repeat(open),
                &body[..len],
                "]".repeat(open)
            ));
        }
        rest = &candidate[open..];
    }
    tables
}
