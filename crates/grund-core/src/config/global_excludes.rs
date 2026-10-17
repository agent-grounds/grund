//! The global excludes pathname shared by scanning and watch-input discovery
//! (§FS-scan.scope.ignore-files.pathname).

use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};

use crate::sources::{observe_input, read_bytes};

/// Preserve `ignore`'s discovery order: HOME's `.gitconfig`, XDG's `git/config`,
/// then XDG's optional `git/ignore`. Parse Git's grammar and interpolate only a
/// leading home abbreviation, once, rather than `ignore`'s replacement of every
/// tilde (§FS-scan.scope.ignore-files.pathname). No subprocess or canonicalization
/// participates, and a resolution failure never falls back to another file.
pub(crate) fn global_excludes_path() -> Result<Option<PathBuf>> {
    // HOME is Git's home override on Windows too; std supplies the platform
    // fallback when it is absent, as it does for the previous loader.
    #[allow(deprecated)]
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(std::env::home_dir);
    let xdg = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| home.as_ref().map(|home| home.join(".config")));
    let candidates = [
        home.as_ref().map(|home| home.join(".gitconfig")),
        xdg.as_ref().map(|xdg| xdg.join("git/config")),
    ];
    for file in candidates.into_iter().flatten() {
        if !observe_input(&file, false) {
            return Err(anyhow!("watch ignore coverage failed"));
        }
        let bytes = match read_bytes(&file) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error).with_context(|| format!("read {}", file.display())),
        };
        let config = gix_config::File::from_bytes_no_includes(
            &bytes,
            gix_config::file::Metadata::from(gix_config::Source::User),
            Default::default(),
        )
        .map_err(|error| anyhow!("parse {}: {error}", file.display()))?;
        if let Some(path) = config.path("core.excludesFile") {
            // An explicit empty value disables the global file; it must not
            // select the optional default instead.
            if path.is_empty() {
                return Ok(Some(PathBuf::new()));
            }
            return path
                .interpolate(gix_config::path::interpolate::Context {
                    home_dir: home.as_deref(),
                    ..Default::default()
                })
                .map(Some)
                .map_err(|error| {
                    anyhow!("resolve core.excludesFile in {}: {error}", file.display())
                });
        }
    }
    Ok(xdg.map(|xdg| xdg.join("git/ignore")))
}
