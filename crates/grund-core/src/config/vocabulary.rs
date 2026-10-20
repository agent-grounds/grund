//! §FS-config-explain.3: the tables and keys each version's reader admits, as
//! `grund config explain` enumerates them: the v2 gates through `v2::admitted`,
//! and v1's lowering table, which the v1 parser admits its keys from
//! (§AR-config.3.3 item 5).

use super::point_sizes::LEAD_SIZE_MEMBERS;
use super::v1::mapping::KEYS;

/// Each table the reader of `version` admits, `[header]` or `""` for the top
/// level, with its keys; an inline table's members follow it as `key.member`.
pub(crate) fn admitted(version: u32) -> Vec<(String, Vec<String>)> {
    if version == 2 {
        return super::v2::admitted();
    }
    let mut tables: Vec<(String, Vec<String>)> = Vec::new();
    for (name, _) in KEYS {
        let (header, key) = match name.split_once("] ") {
            Some((header, key)) => (format!("{header}]"), key),
            None => (String::new(), *name),
        };
        let at = match tables.iter().position(|(seen, _)| *seen == header) {
            Some(at) => at,
            None => {
                tables.push((header, Vec::new()));
                tables.len() - 1
            }
        };
        tables[at].1.push(key.to_string());
        if *name == "[reference] lead_size_warning" {
            let members = LEAD_SIZE_MEMBERS
                .iter()
                .map(|member| format!("{key}.{member}"));
            tables[at].1.extend(members);
        }
    }
    tables
}
