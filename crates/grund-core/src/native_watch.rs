//! The native watcher (§AR-lsp.2.1): the `notify` backend `check --watch` runs, and
//! the one an editor server falls back on for the import roots its client cannot
//! watch. Behind the `native-watch` feature, which only grund-cli and grund-lsp
//! enable, so the bindings never link it.

pub use notify;

use notify::Watcher;
use std::path::PathBuf;

/// A recursive watch over each of `roots`, calling `changed` on every event but a
/// read, until it is dropped. A root that cannot be watched is skipped: one that
/// vanished since the scan is the next rebuild's to drop.
pub fn watch_roots(
    roots: &[PathBuf],
    changed: impl Fn() + Send + 'static,
) -> notify::Result<notify::RecommendedWatcher> {
    let mut watcher = notify::RecommendedWatcher::new(
        move |event: notify::Result<notify::Event>| {
            if !event.is_ok_and(|event| matches!(event.kind, notify::EventKind::Access(_))) {
                changed();
            }
        },
        notify::Config::default(),
    )?;
    for root in roots {
        let _ = watcher.watch(root, notify::RecursiveMode::Recursive);
    }
    Ok(watcher)
}
