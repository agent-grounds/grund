//! The `[scan] exclude` grammar (§FS-config.3.5.16): one directory component
//! name per entry, so an entry containing `/` is refused at the line that wrote
//! it rather than accepted as a path no directory name can ever equal.
//!
//! One file per section of `grund.toml` that carries a grammar of its own, the
//! way `fmt_block.rs` holds the `[fmt] exclude` globs (§AR-core-module-layout.1).
//! Only the refusal is here: what an entry matches is the scanner's, which
//! compares it against each directory's own name and is not changed by it.

use anyhow::Result;
use std::path::Path;

use super::values::{bail_config, parse_string_list};

/// Read `[scan] exclude` at `line`, refusing the first entry in list order that
/// contains `/` with the single located error of §FS-config.4.3, as
/// §FS-config.3.5.16 specifies — unless the pass is `tolerant`, which only
/// migration's is, to diagnose the entry (§FS-config-migrate.3.1.4).
pub(super) fn parse_scan_exclude(
    path: &Path,
    line: usize,
    value: &str,
    tolerant: bool,
) -> Result<Vec<String>> {
    let exclude = parse_string_list(path, line, value)?;
    match scan_exclude_slash_error(&exclude).filter(|_| !tolerant) {
        Some(message) => bail_config(path, line, message),
        None => Ok(exclude),
    }
}

/// The message for the first entry containing `/` (§FS-config.3.5.16): it names
/// the entry and both repairs — removal, which keeps the scan as it was, and the
/// last non-empty component, which excludes every directory of that name at every
/// depth. A file's name would load and exclude nothing, and loading does not look at
/// the tree, so the message states that repair's condition. An entry with no
/// non-empty component, such as `/`, is offered removal alone, and so is one whose
/// last non-empty component is `.` or `..`, a name no directory the walk meets bears.
fn scan_exclude_slash_error(exclude: &[String]) -> Option<String> {
    let entry = exclude.iter().find(|entry| entry.contains('/'))?;
    let removal = format!(
        "[scan] exclude entry `{entry}` contains `/`, not a directory name; \
         remove it to keep the scan unchanged"
    );
    let name = entry
        .rsplit('/')
        .find(|component| !component.is_empty())
        // §FS-config.3.5.16: `.` and `..` name no directory, so they offer no name.
        .filter(|name| !matches!(*name, "." | ".."));
    Some(match name {
        Some(name) => format!(
            "{removal}, or, if `{name}` is a directory, use `{name}` to exclude \
             every directory of that name at any depth"
        ),
        None => removal,
    })
}
