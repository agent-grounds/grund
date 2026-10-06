//! Embedding's zero-config discovery base (§FS-distribution.3.3.3).
//!
//! Existing options do not carry a fallback cwd. Scope that one discovery input
//! to the synchronous embedding call, instead of changing process cwd or existing
//! Rust/CLI defaults. The guard restores prior scope on both return and unwind.
//! Config objects handed to worker threads already contain the resolved root;
//! workspace members use direct load_config_at and do not need this fallback.

use std::cell::RefCell;
use std::path::{Path, PathBuf};

thread_local! {
    static EMBEDDING_BASE: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

pub(crate) fn embedding_base() -> Option<PathBuf> {
    EMBEDDING_BASE.with(|base| base.borrow().clone())
}

pub(crate) fn with_embedding_base<T>(path: &Path, operation: impl FnOnce() -> T) -> T {
    struct Restore(Option<PathBuf>);
    impl Drop for Restore {
        fn drop(&mut self) {
            EMBEDDING_BASE.with(|base| *base.borrow_mut() = self.0.take());
        }
    }
    let base = if path.is_file() {
        path.parent().unwrap_or(path)
    } else {
        path
    };
    let _restore = Restore(EMBEDDING_BASE.with(|slot| slot.replace(Some(base.to_path_buf()))));
    operation()
}
