//! The retry schedule of §RFC-0007-retry-policy and its jitter, §RFC-0012-jittered-backoff.

use skerry::RetryPolicy;
use std::time::Duration;

fn policy() -> RetryPolicy {
    RetryPolicy::exponential(Duration::from_millis(100), Duration::from_secs(5), 8)
}

/// §RFC-0007-retry-policy.reference-level-explanation.schedule
#[test]
fn doubles_until_the_cap() {
    let delays: Vec<u64> = (0..8)
        .map(|n| policy().delay(n).unwrap().as_millis() as u64)
        .collect();
    assert_eq!(delays, [100, 200, 400, 800, 1600, 3200, 5000, 5000]);
}

/// §RFC-0007-retry-policy.reference-level-explanation.limit
#[test]
fn gives_up_after_the_last_retry() {
    assert_eq!(policy().delay(8), None);
    let never = RetryPolicy::exponential(Duration::from_millis(100), Duration::from_secs(5), 0);
    assert_eq!(never.delay(0), None);
}

/// §RFC-0007-retry-policy.reference-level-explanation.overflow
#[test]
fn saturates_to_the_cap() {
    let long = RetryPolicy::exponential(Duration::from_secs(1), Duration::from_secs(60), u32::MAX);
    assert_eq!(long.delay(200), Some(Duration::from_secs(60)));
}

/// §RFC-0012-jittered-backoff.reference-level-explanation.jitter
#[test]
fn jitter_scales_the_capped_delay() {
    let half = policy().with_jitter(|| 0.5);
    assert_eq!(half.delay(3), Some(Duration::from_millis(400)));
    assert_eq!(half.delay(7), Some(Duration::from_millis(2500)));
}
