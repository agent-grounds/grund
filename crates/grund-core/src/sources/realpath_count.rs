//! How many `realpath`s this thread spent (§GOAL-fast-feedback), for the guards that hold a walk
//! to the ones it means to pay (§AR-scanner.1.entry). Test builds only: every function that
//! resolves a path notes it here, so one counter answers for all of them.

use std::cell::Cell;

thread_local! {
    static REALPATHS: Cell<usize> = const { Cell::new(0) };
}

/// One `realpath` spent, called by each function that resolves a path.
pub(super) fn note_realpath() {
    REALPATHS.with(|count| count.set(count.get() + 1));
}

/// What `work` answers, and how many `realpath`s it spent on this thread.
pub(crate) fn realpaths_during<T>(work: impl FnOnce() -> T) -> (T, usize) {
    let before = REALPATHS.with(Cell::get);
    let answer = work();
    (answer, REALPATHS.with(Cell::get) - before)
}
