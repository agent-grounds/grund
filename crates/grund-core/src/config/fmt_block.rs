//! The `[fmt]` section's one grammar (§FS-config.3.10): the `exclude` globs,
//! compiled into a matcher over config-root-relative paths and validated at the
//! line that wrote them.
//!
//! One file per section of `grund.toml` that carries a grammar of its own, the
//! way `workspace_block.rs` holds the `[workspace]` entry rules
//! (§AR-core-module-layout.1). The validator was the formatter's while
//! §AR-system.2.8 was a file-name category, and `parse.rs` read it upward to
//! refuse a malformed glob as it read the key; it came down here when the
//! writers became a module, and the formatter's own matcher
//! (`writers/fmt_suppress.rs`) reads the compiler downward instead
//! (§AR-system.4). Nothing here knows the walk it will be asked about: a
//! pattern list in, a matcher or a message out.

use ignore::gitignore::{Gitignore, GitignoreBuilder};

/// Compile `[fmt] exclude` into a matcher over config-root-relative paths
/// (§FS-config.3.10). The root is left empty on purpose: every caller rebases
/// the path itself, so the matcher never has to guess how much of an absolute
/// path is the project.
pub(crate) fn build_fmt_exclude_matcher(
    patterns: &[String],
) -> std::result::Result<Gitignore, String> {
    let mut builder = GitignoreBuilder::new("");
    for pattern in patterns {
        builder
            .add_line(None, pattern)
            .map_err(|err| err.to_string())?;
    }
    builder.build().map_err(|err| err.to_string())
}

/// §FS-config.3.10: reject a malformed glob at the line that wrote it, rather
/// than at the first `grund fmt` in a repository that has forgotten about it
/// (§FS-config.4.3).
pub(crate) fn validate_fmt_exclude(patterns: &[String]) -> std::result::Result<(), String> {
    build_fmt_exclude_matcher(patterns)
        .map(|_| ())
        .map_err(|message| format!("[fmt] exclude: {message}"))
}
