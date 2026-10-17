//! Config-dependent checking inputs (§FS-check.6.1.1, §FS-check.6.1.3).

use crate::sources::{check_input_observer, input_read_to_string, observe_input};
use std::path::{Path, PathBuf};

/// Subscribe to both discovery names even if absent or shadowed. The frontend
/// supplies shallow, filtered parent anchors (§FS-check.6.1.3).
pub(crate) fn observe_config_candidates(dir: &Path) {
    observe_input(&dir.join("grund.toml"), false);
    observe_input(&dir.join(".agents/grund.toml"), false);
}

/// Inventory from the actual resolved config, before expansion or checking.
/// Source roots still come from the shared root selector (§FS-check.6.1.3).
pub(crate) fn observe_config(config: &super::Config) {
    if check_input_observer().is_none() {
        return;
    }
    observe_config_candidates(&config.root);
    for root in super::root_scope_roots(config.schema(), &config.root, config.scan_full) {
        observe_input(&root, true);
    }
    // §FS-config-v2.schema.places: every place of every row, and each folder's index.
    for row in &config.schema().rows {
        for file in row.files() {
            observe_input(&config.root.join(file), false);
        }
        for folder in row.folders() {
            observe_input(&config.root.join(folder), true);
        }
        for (_, index) in row.index_paths() {
            observe_input(&config.root.join(index), false);
        }
    }
    for agent in ["AGENTS.md", "CLAUDE.md", "GEMINI.md", ".agents/AGENTS.md"] {
        observe_input(&config.root.join(agent), false);
    }
    for member in config
        .workspace_members
        .iter()
        .chain(&config.workspace_optional_members)
    {
        observe_input(
            &config
                .root
                .join(member.strip_suffix("/*").unwrap_or(member)),
            true,
        );
    }
}

/// Cover global-excludes discovery inputs before asking the shared pathname
/// resolver (§AR-scanner.1.1) for the effective path
/// (§FS-check.6.1.1).
pub(crate) fn observe_ignore_inputs(root: &Path) -> bool {
    if check_input_observer().is_none() {
        return true;
    }
    for dir in root.ancestors() {
        for name in [".gitignore", ".ignore", ".git", ".git/info/exclude"] {
            if !observe_input(&dir.join(name), false) {
                return false;
            }
        }
        if !observe_git_worktree(dir) {
            return false;
        }
    }
    // §FS-check.6.1.1: ignore can follow nested worktree indirections outside
    // the source tree. Subscribe before constructing its lazy reader; this
    // metadata-only pass never reads source content or follows directory links.
    let mut directories = vec![root.to_path_buf()];
    while let Some(dir) = directories.pop() {
        if !observe_git_worktree(&dir) {
            return false;
        }
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if entry.file_type().is_ok_and(|kind| kind.is_dir()) && entry.file_name() != ".git"
                {
                    directories.push(entry.path());
                }
            }
        }
    }
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from);
    if let Some(home) = &home {
        if !observe_input(&home.join(".gitconfig"), false) {
            return false;
        }
    }
    let xdg = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| home.map(|p| p.join(".config")));
    if let Some(xdg) = xdg {
        if !observe_input(&xdg.join("git/config"), false)
            || !observe_input(&xdg.join("git/ignore"), false)
        {
            return false;
        }
    }
    let Ok(path) = super::global_excludes_path() else {
        return false;
    };
    if let Some(path) = path {
        if !observe_input(&path, false) {
            return false;
        }
    }
    true
}

/// Observe paths named by ignore's worktree indirections without changing its
/// matcher or precedence (§FS-check.6.1.3). Observe even absent commondir probes.
fn observe_git_worktree(dir: &Path) -> bool {
    let dotgit = dir.join(".git");
    if !observe_input(&dotgit, false) {
        return false;
    }
    if !dotgit.is_file() {
        return true;
    }
    let Ok(text) = input_read_to_string(&dotgit) else {
        return true;
    };
    let Some(target) = text
        .lines()
        .next()
        .and_then(|line| line.strip_prefix("gitdir: "))
    else {
        return true;
    };
    let target = PathBuf::from(target);
    let common = target.join("commondir");
    if !observe_input(&common, false) {
        return false;
    }
    let Ok(text) = input_read_to_string(&common) else {
        return true;
    };
    if let Some(line) = text.lines().next() {
        let path = if line.starts_with('.') {
            target.join(line)
        } else {
            PathBuf::from(line)
        };
        return observe_input(&path.join("info/exclude"), false);
    }
    true
}
