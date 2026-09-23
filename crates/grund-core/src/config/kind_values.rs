//! What the two value-related `[[kinds]]` keys require of the row that sets
//! them (§FS-config.3.4.9, §FS-config.3.4.13), beside `kind_table.rs`'s
//! whole-list rules (§AR-core-module-layout.1). Both are structural: neither
//! reads a declaration's content, and each refusal names a state in which no
//! value coordinate could ever exist.

use anyhow::{Result, anyhow};
use std::fs;
use std::path::Path;

use super::kind_table::ParsedKind;
use super::record::Config;
use crate::model::{format_path, normalize_path_lexically};

/// Every relationship `values` and `value_chapter` need of their own row
/// (§FS-config.3.4.9, §FS-config.3.4.13). Located at the key's own line, so the
/// message opens the line that has to change.
pub(super) fn validate_kind_value_keys(
    path: &Path,
    entry: &ParsedKind,
    config: &Config,
) -> Result<()> {
    let k = &entry.config;
    if k.values {
        let line = entry.values_line.unwrap_or(entry.header_line);
        if !k.citable {
            return Err(anyhow!(
                "{}:{line}: kind `{}` sets `values = true` with `citable = false`",
                format_path(path),
                k.kind
            ));
        }
        let (home, expects_file) = match (&k.file, &k.folder) {
            (Some(home), None) => (home, true),
            (None, Some(home)) => (home, false),
            _ => {
                return Err(anyhow!(
                    "{}:{line}: kind `{}` sets `values = true` without exactly one `file` or `folder` home",
                    format_path(path),
                    k.kind
                ));
            }
        };
        validate_value_home(path, line, &config.root, &k.kind, home, expects_file)?;
    }
    // §FS-config.3.4.13: the chapter key's own relationships. Each is a
    // state in which no coordinate could ever exist, so the config is
    // refused rather than left to produce a chapter nothing reads.
    if k.value_chapter.is_some() {
        let line = entry.value_chapter_line.unwrap_or(entry.header_line);
        // §FS-config.3.2.7 is a prerequisite, not a consequence: without
        // named sections the handle this key names is not even a section.
        if !config.named_sections {
            return Err(anyhow!(
                "{}:{line}: [[kinds]] sets `value_chapter` but [id] named_sections is not true",
                format_path(path)
            ));
        }
        if !k.citable {
            return Err(anyhow!(
                "{}:{line}: kind `{}` sets `value_chapter` with `citable = false`",
                format_path(path),
                k.kind
            ));
        }
        if !k.scan {
            return Err(anyhow!(
                "{}:{line}: kind `{}` sets `value_chapter` with `scan = false` (an unwalked place is never read, so its chapter could never be seen)",
                format_path(path),
                k.kind
            ));
        }
        // §FS-values.2.1: a whole-declaration value's immediate children are
        // a contiguous numeric run, so a named chapter cannot legally exist
        // inside one — the pair is refused rather than merged.
        if k.values {
            return Err(anyhow!(
                "{}:{line}: kind `{}` sets `value_chapter` and `values = true` (a whole-declaration value admits no named chapter)",
                format_path(path),
                k.kind
            ));
        }
        let (home, expects_file) = match (&k.file, &k.folder) {
            (Some(home), None) => (home, true),
            (None, Some(home)) => (home, false),
            _ => {
                return Err(anyhow!(
                    "{}:{line}: kind `{}` sets `value_chapter` without exactly one `file` or `folder` home",
                    format_path(path),
                    k.kind
                ));
            }
        };
        validate_value_home(path, line, &config.root, &k.kind, home, expects_file)?;
    }
    Ok(())
}

/// A value home is mandatory, existing, and physically inside the project root
/// (§FS-config.3.4.9, §FS-values.1). Resolving both paths closes `..`, absolute,
/// and symlink escapes through the same located config error.
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
    let resolved = fs::canonicalize(&candidate).map_err(|_| {
        anyhow!(
            "{}:{line}: value home for kind `{kind}` does not exist: {home}",
            format_path(config_path)
        )
    })?;
    if !resolved.starts_with(&root) {
        return Err(anyhow!(
            "{}:{line}: value home for kind `{kind}` must normalize inside the project root: {home}",
            format_path(config_path)
        ));
    }
    if (expects_file && !resolved.is_file()) || (!expects_file && !resolved.is_dir()) {
        let expected = if expects_file { "file" } else { "folder" };
        return Err(anyhow!(
            "{}:{line}: value home for kind `{kind}` must be an existing {expected}: {home}",
            format_path(config_path)
        ));
    }
    Ok(())
}
