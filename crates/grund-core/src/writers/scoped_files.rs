//! The scoped files a run owns on disk (§FS-init.2.3.5.13.1, §FS-init.2.3.5.14.3):
//! the list the root block records, the companions beside a scoped `AGENTS.md`,
//! and which root companions a scoped directory mirrors. Read the same way by
//! `init` before it writes and by `Expected` before `check` compares.

use std::path::{Component, Path, PathBuf};

use crate::checker::section_in_block;
use crate::grammar::{AgentsBlockLookup, find_agents_block};
use crate::scanner::{CANONICAL_AGENT_ENTRYPOINT, companion_agent_entrypoints, is_symlink_to};

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
            let Ok(text) = std::fs::read_to_string(&path) else {
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

/// The companions beside the scoped `AGENTS.md` in `dir`: each name that
/// exists there, and whether it is a symlink to that `AGENTS.md`.
pub(crate) fn scoped_companions(dir: &Path) -> Vec<(PathBuf, bool)> {
    let agents = dir.join(CANONICAL_AGENT_ENTRYPOINT);
    SCOPED_COMPANIONS
        .iter()
        .map(|name| dir.join(name))
        .filter(|path| path.symlink_metadata().is_ok())
        .map(|path| {
            let linked = is_symlink_to(&path, &agents).unwrap_or(false);
            (path, linked)
        })
        .collect()
}
