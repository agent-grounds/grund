//! How many `realpath`s this thread spent (§GOAL-fast-feedback), for the guards that hold a walk
//! to the ones it means to pay (§AR-scanner.1.entry). Test builds only: `canonical_path` and
//! `physical_path_key` each note theirs here, so one counter answers for both, and any other
//! resolution in the crate goes uncounted.

use std::cell::Cell;

thread_local! {
    static REALPATHS: Cell<usize> = const { Cell::new(0) };
}

/// One `realpath` spent, called by `canonical_path` and `physical_path_key`.
pub(super) fn note_realpath() {
    REALPATHS.with(|count| count.set(count.get() + 1));
}

/// What `work` answers, and how many `realpath`s it spent on this thread.
#[cfg(all(test, unix))]
pub(crate) fn realpaths_during<T>(work: impl FnOnce() -> T) -> (T, usize) {
    let before = REALPATHS.with(Cell::get);
    let answer = work();
    (answer, REALPATHS.with(Cell::get) - before)
}
