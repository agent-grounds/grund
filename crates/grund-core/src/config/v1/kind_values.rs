//! What the two value-related `[[kinds]]` keys require of the row that sets
//! them (§FS-config.3.4.9, §FS-config.3.4.13), beside `kind_table.rs`'s
//! whole-list rules (§AR-core-module-layout.1). Both are structural: neither
//! reads a declaration's content, and each refusal names a state in which no
//! value coordinate could ever exist. The v1 reader asks them per entry, in
//! the same pass as the row refusals they are interleaved with, because the
//! home they check is read off the filesystem under the config root
//! (§AR-config.4). The v2 reader asks the same ones of `form` and a values
//! field, so one spelling has one set of prerequisites (§FS-fields.forms).

use anyhow::Result;
use std::path::Path;

use super::values::bail_config;
use crate::config::home_requirement::{Ask, Home, Verdict, home_requirement, rule_kind_message};
use crate::config::kind::KindConfig;
use crate::config::project::Sources;
use crate::model::Code;

/// The lines a row's value refusals are located at: the key that wrote each,
/// or the row's header (§FS-config.4.3).
pub(in crate::config) struct ValueLines {
    pub(in crate::config) header: usize,
    pub(in crate::config) values: Option<usize>,
    pub(in crate::config) value_chapter: Option<usize>,
}

/// Every relationship `values` and `value_chapter` need of their own row
/// (§FS-config.3.4.9, §FS-config.3.4.13, §FS-fields.forms). Located at the
/// key's own line, so the message opens the line that has to change and the
/// editor marks that same line (§FS-lsp.2.2.4.1). `homes` are every home of the
/// row, each judged (§AR-config.1.6), and what a later absent one earns is
/// returned as a warning at its own line (§FS-config-v2.checks.config-value-home-absent).
/// `sources` is what the project reads, which an absent home's hint asks (§FS-fetch.4.1).
pub(in crate::config) fn validate_kind_value_keys(
    path: &Path,
    k: &KindConfig,
    lines: &ValueLines,
    homes: &[Home<'_>],
    root: &Path,
    named_sections: bool,
    sources: &Sources,
) -> Result<Vec<(usize, Code, String)>> {
    let mut warnings = Vec::new();
    if k.values {
        let line = lines.values.unwrap_or(lines.header);
        if !k.citable {
            return bail_config(
                path,
                line,
                format!(
                    "kind `{}` sets `values = true` with `citable = false`",
                    k.kind
                ),
            );
        }
        if k.file.is_some() == k.folder.is_some() {
            return bail_config(
                path,
                line,
                format!(
                    "kind `{}` sets `values = true` without exactly one `file` or `folder` home",
                    k.kind
                ),
            );
        }
        // §AR-config.1.6: every home, not only the first.
        match judge_homes(Ask::Value, k, homes, root, sources) {
            Ok(found) => warnings.extend(found),
            Err(message) => return bail_config(path, line, message),
        }
    }
    // §FS-config.3.4.13: the chapter key's own relationships. Each is a
    // state in which no coordinate could ever exist, so the config is
    // refused rather than left to produce a chapter nothing reads.
    if k.value_chapter.is_some() {
        let line = lines.value_chapter.unwrap_or(lines.header);
        // §FS-config.3.2.7 is a prerequisite, not a consequence: without
        // named sections the handle this key names is not even a section.
        if !named_sections {
            return bail_config(
                path,
                line,
                "[[kinds]] sets `value_chapter` but [id] named_sections is not true".to_string(),
            );
        }
        if !k.citable {
            return bail_config(
                path,
                line,
                format!(
                    "kind `{}` sets `value_chapter` with `citable = false`",
                    k.kind
                ),
            );
        }
        if !k.scan {
            return bail_config(
                path,
                line,
                format!(
                    "kind `{}` sets `value_chapter` with `scan = false` (an unwalked place is never read, so its chapter could never be seen)",
                    k.kind
                ),
            );
        }
        // §FS-values.2.1: a whole-declaration value's immediate children are
        // a contiguous numeric run, so a named chapter cannot legally exist
        // inside one — the pair is refused rather than merged.
        if k.values {
            return bail_config(
                path,
                line,
                format!(
                    "kind `{}` sets `value_chapter` and `values = true` (a whole-declaration value admits no named chapter)",
                    k.kind
                ),
            );
        }
        if k.file.is_some() == k.folder.is_some() {
            return bail_config(
                path,
                line,
                format!(
                    "kind `{}` sets `value_chapter` without exactly one `file` or `folder` home",
                    k.kind
                ),
            );
        }
        match judge_homes(Ask::Value, k, homes, root, sources) {
            Ok(found) => warnings.extend(found),
            Err(message) => return bail_config(path, line, message),
        }
    }
    Ok(warnings)
}

/// §FS-config.3.4.12, §FS-fields.forms: why a rule kind's row cannot hold
/// rules, if it cannot — the refusal v1 writes unlocated and v2 at `form`. What
/// each home is read as is `home_requirement`'s, asked of every home
/// (§FS-config-v2.schema.sources.rules).
pub(in crate::config) fn rule_kind_refusal(k: &KindConfig) -> Option<String> {
    let refused = k.rules
        && (!k.citable
            || !k.scan
            || (k.folder.is_none() && k.file.is_none())
            || k.values
            || k.fetch.is_some());
    refused.then(|| rule_kind_message(&k.kind))
}

/// §FS-config.3.4.9, §FS-config.3.4.12: every home of a row judged by `home_requirement`, the
/// refusal of the first that refuses, at `line` for a value home and with the form's refusal for
/// a rule home, or the warnings the rest earn at their own lines.
pub(in crate::config) fn judge_homes(
    ask: Ask,
    k: &KindConfig,
    homes: &[Home<'_>],
    root: &Path,
    sources: &Sources,
) -> std::result::Result<Vec<(usize, Code, String)>, String> {
    let mut warnings = Vec::new();
    for home in homes {
        match home_requirement(ask, k, *home, root, sources) {
            Verdict::Holds => {}
            Verdict::RefusesRow => return Err(rule_kind_message(&k.kind)),
            Verdict::Refused(message) => return Err(message),
            Verdict::Warns(code, message) => warnings.push((home.line, code, message)),
        }
    }
    Ok(warnings)
}
