//! The scoped files a run owns on disk (§FS-init.2.3.5.13.1, §FS-init.2.3.5.14.3):
//! the list the root block records, the companions beside a scoped `AGENTS.md`,
//! which root companions a scoped directory mirrors, and which files of an
//! orphan's directory lose their block. Read the same way by `init` before it
//! writes and by `Expected` before `check` compares.

use std::path::{Component, Path, PathBuf};

use crate::checker::section_in_block;
use crate::grammar::{AgentsBlockLookup, find_agents_block};
use crate::scanner::{CANONICAL_AGENT_ENTRYPOINT, companion_agent_entrypoints, is_symlink_to};
use crate::sources::{Sources, link_metadata, observe_input, path_is_entry};

/// The companions a scoped directory can carry; the root-only entrypoints are
/// never mirrored (§FS-init.2.3.5.14.3).
pub(crate) const SCOPED_COMPANIONS: [&str; 2] = ["CLAUDE.md", "GEMINI.md"];

/// What the root entrypoints say about scoped files: whether any carries a
/// managed block, and the paths its `### Scoped guidance` list names.
#[derive(Debug, Default)]
pub(crate) struct ScopedRecord {
    /// Whether a root entrypoint carries a managed block; only then are scoped
    /// entrypoints expected (§FS-check.3.5).
    pub(crate) managed: bool,
    /// The config-root-relative `AGENTS.md` paths the list on disk names.
    pub(crate) listed: Vec<String>,
}

impl ScopedRecord {
    /// Read `AGENTS.md` at `root`, then each companion, and take the list from
    /// the first that carries a block.
    pub(crate) fn read(root: &Path) -> Self {
        let mut candidates = vec![root.join(CANONICAL_AGENT_ENTRYPOINT)];
        candidates.extend(companion_agent_entrypoints(root).unwrap_or_default());
        for path in candidates {
            let Ok(text) = Sources::on_disk().text(&path) else {
                continue;
            };
            if let AgentsBlockLookup::Found(block) = find_agents_block(&text) {
                let block_text = text[block.start..block.end].replace('\r', "");
                return Self {
                    managed: true,
                    listed: listed_scoped_files(&block_text),
                };
            }
        }
        Self::default()
    }

    /// The directories the list on disk names that no destination in `live`
    /// is, in list order: the orphans (§FS-init.2.3.5.13.1).
    pub(crate) fn orphans<'a>(&'a self, live: &[&str]) -> Vec<&'a str> {
        self.listed
            .iter()
            .filter_map(|path| path.strip_suffix("/AGENTS.md"))
            .filter(|dir| !live.contains(dir))
            .collect()
    }
}

/// The paths a root block's `### Scoped guidance` list names: each line
/// `` - `<path>`: … `` of that section whose path is relative, climbs nowhere,
/// and ends in `/AGENTS.md` (§FS-init.2.3.5.13.1).
pub(crate) fn listed_scoped_files(block_text: &str) -> Vec<String> {
    let Some(section) = section_in_block(block_text, "### Scoped guidance") else {
        return Vec::new();
    };
    section
        .lines()
        .filter_map(|line| line.strip_prefix("- `")?.split_once("`:"))
        .map(|(path, _)| path)
        .filter(|path| {
            path.ends_with("/AGENTS.md")
                && Path::new(path)
                    .components()
                    .all(|component| matches!(component, Component::Normal(_)))
        })
        .map(str::to_string)
        .collect()
}

/// Whether the root's `name` is a symlink to the root `AGENTS.md`, which is
/// what a scoped directory mirrors (§FS-init.2.3.5.14.3).
pub(crate) fn mirrored(root: &Path, name: &str) -> bool {
    is_symlink_to(&root.join(name), &root.join(CANONICAL_AGENT_ENTRYPOINT)).unwrap_or(false)
}

/// What a live destination's companion is to `init` (§FS-init.2.3.5.14.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OwnedCompanion {
    /// The name is free: `init` links it to the `AGENTS.md` beside it.
    Free,
    /// A standalone file: `init` splices the block into it, and `check`
    /// compares it.
    Standalone,
}

/// The companions of the live destination `dir` under `root` that `init`
/// owns, each with its name and what it is (§FS-init.2.3.5.14.3): only a
/// mirrored name, and never a symlink, which is left as it is. `init` plans
/// its writes from this and `Expected` its comparisons, so the two agree.
pub(crate) fn owned_companions(root: &Path, dir: &str) -> Vec<(&'static str, OwnedCompanion)> {
    SCOPED_COMPANIONS
        .iter()
        .filter(|name| mirrored(root, name))
        .filter_map(|name| match link_metadata(root.join(dir).join(name)) {
            Err(_) => Some((*name, OwnedCompanion::Free)),
            Ok(meta) if meta.file_type().is_symlink() => None,
            Ok(_) => Some((*name, OwnedCompanion::Standalone)),
        })
        .collect()
}

/// What a file in an orphan's directory is to `init` (§FS-init.2.3.5.14.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OrphanFile {
    /// The orphan's own `AGENTS.md`: its block is cut.
    Agents,
    /// A standalone `CLAUDE.md` or `GEMINI.md`: its block is cut too.
    Standalone,
    /// A symlink to that `AGENTS.md`: deleted when `init` deletes it.
    Linked,
}

/// The files of the orphan directory `dir` under `root` that `init` touches,
/// each with its name and what it is, `AGENTS.md` first (§FS-init.2.3.5.14.3):
/// the `AGENTS.md` and each standalone companion lose their block on their own
/// account, and of the symlinks only one to that `AGENTS.md`, which goes with
/// it. `init` cuts from this and `Expected` compares it, so the two agree.
pub(crate) fn orphan_files(root: &Path, dir: &str) -> Vec<(&'static str, OrphanFile)> {
    let agents = root.join(dir).join(CANONICAL_AGENT_ENTRYPOINT);
    std::iter::once(CANONICAL_AGENT_ENTRYPOINT)
        .chain(SCOPED_COMPANIONS)
        .filter_map(|name| {
            let path = root.join(dir).join(name);
            // §FS-check.6.1.3: an absent one is an input too, which may appear.
            observe_input(&path, false);
            let is_symlink = link_metadata(&path).ok()?.file_type().is_symlink();
            match (name == CANONICAL_AGENT_ENTRYPOINT, is_symlink) {
                (true, false) => Some((name, OrphanFile::Agents)),
                (false, false) => Some((name, OrphanFile::Standalone)),
                (false, true) if is_symlink_to(&path, &agents).unwrap_or(false) => {
                    Some((name, OrphanFile::Linked))
                }
                _ => None,
            }
        })
        .collect()
}

/// The companions beside the scoped `AGENTS.md` in `dir`: each name that
/// exists there.
pub(crate) fn scoped_companions(dir: &Path) -> Vec<PathBuf> {
    SCOPED_COMPANIONS
        .iter()
        .map(|name| dir.join(name))
        .filter(|path| path_is_entry(path))
        .collect()
}
