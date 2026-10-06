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
    let absolute = super::normalize_path_lexically(&absolute);
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
            cursor = super::normalize_path_lexically(&cursor);
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
