//! The native watcher (§AR-lsp.2.1): the `notify` backend `check --watch` runs, and
//! the one an editor server falls back on for the import roots its client cannot
//! watch, with the one debounce both coalesce its events by (§FS-check.6.1.2). The
//! backend is behind the `native-watch` feature, which only grund-cli and grund-lsp
//! enable, so the bindings never link it; the debounce is time arithmetic and needs
//! no feature.

#[cfg(feature = "native-watch")]
pub use notify;

#[cfg(feature = "native-watch")]
use notify::Watcher;
#[cfg(feature = "native-watch")]
use std::path::PathBuf;
use std::time::Duration;

/// The quiet after the last pending event that ends a burst (§FS-check.6.1.2).
const QUIET: Duration = Duration::from_millis(100);
/// The longest a burst waits from its first pending event, so continued writes
/// cannot postpone the work after them indefinitely (§FS-check.6.1.2).
const CEILING: Duration = Duration::from_millis(500);

/// The bounded debounce over native events (§FS-check.6.1.2): a burst is due after
/// 100 ms without an event or 500 ms from its first event, whichever comes
/// first. Times are durations since one instant of the caller's clock, so the policy
/// is independent of how many events the backend delivers, and a test can feed it
/// values.
#[derive(Default)]
pub struct Debounce {
    first: Option<Duration>,
    last: Duration,
}

impl Debounce {
    /// Note a relevant event at `now`.
    pub fn event(&mut self, now: Duration) {
        self.first.get_or_insert(now);
        self.last = now;
    }

    /// How long the pending burst still waits at `now`, zero once it is due, or
    /// `None` with nothing pending.
    pub fn remaining(&self, now: Duration) -> Option<Duration> {
        let first = self.first?;
        let deadline = (first + CEILING).min(self.last + QUIET);
        Some(deadline.saturating_sub(now))
    }
}

/// A recursive watch over each of `roots`, calling `changed` on every event but a
/// read, until it is dropped. A root that cannot be watched is skipped: one that
/// vanished since the scan is the next rebuild's to drop.
#[cfg(feature = "native-watch")]
pub fn watch_roots(
    roots: &[PathBuf],
    changed: impl Fn() + Send + 'static,
) -> notify::Result<notify::RecommendedWatcher> {
    let mut watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
        if !event.is_ok_and(|event| matches!(event.kind, notify::EventKind::Access(_))) {
            changed();
        }
    })?;
    for root in roots {
        let _ = watcher.watch(root, notify::RecursiveMode::Recursive);
    }
    Ok(watcher)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(value: u64) -> Duration {
        Duration::from_millis(value)
    }

    /// §FS-check.6.1.2: events every 10 ms, which never leave 100 ms of quiet, still
    /// make each burst due 500 ms after its first event, and never later, however
    /// long they go on.
    #[test]
    fn a_burst_that_never_quiets_is_due_at_the_ceiling() {
        let mut debounce = Debounce::default();
        let mut first = None;
        let mut due = Vec::new();
        for now in (0..5_000).step_by(10).map(ms) {
            debounce.event(now);
            let started = *first.get_or_insert(now);
            let wait = debounce.remaining(now).expect("pending");
            assert!(
                now + wait <= started + CEILING,
                "due after {:?}",
                now + wait
            );
            if wait.is_zero() {
                due.push((started, now));
                debounce = Debounce::default();
                first = None;
            }
        }
        assert_eq!(due.len(), 9, "{due:?}");
        assert!(
            due.iter().all(|(started, at)| *at == *started + CEILING),
            "{due:?}"
        );
    }

    #[test]
    fn quiet_ends_a_burst_before_the_ceiling() {
        let mut debounce = Debounce::default();
        assert_eq!(debounce.remaining(ms(0)), None);
        debounce.event(ms(0));
        debounce.event(ms(30));
        assert_eq!(debounce.remaining(ms(129)), Some(ms(1)));
        assert_eq!(debounce.remaining(ms(130)), Some(Duration::ZERO));
    }
}
