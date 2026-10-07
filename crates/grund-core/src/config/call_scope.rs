//! Per-call inputs no option carries: embedding's zero-config discovery base
//! (§FS-distribution.3.3.3) and the run's `--path-base` (§FS-cli.3.4).
//!
//! The embedding base: existing options do not carry a fallback cwd. Scope that
//! one discovery input to the synchronous embedding call, instead of changing
//! process cwd or existing Rust/CLI defaults. The guard restores prior scope on both return and unwind.
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

/// The two report bases of `--path-base` (§FS-cli.3.4): the config root the run
/// resolved against, or the CLI base `[output] relative_paths = false` uses.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PathBase {
    Project,
    Invocation,
}

impl PathBase {
    /// The flag's value as written; anything else is the caller's usage error
    /// (§FS-cli.3.5).
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "project" => Some(Self::Project),
            "invocation" => Some(Self::Invocation),
            _ => None,
        }
    }
}

thread_local! {
    static REPORT_PATH_BASE: RefCell<Option<PathBase>> = const { RefCell::new(None) };
}

pub(crate) fn report_path_base() -> Option<PathBase> {
    REPORT_PATH_BASE.with(|base| *base.borrow())
}

/// Scope a run's `--path-base` to one synchronous call (§FS-cli.3.4): every
/// `Config` built inside it carries the override, members and `check --watch`
/// reruns included, and the guard restores the caller's scope on return and
/// unwind. Worker threads read it off the `Config` they were handed.
#[doc(hidden)]
pub fn with_report_path_base<T>(base: Option<PathBase>, operation: impl FnOnce() -> T) -> T {
    struct Restore(Option<PathBase>);
    impl Drop for Restore {
        fn drop(&mut self) {
            REPORT_PATH_BASE.with(|base| *base.borrow_mut() = self.0.take());
        }
    }
    let _restore = Restore(REPORT_PATH_BASE.with(|slot| slot.replace(base)));
    operation()
}
