//! Where the configured scope starts (§AR-system.2.3): the roots a walk of the
//! whole config root begins at, the `[[kinds]]` homes the config lists without
//! walking, and where the config root physically is (§FS-config.3.5,
//! §FS-config.3.4.7, §FS-config.1).
//!
//! Every one of them reads `[scan] include`, the `[[kinds]]` table and
//! `config.root` and nothing else — no walk state, no entry, no line — so they
//! are the config's answer to "what does this configuration name", which the
//! walk then traverses. They sat in `scanner/walk.rs` while the scanner was the
//! only component that asked, which had workspace reading three of them upward
//! for a question it settles before any scan (§AR-system.4).

use std::fs;
use std::path::PathBuf;

use super::record::Config;

/// Where the config root physically is, for the comparisons that have to be made
/// against a resolved path. Equal to `config.root` for every root `grund`
/// discovers, which is canonical already (§FS-config.1).
pub(crate) fn canonical_config_root(config: &Config) -> PathBuf {
    fs::canonicalize(&config.root).unwrap_or_else(|_| config.root.clone())
}

/// The roots a walk of the whole config root starts from (§FS-config.3.5).
///
/// Without `--full` that is `[scan] include`, or the root itself when the key is
/// unset. With `--full` it is the root **and** every `include` root, not the root
/// alone: a walk root is never pruned by `.gitignore`, `[scan] exclude`, or the
/// hidden-directory rule, while the same directory reached as a *descendant* of
/// the config root is. Starting only at the root would therefore read *fewer*
/// files than the plain walk whenever an `include` entry is gitignored, excluded,
/// or hidden — and `--full` would turn a red run green, which is exactly what
/// §FS-check.1.3.4 promises it can never do. `walk_scannable_files` deduplicates
/// the file list, so an `include` root the root walk already covers is read once.
/// Collecting each root through `components()` folds away the `./` and trailing
/// separator an entry may be written with, so two roots naming one directory
/// yield byte-identical descendant paths and the dedup can recognize the reread.
///
/// The `include` roots come **first** under `--full`, ahead of the config root.
/// An `include` root that is a symlink to a directory inside the root, or a case
/// alias of one, reaches its files under a spelling the root walk does not
/// reproduce, so the dedup has to choose between two names for one file; walking
/// `include` first makes the first-seen winner the spelling `grund check` prints
/// without the flag, which is what keeps `--full` purely additive.
///
/// Why `include` beats `scan = false` here: a root is never filtered — that is what
/// makes it a root — so a config that says both "listed, not walked" and "walk this"
/// has to be settled before the walk starts rather than by the walk's own prune.
///
/// Why every kind home is a root: a home is the repository saying "declarations and
/// citations live here", and leaving it out of the scan made its citations *invisible*
/// rather than dangling — the trap a non-citable kind, whose whole content is "this
/// directory matters", would otherwise fall into on its first line of config. The
/// homes are ordered after `include` so the first-seen spelling of a file reached two
/// ways is still `include`'s, which keeps the dedup and `--full`'s additivity
/// unchanged.
pub(crate) fn root_scope_roots(config: &Config, full: bool) -> Vec<PathBuf> {
    // §FS-config.3.4.7.2: an `include` entry at or inside an unwalked home is the one
    // way such a home is still a *root*, where the walk's own prune cannot reach it.
    // The narrower key, the one written on the kind itself, wins.
    let unwalked = if full {
        Vec::new()
    } else {
        unwalked_home_roots(config)
    };
    let include = config
        .include
        .iter()
        .flatten()
        .map(|entry| config.root.join(entry).components().collect::<PathBuf>())
        .filter(move |root| !unwalked.iter().any(|home| root.starts_with(home)))
        .collect::<Vec<_>>()
        .into_iter();
    // §FS-config.3.5.8: every configured kind home is walked whether or not `include`
    // names it; `include` keeps its job, the extra roots. Ordered after `include` so
    // the first-seen spelling of a file reached two ways is still the one it gives.
    let homes = kind_home_roots(config);
    match (full, config.include.is_some()) {
        (true, _) => include
            .chain(homes)
            .chain(std::iter::once(config.root.clone()))
            .collect(),
        (false, true) => include.chain(homes).collect(),
        // No `include` key: the whole root is walked, and every home is under
        // it, so there is nothing left for the homes to add.
        (false, false) => vec![config.root.clone()],
    }
}

/// Every home the config lists without walking (`scan = false`,
/// §FS-config.3.4.7), as an absolute path under the config root — the
/// complement of `kind_home_roots` over the kinds that have a home.
pub(crate) fn unwalked_home_roots(config: &Config) -> Vec<PathBuf> {
    unwalked_homes(config)
        .map(|home| config.root.join(home).components().collect::<PathBuf>())
        .collect()
}

/// Every configured `[[kinds]]` home as a walk root (§FS-config.3.5.8) — `file`
/// homes included, since a single-file kind's document is as much a home as a
/// folder is. A home that does not exist walks as nothing, so a fresh repository
/// whose default homes are not scaffolded yet stays silent. An unwalked home
/// (`scan = false`, §FS-config.3.4.7) is not a root: it is a place the Project
/// map names, and nothing in it is read short of `--full`'s root walk.
fn kind_home_roots(config: &Config) -> impl Iterator<Item = PathBuf> + '_ {
    config.kinds.iter().filter_map(|kind| {
        if !kind.scan {
            return None;
        }
        let home = kind.file.as_deref().or(kind.folder.as_deref())?;
        Some(config.root.join(home).components().collect::<PathBuf>())
    })
}

/// The same homes as the config's own `[[kinds]]` spellings, which is the form
/// the walk's per-entry prune compares (§AR-scanner.2.4, §FS-config.3.4.7).
pub(crate) fn unwalked_homes(config: &Config) -> impl Iterator<Item = &str> + '_ {
    config
        .kinds
        .iter()
        .filter(|kind| !kind.scan)
        .filter_map(|kind| kind.file.as_deref().or(kind.folder.as_deref()))
}
