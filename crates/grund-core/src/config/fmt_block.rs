//! The `[fmt]` section's one grammar (§FS-config.3.10): the `exclude` globs,
//! compiled into a matcher over config-root-relative paths and validated at the
//! line that wrote them.
//!
//! One file per section of `grund.toml` that carries a grammar of its own, the
//! way `workspace_block.rs` holds the `[workspace]` entry rules
//! (§AR-core-module-layout.1). The validator was the formatter's while
//! §AR-system.2.8 was a file-name category, and `parse.rs` read it upward to
//! refuse a malformed glob as it read the key; it came down here when the
//! writers became a module. Nothing here knows the walk it will be asked about:
//! a pattern list in, and a matcher, a message, or the exclusion scope of
//! §FS-fmt.2.5.1 out.
//!
//! Which is why building that scope is here rather than in the record that
//! carries it. `grammar/fmt_suppress.rs` recognizes what is out of reach and
//! reads no `Config` (§AR-system.2.1), so the compiled matcher and the root a
//! path is rebased against are what config hands it — paired once here rather
//! than at each of the two callers, where they could come to disagree.

use anyhow::{Result, anyhow};
use ignore::gitignore::{Gitignore, GitignoreBuilder};

use super::record::Config;
use crate::grammar::FmtExcluded;

/// Compile `[fmt] exclude` into a matcher over config-root-relative paths
/// (§FS-config.3.10.1). The root is left empty on purpose: every caller rebases
/// the path itself, so the matcher never has to guess how much of an absolute
/// path is the project.
fn build_fmt_exclude_matcher(patterns: &[String]) -> std::result::Result<Gitignore, String> {
    let mut builder = GitignoreBuilder::new("");
    for pattern in patterns {
        builder
            .add_line(None, pattern)
            .map_err(|err| err.to_string())?;
    }
    builder.build().map_err(|err| err.to_string())
}

/// §FS-config.3.10.1: reject a malformed glob at the line that wrote it, rather
/// than at the first `grund fmt` in a repository that has forgotten about it
/// (§FS-config.4.3).
pub(crate) fn validate_fmt_exclude(patterns: &[String]) -> std::result::Result<(), String> {
    build_fmt_exclude_matcher(patterns)
        .map(|_| ())
        .map_err(|message| format!("[fmt] exclude: {message}"))
}

/// One project's §FS-fmt.2.5.1 exclusion scope: the patterns compiled, in the
/// record the formatter and the editor both ask (§FS-fmt.2.5, §FS-lsp.1.4.3).
///
/// Patterns were already validated at load (§FS-config.3.10.1), so a failure here
/// is a grund bug rather than a user error — it is still reported rather than
/// swallowed, because the alternative is a `--write` that silently rewrites a
/// protected file.
pub(crate) fn fmt_excluded(config: &Config) -> Result<FmtExcluded> {
    let matcher = if config.fmt_exclude.is_empty() {
        None
    } else {
        Some(
            build_fmt_exclude_matcher(&config.fmt_exclude)
                .map_err(|message| anyhow!("[fmt] exclude: {message}"))?,
        )
    };
    Ok(FmtExcluded::new(&config.root, matcher))
}
