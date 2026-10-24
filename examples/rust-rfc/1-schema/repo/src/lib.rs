//! Retries with capped exponential backoff, as §RFC-0007-retry-policy specifies,
//! and optional full jitter, as §RFC-0012-jittered-backoff specifies.

use std::time::Duration;

/// How long to wait before each retry of a failed operation, and when to stop.
pub struct RetryPolicy {
    base: Duration,
    cap: Duration,
    max_retries: u32,
    jitter: Option<fn() -> f64>,
}

impl RetryPolicy {
    /// A policy whose delay doubles from `base` up to `cap`, for at most
    /// `max_retries` retries (§RFC-0007-retry-policy.reference-level-explanation.schedule).
    pub fn exponential(base: Duration, cap: Duration, max_retries: u32) -> Self {
        RetryPolicy {
            base,
            cap,
            max_retries,
            jitter: None,
        }
    }

    /// Scales each delay by a value `source` draws from `[0, 1)`
    /// (§RFC-0012-jittered-backoff.reference-level-explanation.source).
    pub fn with_jitter(mut self, source: fn() -> f64) -> Self {
        self.jitter = Some(source);
        self
    }

    /// The delay before retry `retry`, counted from zero, or `None` once the
    /// policy gives up.
    pub fn delay(&self, retry: u32) -> Option<Duration> {
        // §RFC-0007-retry-policy.reference-level-explanation.limit
        if retry >= self.max_retries {
            return None;
        }
        // §RFC-0007-retry-policy.reference-level-explanation.overflow
        let factor = 2u32.checked_pow(retry).unwrap_or(u32::MAX);
        let capped = self.base.saturating_mul(factor).min(self.cap);
        match self.jitter {
            // §RFC-0012-jittered-backoff.reference-level-explanation.jitter
            Some(source) => Some(capped.mul_f64(source())),
            None => Some(capped),
        }
    }
}
