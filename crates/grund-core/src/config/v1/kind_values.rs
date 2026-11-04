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
use std::fs;
use std::path::Path;

use super::values::bail_config;
use crate::config::kind::KindConfig;
use crate::model::normalize_path_lexically;

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
/// editor marks that same line (§FS-lsp.2.2.4.1).
pub(in crate::config) fn validate_kind_value_keys(
    path: &Path,
    k: &KindConfig,
    lines: &ValueLines,
    root: &Path,
    named_sections: bool,
) -> Result<()> {
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
        let (home, expects_file) = match (&k.file, &k.folder) {
            (Some(home), None) => (home, true),
            (None, Some(home)) => (home, false),
            _ => {
                return bail_config(
                    path,
                    line,
                    format!(
                        "kind `{}` sets `values = true` without exactly one `file` or `folder` home",
                        k.kind
                    ),
                );
            }
        };
        validate_value_home(path, line, root, &k.kind, home, expects_file)?;
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
        let (home, expects_file) = match (&k.file, &k.folder) {
            (Some(home), None) => (home, true),
            (None, Some(home)) => (home, false),
            _ => {
                return bail_config(
                    path,
                    line,
                    format!(
                        "kind `{}` sets `value_chapter` without exactly one `file` or `folder` home",
                        k.kind
                    ),
                );
            }
        };
        validate_value_home(path, line, root, &k.kind, home, expects_file)?;
    }
    Ok(())
}

/// §FS-config-v2.schema.sources.rules: version 1's answer to whether a row's
/// `file` home is Markdown, its name, because `.md` is the one suffix it reads so.
pub(in crate::config) fn file_home_is_named_markdown(k: &KindConfig) -> bool {
    k.file
        .as_deref()
        .is_none_or(|file| Path::new(file).extension().and_then(|ext| ext.to_str()) == Some("md"))
}

/// §FS-config.3.4.12, §FS-fields.forms: why a rule kind's row cannot hold
/// rules, if it cannot — the refusal v1 writes unlocated and v2 at `form`.
/// `file_homes_are_markdown` is the reader's answer for the row's `file` homes,
/// which each version gives its own way (§FS-config-v2.schema.sources.rules).
pub(in crate::config) fn rule_kind_refusal(
    k: &KindConfig,
    file_homes_are_markdown: bool,
) -> Option<String> {
    let refused = k.rules
        && (!k.citable
            || !k.scan
            || (k.folder.is_none() && k.file.is_none())
            || !file_homes_are_markdown
            || k.values
            || k.fetch.is_some());
    refused.then(|| {
        format!(
            "kind `{}` sets `rules = true` but rule kinds must be citable, scanned Markdown kinds with a `file` or `folder` home and without `values` or `fetch`",
            k.kind
        )
    })
}

/// A value home is mandatory, existing, and physically inside the project root
/// (§FS-config.3.4.9, §FS-values.1). Resolving both paths closes `..`, absolute,
/// and symlink escapes through the same located config error, with its named
/// line preserved for the editor (§FS-lsp.2.2.4.1).
fn validate_value_home(
    config_path: &Path,
    line: usize,
    root: &Path,
    kind: &str,
    home: &str,
    expects_file: bool,
) -> Result<()> {
    let candidate = normalize_path_lexically(&root.join(home));
    let root = fs::canonicalize(root).unwrap_or_else(|_| normalize_path_lexically(root));
    let resolved = fs::canonicalize(&candidate).or_else(|_| {
        bail_config(
            config_path,
            line,
            format!("value home for kind `{kind}` does not exist: {home}"),
        )
    })?;
    if !resolved.starts_with(&root) {
        return bail_config(
            config_path,
            line,
            format!("value home for kind `{kind}` must normalize inside the project root: {home}"),
        );
    }
    if (expects_file && !resolved.is_file()) || (!expects_file && !resolved.is_dir()) {
        let expected = if expects_file { "file" } else { "folder" };
        return bail_config(
            config_path,
            line,
            format!("value home for kind `{kind}` must be an existing {expected}: {home}"),
        );
    }
    Ok(())
}
