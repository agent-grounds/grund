//! Effective checking inputs, reported before reads (§FS-check.6.1.1,
//! §FS-check.6.1.3). No watcher, process, or stream lives in the engine.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Data-only subscription intent. Exact paths also need replacement anchors;
/// trees include ignored descendants (§AR-bindings.3, §FS-check.6.1.3).
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CheckInput {
    pub path: PathBuf,
    pub recursive: bool,
}

#[doc(hidden)]
pub type CheckInputObserver = Arc<dyn Fn(CheckInput) -> bool + Send + Sync>;

thread_local! {
    static OBSERVER: RefCell<Option<CheckInputObserver>> = const { RefCell::new(None) };
}

/// Scope the inventory to one synchronous check. Rayon receives the same
/// observer explicitly; nested calls restore their caller (§AR-bindings.3).
#[doc(hidden)]
pub fn with_check_input_observer<T>(
    observer: Option<CheckInputObserver>,
    run: impl FnOnce() -> T,
) -> T {
    struct Restore(Option<CheckInputObserver>);
    impl Drop for Restore {
        fn drop(&mut self) {
            OBSERVER.with(|slot| *slot.borrow_mut() = self.0.take());
        }
    }
    let _restore = Restore(OBSERVER.with(|slot| slot.replace(observer)));
    run()
}

pub(crate) fn check_input_observer() -> Option<CheckInputObserver> {
    OBSERVER.with(|slot| slot.borrow().clone())
}

/// Observe lexical paths and followed targets before content reads. Broken
/// links retain their intended target for repair (§FS-check.6.1.3).
pub(crate) fn observe_input(path: &Path, recursive: bool) -> bool {
    let Some(observer) = check_input_observer() else {
        return true;
    };
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(path)
    };
    let absolute = crate::model::normalize_path_lexically(&absolute);
    let mut cursor = absolute;
    let mut seen = std::collections::BTreeSet::new();
    // §FS-check.6.1.3: intermediate links can be replaced independently too.
    while seen.insert(cursor.clone()) {
        if !observer(CheckInput {
            path: cursor.clone(),
            recursive,
        }) {
            return false;
        }
        if let Ok(target) = std::fs::read_link(&cursor) {
            cursor = if target.is_absolute() {
                target
            } else {
                cursor.parent().unwrap_or(Path::new("/")).join(target)
            };
            cursor = crate::model::normalize_path_lexically(&cursor);
            continue;
        }
        if let Ok(target) = std::fs::canonicalize(&cursor) {
            if target != cursor
                && !observer(CheckInput {
                    path: target,
                    recursive,
                })
            {
                return false;
            }
        }
        break;
    }
    true
}

pub(crate) fn input_read_to_string(path: impl AsRef<Path>) -> std::io::Result<String> {
    if !observe_input(path.as_ref(), false) {
        return Err(std::io::Error::other("watch input coverage failed"));
    }
    std::fs::read_to_string(path)
}

pub(crate) fn input_read_dir(path: impl AsRef<Path>) -> std::io::Result<std::fs::ReadDir> {
    if !observe_input(path.as_ref(), true) {
        return Err(std::io::Error::other("watch input coverage failed"));
    }
    std::fs::read_dir(path)
}

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
    for root in super::root_scope_roots(config, config.scan_full) {
        observe_input(&root, true);
    }
    for kind in &config.kinds {
        if let Some(file) = &kind.file {
            observe_input(&config.root.join(file), false);
        }
        if let Some(folder) = &kind.folder {
            observe_input(&config.root.join(folder), true);
        }
        if let Some(index) = kind.index_path() {
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

/// `ignore` owns global-excludes precedence. Cover its discovery inputs before
/// asking that same reader for the effective path (§FS-check.6.1.1).
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
    if let Some(path) = ignore::gitignore::gitconfig_excludes_path() {
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
