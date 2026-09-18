//! How a report spells a path, which is a question about one `grund.toml` key
//! (§AR-system.2.3): `[output] relative_paths` picks the base every reported
//! path is rendered against (§FS-config.3.6).
//!
//! It sat in the deprecated path's `output` category while workspace, the
//! scanner, the checker, the queries, the writers and the api all read it upward
//! (§AR-system.4, §AR-system.2.9). The spelling itself — forward slashes, the
//! sort key, the `..` walk out of a base — is a function of a `Path` alone and
//! went one level further down, into `model/paths.rs`; what is left here is the
//! part that needs a `Config` to answer, so it belongs to the component that
//! owns the key.

use std::path::Path;

use super::record::Config;
use crate::model::{format_path, relative_from_base};

/// Render a path the way reports show it: relative to the repo root by default,
/// or relative to the CLI base directory when `[output] relative_paths = false`
/// (§FS-config.3.6 — an in-root target outside that base uses bounded `..`).
pub(crate) fn display_path(config: &Config, path: &Path) -> String {
    let base = if config.relative_paths {
        &config.root
    } else {
        &config.cli_base
    };
    let relative = path
        .strip_prefix(base)
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| {
            if !config.relative_paths && path.starts_with(&config.root) {
                relative_from_base(base, path)
            } else {
                path.to_path_buf()
            }
        });
    format_path(&relative)
}
