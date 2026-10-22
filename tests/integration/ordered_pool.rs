//! The one pool the corpus sweeps share (§REQ-one-answer.proof.delivery.payment):
//! the items spread over every worker, results gathered in item order, so a sweep
//! that overlaps its fixtures prints and asserts exactly what its serial form
//! did (§REQ-deterministic-output.1). The scan-determinism and lsp/cli parity
//! targets include it by `#[path]`, and so does the e2e target.

use std::sync::atomic::{AtomicUsize, Ordering};

/// Every CPU libtest may use: the `<w>` a sweep's `gate time:` line names
/// (§AR-ci.requirements.5.1).
pub fn workers() -> usize {
    std::thread::available_parallelism().map_or(1, |n| n.get())
}

/// `work` over every item on every CPU libtest may use, results in item order
/// (§REQ-one-answer.proof.delivery.payment): one item's work stays on one
/// worker, only different items overlap, and a worker's panic is resumed on
/// the caller (§REQ-deterministic-output.1).
pub fn in_order<T: Sync, R: Send>(items: &[T], work: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let next = AtomicUsize::new(0);
    let mut results: Vec<Option<R>> = items.iter().map(|_| None).collect();
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..workers().clamp(1, items.len().max(1)))
            .map(|_| {
                scope.spawn(|| {
                    let mut done = Vec::new();
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some(item) = items.get(index) else {
                            return done;
                        };
                        done.push((index, work(item)));
                    }
                })
            })
            .collect();
        for handle in handles {
            match handle.join() {
                Ok(done) => {
                    for (index, result) in done {
                        results[index] = Some(result);
                    }
                }
                Err(panic) => std::panic::resume_unwind(panic),
            }
        }
    });
    results
        .into_iter()
        .map(|result| result.expect("every item ran"))
        .collect()
}
