# RFC-0007-retry-policy: Retry with capped exponential backoff

- Feature Name: `retry_policy`
- Start Date: 2026-03-02
- RFC PR: [skerry/rfcs#7](https://forge.example/skerry/rfcs/pull/7)
- Skerry Issue: [skerry/skerry#41](https://forge.example/skerry/skerry/issues/41)

## summary: Summary
[summary]: #summary-summary

Add `RetryPolicy`, which says how long a caller waits before each retry of a
failed operation and when it stops retrying. The wait doubles from a base delay
up to a cap.

## motivation: Motivation
[motivation]: #motivation-motivation

Every service built on skerry writes its own retry loop today. Most retry
immediately and forever, so an overloaded dependency is hit hardest exactly
when it can least take it. A policy in the crate gives every caller the same
schedule, and gives reviewers one place to read it.

## guide-level-explanation: Guide-level explanation
[guide-level-explanation]: #guide-level-explanation-guide-level-explanation

Build a policy once and ask it for the delay before each retry:

```rust
let policy = RetryPolicy::exponential(Duration::from_millis(100), Duration::from_secs(5), 6);
let mut retry = 0;
let reply = loop {
    match fetch() {
        Ok(reply) => break reply,
        Err(err) => match policy.delay(retry) {
            Some(wait) => { thread::sleep(wait); retry += 1; }
            None => return Err(err),
        },
    }
};
```

The first retry waits 100 ms, the next 200 ms, then 400 ms, until the wait
reaches 5 s and stays there. After six retries `delay` returns `None` and the
caller gives up.

## reference-level-explanation: Reference-level explanation
[reference-level-explanation]: #reference-level-explanation-reference-level-explanation

### reference-level-explanation.schedule: The delay schedule

`RetryPolicy::exponential(base, cap, max_retries)` builds a policy. Retries are
counted from zero, and the delay before retry `n` is `min(base × 2ⁿ, cap)`.

### reference-level-explanation.limit: The retry limit

`delay(n)` returns `None` for every `n` at or above `max_retries`, and `Some`
below it. A policy with `max_retries = 0` never retries.

### reference-level-explanation.overflow: Saturation

`base × 2ⁿ` saturates rather than overflowing, so a large `n` yields the cap,
never a panic or a short delay.

## drawbacks: Drawbacks
[drawbacks]: #drawbacks-drawbacks

Clients that failed at the same moment retry at the same moments, because the
schedule is the same for all of them. A dependency that recovers is then hit by
every client at once.

## rationale-and-alternatives: Rationale and alternatives
[rationale-and-alternatives]: #rationale-and-alternatives-rationale-and-alternatives

- A fixed delay is simpler, but either retries too eagerly while a dependency
  is down or too slowly once it is back.
- A closure that computes each delay is more general, but every caller would
  write the schedule again, which is the problem this RFC removes.
- Not doing this leaves each service with the retry loop it has today.

## prior-art: Prior art
[prior-art]: #prior-art-prior-art

TCP's retransmission timer doubles its timeout on each loss, up to a bound.
gRPC's connection backoff uses the same doubling with a cap.

## unresolved-questions: Unresolved questions
[unresolved-questions]: #unresolved-questions-unresolved-questions

### unresolved-questions.jitter: Randomized delays

Whether the schedule should randomize each delay to spread clients apart. This
RFC leaves it to a follow-up, so that the deterministic schedule can ship first.

## future-possibilities: Future possibilities
[future-possibilities]: #future-possibilities-future-possibilities

A budget shared by every call to one dependency, so that retries stop when too
many of them are failing at once.

## amendments: Amendments
[amendments]: #amendments-amendments

- [§RFC-0012-jittered-backoff](0012-jittered-backoff.md#rfc-0012-jittered-backoff-full-jitter-for-the-retry-schedule)
  answers
  [§RFC-0007-retry-policy.unresolved-questions.jitter](0007-retry-policy.md#unresolved-questionsjitter-randomized-delays):
  a policy may scale each delay of
  [§RFC-0007-retry-policy.reference-level-explanation.schedule](0007-retry-policy.md#reference-level-explanationschedule-the-delay-schedule)
  by a random factor.
