//! The global excludes pathname shared by scanning and watch-input discovery
//! (§FS-scan.scope.ignore-files.pathname).

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};

use crate::sources::{observe_input, read_bytes};

/// Preserve `ignore`'s discovery order: HOME's `.gitconfig`, XDG's `git/config`,
/// then XDG's optional `git/ignore`. Parse Git's grammar and interpolate only a
/// leading home abbreviation, once, rather than `ignore`'s replacement of every
/// tilde (§FS-scan.scope.ignore-files.pathname). No subprocess or canonicalization
/// participates. Optional config read/parse failures permit further discovery;
/// failure to resolve a selected pathname does not select a different file.
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
            // Config discovery was optional before the pathname correction,
            // including unreadable files, not just absent ones.
            Err(_) => continue,
        };
        let Ok(config) = gix_config::File::from_bytes_no_includes(
            &bytes,
            gix_config::file::Metadata::from(gix_config::Source::User),
            Default::default(),
        ) else {
            // An unrelated malformed config must not turn optional ignore
            // discovery into a fatal acceptance policy.
            continue;
        };
        // Preserve the previous first-value selection and empty-value fallback,
        // even though Git itself gives the last setting precedence.
        if let Some(value) = config
            .strings("core.excludesFile")
            .into_iter()
            .flatten()
            .find(|value| !value.is_empty())
        {
            return interpolate_path(gix_config::Path::from(value), home.as_deref())
                .map(Some)
                .with_context(|| format!("resolve core.excludesFile in {}", file.display()));
        }
    }
    Ok(xdg.map(|xdg| xdg.join("git/ignore")))
}

/// Interpolate the home abbreviation alone, then concatenate its suffix as Git
/// does (§FS-scan.scope.ignore-files.pathname). PathBuf::push would replace the
/// home for a suffix beginning with another slash, in both ~/ and ~user/ forms.
fn interpolate_path(mut path: gix_config::Path, home: Option<&Path>) -> Result<PathBuf> {
    let context = gix_config::path::interpolate::Context {
        home_dir: home,
        ..Default::default()
    };
    if path.starts_with(b"~")
        && let Some(slash) = path.iter().position(|&byte| byte == b'/')
    {
        let suffix = gix_config::Path {
            value: path.value.split_off(slash).into(),
            is_optional: false,
        };
        let mut expanded = path
            .interpolate(context)
            .map_err(|error| anyhow!("{error}"))?
            .into_os_string();
        // The suffix starts with '/', so converting it cannot expand a tilde.
        expanded.push(
            suffix
                .interpolate(Default::default())
                .map_err(|error| anyhow!("{error}"))?,
        );
        return Ok(PathBuf::from(expanded));
    }
    path.interpolate(context)
        .map_err(|error| anyhow!("{error}"))
}
