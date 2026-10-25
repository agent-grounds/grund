//! §FS-config-explain.5: the readings of a path against one version's own
//! tables, and the tables a bare name names. One derivation serves the answer
//! to a path with one reading, the failure for a path with more, and the list
//! for a bare name (§FS-config-explain.1.1).

use super::target::{find_key, find_table, leading_tables};
use super::{ExplainName, ExplainReading, ExplainUnknown};

/// §FS-config-explain.5: every reading of `segments`, the readings with a name
/// left out first and then the longest-run reading, each named once.
pub(super) fn readings(tables: &[(String, Vec<String>)], segments: &[&str]) -> Vec<ExplainReading> {
    let Some((header, at, rest)) = longest(tables, segments) else {
        return Vec::new();
    };
    let mut found: Vec<ExplainReading> = Vec::new();
    let mut named = None;
    for (slot, placeholder) in filled(&header, &tables[at].0) {
        let mut left_out = segments.to_vec();
        left_out.insert(slot, placeholder);
        let Some(reading) = longest(tables, &left_out)
            .and_then(|(header, at, rest)| reading(tables, header, at, &rest))
        else {
            continue;
        };
        named.get_or_insert_with(|| ExplainName {
            what: what(placeholder),
            name: segments[slot].to_string(),
        });
        if !found.contains(&reading) {
            found.push(reading);
        }
    }
    if let Some(mut taken) = reading(tables, header, at, &rest) {
        taken.named = named;
        if !found
            .iter()
            .any(|reading| reading.spelling == taken.spelling)
        {
            found.push(taken);
        }
    }
    found
}

/// The longest leading run of `segments` that spells an admitted table, the
/// table it spells and the rest of the path, joined by `.`.
fn longest(tables: &[(String, Vec<String>)], segments: &[&str]) -> Option<(String, usize, String)> {
    leading_tables(segments)
        .find_map(|(header, rest)| find_table(tables, &header).map(|at| (header, at, rest)))
}

/// The table `header` spells, or the key `rest` of it, as its bracketed
/// spelling keeps the names as written; `None` where `rest` is no key of it.
fn reading(
    tables: &[(String, Vec<String>)],
    header: String,
    at: usize,
    rest: &str,
) -> Option<ExplainReading> {
    let key = !rest.is_empty();
    let spelling = if key {
        find_key(&tables[at].1, rest)?;
        format!("{header} {rest}")
    } else {
        header
    };
    Some(ExplainReading {
        spelling,
        key,
        named: None,
    })
}

/// Each placeholder of `admitted` that `spelled` fills with a name, by its
/// segment's index in the path.
fn filled<'a>(spelled: &str, admitted: &'a str) -> Vec<(usize, &'a str)> {
    let segments = |table: &'a str| {
        table
            .trim_matches(['[', ']'])
            .split('.')
            .collect::<Vec<_>>()
    };
    let spelled: Vec<&str> = spelled.trim_matches(['[', ']']).split('.').collect();
    segments(admitted)
        .into_iter()
        .zip(spelled)
        .enumerate()
        .filter(|(_, (want, got))| want.starts_with('<') && want != got)
        .map(|(slot, (want, _))| (slot, want))
        .collect()
}

/// §FS-config-explain.5: what a name in a placeholder's place names.
fn what(placeholder: &str) -> &'static str {
    match placeholder {
        "<name>" => "language",
        "<field>" => "field",
        _ => "kind",
    }
}

/// §FS-config-explain.5: a bare name, one segment no own table reads, names
/// each table holding a key of exactly that name and each table whose header
/// ends in it, in index order; a placeholder is no name, so it matches none.
pub(super) fn bare_name(
    tables: &[(String, Vec<String>)],
    segments: &[&str],
) -> Option<ExplainUnknown> {
    let [name] = segments else {
        return None;
    };
    if name.starts_with('<') {
        return None;
    }
    let mut spellings = Vec::new();
    for (header, keys) in tables.iter().filter(|(header, _)| !header.is_empty()) {
        let last = header.trim_matches(['[', ']']).rsplit('.').next();
        if last == Some(*name) {
            spellings.push(ExplainReading {
                spelling: header.clone(),
                key: false,
                named: None,
            });
        }
        if keys.iter().any(|key| key == name) {
            spellings.push(ExplainReading {
                spelling: format!("{header} {name}"),
                key: true,
                named: None,
            });
        }
    }
    (!spellings.is_empty()).then(|| ExplainUnknown::BareName {
        target: name.to_string(),
        spellings,
    })
}
